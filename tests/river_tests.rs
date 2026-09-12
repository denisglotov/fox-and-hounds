use fox_and_hounds::game::level::{
    BoardVariant, BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH, CLASSIC_DIMENSIONS,
};
use fox_and_hounds::ui::board_view::{BOARD_LEFT_WIDTH, BOARD_RIGHT_WIDTH};
use fox_and_hounds::ui::river::RiverPath;
use macroquad::prelude::Vec2;

#[test]
fn test_river_path_continuity_and_board_bounds() {
    let path = RiverPath::new();
    assert!(
        path.total_length > 1300.0 && path.total_length < 1800.0,
        "River total length {} is out of expected span",
        path.total_length
    );

    let num_checks = 100;
    for i in 0..=num_checks {
        let dist = (i as f32 / num_checks as f32) * path.total_length;
        for &v in &[-1.0, -0.5, 0.0, 0.5, 1.0] {
            let (pos, tangent, normal, half_width) = path.sample_at(dist, v);

            assert!(
                pos.x >= -BOARD_LEFT_WIDTH - 50.0
                    && pos.x <= BOARD_IMAGE_WIDTH + BOARD_RIGHT_WIDTH + 50.0,
                "River pos.x {} out of background bounds at dist {}, v {}",
                pos.x,
                dist,
                v
            );
            assert!(
                pos.y >= 500.0 && pos.y <= BOARD_IMAGE_HEIGHT,
                "River pos.y {} out of river corridor at dist {}, v {}",
                pos.y,
                dist,
                v
            );

            assert!(
                (tangent.length() - 1.0).abs() < 1e-3,
                "Tangent should be normalized"
            );
            assert!(
                (normal.length() - 1.0).abs() < 1e-3,
                "Normal should be normalized"
            );
            assert!(
                tangent.dot(normal).abs() < 1e-3,
                "Normal must be orthogonal to tangent"
            );
            assert!(
                (18.0..=35.0).contains(&half_width),
                "Half width {} out of bounds",
                half_width
            );
        }
    }
}

#[test]
fn test_classic_river_path_continuity_and_bounds() {
    let path = RiverPath::classic();
    assert_eq!(path.variant, BoardVariant::Classic);
    assert!(
        path.total_length > 1300.0 && path.total_length < 1600.0,
        "Classic river total length {} is out of expected span",
        path.total_length
    );

    let num_checks = 120;
    for i in 0..=num_checks {
        let dist = (i as f32 / num_checks as f32) * path.total_length;
        for &v in &[-1.0, -0.5, 0.0, 0.5, 1.0] {
            let (pos, tangent, normal, half_width) = path.sample_at(dist, v);

            assert!(
                pos.x >= -30.0 && pos.x <= CLASSIC_DIMENSIONS.image_width + 30.0,
                "Classic river pos.x {} out of bounds at dist {}, v {}",
                pos.x,
                dist,
                v
            );
            assert!(
                pos.y >= -30.0 && pos.y <= CLASSIC_DIMENSIONS.image_height + 30.0,
                "Classic river pos.y {} out of bounds at dist {}, v {}",
                pos.y,
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
                (12.0..=26.0).contains(&half_width),
                "Half width {} out of bounds at dist {}",
                half_width,
                dist
            );
        }
    }
}

#[test]
fn test_bridge_occlusion_detection() {
    let path = RiverPath::new();

    // Railway bridge center
    let rail_occlusion = path.bridge_occlusion(Vec2::new(45.0, 848.0));
    assert!(
        rail_occlusion > 0.5,
        "Railway bridge center should have high occlusion"
    );

    // M6 wooden bridge center
    let wood_occlusion = path.bridge_occlusion(Vec2::new(384.0, 755.0));
    assert!(
        wood_occlusion > 0.5,
        "M6 wooden bridge center should have high occlusion"
    );

    // Open water
    assert_eq!(
        path.bridge_occlusion(Vec2::new(220.0, 780.0)),
        0.0,
        "Open water between bridges should have zero occlusion"
    );
}

#[test]
fn test_variant_river_paths_and_occlusion() {
    // Fox and Dogs (Arthur)
    let arthur_path = RiverPath::for_variant(BoardVariant::FoxAndDogs);
    assert!(arthur_path.total_length > 900.0 && arthur_path.total_length < 1300.0);
    assert!(
        arthur_path.bridge_occlusion(Vec2::new(508.0, 410.0)) > 0.8,
        "Arthur bridge deck must have high occlusion"
    );
    assert_eq!(
        arthur_path.bridge_occlusion(Vec2::new(200.0, 400.0)),
        0.0,
        "Arthur open water must have zero occlusion"
    );

    // The Red Hunt
    let red_path = RiverPath::for_variant(BoardVariant::TheRedHunt);
    assert!(red_path.total_length > 900.0 && red_path.total_length < 1400.0);
    assert!(
        red_path.bridge_occlusion(Vec2::new(512.0, 655.0)) > 0.8,
        "Perekop bridge deck must have high occlusion"
    );
    assert_eq!(
        red_path.bridge_occlusion(Vec2::new(200.0, 650.0)),
        0.0,
        "Open fault chasm must have zero occlusion"
    );
}
