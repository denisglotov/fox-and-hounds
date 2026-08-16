use fox_and_hounds::game::state::Faction;
use fox_and_hounds::ui::camera::{
    ViewportCamera, DOUBLE_TAP_MAX_DISTANCE, DOUBLE_TAP_TIME_WINDOW, DOUBLE_TAP_ZOOM, MAX_ZOOM,
    MIN_ZOOM,
};
use macroquad::prelude::*;

#[test]
fn test_camera_initial_state() {
    let camera = ViewportCamera::new();
    assert_eq!(camera.zoom, MIN_ZOOM);
    assert_eq!(camera.target_zoom, MIN_ZOOM);
    assert_eq!(camera.pan_offset, Vec2::ZERO);
    assert!(!camera.is_dragging);
    assert!(!camera.initialized);
}

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
fn test_zoom_constants_and_ranges() {
    assert_eq!(MIN_ZOOM, 1.0);
    assert_eq!(MAX_ZOOM, 2.5);
    assert_eq!(DOUBLE_TAP_ZOOM, 2.0);
    assert!(DOUBLE_TAP_TIME_WINDOW >= 0.25 && DOUBLE_TAP_TIME_WINDOW <= 0.40);
    assert!(DOUBLE_TAP_MAX_DISTANCE >= 20.0);
}
