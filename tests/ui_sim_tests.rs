//! Ambient board simulations that are pure clocks: the floating paper boat on the Fox and Dogs
//! river and the Curiosity rover on the Red Hunt crust.
//!
//! Both cycles are pinned as tables of absolute times, so a timing change has to be deliberate:
//! every probe lands at least 0.1s away from a phase boundary, which keeps the assertions about
//! the phases themselves rather than about float accumulation.

use fox_and_hounds::audio::SoundTrigger;
use fox_and_hounds::ui::boat::{
    BoatSimulation, CYCLE_DURATION as BOAT_CYCLE_DURATION, INITIAL_DELAY as BOAT_INITIAL_DELAY,
    TRANSIT_DURATION,
};
use fox_and_hounds::ui::rover::{
    RoverSimulation, CYCLE_DURATION as ROVER_CYCLE_DURATION, FORWARD_DURATION,
    INITIAL_DELAY as ROVER_INITIAL_DELAY, REVERSE_DURATION, ROVER_CRATER_Y, ROVER_START_Y, ROVER_X,
};

/// One probe of the boat cycle: advance to `at`, then check what the boat reports.
struct BoatProbe {
    /// Absolute time since the match opened, in seconds.
    at: f32,
    /// Whether the boat is mid-crossing at that moment.
    active: bool,
    /// Expected transit progress, or `None` while it waits out the rest of the cycle.
    progress: Option<f32>,
}

#[test]
fn test_boat_crossing_cycles_across_a_full_period() {
    let mut boat = BoatSimulation::new();
    assert_eq!(boat.elapsed_time, 0.0);
    assert!(!boat.is_active());
    assert_eq!(boat.transit_progress(), None);

    let probes = [
        BoatProbe {
            at: BOAT_INITIAL_DELAY - 0.1,
            active: false,
            progress: None,
        },
        BoatProbe {
            at: BOAT_INITIAL_DELAY + 0.5,
            active: true,
            progress: Some(0.5 / TRANSIT_DURATION),
        },
        BoatProbe {
            at: BOAT_INITIAL_DELAY + TRANSIT_DURATION * 0.5,
            active: true,
            progress: Some(0.5),
        },
        BoatProbe {
            at: BOAT_INITIAL_DELAY + TRANSIT_DURATION + 0.1,
            active: false,
            progress: None,
        },
        BoatProbe {
            at: BOAT_INITIAL_DELAY + BOAT_CYCLE_DURATION + 0.5,
            active: true,
            progress: Some(0.5 / TRANSIT_DURATION),
        },
    ];

    let mut previous = 0.0;
    for probe in probes {
        boat.update(probe.at - previous);
        previous = probe.at;

        assert_eq!(
            boat.is_active(),
            probe.active,
            "boat active state at t = {:.2}s",
            probe.at
        );
        match probe.progress {
            Some(expected) => {
                let progress = boat.transit_progress().unwrap_or_else(|| {
                    panic!("crossing boat must report progress at t = {:.2}s", probe.at)
                });
                assert!(
                    (progress - expected).abs() < 1e-3,
                    "transit progress {progress} at t = {:.2}s (expected {expected})",
                    probe.at
                );
            }
            None => assert!(
                boat.transit_progress().is_none(),
                "waiting boat must not report progress at t = {:.2}s",
                probe.at
            ),
        }
    }
}

/// One probe of the rover cycle: advance to `at`, then check phase, lane and beep.
struct RoverProbe {
    /// Absolute time since the match opened, in seconds.
    at: f32,
    /// Whether the rover is in frame (on the move or parked at the crater).
    active: bool,
    /// True only while it is standing at the crater.
    near_crater: bool,
    /// Expected vertical position along its lane, or `None` while it is off screen.
    y: Option<f32>,
    /// Sound this step fires, when it crosses a beep or a phase boundary.
    sound: Option<SoundTrigger>,
    /// Why the probe is here, quoted when it fails.
    label: &'static str,
}

/// Where the rover sits `elapsed_into_phase` seconds into its descent to the crater.
fn descent_y(elapsed_into_phase: f32) -> f32 {
    ROVER_START_Y + (ROVER_CRATER_Y - ROVER_START_Y) * elapsed_into_phase / FORWARD_DURATION
}

