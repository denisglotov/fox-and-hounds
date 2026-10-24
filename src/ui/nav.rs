use crate::game::state::{Faction, GameState};
use macroquad::input::TouchPhase;
use macroquad::prelude::*;

const MIN_DIRECTION_COS: f32 = 0.35;
const FALLBACK_DIRECTION_COS: f32 = 0.55;
const MOUSE_MOVE_THRESHOLD_SQ: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDir {
    Up,
    Down,
    Left,
    Right,
}

impl NavDir {
    pub const fn unit_vec(self) -> Vec2 {
        match self {
            NavDir::Up => Vec2::new(0.0, -1.0),
            NavDir::Down => Vec2::new(0.0, 1.0),
            NavDir::Left => Vec2::new(-1.0, 0.0),
            NavDir::Right => Vec2::new(1.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NavInput {
    pub dir: Option<NavDir>,
    pub confirm: bool,
    pub back: bool,
    pub pointer_active: bool,
}

impl NavInput {
    pub const fn any_key(self) -> bool {
        self.dir.is_some() || self.confirm || self.back
    }

    pub fn poll(last_mouse_pos: &mut Option<Vec2>, check_pointer: bool) -> Self {
        let dir = if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            Some(NavDir::Up)
        } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            Some(NavDir::Down)
        } else if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            Some(NavDir::Left)
        } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            Some(NavDir::Right)
        } else {
            None
        };

        let confirm = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || is_key_pressed(KeyCode::Space);

        let back = is_key_pressed(KeyCode::Back)
            || is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::Backspace);

        let cur_mouse = Vec2::from(mouse_position());
        let mouse_moved = last_mouse_pos.is_some_and(|prev| {
            prev != Vec2::ZERO
                && cur_mouse != Vec2::ZERO
                && (cur_mouse - prev).length_squared() > MOUSE_MOVE_THRESHOLD_SQ
        });
        if cur_mouse != Vec2::ZERO {
            *last_mouse_pos = Some(cur_mouse);
        }

        let pointer_active = check_pointer
            && (mouse_moved
                || is_mouse_button_pressed(MouseButton::Left)
                || is_mouse_button_released(MouseButton::Left)
                || touches()
                    .into_iter()
                    .any(|t| t.phase != TouchPhase::Cancelled));

