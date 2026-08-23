use macroquad::prelude::*;

const SPLINE_SAMPLES: usize = 480;
const STREAMLINE_CHANNELS: [f32; 7] = [-0.72, -0.48, -0.24, 0.0, 0.24, 0.48, 0.72];
const CAUSTIC_LANES: [f32; 5] = [-0.60, -0.30, 0.0, 0.30, 0.60];
const NUM_TRAVELING_CRESTS: usize = 7;

/// A sampled point along the river's center path.
#[derive(Debug, Clone, Copy)]
pub struct RiverSample {
    pub pos: Vec2,
    pub tangent: Vec2,
    pub normal: Vec2,
    pub half_width: f32,
    pub cum_dist: f32,
}

/// Parameterized river spline curve with arc-length mapping.
#[derive(Debug, Clone)]
pub struct RiverPath {
    pub samples: Vec<RiverSample>,
    pub total_length: f32,
}

impl Default for RiverPath {
    fn default() -> Self {
        Self::new()
    }
}

impl RiverPath {
    pub fn new() -> Self {
        // Control points: (x, y, half_width) defining the continuous river channel across all background panels
        let control_points = [
            // Left extension (board_left.png: x = -384.0 .. 0.0)
            (Vec2::new(-384.0, 721.0), 22.0),
            (Vec2::new(-320.0, 745.0), 20.0),
            (Vec2::new(-256.0, 770.0), 22.0),
            (Vec2::new(-192.0, 790.0), 22.0),
            (Vec2::new(-128.0, 818.0), 20.0),
            (Vec2::new(-64.0, 838.0), 20.0),
            // Central board (board_image.png: x = 0.0 .. 768.0)
            (Vec2::new(0.0, 856.0), 24.0),
            (Vec2::new(45.0, 848.0), 22.0), // Under railroad bridge
            (Vec2::new(100.0, 832.0), 20.0),
            (Vec2::new(160.0, 810.0), 20.0),
            (Vec2::new(220.0, 780.0), 20.0),
            (Vec2::new(280.0, 758.0), 22.0),
            (Vec2::new(384.0, 755.0), 24.0), // Under M6 wooden bridge
            (Vec2::new(480.0, 755.0), 22.0),
            (Vec2::new(540.0, 732.0), 22.0),
            (Vec2::new(600.0, 702.0), 20.0),
            (Vec2::new(660.0, 686.0), 20.0),
            (Vec2::new(720.0, 670.0), 22.0),
            (Vec2::new(768.0, 655.0), 22.0),
            // Right extension (board_right.png: x = 768.0 .. 1024.0)
            (Vec2::new(816.0, 661.0), 24.0),
            (Vec2::new(864.0, 668.0), 26.0),
            (Vec2::new(912.0, 682.0), 24.0),
            (Vec2::new(960.0, 676.0), 24.0),
            (Vec2::new(1008.0, 662.0), 22.0),
            (Vec2::new(1024.0, 658.0), 20.0),
        ];

        let n = control_points.len();
        let mut raw_points = Vec::with_capacity(SPLINE_SAMPLES);

        // Catmull-Rom spline interpolation across control points
        for i in 0..SPLINE_SAMPLES {
            let t = i as f32 / (SPLINE_SAMPLES - 1) as f32;
            let u = t * (n - 1) as f32;
            let seg = (u.floor() as usize).min(n - 2);
            let local_t = u - seg as f32;

            let p0 = if seg > 0 {
                control_points[seg - 1].0
            } else {
                control_points[0].0 * 2.0 - control_points[1].0
            };
            let p1 = control_points[seg].0;
            let p2 = control_points[seg + 1].0;
            let p3 = if seg + 2 < n {
                control_points[seg + 2].0
            } else {
                control_points[n - 1].0 * 2.0 - control_points[n - 2].0
            };

            let pos = catmull_rom(p0, p1, p2, p3, local_t);

            let w1 = control_points[seg].1;
            let w2 = control_points[seg + 1].1;
            let half_width = w1 + (w2 - w1) * local_t;

            raw_points.push((pos, half_width));
        }

        // Compute cumulative arc-length distances and tangent/normal vectors
        let mut samples = Vec::with_capacity(SPLINE_SAMPLES);
        let mut cum_dist = 0.0;

        for i in 0..SPLINE_SAMPLES {
            let (pos, half_width) = raw_points[i];
            if i > 0 {
                cum_dist += (pos - raw_points[i - 1].0).length();
            }

            let tangent = if i == 0 {
                (raw_points[1].0 - pos).normalize_or_zero()
            } else if i == SPLINE_SAMPLES - 1 {
                (pos - raw_points[i - 1].0).normalize_or_zero()
            } else {
                (raw_points[i + 1].0 - raw_points[i - 1].0).normalize_or_zero()
            };

            let normal = Vec2::new(-tangent.y, tangent.x);

            samples.push(RiverSample {
                pos,
                tangent,
                normal,
                half_width,
                cum_dist,
            });
        }

        let total_length = cum_dist;
        Self {
            samples,
            total_length,
        }
    }

