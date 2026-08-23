use crate::audio::SoundTrigger;
use crate::game::level::BOARD_IMAGE_HEIGHT;
use macroquad::prelude::*;

pub const TRACK_X: f32 = 40.0;
pub const TRAIN_START_Y: f32 = -270.0;
pub const TRAIN_END_Y: f32 = 1620.0;
pub const INITIAL_DELAY: f32 = 15.0;
pub const CYCLE_DURATION: f32 = 60.0;
pub const TRANSIT_DURATION: f32 = 5.2;

pub const TRAIN_WIDTH: f32 = 81.0;
pub const TRAIN_HEIGHT: f32 = 486.0;

#[derive(Debug, Clone)]
pub struct TrainSimulation {
    pub elapsed_time: f32,
    pub last_sound_cycle: i32,
}

impl Default for TrainSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl TrainSimulation {
    pub fn new() -> Self {
        Self {
            elapsed_time: 0.0,
            last_sound_cycle: -1,
        }
    }

    pub fn cycle_progress(&self) -> f32 {
        if self.elapsed_time < INITIAL_DELAY {
            self.elapsed_time / INITIAL_DELAY
        } else {
            ((self.elapsed_time - INITIAL_DELAY) % CYCLE_DURATION) / CYCLE_DURATION
        }
    }

    pub fn is_active(&self) -> bool {
        if self.elapsed_time < INITIAL_DELAY {
            false
        } else {
            let t_in_cycle = (self.elapsed_time - INITIAL_DELAY) % CYCLE_DURATION;
            t_in_cycle < TRANSIT_DURATION
        }
    }

    pub fn train_progress(&self) -> Option<f32> {
        if self.elapsed_time < INITIAL_DELAY {
            None
        } else {
            let t_in_cycle = (self.elapsed_time - INITIAL_DELAY) % CYCLE_DURATION;
            if t_in_cycle < TRANSIT_DURATION {
                Some((t_in_cycle / TRANSIT_DURATION).clamp(0.0, 1.0))
            } else {
                None
            }
        }
    }

    pub fn train_locomotive_y(&self) -> Option<f32> {
        self.train_progress()
            .map(|p| TRAIN_START_Y + p * (TRAIN_END_Y - TRAIN_START_Y))
    }

    pub fn update(&mut self, dt: f32) -> Option<SoundTrigger> {
        self.elapsed_time += dt;

        if self.elapsed_time >= INITIAL_DELAY {
            let cycle_idx = ((self.elapsed_time - INITIAL_DELAY) / CYCLE_DURATION).floor() as i32;
            let t_in_cycle = (self.elapsed_time - INITIAL_DELAY) % CYCLE_DURATION;
            if t_in_cycle < TRANSIT_DURATION && self.last_sound_cycle != cycle_idx {
                self.last_sound_cycle = cycle_idx;
                return Some(SoundTrigger::Train);
            }
        }

        None
    }

    pub fn draw(&self, origin: Vec2, scale: f32, texture: Option<&Texture2D>) {
        let loco_y = match self.train_locomotive_y() {
            Some(y) => y,
            None => return,
        };

        let t = self.elapsed_time;
        let sway = (t * 14.0).sin() * 0.7; // subtle rail track vibration
        let train_x = TRACK_X + sway;

        let top_y = loco_y - TRAIN_HEIGHT / 2.0;
        let bot_y = loco_y + TRAIN_HEIGHT / 2.0;

        // Clip strictly within the board image Y bounds [0.0, BOARD_IMAGE_HEIGHT]
        let vis_top = top_y.max(0.0);
        let vis_bot = bot_y.min(BOARD_IMAGE_HEIGHT);

        if vis_top >= vis_bot {
            return;
        }

        // 1. Draw Forward LED Headlight Beam illuminating the rails ahead of the train
        let nose_board_y = bot_y - 24.0;
        if (0.0..BOARD_IMAGE_HEIGHT).contains(&nose_board_y) {
            let nose_y = origin.y + nose_board_y * scale;
            let beam_end_board_y = (nose_board_y + 180.0).min(BOARD_IMAGE_HEIGHT);
            let beam_len = (beam_end_board_y - nose_board_y) * scale;

            if beam_len > 0.0 {
                let train_screen_x = origin.x + train_x * scale;
                let start_half_w = 20.0 * scale;
                let end_half_w =
                    (start_half_w + (68.0 - 20.0) * (beam_len / (180.0 * scale))).min(68.0 * scale);

                let v0 = Vec2::new(train_screen_x - start_half_w, nose_y);
                let v1 = Vec2::new(train_screen_x + start_half_w, nose_y);
                let v2 = Vec2::new(train_screen_x + end_half_w, nose_y + beam_len);
                let v3 = Vec2::new(train_screen_x - end_half_w, nose_y + beam_len);

                let light_top = Color::from_rgba(255, 245, 180, 80);
                let light_bot = Color::from_rgba(255, 235, 120, 0);

                draw_triangle(v0, v1, v2, light_top);
                draw_triangle(v0, v2, v3, light_bot);

                // Core bright spotlight at the front LED headlights
                draw_circle(
                    train_screen_x - 10.0 * scale,
                    nose_y + 6.0 * scale,
                    7.0 * scale,
                    Color::from_rgba(255, 255, 220, 255),
                );
                draw_circle(
                    train_screen_x + 10.0 * scale,
                    nose_y + 6.0 * scale,
                    7.0 * scale,
                    Color::from_rgba(255, 255, 220, 255),
                );
                draw_circle(
                    train_screen_x,
                    nose_y + 6.0 * scale,
                    18.0 * scale,
                    Color::from_rgba(255, 245, 160, 110),
                );
            }
        }

        // 2. Draw Pre-baked Train Texture Asset with sub-rectangle clipping
        if let Some(tex) = texture {
            let tex_w = tex.width();
            let tex_h = tex.height();

            let norm_top = ((vis_top - top_y) / TRAIN_HEIGHT).clamp(0.0, 1.0);
            let norm_bot = ((vis_bot - top_y) / TRAIN_HEIGHT).clamp(0.0, 1.0);

            let src_y = norm_top * tex_h;
            let src_h = (norm_bot - norm_top) * tex_h;

            let dest_x = origin.x + (train_x - TRAIN_WIDTH / 2.0) * scale;
            let dest_y = origin.y + vis_top * scale;
            let dest_w = TRAIN_WIDTH * scale;
            let dest_h = (vis_bot - vis_top) * scale;

            draw_texture_ex(
                tex,
                dest_x,
                dest_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(dest_w, dest_h)),
                    source: Some(Rect::new(0.0, src_y, tex_w, src_h)),
                    ..Default::default()
                },
            );
        }
    }
}
