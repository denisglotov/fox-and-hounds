use macroquad::input::TouchPhase;
use macroquad::prelude::*;

pub const MIN_ZOOM: f32 = 1.0;
pub const MAX_ZOOM: f32 = 2.5;
pub const DOUBLE_TAP_ZOOM: f32 = 2.0;
pub const DOUBLE_TAP_TIME_WINDOW: f64 = 0.30;
pub const DOUBLE_TAP_MAX_DISTANCE: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraAnimation {
    pub start_zoom: f32,
    pub target_zoom: f32,
    pub start_pan: Vec2,
    pub target_pan: Vec2,
    pub duration: f32,
    pub elapsed: f32,
}

pub struct ViewportCamera {
    pub pan_offset: Vec2,
    pub zoom: f32,
    pub target_zoom: f32,
    pub zoom_focal_point: Option<Vec2>,
    pub drag_start: Option<Vec2>,
    pub is_dragging: bool,
    pub last_tap_time: f64,
    pub last_tap_pos: Vec2,
    pub pinch_prev_dist: Option<f32>,
    pub render_target: Option<RenderTarget>,
    pub initialized: bool,
    pub anim: Option<CameraAnimation>,
}

impl Default for ViewportCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewportCamera {
    pub fn new() -> Self {
        Self {
            pan_offset: Vec2::ZERO,
            zoom: MIN_ZOOM,
            target_zoom: MIN_ZOOM,
            zoom_focal_point: None,
            drag_start: None,
            is_dragging: false,
            last_tap_time: 0.0,
            last_tap_pos: Vec2::ZERO,
            pinch_prev_dist: None,
            render_target: None,
            initialized: false,
            anim: None,
        }
    }

    pub fn reset_pan(&mut self) {
        self.pan_offset = Vec2::ZERO;
        self.zoom = MIN_ZOOM;
        self.target_zoom = MIN_ZOOM;
        self.zoom_focal_point = None;
        self.drag_start = None;
        self.is_dragging = false;
        self.last_tap_time = 0.0;
        self.last_tap_pos = Vec2::ZERO;
        self.pinch_prev_dist = None;
        self.initialized = false;
        self.anim = None;
    }

    /// Starts a cinematic slow zooming animation into the Coop-to-Fox playing field at match start.
    pub fn start_coop_fox_intro(
        &mut self,
        viewport_rect: Rect,
        base_board_size: Vec2,
        base_board_scale: f32,
        scale: f32,
        duration: f32,
    ) {
        let pad = 12.0 * scale;

        // 1. Initial wide overview framing at MIN_ZOOM (1.0x)
        let start_zoom = MIN_ZOOM;
        let start_board_size = base_board_size * start_zoom;
        let start_pan_x = (viewport_rect.w - start_board_size.x) / 2.0;
        let start_pan_y = if (start_board_size.y + pad * 2.0) <= viewport_rect.h {
            (viewport_rect.h - start_board_size.y) / 2.0
        } else {
            pad
        };
        let start_pan = Vec2::new(start_pan_x, start_pan_y);

        // 2. Target "Coop-to-Fox" framing
        // Chicken Coop is at y ≈ 80..156, Fox Den is at y ≈ 1052..1120.
        // The playable vertical span is ~1040px out of total BOARD_IMAGE_HEIGHT (1376px).
        const COOP_FOX_PLAYABLE_HEIGHT: f32 = 1040.0;
        const COOP_FOX_CENTER_Y: f32 = 600.0;
        const COOP_FOX_CENTER_X: f32 = crate::game::level::BOARD_IMAGE_WIDTH / 2.0; // 384.0

        let target_zoom = ((viewport_rect.h - pad * 2.0)
            / (COOP_FOX_PLAYABLE_HEIGHT * base_board_scale))
            .clamp(1.25, 1.85);

        let target_board_size = base_board_size * target_zoom;
        let mut target_pan_x =
            viewport_rect.w / 2.0 - (COOP_FOX_CENTER_X * base_board_scale * target_zoom);
        let mut target_pan_y =
            viewport_rect.h / 2.0 - (COOP_FOX_CENTER_Y * base_board_scale * target_zoom);

        // Boundary clamp target pan
        let total_target_w = target_board_size.x + pad * 2.0;
        let total_target_h = target_board_size.y + pad * 2.0;

        if total_target_w <= viewport_rect.w {
            target_pan_x = (viewport_rect.w - target_board_size.x) / 2.0;
        } else {
            let min_x = viewport_rect.w - target_board_size.x - pad;
            let max_x = pad;
            target_pan_x = target_pan_x.clamp(min_x, max_x);
        }

        if total_target_h <= viewport_rect.h {
            target_pan_y = (viewport_rect.h - target_board_size.y) / 2.0;
        } else {
            let min_y = viewport_rect.h - target_board_size.y - pad;
            let max_y = pad;
            target_pan_y = target_pan_y.clamp(min_y, max_y);
        }

        let target_pan = Vec2::new(target_pan_x, target_pan_y);

        self.zoom = start_zoom;
        self.target_zoom = target_zoom;
        self.pan_offset = start_pan;
        self.drag_start = None;
        self.is_dragging = false;
        self.initialized = true;

        self.anim = Some(CameraAnimation {
            start_zoom,
            target_zoom,
            start_pan,
            target_pan,
            duration: duration.max(0.1),
            elapsed: 0.0,
        });
    }

