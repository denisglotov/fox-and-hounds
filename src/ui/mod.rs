pub mod board_view;
pub mod camera;
pub mod fx;
pub mod screens;

use macroquad::prelude::*;

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
    measure_text(text, font, font_size, 1.0)
}
