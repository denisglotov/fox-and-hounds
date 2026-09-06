use fox_and_hounds::ui::train::{TrainSimulation, TRAIN_END_Y, TRAIN_START_Y, TRANSIT_DURATION};

#[test]
fn test_train_60_second_cycle_with_15_second_initial_delay() {
    let mut train = TrainSimulation::new();

    // At t = 0.0s to 14.9s: Waiting during initial 15-second delay
    train.elapsed_time = 0.0;
    assert!(!train.is_active());
    assert_eq!(train.train_progress(), None);

    train.elapsed_time = 10.0;
    assert!(!train.is_active());
    assert_eq!(train.train_progress(), None);

    // At t = 15.0s: First train begins transit
    train.elapsed_time = 15.0;
    assert!(train.is_active());
    assert_eq!(train.train_progress(), Some(0.0));
    assert_eq!(train.train_locomotive_y(), Some(TRAIN_START_Y));

    // At halfway through first transit (t = 15.0 + 2.6 = 17.6s): Midpoint of board
    train.elapsed_time = 15.0 + TRANSIT_DURATION / 2.0;
    assert!(train.is_active());
    let prog = train.train_progress().unwrap();
    assert!((prog - 0.5).abs() < 1e-4);
    let expected_mid_y = TRAIN_START_Y + 0.5 * (TRAIN_END_Y - TRAIN_START_Y);
    assert!((train.train_locomotive_y().unwrap() - expected_mid_y).abs() < 1e-2);

    // At end of first transit (t = 15.0 + 5.2 = 20.2s): Train completes transit
    train.elapsed_time = 15.0 + TRANSIT_DURATION;
    assert!(!train.is_active());
    assert_eq!(train.train_progress(), None);

    // Quiet gameplay period between cycles (e.g. t = 45.0s): Inactive
    train.elapsed_time = 45.0;
    assert!(!train.is_active());
    assert_eq!(train.train_progress(), None);

    // At t = 75.0s (15.0s + 60.0s): Second train begins transit!
    train.elapsed_time = 75.0;
    assert!(train.is_active());
    assert_eq!(train.train_progress(), Some(0.0));
    assert_eq!(train.train_locomotive_y(), Some(TRAIN_START_Y));

    // At t = 135.0s (15.0s + 120.0s): Third train begins transit!
    train.elapsed_time = 135.0;
    assert!(train.is_active());
    assert_eq!(train.train_progress(), Some(0.0));
    assert_eq!(train.train_locomotive_y(), Some(TRAIN_START_Y));
}

#[test]
fn test_train_sound_trigger_lifecycle() {
    use fox_and_hounds::audio::SoundTrigger;

    let mut train = TrainSimulation::new();

    // Step through initial delay in 0.5s intervals (0.0 to 14.5s) -> No sound
    for _ in 0..29 {
        let snd = train.update(0.5);
        assert_eq!(snd, None);
    }
    assert!(!train.is_active());

    // Reach t = 15.0s -> Train begins transit, triggers SoundTrigger::Train!
    let snd = train.update(0.5);
    assert_eq!(snd, Some(SoundTrigger::Train));
    assert!(train.is_active());

    // Subsequent updates during active transit (15.5s, 16.0s, ... 20.0s) -> No duplicate sound
    for _ in 0..10 {
        let snd = train.update(0.5);
        assert_eq!(snd, None);
    }

    // Step across remainder of cycle (20.5s to 74.5s: 109 steps of 0.5s = 54.5s) -> No sound
    for _ in 0..109 {
        let snd = train.update(0.5);
        assert_eq!(snd, None);
    }
    assert!((train.elapsed_time - 74.5).abs() < 1e-4);

    // Cross into next cycle at t = 75.0s -> Triggers SoundTrigger::Train again!
    let snd = train.update(0.5);
    assert_eq!(snd, Some(SoundTrigger::Train));
    assert!(train.is_active());
}

#[test]
fn test_train_large_dt_step_no_panic() {
    let mut train = TrainSimulation::new();

    // Sudden 125-second frame spike (e.g., app sleep/resume)
    let _ = train.update(125.0);
    assert!(train.elapsed_time >= 125.0);
    // Should compute progress or inactivity cleanly without panic
    let _ = train.is_active();
    let _ = train.train_progress();
    let _ = train.train_locomotive_y();
}
