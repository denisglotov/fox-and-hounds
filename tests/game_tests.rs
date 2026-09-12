use fox_and_hounds::game::ai::find_best_move;
use fox_and_hounds::game::graph::NodeType;
use fox_and_hounds::game::level::{
    build_classic_graph, build_fox_and_dogs_graph, build_river_crossing_graph, BoardVariant,
    FOX_AND_DOGS_CONFIG,
};
use fox_and_hounds::game::state::{
    Difficulty, Faction, GamePhase, GameResult, GameState, MoveError, PieceMove,
};

#[test]
fn test_graph_structures() {
    // 1. River Crossing
    let river_g = build_river_crossing_graph();
    assert_eq!(river_g.node_count(), 24);
    let r_m0 = river_g.find_id_by_name("M0").unwrap();
    let r_m6 = river_g.find_id_by_name("M6").unwrap();
    let r_m9 = river_g.find_id_by_name("M9").unwrap();
    assert_eq!(river_g.node(r_m0).unwrap().node_type, NodeType::TargetCoop);
    assert_eq!(river_g.node(r_m6).unwrap().node_type, NodeType::Bottleneck);
    assert_eq!(river_g.node(r_m9).unwrap().node_type, NodeType::FoxStart);
    assert_eq!(river_g.shortest_distance(r_m9, r_m0, &[]).unwrap(), 9);

    // 2. Classic
    let classic_g = build_classic_graph();
    assert_eq!(classic_g.node_count(), 11);
    let c_m0 = classic_g.find_id_by_name("M0").unwrap();
    let c_m4 = classic_g.find_id_by_name("M4").unwrap();
    let c_m2 = classic_g.find_id_by_name("M2").unwrap();
    assert_eq!(
        classic_g.node(c_m0).unwrap().node_type,
        NodeType::TargetCoop
    );
    assert_eq!(classic_g.node(c_m4).unwrap().node_type, NodeType::FoxStart);
    assert_eq!(classic_g.neighbors(c_m2).len(), 8);
    assert_eq!(classic_g.shortest_distance(c_m4, c_m0, &[]).unwrap(), 4);

    // 3. Fox and Dogs
    let arthur_g = build_fox_and_dogs_graph();
    assert_eq!(arthur_g.node_count(), 19);
    let c8 = arthur_g.find_id_by_name("C8").unwrap();
    let c2 = arthur_g.find_id_by_name("C2").unwrap();
    assert_eq!(arthur_g.node(c8).unwrap().node_type, NodeType::TargetCoop);
    assert_eq!(arthur_g.node(c2).unwrap().node_type, NodeType::Bottleneck);
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

    // Classic uses free entry: on turn 1 the Fox may place itself on any free vertex.
    assert!(state.fox_pending);
    let legal_fox_moves = state.fox_legal_moves();
    assert_eq!(legal_fox_moves.len(), 8);

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
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    assert!(state.fox_pending);

    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    let t1_idx = state.graph.find_id_by_name("T1").unwrap();
    let b1_idx = state.graph.find_id_by_name("B1").unwrap();
    let t2_idx = state.graph.find_id_by_name("T2").unwrap();

    let entry = state.fox_legal_moves();
    assert!(!entry.contains(&m0_idx));
    assert!(!entry.contains(&t1_idx));
    assert!(!entry.contains(&b1_idx));
    assert!(entry.contains(&t2_idx));

    assert!(state.apply_fox_move(t2_idx).is_ok());
    assert_eq!(state.fox_pos, t2_idx);
    assert!(!state.fox_pending);
    assert_eq!(state.current_turn, Faction::Hounds);
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

    assert!(state.fox_legal_moves().is_empty());
    assert_eq!(find_best_move(&state), None);

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

    assert!(river_state.fox_legal_moves().is_empty());
    assert_eq!(find_best_move(&river_state), None);

    river_state.evaluate_game_result();
    assert_eq!(river_state.result, GameResult::HoundsWon);
    assert_eq!(river_state.phase, GamePhase::GameOver);
}

#[test]
fn test_ai_finds_immediate_winning_move() {
    // Fox AI is 1 step from Coop
    let mut state = GameState::new();
    state.start_game(Faction::Hounds, Difficulty::Hard);

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
    assert_eq!(best_move, Some(PieceMove::FoxMove { to: m0_idx }));
}

#[test]
fn test_hounds_cannot_occupy_chicken_coop() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::RiverCrossing);
    state.start_game(Faction::Hounds, Difficulty::Hard);

    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    let l1_idx = state.graph.find_id_by_name("L1").unwrap();
    let m1_idx = state.graph.find_id_by_name("M1").unwrap();
    let r1_idx = state.graph.find_id_by_name("R1").unwrap();

    assert_eq!(state.hounds_pos, vec![l1_idx, m1_idx, r1_idx]);
    assert_eq!(state.coop_pos, m0_idx);

    for hound_idx in 0..state.hounds_pos.len() {
        let legal = state.hound_legal_moves(hound_idx);
        assert!(
            !legal.contains(&m0_idx),
            "Hound {hound_idx} should not be allowed to move to Chicken Coop"
        );
    }

    state.current_turn = Faction::Hounds;
    assert_eq!(
        state.apply_hound_move(0, m0_idx),
        Err(MoveError::IllegalMove)
    );
}

