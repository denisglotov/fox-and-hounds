//! Board hints, notices and reticle geometry for `ui::board_view`.
//!
//! Every rule here is driven through the public `BoardViewHints` state and the free layout
//! helpers, so no renderer, texture or match window is involved: the idle timers, the objective
//! reticle window and the special rule notice lifecycle are all pure functions of the game state.

use fox_and_hounds::game::level::BoardVariant;
use fox_and_hounds::game::state::{Difficulty, Faction, GamePhase, GameState, MoveAnimation};
use fox_and_hounds::ui::board_view::{
    fit_special_rule_notice_font_size, is_hound_retreat_click, roll_sit_threshold,
    should_highlight_fox_objective, should_show_special_rule_notice, special_rule_notice_center,
    target_arrow_vertices, wants_special_rule_notice, BoardViewHints,
    FOX_OBJECTIVE_HINT_IDLE_SECONDS, MIN_IDLE_SIT_SECONDS, RANDOM_IDLE_SIT_SECONDS_RANGE,
    SPECIAL_RULE_NOTICE_MIN_FONT_SIZE, SPECIAL_RULE_NOTICE_PIECE_CLEARANCE,
    SPECIAL_RULE_REMINDER_DURATION,
};
use macroquad::prelude::Vec2;

/// A fresh hints state with deterministic settle thresholds, so a hound's sit timing is pinned
/// rather than the random wait `BoardViewHints::new` rolls for it.
fn hints_with_fixed_sit_thresholds() -> BoardViewHints {
    BoardViewHints {
        hound_sit_thresholds: [MIN_IDLE_SIT_SECONDS; 3],
        ..BoardViewHints::new()
    }
}

#[test]
fn test_hound_idle_bounds_safety_and_settle_thresholds() {
    // Every hound gets its own wait inside the documented window, so one can settle early and
    // another late on the same board
    for _ in 0..64 {
        let threshold = roll_sit_threshold();
        assert!(
            (MIN_IDLE_SIT_SECONDS..MIN_IDLE_SIT_SECONDS + RANDOM_IDLE_SIT_SECONDS_RANGE)
                .contains(&threshold),
            "sit threshold {threshold} outside the idle window"
        );
    }

    let mut hints = hints_with_fixed_sit_thresholds();
    assert_eq!(hints.hound_idle_times, [0.0; 3]);
    assert_eq!(hints.hound_sit_blend, [0.0; 3]);

    // A hound that idles past its own threshold settles down to sit
    assert!(!hints.is_hound_sitting(0));
    hints.update_hound_idle(0, false, 12.0);
    assert!(hints.is_hound_sitting(0));

    // Acting again clears the wait and rolls a fresh threshold for the next time it rests
    hints.update_hound_idle(0, true, 0.0);
    assert!(!hints.is_hound_sitting(0));
    assert_eq!(hints.hound_idle_times[0], 0.0);
    assert!(hints.hound_sit_thresholds[0] >= MIN_IDLE_SIT_SECONDS);

    // Out-of-range indices are never sitting and never update, rather than panicking
    assert!(!hints.is_hound_sitting(3));
    assert!(!hints.is_hound_sitting(10));
    hints.update_hound_idle(3, false, 12.0);
    hints.update_hound_idle(99, true, 1.0);
    assert_eq!(hints.hound_idle_times, [0.0; 3]);
    // ...and they leave the hounds that do exist untouched
    assert_eq!(&hints.hound_sit_thresholds[1..], &[MIN_IDLE_SIT_SECONDS; 2]);
}

#[test]
fn test_reset_clears_the_fades_but_keeps_hounds_settled() {
    let mut hints = hints_with_fixed_sit_thresholds();
    hints.update_hound_idle(1, false, 12.0);
    hints.hound_sit_blend[1] = 1.0;
    hints.start_target_alpha = 0.4;
    hints.special_rule_notice_alpha = 0.6;
    hints.special_rule_reminder_seconds = SPECIAL_RULE_REMINDER_DURATION;
    hints.fox_idle_seconds = 7.5;

    hints.reset();

    // A new match inherits none of the fades or waits the previous one left behind
    assert_eq!(hints.start_target_alpha, 0.0);
    assert_eq!(hints.special_rule_notice_alpha, 0.0);
    assert_eq!(hints.special_rule_reminder_seconds, 0.0);
    assert_eq!(hints.fox_idle_seconds, 0.0);

    // ...but a hound that was already sitting must not stand up and re-roll under the player
    assert!(hints.is_hound_sitting(1));
    assert_eq!(hints.hound_idle_times[1], 12.0);
    assert_eq!(hints.hound_sit_blend[1], 1.0);
    assert_eq!(hints.hound_sit_thresholds[1], MIN_IDLE_SIT_SECONDS);
}

