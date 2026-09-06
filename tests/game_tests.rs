use fox_and_hounds::game::ai::{find_best_move, BoardSnapshot};
use fox_and_hounds::game::graph::NodeType;
use fox_and_hounds::game::level::build_river_crossing_graph;
use fox_and_hounds::game::state::{
    Difficulty, Faction, GamePhase, GameResult, GameState, MoveError,
};

#[test]
fn test_graph_structure() {
    let graph = build_river_crossing_graph();
    assert_eq!(graph.node_count(), 24);

    // M0 should be TargetCoop
    let m0_idx = graph.find_id_by_name("M0").expect("M0 should exist");
    let m0_node = graph.node(m0_idx).unwrap();
    assert_eq!(m0_node.node_type, NodeType::TargetCoop);
    assert_eq!(m0_node.row, 0);

    // M6 should be Bottleneck
    let m6_idx = graph.find_id_by_name("M6").expect("M6 should exist");
    let m6_node = graph.node(m6_idx).unwrap();
    assert_eq!(m6_node.node_type, NodeType::Bottleneck);
    assert_eq!(m6_node.row, 6);

    // M9 should be FoxStart
    let m9_idx = graph.find_id_by_name("M9").expect("M9 should exist");
    let m9_node = graph.node(m9_idx).unwrap();
    assert_eq!(m9_node.node_type, NodeType::FoxStart);
    assert_eq!(m9_node.row, 9);

    // Shortest distance from M9 to M0 with no obstacles should be 9
    let dist = graph
        .shortest_distance(m9_idx, m0_idx, &[])
        .expect("Path should exist");
    assert_eq!(dist, 9);
}

#[test]
fn test_initial_state_and_legal_moves() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    assert_eq!(state.current_turn, Faction::Fox);
    assert_eq!(state.result, GameResult::Ongoing);
    assert_eq!(state.phase, GamePhase::Playing);

    // Fox starts at M9 (Row 9)
    // Neighbors of M9 are L8, M8, R8
    let legal_fox_moves = state.fox_legal_moves();
    assert_eq!(legal_fox_moves.len(), 3);

    // Make a legal move to M8
    let m8_idx = state.graph.find_id_by_name("M8").unwrap();
    assert!(legal_fox_moves.contains(&m8_idx));
    assert!(state.apply_fox_move(m8_idx).is_ok());

    assert_eq!(state.fox_pos, m8_idx);
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

    // Trap fox in corner (e.g. M9 surrounded by L8, M8, R8)
    let m9_idx = state.graph.find_id_by_name("M9").unwrap();
    state.fox_pos = m9_idx;
    state.hounds_pos = vec![
        state.graph.find_id_by_name("L8").unwrap(),
        state.graph.find_id_by_name("M8").unwrap(),
        state.graph.find_id_by_name("R8").unwrap(),
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
        assert!(snapshot
            .hound_legal_moves(&state.graph, hound_idx)
            .all(|target| target != m0_idx));
    }
    assert!(snapshot
        .all_hound_moves(&state.graph)
        .all(|(_, target)| target != m0_idx));

    // Manually setting turn to Hounds and attempting to move to M0 must be rejected
    state.current_turn = Faction::Hounds;
    assert_eq!(
        state.apply_hound_move(0, m0_idx),
        Err(MoveError::IllegalMove)
    );
    assert_eq!(
        state.apply_hound_move(1, m0_idx),
        Err(MoveError::IllegalMove)
    );
    assert_eq!(
        state.apply_hound_move(2, m0_idx),
        Err(MoveError::IllegalMove)
    );

    // Best move for Hounds should never be M0
    let best_move = find_best_move(&state);
    if let Some(fox_and_hounds::game::state::PieceMove::HoundMove { to, .. }) = best_move {
        assert_ne!(to, m0_idx, "AI should never pick Chicken Coop for Hound");
    }
}

