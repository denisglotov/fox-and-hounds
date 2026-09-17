use crate::audio::{SoundManager, SoundTrigger};
use crate::game::graph::NodeType;
use crate::game::level::{BoardIntroFraming, BoardVariant, VARIANT_COUNT};
use crate::game::state::{Faction, GamePhase, GameResult, GameState, PieceMove};
use crate::ui::boat::BoatSimulation;
use crate::ui::river::{RiverPath, RiverSimulation};
use crate::ui::rover::RoverSimulation;
use crate::ui::train::TrainSimulation;
use crate::ui::{draw_text_styled, measure_text_styled};
use macroquad::prelude::*;

pub use crate::game::level::{
    BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH, BOARD_LEFT_WIDTH, BOARD_RIGHT_WIDTH, BOARD_TOTAL_WIDTH,
    DEFAULT_PIECE_BASE_SIZE,
};

pub const MIN_IDLE_SIT_SECONDS: f32 = 10.0;
pub const RANDOM_IDLE_SIT_SECONDS_RANGE: f32 = 5.0;

/// Number of small arrows of the start-of-match objective reticle.
pub const START_TARGET_ARROW_COUNT: usize = 6;

/// How long the Fox player may sit on their turn before the objective reticle fades
/// back in as a reminder. Deliberately the same wait as a hound settling down to sit.
pub const FOX_OBJECTIVE_HINT_IDLE_SECONDS: f32 = 10.0;

/// Board-space font size of the start-of-match special rule notice, before width fitting.
pub const SPECIAL_RULE_NOTICE_BASE_FONT_SIZE: f32 = 18.0;

/// Smallest font size the special rule notice may shrink to when a translation runs long.
pub const SPECIAL_RULE_NOTICE_MIN_FONT_SIZE: u16 = 11;

/// Share of the framed field's width the special rule notice plate may occupy.
pub const SPECIAL_RULE_NOTICE_MAX_WIDTH_RATIO: f32 = 0.92;

/// Padding between the special rule notice text and its backing plate, in board units.
pub const SPECIAL_RULE_NOTICE_PLATE_PADDING: f32 = 6.0;

/// Clearance kept between the lowest piece and the special rule notice, in board units: hounds
/// breathe, sway and bark on the spot, so the notice clears a little more than their
/// sprite box.
pub const SPECIAL_RULE_NOTICE_PIECE_CLEARANCE: f32 = 4.0;

/// Ease rate of the special rule notice fading in with the match and out on the opening move.
pub const SPECIAL_RULE_NOTICE_FADE_RATE: f32 = 4.0;

/// How long the special rule notice remains on screen after the user clicks on a spot
/// behind a hound on boards where hounds cannot retreat.
pub const SPECIAL_RULE_REMINDER_DURATION: f32 = 2.5;

pub fn roll_sit_threshold() -> f32 {
    MIN_IDLE_SIT_SECONDS + macroquad::rand::gen_range(0.0, RANDOM_IDLE_SIT_SECONDS_RANGE)
}

/// True while the Fox player owns the move and the board has settled, i.e. the only
/// window in which the objective reticle is offered. `active_anim.is_none()` keeps it
/// from popping up while a piece is still gliding, and `!is_ai_turn()` covers the
/// phase/result/turn checks the same way the input path does.
fn fox_player_waiting(state: &GameState) -> bool {
    state.phase == GamePhase::Playing
        && state.player_faction == Faction::Fox
        && !state.is_ai_turn()
        && state.active_anim.is_none()
}

/// The objective reticle marks the Fox destination (the coop) so the Fox player can read
/// their goal. It shows for the very first decision of a match, and comes back whenever
/// the Fox player has been idling on a later turn for `FOX_OBJECTIVE_HINT_IDLE_SECONDS`.
/// It is a Fox-player aid, so it stays hidden while the hounds are to move.
pub fn should_highlight_fox_objective(state: &GameState, fox_idle_seconds: f32) -> bool {
    fox_player_waiting(state)
        && (state.move_history.is_empty() || fox_idle_seconds >= FOX_OBJECTIVE_HINT_IDLE_SECONDS)
}

/// The special rule notice announces a board's own rule as the match opens. Classic is the
/// board that has one today: its hounds may not fall back toward the coop
/// (`allow_hound_retreat == false`). Like the objective reticle it belongs to the opening
/// decision only: it leaves as soon as the player has played their first move.
pub fn should_show_special_rule_notice(state: &GameState) -> bool {
    state.phase == GamePhase::Playing
        && !state.variant.config().allow_hound_retreat
        && !player_has_moved(state)
}

/// Whether the special rule notice belongs on screen right now, given how long its
/// illegal-retreat reminder has left to run and whether the opening camera zoom has settled.
/// The announcement that opens a match waits for `intro_settled`: a plate riding a board that
/// is still flying into frame is unreadable, so it fades in once the framing has landed. A
/// reminder raised by an illegal retreat click is a direct answer to the player and shows
/// without waiting.
pub fn wants_special_rule_notice(
    state: &GameState,
    reminder_seconds: f32,
    intro_settled: bool,
) -> bool {
    state.phase == GamePhase::Playing
        && !state.variant.config().allow_hound_retreat
        && (reminder_seconds > 0.0 || (intro_settled && should_show_special_rule_notice(state)))
}

/// True when `clicked_node` sits in a row behind the player's hounds (toward the coop)
/// on boards where hounds cannot retreat.
pub fn is_hound_retreat_click(state: &GameState, clicked_node: u8) -> bool {
    if state.variant.config().allow_hound_retreat {
        return false;
    }
    let Some(clicked) = state.graph.node(clicked_node) else {
        return false;
    };
    if let Some(hound_idx) = state.selected_hound_idx {
        state
            .hounds_pos
            .get(hound_idx as usize)
            .and_then(|&p| state.graph.node(p))
            .is_some_and(|hound| clicked.row < hound.row)
    } else {
        state
            .hounds_pos
            .iter()
            .filter_map(|&p| state.graph.node(p))
            .any(|hound| clicked.row < hound.row)
    }
}

/// True once the faction the player controls has made a move of its own. Every board opens
/// with the Fox, so a Hounds player reads the notice until their own first hound move
/// instead of losing it to the AI's opening reply at match start.
fn player_has_moved(state: &GameState) -> bool {
    let player_is_fox = state.player_faction == Faction::Fox;
    state.move_history.iter().any(|mv| match mv {
        PieceMove::FoxMove { .. } => player_is_fox,
        PieceMove::HoundMove { .. } => !player_is_fox,
    })
}

/// Centre of the special rule notice plate: the middle of the framed field horizontally, and the
/// middle of the clear strip between the lowest board node (plus the piece standing on it)
/// and the bottom edge of that field vertically, so the sentence never covers a piece.
pub fn special_rule_notice_center(
    framing: BoardIntroFraming,
    lowest_node_y: f32,
    piece_base_size: f32,
) -> Vec2 {
    let field_bottom = framing.playable_center.y + framing.playable_size.y * 0.5;
    let piece_bottom = lowest_node_y + piece_base_size * 0.5 + SPECIAL_RULE_NOTICE_PIECE_CLEARANCE;
    // A board whose pieces already reach the field's edge keeps the notice on the field.
    Vec2::new(
        framing.playable_center.x,
        ((piece_bottom + field_bottom) * 0.5)
            .max(piece_bottom)
            .min(field_bottom),
    )
}

