use super::graph::Graph;
use super::state::{Difficulty, Faction, GameState, PieceMove};

/// Maximum number of hounds supported in inline stack snapshots.
pub const MAX_HOUNDS: usize = 4;
const UNREACHABLE_DIST: usize = 20;
const WIN_SCORE: i32 = 100_000;
const INF: i32 = 1_000_000;

/// A compact, stack-allocated board state representation for minimax search.
///
/// Uses compact `u8` coordinates and inline fixed-size storage (`[u8; MAX_HOUNDS]`)
/// with `hounds_count: u8` to maintain `Copy` semantics, fit into a tiny ~12-byte
/// memory footprint, and eliminate heap allocations (`Vec` malloc/free churn) inside
/// recursive minimax search loops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardSnapshot {
    pub fox_pos: u8,
    pub fox_pending: bool,
    pub fox_has_left_start: bool,
    pub is_coop_start: bool,
    pub hounds_pos: [u8; MAX_HOUNDS],
    pub hounds_count: u8,
    pub coop_pos: u8,
    pub current_turn: Faction,
    pub allow_hound_retreat: bool,
    pub allow_hounds_in_coop: bool,
}

/// A zero-allocation stack enum iterator over legal Fox moves.
///
/// Dispatches statically between opening entry placement moves ([`FoxMovesIter::Entry`])
/// and standard adjacent graph moves ([`FoxMovesIter::Normal`]), eliminating the need for
/// heap allocation (`Box<dyn Iterator>`) in inner minimax search loops.
pub enum FoxMovesIter<A, B> {
    /// Iterator over initial free-entry placements on the board (e.g. Classic turn 1).
    Entry(A),
    /// Iterator over standard adjacent graph moves from the Fox's current vertex.
    Normal(B),
}

impl<A, B> Iterator for FoxMovesIter<A, B>
where
    A: Iterator<Item = u8>,
    B: Iterator<Item = u8>,
{
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        match self {
            Self::Entry(it) => it.next(),
            Self::Normal(it) => it.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Entry(it) => it.size_hint(),
            Self::Normal(it) => it.size_hint(),
        }
    }
}

impl BoardSnapshot {
    pub fn from_state(state: &GameState) -> Self {
        let count = state.hounds_pos.len().min(MAX_HOUNDS);
        let mut hounds = [0u8; MAX_HOUNDS];
        hounds[..count].copy_from_slice(&state.hounds_pos[..count]);
        Self {
            fox_pos: state.fox_pos,
            fox_pending: state.fox_pending,
            fox_has_left_start: state.fox_has_left_start,
            is_coop_start: state.variant.config().fox_start_node
                == state.variant.config().target_coop_node,
            hounds_pos: hounds,
            hounds_count: count as u8,
            coop_pos: state.coop_pos,
            current_turn: state.current_turn,
            allow_hound_retreat: state.variant.config().allow_hound_retreat,
            allow_hounds_in_coop: state.variant.config().allow_hounds_in_coop,
        }
    }

    #[inline]
    pub fn hounds(&self) -> &[u8] {
        &self.hounds_pos[..self.hounds_count as usize]
    }

    pub const fn active_target(&self) -> u8 {
        self.coop_pos
    }

