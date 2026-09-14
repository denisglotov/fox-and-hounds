use crate::audio::SoundTrigger;
use crate::game::i18n::LocaleStrings;
use crate::game::level::BoardVariant;
use crate::ui::{draw_text_styled, measure_text_styled};
use macroquad::prelude::*;

const VARIANT_COUNT: usize = 5;
const FLANKING_SCALE: f32 = 0.85;
const FLANKING_ALPHA: f32 = 0.50;
const DRAG_CLICK_THRESHOLD: f32 = 6.0;

fn load_variant_texture(bytes: &[u8]) -> Option<Texture2D> {
    let tex = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    tex.set_filter(FilterMode::Linear);
    Some(tex)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CarouselOutcome {
    pub variant_changed: Option<BoardVariant>,
    pub start_game: bool,
    pub sound_trigger: Option<SoundTrigger>,
}

pub struct CarouselDrawConfig<'a> {
    pub bounds: Rect,
    pub clip_bounds: Rect,
    pub scale: f32,
    pub font: Option<&'a Font>,
    pub locales: &'a LocaleStrings,
    pub dt: f32,
}

fn draw_clipped_border(rect: Rect, clip_left: f32, clip_right: f32, thick: f32, color: Color) {
    let x1 = rect.x;
    let x2 = rect.x + rect.w;
    let draw_x1 = x1.max(clip_left);
    let draw_x2 = x2.min(clip_right);
    if draw_x2 <= draw_x1 {
        return;
    }
    draw_line(draw_x1, rect.y, draw_x2, rect.y, thick, color);
    draw_line(
        draw_x1,
        rect.y + rect.h,
        draw_x2,
        rect.y + rect.h,
        thick,
        color,
    );

    if x1 >= clip_left {
        draw_line(x1, rect.y, x1, rect.y + rect.h, thick, color);
    }
    if x2 <= clip_right {
        draw_line(x2, rect.y, x2, rect.y + rect.h, thick, color);
    }
}

#[derive(Debug, Clone, Copy)]
struct VisibleCard {
    variant: BoardVariant,
    tex_idx: usize,
    relative_offset: f32,
    center_x: f32,
    scale_factor: f32,
    alpha: f32,
}

impl VisibleCard {
    const EMPTY: Self = Self {
        variant: BoardVariant::Classic,
        tex_idx: 0,
        relative_offset: 0.0,
        center_x: 0.0,
        scale_factor: 1.0,
        alpha: 0.0,
    };
}

pub struct BoardCarousel {
    textures: [Option<Texture2D>; VARIANT_COUNT],
    textures_loaded: bool,
    scroll_pos: f32,
    target_pos: f32,
    is_dragging: bool,
    drag_start_x: f32,
    drag_start_scroll: f32,
    last_mouse_x: f32,
    velocity: f32,
    drag_distance: f32,
    last_synced_variant: Option<BoardVariant>,
}

impl Default for BoardCarousel {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardCarousel {
    pub fn new() -> Self {
        Self {
            textures: [None, None, None, None, None],
            textures_loaded: false,
            scroll_pos: 0.0,
            target_pos: 0.0,
            is_dragging: false,
            drag_start_x: 0.0,
            drag_start_scroll: 0.0,
            last_mouse_x: 0.0,
            velocity: 0.0,
            drag_distance: 0.0,
            last_synced_variant: None,
        }
    }

    pub fn load_textures(&mut self) {
        let variants = BoardVariant::all();
        for (i, variant) in variants.iter().enumerate() {
            if i < VARIANT_COUNT {
                self.textures[i] = load_variant_texture(variant.config().board_image_bytes);
            }
        }
        self.textures_loaded = true;
    }

    pub fn current_index(&self) -> usize {
        let count = VARIANT_COUNT as f32;
        let idx = self.target_pos.round().rem_euclid(count) as usize;
        idx.min(VARIANT_COUNT - 1)
    }

