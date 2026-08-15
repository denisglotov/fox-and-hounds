#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundTrigger {
    Move,
    Select,
    InvalidMove,
    Win,
    Loss,
    ButtonClick,
}

#[cfg(target_arch = "wasm32")]
mod wasm_backend {
    use super::SoundTrigger;

    #[link(wasm_import_module = "env")]
    extern "C" {
        fn play_sound_move();
        fn play_sound_select();
        fn play_sound_invalid();
        fn play_sound_win();
        fn play_sound_loss();
        fn play_sound_click();
    }

    pub struct SoundBackend;

    impl SoundBackend {
        pub async fn new() -> Self {
            Self
        }

        pub fn play(&self, trigger: SoundTrigger) {
            unsafe {
                match trigger {
                    SoundTrigger::Move => play_sound_move(),
                    SoundTrigger::Select => play_sound_select(),
                    SoundTrigger::InvalidMove => play_sound_invalid(),
                    SoundTrigger::Win => play_sound_win(),
                    SoundTrigger::Loss => play_sound_loss(),
                    SoundTrigger::ButtonClick => play_sound_click(),
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native_backend {
    use super::SoundTrigger;
    use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};

    pub struct SoundBackend {
        snd_move: Option<Sound>,
        snd_select: Option<Sound>,
        snd_invalid: Option<Sound>,
        snd_win: Option<Sound>,
        snd_loss: Option<Sound>,
        snd_click: Option<Sound>,
    }

    impl SoundBackend {
        pub async fn new() -> Self {
            Self {
                snd_move: load_sound_from_bytes(include_bytes!("../assets/move.wav"))
                    .await
                    .ok(),
                snd_select: load_sound_from_bytes(include_bytes!("../assets/select.wav"))
                    .await
                    .ok(),
                snd_invalid: load_sound_from_bytes(include_bytes!("../assets/invalid.wav"))
                    .await
                    .ok(),
                snd_win: load_sound_from_bytes(include_bytes!("../assets/win.wav"))
                    .await
                    .ok(),
                snd_loss: load_sound_from_bytes(include_bytes!("../assets/loss.wav"))
                    .await
                    .ok(),
                snd_click: load_sound_from_bytes(include_bytes!("../assets/click.wav"))
                    .await
                    .ok(),
            }
        }

        pub fn play(&self, trigger: SoundTrigger) {
            let sound = match trigger {
                SoundTrigger::Move => &self.snd_move,
                SoundTrigger::Select => &self.snd_select,
                SoundTrigger::InvalidMove => &self.snd_invalid,
                SoundTrigger::Win => &self.snd_win,
                SoundTrigger::Loss => &self.snd_loss,
                SoundTrigger::ButtonClick => &self.snd_click,
            };

            if let Some(snd) = sound {
                play_sound_once(snd);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_backend::SoundBackend;

#[cfg(not(target_arch = "wasm32"))]
pub use native_backend::SoundBackend;

pub struct SoundManager {
    backend: SoundBackend,
    muted: bool,
}

impl SoundManager {
    pub async fn new() -> Self {
        Self {
            backend: SoundBackend::new().await,
            muted: false,
        }
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn play(&self, trigger: SoundTrigger) {
        if !self.muted {
            self.backend.play(trigger);
        }
    }
}
