use fox_and_hounds::game::ai::{find_best_move, BoardSnapshot};
use fox_and_hounds::game::graph::NodeType;
use fox_and_hounds::game::level::{
    build_classic_graph, build_river_crossing_graph, BoardVariant, CLASSIC_CONFIG,
    FOX_AND_DOGS_ASYMMETRIC_CONFIG, FOX_AND_DOGS_SYMMETRIC_CONFIG, RIVER_CROSSING_CONFIG,
};
use fox_and_hounds::game::state::{
    Difficulty, Faction, GamePhase, GameResult, GameState, MoveError,
};

#[test]
fn test_river_crossing_graph_structure() {
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
fn test_classic_graph_structure() {
    let graph = build_classic_graph();
    assert_eq!(graph.node_count(), 11);

    // M0 should be TargetCoop (Left Apex)
    let m0_idx = graph.find_id_by_name("M0").expect("M0 should exist");
    let m0_node = graph.node(m0_idx).unwrap();
    assert_eq!(m0_node.node_type, NodeType::TargetCoop);
    assert_eq!(m0_node.row, 0);

    // M4 should be FoxStart (Right Apex)
    let m4_idx = graph.find_id_by_name("M4").expect("M4 should exist");
    let m4_node = graph.node(m4_idx).unwrap();
    assert_eq!(m4_node.node_type, NodeType::FoxStart);
    assert_eq!(m4_node.row, 4);

    // M0 neighbors: T1, M1, B1 (degree 3)
    assert_eq!(graph.neighbors(m0_idx).len(), 3);
    // M4 neighbors: T3, M3, B3 (degree 3)
    assert_eq!(graph.neighbors(m4_idx).len(), 3);

    // M2 center hub neighbors: T2, B2, M1, M3, T1, B3, B1, T3 (degree 8)
    let m2_idx = graph.find_id_by_name("M2").expect("M2 should exist");
    assert_eq!(graph.neighbors(m2_idx).len(), 8);

    // Total directed edge entries / 2 = 22 undirected edges
    let total_edges: usize = (0..graph.node_count())
        .map(|id| graph.neighbors(id).len())
        .sum::<usize>()
        / 2;
    assert_eq!(total_edges, 22);

    // Shortest distance from M4 to M0 with no obstacles should be 4
    let dist = graph
        .shortest_distance(m4_idx, m0_idx, &[])
        .expect("Path should exist");
    assert_eq!(dist, 4);
}

#[test]
fn test_initial_state_and_legal_moves() {
    // 1. Classic variant (default)
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    assert_eq!(state.variant, BoardVariant::Classic);
    assert_eq!(state.current_turn, Faction::Fox);
    assert_eq!(state.result, GameResult::Ongoing);
    assert_eq!(state.phase, GamePhase::Playing);

    // Classic uses free entry: on turn 1 the Fox may place itself on any free (non-Hound,
    // non-Coop) vertex. M0 (Coop), T1 and B1 are occupied by Hounds, so 11 - 3 = 8 targets.
    assert!(state.fox_pending);
    let legal_fox_moves = state.fox_legal_moves();
    assert_eq!(legal_fox_moves.len(), 8);

    // Make a legal entry move to M3
    let m3_idx = state.graph.find_id_by_name("M3").unwrap();
    assert!(legal_fox_moves.contains(&m3_idx));
    assert!(state.apply_fox_move(m3_idx).is_ok());

    assert_eq!(state.fox_pos, m3_idx);
    assert!(!state.fox_pending);
    assert_eq!(state.current_turn, Faction::Hounds);

    // 2. River Crossing variant
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Fox, Difficulty::Medium);

    assert_eq!(river_state.variant, BoardVariant::RiverCrossing);
    let legal_river_moves = river_state.fox_legal_moves();
    assert_eq!(legal_river_moves.len(), 3);
    let m8_idx = river_state.graph.find_id_by_name("M8").unwrap();
    assert!(legal_river_moves.contains(&m8_idx));
    assert!(river_state.apply_fox_move(m8_idx).is_ok());
    assert_eq!(river_state.fox_pos, m8_idx);
    assert_eq!(river_state.current_turn, Faction::Hounds);
}

