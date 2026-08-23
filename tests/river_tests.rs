use fox_and_hounds::game::level::{BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH};
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

    // Verify samples along the full arc length
    let num_checks = 100;
    for i in 0..=num_checks {
        let dist = (i as f32 / num_checks as f32) * path.total_length;
        for &v in &[-1.0, -0.5, 0.0, 0.5, 1.0] {
            let (pos, tangent, normal, half_width) = path.sample_at(dist, v);

            // Bounds check across entire left extension, board, and right extension
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

            // Vectors & widths check
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
fn test_river_entrance_and_exit_coordinates() {
    let path = RiverPath::new();

    // Entrance at s = 0 (Leftmost extension boundary x = -384)
    let (start_pos, start_tangent, _, _) = path.sample_at(0.0, 0.0);
    assert!(
        (start_pos.x - (-BOARD_LEFT_WIDTH)).abs() < 2.0,
        "River must start at left extension edge x=-384, got {}",
        start_pos.x
    );
    assert!(
        start_pos.y > 700.0 && start_pos.y < 740.0,
        "River start y must be at western entrance (~721), got {}",
        start_pos.y
    );
    assert!(
        start_tangent.x > 0.5,
        "River flow must head east/northeast at entrance"
    );

    // Exit at s = total_length (Rightmost extension boundary x = 1024)
    let (end_pos, end_tangent, _, _) = path.sample_at(path.total_length, 0.0);
    let expected_exit_x = BOARD_IMAGE_WIDTH + BOARD_RIGHT_WIDTH;
    assert!(
        (end_pos.x - expected_exit_x).abs() < 2.0,
        "River must exit at right extension edge x={}, got {}",
        expected_exit_x,
        end_pos.x
    );
    assert!(
        end_pos.y > 640.0 && end_pos.y < 680.0,
        "River exit y must be at eastern outflow (~658), got {}",
        end_pos.y
    );
    assert!(end_tangent.x > 0.5, "River flow must head east at exit");
}

#[test]
fn test_bridge_occlusion_detection() {
    let path = RiverPath::new();

    // 1. Under railway bridge (x ≈ 45, y ≈ 848)
    let rail_occlusion = path.bridge_occlusion(Vec2::new(45.0, 848.0));
    assert!(
        rail_occlusion > 0.5,
        "Railway bridge center should have high occlusion, got {}",
        rail_occlusion
    );

    // 2. Under M6 wooden bridge (x ≈ 384, y ≈ 755)
    let wood_occlusion = path.bridge_occlusion(Vec2::new(384.0, 755.0));
    assert!(
        wood_occlusion > 0.5,
        "M6 wooden bridge center should have high occlusion, got {}",
        wood_occlusion
    );

    // 3. Open water (not under any bridge)
    let open_water1 = path.bridge_occlusion(Vec2::new(220.0, 780.0));
    assert_eq!(
        open_water1, 0.0,
        "Open water between bridges should have zero occlusion"
    );

    let open_water2 = path.bridge_occlusion(Vec2::new(600.0, 702.0));
    assert_eq!(
        open_water2, 0.0,
        "Open water downstream should have zero occlusion"
    );
}