#[test]
fn test_objective_hint_on_opening_move_and_after_fox_idles() {
    const IDLE: f32 = FOX_OBJECTIVE_HINT_IDLE_SECONDS;

    // Fox-controlled match: the hint is on until the very first move
    let mut fox_game = GameState::new();
    fox_game.start_game(Faction::Fox, Difficulty::Medium);
    assert!(fox_game.move_history.is_empty());
    assert!(should_highlight_fox_objective(&fox_game, 0.0));

    let opening = fox_game.fox_legal_moves();
    assert!(!opening.is_empty());
    assert!(fox_game.apply_fox_move(opening[0]).is_ok());

    // Now the hounds are to move, so it stays hidden no matter how long they take
    assert!(!should_highlight_fox_objective(&fox_game, IDLE * 100.0));

    // Once the hounds have replied the hint waits for the Fox player to idle again
    let hound_moves = fox_game.all_hound_legal_moves();
    assert!(!hound_moves.is_empty());
    assert!(fox_game
        .apply_hound_move(hound_moves[0].0, hound_moves[0].1)
        .is_ok());
    assert_eq!(fox_game.current_turn, Faction::Fox);
    assert!(!fox_game.move_history.is_empty());

    // ...but not until that hound has finished gliding and the board settles
    assert!(!should_highlight_fox_objective(&fox_game, IDLE * 100.0));
    fox_game.active_anim = None;

    // Just short of the wait it is hidden, then it comes back as a reminder
    assert!(!should_highlight_fox_objective(&fox_game, IDLE - 0.1));
    assert!(should_highlight_fox_objective(&fox_game, IDLE));

    // Hound-controlled matches never advertise the Fox objective, however long they idle
    let mut hound_game = GameState::new();
    hound_game.start_game(Faction::Hounds, Difficulty::Medium);
    assert!(!should_highlight_fox_objective(&hound_game, 0.0));
    assert!(!should_highlight_fox_objective(&hound_game, IDLE * 100.0));

    // Finished matches (and the title screen) never show it either
    let mut finished = GameState::new();
    finished.start_game(Faction::Fox, Difficulty::Medium);
    finished.phase = GamePhase::GameOver;
    assert!(!should_highlight_fox_objective(&finished, IDLE * 100.0));

    let mut titled = GameState::new();
    titled.start_game(Faction::Fox, Difficulty::Medium);
    titled.phase = GamePhase::TitleScreen;
    assert!(!should_highlight_fox_objective(&titled, IDLE * 100.0));
}

#[test]
fn test_fox_idle_timer_accrues_only_while_fox_waits() {
    let mut fox_game = GameState::new();
    fox_game.start_game(Faction::Fox, Difficulty::Medium);

    // Idling on the Fox turn accumulates the wait
    let mut hints = BoardViewHints::new();
    hints.update_fox_idle(&fox_game, 4.0);
    assert!((hints.fox_idle_seconds - 4.0).abs() < 1e-6);
    hints.update_fox_idle(&fox_game, 6.5);
    assert!((hints.fox_idle_seconds - 10.5).abs() < 1e-6);
    assert!(should_highlight_fox_objective(
        &fox_game,
        hints.fox_idle_seconds
    ));

    // Moving the fox hands the turn to the hounds and clears the wait
    let opening = fox_game.fox_legal_moves();
    assert!(fox_game.apply_fox_move(opening[0]).is_ok());
    hints.update_fox_idle(&fox_game, 1.0);
    assert_eq!(hints.fox_idle_seconds, 0.0);

    // Nor does it run while a piece is still gliding, or once the match is over
    let mut gliding = GameState::new();
    gliding.start_game(Faction::Fox, Difficulty::Medium);
    gliding.active_anim = Some(MoveAnimation {
        from: Vec2::ZERO,
        to: Vec2::ZERO,
        progress: 0.5,
        duration: 1.0,
        faction: Faction::Fox,
        hound_idx: None,
    });
    hints.fox_idle_seconds = 3.0;
    hints.update_fox_idle(&gliding, 1.0);
    assert_eq!(hints.fox_idle_seconds, 0.0);

    let mut over = GameState::new();
    over.start_game(Faction::Fox, Difficulty::Medium);
    over.phase = GamePhase::GameOver;
    hints.fox_idle_seconds = 3.0;
    hints.update_fox_idle(&over, 1.0);
    assert_eq!(hints.fox_idle_seconds, 0.0);

    // A negative dt can never rewind the wait while the fox is on the clock
    let mut fresh = GameState::new();
    fresh.start_game(Faction::Fox, Difficulty::Medium);
    hints.fox_idle_seconds = 3.0;
    hints.update_fox_idle(&fresh, -1.0);
    assert!((hints.fox_idle_seconds - 3.0).abs() < 1e-6);
}

