use crate::game::level::BoardVariant;
use macroquad::prelude::*;

const SPLINE_SAMPLES: usize = 480;
const STREAMLINE_CHANNELS: [f32; 7] = [-0.72, -0.48, -0.24, 0.0, 0.24, 0.48, 0.72];
const CAUSTIC_LANES: [f32; 5] = [-0.60, -0.30, 0.0, 0.30, 0.60];

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
    pub variant: BoardVariant,
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
        Self::river_crossing()
    }

    pub fn for_variant(variant: BoardVariant) -> Self {
        match variant {
            BoardVariant::Classic => Self::classic(),
            BoardVariant::RiverCrossing => Self::river_crossing(),
        }
    }

    pub fn classic() -> Self {
        let control_points = [
            // Top entrance from northwestern forest
            (Vec2::new(185.0, 0.0), 16.0),
            (Vec2::new(210.0, 60.0), 16.0),
            (Vec2::new(228.0, 120.0), 16.0),
            (Vec2::new(246.0, 180.0), 16.0),
            (Vec2::new(260.0, 240.0), 16.0),
            (Vec2::new(278.0, 300.0), 16.0),
            (Vec2::new(295.0, 355.0), 15.0),
            // Bridge 1: M0-T1
            (Vec2::new(314.0, 411.0), 15.0),
            (Vec2::new(345.0, 418.0), 18.0),
            // Bridge 2: T1-M1
            (Vec2::new(375.5, 419.5), 16.0),
            (Vec2::new(410.0, 420.0), 18.0),
            // Bridge 3: T1-M2
            (Vec2::new(443.0, 419.5), 16.0),
            (Vec2::new(478.0, 416.0), 16.0),
            // Bridge 4: T2-M2
            (Vec2::new(510.5, 414.0), 14.0),
            (Vec2::new(540.0, 422.0), 14.0),
            // Bridge 5: M2-T3
            (Vec2::new(566.0, 442.0), 14.0),
            (Vec2::new(576.0, 475.0), 15.0),
            // Bridge 6: M2-M3
            (Vec2::new(577.5, 510.0), 15.0),
            // Channel between M2-M3 and M3-B3
            (Vec2::new(586.0, 535.0), 15.0),
            (Vec2::new(602.0, 555.0), 15.0),
            (Vec2::new(622.0, 572.0), 14.0),
            // Bridge 7: M3-B3
            (Vec2::new(645.5, 586.0), 14.0),
            // Channel between Bridge 7 and Bridge 8
            (Vec2::new(676.0, 592.0), 14.0),
            // Bridge 8: B3-M4
            (Vec2::new(710.0, 602.0), 14.0),
            // Channel after Bridge 8
            (Vec2::new(722.0, 638.0), 13.0),
            (Vec2::new(725.0, 670.0), 13.0),
            (Vec2::new(718.0, 710.0), 13.0),
            (Vec2::new(708.0, 742.0), 14.0),
            // Whitewater frame notch
            (Vec2::new(701.0, 768.0), 14.0),
            // Southeastern forest outflow
            (Vec2::new(710.0, 815.0), 16.0),
            (Vec2::new(728.0, 860.0), 17.0),
            (Vec2::new(748.0, 905.0), 18.0),
            (Vec2::new(760.0, 950.0), 20.0),
            (Vec2::new(772.0, 990.0), 22.0),
            (Vec2::new(782.0, 1024.0), 24.0),
        ];
        Self::from_control_points(BoardVariant::Classic, &control_points)
    }

    pub fn river_crossing() -> Self {
        // Control points: (x, y, half_width) defining the continuous river channel across the board artwork
        let control_points = [
            // Left margin (x = -384.0 .. 0.0)
            (Vec2::new(-384.0, 721.0), 22.0),
            (Vec2::new(-320.0, 745.0), 20.0),
            (Vec2::new(-256.0, 770.0), 22.0),
            (Vec2::new(-192.0, 790.0), 22.0),
            (Vec2::new(-128.0, 818.0), 20.0),
            (Vec2::new(-64.0, 838.0), 20.0),
            // Central board (x = 0.0 .. 768.0)
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
            // Right margin (x = 768.0 .. 1024.0)
            (Vec2::new(816.0, 661.0), 24.0),
            (Vec2::new(864.0, 668.0), 26.0),
            (Vec2::new(912.0, 682.0), 24.0),
            (Vec2::new(960.0, 676.0), 24.0),
            (Vec2::new(1008.0, 662.0), 22.0),
            (Vec2::new(1024.0, 658.0), 20.0),
        ];
        Self::from_control_points(BoardVariant::RiverCrossing, &control_points)
    }

    pub fn from_control_points(variant: BoardVariant, control_points: &[(Vec2, f32)]) -> Self {
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
            variant,
            samples,
            total_length,
        }
    }

    /// Sample the river curve at an arc-length distance `dist` and cross-channel offset `v` in [-1.0, 1.0].
    pub fn sample_at(&self, dist: f32, v: f32) -> (Vec2, Vec2, Vec2, f32) {
        let clamped_dist = dist.clamp(0.0, self.total_length);

        // Binary search for the bounding segment in cumulative distance
        let idx = match self.samples.binary_search_by(|s| {
            s.cum_dist
                .partial_cmp(&clamped_dist)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
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

    /// Check if a position is occluded beneath a bridge deck.
    /// Returns an occlusion factor in [0.0, 1.0], where 1.0 is fully under the bridge deck.
    pub fn bridge_occlusion(&self, pos: Vec2) -> f32 {
        match self.variant {
            BoardVariant::Classic => classic_bridge_occlusion(pos),
            BoardVariant::RiverCrossing => self.river_crossing_bridge_occlusion(pos),
        }
    }

    fn river_crossing_bridge_occlusion(&self, pos: Vec2) -> f32 {
        // Railroad bridge region (train track centered at x = 40.0, deck span x in [0.0, 85.0], y in [790.0, 905.0])
        let in_rail_bridge = pos.x >= 0.0 && pos.x <= 85.0 && pos.y >= 790.0 && pos.y <= 905.0;
        if in_rail_bridge {
            let edge_dist = ((pos.x - 0.0).min(85.0 - pos.x) / 12.0).clamp(0.0, 1.0);
            return edge_dist;
        }

        // Row 6 M6 bottleneck wooden bridge region (node M6 at x = 384.0, y = 755.0)
        // Wider box accounts for thick river effect line widths bleeding past deck edges
        let in_wood_bridge = pos.x >= 290.0 && pos.x <= 470.0 && pos.y >= 690.0 && pos.y <= 815.0;
        if in_wood_bridge {
            let edge_dist = ((pos.x - 290.0).min(470.0 - pos.x) / 25.0).clamp(0.0, 1.0);
            return edge_dist;
        }

        0.0
    }
}

/// Oriented bounding box representation for bridge deck occlusion.
#[derive(Debug, Clone, Copy)]
struct BridgeBox {
    center: Vec2,
    dir: Vec2,
    half_len: f32,
    half_width: f32,
    fade: f32,
}

impl BridgeBox {
    fn occlusion(&self, pos: Vec2) -> f32 {
        let delta = pos - self.center;
        let u = (delta.dot(self.dir)).abs();
        let perp = Vec2::new(-self.dir.y, self.dir.x);
        let v = (delta.dot(perp)).abs();

        if u <= self.half_len && v <= self.half_width {
            let u_dist = self.half_len - u;
            let v_dist = self.half_width - v;
            (u_dist.min(v_dist) / self.fade).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// The 8 bridges crossing the river in the Classic theme.
const CLASSIC_BRIDGES: [BridgeBox; 8] = [
    // Bridge 1: M0 - T1 (diagonal)
    BridgeBox {
        center: Vec2::new(314.0, 411.0),
        dir: Vec2::new(0.590, -0.807),
        half_len: 36.0,
        half_width: 24.0,
        fade: 6.0,
    },
    // Bridge 2: T1 - M1 (vertical)
    BridgeBox {
        center: Vec2::new(375.5, 419.5),
        dir: Vec2::new(-0.006, 1.000),
        half_len: 36.0,
        half_width: 24.0,
        fade: 6.0,
    },
    // Bridge 3: T1 - M2 (diagonal)
    BridgeBox {
        center: Vec2::new(443.0, 419.5),
        dir: Vec2::new(0.595, 0.804),
        half_len: 36.0,
        half_width: 24.0,
        fade: 6.0,
    },
    // Bridge 4: T2 - M2 (vertical)
    BridgeBox {
        center: Vec2::new(510.5, 414.0),
        dir: Vec2::new(-0.006, 1.000),
        half_len: 36.0,
        half_width: 20.0,
        fade: 6.0,
    },
    // Bridge 5: M2 - T3 (diagonal)
    BridgeBox {
        center: Vec2::new(566.0, 442.0),
        dir: Vec2::new(0.605, -0.796),
        half_len: 36.0,
        half_width: 22.0,
        fade: 6.0,
    },
    // Bridge 6: M2 - M3 (horizontal)
    BridgeBox {
        center: Vec2::new(577.5, 510.0),
        dir: Vec2::new(1.000, 0.000),
        half_len: 44.0,
        half_width: 24.0,
        fade: 6.0,
    },
    // Bridge 7: M3 - B3 (vertical)
    BridgeBox {
        center: Vec2::new(645.5, 600.0),
        dir: Vec2::new(0.006, 1.000),
        half_len: 36.0,
        half_width: 20.0,
        fade: 6.0,
    },
    // Bridge 8: B3 - M4 (diagonal)
    BridgeBox {
        center: Vec2::new(712.0, 602.0),
        dir: Vec2::new(0.591, -0.806),
        half_len: 38.0,
        half_width: 22.0,
        fade: 6.0,
    },
];

fn classic_bridge_occlusion(pos: Vec2) -> f32 {
    CLASSIC_BRIDGES
        .iter()
        .map(|bridge| bridge.occlusion(pos))
        .fold(0.0, f32::max)
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

    pub fn for_variant(variant: BoardVariant) -> Self {
        Self {
            path: RiverPath::for_variant(variant),
            elapsed_time: 0.0,
        }
    }

    pub fn set_path(&mut self, path: RiverPath) {
        self.path = path;
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
            let alpha = 0.28 * breathe * (1.0 - occlusion);

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
                    let alpha = intensity.powf(1.5) * bank_fade * (1.0 - occlusion);

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
                        if wave > 0.35 && occlusion < 0.99 {
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
                        (caustic_val / 0.95 * bank_fade * (1.0 - occlusion)).clamp(0.0, 1.0);

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
                        if caustic_val > 0.45 && occlusion < 0.99 {
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
}
