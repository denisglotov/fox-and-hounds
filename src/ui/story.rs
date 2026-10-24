//! In-engine animated story video playback for Fox and Hounds.
//!
//! Plays the 8-second animated artwork story seamlessly in-place over the hero banner
//! on the title screen without popups, dialogs, or external players across all platforms
//! (Android, WebAssembly, macOS, Windows, Linux).

use crate::audio::SoundTrigger;
use macroquad::prelude::*;

const STORY_MAGIC: &[u8; 8] = b"FAHVID01";

/// In-place animated hero banner player.
pub struct LiveHeroBanner {
    data: &'static [u8],
    offsets: Vec<(usize, usize)>,
    fps: f32,
    duration: f32,
    texture: Option<Texture2D>,
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
        let table_size = frame_count * 8;
        if bytes.len() < table_start + table_size {
            return None;
        }

        let offsets: Option<Vec<(usize, usize)>> = (0..frame_count)
            .map(|i| {
                let entry = table_start + i * 8;
                let off = u32::from_le_bytes(bytes[entry..entry + 4].try_into().ok()?) as usize;
                let len = u32::from_le_bytes(bytes[entry + 4..entry + 8].try_into().ok()?) as usize;
                if off + len <= bytes.len() {
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

        Some(Self {
            data: bytes,
            offsets,
            fps,
            duration,
            texture: None,
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
        if target_frame != self.current_frame {
            self.current_frame = target_frame;
            self.update_frame(target_frame);
        }

        None
    }

    fn update_frame(&mut self, idx: usize) {
        if let Some(&(off, len)) = self.offsets.get(idx) {
            if let Ok(img) =
                Image::from_file_with_format(&self.data[off..off + len], Some(ImageFormat::Jpeg))
            {
                if let Some(tex) = &self.texture {
                    tex.update(&img);
                } else {
                    let tex = Texture2D::from_image(&img);
                    tex.set_filter(FilterMode::Linear);
                    self.texture = Some(tex);
                }
            }
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
            &banner.data[off..off + len],
            Some(macroquad::prelude::ImageFormat::Jpeg),
        )
        .expect("Frame 0 must decode");
        assert_eq!(img.width, 480);
        assert_eq!(img.height, 270);
    }
}