#[test]
fn test_special_rule_notice_resurfaces_on_retreat_click() {
    let mut state = GameState::new();
    state.start_game(Faction::Hounds, Difficulty::Medium);

    let m0 = state.graph.find_id_by_name("M0").unwrap();
    let m1 = state.graph.find_id_by_name("M1").unwrap();
    let m2 = state.graph.find_id_by_name("M2").unwrap();
    let m3 = state.graph.find_id_by_name("M3").unwrap();

    // AI Fox opens at M3, player moves dog from M0 to M1 (row 0 -> row 1)
    assert!(state.apply_fox_move(m3).is_ok());
    let m0_idx = state.hounds_pos.iter().position(|&p| p == m0).unwrap() as u8;
    assert!(state.apply_hound_move(m0_idx, m1).is_ok());
    assert!(!should_show_special_rule_notice(&state));

    // Now hound at M1 (row 1) has neighbor M0 (row 0).
    // Clicking M0 (behind M1) is detected as an attempted retreat move
    state.selected_hound_idx = Some(m0_idx);
    assert!(is_hound_retreat_click(&state, m0));

    // Clicking forward to M2 (row 2) is a forward advance, not retreat
    assert!(!is_hound_retreat_click(&state, m2));

    // Even when no hound is actively selected, clicking M0 detects the retreat
    state.selected_hound_idx = None;
    assert!(is_hound_retreat_click(&state, m0));

    // Boards with retreat enabled never flag retreat clicks
    let mut river = GameState::new();
    river.switch_variant(BoardVariant::RiverCrossing);
    river.start_game(Faction::Hounds, Difficulty::Medium);
    assert!(!is_hound_retreat_click(&river, 0));

    // The click raises a reminder that runs for its full duration
    let mut hints = BoardViewHints::new();
    assert_eq!(hints.special_rule_reminder_seconds, 0.0);
    hints.trigger_special_rule_reminder();
    assert_eq!(
        hints.special_rule_reminder_seconds,
        SPECIAL_RULE_REMINDER_DURATION
    );
}

#[test]
fn test_special_rule_notice_opens_the_match_and_leaves_with_the_first_move() {
    // Classic forbids the hounds to fall back: the notice belongs to the opening turn
    let mut fox_game = GameState::new();
    assert!(!fox_game.variant.config().allow_hound_retreat);
    assert!(!should_show_special_rule_notice(&fox_game)); // still on the title screen

    fox_game.start_game(Faction::Fox, Difficulty::Medium);
    assert!(should_show_special_rule_notice(&fox_game));

    // A Fox player loses the notice on their own opening move
    let opening = fox_game.fox_legal_moves();
    assert!(!opening.is_empty());
    assert!(fox_game.apply_fox_move(opening[0]).is_ok());
    assert!(!should_show_special_rule_notice(&fox_game));

    // A Hounds player keeps it for their opening decision: the AI Fox has answered by
    // then, but the rule is the one they have to play by
    let mut hound_game = GameState::new();
    hound_game.start_game(Faction::Hounds, Difficulty::Medium);
    assert!(should_show_special_rule_notice(&hound_game));

    let opening = hound_game.fox_legal_moves();
    assert!(hound_game.apply_fox_move(opening[0]).is_ok());
    assert!(should_show_special_rule_notice(&hound_game));

    let hound_moves = hound_game.all_hound_legal_moves();
    assert!(!hound_moves.is_empty());
    assert!(hound_game
        .apply_hound_move(hound_moves[0].0, hound_moves[0].1)
        .is_ok());
    assert!(!should_show_special_rule_notice(&hound_game));

    // Boards with free hound movement never announce it
    let mut river = GameState::new();
    river.switch_variant(BoardVariant::RiverCrossing);
    river.start_game(Faction::Fox, Difficulty::Medium);
    assert!(river.variant.config().allow_hound_retreat);
    assert!(river.move_history.is_empty());
    assert!(!should_show_special_rule_notice(&river));

    // A fresh match on the Classic board brings it back
    fox_game.start_game(Faction::Fox, Difficulty::Medium);
    assert!(should_show_special_rule_notice(&fox_game));

    // ...and a finished match keeps it off the board
    fox_game.phase = GamePhase::GameOver;
    assert!(!should_show_special_rule_notice(&fox_game));
}