#[test]
fn test_hound_ai_advances_from_start() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium); // Player is Fox, AI is Hounds

    // Fox moves M9 -> M8
    let m8_idx = state.graph.find_id_by_name("M8").unwrap();
    assert!(state.apply_fox_move(m8_idx).is_ok());
    assert_eq!(state.current_turn, Faction::Hounds);

    // AI Hound chooses move
    let best_move = find_best_move(&state).expect("AI should find a move for Hounds");
    if let fox_and_hounds::game::state::PieceMove::HoundMove {
        hound_idx,
        from,
        to,
    } = best_move
    {
        let from_node = state.graph.node(from).unwrap();
        let to_node = state.graph.node(to).unwrap();
        // The hound must advance from Row 1 to Row 2
        assert_eq!(from_node.row, 1);
        assert_eq!(to_node.row, 2, "Hound {hound_idx} should advance to Row 2");
    } else {
        panic!("Expected a HoundMove");
    }
}

#[test]
fn test_hound_ai_pursues_and_tightens_perimeter() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Hard);

    // Place Fox at M8, and Hounds across row 5
    let m8_idx = state.graph.find_id_by_name("M8").unwrap();
    let l5_idx = state.graph.find_id_by_name("L5").unwrap();
    let m5_idx = state.graph.find_id_by_name("M5").unwrap();
    let r5_idx = state.graph.find_id_by_name("R5").unwrap();
    let m6_idx = state.graph.find_id_by_name("M6").unwrap();

    state.fox_pos = m8_idx;
    state.hounds_pos = vec![l5_idx, m5_idx, r5_idx];
    state.current_turn = Faction::Hounds;

    let best_move = find_best_move(&state).expect("AI should find a move");
    if let fox_and_hounds::game::state::PieceMove::HoundMove { to, from, .. } = best_move {
        let to_node = state.graph.node(to).unwrap();
        let from_node = state.graph.node(from).unwrap();
        assert_eq!(from_node.row, 5);
        assert_eq!(to, m6_idx, "AI Hound should take the bottleneck bridge M6");
        assert_eq!(to_node.row, 6);
    } else {
        panic!("Expected a HoundMove");
    }
}

#[test]
fn test_multi_turn_hounds_advance_and_surround() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    // Start with Fox at M9. Over multiple turns, Fox moves between M9 and L8/R8
    // while AI Hounds take turns.
    let initial_avg_row: f32 = state
        .hounds_pos
        .iter()
        .map(|&p| state.graph.node(p).unwrap().row as f32)
        .sum::<f32>()
        / 3.0;
    assert_eq!(initial_avg_row, 1.0);

    for _ in 0..4 {
        // Fox turn: pick legal move that stays in rows 8-9 if possible
        let fox_moves = state.fox_legal_moves();
        if fox_moves.is_empty() {
            break;
        }
        let chosen_fox_move = *fox_moves
            .iter()
            .max_by_key(|&&m| state.graph.node(m).unwrap().row)
            .unwrap();
        assert!(state.apply_fox_move(chosen_fox_move).is_ok());

        // Hound AI turn
        if let Some(fox_and_hounds::game::state::PieceMove::HoundMove { hound_idx, to, .. }) =
            find_best_move(&state)
        {
            assert!(state.apply_hound_move(hound_idx, to).is_ok());
        }
    }

    let end_avg_row: f32 = state
        .hounds_pos
        .iter()
        .map(|&p| state.graph.node(p).unwrap().row as f32)
        .sum::<f32>()
        / 3.0;

    // Hounds should have progressed significantly from Row 1 down toward Fox
    assert!(
        end_avg_row > 2.0,
        "Hounds should advance down the board over turns (got avg row {end_avg_row})"
    );
}

