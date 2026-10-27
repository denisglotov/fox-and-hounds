use super::graph::Graph;
use super::i18n::{detect_locale_tag, resolve_locale, LocaleStrings};
use super::level::BoardVariant;
use crate::audio::SoundTrigger;
use macroquad::prelude::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    NotYourTurn,
    IllegalMove,
    InvalidHound,
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveError::NotYourTurn => write!(f, "Not your turn"),
            MoveError::IllegalMove => write!(f, "Illegal move"),
            MoveError::InvalidHound => write!(f, "Invalid hound index"),
        }
    }
}

impl std::error::Error for MoveError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Faction {
    Fox,
    Hounds,
}

impl Faction {
    pub const fn toggle(self) -> Self {
        match self {
            Self::Fox => Self::Hounds,
            Self::Hounds => Self::Fox,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub const ALL: [Self; 3] = [Self::Easy, Self::Medium, Self::Hard];

    pub const fn cycle_prev(self) -> Self {
        match self {
            Self::Easy => Self::Hard,
            Self::Medium => Self::Easy,
            Self::Hard => Self::Medium,
        }
    }

    pub const fn cycle_next(self) -> Self {
        match self {
            Self::Easy => Self::Medium,
            Self::Medium => Self::Hard,
            Self::Hard => Self::Easy,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }

    pub fn localized_name(self, locales: &LocaleStrings) -> &str {
        locales.difficulty_name(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    TitleScreen,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Ongoing,
    FoxWon,
    HoundsWon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceMove {
    FoxMove { to: u8 },
    HoundMove { hound_idx: u8, from: u8, to: u8 },
}

#[derive(Debug, Clone)]
pub struct MoveAnimation {
    pub from: Vec2,
    pub to: Vec2,
    pub progress: f32,
    pub duration: f32,
    pub faction: Faction,
    pub hound_idx: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub graph: Graph,
    pub variant: BoardVariant,
    pub is_c8_mode: bool,
    pub fox_pos: u8,
    pub fox_pending: bool,
    pub fox_has_left_start: bool,
    pub hounds_pos: Vec<u8>,
    pub coop_pos: u8,
    pub current_turn: Faction,
    pub player_faction: Faction,
    pub difficulty: Difficulty,
    pub phase: GamePhase,
    pub result: GameResult,
    pub selected_hound_idx: Option<u8>,
    pub turn_count: usize,
    pub move_history: Vec<PieceMove>,
    pub ai_think_delay: f32,
    pub active_anim: Option<MoveAnimation>,
    pub locales: &'static LocaleStrings,
    pub cached_game_over_stats: Option<String>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let detected = detect_locale_tag();
        let locales = resolve_locale(&detected);
        let variant = BoardVariant::default();
        let graph = (variant.config().build_graph)();

        let mut state = Self {
            graph,
            variant,
            is_c8_mode: false,
            fox_pos: 0,
            fox_pending: false,
            fox_has_left_start: false,
            hounds_pos: Vec::new(),
            coop_pos: 0,
            current_turn: Faction::Fox,
            player_faction: Faction::Fox,
            difficulty: Difficulty::Medium,
            phase: GamePhase::TitleScreen,
            result: GameResult::Ongoing,
            selected_hound_idx: None,
            turn_count: 1,
            move_history: Vec::new(),
            ai_think_delay: 0.0,
            active_anim: None,
            locales,
            cached_game_over_stats: None,
        };
        state.reset_board();
        state
    }

    pub fn set_locale(&mut self, tag: &str) {
        self.locales = resolve_locale(tag);
        if self.result != GameResult::Ongoing {
            self.cached_game_over_stats = Some(self.locales.game_over.format_stats(
                self.turn_count,
                self.difficulty.localized_name(self.locales),
                self.variant.localized_name(self.locales),
            ));
        }
    }

    pub fn switch_variant(&mut self, variant: BoardVariant) {
        if self.variant != variant {
            self.variant = variant;
            self.is_c8_mode = false;
            self.graph = (variant.config().build_graph)();
            self.reset_board();
        }
    }

    #[inline]
    pub fn active_config(&self) -> &'static super::level::VariantConfig {
        if self.is_c8_mode {
            match self.variant {
                BoardVariant::FoxAndDogsMaze => &super::level::FOX_AND_DOGS_MAZE_C8_CONFIG,
                BoardVariant::FoxAndDogs => &super::level::FOX_AND_DOGS_C8_CONFIG,
                _ => self.variant.config(),
            }
        } else {
            self.variant.config()
        }
    }

    pub fn set_c8_mode(&mut self, enable: bool) {
        if !self.variant.is_fox_and_dogs() {
            return;
        }
        self.is_c8_mode = enable;
        self.apply_current_config();
        if enable && self.is_ai_turn() {
            self.ai_think_delay = 1.0;
        }
    }

    pub fn start_game(&mut self, player_faction: Faction, difficulty: Difficulty) {
        self.player_faction = player_faction;
        self.difficulty = difficulty;
        self.reset_board();
        self.phase = GamePhase::Playing;
    }

    pub fn reset_board(&mut self) {
        self.is_c8_mode = false;
        self.apply_current_config();
    }

    fn apply_current_config(&mut self) {
        let config = self.active_config();
        self.fox_pos = self
            .graph
            .find_id_by_name(config.fox_start_node)
            .unwrap_or_else(|| self.graph.nodes.len().saturating_sub(1) as u8);
        self.fox_pending = config.fox_free_entry;
        self.fox_has_left_start = false;
        self.hounds_pos = config
            .hounds_start_nodes
            .iter()
            .filter_map(|name| self.graph.find_id_by_name(name))
            .collect();
        self.coop_pos = self
            .graph
            .find_id_by_name(config.target_coop_node)
            .unwrap_or(0);
        self.current_turn = if config.hounds_start_first {
            Faction::Hounds
        } else {
            Faction::Fox
        };
        self.result = GameResult::Ongoing;
        self.selected_hound_idx = None;
        self.turn_count = 1;
        self.move_history.clear();
        self.active_anim = None;
        self.cached_game_over_stats = None;
        self.ai_think_delay = if self.is_ai_turn() { 0.4 } else { 0.0 };
    }

    #[inline]
    pub fn move_duration(&self) -> f32 {
        self.active_config().move_duration
    }

    pub fn is_ai_turn(&self) -> bool {
        self.phase == GamePhase::Playing
            && self.result == GameResult::Ongoing
            && self.current_turn != self.player_faction
    }

    pub const fn active_target_node(&self) -> u8 {
        self.coop_pos
    }

    #[inline]
    pub fn is_fox_move_legal(&self, to: u8) -> bool {
        if self.fox_pending {
            self.graph
                .fox_entry_moves(&self.hounds_pos, self.coop_pos)
                .any(|t| t == to)
        } else {
            self.graph
                .fox_legal_moves(self.fox_pos, &self.hounds_pos)
                .any(|t| t == to)
        }
    }

    #[inline]
    pub fn has_legal_fox_moves(&self) -> bool {
        if self.fox_pending {
            self.graph
                .fox_entry_moves(&self.hounds_pos, self.coop_pos)
                .next()
                .is_some()
        } else {
            self.graph
                .fox_legal_moves(self.fox_pos, &self.hounds_pos)
                .next()
                .is_some()
        }
    }

    #[inline]
    pub fn is_hound_move_legal(&self, hound_idx: u8, to: u8) -> bool {
        let allow_retreat = self.active_config().allow_hound_retreat;
        let allow_coop = self.active_config().allow_hounds_in_coop;
        self.hounds_pos.get(hound_idx as usize).is_some_and(|&pos| {
            self.graph
                .hound_legal_moves(
                    pos,
                    self.fox_pos,
                    self.coop_pos,
                    &self.hounds_pos,
                    allow_retreat,
                    allow_coop,
                )
                .any(|t| t == to)
        })
    }

    #[inline]
    pub fn has_legal_hound_moves(&self) -> bool {
        let allow_retreat = self.active_config().allow_hound_retreat;
        let allow_coop = self.active_config().allow_hounds_in_coop;
        self.hounds_pos.iter().any(|&pos| {
            self.graph
                .hound_legal_moves(
                    pos,
                    self.fox_pos,
                    self.coop_pos,
                    &self.hounds_pos,
                    allow_retreat,
                    allow_coop,
                )
                .next()
                .is_some()
        })
    }

    pub fn fox_legal_moves(&self) -> Vec<u8> {
        if self.fox_pending {
            self.graph
                .fox_entry_moves(&self.hounds_pos, self.coop_pos)
                .collect()
        } else {
            self.graph
                .fox_legal_moves(self.fox_pos, &self.hounds_pos)
                .collect()
        }
    }

    pub fn hound_legal_moves(&self, hound_idx: u8) -> Vec<u8> {
        let allow_retreat = self.active_config().allow_hound_retreat;
        let allow_coop = self.active_config().allow_hounds_in_coop;
        self.hounds_pos
            .get(hound_idx as usize)
            .map_or_else(Vec::new, |&pos| {
                self.graph
                    .hound_legal_moves(
                        pos,
                        self.fox_pos,
                        self.coop_pos,
                        &self.hounds_pos,
                        allow_retreat,
                        allow_coop,
                    )
                    .collect()
            })
    }

    pub fn all_hound_legal_moves(&self) -> Vec<(u8, u8)> {
        (0..self.hounds_pos.len() as u8)
            .flat_map(|idx| {
                self.hound_legal_moves(idx)
                    .into_iter()
                    .map(move |target| (idx, target))
            })
            .collect()
    }

    pub fn apply_fox_move(&mut self, to: u8) -> Result<(), MoveError> {
        if self.current_turn != Faction::Fox {
            return Err(MoveError::NotYourTurn);
        }
        if !self.is_fox_move_legal(to) {
            return Err(MoveError::IllegalMove);
        }

        let from_pos = self.fox_pos;
        let from_visual = self
            .graph
            .node(from_pos)
            .map_or(Vec2::ZERO, |n| n.visual_pos);
        let to_visual = self.graph.node(to).map_or(Vec2::ZERO, |n| n.visual_pos);

        self.fox_pos = to;
        self.fox_pending = false;
        if to != self.coop_pos {
            self.fox_has_left_start = true;
        }
        self.move_history.push(PieceMove::FoxMove { to });
        self.current_turn = Faction::Hounds;
        self.selected_hound_idx = None;
        if self.active_config().hounds_start_first {
            self.turn_count += 1;
        }
        self.ai_think_delay = if self.player_faction == Faction::Fox {
            0.35
        } else {
            0.0
        };

        self.active_anim = Some(MoveAnimation {
            from: from_visual,
            to: to_visual,
            progress: 0.0,
            duration: self.move_duration(),
            faction: Faction::Fox,
            hound_idx: None,
        });

        self.evaluate_game_result();
        Ok(())
    }

    pub fn apply_hound_move(&mut self, hound_idx: u8, to: u8) -> Result<(), MoveError> {
        if self.current_turn != Faction::Hounds {
            return Err(MoveError::NotYourTurn);
        }
        if hound_idx as usize >= self.hounds_pos.len() {
            return Err(MoveError::InvalidHound);
        }
        if !self.is_hound_move_legal(hound_idx, to) {
            return Err(MoveError::IllegalMove);
        }

        let from_pos = self.hounds_pos[hound_idx as usize];
        let from_visual = self
            .graph
            .node(from_pos)
            .map_or(Vec2::ZERO, |n| n.visual_pos);
        let to_visual = self.graph.node(to).map_or(Vec2::ZERO, |n| n.visual_pos);

        self.hounds_pos[hound_idx as usize] = to;
        self.move_history.push(PieceMove::HoundMove {
            hound_idx,
            from: from_pos,
            to,
        });
        self.current_turn = Faction::Fox;
        self.selected_hound_idx = None;
        if !self.active_config().hounds_start_first {
            self.turn_count += 1;
        }
        self.ai_think_delay = if self.player_faction == Faction::Hounds {
            0.35
        } else {
            0.0
        };

        self.active_anim = Some(MoveAnimation {
            from: from_visual,
            to: to_visual,
            progress: 0.0,
            duration: self.move_duration(),
            faction: Faction::Hounds,
            hound_idx: Some(hound_idx),
        });

        self.evaluate_game_result();
        Ok(())
    }

    fn finish_game(&mut self, result: GameResult) {
        self.result = result;
        self.phase = GamePhase::GameOver;
        self.cached_game_over_stats = Some(self.locales.game_over.format_stats(
            self.turn_count,
            self.difficulty.localized_name(self.locales),
            self.variant.localized_name(self.locales),
        ));
    }

    pub fn evaluate_game_result(&mut self) {
        if super::ai::BoardSnapshot::from_state(self).is_fox_win() {
            self.finish_game(GameResult::FoxWon);
            return;
        }

        if self.current_turn == Faction::Fox && !self.has_legal_fox_moves() {
            self.finish_game(GameResult::HoundsWon);
            return;
        }

        if self.current_turn == Faction::Hounds && !self.has_legal_hound_moves() {
            self.finish_game(GameResult::FoxWon);
            return;
        }

        self.result = GameResult::Ongoing;
        self.cached_game_over_stats = None;
    }

    pub fn update(&mut self, dt: f32) -> Option<SoundTrigger> {
        let mut trigger = None;

        // Animate piece movement
        if let Some(anim) = &mut self.active_anim {
            anim.progress += dt / anim.duration;
            if anim.progress >= 1.0 {
                self.active_anim = None;
            }
        }

        // Handle AI thinking timer
        if self.is_ai_turn() && self.active_anim.is_none() {
            if self.ai_think_delay > 0.0 {
                self.ai_think_delay -= dt;
            } else {
                let ai_sound = self.execute_ai_turn();
                if ai_sound.is_some() {
                    trigger = ai_sound;
                }
            }
        }

        trigger
    }

    fn execute_ai_turn(&mut self) -> Option<SoundTrigger> {
        use super::ai::find_best_move;

        let best_move = find_best_move(self);
        match best_move {
            Some(PieceMove::FoxMove { to }) => match self.apply_fox_move(to) {
                Ok(()) => {
                    if self.result == GameResult::FoxWon {
                        Some(if self.player_faction == Faction::Fox {
                            SoundTrigger::Win
                        } else {
                            SoundTrigger::Loss
                        })
                    } else {
                        Some(SoundTrigger::Move)
                    }
                }
                Err(err) => {
                    eprintln!("AI error executing fox move to {}: {:?}", to, err);
                    self.evaluate_game_result();
                    None
                }
            },
            Some(PieceMove::HoundMove { hound_idx, to, .. }) => {
                match self.apply_hound_move(hound_idx, to) {
                    Ok(()) => {
                        if self.result == GameResult::HoundsWon {
                            Some(if self.player_faction == Faction::Hounds {
                                SoundTrigger::Win
                            } else {
                                SoundTrigger::Loss
                            })
                        } else {
                            Some(SoundTrigger::Move)
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "AI error executing hound {} move to {}: {:?}",
                            hound_idx, to, err
                        );
                        self.evaluate_game_result();
                        None
                    }
                }
            }
            None => {
                self.evaluate_game_result();
                None
            }
        }
    }
}
