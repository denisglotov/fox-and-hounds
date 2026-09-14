use fox_and_hounds::game::ai::find_best_move;
use fox_and_hounds::game::graph::NodeType;
use fox_and_hounds::game::level::{
    build_classic_graph, build_fox_and_dogs_graph, build_fox_and_dogs_maze_graph,
    build_river_crossing_graph, BoardVariant,
};
use fox_and_hounds::game::state::{
    Difficulty, Faction, GamePhase, GameResult, GameState, MoveError, PieceMove,
};

#[test]
fn test_graph_structures_and_metrics() {
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
    assert_eq!(arthur_g.node(c8).unwrap().node_type, NodeType::FoxStart);
    assert_eq!(arthur_g.node(c2).unwrap().node_type, NodeType::Bottleneck);

    // 4. Fox and Dogs Maze (R4 disconnected from R5)
    let maze_g = build_fox_and_dogs_maze_graph();
    assert_eq!(maze_g.node_count(), 19);
    let m_r4 = maze_g.find_id_by_name("R4").unwrap();
    let m_r5 = maze_g.find_id_by_name("R5").unwrap();
    assert!(!maze_g.neighbors(m_r4).contains(&m_r5));

    // 5. The Red Hunt piece sizing & central corridor clearance
    let red_g = (BoardVariant::TheRedHunt.config().build_graph)();
    assert_eq!(red_g.node_count(), 22);
    let c1 = red_g.find_id_by_name("C1").unwrap();
    let c2 = red_g.find_id_by_name("C2").unwrap();
    let corridor_dist =
        (red_g.node(c1).unwrap().visual_pos - red_g.node(c2).unwrap().visual_pos).length();
    assert!(corridor_dist >= BoardVariant::TheRedHunt.piece_base_size());
}

#[test]
fn test_initial_state_legal_moves_and_free_entry() {
    // 1. Classic variant: Fox opening free entry (coop and hound nodes excluded)
    let mut state = GameState::new();
    state.start_game(Faction::Fox, Difficulty::Medium);
    assert_eq!(state.variant, BoardVariant::Classic);
    assert_eq!(state.current_turn, Faction::Fox);
    assert!(state.fox_pending);

    let m0_idx = state.graph.find_id_by_name("M0").unwrap();
    let t1_idx = state.graph.find_id_by_name("T1").unwrap();
    let t2_idx = state.graph.find_id_by_name("T2").unwrap();
    let legal = state.fox_legal_moves();
    assert_eq!(legal.len(), 8);
    assert!(!legal.contains(&m0_idx));
    assert!(!legal.contains(&t1_idx));
    assert!(legal.contains(&t2_idx));

    assert!(state.apply_fox_move(t2_idx).is_ok());
    assert_eq!(state.fox_pos, t2_idx);
    assert!(!state.fox_pending);
    assert_eq!(state.current_turn, Faction::Hounds);

    // 2. River Crossing variant opening
    let mut river = GameState::new();
    river.switch_variant(BoardVariant::RiverCrossing);
    river.start_game(Faction::Fox, Difficulty::Medium);
    let m8_idx = river.graph.find_id_by_name("M8").unwrap();
    assert!(river.apply_fox_move(m8_idx).is_ok());
    assert_eq!(river.fox_pos, m8_idx);
    assert_eq!(river.current_turn, Faction::Hounds);

    // 3. Piece collision & turn order validation
    assert_eq!(river.apply_fox_move(m8_idx), Err(MoveError::NotYourTurn));
    assert_eq!(river.apply_hound_move(99, 0), Err(MoveError::InvalidHound));
}

