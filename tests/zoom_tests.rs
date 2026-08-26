use fox_and_hounds::game::state::Faction;
use fox_and_hounds::ui::camera::{ViewportCamera, MIN_ZOOM};
use macroquad::prelude::*;

#[test]
fn test_camera_reset() {
    let mut camera = ViewportCamera::new();
    camera.zoom = 2.0;
    camera.target_zoom = 2.5;
    camera.pan_offset = Vec2::new(100.0, 50.0);
    camera.is_dragging = true;
    camera.initialized = true;

    camera.reset_pan();

    assert_eq!(camera.zoom, MIN_ZOOM);
    assert_eq!(camera.target_zoom, MIN_ZOOM);
    assert_eq!(camera.pan_offset, Vec2::ZERO);
    assert!(!camera.is_dragging);
    assert!(!camera.initialized);
}

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
fn test_coop_fox_intro_initialization() {
    let mut camera = ViewportCamera::new();
    let viewport = Rect::new(0.0, 50.0, 960.0, 1300.0);
    let board_scale = 0.94;
    let board_size = Vec2::new(768.0 * board_scale, 1376.0 * board_scale);
    let scale = 1.0;

    camera.start_coop_fox_intro(viewport, board_size, board_scale, scale, 2.0);

    assert_eq!(camera.zoom, MIN_ZOOM);
    assert!(camera.target_zoom > 1.20 && camera.target_zoom <= 1.85);
    assert!(camera.anim.is_some());

    let anim = camera.anim.unwrap();
    assert_eq!(anim.start_zoom, MIN_ZOOM);
    assert_eq!(anim.target_zoom, camera.target_zoom);
    assert_eq!(anim.duration, 2.0);
    assert_eq!(anim.elapsed, 0.0);

    // Reset should cancel the animation
    camera.reset_pan();
    assert!(camera.anim.is_none());
    assert_eq!(camera.zoom, MIN_ZOOM);
}

#[test]
fn test_1920x1080_landscape_framing_and_gap_symmetry() {
    use fox_and_hounds::game::level::{
        BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH, BOARD_LEFT_WIDTH, BOARD_RIGHT_WIDTH,
    };
    use fox_and_hounds::ui::camera::horizontal_pan_bounds;

    let screen_w: f32 = 1920.0;
    let screen_h: f32 = 1080.0;
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

    // 1. Initial framing at start_zoom (1.0x) is centered horizontally
    let start_scale = board_scale * anim.start_zoom;
    let start_left_edge = anim.start_pan.x - BOARD_LEFT_WIDTH * start_scale;
    let start_right_edge = anim.start_pan.x + (BOARD_IMAGE_WIDTH + BOARD_RIGHT_WIDTH) * start_scale;
    let start_left_gap = start_left_edge;
    let start_right_gap = viewport.w - start_right_edge;

    assert!(
        (start_left_gap - start_right_gap).abs() < 0.01,
        "Start pan left gap ({}) must equal right gap ({}) on 1920x1080",
        start_left_gap,
        start_right_gap
    );

    // 2. Target framing at target_zoom: Both Chicken Coop (y=156) and Fox (y=1052) MUST be in view
    let target_scale = board_scale * anim.target_zoom;
    let coop_screen_y = anim.target_pan.y + 156.0 * target_scale;
    let fox_screen_y = anim.target_pan.y + 1052.0 * target_scale;

    assert!(
        coop_screen_y >= 0.0 && coop_screen_y <= viewport.h,
        "Chicken Coop (y={}) must be in viewport (0..{})",
        coop_screen_y,
        viewport.h
    );
    assert!(
        fox_screen_y >= 0.0 && fox_screen_y <= viewport.h,
        "Fox (y={}) must be in viewport (0..{})",
        fox_screen_y,
        viewport.h
    );

    // 3. Horizontal pan bounds when zoomed in (2.0x) must allow full left-to-right coverage
    let zoom_2x_scale = board_scale * 2.0;
    let (min_x, max_x) = horizontal_pan_bounds(viewport.w, zoom_2x_scale);

    // At min_x (panned all the way to right), right edge of artwork must align with viewport right
    let rightmost_pan_right_edge = min_x + (BOARD_IMAGE_WIDTH + BOARD_RIGHT_WIDTH) * zoom_2x_scale;
    assert!(
        (rightmost_pan_right_edge - viewport.w).abs() < 0.01,
        "At min_x, artwork right edge ({}) must reach viewport width ({})",
        rightmost_pan_right_edge,
        viewport.w
    );

    // At max_x (panned all the way to left), left edge of artwork must align with viewport left (0.0)
    let leftmost_pan_left_edge = max_x - BOARD_LEFT_WIDTH * zoom_2x_scale;
    assert!(
        leftmost_pan_left_edge.abs() < 0.01,
        "At max_x, artwork left edge ({}) must reach viewport left (0.0)",
        leftmost_pan_left_edge
    );
}

#[test]
fn test_multiple_landscape_resolutions_gap_symmetry() {
    use fox_and_hounds::game::level::{BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH};

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