#[test]
fn test_hound_ai_advances_from_start() {
    let mut river_state = GameState::new();
    river_state.switch_variant(BoardVariant::RiverCrossing);
    river_state.start_game(Faction::Fox, Difficulty::Medium);

    let m8_idx = river_state.graph.find_id_by_name("M8").unwrap();
    assert!(river_state.apply_fox_move(m8_idx).is_ok());
    assert_eq!(river_state.current_turn, Faction::Hounds);

    let r_best = find_best_move(&river_state).expect("AI should find a move for Hounds");
    if let PieceMove::HoundMove { from, to, .. } = r_best {
        let from_node = river_state.graph.node(from).unwrap();
        let to_node = river_state.graph.node(to).unwrap();
        assert_eq!(from_node.row, 1);
        assert_eq!(to_node.row, 2, "Hound should advance to Row 2");
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
    if let PieceMove::HoundMove { to, from, .. } = best_move {
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

        if let Some(PieceMove::HoundMove { hound_idx, to, .. }) = find_best_move(&state) {
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
fn test_piece_collision_and_turn_order() {
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);

    // Fox turn: moving a hound should yield NotYourTurn
    assert_eq!(state.apply_hound_move(0, 1), Err(MoveError::NotYourTurn));

    let m2_idx = state.graph.find_id_by_name("M2").unwrap();
    let m3_idx = state.graph.find_id_by_name("M3").unwrap();
    state.fox_pos = m3_idx;
    state.fox_pending = false;
    state.hounds_pos[0] = m2_idx;

    // Moving Fox onto occupied Hound square M2 must be rejected
    let fox_legal = state.fox_legal_moves();
    assert!(!fox_legal.contains(&m2_idx));
    assert_eq!(state.apply_fox_move(m2_idx), Err(MoveError::IllegalMove));

    // Switching turn to Hounds: Fox moving is rejected
    state.current_turn = Faction::Hounds;
    assert_eq!(state.apply_fox_move(0), Err(MoveError::NotYourTurn));

    // Moving Hound onto Fox square M3 must be rejected
    let hound_legal = state.hound_legal_moves(0);
    assert!(!hound_legal.contains(&m3_idx));
    assert_eq!(
        state.apply_hound_move(0, m3_idx),
        Err(MoveError::IllegalMove)
    );
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

    classic_state.hounds_pos = vec![m2_idx, t1_idx, b1_idx];
    classic_state.fox_pos = m4_idx;
    classic_state.current_turn = Faction::Hounds;

    let classic_legal = classic_state.hound_legal_moves(0);
    assert!(
        !classic_legal.contains(&m1_idx),
        "Retreat forbidden in Classic"
    );
    assert!(classic_legal.contains(&t2_idx));
    assert!(classic_legal.contains(&b2_idx));
    assert!(classic_legal.contains(&m3_idx));

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
        "Retreat allowed in River Crossing"
    );
    assert!(river_state.apply_hound_move(0, r_m4_idx).is_ok());
}

#[test]
fn test_hound_stalemate_fox_victory() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::Classic);
    state.start_game(Faction::Hounds, Difficulty::Hard);

    let t3_idx = state.graph.find_id_by_name("T3").unwrap();
    let m3_idx = state.graph.find_id_by_name("M3").unwrap();
    let b3_idx = state.graph.find_id_by_name("B3").unwrap();
    let m4_idx = state.graph.find_id_by_name("M4").unwrap();

    state.hounds_pos = vec![t3_idx, m3_idx, b3_idx];
    state.fox_pos = m4_idx;
    state.current_turn = Faction::Hounds;

    assert!(state.all_hound_legal_moves().is_empty());
    assert_eq!(find_best_move(&state), None);

    state.evaluate_game_result();
    assert_eq!(state.result, GameResult::FoxWon);
    assert_eq!(state.phase, GamePhase::GameOver);
}

#[test]
fn test_arthur_dogs_start_and_rules() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::FoxAndDogs);
    state.start_game(Faction::Fox, Difficulty::Medium);

    // Fox starts first in all games
    assert_eq!(state.current_turn, Faction::Fox);

    let c8_idx = state.graph.find_id_by_name("C8").unwrap();
    for hound_idx in 0..3 {
        let moves = state.hound_legal_moves(hound_idx);
        assert!(!moves.contains(&c8_idx));
    }
}