#[test]
fn test_victory_and_defeat_conditions() {
    // 1. Fox reaches coop in Classic
    let mut classic = GameState::new();
    classic.start_game(Faction::Fox, Difficulty::Medium);
    let m1 = classic.graph.find_id_by_name("M1").unwrap();
    let m0 = classic.graph.find_id_by_name("M0").unwrap();
    classic.fox_pos = m1;
    classic.fox_pending = false;
    classic.hounds_pos = vec![
        classic.graph.find_id_by_name("T3").unwrap(),
        classic.graph.find_id_by_name("M3").unwrap(),
        classic.graph.find_id_by_name("B3").unwrap(),
    ];
    assert!(classic.apply_fox_move(m0).is_ok());
    assert_eq!(classic.result, GameResult::FoxWon);
    assert_eq!(classic.phase, GamePhase::GameOver);

    // 2. Fox trapped by Hounds (checkmate)
    let mut trapped = GameState::new();
    trapped.start_game(Faction::Hounds, Difficulty::Medium);
    trapped.fox_pos = classic.graph.find_id_by_name("M4").unwrap();
    trapped.fox_pending = false;
    trapped.hounds_pos = classic.hounds_pos.clone();
    trapped.current_turn = Faction::Fox;
    assert!(trapped.fox_legal_moves().is_empty());
    trapped.evaluate_game_result();
    assert_eq!(trapped.result, GameResult::HoundsWon);

    // 3. Hound stalemate -> Fox victory
    let mut stale = GameState::new();
    stale.start_game(Faction::Hounds, Difficulty::Hard);
    stale.hounds_pos = classic.hounds_pos;
    stale.fox_pos = classic.graph.find_id_by_name("M4").unwrap();
    stale.current_turn = Faction::Hounds;
    assert!(stale.all_hound_legal_moves().is_empty());
    stale.evaluate_game_result();
    assert_eq!(stale.result, GameResult::FoxWon);

    // 4. Player victory matching logic
    let check_won = |res: GameResult, fac: Faction| {
        matches!(
            (res, fac),
            (GameResult::FoxWon, Faction::Fox) | (GameResult::HoundsWon, Faction::Hounds)
        )
    };
    assert!(check_won(GameResult::FoxWon, Faction::Fox));
    assert!(check_won(GameResult::HoundsWon, Faction::Hounds));
    assert!(!check_won(GameResult::FoxWon, Faction::Hounds));
    assert!(!check_won(GameResult::HoundsWon, Faction::Fox));
}

#[test]
fn test_movement_rules_and_retreat_restrictions() {
    // 1. Classic: Hounds cannot retreat
    let mut classic = GameState::new();
    classic.start_game(Faction::Hounds, Difficulty::Medium);
    let m1 = classic.graph.find_id_by_name("M1").unwrap();
    let m2 = classic.graph.find_id_by_name("M2").unwrap();
    let t1 = classic.graph.find_id_by_name("T1").unwrap();
    let b1 = classic.graph.find_id_by_name("B1").unwrap();
    classic.hounds_pos = vec![m2, t1, b1];
    classic.current_turn = Faction::Hounds;
    let moves = classic.hound_legal_moves(0);
    assert!(!moves.contains(&m1), "Retreat forbidden in Classic");

    // 2. River Crossing: Hounds can retreat
    let mut river = GameState::new();
    river.switch_variant(BoardVariant::RiverCrossing);
    river.start_game(Faction::Hounds, Difficulty::Medium);
    let r_m5 = river.graph.find_id_by_name("M5").unwrap();
    let r_m4 = river.graph.find_id_by_name("M4").unwrap();
    let r_m0 = river.graph.find_id_by_name("M0").unwrap();
    river.hounds_pos = vec![
        r_m5,
        river.graph.find_id_by_name("L4").unwrap(),
        river.graph.find_id_by_name("R4").unwrap(),
    ];
    river.current_turn = Faction::Hounds;
    assert!(
        river.hound_legal_moves(0).contains(&r_m4),
        "Retreat allowed in River Crossing"
    );

    // 3. Chicken coop cannot be occupied by Hounds
    for h_idx in 0..river.hounds_pos.len() as u8 {
        assert!(!river.hound_legal_moves(h_idx).contains(&r_m0));
    }
}

#[test]
fn test_hound_ai_pursuit_and_surrounding() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::RiverCrossing);
    state.start_game(Faction::Fox, Difficulty::Hard);

    let m8 = state.graph.find_id_by_name("M8").unwrap();
    let m6 = state.graph.find_id_by_name("M6").unwrap();
    assert!(state.apply_fox_move(m8).is_ok());

    // AI Hound advances from start
    let r_best = find_best_move(&state).expect("AI should find a hound move");
    if let PieceMove::HoundMove { from, to, .. } = r_best {
        assert_eq!(state.graph.node(from).unwrap().row, 1);
        assert_eq!(state.graph.node(to).unwrap().row, 2);
    } else {
        panic!("Expected HoundMove");
    }

    // AI Hound seizes bottleneck bridge M6
    state.hounds_pos = vec![
        state.graph.find_id_by_name("L5").unwrap(),
        state.graph.find_id_by_name("M5").unwrap(),
        state.graph.find_id_by_name("R5").unwrap(),
    ];
    state.current_turn = Faction::Hounds;
    let bridge_move = find_best_move(&state).expect("AI should take bridge");
    if let PieceMove::HoundMove { to, .. } = bridge_move {
        assert_eq!(to, m6, "Hound AI should seize bottleneck bridge M6");
    } else {
        panic!("Expected HoundMove");
    }

    // Multi-turn pursuit advances average row
    state.start_game(Faction::Fox, Difficulty::Medium);
    for _ in 0..4 {
        let f_moves = state.fox_legal_moves();
        if f_moves.is_empty() {
            break;
        }
        let chosen = *f_moves
            .iter()
            .max_by_key(|&&m| state.graph.node(m).unwrap().row)
            .unwrap();
        assert!(state.apply_fox_move(chosen).is_ok());
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
    assert!(end_avg_row > 2.0);
}

