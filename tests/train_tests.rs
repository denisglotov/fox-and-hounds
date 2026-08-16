use fox_and_hounds::ui::train::{
    TrainSimulation, CYCLE_DURATION, INITIAL_DELAY, TRACK_X, TRAIN_END_Y, TRAIN_HEIGHT,
    TRAIN_START_Y, TRAIN_WIDTH, TRANSIT_DURATION,
};

#[test]
fn test_train_initial_state() {
    let train = TrainSimulation::new();
    assert_eq!(train.elapsed_time, 0.0);
    assert!(!train.is_active()); // Waiting for initial 5-second delay
    assert_eq!(train.train_progress(), None);
    assert_eq!(train.train_locomotive_y(), None);
    assert_eq!(TRACK_X, 50.0);
    assert_eq!(INITIAL_DELAY, 5.0);
    assert_eq!(CYCLE_DURATION, 30.0);
    assert_eq!(TRAIN_WIDTH, 81.0);
    assert_eq!(TRAIN_HEIGHT, 486.0);
}

#[test]
fn test_train_30_second_cycle_with_5_second_initial_delay() {
    let mut train = TrainSimulation::new();

    // At t = 0.0s to 4.9s: Waiting during initial 5-second delay
    train.elapsed_time = 0.0;
    assert!(!train.is_active());
    assert_eq!(train.train_progress(), None);

    train.elapsed_time = 2.5;
    assert!(!train.is_active());
    assert_eq!(train.train_progress(), None);

    // At t = 5.0s: First train begins transit
    train.elapsed_time = 5.0;
    assert!(train.is_active());
    assert_eq!(train.train_progress(), Some(0.0));
    assert_eq!(train.train_locomotive_y(), Some(TRAIN_START_Y));

    // At halfway through first transit (t = 5.0 + 2.6 = 7.6s): Midpoint of board
    train.elapsed_time = 5.0 + TRANSIT_DURATION / 2.0;
    assert!(train.is_active());
    let prog = train.train_progress().unwrap();
    assert!((prog - 0.5).abs() < 1e-4);
    let expected_mid_y = TRAIN_START_Y + 0.5 * (TRAIN_END_Y - TRAIN_START_Y);
    assert!((train.train_locomotive_y().unwrap() - expected_mid_y).abs() < 1e-2);

    // At end of first transit (t = 5.0 + 5.2 = 10.2s): Train completes transit
    train.elapsed_time = 5.0 + TRANSIT_DURATION;
    assert!(!train.is_active());
    assert_eq!(train.train_progress(), None);

    // Quiet gameplay period between cycles (e.g. t = 20.0s): Inactive
    train.elapsed_time = 20.0;
    assert!(!train.is_active());
    assert_eq!(train.train_progress(), None);

    // At t = 35.0s (5.0s + 30.0s): Second train begins transit!
    train.elapsed_time = 35.0;
    assert!(train.is_active());
    assert_eq!(train.train_progress(), Some(0.0));
    assert_eq!(train.train_locomotive_y(), Some(TRAIN_START_Y));

    // At t = 65.0s (5.0s + 60.0s): Third train begins transit!
    train.elapsed_time = 65.0;
    assert!(train.is_active());
    assert_eq!(train.train_progress(), Some(0.0));
    assert_eq!(train.train_locomotive_y(), Some(TRAIN_START_Y));
}

#[test]
fn test_train_track_and_board_bounds() {
    assert!(TRAIN_START_Y < 0.0); // Starts above top screen edge
    assert!(TRAIN_END_Y > 1264.0); // Exits below bottom screen edge
    assert_eq!(TRACK_X, 50.0);
}

#[test]
fn test_train_dimensions_and_travel_span() {
    let train = TrainSimulation::new();
    let total_distance = TRAIN_END_Y - TRAIN_START_Y;
    assert!(total_distance >= 1600.0); // Covers the entire board plus off-screen clearance
    assert!(TRANSIT_DURATION >= 5.0 && TRANSIT_DURATION <= 6.0);
    assert_eq!(train.cycle_progress(), 0.0);
}
