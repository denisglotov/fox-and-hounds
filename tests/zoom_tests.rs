use fox_and_hounds::game::level::{BoardVariant, CLASSIC_DIMENSIONS};
use fox_and_hounds::game::state::Faction;
use fox_and_hounds::ui::camera::{
    horizontal_pan_bounds, vertical_pan_bounds, ViewportCamera, DEFAULT_ZOOM, MAX_ZOOM, MIN_ZOOM,
};
use macroquad::prelude::*;

#[test]
fn test_intro_framing_and_zoom_across_resolutions_and_variants() {
    assert_eq!(MIN_ZOOM, 0.75);
    assert_eq!(MAX_ZOOM, 3.0);

    // 1. Verify zoom range bounds across all variants
    for variant in BoardVariant::all() {
        let dims = &variant.config().dimensions;
        let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let board_scale = viewport.h / dims.image_height;

        for &zoom in &[MIN_ZOOM, 1.0, MAX_ZOOM] {
            let zoom_scale = board_scale * zoom;
            let (min_x, max_x) = horizontal_pan_bounds(viewport.w, zoom_scale, dims);
            assert!(min_x <= max_x, "{variant:?} min_x <= max_x at {zoom}x");

            let cur_board_h = dims.image_height * zoom_scale;
            let (min_y, max_y) = vertical_pan_bounds(viewport.h, cur_board_h);
            assert!(min_y <= max_y, "{variant:?} min_y <= max_y at {zoom}x");
        }

        // 2. Verify match intro animation initialization
        let mut camera = ViewportCamera::new();
        let board_size = Vec2::new(
            dims.image_width * board_scale,
            dims.image_height * board_scale,
        );
        camera.start_intro(*variant, viewport, board_size, board_scale, 2.0);

        assert_eq!(camera.zoom, DEFAULT_ZOOM);
        assert!(camera.target_zoom > DEFAULT_ZOOM);
        let anim = camera.anim.expect("Intro animation must be active");
        assert_eq!(anim.start_zoom, DEFAULT_ZOOM);
        assert_eq!(anim.target_zoom, camera.target_zoom);
        assert_eq!(anim.duration, 2.0);
    }

    // 3. Verify framing across multiple landscape resolutions
    let resolutions: [(f32, f32); 5] = [
        (2560.0, 1440.0),
        (1920.0, 1080.0),
        (1600.0, 900.0),
        (1366.0, 768.0),
        (1280.0, 720.0),
    ];
    let river_dims = &BoardVariant::RiverCrossing.config().dimensions;
    for (w, h) in resolutions {
        let base_scale = (w / 850.0).min(h / 520.0).clamp(0.65, 2.5);
        let vp = Rect::new(0.0, 0.0, w, h);
        let b_scale = (vp.h / river_dims.image_height).max(0.1);
        let b_size = Vec2::new(
            river_dims.image_width * b_scale,
            river_dims.image_height * b_scale,
        );

        let mut camera = ViewportCamera::new();
        camera.start_coop_fox_intro(vp, b_size, b_scale, base_scale, 2.0);

        let anim = camera.anim.expect("Intro anim must be active");
        let target_scale = b_scale * anim.target_zoom;
        let coop_y = anim.target_pan.y + 156.0 * target_scale;
        let fox_y = anim.target_pan.y + 1052.0 * target_scale;

        assert!((0.0..=vp.h).contains(&coop_y), "Coop visible on {w}x{h}");
        assert!((0.0..=vp.h).contains(&fox_y), "Fox visible on {w}x{h}");
    }
}

#[test]
fn test_pan_bounds_clamping_and_centering() {
    // 1. Centering when content fits
    let (c_min_y, c_max_y) = vertical_pan_bounds(1000.0, 800.0);
    assert_eq!(c_min_y, 100.0);
    assert_eq!(c_max_y, 100.0);

    // 2. Clamping when content is taller
    let (t_min_y, t_max_y) = vertical_pan_bounds(600.0, 1000.0);
    assert_eq!(t_min_y, -400.0);
    assert_eq!(t_max_y, 0.0);

    // 3. Faction centering
    let mut camera = ViewportCamera::new();
    let viewport = Rect::new(0.0, 50.0, 400.0, 600.0);
    let large_board = Vec2::new(600.0, 1000.0);
    camera.center_on_faction(Faction::Fox, viewport, large_board);
    assert_eq!(camera.pan_offset.y, 600.0 - 1000.0); // Fox at bottom
    camera.center_on_faction(Faction::Hounds, viewport, large_board);
    assert_eq!(camera.pan_offset.y, 0.0); // Hounds at top

    // 4. Classic zero-left-margin enforcement
    let vp = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let scale_2x = (vp.h / CLASSIC_DIMENSIONS.image_height) * 2.0;
    let (min_x, max_x) = horizontal_pan_bounds(vp.w, scale_2x, &CLASSIC_DIMENSIONS);
    assert_eq!(max_x, 0.0);
    assert!((min_x + CLASSIC_DIMENSIONS.total_width() * scale_2x - vp.w).abs() < 0.01);
}
