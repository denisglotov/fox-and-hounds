//! In-engine animated story video playback for Fox and Hounds.
//!
//! Plays the 8-second animated artwork story seamlessly in-place over the hero banner
//! on the title screen without popups, dialogs, or external players across all platforms
//! (Android, WebAssembly, macOS, Windows, Linux).

use crate::audio::SoundTrigger;
use macroquad::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};

const STORY_MAGIC: &[u8; 8] = b"FAHVID01";

/// A decoded frame is deliberately kept on the CPU until it is due for upload.  This lets the
/// native decoder work one frame ahead without allocating or decoding from the render loop.
struct DecodedFrame {
    #[cfg(not(target_arch = "wasm32"))]
    index: usize,
    image: Image,
}

/// In-place animated hero banner player.
pub struct LiveHeroBanner {
    #[cfg(target_arch = "wasm32")]
    data: &'static [u8],
    offsets: Vec<(usize, usize)>,
    fps: f32,
    duration: f32,
    texture: Option<Texture2D>,
    #[cfg(not(target_arch = "wasm32"))]
    frame_requests: SyncSender<usize>,
    #[cfg(not(target_arch = "wasm32"))]
    decoded_frames: Receiver<DecodedFrame>,
    #[cfg(not(target_arch = "wasm32"))]
    requested_frame: Option<usize>,
    #[cfg(not(target_arch = "wasm32"))]
    prefetched_frame: Option<DecodedFrame>,
    is_playing: bool,
    start_time: f64,
    current_frame: usize,
}