#[test]
fn test_classic_fox_free_entry_first_move() {
    // Classic: on the very first move the Fox may jump to any free vertex (excluding the
    // Coop). After that it moves along edges like normal.
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    assert!(
        state.fox_pending,
        "Classic Fox should await a free-entry choice"
    );

    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    let t1_idx = state.graph.find_id_by_name("T1").unwrap();
    let b1_idx = state.graph.find_id_by_name("B1").unwrap();
    let m2_idx = state.graph.find_id_by_name("M2").unwrap();
    let t2_idx = state.graph.find_id_by_name("T2").unwrap();

    let entry = state.fox_legal_moves();
    // Coop (M0) and the two occupied hound squares (T1, B1) must be excluded.
    assert!(!entry.contains(&m0_idx));
    assert!(!entry.contains(&t1_idx));
    assert!(!entry.contains(&b1_idx));
    // A free, non-adjacent square is a valid entry target.
    assert!(entry.contains(&m2_idx));

    // Should be able to jump directly onto a square the normal graph movement would
    // never allow in one step (here T2 is not adjacent to the holding square M4).
    assert!(entry.contains(&t2_idx));
    assert!(state.apply_fox_move(t2_idx).is_ok());
    assert_eq!(state.fox_pos, t2_idx);
    assert!(!state.fox_pending);
    assert_eq!(state.current_turn, Faction::Hounds);

    // After entry, normal adjacency applies: from T2 the Fox can reach any unoccupied
    // neighbor (T1 here is still a start square of Hound 1).
    state.current_turn = Faction::Fox;
    let mut m2_adj = state.fox_legal_moves();
    let mut free_neighbors: Vec<usize> = state
        .graph
        .neighbors(t2_idx)
        .iter()
        .copied()
        .filter(|&n| !state.hounds_pos.contains(&n))
        .collect();
    m2_adj.sort();
    free_neighbors.sort();
    assert_eq!(m2_adj, free_neighbors);
}

#[test]
fn test_fox_victory_condition() {
    // 1. Classic variant
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    let m1_idx = state.graph.find_id_by_name("M1").unwrap();
    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    state.fox_pos = m1_idx;
    state.fox_pending = false;
    state.hounds_pos = vec![
        state.graph.find_id_by_name("T3").unwrap(),
        state.graph.find_id_by_name("M3").unwrap(),
        state.graph.find_id_by_name("B3").unwrap(),
    ];
    state.current_turn = Faction::Fox;

    let legal = state.fox_legal_moves();
    assert!(legal.contains(&m0_idx));

    assert!(state.apply_fox_move(m0_idx).is_ok());
    assert_eq!(state.result, GameResult::FoxWon);
    assert_eq!(state.phase, GamePhase::GameOver);

    // 2. River Crossing variant
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Fox, Difficulty::Medium);

    let r_m1_idx = river_state.graph.find_id_by_name("M1").unwrap();
    let r_m0_idx = river_state.graph.find_id_by_name("M0").unwrap();
    river_state.fox_pos = r_m1_idx;
    river_state.hounds_pos = vec![
        river_state.graph.find_id_by_name("L4").unwrap(),
        river_state.graph.find_id_by_name("M4").unwrap(),
        river_state.graph.find_id_by_name("R4").unwrap(),
    ];
    river_state.current_turn = Faction::Fox;

    let r_legal = river_state.fox_legal_moves();
    assert!(r_legal.contains(&r_m0_idx));
    assert!(river_state.apply_fox_move(r_m0_idx).is_ok());
    assert_eq!(river_state.result, GameResult::FoxWon);
    assert_eq!(river_state.phase, GamePhase::GameOver);
}

