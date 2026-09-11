use super::graph::Graph;
use super::state::{Difficulty, Faction, GameState, PieceMove};

const WIN_SCORE: i32 = 100_000;
const INF: i32 = 1_000_000;

#[derive(Debug, Clone, Copy)]
pub struct BoardSnapshot {
    pub fox_pos: usize,
    pub fox_pending: bool,
    pub hounds_pos: [usize; 3],
    pub coop_pos: usize,
    pub current_turn: Faction,
    pub allow_hound_retreat: bool,
}

impl BoardSnapshot {
    pub fn from_state(state: &GameState) -> Self {
        let mut hounds = [0; 3];
        for (i, &pos) in state.hounds_pos.iter().take(3).enumerate() {
            hounds[i] = pos;
        }
        Self {
            fox_pos: state.fox_pos,
            fox_pending: state.fox_pending,
            hounds_pos: hounds,
            coop_pos: state.coop_pos,
            current_turn: state.current_turn,
            allow_hound_retreat: state.variant.config().allow_hound_retreat,
        }
    }

    pub const fn active_target(&self) -> usize {
        self.coop_pos
    }

    pub const fn is_fox_win(&self) -> bool {
        self.fox_pos == self.coop_pos
    }

    pub fn fox_legal_moves<'a>(&'a self, graph: &'a Graph) -> Box<dyn Iterator<Item = usize> + 'a> {
        if self.fox_pending {
            Box::new(graph.fox_entry_moves(&self.hounds_pos, self.coop_pos))
        } else {
            Box::new(graph.fox_legal_moves(self.fox_pos, &self.hounds_pos))
        }
    }

    pub fn hound_legal_moves<'a>(
        &'a self,
        graph: &'a Graph,
        hound_idx: usize,
    ) -> impl Iterator<Item = usize> + 'a {
        let pos = self.hounds_pos[hound_idx];
        graph.hound_legal_moves(
            pos,
            self.fox_pos,
            self.coop_pos,
            &self.hounds_pos,
            self.allow_hound_retreat,
        )
    }

    pub fn all_hound_moves<'a>(
        &'a self,
        graph: &'a Graph,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        (0..self.hounds_pos.len()).flat_map(move |idx| {
            self.hound_legal_moves(graph, idx)
                .map(move |target| (idx, target))
        })
    }

    pub fn apply_fox_move(&self, to: usize) -> Self {
        Self {
            fox_pos: to,
            fox_pending: false,
            hounds_pos: self.hounds_pos,
            coop_pos: self.coop_pos,
            current_turn: Faction::Hounds,
            allow_hound_retreat: self.allow_hound_retreat,
        }
    }

    pub fn apply_hound_move(&self, hound_idx: usize, to: usize) -> Self {
        let mut new_hounds = self.hounds_pos;
        new_hounds[hound_idx] = to;
        Self {
            fox_pos: self.fox_pos,
            fox_pending: self.fox_pending,
            hounds_pos: new_hounds,
            coop_pos: self.coop_pos,
            current_turn: Faction::Fox,
            allow_hound_retreat: self.allow_hound_retreat,
        }
    }
}

pub fn find_best_move(state: &GameState) -> Option<PieceMove> {
    let snapshot = BoardSnapshot::from_state(state);
    let depth = match state.difficulty {
        Difficulty::Easy => 2,
        Difficulty::Medium => 4,
        Difficulty::Hard => 6,
    };

    match state.current_turn {
        Faction::Fox => find_best_fox_move(&snapshot, &state.graph, depth, state.difficulty),
        Faction::Hounds => find_best_hound_move(&snapshot, &state.graph, depth, state.difficulty),
    }
}

fn find_best_fox_move(
    board: &BoardSnapshot,
    graph: &Graph,
    max_depth: usize,
    difficulty: Difficulty,
) -> Option<PieceMove> {
    let moves: Vec<usize> = board.fox_legal_moves(graph).collect();
    if moves.is_empty() {
        return None;
    }

    // If any move achieves direct win, take it immediately
    if let Some(&direct_win) = moves
        .iter()
        .find(|&&m| board.apply_fox_move(m).is_fox_win())
    {
        return Some(PieceMove::FoxMove { to: direct_win });
    }

    let mut best_score = -INF;
    let mut candidate_moves = Vec::new();

    for &to in &moves {
        let next_board = board.apply_fox_move(to);
        let score = minimax(&next_board, graph, max_depth - 1, -INF, INF, false);

        if score > best_score {
            best_score = score;
            candidate_moves.clear();
            candidate_moves.push(to);
        } else if score == best_score {
            candidate_moves.push(to);
        }
    }

    // In Easy difficulty, occasionally choose random candidate if available
    let chosen = match difficulty {
        Difficulty::Easy if candidate_moves.len() > 1 => {
            let idx = (macroquad::rand::gen_range(0, candidate_moves.len())) as usize;
            candidate_moves[idx]
        }
        _ => {
            // Pick candidate with best immediate static evaluation
            *candidate_moves
                .iter()
                .max_by_key(|&&to| {
                    let next_b = board.apply_fox_move(to);
                    evaluate_board(&next_b, graph)
                })
                .unwrap_or(&candidate_moves[0])
        }
    };

    Some(PieceMove::FoxMove { to: chosen })
}

