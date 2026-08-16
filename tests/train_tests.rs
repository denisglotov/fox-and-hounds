use fox_and_hounds::ui::train::{TrainSimulation, TRAIN_END_Y, TRAIN_START_Y, TRANSIT_DURATION};

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
