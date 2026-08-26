use super::graph::{Graph, NodeType};
use super::i18n::{detect_locale_tag, resolve_locale, LocaleStrings};
use super::level::{build_river_crossing_graph, RIVER_CROSSING_CONFIG};
use crate::audio::SoundTrigger;
use macroquad::prelude::Vec2;

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
    pub fox_pos: usize,
    pub hounds_pos: Vec<usize>,
    pub coop_pos: usize,
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
        let graph = build_river_crossing_graph();
        let fox_pos = graph
            .find_id_by_name(RIVER_CROSSING_CONFIG.fox_start_node)
            .unwrap_or_else(|| graph.nodes.len().saturating_sub(1));
        let hounds_pos = RIVER_CROSSING_CONFIG
            .hounds_start_nodes
            .iter()
            .filter_map(|name| graph.find_id_by_name(name))
            .collect();
        let coop_pos = graph
            .find_id_by_name(RIVER_CROSSING_CONFIG.target_coop_node)
            .unwrap_or(0);

        Self {
            graph,
            fox_pos,
            hounds_pos,
            coop_pos,
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
        }
    }

    pub fn set_locale(&mut self, tag: &str) {
        self.locales = resolve_locale(tag);
    }

    pub fn start_game(&mut self, player_faction: Faction, difficulty: Difficulty) {
        self.player_faction = player_faction;
        self.difficulty = difficulty;
        self.reset_board();
        self.phase = GamePhase::Playing;
    }

    pub fn reset_board(&mut self) {
        self.fox_pos = self
            .graph
            .find_id_by_name(RIVER_CROSSING_CONFIG.fox_start_node)
            .unwrap_or_else(|| self.graph.nodes.len().saturating_sub(1));
        self.hounds_pos = RIVER_CROSSING_CONFIG
            .hounds_start_nodes
            .iter()
            .filter_map(|name| self.graph.find_id_by_name(name))
            .collect();
        self.coop_pos = self
            .graph
            .find_id_by_name(RIVER_CROSSING_CONFIG.target_coop_node)
            .unwrap_or(0);
        self.current_turn = Faction::Fox;
        self.result = GameResult::Ongoing;
        self.selected_hound_idx = None;
        self.turn_count = 1;
        self.move_history.clear();
        self.ai_think_delay = if self.player_faction == Faction::Hounds {
            0.4
        } else {
            0.0
        };
        self.active_anim = None;
    }

    pub fn is_ai_turn(&self) -> bool {
        self.phase == GamePhase::Playing
            && self.result == GameResult::Ongoing
            && self.current_turn != self.player_faction
    }

    pub fn fox_legal_moves(&self) -> Vec<usize> {
        self.graph
            .neighbors(self.fox_pos)
            .iter()
            .copied()
            .filter(|&target| !self.hounds_pos.contains(&target))
            .collect()
    }

    pub fn hound_legal_moves(&self, hound_idx: usize) -> Vec<usize> {
        self.hounds_pos
            .get(hound_idx)
            .map_or_else(Vec::new, |&pos| {
                self.graph
                    .neighbors(pos)
                    .iter()
                    .copied()
                    .filter(|&target| {
                        target != self.fox_pos
                            && target != self.coop_pos
                            && !self.hounds_pos.contains(&target)
                            && self
                                .graph
                                .node(target)
                                .is_none_or(|n| n.node_type != NodeType::TargetCoop)
                    })
                    .collect()
            })
    }

    pub fn all_hound_legal_moves(&self) -> Vec<(usize, usize)> {
        self.hounds_pos
            .iter()
            .enumerate()
            .flat_map(|(idx, _)| {
                self.hound_legal_moves(idx)
                    .into_iter()
                    .map(move |target| (idx, target))
            })
            .collect()
    }

    pub fn apply_fox_move(&mut self, to: usize) -> Result<(), &'static str> {
        if self.current_turn != Faction::Fox {
            return Err("Not Fox's turn");
        }
        let legal = self.fox_legal_moves();
        if !legal.contains(&to) {
            return Err("Illegal move for Fox");
        }

        let from_pos = self.fox_pos;
        let from_visual = self
            .graph
            .node(from_pos)
            .map_or(Vec2::ZERO, |n| n.visual_pos);
        let to_visual = self.graph.node(to).map_or(Vec2::ZERO, |n| n.visual_pos);

        self.fox_pos = to;
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

    pub fn apply_hound_move(&mut self, hound_idx: usize, to: usize) -> Result<(), &'static str> {
        if self.current_turn != Faction::Hounds {
            return Err("Not Hounds' turn");
        }
        if hound_idx >= self.hounds_pos.len() {
            return Err("Invalid hound index");
        }
        let legal = self.hound_legal_moves(hound_idx);
        if !legal.contains(&to) {
            return Err("Illegal move for Hound");
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
        if self.fox_pos == self.coop_pos {
            self.result = GameResult::FoxWon;
            self.phase = GamePhase::GameOver;
            return;
        }

        if self.current_turn == Faction::Fox && self.fox_legal_moves().is_empty() {
            self.result = GameResult::HoundsWon;
            self.phase = GamePhase::GameOver;
            return;
        }

        self.result = GameResult::Ongoing;
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