    pub fn current_variant(&self) -> BoardVariant {
        BoardVariant::all()[self.current_index()]
    }

    pub fn sync_variant(&mut self, variant: BoardVariant) {
        if self.is_dragging {
            return;
        }
        if self.last_synced_variant == Some(variant) {
            return;
        }
        self.last_synced_variant = Some(variant);
        let variants = BoardVariant::all();
        if let Some(target_idx) = variants.iter().position(|&v| v == variant) {
            if target_idx != self.current_index() {
                self.set_target_index(target_idx);
            }
        }
    }

    pub fn set_target_index(&mut self, target_idx: usize) {
        let count = VARIANT_COUNT as f32;
        let target_f = target_idx as f32;
        let diff = (target_f - self.scroll_pos + count * 0.5).rem_euclid(count) - count * 0.5;
        self.target_pos = self.scroll_pos + diff;
    }

    pub fn step(&mut self, delta: f32) {
        self.target_pos = self.target_pos.round() + delta;
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_dragging {
            if self.scroll_pos == self.target_pos {
                return;
            }
            let smoothing = 1.0 - (-16.0 * dt).exp();
            self.scroll_pos += (self.target_pos - self.scroll_pos) * smoothing;
            if (self.target_pos - self.scroll_pos).abs() < 0.002 {
                self.scroll_pos = self.target_pos;
                let count = VARIANT_COUNT as f32;
                self.target_pos = self.target_pos.rem_euclid(count);
                self.scroll_pos = self.target_pos;
            }
        }
    }