    pub const fn is_fox_win(&self) -> bool {
        if self.fox_pos == self.coop_pos {
            if self.is_coop_start {
                self.fox_has_left_start
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn fox_legal_moves<'a>(&'a self, graph: &'a Graph) -> impl Iterator<Item = u8> + 'a {
        if self.fox_pending {
            FoxMovesIter::Entry(graph.fox_entry_moves(self.hounds(), self.coop_pos))
        } else {
            FoxMovesIter::Normal(graph.fox_legal_moves(self.fox_pos, self.hounds()))
        }
    }

    pub fn hound_legal_moves<'a>(
        &'a self,
        graph: &'a Graph,
        hound_idx: u8,
    ) -> impl Iterator<Item = u8> + 'a {
        let pos = self.hounds_pos[hound_idx as usize];
        graph.hound_legal_moves(
            pos,
            self.fox_pos,
            self.coop_pos,
            self.hounds(),
            self.allow_hound_retreat,
            self.allow_hounds_in_coop,
        )
    }

    pub fn all_hound_moves<'a>(&'a self, graph: &'a Graph) -> impl Iterator<Item = (u8, u8)> + 'a {
        (0..self.hounds_count).flat_map(move |idx| {
            self.hound_legal_moves(graph, idx)
                .map(move |target| (idx, target))
        })
    }

    pub fn apply_fox_move(&self, to: u8) -> Self {
        let fox_has_left_start = self.fox_has_left_start || (to != self.coop_pos);
        Self {
            fox_pos: to,
            fox_pending: false,
            fox_has_left_start,
            is_coop_start: self.is_coop_start,
            hounds_pos: self.hounds_pos,
            hounds_count: self.hounds_count,
            coop_pos: self.coop_pos,
            current_turn: Faction::Hounds,
            allow_hound_retreat: self.allow_hound_retreat,
            allow_hounds_in_coop: self.allow_hounds_in_coop,
        }
    }

    pub fn apply_hound_move(&self, hound_idx: u8, to: u8) -> Self {
        let mut new_hounds = self.hounds_pos;
        new_hounds[hound_idx as usize] = to;
        Self {
            fox_pos: self.fox_pos,
            fox_pending: self.fox_pending,
            fox_has_left_start: self.fox_has_left_start,
            is_coop_start: self.is_coop_start,
            hounds_pos: new_hounds,
            hounds_count: self.hounds_count,
            coop_pos: self.coop_pos,
            current_turn: Faction::Fox,
            allow_hound_retreat: self.allow_hound_retreat,
            allow_hounds_in_coop: self.allow_hounds_in_coop,
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

fn select_best_candidate<T, F>(candidates: &[T], difficulty: Difficulty, mut eval_candidate: F) -> T
where
    T: Copy,
    F: FnMut(T) -> (i32, i32),
{
    match difficulty {
        Difficulty::Easy if candidates.len() > 1 => {
            let idx = (macroquad::rand::gen_range(0, candidates.len())) as usize;
            candidates[idx]
        }
        _ => *candidates
            .iter()
            .max_by_key(|&&item| eval_candidate(item))
            .unwrap_or(&candidates[0]),
    }
}

fn find_best_fox_move(
    board: &BoardSnapshot,
    graph: &Graph,
    max_depth: usize,
    difficulty: Difficulty,
) -> Option<PieceMove> {
    let moves: Vec<u8> = board.fox_legal_moves(graph).collect();
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

    let chosen = select_best_candidate(&candidate_moves, difficulty, |to| {
        let next_b = board.apply_fox_move(to);
        let eval = evaluate_board(&next_b, graph);
        let dist = graph
            .distance(to, next_b.active_target())
            .unwrap_or(UNREACHABLE_DIST);
        (eval, -(dist as i32))
    });

    Some(PieceMove::FoxMove { to: chosen })
}

fn find_best_hound_move(
    board: &BoardSnapshot,
    graph: &Graph,
    max_depth: usize,
    difficulty: Difficulty,
) -> Option<PieceMove> {
    let moves: Vec<(u8, u8)> = board.all_hound_moves(graph).collect();
    if moves.is_empty() {
        return None;
    }

    // If any hound move immediately traps the Fox (checkmate), take it immediately
    if let Some(&(hound_idx, to)) = moves.iter().find(|&&(h_idx, to)| {
        let next_b = board.apply_hound_move(h_idx, to);
        let is_trapped = next_b.fox_legal_moves(graph).next().is_none();
        is_trapped
    }) {
        return Some(PieceMove::HoundMove {
            hound_idx,
            from: board.hounds_pos[hound_idx as usize],
            to,
        });
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

    let chosen = select_best_candidate(&candidate_moves, difficulty, |(hound_idx, to)| {
        let next_b = board.apply_hound_move(hound_idx, to);
        let eval = evaluate_board(&next_b, graph);
        let dist_to_fox = graph
            .distance(to, board.fox_pos)
            .unwrap_or(UNREACHABLE_DIST);
        (-eval, -(dist_to_fox as i32))
    });

    Some(PieceMove::HoundMove {
        hound_idx: chosen.0,
        from: board.hounds_pos[chosen.0 as usize],
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

    if depth == 0 {
        return evaluate_board(board, graph);
    }

    if is_fox_turn {
        let mut fox_moves = [0u8; 32];
        let mut count = 0;
        for to in board.fox_legal_moves(graph) {
            fox_moves[count] = to;
            count += 1;
        }
        if count == 0 {
            return -WIN_SCORE - (depth as i32 * 100);
        }

        // Move ordering: evaluate immediate winning moves and moves closer to coop first
        let coop = board.coop_pos;
        let moves = &mut fox_moves[..count];
        moves.sort_unstable_by_key(|&to| {
            if to == coop {
                0
            } else {
                graph.distance(to, coop).unwrap_or(UNREACHABLE_DIST) + 1
            }
        });

        let mut max_eval = -INF;
        for &to in moves.iter() {
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
        let mut hound_moves = [(0u8, 0u8); 32];
        let mut count = 0;
        for m in board.all_hound_moves(graph) {
            hound_moves[count] = m;
            count += 1;
        }
        if count == 0 {
            // Hounds have no moves on their turn: Fox wins immediately
            return WIN_SCORE + (depth as i32 * 100);
        }

        // Move ordering for Hounds: evaluate moves that close distance to Fox first
        let fox_pos = board.fox_pos;
        let moves = &mut hound_moves[..count];
        moves.sort_unstable_by_key(|&(_, to)| {
            graph.distance(to, fox_pos).unwrap_or(UNREACHABLE_DIST)
        });

        let mut min_eval = INF;
        for &(hound_idx, to) in moves.iter() {
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
    let static_dist = graph.distance(board.fox_pos, target).unwrap_or(15);
    let dist_to_target_score = (18 - static_dist as i32) * 350;

    // Direct unblocked lane bonus: if the hounds leave a clear route to the coop
    let unblocked_lane_score = match graph.shortest_distance(board.fox_pos, target, board.hounds())
    {
        Some(1) => 15_000,
        Some(2) => 6_000,
        Some(3) => 2_500,
        Some(d) if d <= 5 => (8 - d as i32) * 300,
        _ => 0,
    };

    // Breakthrough bonus: Fox is closer to target than any hound
    let min_hound_dist_to_target = board
        .hounds()
        .iter()
        .filter_map(|&h_pos| graph.distance(h_pos, target))
        .min()
        .unwrap_or(0);
    let breakthrough_bonus = if static_dist < min_hound_dist_to_target {
        (min_hound_dist_to_target as i32 - static_dist as i32) * 450
    } else {
        0
    };

    // Pursuit score and threat-based hound penalty computed in a single pass:
    // Pursuit score provides an incentive for hounds to close distance across the board,
    // while kept low (30) so that 1 step toward target (+350) strictly dominates approaching 3 hounds (-90).
    let (pursuit_score, hound_threat_penalty) =
        board
            .hounds()
            .iter()
            .fold((0, 0), |(pursuit, threat), &h_pos| {
                let dist = graph.distance(h_pos, board.fox_pos).unwrap_or(10);
                let threat_delta = match dist {
                    0..=1 => -450,
                    2 => -150,
                    _ => 0,
                };
                (pursuit + dist as i32 * 30, threat + threat_delta)
            });

    let fox_degrees = board.fox_legal_moves(graph).count();
    let mobility_score = match fox_degrees {
        0 => -WIN_SCORE,
        1 => -2_000,
        2 => -350,
        3 => 150,
        _ => 400,
    };

    dist_to_target_score
        + unblocked_lane_score
        + breakthrough_bonus
        + pursuit_score
        + hound_threat_penalty
        + mobility_score
}
