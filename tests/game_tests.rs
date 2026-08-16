use fox_and_hounds::game::ai::{evaluate_board, find_best_move, BoardSnapshot};
use fox_and_hounds::game::graph::NodeType;
use fox_and_hounds::game::level::build_river_crossing_graph;
use fox_and_hounds::game::state::{Difficulty, Faction, GamePhase, GameResult, GameState};

#[test]
fn test_graph_structure() {
    let graph = build_river_crossing_graph();
    assert_eq!(graph.node_count(), 27);

    // M0 should be TargetCoop
    let m0_idx = graph.find_id_by_name("M0").expect("M0 should exist");
    let m0_node = graph.node(m0_idx).unwrap();
    assert_eq!(m0_node.node_type, NodeType::TargetCoop);
    assert_eq!(m0_node.row, 0);

    // M7 should be Bottleneck
    let m7_idx = graph.find_id_by_name("M7").expect("M7 should exist");
    let m7_node = graph.node(m7_idx).unwrap();
    assert_eq!(m7_node.node_type, NodeType::Bottleneck);
    assert_eq!(m7_node.row, 7);

    // M10 should be FoxStart
    let m10_idx = graph.find_id_by_name("M10").expect("M10 should exist");
    let m10_node = graph.node(m10_idx).unwrap();
    assert_eq!(m10_node.node_type, NodeType::FoxStart);
    assert_eq!(m10_node.row, 10);

    // Shortest distance from M10 to M0 with no obstacles should be 10
    let dist = graph
        .shortest_distance(m10_idx, m0_idx, &[])
        .expect("Path should exist");
    assert_eq!(dist, 10);
}

#[test]
fn test_initial_state_and_legal_moves() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    assert_eq!(state.current_turn, Faction::Fox);
    assert_eq!(state.result, GameResult::Ongoing);
    assert_eq!(state.phase, GamePhase::Playing);

    // Fox starts at M10 (Row 10)
    // Neighbors of M10 are L9, M9, R9
    let legal_fox_moves = state.fox_legal_moves();
    assert_eq!(legal_fox_moves.len(), 3);

    // Make a legal move to M9
    let m9_idx = state.graph.find_id_by_name("M9").unwrap();
    assert!(legal_fox_moves.contains(&m9_idx));
    assert!(state.apply_fox_move(m9_idx).is_ok());

    assert_eq!(state.fox_pos, m9_idx);
    assert_eq!(state.current_turn, Faction::Hounds);
}

#[test]
fn test_fox_victory_condition() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    // Place Fox adjacent to M0 (e.g. at M1) and hounds elsewhere
    let m1_idx = state.graph.find_id_by_name("M1").unwrap();
    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    state.fox_pos = m1_idx;
    state.hounds_pos = vec![
        state.graph.find_id_by_name("L4").unwrap(),
        state.graph.find_id_by_name("M4").unwrap(),
        state.graph.find_id_by_name("R4").unwrap(),
    ];
    state.current_turn = Faction::Fox;

    let legal = state.fox_legal_moves();
    assert!(legal.contains(&m0_idx));

    assert!(state.apply_fox_move(m0_idx).is_ok());
    assert_eq!(state.result, GameResult::FoxWon);
    assert_eq!(state.phase, GamePhase::GameOver);
}

#[test]
fn test_hounds_trap_victory_condition() {
    let mut state = GameState::new();
    state.start_game(Faction::Hounds, Difficulty::Medium);

    // Trap fox in corner (e.g. M10 surrounded by L9, M9, R9)
    let m10_idx = state.graph.find_id_by_name("M10").unwrap();
    state.fox_pos = m10_idx;
    state.hounds_pos = vec![
        state.graph.find_id_by_name("L9").unwrap(),
        state.graph.find_id_by_name("M9").unwrap(),
        state.graph.find_id_by_name("R9").unwrap(),
    ];
    state.current_turn = Faction::Fox;

    // Fox has zero legal moves
    let legal = state.fox_legal_moves();
    assert!(legal.is_empty());

    state.evaluate_game_result();
    assert_eq!(state.result, GameResult::HoundsWon);
    assert_eq!(state.phase, GamePhase::GameOver);
}

