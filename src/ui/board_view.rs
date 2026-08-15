use crate::audio::SoundTrigger;
use crate::game::level::{BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH};
use crate::game::state::{Faction, GamePhase, GameResult, GameState};
use macroquad::prelude::*;

pub struct BoardView {
    pub board_texture: Option<Texture2D>,
    pub hover_node_id: Option<usize>,
}

impl BoardView {
    pub async fn new() -> Self {
        let board_texture = match load_texture("assets/board_image.png").await {
            Ok(tex) => {
                tex.set_filter(FilterMode::Linear);
                Some(tex)
            }
            Err(e) => {
                eprintln!("Warning: Failed to load board_image.png: {:?}", e);
                None
            }
        };

        Self {
            board_texture,
            hover_node_id: None,
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

        // 2. Draw Connecting Edges
        self.draw_edges(state, origin, scale, t);

        // 3. Find hovered node & Determine Legal Targets for Player
        let board_mouse = (viewport_mouse_pos - origin) / scale;
        let hit_radius = 32.0;

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

        // 4. Draw Graph Nodes & Interactive Indicators
        self.draw_nodes(state, origin, scale, &legal_destinations, t);

        // 5. Draw Animated Pieces (Fox, Hounds, Chicken Coop)
        self.draw_pieces(state, origin, scale, t);

        // 6. Handle Player Tap / Click Input (only if not dragging)
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

    fn draw_edges(&self, state: &GameState, origin: Vec2, scale: f32, _t: f32) {
        let edge_color = Color::from_rgba(255, 255, 255, 38);
        let edge_shadow = Color::from_rgba(0, 0, 0, 90);
        let thickness = 2.5 * scale;

        for (u_idx, neighbors) in state.graph.adjacency.iter().enumerate() {
            if let Some(u_node) = state.graph.node(u_idx) {
                let p1 = origin + u_node.visual_pos * scale;
                for &v_idx in neighbors {
                    if v_idx > u_idx {
                        if let Some(v_node) = state.graph.node(v_idx) {
                            let p2 = origin + v_node.visual_pos * scale;
                            // Drop shadow line
                            draw_line(
                                p1.x,
                                p1.y + 1.5 * scale,
                                p2.x,
                                p2.y + 1.5 * scale,
                                thickness,
                                edge_shadow,
                            );
                            // Main path line
                            draw_line(p1.x, p1.y, p2.x, p2.y, thickness, edge_color);
                        }
                    }
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
            } else if node.id == 19 {
                // M7 Bridge Chokepoint
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

        // 1. Draw Chicken Coop on M0
        if let Some(coop_node) = state.graph.node(state.coop_pos) {
            let pos = origin + coop_node.visual_pos * scale;
            let coop_r = (26.0 + pulse * 3.0) * scale;

            // Golden Beacon Aura
            draw_circle(pos.x, pos.y, coop_r, Color::from_rgba(255, 193, 7, 70));
            draw_circle(
                pos.x,
                pos.y,
                22.0 * scale,
                Color::from_rgba(255, 224, 130, 230),
            );
            draw_circle_lines(
                pos.x,
                pos.y,
                22.0 * scale,
                2.5 * scale,
                Color::from_rgba(255, 143, 0, 255),
            );

            // Chicken Coop Icon
            let icon_text = "🐔";
            let font_size = (24.0 * scale) as u16;
            let text_dims = measure_text(icon_text, None, font_size, 1.0);
            draw_text(
                icon_text,
                pos.x - text_dims.width / 2.0,
                pos.y + text_dims.height / 3.0,
                font_size as f32,
                WHITE,
            );
        }

        // 2. Draw Hounds
        for (idx, &hound_pos) in state.hounds_pos.iter().enumerate() {
            let is_selected = state.selected_hound_idx == Some(idx);
            let hound_node = state.graph.node(hound_pos);

            let visual_pos = if let Some(anim) = &state.active_anim {
                if anim.faction == Faction::Hounds && state.move_history.last().is_some_and(|m| matches!(m, crate::game::state::PieceMove::HoundMove { hound_idx, .. } if *hound_idx == idx)) {
                    let ease = 1.0 - (1.0 - anim.progress).powi(2);
                    anim.from.lerp(anim.to, ease)
                } else if let Some(n) = hound_node {
                    n.visual_pos
                } else {
                    Vec2::ZERO
                }
            } else if let Some(n) = hound_node {
                n.visual_pos
            } else {
                Vec2::ZERO
            };

            let pos = origin + visual_pos * scale;
            let hound_r = 24.0 * scale;

            // Selected Halo / Active turn glow
            if is_selected {
                draw_circle(
                    pos.x,
                    pos.y,
                    hound_r + 8.0 * scale + pulse * 4.0 * scale,
                    Color::from_rgba(30, 136, 229, 140),
                );
                draw_circle_lines(
                    pos.x,
                    pos.y,
                    hound_r + 6.0 * scale,
                    3.0 * scale,
                    Color::from_rgba(100, 181, 246, 255),
                );
            } else if state.current_turn == Faction::Hounds
                && state.player_faction == Faction::Hounds
            {
                draw_circle(
                    pos.x,
                    pos.y,
                    hound_r + 4.0 * scale + pulse * 2.0 * scale,
                    Color::from_rgba(33, 150, 243, 80),
                );
            }

            // Outer Hound Token Body
            draw_circle(
                pos.x,
                pos.y + 2.0 * scale,
                hound_r,
                Color::from_rgba(10, 25, 47, 180),
            ); // Shadow
            draw_circle(pos.x, pos.y, hound_r, Color::from_rgba(25, 118, 210, 255));
            draw_circle(
                pos.x,
                pos.y,
                hound_r - 3.0 * scale,
                Color::from_rgba(66, 165, 245, 255),
            );
            draw_circle_lines(
                pos.x,
                pos.y,
                hound_r,
                2.0 * scale,
                Color::from_rgba(187, 222, 251, 255),
            );

            // Hound Icon
            let icon_text = "🐶";
            let font_size = (24.0 * scale) as u16;
            let text_dims = measure_text(icon_text, None, font_size, 1.0);
            draw_text(
                icon_text,
                pos.x - text_dims.width / 2.0,
                pos.y + text_dims.height / 3.0,
                font_size as f32,
                WHITE,
            );
        }

        // 3. Draw Fox
        let fox_node = state.graph.node(state.fox_pos);
        let visual_pos = if let Some(anim) = &state.active_anim {
            if anim.faction == Faction::Fox {
                let ease = 1.0 - (1.0 - anim.progress).powi(2);
                anim.from.lerp(anim.to, ease)
            } else if let Some(n) = fox_node {
                n.visual_pos
            } else {
                Vec2::ZERO
            }
        } else if let Some(n) = fox_node {
            n.visual_pos
        } else {
            Vec2::ZERO
        };

        let pos = origin + visual_pos * scale;
        let fox_r = 25.0 * scale;

        // Active Turn Glow for Fox
        if state.current_turn == Faction::Fox {
            draw_circle(
                pos.x,
                pos.y,
                fox_r + 7.0 * scale + pulse * 4.0 * scale,
                Color::from_rgba(255, 112, 67, 130),
            );
            draw_circle_lines(
                pos.x,
                pos.y,
                fox_r + 5.0 * scale,
                2.5 * scale,
                Color::from_rgba(255, 171, 145, 255),
            );
        }

        // Fox Token Body
        draw_circle(
            pos.x,
            pos.y + 2.0 * scale,
            fox_r,
            Color::from_rgba(30, 10, 0, 180),
        ); // Shadow
        draw_circle(pos.x, pos.y, fox_r, Color::from_rgba(230, 81, 0, 255));
        draw_circle(
            pos.x,
            pos.y,
            fox_r - 3.0 * scale,
            Color::from_rgba(245, 124, 0, 255),
        );
        draw_circle_lines(
            pos.x,
            pos.y,
            fox_r,
            2.0 * scale,
            Color::from_rgba(255, 204, 128, 255),
        );

        // Fox Icon
        let icon_text = "🦊";
        let font_size = (26.0 * scale) as u16;
        let text_dims = measure_text(icon_text, None, font_size, 1.0);
        draw_text(
            icon_text,
            pos.x - text_dims.width / 2.0,
            pos.y + text_dims.height / 3.0,
            font_size as f32,
            WHITE,
        );
    }
}