/// Shrinks the special rule notice font from `base_font_size` until `text_width` fits `max_width`,
/// so a long translation stays on the board instead of running off it. Never goes below
/// `SPECIAL_RULE_NOTICE_MIN_FONT_SIZE`, which keeps the sentence legible.
pub fn fit_special_rule_notice_font_size(
    base_font_size: u16,
    text_width: f32,
    max_width: f32,
) -> u16 {
    if text_width <= 0.0 || text_width <= max_width {
        return base_font_size;
    }
    let min_size = SPECIAL_RULE_NOTICE_MIN_FONT_SIZE.min(base_font_size);
    let fitted = f32::from(base_font_size) * max_width / text_width;
    fitted.clamp(f32::from(min_size), f32::from(base_font_size)) as u16
}

/// Vertices of one arrowhead of the objective reticle. `angle` is the outward
/// direction from `center`; the tip sits at `tip_radius` (nearer the center than
/// `base_radius`), so every arrow visually points at the highlighted node.
pub fn target_arrow_vertices(
    center: Vec2,
    angle: f32,
    tip_radius: f32,
    base_radius: f32,
    half_width: f32,
) -> (Vec2, Vec2, Vec2) {
    let dir = Vec2::new(angle.cos(), angle.sin());
    let perp = Vec2::new(-angle.sin(), angle.cos());
    let base = center + dir * base_radius;
    (
        center + dir * tip_radius,
        base + perp * half_width,
        base - perp * half_width,
    )
}

/// Exponential ease of a fade weight, matching the smoothing used for piece rotation.
fn approach_fade(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    current + (target - current) * (1.0 - (-rate * dt).exp())
}

/// Tint of the start-of-match objective reticle. White reads on the meadow boards and
/// on the rust-coloured Martian crust alike, so a single tint covers every board; give
/// a board its own colour here if the artwork ever calls for it.
const START_TARGET_TINT: Color = WHITE;

/// Contrast backing of the reticle: a darker copy drawn just behind the marker keeps
/// the bright tint legible on pale artwork (the hand-drawn sketch board) as well.
const START_TARGET_OUTLINE: Color = Color::new(0.05, 0.06, 0.10, 1.0);

/// Tint of the special rule notice text, matching the contrast backing of the reticle.
const SPECIAL_RULE_NOTICE_TINT: Color = Color::new(0.96, 0.97, 1.0, 1.0);

/// Plate behind the special rule notice: the same dark glass as the in-game HUD pills, so the
/// sentence reads on every board artwork.
const SPECIAL_RULE_NOTICE_PLATE: Color = Color::new(0.04, 0.06, 0.10, 1.0);

/// Scales a colour's alpha so the reticle and its backing fade out together.
fn with_alpha(color: Color, factor: f32) -> Color {
    Color::new(
        color.r,
        color.g,
        color.b,
        (color.a * factor).clamp(0.0, 1.0),
    )
}

fn load_texture(bytes: &[u8]) -> Option<Texture2D> {
    let tex = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    tex.set_filter(FilterMode::Linear);
    Some(tex)
}

pub struct BoardView {
    pub board_textures: [Option<Texture2D>; VARIANT_COUNT],
    pub board_texture: Option<Texture2D>,
    pub current_variant: Option<BoardVariant>,
    pub fox_texture: Option<Texture2D>,
    pub hound_textures: [Option<Texture2D>; 3],
    pub hound_sit_textures: [Option<Texture2D>; 3],
    pub martian_fox_texture: Option<Texture2D>,
    pub martian_hound_textures: [Option<Texture2D>; 3],
    pub martian_hound_sit_textures: [Option<Texture2D>; 3],
    pub train_texture: Option<Texture2D>,
    pub rover_texture: Option<Texture2D>,
    pub boat_texture: Option<Texture2D>,
    pub bridge_texture: Option<Texture2D>,
    pub no_reverse_dog_texture: Option<Texture2D>,
    pub hound_angles: [f32; 3],
    pub fox_angle: f32,
    pub hover_node_id: Option<u8>,
    pub font: Option<Font>,
    pub river: RiverSimulation,
    pub train: TrainSimulation,
    pub rover: RoverSimulation,
    pub boat: BoatSimulation,
    pub last_waf_sound_time: f32,
    pub hound_idle_times: [f32; 3],
    pub hound_sit_thresholds: [f32; 3],
    pub hound_sit_blend: [f32; 3],
    /// Fade weight (0..1) of the objective reticle for the Fox player.
    pub start_target_alpha: f32,
    /// Fade weight (0..1) of the start-of-match special rule notice on boards that forbid the
    /// hounds to fall back toward the coop.
    pub special_rule_notice_alpha: f32,
    /// Seconds remaining to display the special rule notice when triggered as an illegal retreat reminder.
    pub special_rule_reminder_seconds: f32,
    /// Seconds the Fox player has been sitting on their turn without acting, used to
    /// bring the objective reticle back as a reminder.
    pub fox_idle_seconds: f32,
}

fn lerp_angle(current: f32, target: f32, speed: f32, dt: f32) -> f32 {
    let diff = (target - current + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    current + diff * (1.0 - (-speed * dt).exp())
}

pub struct BoardViewParams<'a> {
    pub origin: Vec2,
    pub scale: f32,
    pub viewport_mouse_pos: Vec2,
    /// Size of the render viewport in screen pixels. Screen-sized notices use it to stay
    /// inside the visible area even when the board is wider than the window.
    pub viewport_size: Vec2,
    pub was_dragging: bool,
    /// True once the opening camera zoom has settled. The special rule notice waits for it so
    /// the sentence is not thrown at the player while the board is still flying into frame.
    pub intro_settled: bool,
    pub sound_manager: &'a SoundManager,
    pub dt: f32,
}

impl BoardView {
    pub async fn new(font: Option<Font>) -> Self {
        const NO_TEX: Option<Texture2D> = None;
        let board_textures = [NO_TEX; VARIANT_COUNT];
        let current_variant = None;
        let board_texture = None;
        let fox_texture = load_texture(include_bytes!("../../assets/fox_figure.png"));
        let hound_textures = [
            load_texture(include_bytes!("../../assets/hound1_figure.png")),
            load_texture(include_bytes!("../../assets/hound2_figure.png")),
            load_texture(include_bytes!("../../assets/hound3_figure.png")),
        ];
        let hound_sit_textures = [
            load_texture(include_bytes!("../../assets/hound1_figure_sit.png")),
            load_texture(include_bytes!("../../assets/hound2_figure_sit.png")),
            load_texture(include_bytes!("../../assets/hound3_figure_sit.png")),
        ];
        let martian_fox_texture =
            load_texture(include_bytes!("../../assets/martian_fox_figure.png"));
        let martian_hound_textures = [
            load_texture(include_bytes!("../../assets/martian_hound1_figure.png")),
            load_texture(include_bytes!("../../assets/martian_hound2_figure.png")),
            load_texture(include_bytes!("../../assets/martian_hound3_figure.png")),
        ];
        let martian_hound_sit_textures = [
            load_texture(include_bytes!("../../assets/martian_hound1_figure_sit.png")),
            load_texture(include_bytes!("../../assets/martian_hound2_figure_sit.png")),
            load_texture(include_bytes!("../../assets/martian_hound3_figure_sit.png")),
        ];
        let train_texture = load_texture(include_bytes!("../../assets/train_figure.png"));
        let rover_texture = load_texture(include_bytes!("../../assets/rover_curiosity.png"));
        let boat_texture = load_texture(include_bytes!("../../assets/paper_boat.png"));
        let bridge_texture = load_texture(include_bytes!("../../assets/fox_and_dogs_bridge.png"));
        let no_reverse_dog_texture =
            load_texture(include_bytes!("../../assets/no_reverse_dog.png"));

        Self {
            board_textures,
            board_texture,
            current_variant,
            fox_texture,
            hound_textures,
            hound_sit_textures,
            martian_fox_texture,
            martian_hound_textures,
            martian_hound_sit_textures,
            train_texture,
            rover_texture,
            boat_texture,
            bridge_texture,
            no_reverse_dog_texture,
            hound_angles: [0.0; 3],
            fox_angle: 0.0,
            hover_node_id: None,
            font,
            river: RiverSimulation::for_variant(BoardVariant::Classic),
            train: TrainSimulation::new(),
            rover: RoverSimulation::new(),
            boat: BoatSimulation::new(),
            last_waf_sound_time: 0.0,
            hound_idle_times: [0.0; 3],
            hound_sit_thresholds: [
                roll_sit_threshold(),
                roll_sit_threshold(),
                roll_sit_threshold(),
            ],
            hound_sit_blend: [0.0; 3],
            start_target_alpha: 0.0,
            special_rule_notice_alpha: 0.0,
            special_rule_reminder_seconds: 0.0,
            fox_idle_seconds: 0.0,
        }
    }