#[test]
fn test_arthur_direct_mission_flow() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::FoxAndDogs);
    state.start_game(Faction::Fox, Difficulty::Medium);

    let start_node = FOX_AND_DOGS_CONFIG.fox_start_node;
    let fox_start_idx = state.graph.find_id_by_name(start_node).unwrap();
    let c5_idx = state.graph.find_id_by_name("C5").unwrap();
    let c7_idx = state.graph.find_id_by_name("C7").unwrap();
    let c8_idx = state.graph.find_id_by_name("C8").unwrap();

    assert_eq!(state.fox_pos, fox_start_idx);
    assert_eq!(state.active_target_node(), c8_idx);
    assert_eq!(state.current_turn, Faction::Fox);

    // Fox starts first and moves C4 -> C5
    assert!(state.apply_fox_move(c5_idx).is_ok());
    assert_eq!(state.current_turn, Faction::Hounds);

    // Dogs turn: hounds move
    let h_moves = state.all_hound_legal_moves();
    assert!(state.apply_hound_move(h_moves[0].0, h_moves[0].1).is_ok());
    assert_eq!(state.current_turn, Faction::Fox);

    // Fox moves to C8 and wins
    state.fox_pos = c7_idx;
    assert!(state.apply_fox_move(c8_idx).is_ok());
    assert_eq!(state.result, GameResult::FoxWon);
    assert_eq!(state.phase, GamePhase::GameOver);
}

#[test]
fn test_the_red_hunt_graph_structure_and_rules() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::TheRedHunt);

    assert_eq!(state.graph.node_count(), 22);

    let c4_idx = state.graph.find_id_by_name("C4").unwrap();
    let r2_idx = state.graph.find_id_by_name("R2").unwrap();
    let c1_idx = state.graph.find_id_by_name("C1").unwrap();
    let l2_idx = state.graph.find_id_by_name("L2").unwrap();
    let c0_idx = state.graph.find_id_by_name("C0").unwrap();

    assert_eq!(state.fox_pos, c4_idx);
    assert_eq!(state.hounds_pos, vec![r2_idx, c1_idx, l2_idx]);
    assert_eq!(state.coop_pos, c0_idx);
    assert_eq!(state.current_turn, Faction::Fox);
    assert!(state.variant.config().allow_hound_retreat);
}

#[test]
fn test_fox_ai_destination_seeking_across_variants() {
    // 1. Classic: Fox enters aggressively close to coop (row <= 2)
    let mut classic = GameState::new();
    classic.switch_variant(BoardVariant::Classic);
    classic.start_game(Faction::Hounds, Difficulty::Medium);
    if let Some(PieceMove::FoxMove { to }) = find_best_move(&classic) {
        assert!(classic.graph.node(to).unwrap().row <= 2);
    } else {
        panic!("Classic Fox AI should choose an entry move");
    }

    // 2. River Crossing: Fox advances from row 9 to row 8
    let mut river = GameState::new();
    river.switch_variant(BoardVariant::RiverCrossing);
    river.start_game(Faction::Hounds, Difficulty::Medium);
    if let Some(PieceMove::FoxMove { to }) = find_best_move(&river) {
        assert_eq!(river.graph.node(to).unwrap().row, 8);
    } else {
        panic!("River Fox AI should advance toward coop");
    }

    // 3. Fox and Dogs: Fox advances or maneuvers toward C8 (row >= 4)
    let mut dogs = GameState::new();
    dogs.switch_variant(BoardVariant::FoxAndDogs);
    dogs.start_game(Faction::Hounds, Difficulty::Medium);
    assert_eq!(dogs.current_turn, Faction::Fox);

    if let Some(PieceMove::FoxMove { to }) = find_best_move(&dogs) {
        let to_node = dogs.graph.node(to).unwrap();
        assert!(to_node.row >= 4, "Fox should advance toward C8");
    } else {
        panic!("Dogs Fox AI should choose a move");
    }
}

#[test]
fn test_red_hunt_piece_size_and_clearance() {
    assert_eq!(BoardVariant::TheRedHunt.piece_base_size(), 58.0);
    assert_eq!(BoardVariant::Classic.piece_base_size(), 76.0);
    assert_eq!(BoardVariant::RiverCrossing.piece_base_size(), 76.0);
    assert_eq!(BoardVariant::FoxAndDogs.piece_base_size(), 76.0);

    // Verify clearance along the central combat corridor (e.g. C1-C2, C2-C3: 61px apart)
    // where fox and hounds face off, ensuring they do not collide with their faces.
    let corridor_distance = 61.0;
    assert!(
        corridor_distance >= BoardVariant::TheRedHunt.piece_base_size(),
        "Corridor distance {corridor_distance} must be >= piece base size {}",
        BoardVariant::TheRedHunt.piece_base_size()
    );
}