    /// Sample the river curve at an arc-length distance `dist` and cross-channel offset `v` in [-1.0, 1.0].
    pub fn sample_at(&self, dist: f32, v: f32) -> (Vec2, Vec2, Vec2, f32) {
        let clamped_dist = dist.clamp(0.0, self.total_length);

        // Binary search for the bounding segment in cumulative distance
        let idx = match self
            .samples
            .binary_search_by(|s| s.cum_dist.partial_cmp(&clamped_dist).unwrap())
        {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
        };

        if idx >= self.samples.len() - 1 {
            let last = &self.samples[self.samples.len() - 1];
            let pt = last.pos + last.normal * (v * last.half_width);
            return (pt, last.tangent, last.normal, last.half_width);
        }

        let s0 = &self.samples[idx];
        let s1 = &self.samples[idx + 1];
        let seg_len = (s1.cum_dist - s0.cum_dist).max(1e-4);
        let t = ((clamped_dist - s0.cum_dist) / seg_len).clamp(0.0, 1.0);

        let pos = s0.pos.lerp(s1.pos, t);
        let tangent = s0.tangent.lerp(s1.tangent, t).normalize_or_zero();
        let normal = s0.normal.lerp(s1.normal, t).normalize_or_zero();
        let half_width = s0.half_width + (s1.half_width - s0.half_width) * t;

        let channel_pos = pos + normal * (v * half_width);
        (channel_pos, tangent, normal, half_width)
    }

    /// Check if a position is occluded beneath either the railway bridge or the M6 bottleneck bridge.
    /// Returns an occlusion factor in [0.0, 1.0], where 1.0 is fully under the bridge deck.
    pub fn bridge_occlusion(&self, pos: Vec2) -> f32 {
        // Railroad bridge region
        let in_rail_bridge = pos.x >= 10.0 && pos.x <= 75.0 && pos.y >= 800.0 && pos.y <= 895.0;
        if in_rail_bridge {
            let edge_dist = ((pos.x - 10.0).min(75.0 - pos.x) / 16.0).clamp(0.0, 1.0);
            return 0.88 * edge_dist;
        }

        // Row 6 M6 bottleneck bridge region
        let in_wood_bridge = pos.x >= 300.0 && pos.x <= 470.0 && pos.y >= 700.0 && pos.y <= 810.0;
        if in_wood_bridge {
            let edge_dist = ((pos.x - 300.0).min(470.0 - pos.x) / 24.0).clamp(0.0, 1.0);
            return 0.88 * edge_dist;
        }

        0.0
    }
}

fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// Dynamic River Simulation creating a silky smooth flowing water current and sunlight caustics.
#[derive(Debug, Clone)]
pub struct RiverSimulation {
    pub path: RiverPath,
    pub elapsed_time: f32,
}