    pub fn is_hound_sitting(&self, hound_idx: usize) -> bool {
        hound_idx < self.hound_idle_times.len()
            && self.hound_idle_times[hound_idx] >= self.hound_sit_thresholds[hound_idx]
    }

    pub fn active_hound_texture(
        &self,
        variant: BoardVariant,
        idx: usize,
        is_sitting: bool,
    ) -> Option<&Texture2D> {
        if variant.is_red_hunt() {
            if is_sitting {
                self.martian_hound_sit_textures
                    .get(idx)
                    .and_then(|t| t.as_ref())
            } else {
                self.martian_hound_textures
                    .get(idx)
                    .and_then(|t| t.as_ref())
            }
        } else if is_sitting {
            self.hound_sit_textures.get(idx).and_then(|t| t.as_ref())
        } else {
            self.hound_textures.get(idx).and_then(|t| t.as_ref())
        }
    }

    pub fn active_fox_texture(&self, variant: BoardVariant) -> Option<&Texture2D> {
        if variant.is_red_hunt() {
            self.martian_fox_texture.as_ref()
        } else {
            self.fox_texture.as_ref()
        }
    }

    pub fn update_hound_idle(&mut self, hound_idx: usize, is_active: bool, dt: f32) {
        if hound_idx >= self.hound_idle_times.len() {
            return;
        }
        if is_active {
            self.hound_idle_times[hound_idx] = 0.0;
            self.hound_sit_thresholds[hound_idx] = roll_sit_threshold();
        } else {
            self.hound_idle_times[hound_idx] += dt;
        }
    }

    pub async fn ensure_board_loaded(&mut self, variant: BoardVariant) {
        let idx = variant.index();
        if self.board_textures[idx].is_none() {
            let filename = variant.config().board_image_filename;
            match macroquad::texture::load_texture(filename).await {
                Ok(tex) => {
                    tex.set_filter(FilterMode::Linear);
                    self.board_textures[idx] = Some(tex);
                }
                Err(err) => {
                    eprintln!("Failed to load board texture '{filename}': {err:?}");
                }
            }
        }
        self.current_variant = Some(variant);
        self.board_texture = self.board_textures[idx].clone();
        self.river.set_path(RiverPath::for_variant(variant));
    }

    pub fn reset_simulations(&mut self) {
        self.train = TrainSimulation::new();
        self.rover = RoverSimulation::new();
        self.boat = BoatSimulation::new();
        self.last_waf_sound_time = 0.0;
        self.start_target_alpha = 0.0;
        self.special_rule_notice_alpha = 0.0;
        self.special_rule_reminder_seconds = 0.0;
        self.fox_idle_seconds = 0.0;
    }

    /// Triggers the start-of-match special rule notice to show again as a reminder
    /// (e.g. when the player clicks on a spot behind their dog on a no-retreat board).
    pub fn trigger_special_rule_reminder(&mut self) {
        self.special_rule_reminder_seconds = SPECIAL_RULE_REMINDER_DURATION;
    }

    /// Advances the Fox idle timer that brings the objective reticle back as a reminder.
    /// It only accrues while `fox_player_waiting`, so the hounds' turn, a piece still
    /// gliding and the game-over screen all hold it at zero.
    pub fn update_fox_idle(&mut self, state: &GameState, dt: f32) {
        if fox_player_waiting(state) {
            self.fox_idle_seconds += dt.max(0.0);
        } else {
            self.fox_idle_seconds = 0.0;
        }
    }

