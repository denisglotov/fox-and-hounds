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

#[test]
fn test_river_sample_at_bounds_clamping() {
    let path = RiverPath::new();

    // Negative distance should clamp to 0.0
    let (neg_pos, neg_tangent, _, _) = path.sample_at(-50.0, 0.0);
    let (zero_pos, zero_tangent, _, _) = path.sample_at(0.0, 0.0);
    assert_eq!(neg_pos, zero_pos);
    assert_eq!(neg_tangent, zero_tangent);

    // Distance exceeding total_length should clamp to total_length
    let (overshoot_pos, overshoot_tangent, _, _) = path.sample_at(path.total_length + 200.0, 0.0);
    let (end_pos, end_tangent, _, _) = path.sample_at(path.total_length, 0.0);
    assert_eq!(overshoot_pos, end_pos);
    assert_eq!(overshoot_tangent, end_tangent);
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

            // Bounds check across 1024x1024 board dimensions (with minor margin for half_width)
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

            // Vector normalization & orthogonality
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
fn test_classic_river_entrance_and_exit_coordinates() {
    let path = RiverPath::classic();

    // Entrance at s = 0 (Top edge of 1024x1024 board, x ≈ 185, y = 0)
    let (start_pos, start_tangent, _, _) = path.sample_at(0.0, 0.0);
    assert!(
        (start_pos.x - 185.0).abs() < 2.0,
        "River entrance x should be ~185.0, got {}",
        start_pos.x
    );
    assert!(
        start_pos.y.abs() < 2.0,
        "River entrance y should be at top boundary y=0.0, got {}",
        start_pos.y
    );
    assert!(
        start_tangent.y > 0.5,
        "River flow must head south into board at entrance"
    );

    // Exit at s = total_length (Bottom edge of 1024x1024 board, x ≈ 782, y = 1024)
    let (end_pos, end_tangent, _, _) = path.sample_at(path.total_length, 0.0);
    assert!(
        (end_pos.x - 782.0).abs() < 2.0,
        "River exit x should be ~782.0, got {}",
        end_pos.x
    );
    assert!(
        (end_pos.y - 1024.0).abs() < 2.0,
        "River exit y should be at bottom boundary y=1024.0, got {}",
        end_pos.y
    );
    assert!(
        end_tangent.y > 0.5,
        "River flow must head south out of board at exit"
    );
}

#[test]
fn test_classic_all_8_bridges_occlusion() {
    let path = RiverPath::classic();

    // All 8 bridges must have high occlusion at their centers
    let bridges = [
        ("Bridge 1 (M0-T1)", Vec2::new(314.0, 411.0)),
        ("Bridge 2 (T1-M1)", Vec2::new(375.5, 419.5)),
        ("Bridge 3 (T1-M2)", Vec2::new(443.0, 419.5)),
        ("Bridge 4 (T2-M2)", Vec2::new(510.5, 414.0)),
        ("Bridge 5 (M2-T3)", Vec2::new(566.0, 442.0)),
        ("Bridge 6 (M2-M3)", Vec2::new(577.5, 510.0)),
        ("Bridge 7 (M3-B3)", Vec2::new(645.5, 600.0)),
        ("Bridge 8 (B3-M4)", Vec2::new(712.0, 602.0)),
    ];

    for (name, center) in bridges {
        let occ = path.bridge_occlusion(center);
        assert!(
            occ > 0.8,
            "{} center {:?} must have high occlusion, got {}",
            name,
            center,
            occ
        );
    }

    // Open water points must have zero occlusion
    let open_water_points = [
        ("Northwestern forest", Vec2::new(228.0, 120.0)),
        ("Between B1 and B2", Vec2::new(345.0, 418.0)),
        ("Between B2 and B3", Vec2::new(410.0, 420.0)),
        ("Between B3 and B4", Vec2::new(478.0, 416.0)),
        ("Channel between B6 and B7", Vec2::new(602.0, 555.0)),
        ("Between B7 and B8", Vec2::new(676.0, 592.0)),
        ("Channel after Bridge 8", Vec2::new(725.0, 670.0)),
        ("Southeastern forest", Vec2::new(748.0, 905.0)),
    ];

    for (name, pos) in open_water_points {
        let occ = path.bridge_occlusion(pos);
        assert_eq!(
            occ, 0.0,
            "Open water at {} {:?} must have 0.0 occlusion, got {}",
            name, pos, occ
        );
    }
}

#[test]
fn test_river_path_for_variant() {
    let classic = RiverPath::for_variant(BoardVariant::Classic);
    assert_eq!(classic.variant, BoardVariant::Classic);

    let river_crossing = RiverPath::for_variant(BoardVariant::RiverCrossing);
    assert_eq!(river_crossing.variant, BoardVariant::RiverCrossing);
}