    /// Centers camera on player's pieces at match start (bottom for Fox, top for Hounds).
    pub fn center_on_faction(
        &mut self,
        faction: crate::game::state::Faction,
        viewport_rect: Rect,
        base_board_size: Vec2,
    ) {
        let cur_board_size = base_board_size * self.zoom;
        self.pan_offset.x = (viewport_rect.w - cur_board_size.x) / 2.0;
        if cur_board_size.y <= viewport_rect.h {
            self.pan_offset.y = (viewport_rect.h - cur_board_size.y) / 2.0;
        } else {
            self.pan_offset.y = match faction {
                crate::game::state::Faction::Fox => (viewport_rect.h - cur_board_size.y).min(0.0),
                crate::game::state::Faction::Hounds => 12.0,
            };
        }
        self.drag_start = None;
        self.is_dragging = false;
        self.initialized = true;
    }

    /// Handles pinch-to-zoom, double-tap zoom, mouse drag/pan, and wheel zoom/scroll.
    /// Returns (render_target, pan_offset, effective_scale, was_dragging).
    pub fn update_and_begin(
        &mut self,
        viewport_rect: Rect,
        base_board_size: Vec2,
        base_board_scale: f32,
        scale: f32,
        dt: f32,
    ) -> (RenderTarget, Vec2, f32, bool) {
        let mouse_pos = Vec2::from(mouse_position());
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_released = is_mouse_button_released(MouseButton::Left);
        let (wheel_x, wheel_y) = mouse_wheel();
        let in_viewport = viewport_rect.contains(mouse_pos);

        // 1. Multi-Touch Pinch-to-Zoom
        let active_touches: Vec<_> = touches()
            .into_iter()
            .filter(|t| t.phase != TouchPhase::Cancelled && t.phase != TouchPhase::Ended)
            .collect();

        let mut is_pinching = false;
        if active_touches.len() >= 2 {
            let p0 = active_touches[0].position;
            let p1 = active_touches[1].position;
            let current_dist = (p0 - p1).length();
            let focal_screen = (p0 + p1) * 0.5;
            let focal_vp = focal_screen - Vec2::new(viewport_rect.x, viewport_rect.y);

            if let Some(prev_dist) = self.pinch_prev_dist {
                if prev_dist > 1.0 && current_dist > 1.0 {
                    let ratio = current_dist / prev_dist;
                    let old_zoom = self.zoom;
                    let new_zoom = (self.zoom * ratio).clamp(MIN_ZOOM, MAX_ZOOM);
                    if old_zoom > 0.0 {
                        let actual_ratio = new_zoom / old_zoom;
                        self.pan_offset = focal_vp - (focal_vp - self.pan_offset) * actual_ratio;
                        self.zoom = new_zoom;
                        self.target_zoom = new_zoom;
                    }
                }
            }
            self.pinch_prev_dist = Some(current_dist);
            self.is_dragging = true;
            is_pinching = true;
            self.drag_start = None;
        } else {
            self.pinch_prev_dist = None;
        }

        // 2. Double-Tap Zoom Detection (when tapping on screen/touch)
        let mut double_tap_triggered = false;
        if mouse_released && in_viewport && !self.is_dragging && !is_pinching {
            let now = get_time();
            let dt_tap = now - self.last_tap_time;
            let dist_tap = (mouse_pos - self.last_tap_pos).length();

            if dt_tap < DOUBLE_TAP_TIME_WINDOW && dist_tap < DOUBLE_TAP_MAX_DISTANCE * scale {
                // Toggle between 1.0x and 2.0x
                let focal_vp = mouse_pos - Vec2::new(viewport_rect.x, viewport_rect.y);
                self.zoom_focal_point = Some(focal_vp);
                if self.target_zoom > 1.2 {
                    self.target_zoom = MIN_ZOOM;
                } else {
                    self.target_zoom = DOUBLE_TAP_ZOOM;
                }
                self.last_tap_time = 0.0;
                double_tap_triggered = true;
            } else {
                self.last_tap_time = now;
                self.last_tap_pos = mouse_pos;
            }
        }

        // 3. Mouse Wheel / Trackpad Pinch / Zoom
        let is_ctrl = is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper);

