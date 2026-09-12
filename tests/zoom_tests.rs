use fox_and_hounds::game::level::{BoardVariant, BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH};
use fox_and_hounds::game::state::Faction;
use fox_and_hounds::ui::camera::{ViewportCamera, MIN_ZOOM};
use macroquad::prelude::*;

#[test]
fn test_camera_center_on_faction() {
    let mut camera = ViewportCamera::new();
    let viewport = Rect::new(0.0, 50.0, 400.0, 600.0);
    let board_size = Vec2::new(300.0, 500.0);

    // When board fits in viewport
    camera.center_on_faction(Faction::Fox, viewport, board_size);
    assert_eq!(camera.pan_offset.x, (400.0 - 300.0) / 2.0);
    assert_eq!(camera.pan_offset.y, (600.0 - 500.0) / 2.0);

    // When board is larger than viewport (e.g. zoomed in)
    let large_board = Vec2::new(600.0, 1000.0);
    camera.center_on_faction(Faction::Fox, viewport, large_board);
    assert_eq!(camera.pan_offset.x, (400.0 - 600.0) / 2.0);
    assert_eq!(camera.pan_offset.y, 600.0 - 1000.0); // Fox at bottom

    camera.center_on_faction(Faction::Hounds, viewport, large_board);
    assert_eq!(camera.pan_offset.y, 0.0); // Hounds at top (flush with viewport)
}

#[test]
fn test_multiple_landscape_resolutions_gap_symmetry() {
    let resolutions: [(f32, f32); 5] = [
        (2560.0, 1440.0), // 1440p
        (1920.0, 1080.0), // 1080p
        (1600.0, 900.0),  // 900p
        (1366.0, 768.0),  // WXGA
        (1280.0, 720.0),  // 720p
    ];

    for (screen_w, screen_h) in resolutions {
        let base_scale: f32 = (screen_w / 850.0f32)
            .min(screen_h / 520.0f32)
            .clamp(0.65, 2.5);
        let viewport = Rect::new(0.0, 0.0, screen_w, screen_h);
        let board_scale = (viewport.h / BOARD_IMAGE_HEIGHT).max(0.1);
        let board_size = Vec2::new(
            BOARD_IMAGE_WIDTH * board_scale,
            BOARD_IMAGE_HEIGHT * board_scale,
        );

        let mut camera = ViewportCamera::new();
        camera.start_coop_fox_intro(viewport, board_size, board_scale, base_scale, 2.0);

        let anim = camera.anim.expect("Intro animation must be started");
        let target_scale = board_scale * anim.target_zoom;
        let coop_screen_y = anim.target_pan.y + 156.0 * target_scale;
        let fox_screen_y = anim.target_pan.y + 1052.0 * target_scale;

        assert!(
            coop_screen_y >= 0.0 && coop_screen_y <= viewport.h,
            "On {}x{}, Chicken Coop (y={}) must be in viewport (0..{})",
            screen_w,
            screen_h,
            coop_screen_y,
            viewport.h
        );
        assert!(
            fox_screen_y >= 0.0 && fox_screen_y <= viewport.h,
            "On {}x{}, Fox (y={}) must be in viewport (0..{})",
            screen_w,
            screen_h,
            fox_screen_y,
            viewport.h
        );
    }
}

#[test]
fn test_classic_board_pan_bounds_cannot_scroll_past_left_border() {
    use fox_and_hounds::game::level::CLASSIC_DIMENSIONS;
    use fox_and_hounds::ui::camera::horizontal_pan_bounds;

    // 1. Zoomed-in scenario in landscape (1920x1080 at 2.0x zoom)
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let board_scale = viewport.h / CLASSIC_DIMENSIONS.image_height;
    let zoom_2x_scale = board_scale * 2.0;
    let total_w = CLASSIC_DIMENSIONS.total_width() * zoom_2x_scale;

    let (min_x, max_x) = horizontal_pan_bounds(viewport.w, zoom_2x_scale, &CLASSIC_DIMENSIONS);

    assert_eq!(
        max_x, 0.0,
        "Classic board max_x must be 0.0 to prevent scrolling left past the image border"
    );

    let right_edge = min_x + total_w;
    assert!(
        (right_edge - viewport.w).abs() < 0.01,
        "Classic board right edge ({}) must reach viewport width ({}) at min_x",
        right_edge,
        viewport.w
    );

    // 2. Mobile portrait scenario (e.g. 1080x2400) where board is wider than screen even at 1.0x
    let portrait_vp = Rect::new(0.0, 0.0, 1080.0, 2400.0);
    let portrait_board_scale = portrait_vp.h / CLASSIC_DIMENSIONS.image_height;
    let portrait_total_w = CLASSIC_DIMENSIONS.total_width() * portrait_board_scale;

    let (p_min_x, p_max_x) =
        horizontal_pan_bounds(portrait_vp.w, portrait_board_scale, &CLASSIC_DIMENSIONS);
    assert_eq!(
        p_max_x, 0.0,
        "Classic board max_x on mobile portrait must be 0.0 to prevent scrolling left past the image border"
    );
    assert!(
        (p_min_x + portrait_total_w - portrait_vp.w).abs() < 0.01,
        "Classic board right edge must reach viewport width on mobile portrait at min_x"
    );
}

#[test]
fn test_all_variants_start_intro_animation() {
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    for variant in BoardVariant::all() {
        let mut camera = ViewportCamera::new();
        let dims = &variant.config().dimensions;
        let board_scale = viewport.h / dims.image_height;
        let board_size = Vec2::new(
            dims.image_width * board_scale,
            dims.image_height * board_scale,
        );

        camera.start_intro(*variant, viewport, board_size, board_scale, 2.0);

        assert_eq!(
            camera.zoom, MIN_ZOOM,
            "{:?} must start at MIN_ZOOM",
            variant
        );
        assert!(
            camera.target_zoom > MIN_ZOOM,
            "{:?} must target zoom > MIN_ZOOM",
            variant
        );
        assert!(
            camera.anim.is_some(),
            "{:?} must have intro animation active",
            variant
        );
        let anim = camera.anim.unwrap();
        assert_eq!(anim.start_zoom, MIN_ZOOM);
        assert_eq!(anim.target_zoom, camera.target_zoom);
        assert_eq!(anim.duration, 2.0);
    }
}