    pub fn draw_and_handle_input(&mut self, state: &mut GameState, params: &BoardViewParams) {
        let origin = params.origin;
        let scale = params.scale;
        let viewport_mouse_pos = params.viewport_mouse_pos;
        let was_dragging = params.was_dragging;
        let sound_manager = params.sound_manager;
        let dt = params.dt;
        let t = get_time() as f32;

        // Switch board texture and river path if variant changed
        if self.current_variant != Some(state.variant) {
            self.current_variant = Some(state.variant);
            self.board_texture = self.board_textures[state.variant.index()].clone();
            self.river.set_path(RiverPath::for_variant(state.variant));
            self.reset_simulations();
        }

        let dims = state.variant.config().dimensions;

        // 1. Draw Background Board Image (exact natural 1:1 proportions, no distortion)
        if let Some(tex) = &self.board_texture {
            draw_texture_ex(
                tex,
                origin.x - dims.left_width * scale,
                origin.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        dims.total_width() * scale,
                        dims.image_height * scale,
                    )),
                    ..Default::default()
                },
            );
        } else {
            // Fallback dark board container
            draw_rectangle(
                origin.x - dims.left_width * scale,
                origin.y,
                dims.total_width() * scale,
                dims.image_height * scale,
                Color::from_rgba(18, 28, 42, 255),
            );
        }

        // 2. Update & Draw Floating Water in River
        self.river.update(dt);
        self.river.draw(origin, scale);

        // 3. Update & Draw Train on the railway tracks (only for River Crossing)
        if state.variant == BoardVariant::RiverCrossing {
            if let Some(snd) = self.train.update(dt) {
                sound_manager.play(snd);
            }
            self.train.draw(origin, scale, self.train_texture.as_ref());
        }

        // 4. Update & Draw Curiosity Mars Rover (only for The Red Hunt)
        if state.variant == BoardVariant::TheRedHunt {
            if let Some(snd) = self.rover.update(dt) {
                sound_manager.play(snd);
            }
            self.rover.draw(origin, scale, self.rover_texture.as_ref());
        }

        // 5. Update & Draw Paper Boat (only for Fox and Dogs; omitted on FoxAndDogsMaze
        // to preserve the clean hand-drawn sketch illustration aesthetic)
        if state.variant == BoardVariant::FoxAndDogs {
            self.boat.update(dt);
            self.boat
                .draw(origin, scale, &self.river.path, self.boat_texture.as_ref());

            // 6. Draw wooden bridge overlay on top of river and boat so it slides realistically beneath it
            if let Some(bridge_tex) = &self.bridge_texture {
                draw_texture_ex(
                    bridge_tex,
                    origin.x + 464.0 * scale,
                    origin.y + 336.0 * scale,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(88.0 * scale, 144.0 * scale)),
                        ..Default::default()
                    },
                );
            }
        }

        // 4. Find hovered node & Determine Legal Targets for Player
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
                Faction::Hounds => state
                    .selected_hound_idx
                    .map_or_else(Vec::new, |h_idx| state.hound_legal_moves(h_idx)),
            }
        } else {
            Vec::new()
        };

        // 4. Track how long the Fox player has been sitting on their turn, then ease the
        // objective hint (Fox player only) in and out: it opens the match and returns as
        // a reminder whenever the Fox player idles on a later turn
        self.update_fox_idle(state, dt);
        let wants_objective_hint = should_highlight_fox_objective(state, self.fox_idle_seconds);
        self.start_target_alpha = approach_fade(
            self.start_target_alpha,
            if wants_objective_hint { 1.0 } else { 0.0 },
            if wants_objective_hint { 2.2 } else { 5.0 },
            dt,
        );

        // 4b. The special rule notice of boards that forbid the hounds to fall back opens the match
        // once the opening camera zoom has settled (a plate riding the flying board would be
        // unreadable), and eases out again the moment the player has played their opening move,
        // or resurfaces as a reminder when the user attempts an illegal retreat move.
        self.special_rule_reminder_seconds = (self.special_rule_reminder_seconds - dt).max(0.0);
        let show_special_rule_notice = wants_special_rule_notice(
            state,
            self.special_rule_reminder_seconds,
            params.intro_settled,
        );
        self.special_rule_notice_alpha = approach_fade(
            self.special_rule_notice_alpha,
            if show_special_rule_notice { 1.0 } else { 0.0 },
            SPECIAL_RULE_NOTICE_FADE_RATE,
            dt,
        );

        // 5. Draw Graph Nodes & Interactive Indicators
        self.draw_nodes(state, origin, scale, &legal_destinations, t);

        // 6. Draw the objective reticle pointing at the Fox destination
        self.draw_start_target_marker(state, origin, scale, t);

        // 7. Draw Animated Live Characters (Fox & 3 Unique Hounds) & play waffing sound
        if let Some(snd) = self.draw_pieces(state, origin, scale, t, dt) {
            sound_manager.play(snd);
        }

        // 8. Draw the special rule notice of boards that forbid the hounds to fall back
        self.draw_special_rule_notice(state, origin, scale, params.viewport_size);

        // 9. Handle Player Tap / Click Input (only if not dragging)
        if is_mouse_button_released(MouseButton::Left)
            && !was_dragging
            && state.phase == GamePhase::Playing
            && !state.is_ai_turn()
            && state.result == GameResult::Ongoing
        {
            if let Some(clicked_node) = self.hover_node_id {
                if let Some(snd) = self.handle_node_click(state, clicked_node) {
                    sound_manager.play(snd);
                }
            }
        }
    }

    fn handle_node_click(
        &mut self,
        state: &mut GameState,
        clicked_node: u8,
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
                    state.selected_hound_idx = Some(hound_idx as u8);
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
                        if is_hound_retreat_click(state, clicked_node) {
                            self.trigger_special_rule_reminder();
                        }
                        Some(SoundTrigger::InvalidMove)
                    }
                } else {
                    if is_hound_retreat_click(state, clicked_node) {
                        self.trigger_special_rule_reminder();
                    }
                    Some(SoundTrigger::InvalidMove)
                }
            }
        }
    }

    /// Draws a targeting reticle of small arrows converging on the Fox's destination
    /// (the coop) for a Fox player: the arrows orbit the spot, slide inwards and point at
    /// it, while a soft glow marks it as the hot spot. It opens a match and fades back in
    /// whenever the Fox player has been idling on a later turn (see
    /// `should_highlight_fox_objective`).
    fn draw_start_target_marker(&self, state: &GameState, origin: Vec2, scale: f32, t: f32) {
        let alpha = self.start_target_alpha;
        if alpha <= 0.01 {
            return;
        }
        let Some(target) = state.graph.node(state.active_target_node()) else {
            return;
        };
        let center = origin + target.visual_pos * scale;
        let breathing = (t * 2.6).sin() * 0.5 + 0.5;

        // Soft glow so the destination reads as the hot spot on the board
        draw_circle(
            center.x,
            center.y,
            (24.0 + breathing * 5.0) * scale,
            with_alpha(START_TARGET_TINT, alpha * 0.16),
        );

        // Reticle ring hugging the objective, backed by a darker copy for contrast
        let ring_radius = (34.0 + breathing * 3.0) * scale;
        draw_circle_lines(
            center.x,
            center.y,
            ring_radius,
            4.2 * scale,
            with_alpha(START_TARGET_OUTLINE, alpha * 0.32),
        );
        draw_circle_lines(
            center.x,
            center.y,
            ring_radius,
            2.2 * scale,
            with_alpha(START_TARGET_TINT, alpha * 0.95),
        );

        // Arrows slowly orbit while sliding inwards, then fade out and repeat
        let spin = t * 0.45;
        let step = std::f32::consts::TAU / START_TARGET_ARROW_COUNT as f32;
        for i in 0..START_TARGET_ARROW_COUNT {
            let angle = spin + i as f32 * step;
            let flow = (t * 0.85 + i as f32 * 0.19).rem_euclid(1.0);
            let base_radius = ring_radius + (1.0 - flow) * 15.0 * scale;
            let half_width = (5.5 - flow * 1.5) * scale;
            let tip_radius = base_radius - 11.0 * scale;
            let fade = alpha * (1.0 - flow * 0.45);

            // Darker backing, a hair larger, keeps the bright arrow legible anywhere
            let (back_tip, back_left, back_right) = target_arrow_vertices(
                center,
                angle,
                tip_radius - 1.6 * scale,
                base_radius + 1.6 * scale,
                half_width + 1.6 * scale,
            );
            draw_triangle(
                back_tip,
                back_left,
                back_right,
                with_alpha(START_TARGET_OUTLINE, fade * 0.45),
            );

            let (tip, left, right) =
                target_arrow_vertices(center, angle, tip_radius, base_radius, half_width);
            draw_triangle(tip, left, right, with_alpha(START_TARGET_TINT, fade));
        }
    }

    /// Draws the start-of-match special rule notice ("hounds cannot retreat on this board")
    /// for boards that forbid the hounds to fall back toward the coop. Like the objective
    /// reticle it opens the match and eases out once the player has played their opening move,
    /// but only after the intro zoom has settled (see `wants_special_rule_notice`); the font
    /// shrinks for long translations so the sentence always stays on the framed field (or on
    /// the viewport, whichever is narrower), and a dark plate keeps it readable on any artwork.
    fn draw_special_rule_notice(
        &self,
        state: &GameState,
        origin: Vec2,
        scale: f32,
        viewport_size: Vec2,
    ) {
        let alpha = self.special_rule_notice_alpha;
        if alpha <= 0.01 {
            return;
        }

        let config = state.variant.config();
        let text = state.locales.hud.special_rule_notice.as_str();
        let font = self.font.as_ref();

        let icon_tex = self.no_reverse_dog_texture.as_ref();
        let icon_size = (SPECIAL_RULE_NOTICE_BASE_FONT_SIZE * 1.75 * scale).round();
        let icon_gap = if icon_tex.is_some() { 8.0 * scale } else { 0.0 };
        let icon_extra_w = if icon_tex.is_some() {
            icon_size + icon_gap
        } else {
            0.0
        };

        // Measure the sentence at the base size, then shrink it until the plate fits the
        // board's framed field, or the visible width on boards wider than the window
        let base_size = (SPECIAL_RULE_NOTICE_BASE_FONT_SIZE * scale)
            .round()
            .max(1.0) as u16;
        let base_width = measure_text_styled(text, base_size, font).width;
        let field_width = config.intro_framing.playable_size.x * scale;
        let max_width = (field_width.min(viewport_size.x) * SPECIAL_RULE_NOTICE_MAX_WIDTH_RATIO
            - 2.0 * SPECIAL_RULE_NOTICE_PLATE_PADDING * scale
            - icon_extra_w)
            .max(10.0);
        let font_size = fit_special_rule_notice_font_size(base_size, base_width, max_width);
        let text_dims = measure_text_styled(text, font_size, font);

        let lowest_node_y = state
            .graph
            .nodes
            .iter()
            .fold(f32::MIN, |lowest, node| lowest.max(node.visual_pos.y));
        let center = origin
            + special_rule_notice_center(
                config.intro_framing,
                lowest_node_y,
                config.piece_base_size,
            ) * scale;

        let pad = SPECIAL_RULE_NOTICE_PLATE_PADDING * scale;
        let content_w = text_dims.width + icon_extra_w;
        let content_h = text_dims
            .height
            .max(if icon_tex.is_some() { icon_size } else { 0.0 });
        let plate_w = content_w + pad * 2.0;
        let plate_h = content_h + pad * 2.0;
        let plate_x = center.x - plate_w / 2.0;
        let plate_y = center.y - plate_h / 2.0;

        draw_rectangle(
            plate_x,
            plate_y,
            plate_w,
            plate_h,
            with_alpha(SPECIAL_RULE_NOTICE_PLATE, alpha * 0.50),
        );
        draw_rectangle_lines(
            plate_x,
            plate_y,
            plate_w,
            plate_h,
            1.2 * scale,
            with_alpha(SPECIAL_RULE_NOTICE_TINT, alpha * 0.28),
        );

        let mut content_x = plate_x + pad;
        if let Some(tex) = icon_tex {
            let icon_y = center.y - icon_size / 2.0;
            draw_texture_ex(
                tex,
                content_x,
                icon_y,
                with_alpha(WHITE, alpha),
                DrawTextureParams {
                    dest_size: Some(Vec2::new(icon_size, icon_size)),
                    ..Default::default()
                },
            );
            content_x += icon_size + icon_gap;
        }

        draw_text_styled(
            text,
            content_x,
            center.y + text_dims.height / 3.0,
            font_size,
            with_alpha(SPECIAL_RULE_NOTICE_TINT, alpha),
            font,
        );
    }

    fn draw_nodes(
        &self,
        state: &GameState,
        origin: Vec2,
        scale: f32,
        legal_destinations: &[u8],
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

            // 2. Base Node Circle Plate (skip for the chicken coop in River Crossing)
            if state.variant != BoardVariant::RiverCrossing || node.id != state.coop_pos {
                let base_radius = if is_hovered {
                    20.0 * scale
                } else {
                    16.0 * scale
                };
                let base_color = if node.node_type == NodeType::Bottleneck {
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

            // 3. Active Target Objective Halo
            let is_active_target = node.id == state.active_target_node();
            if is_active_target && !is_legal && state.phase == GamePhase::Playing {
                let target_radius = (22.0 + pulse * 3.0) * scale;
                draw_circle_lines(
                    pos.x,
                    pos.y,
                    target_radius,
                    2.0 * scale,
                    Color::from_rgba(255, 215, 64, 140 + (pulse * 80.0) as u8),
                );
            }
        }
    }

    fn draw_pieces(
        &mut self,
        state: &GameState,
        origin: Vec2,
        scale: f32,
        t: f32,
        dt: f32,
    ) -> Option<SoundTrigger> {
        let pulse = (t * 3.5).sin() * 0.5 + 0.5;
        let mut waf_sound = None;

        // 1. Calculate live visual position of Fox for Hound tracking
        let fox_node = state.graph.node(state.fox_pos);
        let is_fox_moving = state
            .active_anim
            .as_ref()
            .is_some_and(|anim| anim.faction == Faction::Fox);

        let (fox_visual_pos, fox_jump_lift, fox_target_rot) = if is_fox_moving {
            let anim = state.active_anim.as_ref().unwrap();
            let ease = 1.0 - (1.0 - anim.progress).powi(2);
            let pos = anim.from.lerp(anim.to, ease);
            let jump = (anim.progress * std::f32::consts::PI).sin() * 26.0;

            // Dynamic rotation along Fox movement trajectory
            let delta = anim.to - anim.from;
            let angle = (delta.y).atan2(delta.x) + std::f32::consts::FRAC_PI_2;
            (pos, jump, angle)
        } else if let Some(n) = fox_node {
            // While idle, orient towards active objective target
            let target_pos = state
                .graph
                .node(state.active_target_node())
                .map(|cn| cn.visual_pos)
                .unwrap_or(Vec2::ZERO);
            let delta = target_pos - n.visual_pos;
            let angle = if delta.length_squared() > 1e-4 {
                (delta.y).atan2(delta.x) + std::f32::consts::FRAC_PI_2
            } else {
                0.0
            };
            (n.visual_pos, 0.0, angle)
        } else {
            (Vec2::ZERO, 0.0, 0.0)
        };

        // Smoothly interpolate Fox rotation angle
        self.fox_angle = lerp_angle(self.fox_angle, fox_target_rot, 10.0, dt);

        // 2. Draw Hounds (Left: Terrier/User's white dog, Mid: Beagle, Right: Golden)
        for (idx, &hound_pos) in state.hounds_pos.iter().enumerate() {
            if idx >= self.hound_angles.len() {
                break;
            }
            let is_selected = state.selected_hound_idx == Some(idx as u8);
            let hound_node = state.graph.node(hound_pos);

            let is_moving = state.active_anim.as_ref().is_some_and(|anim| {
                anim.faction == Faction::Hounds && anim.hound_idx == Some(idx as u8)
            });

            // Hound 1 (idx 0) reacts to environmental hazards:
            // - Train on tracks in River Crossing
            // - Curiosity rover stopped near golden crater in The Red Hunt
            let is_hazard_tracking = idx == 0
                && ((state.variant == BoardVariant::RiverCrossing && self.train.is_active())
                    || (state.variant == BoardVariant::TheRedHunt && self.rover.is_active()));

            let is_waffing = idx == 0
                && !is_moving
                && ((state.variant == BoardVariant::RiverCrossing && self.train.is_active())
                    || (state.variant == BoardVariant::TheRedHunt && self.rover.is_near_crater()));

            if is_waffing {
                if self.last_waf_sound_time == 0.0 {
                    // First enthusiastic bark shortly after hazard appears
                    self.last_waf_sound_time = t + 0.35;
                } else if t >= self.last_waf_sound_time {
                    self.last_waf_sound_time = t + 0.75;
                    waf_sound = Some(SoundTrigger::InvalidMove);
                }
            } else if idx == 0 {
                self.last_waf_sound_time = 0.0;
            }

            let (visual_pos, jump_lift, target_hound_rot) = if is_moving {
                let anim = state.active_anim.as_ref().unwrap();
                let ease = 1.0 - (1.0 - anim.progress).powi(2);
                let pos = anim.from.lerp(anim.to, ease);
                let jump = (anim.progress * std::f32::consts::PI).sin() * 24.0;

                // Face movement path during leap
                let delta = anim.to - anim.from;
                let angle = (delta.y).atan2(delta.x) - std::f32::consts::FRAC_PI_2;
                (pos, jump, angle)
            } else if let Some(n) = hound_node {
                // If Hound 1 (idx 0) and hazard is active: Face directly towards it!
                let angle = if idx == 0
                    && state.variant == BoardVariant::RiverCrossing
                    && self.train.is_active()
                {
                    let train_loco_y = self.train.train_locomotive_y().unwrap_or(0.0);
                    let to_train =
                        Vec2::new(crate::ui::train::TRACK_X, train_loco_y) - n.visual_pos;
                    if to_train.length_squared() > 1e-4 {
                        (to_train.y).atan2(to_train.x) - std::f32::consts::FRAC_PI_2
                    } else {
                        0.0
                    }
                } else if idx == 0
                    && state.variant == BoardVariant::TheRedHunt
                    && self.rover.is_active()
                {
                    let rover_pos = self.rover.rover_pos().unwrap_or(Vec2::new(
                        crate::ui::rover::ROVER_X,
                        crate::ui::rover::ROVER_CRATER_Y,
                    ));
                    let to_rover = rover_pos - n.visual_pos;
                    if to_rover.length_squared() > 1e-4 {
                        (to_rover.y).atan2(to_rover.x) - std::f32::consts::FRAC_PI_2
                    } else {
                        0.0
                    }
                } else {
                    // When stationary/idle: Face directly towards the Fox's live position!
                    let to_fox = fox_visual_pos - n.visual_pos;
                    if to_fox.length_squared() > 1e-4 {
                        (to_fox.y).atan2(to_fox.x) - std::f32::consts::FRAC_PI_2
                    } else {
                        0.0
                    }
                };
                (n.visual_pos, 0.0, angle)
            } else {
                (Vec2::ZERO, 0.0, 0.0)
            };

            // Smoothly interpolate Hound angle towards target (faster during leap for crisp orientation)
            let rot_speed = if is_moving {
                24.0
            } else if is_waffing || is_hazard_tracking {
                16.0
            } else {
                12.0
            };
            self.hound_angles[idx] =
                lerp_angle(self.hound_angles[idx], target_hound_rot, rot_speed, dt);

            let ground_pos = origin + visual_pos * scale;
            let pick_up_lift = if is_selected { 8.0 * scale } else { 0.0 };

            // Barking / waffing bounce and excited tail-wag jitter
            let bark_jitter = if is_waffing {
                (t * 22.0).sin() * 0.10
            } else {
                0.0
            };
            let bark_lift = if is_waffing {
                (t * 14.0).sin().abs() * 3.5 * scale
            } else {
                0.0
            };

            let render_pos = Vec2::new(
                ground_pos.x,
                ground_pos.y - (jump_lift * scale) - pick_up_lift - bark_lift,
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

            // Track hound idleness:
            // When moving, selected, waffing, or tracking hazard -> active, resets idle timer and re-rolls threshold
            let is_active = is_moving || is_selected || is_waffing || is_hazard_tracking;
            self.update_hound_idle(idx, is_active, dt);
            if is_moving {
                self.hound_sit_blend[idx] = 0.0;
            }
            let is_sitting = self.is_hound_sitting(idx);

            // Smoothly blend sitting transition
            let target_sit = if is_sitting { 1.0 } else { 0.0 };
            self.hound_sit_blend[idx] +=
                (target_sit - self.hound_sit_blend[idx]) * (1.0 - (-10.0 * dt).exp());

            // Select Hound Texture (standing vs sitting, terrestrial vs martian)
            let hound_tex = self.active_hound_texture(state.variant, idx, is_sitting);

            if let Some(tex) = hound_tex {
                // Subtle breathing / idle micro-sway (calmer when sitting)
                let idle_breathe = if !is_moving && !is_waffing {
                    if is_sitting {
                        (t * 2.2 + idx as f32 * 1.5).sin() * 0.015
                    } else {
                        (t * 3.0 + idx as f32 * 1.5).sin() * 0.02
                    }
                } else {
                    0.0
                };
                let idle_sway = if !is_moving && !is_waffing {
                    if is_sitting {
                        (t * 1.8 + idx as f32 * 1.2).sin() * 0.012
                    } else {
                        (t * 2.5 + idx as f32 * 1.2).sin() * 0.02
                    }
                } else {
                    0.0
                };

                // When sitting, the dog visibly compacts in length on the board
                let piece_size = state.variant.piece_base_size();
                let sit_compression = if state.variant.is_red_hunt() {
                    8.0
                } else {
                    12.0
                };
                let base_h = piece_size - self.hound_sit_blend[idx] * sit_compression;
                let breathe_amp = (piece_size / DEFAULT_PIECE_BASE_SIZE) * 15.0;
                let target_h = (base_h + idle_breathe * breathe_amp) * scale;
                let aspect = tex.width() / tex.height();
                let target_w = target_h * aspect;

                // Cheerful tail wag when comfortably sitting
                let sit_tail_wag = if is_sitting {
                    (t * 4.5 + idx as f32 * 2.1).sin() * 0.045 * self.hound_sit_blend[idx]
                } else {
                    0.0
                };

                let rot = self.hound_angles[idx] + idle_sway + bark_jitter + sit_tail_wag;

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

        // 3. Draw Fox
        let ground_pos = origin + fox_visual_pos * scale;
        // Gentle anticipation hop while the Fox is still choosing its entry square (Classic move 1)
        let pending_hop = if state.fox_pending {
            (t * 5.0).sin().abs() * 6.0 * scale
        } else {
            0.0
        };
        let render_pos = Vec2::new(
            ground_pos.x,
            ground_pos.y - (fox_jump_lift * scale) - pending_hop,
        );

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

        let fox_tex = self.active_fox_texture(state.variant);
        if let Some(tex) = fox_tex {
            let idle_breathe = if !is_fox_moving {
                (t * 3.2).sin() * 0.02
            } else {
                0.0
            };
            let idle_sway = if !is_fox_moving {
                (t * 2.2).sin() * 0.02
            } else {
                0.0
            };

            let piece_size = state.variant.piece_base_size();
            let breathe_amp = (piece_size / DEFAULT_PIECE_BASE_SIZE) * 15.0;
            let target_h = (piece_size + idle_breathe * breathe_amp) * scale;
            let aspect = tex.width() / tex.height();
            let target_w = target_h * aspect;

            let rot = self.fox_angle + idle_sway;

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

        waf_sound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::state::{Difficulty, MoveAnimation};

    /// Headless BoardView for tests that only exercise timing/logic helpers; every
    /// texture stays `None` so nothing touches the GPU.
    fn test_view() -> BoardView {
        const NO_TEX: Option<Texture2D> = None;
        BoardView {
            board_textures: [NO_TEX; VARIANT_COUNT],
            board_texture: None,
            current_variant: None,
            fox_texture: None,
            hound_textures: [None, None, None],
            hound_sit_textures: [None, None, None],
            martian_fox_texture: None,
            martian_hound_textures: [None, None, None],
            martian_hound_sit_textures: [None, None, None],
            train_texture: None,
            rover_texture: None,
            boat_texture: None,
            bridge_texture: None,
            no_reverse_dog_texture: None,
            hound_angles: [0.0; 3],
            fox_angle: 0.0,
            hover_node_id: None,
            font: None,
            river: RiverSimulation::for_variant(BoardVariant::Classic),
            train: TrainSimulation::new(),
            rover: RoverSimulation::new(),
            boat: BoatSimulation::new(),
            last_waf_sound_time: 0.0,
            hound_idle_times: [0.0; 3],
            hound_sit_thresholds: [10.0, 10.0, 10.0],
            hound_sit_blend: [0.0; 3],
            start_target_alpha: 0.0,
            special_rule_notice_alpha: 0.0,
            special_rule_reminder_seconds: 0.0,
            fox_idle_seconds: 0.0,
        }
    }

    #[test]
    fn test_hound_idle_bounds_safety() {
        let mut view = test_view();

        // The opening objective reticle stays hidden until a match begins
        assert_eq!(view.start_target_alpha, 0.0);
        assert_eq!(view.fox_idle_seconds, 0.0);

        // Indices within bounds
        assert!(!view.is_hound_sitting(0));
        view.update_hound_idle(0, false, 12.0);
        assert!(view.is_hound_sitting(0));

        // Indices out of bounds must not panic
        assert!(!view.is_hound_sitting(3));
        assert!(!view.is_hound_sitting(10));
        view.update_hound_idle(3, false, 12.0);
        view.update_hound_idle(99, true, 1.0);
    }

    #[test]
    fn test_objective_hint_on_opening_move_and_after_fox_idles() {
        const IDLE: f32 = FOX_OBJECTIVE_HINT_IDLE_SECONDS;

        // Fox-controlled match: the hint is on until the very first move
        let mut fox_game = GameState::new();
        fox_game.start_game(Faction::Fox, Difficulty::Medium);
        assert!(fox_game.move_history.is_empty());
        assert!(should_highlight_fox_objective(&fox_game, 0.0));

        let opening = fox_game.fox_legal_moves();
        assert!(!opening.is_empty());
        assert!(fox_game.apply_fox_move(opening[0]).is_ok());

        // Now the hounds are to move, so it stays hidden no matter how long they take
        assert!(!should_highlight_fox_objective(&fox_game, IDLE * 100.0));

        // Once the hounds have replied the hint waits for the Fox player to idle again
        let hound_moves = fox_game.all_hound_legal_moves();
        assert!(!hound_moves.is_empty());
        assert!(fox_game
            .apply_hound_move(hound_moves[0].0, hound_moves[0].1)
            .is_ok());
        assert_eq!(fox_game.current_turn, Faction::Fox);
        assert!(!fox_game.move_history.is_empty());

        // ...but not until that hound has finished gliding and the board settles
        assert!(!should_highlight_fox_objective(&fox_game, IDLE * 100.0));
        fox_game.active_anim = None;

        // Just short of the wait it is hidden, then it comes back as a reminder
        assert!(!should_highlight_fox_objective(&fox_game, IDLE - 0.1));
        assert!(should_highlight_fox_objective(&fox_game, IDLE));

        // Hound-controlled matches never advertise the Fox objective, however long
        // they idle
        let mut hound_game = GameState::new();
        hound_game.start_game(Faction::Hounds, Difficulty::Medium);
        assert!(!should_highlight_fox_objective(&hound_game, 0.0));
        assert!(!should_highlight_fox_objective(&hound_game, IDLE * 100.0));

        // Finished matches (and the title screen) never show it either
        let mut finished = GameState::new();
        finished.start_game(Faction::Fox, Difficulty::Medium);
        finished.phase = GamePhase::GameOver;
        assert!(!should_highlight_fox_objective(&finished, IDLE * 100.0));

        let mut titled = GameState::new();
        titled.start_game(Faction::Fox, Difficulty::Medium);
        titled.phase = GamePhase::TitleScreen;
        assert!(!should_highlight_fox_objective(&titled, IDLE * 100.0));
    }

    #[test]
    fn test_fox_idle_timer_accrues_only_while_fox_waits() {
        let mut fox_game = GameState::new();
        fox_game.start_game(Faction::Fox, Difficulty::Medium);

        // Idling on the Fox turn accumulates the wait
        let mut view = test_view();
        view.update_fox_idle(&fox_game, 4.0);
        assert!((view.fox_idle_seconds - 4.0).abs() < 1e-6);
        view.update_fox_idle(&fox_game, 6.5);
        assert!((view.fox_idle_seconds - 10.5).abs() < 1e-6);
        assert!(should_highlight_fox_objective(
            &fox_game,
            view.fox_idle_seconds
        ));

        // Moving the fox hands the turn to the hounds and clears the wait
        let opening = fox_game.fox_legal_moves();
        assert!(fox_game.apply_fox_move(opening[0]).is_ok());
        view.update_fox_idle(&fox_game, 1.0);
        assert_eq!(view.fox_idle_seconds, 0.0);

        // Nor does it run while a piece is still gliding, or once the match is over
        let mut gliding = GameState::new();
        gliding.start_game(Faction::Fox, Difficulty::Medium);
        gliding.active_anim = Some(MoveAnimation {
            from: Vec2::ZERO,
            to: Vec2::ZERO,
            progress: 0.5,
            duration: 1.0,
            faction: Faction::Fox,
            hound_idx: None,
        });
        view.fox_idle_seconds = 3.0;
        view.update_fox_idle(&gliding, 1.0);
        assert_eq!(view.fox_idle_seconds, 0.0);

        let mut over = GameState::new();
        over.start_game(Faction::Fox, Difficulty::Medium);
        over.phase = GamePhase::GameOver;
        view.fox_idle_seconds = 3.0;
        view.update_fox_idle(&over, 1.0);
        assert_eq!(view.fox_idle_seconds, 0.0);

        // A negative dt can never rewind the wait while the fox is on the clock
        let mut fresh = GameState::new();
        fresh.start_game(Faction::Fox, Difficulty::Medium);
        view.fox_idle_seconds = 3.0;
        view.update_fox_idle(&fresh, -1.0);
        assert!((view.fox_idle_seconds - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset_simulations_clears_fox_idle() {
        let mut view = test_view();
        view.fox_idle_seconds = 7.5;
        view.start_target_alpha = 0.4;
        view.special_rule_notice_alpha = 0.6;
        view.special_rule_reminder_seconds = 2.0;
        view.reset_simulations();
        assert_eq!(view.fox_idle_seconds, 0.0);
        assert_eq!(view.start_target_alpha, 0.0);
        assert_eq!(view.special_rule_notice_alpha, 0.0);
        assert_eq!(view.special_rule_reminder_seconds, 0.0);
    }

    #[test]
    fn test_special_rule_notice_resurfaces_on_retreat_click() {
        let mut state = GameState::new();
        state.start_game(Faction::Hounds, Difficulty::Medium);

        let m0 = state.graph.find_id_by_name("M0").unwrap();
        let m1 = state.graph.find_id_by_name("M1").unwrap();
        let m2 = state.graph.find_id_by_name("M2").unwrap();
        let m3 = state.graph.find_id_by_name("M3").unwrap();

        // AI Fox opens at M3, player moves dog from M0 to M1 (row 0 -> row 1)
        assert!(state.apply_fox_move(m3).is_ok());
        let m0_idx = state.hounds_pos.iter().position(|&p| p == m0).unwrap() as u8;
        assert!(state.apply_hound_move(m0_idx, m1).is_ok());
        assert!(!should_show_special_rule_notice(&state));

        // Now hound at M1 (row 1) has neighbor M0 (row 0).
        // Clicking M0 (behind M1) is detected as an attempted retreat move
        state.selected_hound_idx = Some(m0_idx);
        assert!(is_hound_retreat_click(&state, m0));

        // Clicking forward to M2 (row 2) is a forward advance, not retreat
        assert!(!is_hound_retreat_click(&state, m2));

        // Even when no hound is actively selected, clicking M0 detects the retreat
        state.selected_hound_idx = None;
        assert!(is_hound_retreat_click(&state, m0));

        // Boards with retreat enabled never flag retreat clicks
        let mut river = GameState::new();
        river.switch_variant(BoardVariant::RiverCrossing);
        river.start_game(Faction::Hounds, Difficulty::Medium);
        assert!(!is_hound_retreat_click(&river, 0));

        // Verify reminder timer trigger
        let mut view = test_view();
        assert_eq!(view.special_rule_reminder_seconds, 0.0);
        view.trigger_special_rule_reminder();
        assert_eq!(
            view.special_rule_reminder_seconds,
            SPECIAL_RULE_REMINDER_DURATION
        );
    }

    #[test]
    fn test_special_rule_notice_opens_the_match_and_leaves_with_the_first_move() {
        // Classic forbids the hounds to fall back: the notice belongs to the opening turn
        let mut fox_game = GameState::new();
        assert!(!fox_game.variant.config().allow_hound_retreat);
        assert!(!should_show_special_rule_notice(&fox_game)); // still on the title screen

        fox_game.start_game(Faction::Fox, Difficulty::Medium);
        assert!(should_show_special_rule_notice(&fox_game));

        // A Fox player loses the notice on their own opening move
        let opening = fox_game.fox_legal_moves();
        assert!(!opening.is_empty());
        assert!(fox_game.apply_fox_move(opening[0]).is_ok());
        assert!(!should_show_special_rule_notice(&fox_game));

        // A Hounds player keeps it for their opening decision: the AI Fox has answered by
        // then, but the rule is the one they have to play by
        let mut hound_game = GameState::new();
        hound_game.start_game(Faction::Hounds, Difficulty::Medium);
        assert!(should_show_special_rule_notice(&hound_game));

        let opening = hound_game.fox_legal_moves();
        assert!(hound_game.apply_fox_move(opening[0]).is_ok());
        assert!(should_show_special_rule_notice(&hound_game));

        let hound_moves = hound_game.all_hound_legal_moves();
        assert!(!hound_moves.is_empty());
        assert!(hound_game
            .apply_hound_move(hound_moves[0].0, hound_moves[0].1)
            .is_ok());
        assert!(!should_show_special_rule_notice(&hound_game));

        // Boards with free hound movement never announce it
        let mut river = GameState::new();
        river.switch_variant(BoardVariant::RiverCrossing);
        river.start_game(Faction::Fox, Difficulty::Medium);
        assert!(river.variant.config().allow_hound_retreat);
        assert!(river.move_history.is_empty());
        assert!(!should_show_special_rule_notice(&river));

        // A fresh match on the Classic board brings it back
        fox_game.start_game(Faction::Fox, Difficulty::Medium);
        assert!(should_show_special_rule_notice(&fox_game));

        // ...and a finished match keeps it off the board
        fox_game.phase = GamePhase::GameOver;
        assert!(!should_show_special_rule_notice(&fox_game));
    }

    #[test]
    fn test_special_rule_notice_waits_for_the_opening_zoom() {
        let mut game = GameState::new();
        game.start_game(Faction::Fox, Difficulty::Medium);
        assert!(should_show_special_rule_notice(&game));

        // The intro zoom owns the screen while it flies into the field: the notice waits for
        // the framing to settle and only then fades in
        assert!(!wants_special_rule_notice(&game, 0.0, false));
        assert!(wants_special_rule_notice(&game, 0.0, true));

        // An illegal retreat click is a direct answer to the player, so its reminder does not
        // wait for the camera
        assert!(wants_special_rule_notice(
            &game,
            SPECIAL_RULE_REMINDER_DURATION,
            false
        ));

        // The player's own opening move retires the announcement for good
        let opening = game.fox_legal_moves();
        assert!(!opening.is_empty());
        assert!(game.apply_fox_move(opening[0]).is_ok());
        assert!(!wants_special_rule_notice(&game, 0.0, true));

        // ...though a later illegal retreat click still brings it back as a reminder
        assert!(wants_special_rule_notice(
            &game,
            SPECIAL_RULE_REMINDER_DURATION,
            true
        ));

        // Boards with free hound movement never announce a rule, reminder or not
        let mut river = GameState::new();
        river.switch_variant(BoardVariant::RiverCrossing);
        river.start_game(Faction::Fox, Difficulty::Medium);
        assert!(!wants_special_rule_notice(
            &river,
            SPECIAL_RULE_REMINDER_DURATION,
            true
        ));

        // A finished match keeps it off the board too
        game.phase = GamePhase::GameOver;
        assert!(!wants_special_rule_notice(
            &game,
            SPECIAL_RULE_REMINDER_DURATION,
            true
        ));
    }

    #[test]
    fn test_special_rule_notice_layout_clears_the_pieces_and_fits_the_field() {
        let framing = BoardVariant::Classic.config().intro_framing;
        let piece_size = BoardVariant::Classic.config().piece_base_size;

        // Classic: the notice sits in the clear strip below the hound line (B3 at y = 690)
        let lowest_node_y = (BoardVariant::Classic.config().build_graph)()
            .nodes
            .iter()
            .fold(f32::MIN, |lowest, node| lowest.max(node.visual_pos.y));
        assert!((lowest_node_y - 690.0).abs() < 0.01);

        let center = special_rule_notice_center(framing, lowest_node_y, piece_size);
        let field_bottom = framing.playable_center.y + framing.playable_size.y * 0.5;
        let piece_bottom = lowest_node_y + piece_size * 0.5 + SPECIAL_RULE_NOTICE_PIECE_CLEARANCE;
        assert!((center.x - framing.playable_center.x).abs() < 0.01);
        assert!(center.y > piece_bottom);
        assert!(center.y < field_bottom);

        // The Classic field leaves enough room below the hound line for the base plate
        assert!(
            field_bottom - piece_bottom
                >= SPECIAL_RULE_NOTICE_BASE_FONT_SIZE + 2.0 * SPECIAL_RULE_NOTICE_PLATE_PADDING
        );

        // A board whose pieces reach the bottom edge of the field keeps the notice on the field
        let cramped = special_rule_notice_center(framing, field_bottom + 40.0, piece_size);
        assert!((cramped.y - field_bottom).abs() < 0.01);

        // A sentence that already fits keeps its base size
        assert_eq!(fit_special_rule_notice_font_size(20, 120.0, 200.0), 20);
        assert_eq!(fit_special_rule_notice_font_size(20, 0.0, 200.0), 20);

        // A long translation shrinks until it fits the plate
        let shrunk = fit_special_rule_notice_font_size(20, 400.0, 200.0);
        assert!(shrunk < 20);
        assert!(shrunk >= SPECIAL_RULE_NOTICE_MIN_FONT_SIZE);
        // ...but never below the legibility floor
        assert_eq!(
            fit_special_rule_notice_font_size(20, 10_000.0, 200.0),
            SPECIAL_RULE_NOTICE_MIN_FONT_SIZE
        );

        // Small base sizes below the floor never expand above base size when text overflows
        assert_eq!(fit_special_rule_notice_font_size(8, 400.0, 200.0), 8);
        assert_eq!(fit_special_rule_notice_font_size(8, 100.0, 200.0), 8);
    }

    #[test]
    fn test_reticle_arrow_geometry_and_fade() {
        let center = Vec2::new(300.0, 200.0);
        let (tip, left, right) = target_arrow_vertices(center, 0.0, 20.0, 30.0, 5.0);
        assert!((tip - Vec2::new(320.0, 200.0)).length() < 0.01);
        assert!((left - Vec2::new(330.0, 205.0)).length() < 0.01);
        assert!((right - Vec2::new(330.0, 195.0)).length() < 0.01);

        // The tip sits nearer the spot than the base, so the arrow targets the node
        assert!((tip - center).length() < (left - center).length());

        // Rotating an arrow keeps its tip on the reticle circle
        let (rotated_tip, _, _) =
            target_arrow_vertices(center, std::f32::consts::FRAC_PI_2, 20.0, 30.0, 5.0);
        assert!((rotated_tip - Vec2::new(300.0, 220.0)).length() < 0.01);

        // Fade weight eases toward its target without overshooting
        let rising = approach_fade(0.0, 1.0, 2.2, 0.016);
        assert!((0.0..=1.0).contains(&rising));
        let falling = approach_fade(1.0, 0.0, 5.0, 0.016);
        assert!((0.0..1.0).contains(&falling));
        assert!((approach_fade(1.0, 1.0, 2.2, 0.016) - 1.0).abs() < 1e-6);
    }
}
