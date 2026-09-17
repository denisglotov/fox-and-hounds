use crate::audio::SoundTrigger;
use macroquad::prelude::*;

pub const ROVER_X: f32 = 512.0;
pub const ROVER_START_Y: f32 = -60.0;
pub const ROVER_CRATER_Y: f32 = 140.0;

pub const INITIAL_DELAY: f32 = 15.0;
pub const CYCLE_DURATION: f32 = 60.0;

pub const FORWARD_DURATION: f32 = 6.0;
pub const STOP_DURATION: f32 = 2.0;
pub const REVERSE_DURATION: f32 = 6.0;
pub const TOTAL_ACTIVE_DURATION: f32 = FORWARD_DURATION + STOP_DURATION + REVERSE_DURATION;

pub const ROVER_WIDTH: f32 = 50.0;
pub const ROVER_HEIGHT: f32 = 62.9;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoverPhase {
    Forward(f32),
    StoppedAtCrater,
    Reverse(f32),
    Inactive,
}

#[derive(Debug, Clone)]
pub struct RoverSimulation {
    pub elapsed_time: f32,
    pub last_sound_cycle: i32,
    pub last_beep_time: f32,
}

impl Default for RoverSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl RoverSimulation {
    pub fn new() -> Self {
        Self {
            elapsed_time: 0.0,
            last_sound_cycle: -1,
            last_beep_time: -1.0,
        }
    }

    pub fn phase(&self) -> RoverPhase {
        if self.elapsed_time < INITIAL_DELAY {
            return RoverPhase::Inactive;
        }

        let t_in_cycle = (self.elapsed_time - INITIAL_DELAY) % CYCLE_DURATION;
        if t_in_cycle < FORWARD_DURATION {
            RoverPhase::Forward(t_in_cycle / FORWARD_DURATION)
        } else if t_in_cycle < FORWARD_DURATION + STOP_DURATION {
            RoverPhase::StoppedAtCrater
        } else if t_in_cycle < TOTAL_ACTIVE_DURATION {
            let rev_t = t_in_cycle - FORWARD_DURATION - STOP_DURATION;
            RoverPhase::Reverse(rev_t / REVERSE_DURATION)
        } else {
            RoverPhase::Inactive
        }
    }

    pub fn is_active(&self) -> bool {
        self.phase() != RoverPhase::Inactive
    }

    pub fn is_near_crater(&self) -> bool {
        matches!(self.phase(), RoverPhase::StoppedAtCrater)
    }

    pub fn rover_pos(&self) -> Option<Vec2> {
        let y = match self.phase() {
            RoverPhase::Forward(progress) => {
                let ease = progress.clamp(0.0, 1.0);
                ROVER_START_Y + ease * (ROVER_CRATER_Y - ROVER_START_Y)
            }
            RoverPhase::StoppedAtCrater => ROVER_CRATER_Y,
            RoverPhase::Reverse(progress) => {
                let ease = progress.clamp(0.0, 1.0);
                ROVER_CRATER_Y + ease * (ROVER_START_Y - ROVER_CRATER_Y)
            }
            RoverPhase::Inactive => return None,
        };

        Some(Vec2::new(ROVER_X, y))
    }

    pub fn update(&mut self, dt: f32) -> Option<SoundTrigger> {
        self.elapsed_time += dt;

        if self.elapsed_time < INITIAL_DELAY {
            return None;
        }

        let t_in_cycle = (self.elapsed_time - INITIAL_DELAY) % CYCLE_DURATION;
        let cycle_idx = ((self.elapsed_time - INITIAL_DELAY) / CYCLE_DURATION).floor() as i32;

        if self.last_sound_cycle != cycle_idx {
            self.last_sound_cycle = cycle_idx;
            self.last_beep_time = -1.0;
        }

        // Rhythmic forward beep-beep (approx every 1.0s)
        if t_in_cycle < FORWARD_DURATION {
            let beep_step = (t_in_cycle / 1.0).floor() as i32;
            let expected_beep_time = beep_step as f32 * 1.0;
            if self.last_beep_time < expected_beep_time {
                self.last_beep_time = expected_beep_time;
                return Some(SoundTrigger::RoverForward);
            }
        }
        // Continuous rhythmic reverse backup beeping (every 0.8s)
        else if (FORWARD_DURATION + STOP_DURATION..TOTAL_ACTIVE_DURATION).contains(&t_in_cycle) {
            let rev_t = t_in_cycle - (FORWARD_DURATION + STOP_DURATION);
            let beep_step = (rev_t / 0.8).floor() as i32;
            let expected_beep_time = 100.0 + (beep_step as f32 * 0.8);
            if self.last_beep_time < expected_beep_time {
                self.last_beep_time = expected_beep_time;
                return Some(SoundTrigger::RoverReverse);
            }
        }

        None
    }

    pub fn draw(&self, origin: Vec2, scale: f32, texture: Option<&Texture2D>) {
        let Some(pos) = self.rover_pos() else {
            return;
        };

        let t = self.elapsed_time;
        let is_moving = matches!(
            self.phase(),
            RoverPhase::Forward(_) | RoverPhase::Reverse(_)
        );

        // Subtle mechanical vibration while slowly rolling
        let sway = if is_moving {
            (t * 8.0).sin() * 0.25
        } else {
            0.0
        };

        let draw_x = pos.x + sway;
        let draw_y = pos.y;

        let screen_x = origin.x + draw_x * scale;
        let screen_y = origin.y + draw_y * scale;
        let draw_w = ROVER_WIDTH * scale;
        let draw_h = ROVER_HEIGHT * scale;

        // 1. Draw Rover Tire Tracks on Martian Soil
        let track_top_y = (ROVER_START_Y + 25.0).max(0.0);
        let track_cur_y = draw_y;
        if track_cur_y > track_top_y {
            let left_wheel_x = screen_x - 18.0 * scale;
            let right_wheel_x = screen_x + 18.0 * scale;
            let track_len = (track_cur_y - track_top_y) * scale;
            let track_top_screen_y = origin.y + track_top_y * scale;

            let track_col = Color::from_rgba(110, 45, 25, 45);
            let track_w = 6.0 * scale;

            draw_rectangle(
                left_wheel_x - track_w / 2.0,
                track_top_screen_y,
                track_w,
                track_len,
                track_col,
            );
            draw_rectangle(
                right_wheel_x - track_w / 2.0,
                track_top_screen_y,
                track_w,
                track_len,
                track_col,
            );
        }

        // 2. Draw Rover Texture
        if let Some(tex) = texture {
            draw_texture_ex(
                tex,
                screen_x - draw_w / 2.0,
                screen_y - draw_h / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(draw_w, draw_h)),
                    ..Default::default()
                },
            );
        } else {
            // Fallback placeholder
            draw_rectangle(
                screen_x - draw_w / 2.0,
                screen_y - draw_h / 2.0,
                draw_w,
                draw_h,
                Color::from_rgba(220, 220, 225, 255),
            );
        }

        // 4. Subtle front sensor LED / hazard glow
        let nose_y = screen_y + draw_h * 0.42;
        let sensor_col = if is_moving {
            Color::from_rgba(120, 220, 255, 180)
        } else {
            Color::from_rgba(255, 180, 80, 220)
        };
        draw_circle(screen_x - 6.0 * scale, nose_y, 1.5 * scale, sensor_col);
        draw_circle(screen_x + 6.0 * scale, nose_y, 1.5 * scale, sensor_col);
    }
}