#[test]
fn test_move_animation_state_tracking() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Easy);

    let fox_legal = state.fox_legal_moves();
    assert!(!fox_legal.is_empty());
    let fox_target = fox_legal[0];

    assert!(state.apply_fox_move(fox_target).is_ok());
    let fox_anim = state
        .active_anim
        .as_ref()
        .expect("Fox move should create active_anim");
    assert_eq!(fox_anim.faction, Faction::Fox);
    assert_eq!(fox_anim.hound_idx, None);
    assert_eq!(fox_anim.progress, 0.0);
    assert!(
        fox_anim.duration >= 0.25,
        "Duration should be at least 0.25s for smooth animation"
    );

    // Advance animation partially
    state.update(0.10);
    let updated_anim = state
        .active_anim
        .as_ref()
        .expect("Animation should still be active");
    assert!(updated_anim.progress > 0.0 && updated_anim.progress < 1.0);

    // Complete animation
    state.update(0.30);
    assert!(
        state.active_anim.is_none(),
        "Animation should complete after full duration"
    );

    // Now test Hound move animation
    state.current_turn = Faction::Hounds;
    let hound_legal = state.hound_legal_moves(1);
    assert!(!hound_legal.is_empty());
    let hound_target = hound_legal[0];

    assert!(state.apply_hound_move(1, hound_target).is_ok());
    let hound_anim = state
        .active_anim
        .as_ref()
        .expect("Hound move should create active_anim");
    assert_eq!(hound_anim.faction, Faction::Hounds);
    assert_eq!(hound_anim.hound_idx, Some(1));
    assert_eq!(hound_anim.progress, 0.0);
    assert!(hound_anim.duration >= 0.25);
}

#[test]
fn test_move_errors_and_turn_validation() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    // Fox turn: moving a hound should yield NotYourTurn
    assert_eq!(state.apply_hound_move(0, 1), Err(MoveError::NotYourTurn));

    // Invalid hound index
    state.current_turn = Faction::Hounds;
    assert_eq!(state.apply_hound_move(99, 1), Err(MoveError::InvalidHound));

    // Hounds turn: moving the fox should yield NotYourTurn
    assert_eq!(state.apply_fox_move(0), Err(MoveError::NotYourTurn));

    // Verify Display on MoveError
    assert_eq!(format!("{}", MoveError::NotYourTurn), "Not your turn");
    assert_eq!(format!("{}", MoveError::IllegalMove), "Illegal move");
    assert_eq!(
        format!("{}", MoveError::InvalidHound),
        "Invalid hound index"
    );
}

#[test]
fn test_piece_collision_rejection() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    // Position Fox directly adjacent to Hound 0
    let m6_idx = state.graph.find_id_by_name("M6").unwrap();
    let m7_idx = state.graph.find_id_by_name("M7").unwrap();
    state.fox_pos = m7_idx;
    state.hounds_pos[0] = m6_idx; // M6 is adjacent to M7

    // Fox cannot move onto the hound's node
    let fox_legal = state.fox_legal_moves();
    assert!(!fox_legal.contains(&m6_idx));
    assert_eq!(state.apply_fox_move(m6_idx), Err(MoveError::IllegalMove));

    // Hound cannot move onto the fox's node
    state.current_turn = Faction::Hounds;
    let hound_legal = state.hound_legal_moves(0);
    assert!(!hound_legal.contains(&m7_idx));
    assert_eq!(
        state.apply_hound_move(0, m7_idx),
        Err(MoveError::IllegalMove)
    );

    // Hound cannot move onto another hound's node
    let r7_idx = state.graph.find_id_by_name("R7").unwrap();
    state.hounds_pos[1] = r7_idx;
    let hound_legal = state.hound_legal_moves(0);
    assert!(!hound_legal.contains(&r7_idx));
    assert_eq!(
        state.apply_hound_move(0, r7_idx),
        Err(MoveError::IllegalMove)
    );
}

#[test]
fn test_ai_handles_trapped_fox_position() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Hard);

    // Trap fox at M9 by placing hounds on L8, M8, R8 (all neighbors of M9)
    let m9_idx = state.graph.find_id_by_name("M9").unwrap();
    let l8_idx = state.graph.find_id_by_name("L8").unwrap();
    let m8_idx = state.graph.find_id_by_name("M8").unwrap();
    let r8_idx = state.graph.find_id_by_name("R8").unwrap();

    state.fox_pos = m9_idx;
    state.hounds_pos = vec![l8_idx, m8_idx, r8_idx];
    state.current_turn = Faction::Fox;

    // Fox has 0 legal moves
    assert_eq!(state.fox_legal_moves().len(), 0);

    // AI should evaluate this without crashing
    let best_move = find_best_move(&state);
    assert_eq!(best_move, None);

    // Evaluating game result triggers game over
    state.evaluate_game_result();
    assert_eq!(state.result, GameResult::HoundsWon);
    assert_eq!(state.phase, GamePhase::GameOver);
    assert!(state.cached_game_over_stats.is_some());
}