#[test]
fn test_special_rule_notice_waits_for_the_opening_zoom() {
    let mut game = GameState::new();
    game.start_game(Faction::Fox, Difficulty::Medium);
    assert!(should_show_special_rule_notice(&game));

    // The intro zoom owns the screen while it flies into the field: the notice waits for
    // the framing to settle and only then fades in
    assert!(!wants_special_rule_notice(&game, 0.0, false));
    assert!(wants_special_rule_notice(&game, 0.0, true));

    // An illegal retreat click is a direct answer to the player, so its reminder does not
    // wait for the camera
    assert!(wants_special_rule_notice(
        &game,
        SPECIAL_RULE_REMINDER_DURATION,
        false
    ));

    // The player's own opening move retires the announcement for good
    let opening = game.fox_legal_moves();
    assert!(!opening.is_empty());
    assert!(game.apply_fox_move(opening[0]).is_ok());
    assert!(!wants_special_rule_notice(&game, 0.0, true));

    // ...though a later illegal retreat click still brings it back as a reminder
    assert!(wants_special_rule_notice(
        &game,
        SPECIAL_RULE_REMINDER_DURATION,
        true
    ));

    // Boards with free hound movement never announce a rule, reminder or not
    let mut river = GameState::new();
    river.switch_variant(BoardVariant::RiverCrossing);
    river.start_game(Faction::Fox, Difficulty::Medium);
    assert!(!wants_special_rule_notice(
        &river,
        SPECIAL_RULE_REMINDER_DURATION,
        true
    ));

    // A finished match keeps it off the board too
    game.phase = GamePhase::GameOver;
    assert!(!wants_special_rule_notice(
        &game,
        SPECIAL_RULE_REMINDER_DURATION,
        true
    ));
}

#[test]
fn test_special_rule_notice_layout_clears_the_pieces_and_fits_the_field() {
    let framing = BoardVariant::Classic.config().intro_framing;
    let piece_size = BoardVariant::Classic.config().piece_base_size;

    // Classic: the notice drops into the clear strip below the hound line. The screen-level
    // counterpart of this check, which keeps the plate inside the viewport on every supported
    // resolution, lives in tests/layout_tests.rs.
    let lowest_node_y = (BoardVariant::Classic.config().build_graph)()
        .nodes
        .iter()
        .fold(f32::MIN, |lowest, node| lowest.max(node.visual_pos.y));

    let center = special_rule_notice_center(framing, lowest_node_y, piece_size);
    let field_bottom = framing.playable_center.y + framing.playable_size.y * 0.5;
    let piece_bottom = lowest_node_y + piece_size * 0.5 + SPECIAL_RULE_NOTICE_PIECE_CLEARANCE;
    assert!((center.x - framing.playable_center.x).abs() < 0.01);
    assert!(center.y > piece_bottom);
    assert!(center.y < field_bottom);

    // A board whose pieces reach the bottom edge of the field keeps the notice on the field
    let cramped = special_rule_notice_center(framing, field_bottom + 40.0, piece_size);
    assert!((cramped.y - field_bottom).abs() < 0.01);

    // A sentence that already fits keeps its base size
    assert_eq!(fit_special_rule_notice_font_size(20, 120.0, 200.0), 20);
    assert_eq!(fit_special_rule_notice_font_size(20, 0.0, 200.0), 20);

    // A long translation shrinks until it fits the plate
    let shrunk = fit_special_rule_notice_font_size(20, 400.0, 200.0);
    assert!(shrunk < 20);
    assert!(shrunk >= SPECIAL_RULE_NOTICE_MIN_FONT_SIZE);
    // ...but never below the legibility floor
    assert_eq!(
        fit_special_rule_notice_font_size(20, 10_000.0, 200.0),
        SPECIAL_RULE_NOTICE_MIN_FONT_SIZE
    );

    // Small base sizes below the floor never expand above base size when text overflows
    assert_eq!(fit_special_rule_notice_font_size(8, 400.0, 200.0), 8);
    assert_eq!(fit_special_rule_notice_font_size(8, 100.0, 200.0), 8);
}

#[test]
fn test_reticle_arrow_geometry_points_at_the_node() {
    let center = Vec2::new(300.0, 200.0);
    let (tip, left, right) = target_arrow_vertices(center, 0.0, 20.0, 30.0, 5.0);
    assert!((tip - Vec2::new(320.0, 200.0)).length() < 0.01);
    assert!((left - Vec2::new(330.0, 205.0)).length() < 0.01);
    assert!((right - Vec2::new(330.0, 195.0)).length() < 0.01);

    // The tip sits nearer the spot than the base, so the arrow targets the node
    assert!((tip - center).length() < (left - center).length());

    // Rotating an arrow keeps its tip on the reticle circle
    let (rotated_tip, _, _) =
        target_arrow_vertices(center, std::f32::consts::FRAC_PI_2, 20.0, 30.0, 5.0);
    assert!((rotated_tip - Vec2::new(300.0, 220.0)).length() < 0.01);
}
