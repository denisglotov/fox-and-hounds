use crate::audio::SoundTrigger;
use crate::game::graph::NodeType;
use crate::game::level::{BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH};
use crate::game::state::{Faction, GamePhase, GameResult, GameState};
use macroquad::prelude::*;

pub struct BoardView {
    pub board_texture: Option<Texture2D>,
    pub fox_texture: Option<Texture2D>,
    pub hound1_texture: Option<Texture2D>,
    pub hound2_texture: Option<Texture2D>,
    pub hound3_texture: Option<Texture2D>,
    pub hover_node_id: Option<usize>,
    pub font: Option<Font>,
}

impl BoardView {
    pub async fn new(font: Option<Font>) -> Self {
        let board_texture = {
            let tex = Texture2D::from_file_with_format(
                include_bytes!("../../assets/board_image.png"),
                Some(ImageFormat::Png),
            );
            tex.set_filter(FilterMode::Linear);
            Some(tex)
        };

        let fox_texture = {
            let tex = Texture2D::from_file_with_format(
                include_bytes!("../../assets/fox_figure.png"),
                Some(ImageFormat::Png),
            );
            tex.set_filter(FilterMode::Linear);
            Some(tex)
        };

        let hound1_texture = {
            let tex = Texture2D::from_file_with_format(
                include_bytes!("../../assets/hound1_figure.png"),
                Some(ImageFormat::Png),
            );
            tex.set_filter(FilterMode::Linear);
            Some(tex)
        };

        let hound2_texture = {
            let tex = Texture2D::from_file_with_format(
                include_bytes!("../../assets/hound2_figure.png"),
                Some(ImageFormat::Png),
            );
            tex.set_filter(FilterMode::Linear);
            Some(tex)
        };

        let hound3_texture = {
            let tex = Texture2D::from_file_with_format(
                include_bytes!("../../assets/hound3_figure.png"),
                Some(ImageFormat::Png),
            );
            tex.set_filter(FilterMode::Linear);
            Some(tex)
        };

        Self {
            board_texture,
            fox_texture,
            hound1_texture,
            hound2_texture,
            hound3_texture,
            hover_node_id: None,
            font,
        }
    }

