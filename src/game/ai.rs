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

    pub fn fox_legal_moves(&self, graph: &Graph) -> Vec<usize> {
        graph
            .neighbors(self.fox_pos)
            .iter()
            .copied()
            .filter(|&target| !self.hounds_pos.contains(&target))
            .collect()
    }

    pub fn hound_legal_moves(&self, graph: &Graph, hound_idx: usize) -> Vec<usize> {
        let pos = self.hounds_pos[hound_idx];
        graph
            .neighbors(pos)
            .iter()
            .copied()
            .filter(|&target| {
                target != self.fox_pos
                    && target != self.coop_pos
                    && !self.hounds_pos.contains(&target)
                    && graph
                        .node(target)
                        .is_none_or(|n| n.node_type != NodeType::TargetCoop)
            })
            .collect()
    }

    pub fn all_hound_moves(&self, graph: &Graph) -> Vec<(usize, usize)> {
        (0..3)
            .flat_map(|idx| {
                self.hound_legal_moves(graph, idx)
                    .into_iter()
                    .map(move |target| (idx, target))
            })
            .collect()
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
    let moves = board.fox_legal_moves(graph);
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

    // In Easy difficulty, occasionally choose second best if available
    let chosen = match difficulty {
        Difficulty::Easy if candidate_moves.len() > 1 => {
            let idx = (macroquad::rand::gen_range(0, candidate_moves.len())) as usize;
            candidate_moves[idx]
        }
        _ => candidate_moves[0],
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
    let moves = board.all_hound_moves(graph);
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
        _ => candidate_moves[0],
    };

    Some(PieceMove::HoundMove {
        hound_idx: chosen.0,
        from: board.hounds_pos[chosen.0],
        to: chosen.1,
    })
}

fn minimax(
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
        let fox_moves = board.fox_legal_moves(graph);
        if fox_moves.is_empty() {
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
        let hound_moves = board.all_hound_moves(graph);
        if hound_moves.is_empty() {
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

    // 1. Shortest path distance to coop (ignoring or factoring hounds)
    let shortest_dist = graph
        .shortest_distance(board.fox_pos, coop_pos, &board.hounds_pos)
        .unwrap_or(20);
    let dist_to_coop_score = (15 - shortest_dist as i32) * 250;

    // 2. Row progression score (Row 0 is target, Row 10 is start)
    let row_progress = (10 - fox_node.row as i32) * 120;

    // 3. Fox degrees of freedom / mobility
    let fox_degrees = board.fox_legal_moves(graph).len() as i32;
    let mobility_score = match fox_degrees {
        0 => -WIN_SCORE,
        1 => -800,
        2 => 100,
        3 => 300,
        _ => 500,
    };

    let max_row = graph.nodes.iter().map(|n| n.row).max().unwrap_or(9);
    let min_hound_row = hound_nodes.iter().map(|n| n.row).min().unwrap_or(0);
    let max_hound_row = hound_nodes.iter().map(|n| n.row).max().unwrap_or(max_row);

    let breakthrough_bonus = if fox_node.row < min_hound_row {
        8_000 // Fox is past all hounds!
    } else if fox_node.row <= max_hound_row {
        // Fox is in the fray with hounds
        1_000
    } else {
        0
    };

    // 5. Hound cohesion & blockade penalty
    // Check if hounds are positioned on same row or adjacent rows blocking lanes
    let hound_row_span = (max_hound_row - min_hound_row) as i32;
    let hound_cohesion_bonus = if hound_row_span <= 1 { -300 } else { 150 };

    // 6. Proximity penalty: If Fox is within 1 step of multiple hounds, higher danger
    let close_hounds = board
        .hounds_pos
        .iter()
        .filter(|&&h| graph.neighbors(board.fox_pos).contains(&h))
        .count() as i32;
    let danger_penalty = close_hounds * -250;

    // 7. Chokepoint (Bridge Bottleneck) control bonus
    let bridge_bonus = if let Some(bridge_node) = graph.nodes.iter().find(|n| n.node_type == NodeType::Bottleneck) {
        if board.fox_pos == bridge_node.id {
            // Fox is on bridge
            400
        } else if board.hounds_pos.contains(&bridge_node.id) {
            // Hounds block bridge
            -500
        } else {
            0
        }
    } else {
        0
    };

    dist_to_coop_score
        + row_progress
        + mobility_score
        + breakthrough_bonus
        + hound_cohesion_bonus
        + danger_penalty
        + bridge_bonus
}