#[test]
fn test_hounds_trap_victory_condition() {
    // 1. Classic variant: Fox at M4 trapped by hounds on T3, M3, B3
    let mut state = GameState::new();
    state.start_game(Faction::Hounds, Difficulty::Medium);

    let m4_idx = state.graph.find_id_by_name("M4").unwrap();
    state.fox_pos = m4_idx;
    state.fox_pending = false;
    state.hounds_pos = vec![
        state.graph.find_id_by_name("T3").unwrap(),
        state.graph.find_id_by_name("M3").unwrap(),
        state.graph.find_id_by_name("B3").unwrap(),
    ];
    state.current_turn = Faction::Fox;

    let legal = state.fox_legal_moves();
    assert!(legal.is_empty());

    state.evaluate_game_result();
    assert_eq!(state.result, GameResult::HoundsWon);
    assert_eq!(state.phase, GamePhase::GameOver);

    // 2. River Crossing variant: Fox at M9 trapped by hounds on L8, M8, R8
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Hounds, Difficulty::Medium);

    let m9_idx = river_state.graph.find_id_by_name("M9").unwrap();
    river_state.fox_pos = m9_idx;
    river_state.hounds_pos = vec![
        river_state.graph.find_id_by_name("L8").unwrap(),
        river_state.graph.find_id_by_name("M8").unwrap(),
        river_state.graph.find_id_by_name("R8").unwrap(),
    ];
    river_state.current_turn = Faction::Fox;

    let r_legal = river_state.fox_legal_moves();
    assert!(r_legal.is_empty());

    river_state.evaluate_game_result();
    assert_eq!(river_state.result, GameResult::HoundsWon);
    assert_eq!(river_state.phase, GamePhase::GameOver);
}

#[test]
fn test_ai_finds_immediate_winning_move() {
    // 1. Classic
    let mut state = GameState::new();
    state.start_game(Faction::Hounds, Difficulty::Hard); // Player is Hounds, AI is Fox

    let m1_idx = state.graph.find_id_by_name("M1").unwrap();
    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    state.fox_pos = m1_idx;
    state.fox_pending = false;
    state.hounds_pos = vec![
        state.graph.find_id_by_name("T3").unwrap(),
        state.graph.find_id_by_name("M3").unwrap(),
        state.graph.find_id_by_name("B3").unwrap(),
    ];
    state.current_turn = Faction::Fox;

    let best_move = find_best_move(&state);
    assert_eq!(
        best_move,
        Some(fox_and_hounds::game::state::PieceMove::FoxMove { to: m0_idx })
    );

    // 2. River Crossing
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Hounds, Difficulty::Hard);

    let r_m1_idx = river_state.graph.find_id_by_name("M1").unwrap();
    let r_m0_idx = river_state.graph.find_id_by_name("M0").unwrap();
    river_state.fox_pos = r_m1_idx;
    river_state.hounds_pos = vec![
        river_state.graph.find_id_by_name("L4").unwrap(),
        river_state.graph.find_id_by_name("M4").unwrap(),
        river_state.graph.find_id_by_name("R4").unwrap(),
    ];
    river_state.current_turn = Faction::Fox;

    let r_best = find_best_move(&river_state);
    assert_eq!(
        r_best,
        Some(fox_and_hounds::game::state::PieceMove::FoxMove { to: r_m0_idx })
    );
}

#[test]
fn test_hounds_cannot_occupy_chicken_coop() {
    // 1. River Crossing: hounds start at L1, M1, R1 and cannot move to M0
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::RiverCrossing);
    state.start_game(Faction::Hounds, Difficulty::Hard);

    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    let l1_idx = state.graph.find_id_by_name("L1").unwrap();
    let m1_idx = state.graph.find_id_by_name("M1").unwrap();
    let r1_idx = state.graph.find_id_by_name("R1").unwrap();

    assert_eq!(state.hounds_pos, vec![l1_idx, m1_idx, r1_idx]);
    assert_eq!(state.coop_pos, m0_idx);

    assert!(state.graph.neighbors(l1_idx).contains(&m0_idx));
    assert!(state.graph.neighbors(m1_idx).contains(&m0_idx));
    assert!(state.graph.neighbors(r1_idx).contains(&m0_idx));

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

    let snapshot = BoardSnapshot::from_state(&state);
    for hound_idx in 0..3 {
        assert!(snapshot
            .hound_legal_moves(&state.graph, hound_idx)
            .all(|target| target != m0_idx));
    }
    assert!(snapshot
        .all_hound_moves(&state.graph)
        .all(|(_, target)| target != m0_idx));

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

    let best_move = find_best_move(&state);
    if let Some(fox_and_hounds::game::state::PieceMove::HoundMove { to, .. }) = best_move {
        assert_ne!(to, m0_idx, "AI should never pick Chicken Coop for Hound");
    }

    // 2. Classic: Hound 0 starts on M0 and moves away to M1. Afterward, no hound can enter M0.
    let mut classic_state = GameState::new();
    classic_state.start_game(Faction::Hounds, Difficulty::Hard);
    let c_m0 = classic_state.graph.find_id_by_name("M0").unwrap();
    let c_m1 = classic_state.graph.find_id_by_name("M1").unwrap();
    classic_state.current_turn = Faction::Hounds;
    assert!(classic_state.apply_hound_move(0, c_m1).is_ok());

    for idx in 0..3 {
        assert!(
            !classic_state.hound_legal_moves(idx).contains(&c_m0),
            "Hound {idx} must not re-enter M0 in Classic"
        );
    }
}