#[test]
fn test_ai_finds_immediate_winning_move() {
    let mut state = GameState::new();
    state.start_game(Faction::Hounds, Difficulty::Hard); // Player is Hounds, AI is Fox

    let m1_idx = state.graph.find_id_by_name("M1").unwrap();
    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    state.fox_pos = m1_idx;
    state.hounds_pos = vec![
        state.graph.find_id_by_name("L4").unwrap(),
        state.graph.find_id_by_name("M4").unwrap(),
        state.graph.find_id_by_name("R4").unwrap(),
    ];
    state.current_turn = Faction::Fox;

    let best_move = find_best_move(&state);
    assert_eq!(
        best_move,
        Some(fox_and_hounds::game::state::PieceMove::FoxMove { to: m0_idx })
    );
}

#[test]
fn test_board_snapshot_evaluation() {
    let state = GameState::new();
    let snapshot = BoardSnapshot::from_state(&state);
    let eval = evaluate_board(&snapshot, &state.graph, state.coop_pos);
    // Initial evaluation should be finite
    assert!(eval > -100_000 && eval < 100_000);
}

#[test]
fn test_hounds_cannot_occupy_chicken_coop() {
    let mut state = GameState::new();
    state.start_game(Faction::Hounds, Difficulty::Hard);

    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    let l1_idx = state.graph.find_id_by_name("L1").unwrap();
    let m1_idx = state.graph.find_id_by_name("M1").unwrap();
    let r1_idx = state.graph.find_id_by_name("R1").unwrap();

    // Hounds start at L1, M1, R1 - all adjacent to M0 (Chicken Coop)
    assert_eq!(state.hounds_pos, vec![l1_idx, m1_idx, r1_idx]);
    assert_eq!(state.coop_pos, m0_idx);

    // Verify neighbors of L1, M1, R1 in the graph include M0
    assert!(state.graph.neighbors(l1_idx).contains(&m0_idx));
    assert!(state.graph.neighbors(m1_idx).contains(&m0_idx));
    assert!(state.graph.neighbors(r1_idx).contains(&m0_idx));

    // But hound legal moves must NEVER include M0 (Chicken Coop)
    for hound_idx in 0..state.hounds_pos.len() {
        let legal = state.hound_legal_moves(hound_idx);
        assert!(
            !legal.contains(&m0_idx),
            "Hound {hound_idx} should not be allowed to move to Chicken Coop (M0)"
        );
    }

    let all_moves = state.all_hound_legal_moves();
    assert!(
        all_moves.iter().all(|&(_, target)| target != m0_idx),
        "No hound move should target Chicken Coop (M0)"
    );

    // AI snapshot must also exclude M0
    let snapshot = BoardSnapshot::from_state(&state);
    for hound_idx in 0..3 {
        let legal = snapshot.hound_legal_moves(&state.graph, hound_idx);
        assert!(!legal.contains(&m0_idx));
    }
    let all_ai_moves = snapshot.all_hound_moves(&state.graph);
    assert!(all_ai_moves.iter().all(|&(_, target)| target != m0_idx));

    // Manually setting turn to Hounds and attempting to move to M0 must be rejected
    state.current_turn = Faction::Hounds;
    assert_eq!(
        state.apply_hound_move(0, m0_idx),
        Err("Illegal move for Hound")
    );
    assert_eq!(
        state.apply_hound_move(1, m0_idx),
        Err("Illegal move for Hound")
    );
    assert_eq!(
        state.apply_hound_move(2, m0_idx),
        Err("Illegal move for Hound")
    );

    // Best move for Hounds should never be M0
    let best_move = find_best_move(&state);
    if let Some(fox_and_hounds::game::state::PieceMove::HoundMove { to, .. }) = best_move {
        assert_ne!(to, m0_idx, "AI should never pick Chicken Coop for Hound");
    }
}
