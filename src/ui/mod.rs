pub mod board_view;
pub mod boat;
pub mod camera;
pub mod carousel;
pub mod fx;
pub mod nav;
pub mod river;
pub mod rover;
pub mod screens;
pub mod story;
pub mod train;

use macroquad::prelude::*;
use std::cell::Cell;

#[derive(Clone, Copy)]
struct MeasureCacheEntry {
    ptr: usize,
    len: usize,
    font_size: u16,
    hash: u64,
    dims: TextDimensions,
}

thread_local! {
    static MEASURE_CACHE: Cell<[Option<MeasureCacheEntry>; 128]> = const { Cell::new([None; 128]) };
}

#[inline]
fn quick_str_hash(text: &str) -> u64 {
    let bytes = text.as_bytes();
    let mut h = bytes.len() as u64;
    for &b in bytes.iter().take(32) {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    h
}

pub fn draw_text_styled(
    text: &str,
    x: f32,
    y: f32,
    font_size: u16,
    color: Color,
    font: Option<&Font>,
) {
    if let Some(f) = font {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(f),
                font_size,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color,
            },
        );
    } else {
        draw_text(text, x, y, font_size as f32, color);
    }
}

pub fn measure_text_styled(text: &str, font_size: u16, font: Option<&Font>) -> TextDimensions {
    let ptr = text.as_ptr() as usize;
    let len = text.len();
    let hash = quick_str_hash(text);
    let idx = (ptr ^ (font_size as usize) ^ (hash as usize)) & 127;

    let cached = MEASURE_CACHE.with(|c| {
        let entries = c.get();
        entries[idx]
    });

    if let Some(e) = cached {
        if e.ptr == ptr && e.len == len && e.font_size == font_size && e.hash == hash {
            return e.dims;
        }
    }

    let dims = measure_text(text, font, font_size, 1.0);
    MEASURE_CACHE.with(|c| {
        let mut entries = c.get();
        entries[idx] = Some(MeasureCacheEntry {
            ptr,
            len,
            font_size,
            hash,
            dims,
        });
        c.set(entries);
    });
    dims
}

pub fn draw_loading_overlay(
    screen_w: f32,
    screen_h: f32,
    scale: f32,
    icon_texture: Option<&Texture2D>,
    time: f32,
) {
    // 1. Full-screen background (#0b1118)
    draw_rectangle(
        0.0,
        0.0,
        screen_w,
        screen_h,
        Color::from_rgba(11, 17, 24, 255),
    );

    let cx = screen_w / 2.0;
    let cy = screen_h / 2.0;
    let pulse = (time * 2.856).sin() * 0.5 + 0.5;

    // 2. Ambient glows (orange left/top, sky blue right/bottom)
    draw_circle(
        cx,
        cy,
        (68.0 + pulse * 6.0) * scale,
        Color::from_rgba(249, 115, 22, (16.0 + pulse * 8.0) as u8),
    );
    draw_circle(
        cx,
        cy,
        (62.0 + (1.0 - pulse) * 6.0) * scale,
        Color::from_rgba(56, 189, 248, (16.0 + (1.0 - pulse) * 8.0) as u8),
    );

    // 3. Spinner track ring (.spinner-track: 2px solid rgba(255, 255, 255, 0.06))
    let outer_r = 52.0 * scale;
    draw_circle_lines(
        cx,
        cy,
        outer_r,
        2.0 * scale,
        Color::from_rgba(255, 255, 255, 18),
    );

    // 4. Spinner outer arc (.spinner-arc: spin-clockwise 1.2s, top orange #f97316, right blue #38bdf8)
    let clockwise_rot = (time * 300.0) % 360.0;
    draw_arc(
        cx,
        cy,
        32,
        outer_r,
        clockwise_rot,
        3.2 * scale,
        100.0,
        Color::from_rgba(249, 115, 22, 255),
    );
    draw_arc(
        cx,
        cy,
        32,
        outer_r,
        clockwise_rot + 95.0,
        3.2 * scale,
        100.0,
        Color::from_rgba(56, 189, 248, 255),
    );

    // 5. Spinner inner arc (.spinner-arc-inner: inset 7px, dashed rgba(56, 189, 248, 0.35), counter-clockwise 3.6s)
    let inner_r = 45.0 * scale;
    let counter_rot = (-time * 100.0) % 360.0;
    let inner_color = Color::from_rgba(56, 189, 248, 90);
    for i in 0..4 {
        let seg_angle = counter_rot + (i as f32) * 90.0;
        draw_arc(
            cx,
            cy,
            16,
            inner_r,
            seg_angle,
            2.0 * scale,
            45.0,
            inner_color,
        );
    }

    // 6. Spinner core (.spinner-core: 60px, radial gradient #1e293b, core-pulse 2.2s)
    let core_scale = 0.96 + pulse * 0.08;
    let core_r = 30.0 * scale * core_scale;
    draw_circle(cx, cy, core_r, Color::from_rgba(30, 41, 59, 255));
    draw_circle_lines(
        cx,
        cy,
        core_r,
        1.5 * scale,
        Color::from_rgba(255, 255, 255, 25),
    );

    // 7. Center icon (.spinner-icon: 42px)
    if let Some(tex) = icon_texture {
        let icon_size = 42.0 * scale * core_scale;
        draw_texture_ex(
            tex,
            cx - icon_size / 2.0,
            cy - icon_size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(icon_size, icon_size)),
                ..Default::default()
            },
        );
    }
}