/// Where the rover sits `elapsed_into_phase` seconds into its climb back off the crust.
fn climb_y(elapsed_into_phase: f32) -> f32 {
    ROVER_CRATER_Y + (ROVER_START_Y - ROVER_CRATER_Y) * elapsed_into_phase / REVERSE_DURATION
}

#[test]
fn test_rover_cycle_phases_and_beep_cadence() {
    let mut rover = RoverSimulation::new();
    assert!(!rover.is_active());
    assert!(!rover.is_near_crater());
    assert_eq!(rover.rover_pos(), None);

    let probes = [
        RoverProbe {
            at: 0.0,
            active: false,
            near_crater: false,
            y: None,
            sound: None,
            label: "opens off screen",
        },
        RoverProbe {
            at: ROVER_INITIAL_DELAY - 0.1,
            active: false,
            near_crater: false,
            y: None,
            sound: None,
            label: "still waiting out the initial delay",
        },
        RoverProbe {
            at: ROVER_INITIAL_DELAY + 0.5,
            active: true,
            near_crater: false,
            y: Some(descent_y(0.5)),
            sound: Some(SoundTrigger::RoverForward),
            label: "starts the descent with its first forward beep",
        },
        RoverProbe {
            at: ROVER_INITIAL_DELAY + 0.6,
            active: true,
            near_crater: false,
            y: Some(descent_y(0.6)),
            sound: None,
            label: "holds forward beeps to one per second",
        },
        RoverProbe {
            at: ROVER_INITIAL_DELAY + 1.5,
            active: true,
            near_crater: false,
            y: Some(descent_y(1.5)),
            sound: Some(SoundTrigger::RoverForward),
            label: "beeps again in the next second",
        },
        RoverProbe {
            at: ROVER_INITIAL_DELAY + FORWARD_DURATION + 0.5,
            active: true,
            near_crater: true,
            y: Some(ROVER_CRATER_Y),
            sound: None,
            label: "parks silently at the crater",
        },
        RoverProbe {
            at: ROVER_INITIAL_DELAY + FORWARD_DURATION + 2.5,
            active: true,
            near_crater: false,
            y: Some(climb_y(0.5)),
            sound: Some(SoundTrigger::RoverReverse),
            label: "backs away with its first warning beep",
        },
        RoverProbe {
            at: ROVER_INITIAL_DELAY + FORWARD_DURATION + 2.0 + REVERSE_DURATION + 0.5,
            active: false,
            near_crater: false,
            y: None,
            sound: None,
            label: "leaves the frame",
        },
        RoverProbe {
            at: ROVER_INITIAL_DELAY + ROVER_CYCLE_DURATION + 0.5,
            active: true,
            near_crater: false,
            y: Some(descent_y(0.5)),
            sound: Some(SoundTrigger::RoverForward),
            label: "starts the next pass a cycle later",
        },
    ];

    let mut previous = 0.0;
    for probe in probes {
        let sound = rover.update(probe.at - previous);
        previous = probe.at;

        assert_eq!(
            rover.is_active(),
            probe.active,
            "{}: rover active state at t = {:.2}s",
            probe.label,
            probe.at
        );
        assert_eq!(
            rover.is_near_crater(),
            probe.near_crater,
            "{}: crater parking at t = {:.2}s",
            probe.label,
            probe.at
        );
        assert_eq!(
            sound, probe.sound,
            "{}: sound at t = {:.2}s",
            probe.label, probe.at
        );

        match (probe.y, rover.rover_pos()) {
            (Some(expected_y), Some(pos)) => {
                assert_eq!(pos.x, ROVER_X, "{}: rover lane", probe.label);
                assert!(
                    (pos.y - expected_y).abs() < 0.01,
                    "{}: rover y {:.2} at t = {:.2}s (expected {expected_y:.2})",
                    probe.label,
                    pos.y,
                    probe.at
                );
            }
            (None, None) => {}
            (expected, actual) => panic!(
                "{}: expected a position of {expected:?} at t = {:.2}s, got {actual:?}",
                probe.label, probe.at
            ),
        }
    }
}
