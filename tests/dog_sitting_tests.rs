use fox_and_hounds::ui::board_view::{
    roll_sit_threshold, MIN_IDLE_SIT_SECONDS, RANDOM_IDLE_SIT_SECONDS_RANGE,
};

#[test]
fn test_sit_threshold_range() {
    for _ in 0..100 {
        let threshold = roll_sit_threshold();
        assert!(
            threshold >= MIN_IDLE_SIT_SECONDS,
            "Threshold {threshold} should be at least {MIN_IDLE_SIT_SECONDS}"
        );
        assert!(
            threshold <= MIN_IDLE_SIT_SECONDS + RANDOM_IDLE_SIT_SECONDS_RANGE,
            "Threshold {threshold} should not exceed {}",
            MIN_IDLE_SIT_SECONDS + RANDOM_IDLE_SIT_SECONDS_RANGE
        );
    }
}
