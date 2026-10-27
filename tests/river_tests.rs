use fox_and_hounds::game::level::BoardVariant;
use fox_and_hounds::ui::river::{RiverPath, RiverSimulation};
use macroquad::prelude::Vec2;
use std::ops::RangeInclusive;

fn assert_river_path_continuity(
    path: &RiverPath,
    num_checks: usize,
    x_bounds: RangeInclusive<f32>,
    y_bounds: RangeInclusive<f32>,
    half_width_bounds: RangeInclusive<f32>,
) {
    for i in 0..=num_checks {
        let dist = (i as f32 / num_checks as f32) * path.total_length;
        for &v in &[-1.0, -0.5, 0.0, 0.5, 1.0] {
            let (pos, tangent, normal, half_width) = path.sample_at(dist, v);

            assert!(
                x_bounds.contains(&pos.x),
                "River pos.x {} out of bounds ({:?}) at dist {}, v {}",
                pos.x,
                x_bounds,
                dist,
                v
            );
            assert!(
                y_bounds.contains(&pos.y),
                "River pos.y {} out of bounds ({:?}) at dist {}, v {}",
                pos.y,
                y_bounds,
                dist,
                v
            );

            assert!(
                (tangent.length() - 1.0).abs() < 1e-3,
                "Tangent should be normalized at dist {}",
                dist
            );
            assert!(
                (normal.length() - 1.0).abs() < 1e-3,
                "Normal should be normalized at dist {}",
                dist
            );
            assert!(
                tangent.dot(normal).abs() < 1e-3,
                "Normal must be orthogonal to tangent at dist {}",
                dist
            );
            assert!(
                half_width_bounds.contains(&half_width),
                "Half width {} out of bounds ({:?}) at dist {}",
                half_width,
                half_width_bounds,
                dist
            );
        }
    }
}

#[test]
fn test_all_variant_river_paths_continuity_and_bounds() {
    struct VariantPathExpectation {
        variant: BoardVariant,
        length_range: RangeInclusive<f32>,
        x_bounds: RangeInclusive<f32>,
        y_bounds: RangeInclusive<f32>,
        width_bounds: RangeInclusive<f32>,
    }

    let expectations = [
        VariantPathExpectation {
            variant: BoardVariant::RiverCrossing,
            length_range: 1300.0..=1800.0,
            x_bounds: -434.0..=1074.0,
            y_bounds: 500.0..=1376.0,
            width_bounds: 18.0..=35.0,
        },
        VariantPathExpectation {
            variant: BoardVariant::Classic,
            length_range: 1300.0..=1600.0,
            x_bounds: -30.0..=1054.0,
            y_bounds: -30.0..=1054.0,
            width_bounds: 12.0..=26.0,
        },
        VariantPathExpectation {
            variant: BoardVariant::FoxAndDogs,
            length_range: 900.0..=1300.0,
            x_bounds: -50.0..=1074.0,
            y_bounds: 300.0..=600.0,
            width_bounds: 30.0..=50.0,
        },
        VariantPathExpectation {
            variant: BoardVariant::FoxAndDogsMaze,
            length_range: 900.0..=1300.0,
            x_bounds: -50.0..=1074.0,
            y_bounds: 300.0..=600.0,
            width_bounds: 20.0..=35.0,
        },
        VariantPathExpectation {
            variant: BoardVariant::TheRedHunt,
            length_range: 900.0..=1400.0,
            x_bounds: -50.0..=1074.0,
            y_bounds: 550.0..=750.0,
            width_bounds: 25.0..=38.0,
        },
    ];

    for exp in expectations {
        let path = RiverPath::for_variant(exp.variant);
        assert!(
            exp.length_range.contains(&path.total_length),
            "{:?} river length {} not in {:?}",
            exp.variant,
            path.total_length,
            exp.length_range
        );
        assert_river_path_continuity(&path, 80, exp.x_bounds, exp.y_bounds, exp.width_bounds);
    }
}

#[test]
fn test_river_simulation_reuses_geometry_for_the_same_variant() {
    let mut river = RiverSimulation::for_variant(BoardVariant::Classic);

    assert!(!river.set_variant(BoardVariant::Classic));
    assert_eq!(river.path.variant, BoardVariant::Classic);
    assert!(river.set_variant(BoardVariant::RiverCrossing));
    assert_eq!(river.path.variant, BoardVariant::RiverCrossing);
}

#[test]
fn test_bridge_occlusion_across_variants() {
    let cases = [
        // River crossing rail bridge, wooden bridge, open water
        (
            BoardVariant::RiverCrossing,
            Vec2::new(45.0, 848.0),
            0.5,
            true,
        ),
        (
            BoardVariant::RiverCrossing,
            Vec2::new(384.0, 755.0),
            0.5,
            true,
        ),
        (
            BoardVariant::RiverCrossing,
            Vec2::new(220.0, 780.0),
            0.0,
            false,
        ),
        // Fox and Dogs bridge deck & open water
        (BoardVariant::FoxAndDogs, Vec2::new(508.0, 410.0), 0.8, true),
        (
            BoardVariant::FoxAndDogs,
            Vec2::new(200.0, 400.0),
            0.0,
            false,
        ),
        // The Red Hunt Perekop bridge & open chasm
        (BoardVariant::TheRedHunt, Vec2::new(512.0, 655.0), 0.8, true),
        (
            BoardVariant::TheRedHunt,
            Vec2::new(200.0, 650.0),
            0.0,
            false,
        ),
        // Fox and Dogs Maze bridge deck & open water
        (
            BoardVariant::FoxAndDogsMaze,
            Vec2::new(508.0, 410.0),
            0.8,
            true,
        ),
        (
            BoardVariant::FoxAndDogsMaze,
            Vec2::new(200.0, 400.0),
            0.0,
            false,
        ),
    ];

    for (variant, pos, threshold, is_occluded) in cases {
        let path = RiverPath::for_variant(variant);
        let occlusion = path.bridge_occlusion(pos);
        if is_occluded {
            assert!(
                occlusion > threshold,
                "{:?} bridge deck at {:?} should have occlusion > {}, got {}",
                variant,
                pos,
                threshold,
                occlusion
            );
        } else {
            assert_eq!(
                occlusion, 0.0,
                "{:?} open area at {:?} should have zero occlusion, got {}",
                variant, pos, occlusion
            );
        }
    }
}

#[test]
fn test_precomputed_river_segments_non_empty_and_valid() {
    for &variant in BoardVariant::all() {
        let path = RiverPath::for_variant(variant);
        assert!(
            !path.ambient_segments.is_empty(),
            "{:?} ambient segments empty",
            variant
        );
        assert!(
            !path.streamline_segments.is_empty(),
            "{:?} streamline segments empty",
            variant
        );
        assert!(
            !path.caustic_segments.is_empty(),
            "{:?} caustic segments empty",
            variant
        );

        if variant == BoardVariant::TheRedHunt {
            assert!(
                !path.lava_ambient_segments.is_empty(),
                "TheRedHunt lava ambient empty"
            );
        } else {
            assert!(path.lava_ambient_segments.is_empty());
        }

        // Verify segment distance monotonicity and finite coordinates
        for seg in &path.ambient_segments {
            assert!(seg.p0.x.is_finite() && seg.p0.y.is_finite());
            assert!(seg.p1.x.is_finite() && seg.p1.y.is_finite());
            assert!(seg.avg_width > 0.0);
            assert!(seg.occlusion >= 0.0 && seg.occlusion <= 1.0);
        }
    }
}