#[test]
fn test_hound_ai_advances_from_start() {
    // 1. Classic variant
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium); // Player is Fox, AI is Hounds

    let m3_idx = state.graph.find_id_by_name("M3").unwrap();
    assert!(state.apply_fox_move(m3_idx).is_ok());
    assert_eq!(state.current_turn, Faction::Hounds);

    let best_move = find_best_move(&state).expect("AI should find a move for Hounds");
    if let fox_and_hounds::game::state::PieceMove::HoundMove { from, to, .. } = best_move {
        let from_node = state.graph.node(from).unwrap();
        let to_node = state.graph.node(to).unwrap();
        assert!(to_node.row >= from_node.row);
    } else {
        panic!("Expected a HoundMove");
    }

    // 2. River Crossing variant
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Fox, Difficulty::Medium);

    let m8_idx = river_state.graph.find_id_by_name("M8").unwrap();
    assert!(river_state.apply_fox_move(m8_idx).is_ok());
    assert_eq!(river_state.current_turn, Faction::Hounds);

    let r_best = find_best_move(&river_state).expect("AI should find a move for Hounds");
    if let fox_and_hounds::game::state::PieceMove::HoundMove {
        hound_idx,
        from,
        to,
    } = r_best
    {
        let from_node = river_state.graph.node(from).unwrap();
        let to_node = river_state.graph.node(to).unwrap();
        assert_eq!(from_node.row, 1);
        assert_eq!(to_node.row, 2, "Hound {hound_idx} should advance to Row 2");
    } else {
        panic!("Expected a HoundMove");
    }
}

#[test]
fn test_hound_ai_pursues_and_tightens_perimeter() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::RiverCrossing);
    state.start_game(Faction::Fox, Difficulty::Hard);

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
    state.switch_variant(BoardVariant::RiverCrossing);
    state.start_game(Faction::Fox, Difficulty::Medium);

    let initial_avg_row: f32 = state
        .hounds_pos
        .iter()
        .map(|&p| state.graph.node(p).unwrap().row as f32)
        .sum::<f32>()
        / 3.0;
    assert_eq!(initial_avg_row, 1.0);

    for _ in 0..4 {
        let fox_moves = state.fox_legal_moves();
        if fox_moves.is_empty() {
            break;
        }
        let chosen_fox_move = *fox_moves
            .iter()
            .max_by_key(|&&m| state.graph.node(m).unwrap().row)
            .unwrap();
        assert!(state.apply_fox_move(chosen_fox_move).is_ok());

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

    assert!(
        end_avg_row > 2.0,
        "Hounds should advance down the board over turns (got avg row {end_avg_row})"
    );
}