        Self {
            dir,
            confirm,
            back,
            pointer_active,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TitleFocus {
    HeroBanner,
    #[default]
    VariantCarousel,
    Faction,
    Difficulty,
    StartButton,
}

impl TitleFocus {
    pub const fn step_vertical(self, dir: NavDir, has_hero_banner: bool) -> Self {
        match (self, dir) {
            (Self::HeroBanner, NavDir::Down) => Self::VariantCarousel,
            (Self::VariantCarousel, NavDir::Down) => Self::Faction,
            (Self::Faction, NavDir::Down) => Self::Difficulty,
            (Self::Difficulty, NavDir::Down) => Self::StartButton,
            (Self::StartButton, NavDir::Down) => {
                if has_hero_banner {
                    Self::HeroBanner
                } else {
                    Self::VariantCarousel
                }
            }
            (Self::VariantCarousel, NavDir::Up) => {
                if has_hero_banner {
                    Self::HeroBanner
                } else {
                    Self::StartButton
                }
            }
            (Self::HeroBanner, NavDir::Up) => Self::StartButton,
            (Self::Faction, NavDir::Up) => Self::VariantCarousel,
            (Self::Difficulty, NavDir::Up) => Self::Faction,
            (Self::StartButton, NavDir::Up) => Self::Difficulty,
            (other, _) => other,
        }
    }

    pub const fn step_horizontal(
        self,
        dir: NavDir,
        is_landscape: bool,
        has_hero_banner: bool,
    ) -> Option<Self> {
        match (self, dir) {
            (Self::HeroBanner, NavDir::Right) => Some(Self::VariantCarousel),
            (Self::StartButton, NavDir::Left) if is_landscape && has_hero_banner => {
                Some(Self::HeroBanner)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HudButton {
    Mute,
    Restart,
    #[default]
    Menu,
}

impl HudButton {
    pub const fn step_horizontal(self, dir: NavDir) -> Self {
        match (self, dir) {
            (Self::Mute, NavDir::Right) => Self::Restart,
            (Self::Restart, NavDir::Right) => Self::Menu,
            (Self::Menu, NavDir::Right) => Self::Mute,
            (Self::Menu, NavDir::Left) => Self::Restart,
            (Self::Restart, NavDir::Left) => Self::Mute,
            (Self::Mute, NavDir::Left) => Self::Menu,
            (other, _) => other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InGameFocus {
    #[default]
    Board,
    Hud(HudButton),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameOverFocus {
    #[default]
    PlayAgain,
    MainMenu,
}

impl GameOverFocus {
    pub const fn toggle(self) -> Self {
        match self {
            Self::PlayAgain => Self::MainMenu,
            Self::MainMenu => Self::PlayAgain,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardNavStep {
    Node(u8),
    ToHud,
    Stay,
}

#[derive(Debug, Clone)]
pub struct UiNavState {
    pub focus_visible: bool,
    pub title_focus: TitleFocus,
    pub ingame_focus: InGameFocus,
    pub game_over_focus: GameOverFocus,
    last_mouse_pos: Option<Vec2>,
}

impl Default for UiNavState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNavState {
    pub fn new() -> Self {
        Self {
            focus_visible: is_android_tv(),
            title_focus: TitleFocus::default(),
            ingame_focus: InGameFocus::default(),
            game_over_focus: GameOverFocus::default(),
            last_mouse_pos: None,
        }
    }

    pub fn poll_input(&mut self) -> NavInput {
        let input = NavInput::poll(&mut self.last_mouse_pos, self.focus_visible);
        if input.any_key() {
            self.focus_visible = true;
        } else if input.pointer_active {
            self.focus_visible = false;
        }
        input
    }
}

/// Returns the default board node to place the cursor on at the start of a turn.
pub fn default_turn_cursor_node(state: &GameState) -> Option<u8> {
    match state.player_faction {
        Faction::Fox => Some(state.fox_pos),
        Faction::Hounds => {
            if let Some(sel_idx) = state.selected_hound_idx {
                if let Some(&pos) = state.hounds_pos.get(sel_idx as usize) {
                    return Some(pos);
                }
            }
            // Prefer the middle hound (idx 1) if it has legal moves, otherwise any movable hound
            let preferred_order = [1usize, 0, 2];
            preferred_order
                .into_iter()
                .find_map(|idx| {
                    let &pos = state.hounds_pos.get(idx)?;
                    (!state.hound_legal_moves(idx as u8).is_empty()).then_some(pos)
                })
                .or_else(|| state.hounds_pos.first().copied())
        }
    }
}

/// Evaluates the best candidate node from `candidates` lying in direction `dir` from `origin_pos`.
fn pick_best_directional_node(
    state: &GameState,
    origin_id: u8,
    origin_pos: Vec2,
    dir: NavDir,
    min_cos: f32,
    candidates: impl IntoIterator<Item = u8>,
) -> Option<u8> {
    let dir_vec = dir.unit_vec();
    candidates
        .into_iter()
        .filter(|&cid| cid != origin_id)
        .filter_map(|cid| {
            let target_pos = state.graph.node(cid)?.visual_pos;
            let delta = target_pos - origin_pos;
            let dist = delta.length();
            if dist <= 1.0 {
                return None;
            }
            let cos_theta = (delta / dist).dot(dir_vec);
            (cos_theta >= min_cos).then(|| {
                let score = dist / (cos_theta * cos_theta);
                (cid, score)
            })
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(cid, _)| cid)
}

fn pick_two_tier_node(
    state: &GameState,
    current_node: u8,
    curr_pos: Vec2,
    dir: NavDir,
    on_smart_node: bool,
    tier1: impl IntoIterator<Item = u8>,
    tier2_extra: impl IntoIterator<Item = u8>,
) -> Option<u8> {
    on_smart_node
        .then(|| {
            pick_best_directional_node(state, current_node, curr_pos, dir, MIN_DIRECTION_COS, tier1)
        })
        .flatten()
        .or_else(|| {
            pick_best_directional_node(
                state,
                current_node,
                curr_pos,
                dir,
                MIN_DIRECTION_COS,
                state
                    .graph
                    .neighbors(current_node)
                    .iter()
                    .copied()
                    .chain(tier2_extra),
            )
        })
}

/// Hybrid D-pad navigation step on the board graph:
/// 1. Snaps between active pieces and their legal destination nodes when on a piece/legal node.
/// 2. Falls back to stepping along adjacent graph edges (e.g. to reach C8 or attempt a retreat).
/// 3. Transitions to the top HUD bar (`BoardNavStep::ToHud`) when pressing Up from topmost nodes.
pub fn step_board_cursor(state: &GameState, current_node: u8, dir: NavDir) -> BoardNavStep {
    let Some(curr) = state.graph.node(current_node) else {
        return BoardNavStep::Stay;
    };
    let curr_pos = curr.visual_pos;

    let chosen = if state.is_ai_turn() {
        pick_best_directional_node(
            state,
            current_node,
            curr_pos,
            dir,
            MIN_DIRECTION_COS,
            state.graph.neighbors(current_node).iter().copied(),
        )
    } else {
        match state.player_faction {
            Faction::Fox => {
                let legal = state.fox_legal_moves();
                let on_smart = current_node == state.fox_pos || legal.contains(&current_node);
                pick_two_tier_node(
                    state,
                    current_node,
                    curr_pos,
                    dir,
                    on_smart,
                    legal
                        .iter()
                        .copied()
                        .chain((!state.fox_pending).then_some(state.fox_pos)),
                    legal.iter().copied(),
                )
            }
            Faction::Hounds => {
                if let Some(h_idx) = state.selected_hound_idx {
                    let hound_pos = state.hounds_pos.get(h_idx as usize).copied();
                    let legal = state.hound_legal_moves(h_idx);
                    let on_smart = Some(current_node) == hound_pos || legal.contains(&current_node);
                    pick_two_tier_node(
                        state,
                        current_node,
                        curr_pos,
                        dir,
                        on_smart,
                        legal.iter().copied().chain(hound_pos),
                        state
                            .hounds_pos
                            .iter()
                            .copied()
                            .chain(legal.iter().copied()),
                    )
                } else {
                    pick_two_tier_node(
                        state,
                        current_node,
                        curr_pos,
                        dir,
                        state.hounds_pos.contains(&current_node),
                        state.hounds_pos.iter().copied(),
                        state.hounds_pos.iter().copied(),
                    )
                }
            }
        }
    }
    .or_else(|| {
        // Fallback for disconnected holding nodes (e.g. M4 Fox Den on Classic)
        pick_best_directional_node(
            state,
            current_node,
            curr_pos,
            dir,
            FALLBACK_DIRECTION_COS,
            state.graph.nodes.iter().map(|n| n.id),
        )
    });

    match chosen {
        Some(next_id) => BoardNavStep::Node(next_id),
        None if dir == NavDir::Up => BoardNavStep::ToHud,
        None => BoardNavStep::Stay,
    }
}

/// Draws a high-contrast gold focus ring around a rectangular UI element for TV / keyboard focus.
pub fn draw_focus_rect(bounds: Rect, scale: f32, t: f32) {
    let pulse = (t * 4.0).sin() * 0.5 + 0.5;
    let expand = (3.0 + pulse * 1.5) * scale;
    let outer = Rect::new(
        bounds.x - expand,
        bounds.y - expand,
        bounds.w + expand * 2.0,
        bounds.h + expand * 2.0,
    );

    // Dark contrast backing
    draw_rectangle_lines(
        outer.x - 1.0 * scale,
        outer.y - 1.0 * scale,
        outer.w + 2.0 * scale,
        outer.h + 2.0 * scale,
        3.5 * scale,
        Color::from_rgba(10, 16, 26, 210),
    );
    // Bright gold focus outline
    draw_rectangle_lines(
        outer.x,
        outer.y,
        outer.w,
        outer.h,
        2.5 * scale,
        Color::from_rgba(255, 224, 130, 255),
    );
}

#[cfg(target_os = "android")]
pub fn is_android_tv() -> bool {
    const UI_MODE_TYPE_MASK: i32 = 0x0f;
    const UI_MODE_TYPE_TELEVISION: i32 = 0x04;

    unsafe fn query_tv_mode() -> Option<bool> {
        let env = macroquad::miniquad::native::android::attach_jni_env();
        let activity = macroquad::miniquad::native::android::ACTIVITY;
        if env.is_null() || activity.is_null() {
            return None;
        }
        let find_class = (**env).FindClass?;
        let get_method_id = (**env).GetMethodID?;
        let call_object_method = (**env).CallObjectMethod?;
        let get_field_id = (**env).GetFieldID?;
        let get_int_field = (**env).GetIntField?;

        let non_null = |p: *mut std::ffi::c_void| (!p.is_null()).then_some(p);

        let ctx_cls = non_null(find_class(env, b"android/content/Context\0".as_ptr() as _) as _)?;
        let get_res_mid = non_null(get_method_id(
            env,
            ctx_cls as _,
            b"getResources\0".as_ptr() as _,
            b"()Landroid/content/res/Resources;\0".as_ptr() as _,
        ) as _)?;
        let res_obj = non_null(call_object_method(env, activity, get_res_mid as _) as _)?;

        let res_cls =
            non_null(find_class(env, b"android/content/res/Resources\0".as_ptr() as _) as _)?;
        let get_cfg_mid = non_null(get_method_id(
            env,
            res_cls as _,
            b"getConfiguration\0".as_ptr() as _,
            b"()Landroid/content/res/Configuration;\0".as_ptr() as _,
        ) as _)?;
        let cfg_obj = non_null(call_object_method(env, res_obj as _, get_cfg_mid as _) as _)?;

        let cfg_cls =
            non_null(find_class(env, b"android/content/res/Configuration\0".as_ptr() as _) as _)?;
        let ui_mode_fid = non_null(get_field_id(
            env,
            cfg_cls as _,
            b"uiMode\0".as_ptr() as _,
            b"I\0".as_ptr() as _,
        ) as _)?;
        let ui_mode = get_int_field(env, cfg_obj as _, ui_mode_fid as _);
        Some((ui_mode & UI_MODE_TYPE_MASK) == UI_MODE_TYPE_TELEVISION)
    }

    unsafe { query_tv_mode().unwrap_or(false) }
}

#[cfg(not(target_os = "android"))]
pub fn is_android_tv() -> bool {
    false
}

#[cfg(target_os = "android")]
pub fn request_app_exit() {
    unsafe fn finish_activity() -> Option<()> {
        let env = macroquad::miniquad::native::android::attach_jni_env();
        let activity = macroquad::miniquad::native::android::ACTIVITY;
        if env.is_null() || activity.is_null() {
            return None;
        }
        let find_class = (**env).FindClass?;
        let get_method_id = (**env).GetMethodID?;
        let call_void_method = (**env).CallVoidMethod?;

        let cls = find_class(env, b"android/app/Activity\0".as_ptr() as _);
        if cls.is_null() {
            return None;
        }
        let mid = get_method_id(env, cls, b"finish\0".as_ptr() as _, b"()V\0".as_ptr() as _);
        if !mid.is_null() {
            call_void_method(env, activity, mid);
        }
        Some(())
    }

    unsafe {
        let _ = finish_activity();
    }
}

#[cfg(not(target_os = "android"))]
pub fn request_app_exit() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::level::BoardVariant;
    use crate::game::state::Difficulty;

    #[test]
    fn test_title_focus_vertical_cycle() {
        let f0 = TitleFocus::VariantCarousel;
        let f1 = f0.step_vertical(NavDir::Down, true);
        assert_eq!(f1, TitleFocus::Faction);
        let f2 = f1.step_vertical(NavDir::Down, true);
        assert_eq!(f2, TitleFocus::Difficulty);
        let f3 = f2.step_vertical(NavDir::Down, true);
        assert_eq!(f3, TitleFocus::StartButton);
        let f4 = f3.step_vertical(NavDir::Down, true);
        assert_eq!(f4, TitleFocus::HeroBanner);
        let f5 = f4.step_vertical(NavDir::Down, true);
        assert_eq!(f5, TitleFocus::VariantCarousel);

        assert_eq!(f0.step_vertical(NavDir::Up, true), TitleFocus::HeroBanner);
        assert_eq!(f4.step_vertical(NavDir::Up, true), TitleFocus::StartButton);
        assert_eq!(f3.step_vertical(NavDir::Up, true), TitleFocus::Difficulty);

        assert_eq!(f0.step_vertical(NavDir::Up, false), TitleFocus::StartButton);
        assert_eq!(
            f3.step_vertical(NavDir::Down, false),
            TitleFocus::VariantCarousel
        );
    }

    #[test]
    fn test_title_focus_horizontal() {
        assert_eq!(
            TitleFocus::HeroBanner.step_horizontal(NavDir::Right, true, true),
            Some(TitleFocus::VariantCarousel)
        );
        assert_eq!(
            TitleFocus::StartButton.step_horizontal(NavDir::Left, true, true),
            Some(TitleFocus::HeroBanner)
        );
        assert_eq!(
            TitleFocus::StartButton.step_horizontal(NavDir::Left, false, true),
            None
        );
    }

    #[test]
    fn test_hud_button_horizontal_cycle() {
        let b0 = HudButton::Menu;
        assert_eq!(b0.step_horizontal(NavDir::Left), HudButton::Restart);
        assert_eq!(
            b0.step_horizontal(NavDir::Left)
                .step_horizontal(NavDir::Left),
            HudButton::Mute
        );
        assert_eq!(b0.step_horizontal(NavDir::Right), HudButton::Mute);
    }

    #[test]
    fn test_default_turn_cursor_node_fox_and_hounds() {
        let mut state = GameState::new();
        state.switch_variant(BoardVariant::RiverCrossing);
        state.start_game(Faction::Fox, Difficulty::Medium);
        assert_eq!(default_turn_cursor_node(&state), Some(state.fox_pos));

        state.start_game(Faction::Hounds, Difficulty::Medium);
        let cursor = default_turn_cursor_node(&state).unwrap();
        assert!(state.hounds_pos.contains(&cursor));
    }

    #[test]
    fn test_step_board_cursor_classic_opening_and_hud_transition() {
        let mut state = GameState::new();
        state.switch_variant(BoardVariant::Classic);
        state.start_game(Faction::Fox, Difficulty::Medium);

        // Fox starts at M4 holding square on Classic (right side); pressing Left/Up steps onto entry nodes (T3/M3/B3)
        let start_id = state.fox_pos;
        let step = step_board_cursor(&state, start_id, NavDir::Left);
        let BoardNavStep::Node(entry_id) = step else {
            panic!("Expected Node step from M4, got {step:?}");
        };
        assert!(state.fox_legal_moves().contains(&entry_id));

        // Pressing Up/Down switches among the 3 vertical entry nodes (T3, M3, B3)
        let vertical_up = step_board_cursor(&state, entry_id, NavDir::Up);
        let vertical_down = step_board_cursor(&state, entry_id, NavDir::Down);
        assert!(
            matches!(vertical_up, BoardNavStep::Node(_))
                || matches!(vertical_down, BoardNavStep::Node(_))
        );

        // Pressing Up from a topmost node (T2) transitions to the HUD bar
        let t2 = state.graph.find_id_by_name("T2").unwrap();
        assert_eq!(
            step_board_cursor(&state, t2, NavDir::Up),
            BoardNavStep::ToHud
        );
    }

    #[test]
    fn test_step_board_cursor_reaches_c8_on_fox_and_dogs() {
        let mut state = GameState::new();
        state.switch_variant(BoardVariant::FoxAndDogsMaze);
        state.start_game(Faction::Fox, Difficulty::Medium);

        let c8 = state.graph.find_id_by_name("C8").unwrap();
        let mut cur = state.fox_pos;
        let mut visited_c8 = cur == c8;
        for _ in 0..16 {
            match step_board_cursor(&state, cur, NavDir::Down) {
                BoardNavStep::Node(next) => {
                    cur = next;
                    if cur == c8 {
                        visited_c8 = true;
                        break;
                    }
                }
                _ => break,
            }
        }
        assert!(
            visited_c8,
            "D-pad Down should be able to reach C8 from C0 on FoxAndDogsMaze"
        );
    }

    #[test]
    fn test_step_board_cursor_hound_selection_and_retreat_fallback() {
        let mut state = GameState::new();
        state.switch_variant(BoardVariant::Classic);
        state.start_game(Faction::Hounds, Difficulty::Medium);

        // Complete AI Fox opening move so it is Hounds' turn
        let ai_dest = state.fox_legal_moves()[0];
        state.apply_fox_move(ai_dest).unwrap();
        state.active_anim = None;
        assert_eq!(state.current_turn, Faction::Hounds);

        // When no Hound is selected, Up/Down/Right switches between Hounds (M0, T1, B1)
        let mid_hound = state.hounds_pos[0]; // M0
        let up_to_t1 = step_board_cursor(&state, mid_hound, NavDir::Up);
        let down_to_b1 = step_board_cursor(&state, mid_hound, NavDir::Down);
        assert_eq!(up_to_t1, BoardNavStep::Node(state.hounds_pos[1]));
        assert_eq!(down_to_b1, BoardNavStep::Node(state.hounds_pos[2]));

        // Select Hound 1 (at T1) and advance it right to T2 so T1 is behind it
        let t2 = state.graph.find_id_by_name("T2").unwrap();
        state.apply_hound_move(1, t2).unwrap();
        state.active_anim = None;
        let fox_next = state.fox_legal_moves()[0];
        state.apply_fox_move(fox_next).unwrap();
        state.active_anim = None;

        state.selected_hound_idx = Some(1);
        let h1_pos = state.hounds_pos[1];
        // Pressing Left (towards M0/T1, which is an illegal retreat on Classic) falls back to Tier 2 neighbor T1
        let retreat_step = step_board_cursor(&state, h1_pos, NavDir::Left);
        let t1 = state.graph.find_id_by_name("T1").unwrap();
        assert_eq!(retreat_step, BoardNavStep::Node(t1));
    }
}