        if is_ctrl && wheel_y != 0.0 && in_viewport {
            self.anim = None;
            let old_zoom = self.zoom;
            let zoom_delta = wheel_y * 0.15;
            self.target_zoom = (self.zoom + zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
            self.zoom = self.target_zoom;
            if old_zoom > 0.0 {
                let zoom_ratio = self.zoom / old_zoom;
                let focal_vp = mouse_pos - Vec2::new(viewport_rect.x, viewport_rect.y);
                self.pan_offset = focal_vp - (focal_vp - self.pan_offset) * zoom_ratio;
            }
        } else if wheel_y != 0.0 || wheel_x != 0.0 {
            self.anim = None;
            let scroll_speed = 35.0 * scale;
            self.pan_offset.y += wheel_y * scroll_speed;
            self.pan_offset.x -= wheel_x * scroll_speed;
        }

        if is_pinching || double_tap_triggered {
            self.anim = None;
        }

        // 4. Smooth Zoom / Camera Intro Animation
        if let Some(mut anim) = self.anim {
            anim.elapsed += dt;
            let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);
            // Smooth cubic ease-out deceleration
            let ease = 1.0 - (1.0 - t).powi(3);

            self.zoom = anim.start_zoom + (anim.target_zoom - anim.start_zoom) * ease;
            self.target_zoom = anim.target_zoom;
            self.pan_offset = anim.start_pan.lerp(anim.target_pan, ease);

            if t >= 1.0 {
                self.anim = None;
            } else {
                self.anim = Some(anim);
            }
        } else if (self.zoom - self.target_zoom).abs() > 0.001 {
            let old_zoom = self.zoom;
            let lerp_rate = 14.0;
            let lerp_factor = (1.0 - (-lerp_rate * dt).exp()).clamp(0.0, 1.0);
            self.zoom += (self.target_zoom - self.zoom) * lerp_factor;
            if (self.zoom - self.target_zoom).abs() < 0.002 {
                self.zoom = self.target_zoom;
            }
            if old_zoom > 0.0 {
                let zoom_ratio = self.zoom / old_zoom;
                let focal_vp = self
                    .zoom_focal_point
                    .unwrap_or(Vec2::new(viewport_rect.w / 2.0, viewport_rect.h / 2.0));
                self.pan_offset = focal_vp - (focal_vp - self.pan_offset) * zoom_ratio;
            }
        }

        // 5. Drag / Swipe Panning Logic (single finger / mouse)
        if !is_pinching {
            if mouse_pressed && in_viewport {
                self.drag_start = Some(mouse_pos);
                self.is_dragging = false;
            }

            if mouse_down {
                if let Some(start) = self.drag_start {
                    let delta = mouse_pos - start;
                    if delta.length_squared() > (6.0 * scale) * (6.0 * scale) {
                        self.is_dragging = true;
                        self.anim = None;
                    }

                    if self.is_dragging {
                        self.pan_offset += delta;
                        self.drag_start = Some(mouse_pos);
                    }
                }
            }
        }

        let was_dragging = self.is_dragging || double_tap_triggered || is_pinching;
        if mouse_released {
            self.drag_start = None;
            self.is_dragging = false;
        }

        // 6. Dynamic Boundary Clamping based on effective board and extension sizes
        let cur_board_size = base_board_size * self.zoom;
        let pad = 12.0 * scale;
        let left_ext = 384.0 * base_board_scale * self.zoom;
        let right_ext = 256.0 * base_board_scale * self.zoom;

        if !self.initialized {
            self.pan_offset.x = (viewport_rect.w - cur_board_size.x) / 2.0;
            self.pan_offset.y = if (cur_board_size.y + pad * 2.0) <= viewport_rect.h {
                (viewport_rect.h - cur_board_size.y) / 2.0
            } else {
                pad
            };
            self.initialized = true;
        }

        // Clamp horizontally across board and left/right scenery extensions
        let min_x = (viewport_rect.w - cur_board_size.x - right_ext)
            .min((viewport_rect.w - cur_board_size.x) / 2.0);
        let max_x = (left_ext).max((viewport_rect.w - cur_board_size.x) / 2.0);
        self.pan_offset.x = self.pan_offset.x.clamp(min_x, max_x);

        // Clamp vertically across board
        let total_h = cur_board_size.y + pad * 2.0;
        if total_h <= viewport_rect.h {
            self.pan_offset.y = (viewport_rect.h - cur_board_size.y) / 2.0;
        } else {
            let min_y = viewport_rect.h - cur_board_size.y - pad;
            let max_y = pad;
            self.pan_offset.y = self.pan_offset.y.clamp(min_y, max_y);
        }

        // 7. Render Target Setup for Crisp Smooth Subpixel Rendering
        let rt_w = (viewport_rect.w as u32).max(1);
        let rt_h = (viewport_rect.h as u32).max(1);
        if self.render_target.as_ref().is_none_or(|rt| {
            rt.texture.width() as u32 != rt_w || rt.texture.height() as u32 != rt_h
        }) {
            self.render_target = Some(render_target(rt_w, rt_h));
        }

        let rt = self.render_target.as_ref().unwrap().clone();

        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, viewport_rect.w, viewport_rect.h));
        camera.render_target = Some(rt.clone());
        set_camera(&camera);

        clear_background(Color::from_rgba(11, 17, 24, 255));

        let effective_scale = base_board_scale * self.zoom;
        (rt, self.pan_offset, effective_scale, was_dragging)
    }

    pub fn end_camera(&self, viewport_rect: Rect, rt: RenderTarget) {
        set_default_camera();

        // Render the target texture to screen with Y flip
        draw_texture_ex(
            &rt.texture,
            viewport_rect.x,
            viewport_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(viewport_rect.w, viewport_rect.h)),
                flip_y: true,
                ..Default::default()
            },
        );
    }
}