    pub fn draw_and_handle_input(
        &mut self,
        state: &mut GameState,
        origin: Vec2,
        scale: f32,
        viewport_mouse_pos: Vec2,
        was_dragging: bool,
    ) -> Option<SoundTrigger> {
        let mut sound_trigger = None;
        let t = get_time() as f32;

        // 1. Draw Background Board Image
        if let Some(tex) = &self.board_texture {
            draw_texture_ex(
                tex,
                origin.x,
                origin.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        BOARD_IMAGE_WIDTH * scale,
                        BOARD_IMAGE_HEIGHT * scale,
                    )),
                    ..Default::default()
                },
            );
        } else {
            // Fallback dark board container
            draw_rectangle(
                origin.x,
                origin.y,
                BOARD_IMAGE_WIDTH * scale,
                BOARD_IMAGE_HEIGHT * scale,
                Color::from_rgba(18, 28, 42, 255),
            );
        }

        // 2. Find hovered node & Determine Legal Targets for Player
        let board_mouse = (viewport_mouse_pos - origin) / scale;
        let hit_radius = 36.0;

        self.hover_node_id = state.graph.nodes.iter().find_map(|node| {
            if (node.visual_pos - board_mouse).length() <= hit_radius {
                Some(node.id)
            } else {
                None
            }
        });

        let legal_destinations = if state.phase == GamePhase::Playing && !state.is_ai_turn() {
            match state.player_faction {
                Faction::Fox => state.fox_legal_moves(),
                Faction::Hounds => {
                    if let Some(h_idx) = state.selected_hound_idx {
                        state.hound_legal_moves(h_idx)
                    } else {
                        Vec::new()
                    }
                }
            }
        } else {
            Vec::new()
        };

        // 3. Draw Graph Nodes & Interactive Indicators
        self.draw_nodes(state, origin, scale, &legal_destinations, t);

        // 4. Draw Animated Live Characters (Fox & 3 Unique Hounds)
        self.draw_pieces(state, origin, scale, t);

        // 5. Handle Player Tap / Click Input (only if not dragging)
        if is_mouse_button_released(MouseButton::Left)
            && !was_dragging
            && state.phase == GamePhase::Playing
            && !state.is_ai_turn()
            && state.result == GameResult::Ongoing
        {
            if let Some(clicked_node) = self.hover_node_id {
                sound_trigger = self.handle_node_click(state, clicked_node);
            }
        }

        sound_trigger
    }

    fn handle_node_click(
        &mut self,
        state: &mut GameState,
        clicked_node: usize,
    ) -> Option<SoundTrigger> {
        match state.player_faction {
            Faction::Fox => {
                let legal = state.fox_legal_moves();
                if legal.contains(&clicked_node) {
                    if state.apply_fox_move(clicked_node).is_ok() {
                        if state.result == GameResult::FoxWon {
                            Some(SoundTrigger::Win)
                        } else {
                            Some(SoundTrigger::Move)
                        }
                    } else {
                        Some(SoundTrigger::InvalidMove)
                    }
                } else if clicked_node == state.fox_pos {
                    Some(SoundTrigger::Select)
                } else {
                    Some(SoundTrigger::InvalidMove)
                }
            }
            Faction::Hounds => {
                // Check if user clicked on one of their Hounds
                if let Some(hound_idx) = state.hounds_pos.iter().position(|&p| p == clicked_node) {
                    state.selected_hound_idx = Some(hound_idx);
                    Some(SoundTrigger::Select)
                } else if let Some(hound_idx) = state.selected_hound_idx {
                    // Try to move selected Hound to clicked node
                    let legal = state.hound_legal_moves(hound_idx);
                    if legal.contains(&clicked_node) {
                        if state.apply_hound_move(hound_idx, clicked_node).is_ok() {
                            if state.result == GameResult::HoundsWon {
                                Some(SoundTrigger::Win)
                            } else {
                                Some(SoundTrigger::Move)
                            }
                        } else {
                            Some(SoundTrigger::InvalidMove)
                        }
                    } else {
                        Some(SoundTrigger::InvalidMove)
                    }
                } else {
                    Some(SoundTrigger::InvalidMove)
                }
            }
        }
    }

    fn draw_nodes(
        &self,
        state: &GameState,
        origin: Vec2,
        scale: f32,
        legal_destinations: &[usize],
        t: f32,
    ) {
        let pulse = (t * 4.0).sin() * 0.5 + 0.5;

        for node in &state.graph.nodes {
            let pos = origin + node.visual_pos * scale;
            let is_hovered = self.hover_node_id == Some(node.id);
            let is_legal = legal_destinations.contains(&node.id);

            // 1. Legal Destination Glowing Halo
            if is_legal {
                let halo_radius = (28.0 + pulse * 6.0) * scale;
                let halo_color = match state.player_faction {
                    Faction::Fox => Color::from_rgba(255, 152, 0, 110 + (pulse * 80.0) as u8),
                    Faction::Hounds => Color::from_rgba(33, 150, 243, 110 + (pulse * 80.0) as u8),
                };
                draw_circle(pos.x, pos.y, halo_radius, halo_color);

                // Inner target ring
                let ring_color = match state.player_faction {
                    Faction::Fox => Color::from_rgba(255, 238, 88, 240),
                    Faction::Hounds => Color::from_rgba(129, 212, 250, 240),
                };
                draw_circle_lines(
                    pos.x,
                    pos.y,
                    (18.0 + pulse * 2.0) * scale,
                    2.5 * scale,
                    ring_color,
                );
            }

            // 2. Base Node Circle Plate
            let base_radius = if is_hovered {
                20.0 * scale
            } else {
                16.0 * scale
            };
            let base_color = if node.id == state.coop_pos {
                Color::from_rgba(255, 215, 0, 140)
            } else if node.node_type == NodeType::Bottleneck {
                // Bridge Chokepoint
                Color::from_rgba(79, 195, 247, 90)
            } else {
                Color::from_rgba(255, 255, 255, 45)
            };

            draw_circle(pos.x, pos.y, base_radius, base_color);
            draw_circle_lines(
                pos.x,
                pos.y,
                base_radius,
                1.5 * scale,
                Color::from_rgba(255, 255, 255, 120),
            );

            // Hover indicator ring
            if is_hovered {
                draw_circle_lines(
                    pos.x,
                    pos.y,
                    base_radius + 4.0 * scale,
                    2.0 * scale,
                    Color::from_rgba(255, 255, 255, 200),
                );
            }
        }
    }

    fn draw_pieces(&self, state: &GameState, origin: Vec2, scale: f32, t: f32) {
        let pulse = (t * 3.5).sin() * 0.5 + 0.5;

        // 1. Draw Hounds (Left: Terrier/User's dog, Mid: Beagle, Right: Golden)
        for (idx, &hound_pos) in state.hounds_pos.iter().enumerate() {
            let is_selected = state.selected_hound_idx == Some(idx);
            let hound_node = state.graph.node(hound_pos);

            let is_moving = state.active_anim.as_ref().is_some_and(|anim| {
                anim.faction == Faction::Hounds
                    && state.move_history.last().is_some_and(|m| {
                        matches!(m, crate::game::state::PieceMove::HoundMove { hound_idx, .. } if *hound_idx == idx)
                    })
            });

            let (visual_pos, jump_lift, move_rot) = if is_moving {
                let anim = state.active_anim.as_ref().unwrap();
                let ease = 1.0 - (1.0 - anim.progress).powi(2);
                let pos = anim.from.lerp(anim.to, ease);
                let jump = (anim.progress * std::f32::consts::PI).sin() * 24.0;

                // Dynamic rotation during move
                let delta = anim.to - anim.from;
                let angle = (delta.y).atan2(delta.x) - std::f32::consts::FRAC_PI_2;
                (pos, jump, angle)
            } else if let Some(n) = hound_node {
                (n.visual_pos, 0.0, 0.0)
            } else {
                (Vec2::ZERO, 0.0, 0.0)
            };

            let ground_pos = origin + visual_pos * scale;
            let pick_up_lift = if is_selected { 8.0 * scale } else { 0.0 };
            let render_pos = Vec2::new(
                ground_pos.x,
                ground_pos.y - (jump_lift * scale) - pick_up_lift,
            );

            // Selection / Active Turn Indicator beneath paws
            if is_selected {
                draw_circle(
                    ground_pos.x,
                    ground_pos.y,
                    (22.0 + pulse * 4.0) * scale,
                    Color::from_rgba(30, 136, 229, 90),
                );
                draw_circle_lines(
                    ground_pos.x,
                    ground_pos.y,
                    (18.0 + pulse * 2.0) * scale,
                    2.5 * scale,
                    Color::from_rgba(100, 181, 246, 255),
                );
            } else if state.current_turn == Faction::Hounds
                && state.player_faction == Faction::Hounds
            {
                draw_circle(
                    ground_pos.x,
                    ground_pos.y,
                    (18.0 + pulse * 3.0) * scale,
                    Color::from_rgba(33, 150, 243, 60),
                );
            }

            // Select Hound Texture:
            // idx 0 -> Terrier (user's dog)
            // idx 1 -> Beagle
            // idx 2 -> Golden Hound
            let hound_tex = match idx {
                0 => self.hound1_texture.as_ref(),
                1 => self.hound2_texture.as_ref(),
                _ => self.hound3_texture.as_ref(),
            };

            if let Some(tex) = hound_tex {
                // Subtle breathing / idle sway
                let idle_breathe = if !is_moving {
                    (t * 3.0 + idx as f32 * 1.5).sin() * 0.02
                } else {
                    0.0
                };
                let idle_sway = if !is_moving {
                    (t * 2.5 + idx as f32 * 1.2).sin() * 0.03
                } else {
                    0.0
                };

                let target_h = (72.0 + idle_breathe * 15.0) * scale;
                let aspect = tex.width() / tex.height();
                let target_w = target_h * aspect;

                let rot = move_rot + idle_sway;

                draw_texture_ex(
                    tex,
                    render_pos.x - target_w / 2.0,
                    render_pos.y - target_h / 2.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(target_w, target_h)),
                        rotation: rot,
                        pivot: Some(render_pos),
                        ..Default::default()
                    },
                );
            }
        }

        // 2. Draw Fox
        let fox_node = state.graph.node(state.fox_pos);
        let is_fox_moving = state
            .active_anim
            .as_ref()
            .is_some_and(|anim| anim.faction == Faction::Fox);

        let (visual_pos, jump_lift, move_rot) = if is_fox_moving {
            let anim = state.active_anim.as_ref().unwrap();
            let ease = 1.0 - (1.0 - anim.progress).powi(2);
            let pos = anim.from.lerp(anim.to, ease);
            let jump = (anim.progress * std::f32::consts::PI).sin() * 26.0;

            // Dynamic rotation during Fox move
            let delta = anim.to - anim.from;
            let angle = (delta.y).atan2(delta.x) + std::f32::consts::FRAC_PI_2;
            (pos, jump, angle)
        } else if let Some(n) = fox_node {
            (n.visual_pos, 0.0, 0.0)
        } else {
            (Vec2::ZERO, 0.0, 0.0)
        };

        let ground_pos = origin + visual_pos * scale;
        let render_pos = Vec2::new(ground_pos.x, ground_pos.y - (jump_lift * scale));

        // Active Turn Glow for Fox
        if state.current_turn == Faction::Fox {
            draw_circle(
                ground_pos.x,
                ground_pos.y,
                (20.0 + pulse * 4.0) * scale,
                Color::from_rgba(255, 112, 67, 85),
            );
            draw_circle_lines(
                ground_pos.x,
                ground_pos.y,
                (16.0 + pulse * 2.0) * scale,
                2.0 * scale,
                Color::from_rgba(255, 171, 145, 200),
            );
        }

        if let Some(tex) = &self.fox_texture {
            let idle_breathe = if !is_fox_moving {
                (t * 3.2).sin() * 0.02
            } else {
                0.0
            };
            let idle_sway = if !is_fox_moving {
                (t * 2.2).sin() * 0.03
            } else {
                0.0
            };

            let target_h = (76.0 + idle_breathe * 15.0) * scale;
            let aspect = tex.width() / tex.height();
            let target_w = target_h * aspect;

            let rot = move_rot + idle_sway;

            draw_texture_ex(
                tex,
                render_pos.x - target_w / 2.0,
                render_pos.y - target_h / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(target_w, target_h)),
                    rotation: rot,
                    pivot: Some(render_pos),
                    ..Default::default()
                },
            );
        }
    }
}
