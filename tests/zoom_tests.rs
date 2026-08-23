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
    assert_eq!(camera.pan_offset.y, 12.0); // Hounds at top
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