fn find_best_hound_move(
    board: &BoardSnapshot,
    graph: &Graph,
    max_depth: usize,
    difficulty: Difficulty,
) -> Option<PieceMove> {
    let moves: Vec<(usize, usize)> = board.all_hound_moves(graph).collect();
    if moves.is_empty() {
        return None;
    }

    let mut best_score = INF; // Hounds minimize Fox's score
    let mut candidate_moves = Vec::new();

    for &(hound_idx, to) in &moves {
        let next_board = board.apply_hound_move(hound_idx, to);
        let score = minimax(&next_board, graph, max_depth - 1, -INF, INF, true);

        if score < best_score {
            best_score = score;
            candidate_moves.clear();
            candidate_moves.push((hound_idx, to));
        } else if score == best_score {
            candidate_moves.push((hound_idx, to));
        }
    }

    let chosen = match difficulty {
        Difficulty::Easy if candidate_moves.len() > 1 => {
            let idx = (macroquad::rand::gen_range(0, candidate_moves.len())) as usize;
            candidate_moves[idx]
        }
        _ => {
            // Pick candidate with lowest (best for Hounds) immediate static evaluation
            *candidate_moves
                .iter()
                .min_by_key(|&&(hound_idx, to)| {
                    let next_b = board.apply_hound_move(hound_idx, to);
                    evaluate_board(&next_b, graph)
                })
                .unwrap_or(&candidate_moves[0])
        }
    };

    Some(PieceMove::HoundMove {
        hound_idx: chosen.0,
        from: board.hounds_pos[chosen.0],
        to: chosen.1,
    })
}

pub fn minimax(
    board: &BoardSnapshot,
    graph: &Graph,
    depth: usize,
    mut alpha: i32,
    mut beta: i32,
    is_fox_turn: bool,
) -> i32 {
    // 1. Terminal condition checks
    if board.is_fox_win() {
        return WIN_SCORE + (depth as i32 * 100);
    }

    if is_fox_turn {
        let mut fox_moves = board.fox_legal_moves(graph).peekable();
        if fox_moves.peek().is_none() {
            return -WIN_SCORE - (depth as i32 * 100);
        }

        if depth == 0 {
            return evaluate_board(board, graph);
        }

        let mut max_eval = -INF;
        for to in fox_moves {
            let next_board = board.apply_fox_move(to);
            let eval = minimax(&next_board, graph, depth - 1, alpha, beta, false);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break; // Alpha-beta cutoff
            }
        }
        max_eval
    } else {
        let mut hound_moves = board.all_hound_moves(graph).peekable();
        if hound_moves.peek().is_none() {
            // Hounds have no moves on their turn: Fox wins immediately
            return WIN_SCORE + (depth as i32 * 100);
        }

        if depth == 0 {
            return evaluate_board(board, graph);
        }

        let mut min_eval = INF;
        for (hound_idx, to) in hound_moves {
            let next_board = board.apply_hound_move(hound_idx, to);
            let eval = minimax(&next_board, graph, depth - 1, alpha, beta, true);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break; // Alpha-beta cutoff
            }
        }
        min_eval
    }
}

/// Evaluation score from Fox perspective (positive = Fox advantage, negative = Hounds advantage)
pub fn evaluate_board(board: &BoardSnapshot, graph: &Graph) -> i32 {
    if board.is_fox_win() {
        return WIN_SCORE;
    }

    let target = board.active_target();
    let dist_to_target = graph.distance(board.fox_pos, target).unwrap_or(10);
    let dist_to_target_score = (12 - dist_to_target as i32) * 160;

    let total_hound_dist: i32 = board
        .hounds_pos
        .iter()
        .map(|&h_pos| graph.distance(h_pos, board.fox_pos).unwrap_or(10) as i32)
        .sum();
    let pursuit_score = total_hound_dist * 80;

    let fox_degrees = board.fox_legal_moves(graph).count();
    let mobility_score = match fox_degrees {
        0 => -WIN_SCORE,
        1 => -2_000,
        2 => -400,
        3 => 200,
        _ => 600,
    };

    let close_hounds = board
        .hounds_pos
        .iter()
        .filter(|&&h| graph.neighbors(board.fox_pos).contains(&h))
        .count() as i32;
    let pressure_penalty = close_hounds * -400;

    dist_to_target_score + pursuit_score + mobility_score + pressure_penalty
}