    pub fn draw(&mut self, config: &CarouselDrawConfig) -> CarouselOutcome {
        if !self.textures_loaded {
            self.load_textures();
        }
        self.update(config.dt);

        let mut outcome = CarouselOutcome::default();
        let variants = BoardVariant::all();

        // 1. Carousel Track Metrics
        let track = config.bounds;
        let track_cx = track.x + track.w / 2.0;
        let track_cy = track.y + track.h / 2.0;
        let card_w = (track.w * 0.62).min(270.0 * config.scale);
        let card_h = track.h;
        let button_gap = 10.0 * config.scale;
        let card_step =
            (card_w * (1.0 + FLANKING_SCALE) * 0.5 + button_gap).max(40.0 * config.scale);
        let center_card_half_w = card_w * 0.5 + button_gap * 0.5;

        let clip_left = config.clip_bounds.x;
        let clip_right = config.clip_bounds.x + config.clip_bounds.w;

        // 2. Mouse / Touch Gestures (bounded within interactive carousel area)
        let mouse_pos = Vec2::from(mouse_position());
        let mouse_in_interactive_area = mouse_pos.x >= clip_left
            && mouse_pos.x <= clip_right
            && mouse_pos.y >= track.y
            && mouse_pos.y <= track.y + track.h;

        if is_mouse_button_pressed(MouseButton::Left) && mouse_in_interactive_area {
            self.is_dragging = true;
            self.drag_start_x = mouse_pos.x;
            self.drag_start_scroll = self.scroll_pos;
            self.last_mouse_x = mouse_pos.x;
            self.velocity = 0.0;
            self.drag_distance = 0.0;
        }

        if self.is_dragging {
            let dx = mouse_pos.x - self.last_mouse_x;
            self.drag_distance += (mouse_pos.x - self.last_mouse_x).abs();
            if config.dt > 0.0001 {
                let inst_vel = dx / config.dt;
                self.velocity = self.velocity * 0.6 + inst_vel * 0.4;
            }
            self.last_mouse_x = mouse_pos.x;

            let total_dx = mouse_pos.x - self.drag_start_x;
            self.scroll_pos = self.drag_start_scroll - total_dx / card_step;
            self.target_pos = self.scroll_pos;

            // Dynamically update variant as cards cross the center while swiping
            let active_v = self.current_variant();
            if self.last_synced_variant != Some(active_v) {
                self.last_synced_variant = Some(active_v);
                outcome.variant_changed = Some(active_v);
            }

            if is_mouse_button_released(MouseButton::Left)
                || !is_mouse_button_down(MouseButton::Left)
            {
                self.is_dragging = false;
                let start_idx = (self
                    .drag_start_scroll
                    .round()
                    .rem_euclid(VARIANT_COUNT as f32) as usize)
                    .min(VARIANT_COUNT - 1);

                if self.drag_distance > DRAG_CLICK_THRESHOLD * config.scale {
                    // Gesture was a drag: fling and snap
                    let fling = (self.velocity * 0.08 / card_step).clamp(-1.5, 1.5);
                    self.target_pos = (self.scroll_pos - fling).round();
                    let final_v = self.current_variant();
                    outcome.variant_changed = Some(final_v);
                    self.last_synced_variant = Some(final_v);
                    if self.current_index() != start_idx {
                        outcome.sound_trigger = Some(SoundTrigger::ButtonClick);
                    }
                } else {
                    // Gesture was a click: inspect hit card
                    let click_dx = mouse_pos.x - track_cx;
                    if click_dx < -center_card_half_w {
                        // Clicked on left peeking card -> step left
                        self.step(-1.0);
                        let v = self.current_variant();
                        outcome.variant_changed = Some(v);
                        self.last_synced_variant = Some(v);
                        outcome.sound_trigger = Some(SoundTrigger::ButtonClick);
                    } else if click_dx > center_card_half_w {
                        // Clicked on right peeking card -> step right
                        self.step(1.0);
                        let v = self.current_variant();
                        outcome.variant_changed = Some(v);
                        self.last_synced_variant = Some(v);
                        outcome.sound_trigger = Some(SoundTrigger::ButtonClick);
                    } else {
                        // Clicked on the selected (center) card -> start the game!
                        outcome.start_game = true;
                        outcome.sound_trigger = Some(SoundTrigger::ButtonClick);
                    }
                }
            }
        }

        // 3. Determine Visible Cards (Stack-allocated fixed array for zero heap churn)
        let base_idx_f = self.scroll_pos.round();
        let base_idx = base_idx_f as i32;

        let mut visible_cards = [VisibleCard::EMPTY; 5];
        let mut visible_count = 0;

        for offset in -2..=2 {
            let card_idx_i = (base_idx + offset).rem_euclid(VARIANT_COUNT as i32);
            let variant = variants[card_idx_i as usize];
            let continuous_card_pos = (base_idx + offset) as f32;
            let rel_offset = continuous_card_pos - self.scroll_pos;
            let center_x = track_cx + rel_offset * card_step;

            let dist_from_center = rel_offset.abs();
            if dist_from_center > 1.8 {
                continue;
            }

            let focus_factor = (1.0 - dist_from_center.min(1.0)).clamp(0.0, 1.0);
            // Fast hardware square root for x^1.5 instead of expensive powf
            let scale_factor =
                FLANKING_SCALE + (1.0 - FLANKING_SCALE) * (focus_factor * focus_factor.sqrt());

            let edge_fade = if dist_from_center <= 1.05 {
                1.0
            } else {
                (1.0 - (dist_from_center - 1.05) / 0.75).clamp(0.0, 1.0)
            };

            let alpha = (FLANKING_ALPHA + (1.0 - FLANKING_ALPHA) * focus_factor) * edge_fade;
            if alpha < 0.02 {
                continue;
            }

            if visible_count < visible_cards.len() {
                visible_cards[visible_count] = VisibleCard {
                    variant,
                    tex_idx: card_idx_i as usize,
                    relative_offset: rel_offset,
                    center_x,
                    scale_factor,
                    alpha,
                };
                visible_count += 1;
            }
        }

        // 4. Render Cards (Sort back-to-front in-place on stack so center card draws over flanks)
        let cards = &mut visible_cards[..visible_count];
        cards.sort_unstable_by(|a, b| b.relative_offset.abs().total_cmp(&a.relative_offset.abs()));

        for card in cards.iter() {
            let s = card.scale_factor;
            let w = card_w * s;
            let h = card_h * s;
            let rect = Rect::new(card.center_x - w / 2.0, track_cy - h / 2.0, w, h);

            let card_x1 = rect.x;
            let card_x2 = rect.x + rect.w;

            // Skip cards wholly outside the clip boundary
            if card_x2 <= clip_left || card_x1 >= clip_right {
                continue;
            }

            let draw_x1 = card_x1.max(clip_left);
            let draw_x2 = card_x2.min(clip_right);
            let draw_w = draw_x2 - draw_x1;
            if draw_w <= 0.0 {
                continue;
            }

            let is_center = card.relative_offset.abs() < 0.35;
            let is_center_hovered = is_center
                && mouse_in_interactive_area
                && (mouse_pos.x - track_cx).abs() <= center_card_half_w;
            let bg_alpha = if is_center { 245.0 } else { 190.0 };
            let bg_color = if is_center_hovered {
                Color::from_rgba(35, 52, 75, (255.0 * card.alpha) as u8)
            } else {
                Color::from_rgba(26, 36, 50, (bg_alpha * card.alpha) as u8)
            };

            // Card body plate (clipped to [draw_x1, draw_x2])
            draw_rectangle(draw_x1, rect.y, draw_w, rect.h, bg_color);

            // Border (gold glow for center, brighter on hover, subtle border for peeking cards)
            let border_color = if is_center {
                if is_center_hovered {
                    Color::from_rgba(255, 238, 150, (255.0 * card.alpha) as u8)
                } else {
                    let glow = (1.0 - (card.relative_offset.abs() / 0.35)).clamp(0.0, 1.0);
                    Color::from_rgba(255, 224, 130, (230.0 * glow * card.alpha) as u8)
                }
            } else {
                Color::from_rgba(255, 255, 255, (45.0 * card.alpha) as u8)
            };
            let border_thick = if is_center {
                if is_center_hovered {
                    2.2 * config.scale * s
                } else {
                    1.8 * config.scale * s
                }
            } else {
                1.0 * config.scale * s
            };

            draw_clipped_border(rect, clip_left, clip_right, border_thick, border_color);

            // Mini Thumbnail on Left (anchor inside visible portion if clipped on left)
            let pad = 6.0 * config.scale * s;
            let thumb_sz = (rect.h - pad * 2.0).max(12.0 * config.scale);
            let thumb_x = if card_x1 < clip_left {
                (draw_x2 - pad - thumb_sz).max(draw_x1 + pad)
            } else {
                rect.x + pad
            };
            let thumb_rect = Rect::new(thumb_x, rect.y + pad, thumb_sz, thumb_sz);

            let thumb_x1 = thumb_rect.x;
            let thumb_x2 = thumb_rect.x + thumb_rect.w;
            if thumb_x2 > clip_left && thumb_x1 < clip_right {
                let t_draw_x1 = thumb_x1.max(clip_left);
                let t_draw_x2 = thumb_x2.min(clip_right);
                let t_draw_w = t_draw_x2 - t_draw_x1;

                draw_rectangle(
                    t_draw_x1,
                    thumb_rect.y,
                    t_draw_w,
                    thumb_rect.h,
                    Color::from_rgba(14, 20, 30, (230.0 * card.alpha) as u8),
                );

                if let Some(Some(tex)) = self.textures.get(card.tex_idx) {
                    let tex_aspect = tex.width() / tex.height().max(1.0);
                    let (draw_w, draw_h) = if tex_aspect >= 1.0 {
                        (thumb_sz, thumb_sz / tex_aspect)
                    } else {
                        (thumb_sz * tex_aspect, thumb_sz)
                    };
                    let raw_tx = thumb_rect.x + (thumb_sz - draw_w) / 2.0;
                    let ty = thumb_rect.y + (thumb_sz - draw_h) / 2.0;

                    let img_x1 = raw_tx.max(clip_left);
                    let img_x2 = (raw_tx + draw_w).min(clip_right);
                    if img_x2 > img_x1 {
                        let tex_w = tex.width();
                        let tex_h = tex.height();
                        let u1 = ((img_x1 - raw_tx) / draw_w).clamp(0.0, 1.0) * tex_w;
                        let u2 = ((img_x2 - raw_tx) / draw_w).clamp(0.0, 1.0) * tex_w;
                        let sub_w = (u2 - u1).max(0.1);
                        let dest_w = img_x2 - img_x1;

                        draw_texture_ex(
                            tex,
                            img_x1,
                            ty,
                            Color::new(1.0, 1.0, 1.0, card.alpha),
                            DrawTextureParams {
                                dest_size: Some(vec2(dest_w, draw_h)),
                                source: Some(Rect::new(u1, 0.0, sub_w, tex_h)),
                                ..Default::default()
                            },
                        );
                    }
                }

                draw_clipped_border(
                    thumb_rect,
                    clip_left,
                    clip_right,
                    1.0 * config.scale * s,
                    Color::from_rgba(255, 255, 255, (30.0 * card.alpha) as u8),
                );
            }

            // Right side: Localized Name and Subtitle
            let text_x = thumb_rect.x + thumb_rect.w + 8.0 * config.scale * s;
            let right_pad = 8.0 * config.scale * s;
            let content_right = (rect.x + rect.w).min(clip_right) - right_pad;
            let max_text_w = content_right - text_x;

            if text_x >= clip_left && max_text_w >= 20.0 * config.scale * s {
                let title_text = card.variant.localized_name(config.locales);
                let sub_text = card.variant.localized_sub(config.locales);

                let title_size = ((13.5 * config.scale * s) as u16).max(8);
                let sub_size = ((10.5 * config.scale * s) as u16).max(7);

                let title_dims = measure_text_styled(title_text, title_size, config.font);
                let sub_dims = measure_text_styled(sub_text, sub_size, config.font);

                let text_total_h = title_dims.height + 4.0 * config.scale * s + sub_dims.height;
                let text_start_y = rect.y + (rect.h - text_total_h) / 2.0 + title_dims.height;

                let title_color = Color::new(1.0, 1.0, 1.0, card.alpha);
                let sub_color = Color::new(0.72, 0.80, 0.88, card.alpha * 0.88);

                // Scale text if needed to fit width
                if title_dims.width > max_text_w {
                    let fit_ratio = (max_text_w / title_dims.width).clamp(0.5, 1.0);
                    let scaled_size = ((title_size as f32) * fit_ratio) as u16;
                    draw_text_styled(
                        title_text,
                        text_x,
                        text_start_y,
                        scaled_size,
                        title_color,
                        config.font,
                    );
                } else {
                    draw_text_styled(
                        title_text,
                        text_x,
                        text_start_y,
                        title_size,
                        title_color,
                        config.font,
                    );
                }

                if sub_dims.width > max_text_w {
                    let fit_ratio = (max_text_w / sub_dims.width).clamp(0.5, 1.0);
                    let scaled_size = ((sub_size as f32) * fit_ratio) as u16;
                    draw_text_styled(
                        sub_text,
                        text_x,
                        text_start_y + 4.0 * config.scale * s + sub_dims.height,
                        scaled_size,
                        sub_color,
                        config.font,
                    );
                } else {
                    draw_text_styled(
                        sub_text,
                        text_x,
                        text_start_y + 4.0 * config.scale * s + sub_dims.height,
                        sub_size,
                        sub_color,
                        config.font,
                    );
                }
            }
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carousel_navigation_and_wrapping() {
        let mut carousel = BoardCarousel::new();
        assert_eq!(carousel.current_index(), 0);
        assert_eq!(carousel.current_variant(), BoardVariant::Classic);

        // Step forward
        carousel.step(1.0);
        assert_eq!(carousel.current_index(), 1);
        assert_eq!(carousel.current_variant(), BoardVariant::RiverCrossing);

        // Target index directly
        carousel.set_target_index(4);
        assert_eq!(carousel.current_index(), 4);
        assert_eq!(carousel.current_variant(), BoardVariant::TheRedHunt);

        // Circular wrap forward from 4 -> 0
        carousel.step(1.0);
        assert_eq!(carousel.current_index(), 0);
        assert_eq!(carousel.current_variant(), BoardVariant::Classic);

        // Circular wrap backward from 0 -> 4
        carousel.step(-1.0);
        assert_eq!(carousel.current_index(), 4);
        assert_eq!(carousel.current_variant(), BoardVariant::TheRedHunt);
    }

    #[test]
    fn test_carousel_smooth_update() {
        let mut carousel = BoardCarousel::new();
        carousel.step(1.0);
        // Simulate frames
        for _ in 0..30 {
            carousel.update(0.016);
        }
        assert_eq!(carousel.current_index(), 1);
        assert!((carousel.scroll_pos - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_carousel_sync_variant() {
        let mut carousel = BoardCarousel::new();
        carousel.sync_variant(BoardVariant::FoxAndDogs);
        assert_eq!(carousel.current_variant(), BoardVariant::FoxAndDogs);

        // When already synced, calling sync_variant does not reset anything
        carousel.sync_variant(BoardVariant::FoxAndDogs);
        assert_eq!(carousel.current_variant(), BoardVariant::FoxAndDogs);

        // While dragging, sync_variant is a no-op
        carousel.is_dragging = true;
        carousel.sync_variant(BoardVariant::Classic);
        assert_eq!(carousel.current_variant(), BoardVariant::FoxAndDogs);
    }

    #[test]
    fn test_carousel_clip_math() {
        let clip_left: f32 = 50.0;
        let clip_right: f32 = 250.0;

        // Card sticking out on the left: clipped to clip_left
        let c_left: f32 = 20.0;
        let c_right: f32 = 120.0;
        let draw_x1 = c_left.max(clip_left);
        let draw_x2 = c_right.min(clip_right);
        assert_eq!(draw_x1, 50.0);
        assert_eq!(draw_x2, 120.0);
        assert!(draw_x1 >= clip_left && draw_x2 <= clip_right);

        // Card sticking out on the right: clipped to clip_right
        let c2_left: f32 = 180.0;
        let c2_right: f32 = 290.0;
        let draw2_x1 = c2_left.max(clip_left);
        let draw2_x2 = c2_right.min(clip_right);
        assert_eq!(draw2_x1, 180.0);
        assert_eq!(draw2_x2, 250.0);
        assert!(draw2_x1 >= clip_left && draw2_x2 <= clip_right);

        // Card wholly outside to the left
        let _c3_left: f32 = -100.0;
        let c3_right: f32 = 40.0;
        assert!(c3_right <= clip_left);

        // Card wholly outside to the right
        let c4_left: f32 = 260.0;
        let _c4_right: f32 = 360.0;
        assert!(c4_left >= clip_right);
    }

    #[test]
    fn test_carousel_button_gap() {
        let scale: f32 = 1.5;
        let button_gap = 10.0 * scale;
        let card_w = 200.0 * scale;
        let card_step = card_w * (1.0 + FLANKING_SCALE) * 0.5 + button_gap;

        let center_cx = 300.0;
        let center_right = center_cx + card_w * 0.5;

        let flank_cx = center_cx + card_step;
        let flank_w = card_w * FLANKING_SCALE;
        let flank_left = flank_cx - flank_w * 0.5;

        let gap = flank_left - center_right;
        assert!((gap - button_gap).abs() < 0.001);
    }
}