#[test]
fn test_move_animation_state_tracking() {
    let mut state = GameState::new();
    // Player is Hounds so the AI (Fox) does not auto-move during animation updates,
    // keeping this animation-only test deterministic.
    state.start_game(Faction::Hounds, Difficulty::Easy);

    // Fox move animation (from its holding square to M3).
    let m3_idx = state.graph.find_id_by_name("M3").unwrap();
    assert!(state.apply_fox_move(m3_idx).is_ok());
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

    // Now test Hound move animation (it's the Hounds' turn, player controls Hounds).
    state.current_turn = Faction::Hounds;
    let hound_legal = state.hound_legal_moves(0);
    assert!(!hound_legal.is_empty());
    let hound_target = hound_legal[0];

    assert!(state.apply_hound_move(0, hound_target).is_ok());
    let hound_anim = state
        .active_anim
        .as_ref()
        .expect("Hound move should create active_anim");
    assert_eq!(hound_anim.faction, Faction::Hounds);
    assert_eq!(hound_anim.hound_idx, Some(0));
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
    // 1. Classic
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    let m2_idx = state.graph.find_id_by_name("M2").unwrap();
    let m3_idx = state.graph.find_id_by_name("M3").unwrap();
    state.fox_pos = m3_idx;
    state.fox_pending = false;
    state.hounds_pos[0] = m2_idx; // M2 is adjacent to M3

    let fox_legal = state.fox_legal_moves();
    assert!(!fox_legal.contains(&m2_idx));
    assert_eq!(state.apply_fox_move(m2_idx), Err(MoveError::IllegalMove));

    state.current_turn = Faction::Hounds;
    let hound_legal = state.hound_legal_moves(0);
    assert!(!hound_legal.contains(&m3_idx));
    assert_eq!(
        state.apply_hound_move(0, m3_idx),
        Err(MoveError::IllegalMove)
    );

    let t2_idx = state.graph.find_id_by_name("T2").unwrap();
    state.hounds_pos[1] = t2_idx;
    let hound_legal = state.hound_legal_moves(0);
    assert!(!hound_legal.contains(&t2_idx));
    assert_eq!(
        state.apply_hound_move(0, t2_idx),
        Err(MoveError::IllegalMove)
    );
}

#[test]
fn test_ai_handles_trapped_fox_position() {
    // 1. Classic
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Hard);

    let m4_idx = state.graph.find_id_by_name("M4").unwrap();
    let t3_idx = state.graph.find_id_by_name("T3").unwrap();
    let m3_idx = state.graph.find_id_by_name("M3").unwrap();
    let b3_idx = state.graph.find_id_by_name("B3").unwrap();

    state.fox_pos = m4_idx;
    state.fox_pending = false;
    state.hounds_pos = vec![t3_idx, m3_idx, b3_idx];
    state.current_turn = Faction::Fox;

    assert_eq!(state.fox_legal_moves().len(), 0);
    assert_eq!(find_best_move(&state), None);

    state.evaluate_game_result();
    assert_eq!(state.result, GameResult::HoundsWon);
    assert_eq!(state.phase, GamePhase::GameOver);
    assert!(state.cached_game_over_stats.is_some());

    // 2. River Crossing
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Fox, Difficulty::Hard);

    let r_m9_idx = river_state.graph.find_id_by_name("M9").unwrap();
    let r_l8_idx = river_state.graph.find_id_by_name("L8").unwrap();
    let r_m8_idx = river_state.graph.find_id_by_name("M8").unwrap();
    let r_r8_idx = river_state.graph.find_id_by_name("R8").unwrap();

    river_state.fox_pos = r_m9_idx;
    river_state.hounds_pos = vec![r_l8_idx, r_m8_idx, r_r8_idx];
    river_state.current_turn = Faction::Fox;

    assert_eq!(river_state.fox_legal_moves().len(), 0);
    assert_eq!(find_best_move(&river_state), None);

    river_state.evaluate_game_result();
    assert_eq!(river_state.result, GameResult::HoundsWon);
    assert_eq!(river_state.phase, GamePhase::GameOver);
}

