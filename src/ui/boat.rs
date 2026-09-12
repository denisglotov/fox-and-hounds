use crate::ui::river::RiverPath;
use macroquad::prelude::*;

pub const INITIAL_DELAY: f32 = 15.0;
pub const CYCLE_DURATION: f32 = 60.0;
pub const TRANSIT_DURATION: f32 = 13.0;
pub const BOAT_SIZE: f32 = 48.0;

#[derive(Debug, Clone)]
pub struct BoatSimulation {
    pub elapsed_time: f32,
}

impl Default for BoatSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl BoatSimulation {
    pub fn new() -> Self {
        Self { elapsed_time: 0.0 }
    }

    pub fn cycle_progress(&self) -> f32 {
        if self.elapsed_time < INITIAL_DELAY {
            self.elapsed_time / INITIAL_DELAY
        } else {
            ((self.elapsed_time - INITIAL_DELAY) % CYCLE_DURATION) / CYCLE_DURATION
        }
    }

    pub fn is_active(&self) -> bool {
        self.transit_progress().is_some()
    }

    pub fn transit_progress(&self) -> Option<f32> {
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

    pub fn update(&mut self, dt: f32) {
        self.elapsed_time += dt;
    }

    pub fn draw(
        &self,
        origin: Vec2,
        scale: f32,
        river_path: &RiverPath,
        texture: Option<&Texture2D>,
    ) {
        let progress = match self.transit_progress() {
            Some(p) => p,
            None => return,
        };

        let t = self.elapsed_time;
        let dist = progress * river_path.total_length;
        // Subtle meandering within the central 10% of the channel
        let v_offset = (dist * 0.008 + (t * 0.6).sin() * 0.4).sin() * 0.12;

        let (pos, tangent, _normal, _half_width) = river_path.sample_at(dist, v_offset);

        // Downstream alignment with gentle ripple rocking motion
        let base_angle = tangent.y.atan2(tangent.x);
        let rock_angle = (t * 6.0).sin() * 0.08;
        let rotation = base_angle + rock_angle;

        let render_pos = origin + pos * scale;
        let size = BOAT_SIZE * scale;

        if let Some(tex) = texture {
            draw_texture_ex(
                tex,
                render_pos.x - size / 2.0,
                render_pos.y - size / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(size, size)),
                    rotation,
                    pivot: Some(render_pos),
                    ..Default::default()
                },
            );
        } else {
            // Fallback origami boat shape if texture is missing
            draw_circle(render_pos.x, render_pos.y, size * 0.4, WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boat_initial_delay_and_cycle_timing() {
        let mut boat = BoatSimulation::new();

        // 0s: inactive
        assert!(!boat.is_active());
        assert_eq!(boat.transit_progress(), None);

        // Advance to 14.9s: still inactive
        boat.update(14.9);
        assert!(!boat.is_active());
        assert_eq!(boat.transit_progress(), None);

        // Advance to 15.0s: activates at start of transit
        boat.update(0.1);
        assert!(boat.is_active());
        let p15 = boat.transit_progress().expect("should be active at 15s");
        assert!(p15.abs() < 1e-4);

        // Advance 6.5s (halfway through 13.0s transit): progress is ~0.5
        boat.update(6.5);
        assert!(boat.is_active());
        let p_half = boat.transit_progress().expect("should be active halfway");
        assert!((p_half - 0.5).abs() < 1e-3);

        // Advance past 13.0s transit: inactive
        boat.update(6.6);
        assert!(!boat.is_active());
        assert_eq!(boat.transit_progress(), None);

        // Advance to next cycle: 75.0s (15 + 60)
        boat.update(46.9);
        assert!(boat.is_active());
        let p_next = boat.transit_progress().expect("should be active at 75s");
        assert!(p_next.abs() < 1e-2);
    }
}
