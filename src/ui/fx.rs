use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct ConfettiParticle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub size: f32,
    pub rotation: f32,
    pub rot_speed: f32,
    pub life: f32,
}

pub struct FxManager {
    pub particles: Vec<ConfettiParticle>,
}

impl Default for FxManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FxManager {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    pub fn spawn_victory_burst(&mut self, center: Vec2, count: usize) {
        let colors = [
            Color::from_rgba(255, 193, 7, 255),  // Gold
            Color::from_rgba(255, 87, 34, 255),  // Orange
            Color::from_rgba(33, 150, 243, 255), // Blue
            Color::from_rgba(76, 175, 80, 255),  // Green
            Color::from_rgba(233, 30, 99, 255),  // Pink
            Color::from_rgba(255, 235, 59, 255), // Yellow
        ];

        for _ in 0..count {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(120.0, 480.0);
            let color = colors[rand::gen_range(0, colors.len())];

            self.particles.push(ConfettiParticle {
                pos: center,
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed - 150.0),
                color,
                size: rand::gen_range(6.0, 14.0),
                rotation: rand::gen_range(0.0, std::f32::consts::TAU),
                rot_speed: rand::gen_range(-8.0, 8.0),
                life: rand::gen_range(1.5, 3.2),
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.particles.retain_mut(|p| {
            p.life -= dt;
            p.pos += p.vel * dt;
            p.vel.y += 350.0 * dt; // Gravity
            p.vel.x *= 0.98; // Drag
            p.rotation += p.rot_speed * dt;
            p.life > 0.0
        });
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let alpha = (p.life / 1.5).min(1.0);
            let mut c = p.color;
            c.a *= alpha;

            draw_rectangle_ex(
                p.pos.x,
                p.pos.y,
                p.size,
                p.size * 0.6,
                DrawRectangleParams {
                    offset: Vec2::new(0.5, 0.5),
                    rotation: p.rotation,
                    color: c,
                },
            );
        }
    }
}