#[test]
fn test_variant_properties() {
    const {
        assert!(!CLASSIC_CONFIG.allow_hound_retreat);
        assert!(RIVER_CROSSING_CONFIG.allow_hound_retreat);
        assert!(CLASSIC_CONFIG.fox_free_entry);
        assert!(!RIVER_CROSSING_CONFIG.fox_free_entry);
    }

    assert_eq!(CLASSIC_CONFIG.fox_start_node, "M4");
    assert_eq!(CLASSIC_CONFIG.target_coop_node, "M0");
    assert_eq!(CLASSIC_CONFIG.hounds_start_nodes, &["M0", "T1", "B1"]);

    assert_eq!(RIVER_CROSSING_CONFIG.fox_start_node, "M9");
    assert_eq!(RIVER_CROSSING_CONFIG.target_coop_node, "M0");
    assert_eq!(
        RIVER_CROSSING_CONFIG.hounds_start_nodes,
        &["L1", "M1", "R1"]
    );

    assert_eq!(FOX_AND_DOGS_SYMMETRIC_CONFIG.fox_start_node, "C0");
    assert_eq!(FOX_AND_DOGS_SYMMETRIC_CONFIG.target_coop_node, "C8");

    assert_eq!(FOX_AND_DOGS_ASYMMETRIC_CONFIG.fox_start_node, "C0");
    assert_eq!(FOX_AND_DOGS_ASYMMETRIC_CONFIG.target_coop_node, "C8");

    let g_classic = (CLASSIC_CONFIG.build_graph)();
    let g_river = (RIVER_CROSSING_CONFIG.build_graph)();
    assert_eq!(g_classic.node_count(), 11);
    assert_eq!(g_river.node_count(), 24);

    let mut state = GameState::new();
    assert_eq!(state.variant, BoardVariant::Classic);

    state.switch_variant(BoardVariant::RiverCrossing);
    assert_eq!(state.variant, BoardVariant::RiverCrossing);
    assert!(state.variant.config().allow_hound_retreat);

    state.switch_variant(BoardVariant::Classic);
    assert_eq!(state.variant, BoardVariant::Classic);
    assert!(!state.variant.config().allow_hound_retreat);
}

#[test]
fn test_classic_no_retreat_vs_river_crossing_free_movement() {
    // 1. Classic variant: retreat towards column 0 (target.row < hound.row) is forbidden.
    let mut classic_state = GameState::new();
    classic_state.switch_variant(BoardVariant::Classic);
    classic_state.start_game(Faction::Hounds, Difficulty::Medium);

    let t1_idx = classic_state.graph.find_id_by_name("T1").unwrap();
    let b1_idx = classic_state.graph.find_id_by_name("B1").unwrap();
    let m1_idx = classic_state.graph.find_id_by_name("M1").unwrap();
    let t2_idx = classic_state.graph.find_id_by_name("T2").unwrap();
    let m2_idx = classic_state.graph.find_id_by_name("M2").unwrap();
    let b2_idx = classic_state.graph.find_id_by_name("B2").unwrap();
    let m3_idx = classic_state.graph.find_id_by_name("M3").unwrap();
    let m4_idx = classic_state.graph.find_id_by_name("M4").unwrap();

    // Place hound 0 at M2 (Row 2). Hound 1 at T1, Hound 2 at B1. Fox far away at M4.
    classic_state.hounds_pos = vec![m2_idx, t1_idx, b1_idx];
    classic_state.fox_pos = m4_idx;
    classic_state.current_turn = Faction::Hounds;

    let classic_legal = classic_state.hound_legal_moves(0);

    // M1 is Row 1 (< Row 2). In Classic, M1 (retreat) is illegal.
    assert!(
        !classic_legal.contains(&m1_idx),
        "Hound must not retreat to Row 1 in Classic variant"
    );

    // Lateral moves (Row 2 == Row 2) to T2 and B2 are legal
    assert!(
        classic_legal.contains(&t2_idx),
        "Hound should be able to move laterally to T2"
    );
    assert!(
        classic_legal.contains(&b2_idx),
        "Hound should be able to move laterally to B2"
    );

    // Forward move (Row 3 > Row 2) to M3 is legal
    assert!(
        classic_legal.contains(&m3_idx),
        "Hound should be able to advance forward to M3"
    );

    // Attempting to move hound 0 to M1 in Classic must be rejected
    assert_eq!(
        classic_state.apply_hound_move(0, m1_idx),
        Err(MoveError::IllegalMove)
    );

    // 2. River Crossing variant: retreat towards Coop is allowed.
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Hounds, Difficulty::Medium);

    let r_m5_idx = river_state.graph.find_id_by_name("M5").unwrap();
    let r_l4_idx = river_state.graph.find_id_by_name("L4").unwrap();
    let r_r4_idx = river_state.graph.find_id_by_name("R4").unwrap();
    let r_m4_idx = river_state.graph.find_id_by_name("M4").unwrap();
    let r_m0_idx = river_state.graph.find_id_by_name("M0").unwrap();

    river_state.hounds_pos = vec![r_m5_idx, r_l4_idx, r_r4_idx];
    river_state.fox_pos = r_m0_idx;
    river_state.current_turn = Faction::Hounds;

    let river_legal = river_state.hound_legal_moves(0);
    assert!(
        river_legal.contains(&r_m4_idx),
        "Hound should be allowed to retreat to M4 in River Crossing variant"
    );
    assert!(river_state.apply_hound_move(0, r_m4_idx).is_ok());
    assert_eq!(river_state.hounds_pos[0], r_m4_idx);
}