#[test]
fn test_fox_ai_pathfinding_and_goal_seeking() {
    // 1. Fox AI finds immediate winning move
    let mut state = GameState::new();
    state.start_game(Faction::Hounds, Difficulty::Hard);
    let m1 = state.graph.find_id_by_name("M1").unwrap();
    let m0 = state.graph.find_id_by_name("M0").unwrap();
    state.fox_pos = m1;
    state.fox_pending = false;
    state.hounds_pos = vec![
        state.graph.find_id_by_name("T3").unwrap(),
        state.graph.find_id_by_name("M3").unwrap(),
        state.graph.find_id_by_name("B3").unwrap(),
    ];
    state.current_turn = Faction::Fox;
    assert_eq!(find_best_move(&state), Some(PieceMove::FoxMove { to: m0 }));

    // 2. Classic Fox AI chooses entry move close to coop (row <= 2)
    let mut classic = GameState::new();
    classic.start_game(Faction::Hounds, Difficulty::Medium);
    if let Some(PieceMove::FoxMove { to }) = find_best_move(&classic) {
        assert!(classic.graph.node(to).unwrap().row <= 2);
    } else {
        panic!("Classic Fox AI should choose an entry move");
    }

    // 3. River Crossing Fox advances toward coop
    let mut river = GameState::new();
    river.switch_variant(BoardVariant::RiverCrossing);
    river.start_game(Faction::Hounds, Difficulty::Medium);
    if let Some(PieceMove::FoxMove { to }) = find_best_move(&river) {
        assert_eq!(river.graph.node(to).unwrap().row, 8);
    } else {
        panic!("River Fox AI should advance toward coop");
    }
}

#[test]
fn test_fox_and_dogs_symmetric_and_maze_rules() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::FoxAndDogs);
    state.start_game(Faction::Fox, Difficulty::Medium);

    // Dogs start first
    assert_eq!(state.current_turn, Faction::Hounds);
    let c8 = state.graph.find_id_by_name("C8").unwrap();
    let c7 = state.graph.find_id_by_name("C7").unwrap();
    let c6 = state.graph.find_id_by_name("C6").unwrap();
    let l7 = state.graph.find_id_by_name("L7").unwrap();

    // Dogs cannot enter C8 while Fox is on C8
    for h in 0..3 {
        assert!(!state.hound_legal_moves(h).contains(&c8));
    }

    // Dogs move C7 -> C6
    let c7_idx = state.hounds_pos.iter().position(|&p| p == c7).unwrap() as u8;
    assert!(state.apply_hound_move(c7_idx, c6).is_ok());

    // Fox moves C8 -> C7
    assert!(state.apply_fox_move(c7).is_ok());
    assert!(state.fox_has_left_start);

    // C8 is now empty: adjacent dogs can jump to C8
    let l7_idx = state.hounds_pos.iter().position(|&p| p == l7).unwrap() as u8;
    assert!(state.hound_legal_moves(l7_idx).contains(&c8));

    // Fox returns to C8 -> Fox victory
    state.current_turn = Faction::Fox;
    assert!(state.apply_fox_move(c8).is_ok());
    assert_eq!(state.result, GameResult::FoxWon);
    assert_eq!(state.phase, GamePhase::GameOver);

    // Maze variant opening & AI move
    let mut maze = GameState::new();
    maze.switch_variant(BoardVariant::FoxAndDogsMaze);
    maze.start_game(Faction::Fox, Difficulty::Hard);
    assert_eq!(maze.current_turn, Faction::Hounds);
    let dog_ai = find_best_move(&maze).expect("AI should find opening hound move");
    if let PieceMove::HoundMove { hound_idx, to, .. } = dog_ai {
        assert!(maze.apply_hound_move(hound_idx, to).is_ok());
        assert!(!maze.fox_legal_moves().is_empty());
    } else {
        panic!("Expected HoundMove");
    }
}

#[test]
fn test_the_red_hunt_rules_and_ai() {
    let mut state = GameState::new();
    state.switch_variant(BoardVariant::TheRedHunt);
    state.start_game(Faction::Fox, Difficulty::Hard);

    let c4 = state.graph.find_id_by_name("C4").unwrap();
    let c0 = state.graph.find_id_by_name("C0").unwrap();
    assert_eq!(state.fox_pos, c4);
    assert_eq!(state.coop_pos, c0);
    assert_eq!(state.current_turn, Faction::Fox);
    assert!(state.variant.config().allow_hound_retreat);

    // Fox has valid opening moves & AI finds move
    assert!(!state.fox_legal_moves().is_empty());
    assert!(find_best_move(&state).is_some());
}
