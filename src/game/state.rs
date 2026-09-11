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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
    Fox,
    Hounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
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
    FoxMove {
        to: usize,
    },
    HoundMove {
        hound_idx: usize,
        from: usize,
        to: usize,
    },
}

#[derive(Debug, Clone)]
pub struct MoveAnimation {
    pub from: Vec2,
    pub to: Vec2,
    pub progress: f32,
    pub duration: f32,
    pub faction: Faction,
    pub hound_idx: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub graph: Graph,
    pub variant: BoardVariant,
    pub fox_pos: usize,
    pub fox_pending: bool,
    pub hounds_pos: Vec<usize>,
    pub coop_pos: usize,
    pub waypoint_pos: Option<usize>,
    pub fox_visited_waypoint: bool,
    pub fox_waiting_at_waypoint: bool,
    pub waypoint_wait_timer: f32,
    pub current_turn: Faction,
    pub player_faction: Faction,
    pub difficulty: Difficulty,
    pub phase: GamePhase,
    pub result: GameResult,
    pub selected_hound_idx: Option<usize>,
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
        let variant = BoardVariant::Classic;
        let graph = (variant.config().build_graph)();

        let mut state = Self {
            graph,
            variant,
            fox_pos: 0,
            fox_pending: false,
            hounds_pos: Vec::new(),
            coop_pos: 0,
            waypoint_pos: None,
            fox_visited_waypoint: false,
            fox_waiting_at_waypoint: false,
            waypoint_wait_timer: 0.0,
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
            self.graph = (variant.config().build_graph)();
            self.reset_board();
        }
    }

    pub fn start_game(&mut self, player_faction: Faction, difficulty: Difficulty) {
        self.player_faction = player_faction;
        self.difficulty = difficulty;
        self.reset_board();
        self.phase = GamePhase::Playing;
    }

    pub fn reset_board(&mut self) {
        let config = self.variant.config();
        self.fox_pos = self
            .graph
            .find_id_by_name(config.fox_start_node)
            .unwrap_or_else(|| self.graph.nodes.len().saturating_sub(1));
        self.fox_pending = config.fox_free_entry;
        self.hounds_pos = config
            .hounds_start_nodes
            .iter()
            .filter_map(|name| self.graph.find_id_by_name(name))
            .collect();
        self.coop_pos = self
            .graph
            .find_id_by_name(config.target_coop_node)
            .unwrap_or(0);
        self.waypoint_pos = config
            .target_waypoint_node
            .and_then(|name| self.graph.find_id_by_name(name));
        self.fox_visited_waypoint = false;
        self.fox_waiting_at_waypoint = false;
        self.waypoint_wait_timer = 0.0;
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

    pub fn is_ai_turn(&self) -> bool {
        self.phase == GamePhase::Playing
            && self.result == GameResult::Ongoing
            && !self.fox_waiting_at_waypoint
            && self.current_turn != self.player_faction
    }

    pub fn active_target_node(&self) -> usize {
        if let Some(waypoint) = self.waypoint_pos {
            if !self.fox_visited_waypoint {
                return waypoint;
            }
        }
        self.coop_pos
    }

    pub fn fox_legal_moves(&self) -> Vec<usize> {
        if self.fox_waiting_at_waypoint {
            return Vec::new();
        }
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

    pub fn hound_legal_moves(&self, hound_idx: usize) -> Vec<usize> {
        let allow_retreat = self.variant.config().allow_hound_retreat;
        self.hounds_pos
            .get(hound_idx)
            .map_or_else(Vec::new, |&pos| {
                self.graph
                    .hound_legal_moves(
                        pos,
                        self.fox_pos,
                        self.coop_pos,
                        &self.hounds_pos,
                        allow_retreat,
                    )
                    .collect()
            })
    }

    pub fn all_hound_legal_moves(&self) -> Vec<(usize, usize)> {
        let allow_retreat = self.variant.config().allow_hound_retreat;
        self.hounds_pos
            .iter()
            .enumerate()
            .flat_map(|(idx, &pos)| {
                self.graph
                    .hound_legal_moves(
                        pos,
                        self.fox_pos,
                        self.coop_pos,
                        &self.hounds_pos,
                        allow_retreat,
                    )
                    .map(move |target| (idx, target))
            })
            .collect()
    }

    pub fn apply_fox_move(&mut self, to: usize) -> Result<(), MoveError> {
        if self.current_turn != Faction::Fox {
            return Err(MoveError::NotYourTurn);
        }
        let legal = self.fox_legal_moves();
        if !legal.contains(&to) {
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
        if self.waypoint_pos == Some(to) && !self.fox_visited_waypoint {
            self.fox_waiting_at_waypoint = true;
            self.waypoint_wait_timer = 0.0;
        }
        self.move_history.push(PieceMove::FoxMove { to });
        self.current_turn = Faction::Hounds;
        self.selected_hound_idx = None;
        self.ai_think_delay = if self.player_faction == Faction::Fox {
            0.35
        } else {
            0.0
        };

        self.active_anim = Some(MoveAnimation {
            from: from_visual,
            to: to_visual,
            progress: 0.0,
            duration: 0.26,
            faction: Faction::Fox,
            hound_idx: None,
        });

        self.evaluate_game_result();
        Ok(())
    }

    pub fn apply_hound_move(&mut self, hound_idx: usize, to: usize) -> Result<(), MoveError> {
        if self.current_turn != Faction::Hounds {
            return Err(MoveError::NotYourTurn);
        }
        if hound_idx >= self.hounds_pos.len() {
            return Err(MoveError::InvalidHound);
        }
        let legal = self.hound_legal_moves(hound_idx);
        if !legal.contains(&to) {
            return Err(MoveError::IllegalMove);
        }

        let from_pos = self.hounds_pos[hound_idx];
        let from_visual = self
            .graph
            .node(from_pos)
            .map_or(Vec2::ZERO, |n| n.visual_pos);
        let to_visual = self.graph.node(to).map_or(Vec2::ZERO, |n| n.visual_pos);

        self.hounds_pos[hound_idx] = to;
        self.move_history.push(PieceMove::HoundMove {
            hound_idx,
            from: from_pos,
            to,
        });
        self.current_turn = Faction::Fox;
        self.selected_hound_idx = None;
        self.turn_count += 1;
        self.ai_think_delay = if self.player_faction == Faction::Hounds {
            0.35
        } else {
            0.0
        };

        self.active_anim = Some(MoveAnimation {
            from: from_visual,
            to: to_visual,
            progress: 0.0,
            duration: 0.26,
            faction: Faction::Hounds,
            hound_idx: Some(hound_idx),
        });

        self.evaluate_game_result();
        Ok(())
    }

    pub fn evaluate_game_result(&mut self) {
        let fox_won = match self.waypoint_pos {
            Some(_) => self.fox_visited_waypoint && self.fox_pos == self.coop_pos,
            None => self.fox_pos == self.coop_pos,
        };
        if fox_won {
            self.result = GameResult::FoxWon;
            self.phase = GamePhase::GameOver;
            self.cached_game_over_stats = Some(self.locales.game_over.format_stats(
                self.turn_count,
                self.difficulty.localized_name(self.locales),
                self.variant.localized_name(self.locales),
            ));
            return;
        }

        if self.current_turn == Faction::Fox
            && !self.fox_waiting_at_waypoint
            && self.fox_legal_moves().is_empty()
        {
            self.result = GameResult::HoundsWon;
            self.phase = GamePhase::GameOver;
            self.cached_game_over_stats = Some(self.locales.game_over.format_stats(
                self.turn_count,
                self.difficulty.localized_name(self.locales),
                self.variant.localized_name(self.locales),
            ));
            return;
        }

        if self.current_turn == Faction::Hounds && self.all_hound_legal_moves().is_empty() {
            self.result = GameResult::FoxWon;
            self.phase = GamePhase::GameOver;
            self.cached_game_over_stats = Some(self.locales.game_over.format_stats(
                self.turn_count,
                self.difficulty.localized_name(self.locales),
                self.variant.localized_name(self.locales),
            ));
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

        // Automatic waypoint wait / prey grabbing turn
        if self.phase == GamePhase::Playing
            && self.result == GameResult::Ongoing
            && self.current_turn == Faction::Fox
            && self.fox_waiting_at_waypoint
            && self.active_anim.is_none()
        {
            self.waypoint_wait_timer += dt;
            if self.waypoint_wait_timer >= 0.5 {
                self.waypoint_wait_timer = 0.0;
                self.fox_waiting_at_waypoint = false;
                self.fox_visited_waypoint = true;
                self.current_turn = Faction::Hounds;
                self.turn_count += 1;
                self.ai_think_delay = if self.player_faction == Faction::Fox {
                    0.35
                } else {
                    0.0
                };
                self.evaluate_game_result();
                trigger = Some(SoundTrigger::Select);
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
            Some(PieceMove::FoxMove { to }) => {
                if self.apply_fox_move(to).is_ok() {
                    if self.result == GameResult::FoxWon {
                        Some(if self.player_faction == Faction::Fox {
                            SoundTrigger::Win
                        } else {
                            SoundTrigger::Loss
                        })
                    } else {
                        Some(SoundTrigger::Move)
                    }
                } else {
                    None
                }
            }
            Some(PieceMove::HoundMove { hound_idx, to, .. }) => {
                if self.apply_hound_move(hound_idx, to).is_ok() {
                    if self.result == GameResult::HoundsWon {
                        Some(if self.player_faction == Faction::Hounds {
                            SoundTrigger::Win
                        } else {
                            SoundTrigger::Loss
                        })
                    } else {
                        Some(SoundTrigger::Move)
                    }
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