impl LiveHeroBanner {
    pub fn new(bytes: &'static [u8]) -> Option<Self> {
        if bytes.len() < 24 || &bytes[0..8] != STORY_MAGIC {
            return None;
        }

        let frame_count = u32::from_le_bytes(bytes[8..12].try_into().ok()?) as usize;
        let fps = u32::from_le_bytes(bytes[12..16].try_into().ok()?) as f32;
        let table_start = 24;
        let table_size = frame_count.checked_mul(8)?;
        if table_size
            .checked_add(table_start)
            .is_none_or(|end| bytes.len() < end)
        {
            return None;
        }

        let offsets: Option<Vec<(usize, usize)>> = (0..frame_count)
            .map(|i| {
                let entry = table_start + i * 8;
                let off = u32::from_le_bytes(bytes[entry..entry + 4].try_into().ok()?) as usize;
                let len = u32::from_le_bytes(bytes[entry + 4..entry + 8].try_into().ok()?) as usize;
                if off.checked_add(len).is_some_and(|end| end <= bytes.len()) {
                    Some((off, len))
                } else {
                    None
                }
            })
            .collect();

        let offsets = offsets?;
        let duration = if fps > 0.0 {
            frame_count as f32 / fps
        } else {
            0.0
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (frame_requests, decoded_frames) = Self::spawn_frame_decoder(bytes, offsets.clone());
        #[cfg(not(target_arch = "wasm32"))]
        let requested_frame = frame_requests.try_send(0).ok().map(|()| 0);

        Some(Self {
            #[cfg(target_arch = "wasm32")]
            data: bytes,
            offsets,
            fps,
            duration,
            texture: None,
            #[cfg(not(target_arch = "wasm32"))]
            frame_requests,
            #[cfg(not(target_arch = "wasm32"))]
            decoded_frames,
            #[cfg(not(target_arch = "wasm32"))]
            requested_frame,
            #[cfg(not(target_arch = "wasm32"))]
            prefetched_frame: None,
            is_playing: false,
            start_time: 0.0,
            current_frame: 0,
        })
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn texture(&self) -> Option<&Texture2D> {
        if self.is_playing {
            self.texture.as_ref()
        } else {
            None
        }
    }

    pub fn toggle(&mut self) -> Option<SoundTrigger> {
        if self.is_playing {
            self.stop();
            Some(SoundTrigger::StopStory)
        } else {
            self.start();
            Some(SoundTrigger::Story)
        }
    }

    pub fn start(&mut self) {
        self.is_playing = true;
        self.start_time = get_time();
        self.current_frame = 0;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.prefetched_frame = None;
            self.request_frame(0);
        }
        #[cfg(target_arch = "wasm32")]
        self.update_frame(0);
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.start_time = 0.0;
        self.current_frame = 0;
    }

    pub fn update(&mut self) -> Option<SoundTrigger> {
        if !self.is_playing {
            return None;
        }

        let elapsed = (get_time() - self.start_time).max(0.0) as f32;
        if elapsed >= self.duration {
            self.stop();
            return Some(SoundTrigger::StopStory);
        }

        let target_frame =
            ((elapsed * self.fps) as usize).min(self.offsets.len().saturating_sub(1));
        #[cfg(not(target_arch = "wasm32"))]
        self.update_prefetched_frame(target_frame);
        #[cfg(target_arch = "wasm32")]
        if target_frame != self.current_frame {
            self.current_frame = target_frame;
            self.update_frame(target_frame);
        }

        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_frame_decoder(
        data: &'static [u8],
        offsets: Vec<(usize, usize)>,
    ) -> (SyncSender<usize>, Receiver<DecodedFrame>) {
        let (request_tx, request_rx) = mpsc::sync_channel(1);
        let (frame_tx, frame_rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            while let Ok(index) = request_rx.recv() {
                let frame = Self::decode_frame(data, &offsets, index);
                if frame.is_some_and(|frame| frame_tx.send(frame).is_err()) {
                    break;
                }
            }
        });
        (request_tx, frame_rx)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn request_frame(&mut self, index: usize) {
        if self.requested_frame == Some(index)
            || self
                .prefetched_frame
                .as_ref()
                .is_some_and(|frame| frame.index == index)
        {
            return;
        }

        match self.frame_requests.try_send(index) {
            Ok(()) => self.requested_frame = Some(index),
            Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => self.requested_frame = None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_prefetched_frame(&mut self, target_frame: usize) {
        while let Ok(frame) = self.decoded_frames.try_recv() {
            self.requested_frame = None;
            if frame.index == self.current_frame {
                self.upload_frame(&frame.image);
            } else if frame.index == self.current_frame.saturating_add(1) {
                self.prefetched_frame = Some(frame);
            }
        }

        if target_frame == self.current_frame {
            self.request_frame(self.current_frame.saturating_add(1));
            return;
        }

        if self
            .prefetched_frame
            .as_ref()
            .is_some_and(|frame| frame.index == target_frame)
        {
            let frame = self.prefetched_frame.take().expect("checked above");
            self.current_frame = frame.index;
            self.upload_frame(&frame.image);
            self.request_frame(self.current_frame.saturating_add(1));
        } else {
            // If a frame was dropped (for example while the app was suspended), wait for the
            // decoder rather than doing JPEG work on the render thread.
            self.request_frame(target_frame);
        }
    }

    fn decode_frame(data: &[u8], offsets: &[(usize, usize)], index: usize) -> Option<DecodedFrame> {
        let &(off, len) = offsets.get(index)?;
        Image::from_file_with_format(&data[off..off + len], Some(ImageFormat::Jpeg))
            .ok()
            .map(|image| DecodedFrame {
                #[cfg(not(target_arch = "wasm32"))]
                index,
                image,
            })
    }

    fn upload_frame(&mut self, image: &Image) {
        if let Some(tex) = &self.texture {
            tex.update(image);
        } else {
            let tex = Texture2D::from_image(image);
            tex.set_filter(FilterMode::Linear);
            self.texture = Some(tex);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn update_frame(&mut self, idx: usize) {
        if let Some(frame) = Self::decode_frame(self.data, &self.offsets, idx) {
            self.upload_frame(&frame.image);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL_STORY_BYTES: &[u8] = include_bytes!("../../assets/fox_and_hounds_story.bin");

    #[test]
    fn test_live_hero_banner_parse_and_decode() {
        assert!(LiveHeroBanner::new(&[]).is_none());
        assert!(LiveHeroBanner::new(b"SHORT").is_none());

        let banner = LiveHeroBanner::new(REAL_STORY_BYTES).expect("Must parse asset");
        assert_eq!(banner.fps, 18.0);
        assert!(banner.duration >= 7.5 && banner.duration <= 8.5);
        assert_eq!(banner.offsets.len(), 146);

        let (off, len) = banner.offsets[0];
        let img = Image::from_file_with_format(
            &REAL_STORY_BYTES[off..off + len],
            Some(macroquad::prelude::ImageFormat::Jpeg),
        )
        .expect("Frame 0 must decode");
        assert_eq!(img.width, 480);
        assert_eq!(img.height, 270);
    }
}