impl Default for RiverSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl RiverSimulation {
    pub fn new() -> Self {
        Self {
            path: RiverPath::new(),
            elapsed_time: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed_time += dt;
    }

    pub fn draw(&self, origin: Vec2, scale: f32) {
        let t = self.elapsed_time;

        // 1. Soft Ambient Liquid Glow along riverbed
        self.draw_riverbed_ambient_glow(origin, scale, t);

        // 2. Silky Flow Streamlines & Wave Shimmer
        self.draw_silky_streamlines(origin, scale, t);

        // 3. Sunlight Caustics Network (Glistening Caustic Webs)
        self.draw_sunlight_caustics(origin, scale, t);

        // 4. Gentle Traveling Curved Wave Crests
        self.draw_traveling_wave_crests(origin, scale, t);
    }

    /// Soft, luminous liquid glow that gives the river clear depth and vibrancy
    fn draw_riverbed_ambient_glow(&self, origin: Vec2, scale: f32, t: f32) {
        let step = 20.0;
        let num_steps = (self.path.total_length / step).ceil() as usize;

        for i in 0..num_steps {
            let d0 = (i as f32 * step).min(self.path.total_length);
            let d1 = ((i + 1) as f32 * step).min(self.path.total_length);
            if (d1 - d0) < 1.0 {
                continue;
            }

            let mid_d = (d0 + d1) * 0.5;
            let (p0, _, _, w0) = self.path.sample_at(d0, 0.0);
            let (p1, _, _, w1) = self.path.sample_at(d1, 0.0);

            let occlusion = self.path.bridge_occlusion((p0 + p1) * 0.5);
            let breathe = ((mid_d * 0.03 - t * 1.5).sin() * 0.5 + 0.5) * 0.3 + 0.7;
            let alpha = 0.28 * breathe * (1.0 - occlusion * 0.85);

            if alpha > 0.02 {
                let screen_p0 = origin + p0 * scale;
                let screen_p1 = origin + p1 * scale;
                let avg_width = (w0 + w1) * 0.5 * 1.35 * scale;

                let glow_color =
                    Color::from_rgba(77, 208, 225, (alpha * 110.0).clamp(0.0, 255.0) as u8);
                draw_line(
                    screen_p0.x,
                    screen_p0.y,
                    screen_p1.x,
                    screen_p1.y,
                    avg_width,
                    glow_color,
                );
            }
        }
    }

    /// Continuous silky streamlines that flow smoothly downstream with harmonic shimmer
    fn draw_silky_streamlines(&self, origin: Vec2, scale: f32, t: f32) {
        let step = 10.0;
        let num_steps = (self.path.total_length / step).ceil() as usize;

        for &v in &STREAMLINE_CHANNELS {
            let bank_fade = (1.0 - v * v).powf(1.4); // Smooth bank falloff

            for i in 0..num_steps {
                let d0 = (i as f32 * step).min(self.path.total_length);
                let d1 = ((i + 1) as f32 * step).min(self.path.total_length);
                if (d1 - d0) < 1.0 {
                    continue;
                }

                let mid_d = (d0 + d1) * 0.5;
                let (p0, _, _, _) = self.path.sample_at(d0, v);
                let (p1, _, _, _) = self.path.sample_at(d1, v);

                // Multi-frequency wave traveling downstream
                // Velocity is slightly faster in center (v=0)
                let flow_speed = 3.2 + (1.0 - v.abs()) * 0.8;
                let w1 = (mid_d * 0.048 - t * flow_speed + v * 2.4).sin();
                let w2 = (mid_d * 0.095 - t * (flow_speed * 1.35) - v * 3.1).sin();
                let w3 = (mid_d * 0.022 - t * 1.2).cos();
                let wave = w1 * 0.45 + w2 * 0.35 + w3 * 0.20;

                if wave > -0.15 {
                    let occlusion = self.path.bridge_occlusion((p0 + p1) * 0.5);
                    let intensity = ((wave + 0.15) / 1.15).clamp(0.0, 1.0);
                    let alpha = intensity.powf(1.5) * bank_fade * (1.0 - occlusion * 0.88);

                    if alpha > 0.015 {
                        let screen_p0 = origin + p0 * scale;
                        let screen_p1 = origin + p1 * scale;

                        // Layer 1: Soft aqua ribbon
                        let ribbon_color = Color::from_rgba(
                            128,
                            222,
                            234,
                            (alpha * 135.0).clamp(0.0, 255.0) as u8,
                        );
                        draw_line(
                            screen_p0.x,
                            screen_p0.y,
                            screen_p1.x,
                            screen_p1.y,
                            (2.2 + wave * 1.2) * scale,
                            ribbon_color,
                        );

                        // Layer 2: Silky bright specular center
                        if wave > 0.35 {
                            let core_intensity = ((wave - 0.35) / 0.65 * bank_fade).clamp(0.0, 1.0);
                            let core_color = Color::from_rgba(
                                255,
                                255,
                                255,
                                (core_intensity * (1.0 - occlusion) * 195.0).clamp(0.0, 255.0)
                                    as u8,
                            );
                            draw_line(
                                screen_p0.x,
                                screen_p0.y,
                                screen_p1.x,
                                screen_p1.y,
                                1.1 * scale,
                                core_color,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Sunlight caustics that form a shimmering, moving web of sunlight rays across the water
    fn draw_sunlight_caustics(&self, origin: Vec2, scale: f32, t: f32) {
        let step = 16.0;
        let num_steps = (self.path.total_length / step).ceil() as usize;

        for &v in &CAUSTIC_LANES {
            let bank_fade = (1.0 - v * v).max(0.1);

            for i in 0..num_steps {
                let d0 = (i as f32 * step).min(self.path.total_length);
                let d1 = ((i + 1) as f32 * step).min(self.path.total_length);
                if (d1 - d0) < 1.0 {
                    continue;
                }

                let mid_d = (d0 + d1) * 0.5;
                let (p0, _, _, _) = self.path.sample_at(d0, v);
                let (p1, _, _, _) = self.path.sample_at(d1, v);

                // Procedural cellular caustic function
                let c1 = (mid_d * 0.065 - t * 2.2 + v * 3.5).sin();
                let c2 = (mid_d * 0.115 + t * 1.7 - v * 4.2).sin();
                let c3 = (mid_d * 0.038 - t * 2.8).cos();
                let caustic_val = ((c1 + c2 + c3 - 1.1) / 1.9).max(0.0).powi(2);

                if caustic_val > 0.05 {
                    let occlusion = self.path.bridge_occlusion((p0 + p1) * 0.5);
                    let alpha =
                        (caustic_val / 0.95 * bank_fade * (1.0 - occlusion * 0.90)).clamp(0.0, 1.0);

                    if alpha > 0.02 {
                        let screen_p0 = origin + p0 * scale;
                        let screen_p1 = origin + p1 * scale;

                        // Shimmering caustic filament
                        let caustic_color = Color::from_rgba(
                            255,
                            255,
                            255,
                            (alpha * 180.0).clamp(0.0, 255.0) as u8,
                        );
                        draw_line(
                            screen_p0.x,
                            screen_p0.y,
                            screen_p1.x,
                            screen_p1.y,
                            (1.5 + caustic_val * 1.5) * scale,
                            caustic_color,
                        );

                        // Occasional caustic sparkle node at intersections
                        if caustic_val > 0.45 {
                            let sparkle_alpha =
                                ((caustic_val - 0.45) / 0.55 * bank_fade * (1.0 - occlusion))
                                    .clamp(0.0, 1.0);
                            let center_pos = (screen_p0 + screen_p1) * 0.5;
                            let r = (2.2 + caustic_val * 2.0) * scale;

                            draw_circle(
                                center_pos.x,
                                center_pos.y,
                                r,
                                Color::from_rgba(
                                    224,
                                    247,
                                    250,
                                    (sparkle_alpha * 150.0).clamp(0.0, 255.0) as u8,
                                ),
                            );
                            draw_circle(
                                center_pos.x,
                                center_pos.y,
                                r * 0.5,
                                Color::from_rgba(
                                    255,
                                    255,
                                    255,
                                    (sparkle_alpha * 220.0).clamp(0.0, 255.0) as u8,
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    /// Delicate curved wave crests that gently travel downstream
    fn draw_traveling_wave_crests(&self, origin: Vec2, scale: f32, t: f32) {
        let total_len = self.path.total_length;
        let wave_speed = 42.0; // px / sec
        let cycle_len = total_len / NUM_TRAVELING_CRESTS as f32;

        for k in 0..NUM_TRAVELING_CRESTS {
            let crest_d =
                ((t * wave_speed + k as f32 * cycle_len) % total_len).clamp(0.0, total_len);

            // Draw a curved wave crest across the channel from v = -0.75 to v = 0.75
            const NUM_ARC_PTS: usize = 12;
            let mut arc_points = [(Vec2::ZERO, 0.0f32); NUM_ARC_PTS];

            for (j, pt) in arc_points.iter_mut().enumerate() {
                let frac = j as f32 / (NUM_ARC_PTS - 1) as f32;
                let v = -0.75 + frac * 1.50; // cross channel [-0.75, 0.75]

                // Parabolic downstream curve: center (v=0) travels slightly ahead of edges
                let curve_lead = (1.0 - (v / 0.75).powi(2)) * 14.0;
                let sample_d = (crest_d + curve_lead).clamp(0.0, total_len);

                let (board_pos, _, _, _) = self.path.sample_at(sample_d, v);
                let occlusion = self.path.bridge_occlusion(board_pos);
                let bank_fade = (1.0 - (v / 0.85).abs()).max(0.0);
                let point_alpha = (bank_fade * (1.0 - occlusion * 0.90)).clamp(0.0, 1.0);

                let screen_pos = origin + board_pos * scale;
                *pt = (screen_pos, point_alpha);
            }

            // Draw connected crest segments
            for j in 0..NUM_ARC_PTS - 1 {
                let (p0, a0) = arc_points[j];
                let (p1, a1) = arc_points[j + 1];
                let seg_alpha = (a0 + a1) * 0.5;

                if seg_alpha > 0.05 {
                    let crest_color = Color::from_rgba(
                        255,
                        255,
                        255,
                        (seg_alpha * 140.0).clamp(0.0, 255.0) as u8,
                    );
                    let soft_cyan =
                        Color::from_rgba(178, 235, 242, (seg_alpha * 90.0).clamp(0.0, 255.0) as u8);

                    // Soft back ripple
                    draw_line(p0.x, p0.y, p1.x, p1.y, 2.4 * scale, soft_cyan);
                    // Fine white crest
                    draw_line(p0.x, p0.y, p1.x, p1.y, 1.2 * scale, crest_color);
                }
            }
        }
    }
}