#[test]
fn test_hound_stalemate_fox_victory() {
    // In Classic, when hounds cannot advance or move laterally, and cannot retreat,
    // hounds have 0 legal moves on their turn, yielding GameResult::FoxWon.
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::Classic);
    state.start_game(Faction::Hounds, Difficulty::Hard);

    let t3_idx = state.graph.find_id_by_name("T3").unwrap();
    let m3_idx = state.graph.find_id_by_name("M3").unwrap();
    let b3_idx = state.graph.find_id_by_name("B3").unwrap();
    let m4_idx = state.graph.find_id_by_name("M4").unwrap();

    // Hounds at T3, M3, B3. Fox at M4.
    state.hounds_pos = vec![t3_idx, m3_idx, b3_idx];
    state.fox_pos = m4_idx;
    state.current_turn = Faction::Hounds;

    // Verify all hounds have 0 legal moves
    assert_eq!(state.hound_legal_moves(0).len(), 0);
    assert_eq!(state.hound_legal_moves(1).len(), 0);
    assert_eq!(state.hound_legal_moves(2).len(), 0);
    assert!(state.all_hound_legal_moves().is_empty());

    // AI should gracefully return None
    assert_eq!(find_best_move(&state), None);

    // Evaluating game result triggers immediate Fox victory
    state.evaluate_game_result();
    assert_eq!(state.result, GameResult::FoxWon);
    assert_eq!(state.phase, GamePhase::GameOver);
    assert!(state.cached_game_over_stats.is_some());

    // Conversely, in River Crossing, hounds can retreat to Row 7
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Hounds, Difficulty::Hard);

    let l8_idx = river_state.graph.find_id_by_name("L8").unwrap();
    let m8_idx = river_state.graph.find_id_by_name("M8").unwrap();
    let r8_idx = river_state.graph.find_id_by_name("R8").unwrap();
    let m9_idx = river_state.graph.find_id_by_name("M9").unwrap();

    river_state.hounds_pos = vec![l8_idx, m8_idx, r8_idx];
    river_state.fox_pos = m9_idx;
    river_state.current_turn = Faction::Hounds;

    assert!(!river_state.all_hound_legal_moves().is_empty());
    river_state.evaluate_game_result();
    assert_eq!(river_state.result, GameResult::Ongoing);
}

#[test]
fn test_arthur_graph_structure_and_symmetry() {
    use fox_and_hounds::game::level::{
        build_arthur_asymmetric_graph, build_arthur_symmetric_graph,
    };

    let sym = build_arthur_symmetric_graph();
    let asym = build_arthur_asymmetric_graph();

    assert_eq!(sym.node_count(), 19);
    assert_eq!(asym.node_count(), 19);

    let l4_idx = sym.find_id_by_name("L4").unwrap();
    let l5_idx = sym.find_id_by_name("L5").unwrap();
    let r4_idx = sym.find_id_by_name("R4").unwrap();
    let r5_idx = sym.find_id_by_name("R5").unwrap();

    // Symmetric variant HAS L4-L5 (matching assets/fox_and_dogs_board.png)
    assert!(sym.neighbors(l4_idx).contains(&l5_idx));
    assert!(sym.neighbors(l5_idx).contains(&l4_idx));

    // Asymmetric variant lacks L4-L5 (matching assets/fox_and_dogs_assymetric_board.png)
    assert!(!asym.neighbors(l4_idx).contains(&l5_idx));
    assert!(!asym.neighbors(l5_idx).contains(&l4_idx));

    // Both variants have R4-R5
    assert!(sym.neighbors(r4_idx).contains(&r5_idx));
    assert!(sym.neighbors(r5_idx).contains(&r4_idx));
    assert!(asym.neighbors(r4_idx).contains(&r5_idx));
    assert!(asym.neighbors(r5_idx).contains(&r4_idx));

    // C8 is TargetCoop
    let c8_idx = sym.find_id_by_name("C8").unwrap();
    assert_eq!(sym.node(c8_idx).unwrap().node_type, NodeType::TargetCoop);

    // C2 and C3 are Bottlenecks
    let c2_idx = sym.find_id_by_name("C2").unwrap();
    let c3_idx = sym.find_id_by_name("C3").unwrap();
    assert_eq!(sym.node(c2_idx).unwrap().node_type, NodeType::Bottleneck);
    assert_eq!(sym.node(c3_idx).unwrap().node_type, NodeType::Bottleneck);

    // C0 is FoxStart (dogs can enter)
    let c0_idx = sym.find_id_by_name("C0").unwrap();
    assert_eq!(sym.node(c0_idx).unwrap().node_type, NodeType::FoxStart);
}

