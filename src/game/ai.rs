use super::graph::{Graph, NodeType};
use super::state::{Difficulty, Faction, GameState, PieceMove};

const WIN_SCORE: i32 = 100_000;
const INF: i32 = 1_000_000;

#[derive(Debug, Clone, Copy)]
pub struct BoardSnapshot {
    pub fox_pos: usize,
    pub hounds_pos: [usize; 3],
    pub coop_pos: usize,
    pub current_turn: Faction,
}

impl BoardSnapshot {
    pub fn from_state(state: &GameState) -> Self {
        let mut hounds = [0; 3];
        for (i, &pos) in state.hounds_pos.iter().take(3).enumerate() {
            hounds[i] = pos;
        }
        Self {
            fox_pos: state.fox_pos,
            hounds_pos: hounds,
            coop_pos: state.coop_pos,
            current_turn: state.current_turn,
        }
    }

    pub fn fox_legal_moves<'a>(&'a self, graph: &'a Graph) -> impl Iterator<Item = usize> + 'a {
        graph.fox_legal_moves(self.fox_pos, &self.hounds_pos)
    }

    pub fn hound_legal_moves<'a>(
        &'a self,
        graph: &'a Graph,
        hound_idx: usize,
    ) -> impl Iterator<Item = usize> + 'a {
        let pos = self.hounds_pos[hound_idx];
        graph.hound_legal_moves(pos, self.fox_pos, self.coop_pos, &self.hounds_pos)
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
            hounds_pos: self.hounds_pos,
            coop_pos: self.coop_pos,
            current_turn: Faction::Hounds,
        }
    }

    pub fn apply_hound_move(&self, hound_idx: usize, to: usize) -> Self {
        let mut new_hounds = self.hounds_pos;
        new_hounds[hound_idx] = to;
        Self {
            fox_pos: self.fox_pos,
            hounds_pos: new_hounds,
            coop_pos: self.coop_pos,
            current_turn: Faction::Fox,
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
        Faction::Fox => find_best_fox_move(
            &snapshot,
            &state.graph,
            state.coop_pos,
            depth,
            state.difficulty,
        ),
        Faction::Hounds => find_best_hound_move(
            &snapshot,
            &state.graph,
            state.coop_pos,
            depth,
            state.difficulty,
        ),
    }
}

fn find_best_fox_move(
    board: &BoardSnapshot,
    graph: &Graph,
    coop_pos: usize,
    max_depth: usize,
    difficulty: Difficulty,
) -> Option<PieceMove> {
    let moves: Vec<usize> = board.fox_legal_moves(graph).collect();
    if moves.is_empty() {
        return None;
    }

    // If any move reaches coop directly, take it immediately
    if let Some(&direct_win) = moves.iter().find(|&&m| m == coop_pos) {
        return Some(PieceMove::FoxMove { to: direct_win });
    }

    let mut best_score = -INF;
    let mut candidate_moves = Vec::new();

    for &to in &moves {
        let next_board = board.apply_fox_move(to);
        let score = minimax(
            &next_board,
            graph,
            coop_pos,
            max_depth - 1,
            -INF,
            INF,
            false,
        );

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
                    evaluate_board(&next_b, graph, coop_pos)
                })
                .unwrap_or(&candidate_moves[0])
        }
    };

    Some(PieceMove::FoxMove { to: chosen })
}

