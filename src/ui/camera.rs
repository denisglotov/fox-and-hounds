use macroquad::prelude::*;

pub struct ViewportCamera {
    pub pan_offset: Vec2,
    pub drag_start: Option<Vec2>,
    pub is_dragging: bool,
    pub render_target: Option<RenderTarget>,
    pub last_board_size: Vec2,
    pub initialized: bool,
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
            drag_start: None,
            is_dragging: false,
            render_target: None,
            last_board_size: Vec2::ZERO,
            initialized: false,
        }
    }

    pub fn reset_pan(&mut self) {
        self.pan_offset = Vec2::ZERO;
        self.drag_start = None;
        self.is_dragging = false;
        self.initialized = false;
    }

    /// Centers camera on player's pieces at match start (bottom for Fox, top for Hounds).
    pub fn center_on_faction(
        &mut self,
        faction: crate::game::state::Faction,
        viewport_rect: Rect,
        board_size: Vec2,
    ) {
        self.pan_offset.x = (viewport_rect.w - board_size.x) / 2.0;
        if board_size.y <= viewport_rect.h {
            self.pan_offset.y = (viewport_rect.h - board_size.y) / 2.0;
        } else {
            self.pan_offset.y = match faction {
                crate::game::state::Faction::Fox => (viewport_rect.h - board_size.y).min(0.0),
                crate::game::state::Faction::Hounds => 12.0,
            };
        }
        self.drag_start = None;
        self.is_dragging = false;
        self.initialized = true;
    }

    /// Handles touch drag, mouse drag, and wheel scroll. Clamps pan offsets to keep board in view.
    pub fn update_and_begin(
        &mut self,
        viewport_rect: Rect,
        board_size: Vec2,
        scale: f32,
    ) -> (RenderTarget, Vec2, bool) {
        let mouse_pos = Vec2::from(mouse_position());
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_released = is_mouse_button_released(MouseButton::Left);
        let (wheel_x, wheel_y) = mouse_wheel();

        // 1. Mouse wheel / Trackpad scrolling
        if wheel_y != 0.0 || wheel_x != 0.0 {
            let scroll_speed = 35.0 * scale;
            self.pan_offset.y += wheel_y * scroll_speed;
            self.pan_offset.x -= wheel_x * scroll_speed;
        }

        // 2. Drag / Swipe Panning Logic
        let in_viewport = viewport_rect.contains(mouse_pos);

        if mouse_pressed && in_viewport {
            self.drag_start = Some(mouse_pos);
            self.is_dragging = false;
        }

        if mouse_down {
            if let Some(start) = self.drag_start {
                let delta = mouse_pos - start;
                if delta.length_squared() > (6.0 * scale) * (6.0 * scale) {
                    self.is_dragging = true;
                }

                if self.is_dragging {
                    self.pan_offset += delta;
                    self.drag_start = Some(mouse_pos);
                }
            }
        }

        let was_dragging = self.is_dragging;
        if mouse_released {
            self.drag_start = None;
            self.is_dragging = false;
        }

        let pad = 12.0 * scale;
        let total_w = board_size.x + pad * 2.0;
        let total_h = board_size.y + pad * 2.0;

        // Auto-center horizontally and initialize vertical position on first run
        if !self.initialized {
            self.pan_offset.x = (viewport_rect.w - board_size.x) / 2.0;
            self.pan_offset.y = if total_h <= viewport_rect.h {
                (viewport_rect.h - board_size.y) / 2.0
            } else {
                pad
            };
            self.initialized = true;
        }

        // 3. Pan clamping (auto-center if board fits, otherwise clamp scroll)
        if total_w <= viewport_rect.w {
            self.pan_offset.x = (viewport_rect.w - board_size.x) / 2.0;
        } else {
            let min_x = viewport_rect.w - board_size.x - pad;
            let max_x = pad;
            self.pan_offset.x = self.pan_offset.x.clamp(min_x, max_x);
        }

        if total_h <= viewport_rect.h {
            self.pan_offset.y = (viewport_rect.h - board_size.y) / 2.0;
        } else {
            let min_y = viewport_rect.h - board_size.y - pad;
            let max_y = pad;
            self.pan_offset.y = self.pan_offset.y.clamp(min_y, max_y);
        }

        // Recreate render target texture if viewport dimensions change
        let rt_w = (viewport_rect.w as u32).max(1);
        let rt_h = (viewport_rect.h as u32).max(1);
        if self.render_target.as_ref().is_none_or(|rt| {
            rt.texture.width() as u32 != rt_w || rt.texture.height() as u32 != rt_h
        }) {
            self.render_target = Some(render_target(rt_w, rt_h));
        }

        let rt = self.render_target.as_ref().unwrap().clone();

        // Setup camera for subpixel smooth rendering
        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, viewport_rect.w, viewport_rect.h));
        camera.render_target = Some(rt.clone());
        set_camera(&camera);

        clear_background(Color::from_rgba(11, 17, 24, 255));

        (rt, self.pan_offset, was_dragging)
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