#[test]
fn test_arthur_dogs_start_and_rules() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::FoxAndDogsSymmetric);
    state.start_game(Faction::Fox, Difficulty::Medium);

    // Dogs start first!
    assert_eq!(state.current_turn, Faction::Hounds);

    // Initial dog positions: R7, C7, L7
    let r7_idx = state.graph.find_id_by_name("R7").unwrap();
    let c7_idx = state.graph.find_id_by_name("C7").unwrap();
    let l7_idx = state.graph.find_id_by_name("L7").unwrap();
    assert_eq!(state.hounds_pos, vec![r7_idx, c7_idx, l7_idx]);

    // Initial fox position: C0
    let c0_idx = state.graph.find_id_by_name("C0").unwrap();
    let c8_idx = state.graph.find_id_by_name("C8").unwrap();
    assert_eq!(state.fox_pos, c0_idx);

    // Dogs can retreat (move backwards)
    assert!(state.variant.config().allow_hound_retreat);

    // Dogs cannot enter C8
    for hound_idx in 0..3 {
        let moves = state.hound_legal_moves(hound_idx);
        assert!(!moves.contains(&c8_idx));
    }
}

#[test]
fn test_arthur_dogs_can_enter_c0() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::FoxAndDogsAsymmetric);
    state.start_game(Faction::Hounds, Difficulty::Medium);

    let c0_idx = state.graph.find_id_by_name("C0").unwrap();
    let c1_idx = state.graph.find_id_by_name("C1").unwrap();

    let c8_idx = state.graph.find_id_by_name("C8").unwrap();

    // Place a dog on C1 adjacent to C0, fox at C8
    state.hounds_pos[0] = c1_idx;
    state.fox_pos = c8_idx;
    state.current_turn = Faction::Hounds;

    let moves = state.hound_legal_moves(0);
    assert!(moves.contains(&c0_idx), "Dogs must be allowed to enter C0");
}

#[test]
fn test_arthur_direct_mission_flow() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::FoxAndDogsSymmetric);
    state.start_game(Faction::Fox, Difficulty::Medium);

    let c0_idx = state.graph.find_id_by_name("C0").unwrap();
    let c7_idx = state.graph.find_id_by_name("C7").unwrap();
    let c8_idx = state.graph.find_id_by_name("C8").unwrap();

    // Active target initially is C8 (Chicken Coop)
    assert_eq!(state.fox_pos, c0_idx);
    assert_eq!(state.active_target_node(), c8_idx);

    // Dogs start first
    assert_eq!(state.current_turn, Faction::Hounds);
    let h_moves = state.all_hound_legal_moves();
    assert!(!h_moves.is_empty());
    assert!(state.apply_hound_move(h_moves[0].0, h_moves[0].1).is_ok());

    // Turn passes to Fox
    assert_eq!(state.current_turn, Faction::Fox);

    // Fox moves towards C8 and lands on C8
    state.fox_pos = c7_idx;
    assert!(state.apply_fox_move(c8_idx).is_ok());

    // Fox wins immediately!
    assert_eq!(state.result, GameResult::FoxWon);
    assert_eq!(state.phase, GamePhase::GameOver);
}