fn find_best_hound_move(
    board: &BoardSnapshot,
    graph: &Graph,
    coop_pos: usize,
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
        let score = minimax(&next_board, graph, coop_pos, max_depth - 1, -INF, INF, true);

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
                    evaluate_board(&next_b, graph, coop_pos)
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
    coop_pos: usize,
    depth: usize,
    mut alpha: i32,
    mut beta: i32,
    is_fox_turn: bool,
) -> i32 {
    // 1. Terminal condition checks
    if board.fox_pos == coop_pos {
        return WIN_SCORE + (depth as i32 * 100);
    }

    if is_fox_turn {
        let mut fox_moves = board.fox_legal_moves(graph).peekable();
        if fox_moves.peek().is_none() {
            return -WIN_SCORE - (depth as i32 * 100);
        }

        if depth == 0 {
            return evaluate_board(board, graph, coop_pos);
        }

        let mut max_eval = -INF;
        for to in fox_moves {
            let next_board = board.apply_fox_move(to);
            let eval = minimax(&next_board, graph, coop_pos, depth - 1, alpha, beta, false);
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
            // Hounds have no moves, treat as neutral/evaluate
            return evaluate_board(board, graph, coop_pos);
        }

        if depth == 0 {
            return evaluate_board(board, graph, coop_pos);
        }

        let mut min_eval = INF;
        for (hound_idx, to) in hound_moves {
            let next_board = board.apply_hound_move(hound_idx, to);
            let eval = minimax(&next_board, graph, coop_pos, depth - 1, alpha, beta, true);
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
pub fn evaluate_board(board: &BoardSnapshot, graph: &Graph, coop_pos: usize) -> i32 {
    if board.fox_pos == coop_pos {
        return WIN_SCORE;
    }

    let fox_node = match graph.node(board.fox_pos) {
        Some(n) => n,
        None => return 0,
    };

    let hound_nodes: Vec<_> = board
        .hounds_pos
        .iter()
        .filter_map(|&pos| graph.node(pos))
        .collect();

    let max_row = graph.nodes.iter().map(|n| n.row).max().unwrap_or(9);
    let min_hound_row = hound_nodes.iter().map(|n| n.row).min().unwrap_or(0);
    let max_hound_row = hound_nodes.iter().map(|n| n.row).max().unwrap_or(max_row);

    // 1. Fox breakthrough bonus: if Fox slipped strictly past ALL hounds towards Coop
    let breakthrough_bonus = if fox_node.row < min_hound_row {
        10_000
    } else {
        0
    };

    // 2. Fox distance to coop & row progress
    let dist_to_coop = graph.distance(board.fox_pos, coop_pos).unwrap_or(10);
    let dist_to_coop_score = (10 - dist_to_coop as i32) * 150;
    let row_progress = (max_row as i32 - fox_node.row as i32) * 150;

    // 3. Defensive Blockade: Count how many hounds are positioned between Fox and Coop
    let hounds_ahead = hound_nodes.iter().filter(|h| h.row <= fox_node.row).count() as i32;
    let blockade_score = (3 - hounds_ahead) * 400;

    // 4. Hound Pursuit / Proximity: Distance from each hound to the Fox
    // Hounds want to minimize distance to Fox; Fox wants to maximize it.
    let total_hound_dist: i32 = board
        .hounds_pos
        .iter()
        .map(|&h_pos| graph.distance(h_pos, board.fox_pos).unwrap_or(10) as i32)
        .sum();
    let pursuit_score = total_hound_dist * 80;

    // 5. Hound Line Advancement: Reward hounds for advancing their frontline towards the Fox
    let avg_hound_row =
        hound_nodes.iter().map(|n| n.row as f32).sum::<f32>() / hound_nodes.len().max(1) as f32;
    let hound_advance_score = (avg_hound_row * -140.0) as i32;

    // 6. Fox degrees of freedom / mobility & cornering
    let fox_degrees = board.fox_legal_moves(graph).count();
    let mobility_score = match fox_degrees {
        0 => -WIN_SCORE,
        1 => -2_000, // Fox is on the verge of capture
        2 => -400,   // Fox options are constrained
        3 => 200,    // Fox has moderate mobility
        _ => 600,    // Fox is free to roam
    };

    // 7. Immediate surrounding pressure: Hounds directly adjacent to Fox
    let close_hounds = board
        .hounds_pos
        .iter()
        .filter(|&&h| graph.neighbors(board.fox_pos).contains(&h))
        .count() as i32;
    let pressure_penalty = close_hounds * -400;

    // 8. Hound cohesion: Reward maintaining a united rank, penalize disjointed lines
    let hound_row_span = (max_hound_row - min_hound_row) as i32;
    let cohesion_score = if hound_row_span <= 1 {
        -250
    } else if hound_row_span == 2 {
        0
    } else {
        350
    };

    // 9. Bottleneck control: Controlling the river bridge (M6)
    let bridge_score = graph
        .nodes
        .iter()
        .find(|n| n.node_type == NodeType::Bottleneck)
        .map_or(0, |bridge_node| {
            if board.fox_pos == bridge_node.id {
                600
            } else if board.hounds_pos.contains(&bridge_node.id) {
                if fox_node.row >= bridge_node.row {
                    -800 // Hound locks down bridge while Fox is south
                } else {
                    -200
                }
            } else {
                0
            }
        });

    breakthrough_bonus
        + dist_to_coop_score
        + row_progress
        + blockade_score
        + pursuit_score
        + hound_advance_score
        + mobility_score
        + pressure_penalty
        + cohesion_score
        + bridge_score
}
