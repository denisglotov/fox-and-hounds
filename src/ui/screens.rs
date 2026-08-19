use crate::audio::SoundTrigger;
use crate::game::state::{Difficulty, Faction, GamePhase, GameResult, GameState};
use crate::ui::{draw_text_styled, measure_text_styled};
use macroquad::prelude::*;

pub struct SelectableButtonConfig<'a> {
    pub bounds: Rect,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub is_selected: bool,
    pub accent_color: Color,
    pub scale: f32,
    pub font: Option<&'a Font>,
}

pub struct ActionButtonConfig<'a> {
    pub bounds: Rect,
    pub text: &'a str,
    pub normal_color: Color,
    pub hover_color: Color,
    pub scale: f32,
    pub font: Option<&'a Font>,
}

pub struct IconButtonConfig<'a> {
    pub bounds: Rect,
    pub icon: &'a str,
    pub scale: f32,
    pub font: Option<&'a Font>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitleScreenLayout {
    pub is_landscape: bool,
    pub card_bounds: Rect,
    pub left_column: Rect,
    pub right_column: Rect,
    pub hero_bounds: Option<Rect>,
    pub fox_btn_bounds: Rect,
    pub hounds_btn_bounds: Rect,
    pub difficulty_btn_bounds: [Rect; 3],
    pub start_btn_bounds: Rect,
}

impl TitleScreenLayout {
    pub fn compute(
        screen_w: f32,
        screen_h: f32,
        scale: f32,
        has_hero_texture: bool,
        hero_aspect: f32,
    ) -> Self {
        let is_landscape = screen_w > screen_h * 1.1;
        if is_landscape {
            Self::compute_landscape(screen_w, screen_h, scale, has_hero_texture, hero_aspect)
        } else {
            Self::compute_portrait(screen_w, screen_h, scale, has_hero_texture, hero_aspect)
        }
    }

    fn compute_landscape(
        screen_w: f32,
        screen_h: f32,
        scale: f32,
        has_hero_texture: bool,
        hero_aspect: f32,
    ) -> Self {
        let max_card_w = (screen_w - 32.0 * scale).min(780.0 * scale);
        let card_w = max_card_w
            .max((screen_w - 16.0 * scale).min(320.0 * scale))
            .min(screen_w - 16.0 * scale);
        let max_card_h = (screen_h - 24.0 * scale).min(420.0 * scale);
        let card_h = max_card_h
            .max((screen_h - 16.0 * scale).min(240.0 * scale))
            .min(screen_h - 16.0 * scale);
        let card_x = (screen_w - card_w) / 2.0;
        let card_y = (screen_h - card_h) / 2.0;
        let card_bounds = Rect::new(card_x, card_y, card_w, card_h);

        let pad = (16.0 * scale).min(card_w * 0.04).min(card_h * 0.06);
        let col_gap = (16.0 * scale).min(card_w * 0.04);
        let col_w = (card_w - pad * 2.0 - col_gap) / 2.0;

        let left_column = Rect::new(card_x + pad, card_y + pad, col_w, card_h - pad * 2.0);
        let right_column = Rect::new(
            left_column.x + col_w + col_gap,
            card_y + pad,
            col_w,
            card_h - pad * 2.0,
        );

        let title_h = 24.0 * scale;
        let sub_h = 14.0 * scale;
        let left_header_h = title_h + sub_h + 12.0 * scale;
        let avail_hero_h = (left_column.h - left_header_h - 8.0 * scale).max(0.0);

        let hero_bounds = if has_hero_texture && avail_hero_h > 30.0 * scale {
            let aspect = if hero_aspect > 0.0 {
                hero_aspect
            } else {
                16.0 / 9.0
            };
            let hero_w = col_w.min(avail_hero_h * aspect);
            let hero_h = (hero_w / aspect).min(avail_hero_h);
            let hero_x = left_column.x + (col_w - hero_w) / 2.0;
            let hero_y = left_column.y + left_header_h + (avail_hero_h - hero_h) / 2.0;
            Some(Rect::new(hero_x, hero_y, hero_w, hero_h))
        } else {
            None
        };

        let f_btn_h = (46.0 * scale).min(right_column.h * 0.20);
        let d_btn_h = (34.0 * scale).min(right_column.h * 0.16);
        let s_btn_h = (44.0 * scale).min(right_column.h * 0.20);
        let label_h = (12.0 * scale).min(right_column.h * 0.07);

        let total_fixed = label_h * 2.0 + f_btn_h + d_btn_h + s_btn_h;
        let remaining_h = (right_column.h - total_fixed).max(0.0);
        let spacing = (remaining_h / 4.0).min(14.0 * scale);
        let total_content_h = total_fixed + spacing * 4.0;
        let mut curr_y = right_column.y + (right_column.h - total_content_h) / 2.0;

        curr_y += label_h + spacing * 0.5;

        let f_gap = (8.0 * scale).min(col_w * 0.04);
        let f_w = (col_w - f_gap) / 2.0;
        let fox_btn_bounds = Rect::new(right_column.x, curr_y, f_w, f_btn_h);
        let hounds_btn_bounds = Rect::new(right_column.x + f_w + f_gap, curr_y, f_w, f_btn_h);
        curr_y += f_btn_h + spacing;

        curr_y += label_h + spacing * 0.5;

        let d_gap = (6.0 * scale).min(col_w * 0.03);
        let d_w = (col_w - d_gap * 2.0) / 3.0;
        let difficulty_btn_bounds = [
            Rect::new(right_column.x, curr_y, d_w, d_btn_h),
            Rect::new(right_column.x + d_w + d_gap, curr_y, d_w, d_btn_h),
            Rect::new(right_column.x + (d_w + d_gap) * 2.0, curr_y, d_w, d_btn_h),
        ];
        curr_y += d_btn_h + spacing * 1.1;

        let start_btn_bounds = Rect::new(right_column.x, curr_y, col_w, s_btn_h);

        Self {
            is_landscape: true,
            card_bounds,
            left_column,
            right_column,
            hero_bounds,
            fox_btn_bounds,
            hounds_btn_bounds,
            difficulty_btn_bounds,
            start_btn_bounds,
        }
    }

    fn compute_portrait(
        screen_w: f32,
        screen_h: f32,
        scale: f32,
        has_hero_texture: bool,
        hero_aspect: f32,
    ) -> Self {
        let card_w = (screen_w - 32.0 * scale)
            .min(460.0 * scale)
            .max(280.0 * scale)
            .min(screen_w - 16.0 * scale);
        let avail_h = screen_h - 24.0 * scale;

        let banner_pad = (16.0 * scale).min(card_w * 0.05);
        let banner_w = card_w - banner_pad * 2.0;

        let f_btn_h = (50.0 * scale).min(avail_h * 0.12);
        let d_btn_h = (38.0 * scale).min(avail_h * 0.09);
        let s_btn_h = (48.0 * scale).min(avail_h * 0.11);

        let non_banner_h = 20.0 * scale
            + 28.0 * scale // title
            + 18.0 * scale // subtitle
            + 16.0 * scale // faction label
            + f_btn_h
            + 18.0 * scale // diff label
            + d_btn_h
            + 20.0 * scale
            + s_btn_h
            + 20.0 * scale;

        let aspect = if hero_aspect > 0.0 {
            hero_aspect
        } else {
            16.0 / 9.0
        };
        let ideal_banner_h = banner_w / aspect;
        let max_banner_h = (avail_h - non_banner_h - 16.0 * scale).max(0.0);
        let banner_h = ideal_banner_h.min(max_banner_h).min(180.0 * scale);
        let show_banner = has_hero_texture && banner_h >= 40.0 * scale;

        let content_h = non_banner_h
            + if show_banner {
                banner_h + 16.0 * scale
            } else {
                16.0 * scale
            };
        let card_h = content_h.min(avail_h);
        let card_x = (screen_w - card_w) / 2.0;
        let card_y = ((screen_h - card_h) / 2.0).max(12.0 * scale);
        let card_bounds = Rect::new(card_x, card_y, card_w, card_h);

        let mut curr_y = card_y + 20.0 * scale + 28.0 * scale + 18.0 * scale;

        let hero_bounds = if show_banner {
            let h_rect = Rect::new(card_x + banner_pad, curr_y, banner_w, banner_h);
            curr_y += banner_h + 16.0 * scale;
            Some(h_rect)
        } else {
            curr_y += 16.0 * scale;
            None
        };

        curr_y += 16.0 * scale; // faction label

        let f_gap = 10.0 * scale;
        let f_w = (card_w - 36.0 * scale - f_gap) / 2.0;
        let fox_btn_bounds = Rect::new(card_x + 18.0 * scale, curr_y, f_w, f_btn_h);
        let hounds_btn_bounds =
            Rect::new(card_x + 18.0 * scale + f_w + f_gap, curr_y, f_w, f_btn_h);
        curr_y += f_btn_h + 18.0 * scale;

        curr_y += 18.0 * scale; // diff label

        let d_gap = 10.0 * scale;
        let d_w = (card_w - 36.0 * scale - d_gap * 2.0) / 3.0;
        let difficulty_btn_bounds = [
            Rect::new(card_x + 18.0 * scale, curr_y, d_w, d_btn_h),
            Rect::new(card_x + 18.0 * scale + d_w + d_gap, curr_y, d_w, d_btn_h),
            Rect::new(
                card_x + 18.0 * scale + (d_w + d_gap) * 2.0,
                curr_y,
                d_w,
                d_btn_h,
            ),
        ];
        curr_y += d_btn_h + 20.0 * scale;

        let start_w = card_w - 36.0 * scale;
        let start_btn_bounds = Rect::new(card_x + 18.0 * scale, curr_y, start_w, s_btn_h);

        Self {
            is_landscape: false,
            card_bounds,
            left_column: card_bounds,
            right_column: card_bounds,
            hero_bounds,
            fox_btn_bounds,
            hounds_btn_bounds,
            difficulty_btn_bounds,
            start_btn_bounds,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameOverModalLayout {
    pub modal_bounds: Rect,
    pub rematch_btn_bounds: Rect,
    pub menu_btn_bounds: Rect,
}

impl GameOverModalLayout {
    pub fn compute(screen_w: f32, screen_h: f32, scale: f32) -> Self {
        let modal_w = (screen_w - 32.0 * scale)
            .min(380.0 * scale)
            .max(280.0 * scale)
            .min(screen_w - 16.0 * scale);
        let modal_h = (screen_h - 32.0 * scale)
            .min(320.0 * scale)
            .max(240.0 * scale)
            .min(screen_h - 16.0 * scale);
        let modal_x = (screen_w - modal_w) / 2.0;
        let modal_y = (screen_h - modal_h) / 2.0;
        let modal_bounds = Rect::new(modal_x, modal_y, modal_w, modal_h);

        let btn_w = modal_w - 40.0 * scale;
        let btn_h = (44.0 * scale).min(modal_h * 0.16);
        let btn_x = modal_x + 20.0 * scale;
        let menu_y = modal_y + modal_h - 18.0 * scale - btn_h;
        let rematch_y = menu_y - 10.0 * scale - btn_h;

        Self {
            modal_bounds,
            rematch_btn_bounds: Rect::new(btn_x, rematch_y, btn_w, btn_h),
            menu_btn_bounds: Rect::new(btn_x, menu_y, btn_w, btn_h),
        }
    }
}

pub struct Screens;

impl Screens {
    pub fn draw_title_screen(
        state: &mut GameState,
        screen_w: f32,
        screen_h: f32,
        scale: f32,
        font: Option<&Font>,
        hero_texture: Option<&Texture2D>,
    ) -> Option<SoundTrigger> {
        let mut sound_trigger = None;
        let t = get_time() as f32;

        // Dark Atmospheric Background Plate
        draw_rectangle(
            0.0,
            0.0,
            screen_w,
            screen_h,
            Color::from_rgba(11, 17, 24, 255),
        );

        // Ambient radial glows (Fox orange left, Hound blue right)
        let pulse = (t * 2.0).sin() * 0.5 + 0.5;
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;
        draw_circle(
            center_x - 90.0 * scale,
            center_y - 120.0 * scale,
            240.0 * scale,
            Color::from_rgba(230, 81, 0, 16 + (pulse * 10.0) as u8),
        );
        draw_circle(
            center_x + 90.0 * scale,
            center_y - 120.0 * scale,
            240.0 * scale,
            Color::from_rgba(25, 118, 210, 14 + ((1.0 - pulse) * 10.0) as u8),
        );

        let hero_aspect = hero_texture
            .map(|t| t.width() / t.height().max(1.0))
            .unwrap_or(16.0 / 9.0);
        let layout = TitleScreenLayout::compute(
            screen_w,
            screen_h,
            scale,
            hero_texture.is_some(),
            hero_aspect,
        );

        // Card Background with Sleek Border
        draw_rectangle(
            layout.card_bounds.x,
            layout.card_bounds.y,
            layout.card_bounds.w,
            layout.card_bounds.h,
            Color::from_rgba(18, 28, 42, 240),
        );
        draw_rectangle_lines(
            layout.card_bounds.x,
            layout.card_bounds.y,
            layout.card_bounds.w,
            layout.card_bounds.h,
            2.0 * scale,
            Color::from_rgba(255, 255, 255, 35),
        );

        if layout.is_landscape {
            let left = layout.left_column;
            let right = layout.right_column;

            // 1. Title & Subtitle in Left Column
            let title_text = &state.locales.title_screen.title;
            let title_font_size = (24.0 * scale) as u16;
            let title_dims = measure_text_styled(title_text, title_font_size, font);
            draw_text_styled(
                title_text,
                left.x + (left.w - title_dims.width) / 2.0,
                left.y + title_dims.height / 1.2,
                title_font_size,
                Color::from_rgba(255, 224, 130, 255),
                font,
            );

            let subtitle_text = &state.locales.title_screen.subtitle;
            let sub_font_size = (12.0 * scale) as u16;
            let sub_dims = measure_text_styled(subtitle_text, sub_font_size, font);
            draw_text_styled(
                subtitle_text,
                left.x + (left.w - sub_dims.width) / 2.0,
                left.y + 26.0 * scale + sub_dims.height / 1.2,
                sub_font_size,
                Color::from_rgba(176, 190, 197, 255),
                font,
            );

            // 2. Hero Artwork Banner in Left Column
            if let (Some(tex), Some(hb)) = (hero_texture, layout.hero_bounds) {
                draw_rectangle(hb.x, hb.y, hb.w, hb.h, Color::from_rgba(10, 16, 26, 255));
                draw_texture_ex(
                    tex,
                    hb.x,
                    hb.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(hb.w, hb.h)),
                        ..Default::default()
                    },
                );
                draw_rectangle_lines(
                    hb.x,
                    hb.y,
                    hb.w,
                    hb.h,
                    1.5 * scale,
                    Color::from_rgba(255, 255, 255, 60),
                );
            }

            // Divider between columns
            let div_x = left.x + left.w + (right.x - (left.x + left.w)) / 2.0;
            draw_line(
                div_x,
                layout.card_bounds.y + 16.0 * scale,
                div_x,
                layout.card_bounds.y + layout.card_bounds.h - 16.0 * scale,
                1.0 * scale,
                Color::from_rgba(255, 255, 255, 25),
            );

            // 3. Right Column: Faction Selection
            let role_label = &state.locales.title_screen.choose_faction;
            let label_size = (12.0 * scale) as u16;
            let label_dims = measure_text_styled(role_label, label_size, font);
            draw_text_styled(
                role_label,
                right.x + (right.w - label_dims.width) / 2.0,
                layout.fox_btn_bounds.y - 6.0 * scale,
                label_size,
                Color::from_rgba(144, 164, 174, 255),
                font,
            );

            let is_fox = state.player_faction == Faction::Fox;
            let fox_clicked = Self::draw_selectable_button(&SelectableButtonConfig {
                bounds: layout.fox_btn_bounds,
                title: &state.locales.title_screen.fox_title,
                subtitle: &state.locales.title_screen.fox_subtitle,
                is_selected: is_fox,
                accent_color: Color::from_rgba(230, 81, 0, 255),
                scale,
                font,
            });
            if fox_clicked {
                state.player_faction = Faction::Fox;
                sound_trigger = Some(SoundTrigger::ButtonClick);
            }

            let is_hound = state.player_faction == Faction::Hounds;
            let hound_clicked = Self::draw_selectable_button(&SelectableButtonConfig {
                bounds: layout.hounds_btn_bounds,
                title: &state.locales.title_screen.hounds_title,
                subtitle: &state.locales.title_screen.hounds_subtitle,
                is_selected: is_hound,
                accent_color: Color::from_rgba(25, 118, 210, 255),
                scale,
                font,
            });
            if hound_clicked {
                state.player_faction = Faction::Hounds;
                sound_trigger = Some(SoundTrigger::ButtonClick);
            }

            // 4. Right Column: AI Difficulty
            let diff_label = &state.locales.title_screen.ai_difficulty;
            let diff_dims = measure_text_styled(diff_label, label_size, font);
            draw_text_styled(
                diff_label,
                right.x + (right.w - diff_dims.width) / 2.0,
                layout.difficulty_btn_bounds[0].y - 6.0 * scale,
                label_size,
                Color::from_rgba(144, 164, 174, 255),
                font,
            );

            let difficulties = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
            for (idx, &diff) in difficulties.iter().enumerate() {
                let is_sel = state.difficulty == diff;
                let clicked = Self::draw_selectable_button(&SelectableButtonConfig {
                    bounds: layout.difficulty_btn_bounds[idx],
                    title: diff.localized_name(state.locales),
                    subtitle: "",
                    is_selected: is_sel,
                    accent_color: Color::from_rgba(78, 52, 46, 255),
                    scale,
                    font,
                });
                if clicked {
                    state.difficulty = diff;
                    sound_trigger = Some(SoundTrigger::ButtonClick);
                }
            }

            // 5. Right Column: Start Match Button
            let start_clicked = Self::draw_action_button(&ActionButtonConfig {
                bounds: layout.start_btn_bounds,
                text: &state.locales.title_screen.start_match,
                normal_color: Color::from_rgba(46, 125, 50, 255),
                hover_color: Color::from_rgba(76, 175, 80, 255),
                scale,
                font,
            });
            if start_clicked {
                state.start_game(state.player_faction, state.difficulty);
                sound_trigger = Some(SoundTrigger::ButtonClick);
            }
        } else {
            let card_x = layout.card_bounds.x;
            let card_y = layout.card_bounds.y;
            let card_w = layout.card_bounds.w;
            let mut curr_y = card_y + 20.0 * scale;

            // 1. Game Title & Subtitle
            let title_text = &state.locales.title_screen.title;
            let title_font_size = (30.0 * scale) as u16;
            let title_dims = measure_text_styled(title_text, title_font_size, font);
            draw_text_styled(
                title_text,
                center_x - title_dims.width / 2.0,
                curr_y + title_dims.height / 1.2,
                title_font_size,
                Color::from_rgba(255, 224, 130, 255),
                font,
            );
            curr_y += 30.0 * scale;

            let subtitle_text = &state.locales.title_screen.subtitle;
            let sub_font_size = (13.0 * scale) as u16;
            let sub_dims = measure_text_styled(subtitle_text, sub_font_size, font);
            draw_text_styled(
                subtitle_text,
                center_x - sub_dims.width / 2.0,
                curr_y + sub_dims.height / 1.2,
                sub_font_size,
                Color::from_rgba(176, 190, 197, 255),
                font,
            );
            curr_y += 18.0 * scale;

            // 2. Character Artwork Hero Banner
            if let (Some(tex), Some(hb)) = (hero_texture, layout.hero_bounds) {
                draw_rectangle(hb.x, hb.y, hb.w, hb.h, Color::from_rgba(10, 16, 26, 255));
                draw_texture_ex(
                    tex,
                    hb.x,
                    hb.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(hb.w, hb.h)),
                        ..Default::default()
                    },
                );
                draw_rectangle_lines(
                    hb.x,
                    hb.y,
                    hb.w,
                    hb.h,
                    1.5 * scale,
                    Color::from_rgba(255, 255, 255, 60),
                );
                curr_y += hb.h + 16.0 * scale;
            } else {
                draw_line(
                    card_x + 30.0 * scale,
                    curr_y,
                    card_x + card_w - 30.0 * scale,
                    curr_y,
                    1.0 * scale,
                    Color::from_rgba(255, 255, 255, 25),
                );
                curr_y += 16.0 * scale;
            }

            // 3. Select Faction Header
            let role_label = &state.locales.title_screen.choose_faction;
            let label_size = (12.0 * scale) as u16;
            let label_dims = measure_text_styled(role_label, label_size, font);
            draw_text_styled(
                role_label,
                center_x - label_dims.width / 2.0,
                curr_y + label_dims.height / 1.2,
                label_size,
                Color::from_rgba(144, 164, 174, 255),
                font,
            );

            let is_fox = state.player_faction == Faction::Fox;
            let fox_clicked = Self::draw_selectable_button(&SelectableButtonConfig {
                bounds: layout.fox_btn_bounds,
                title: &state.locales.title_screen.fox_title,
                subtitle: &state.locales.title_screen.fox_subtitle,
                is_selected: is_fox,
                accent_color: Color::from_rgba(230, 81, 0, 255),
                scale,
                font,
            });
            if fox_clicked {
                state.player_faction = Faction::Fox;
                sound_trigger = Some(SoundTrigger::ButtonClick);
            }

            let is_hound = state.player_faction == Faction::Hounds;
            let hound_clicked = Self::draw_selectable_button(&SelectableButtonConfig {
                bounds: layout.hounds_btn_bounds,
                title: &state.locales.title_screen.hounds_title,
                subtitle: &state.locales.title_screen.hounds_subtitle,
                is_selected: is_hound,
                accent_color: Color::from_rgba(25, 118, 210, 255),
                scale,
                font,
            });
            if hound_clicked {
                state.player_faction = Faction::Hounds;
                sound_trigger = Some(SoundTrigger::ButtonClick);
            }

            // 4. AI Difficulty Selector
            let diff_label = &state.locales.title_screen.ai_difficulty;
            let diff_label_dims = measure_text_styled(diff_label, label_size, font);
            draw_text_styled(
                diff_label,
                center_x - diff_label_dims.width / 2.0,
                layout.difficulty_btn_bounds[0].y - 6.0 * scale,
                label_size,
                Color::from_rgba(144, 164, 174, 255),
                font,
            );

            let difficulties = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
            for (idx, &diff) in difficulties.iter().enumerate() {
                let is_sel = state.difficulty == diff;
                let clicked = Self::draw_selectable_button(&SelectableButtonConfig {
                    bounds: layout.difficulty_btn_bounds[idx],
                    title: diff.localized_name(state.locales),
                    subtitle: "",
                    is_selected: is_sel,
                    accent_color: Color::from_rgba(78, 52, 46, 255),
                    scale,
                    font,
                });
                if clicked {
                    state.difficulty = diff;
                    sound_trigger = Some(SoundTrigger::ButtonClick);
                }
            }

            // 5. Start Match Button
            let start_clicked = Self::draw_action_button(&ActionButtonConfig {
                bounds: layout.start_btn_bounds,
                text: &state.locales.title_screen.start_match,
                normal_color: Color::from_rgba(46, 125, 50, 255),
                hover_color: Color::from_rgba(76, 175, 80, 255),
                scale,
                font,
            });
            if start_clicked {
                state.start_game(state.player_faction, state.difficulty);
                sound_trigger = Some(SoundTrigger::ButtonClick);
            }
        }

        sound_trigger
    }

    pub fn draw_ingame_hud(
        state: &mut GameState,
        screen_w: f32,
        _screen_h: f32,
        scale: f32,
        is_muted: bool,
        font: Option<&Font>,
    ) -> (Option<SoundTrigger>, bool) {
        let mut sound_trigger = None;
        let mut toggle_mute_requested = false;

        let hud_h = 56.0 * scale;
        let pad = 12.0 * scale;

        // Semi-transparent Top HUD Banner
        draw_rectangle(0.0, 0.0, screen_w, hud_h, Color::from_rgba(11, 17, 24, 215));
        draw_line(
            0.0,
            hud_h,
            screen_w,
            hud_h,
            1.0 * scale,
            Color::from_rgba(255, 255, 255, 25),
        );

        // 1. Turn Badge Pill (Left)
        let is_ai = state.is_ai_turn();
        let (turn_icon, turn_title, badge_color) = match state.current_turn {
            Faction::Fox => (
                "🦊",
                if is_ai {
                    &state.locales.hud.thinking
                } else {
                    &state.locales.hud.fox_turn
                },
                Color::from_rgba(230, 81, 0, 220),
            ),
            Faction::Hounds => (
                "🐶",
                if is_ai {
                    &state.locales.hud.thinking
                } else {
                    &state.locales.hud.hounds_turn
                },
                Color::from_rgba(25, 118, 210, 220),
            ),
        };

        let badge_text = format!("{} {}", turn_icon, turn_title);
        let badge_font = (13.0 * scale) as u16;
        let badge_dims = measure_text_styled(&badge_text, badge_font, font);
        let badge_w = badge_dims.width + 20.0 * scale;
        let badge_h = 36.0 * scale;
        let badge_y = (hud_h - badge_h) / 2.0;
        let badge_x = pad;

        draw_rectangle(badge_x, badge_y, badge_w, badge_h, badge_color);
        draw_rectangle_lines(
            badge_x,
            badge_y,
            badge_w,
            badge_h,
            1.5 * scale,
            Color::from_rgba(255, 255, 255, 80),
        );

        draw_text_styled(
            &badge_text,
            badge_x + (badge_w - badge_dims.width) / 2.0,
            badge_y + badge_h / 2.0 + badge_dims.height / 3.0,
            badge_font,
            WHITE,
            font,
        );

        // 2. Right Action Buttons: Mute, Restart, Menu
        let btn_size = 36.0 * scale;
        let btn_y = (hud_h - btn_size) / 2.0;
        let btn_spacing = 8.0 * scale;

        // Menu Button
        let menu_x = screen_w - pad - btn_size;
        let menu_clicked = Self::draw_icon_button(&IconButtonConfig {
            bounds: Rect::new(menu_x, btn_y, btn_size, btn_size),
            icon: "🏠",
            scale,
            font,
        });
        if menu_clicked {
            state.phase = GamePhase::TitleScreen;
            sound_trigger = Some(SoundTrigger::ButtonClick);
        }

        // Restart Button
        let restart_x = menu_x - btn_size - btn_spacing;
        let restart_clicked = Self::draw_icon_button(&IconButtonConfig {
            bounds: Rect::new(restart_x, btn_y, btn_size, btn_size),
            icon: "🔄",
            scale,
            font,
        });
        if restart_clicked {
            state.reset_board();
            sound_trigger = Some(SoundTrigger::ButtonClick);
        }

        // Mute Button
        let mute_x = restart_x - btn_size - btn_spacing;
        let mute_icon = if is_muted { "🔇" } else { "🔊" };
        let mute_clicked = Self::draw_icon_button(&IconButtonConfig {
            bounds: Rect::new(mute_x, btn_y, btn_size, btn_size),
            icon: mute_icon,
            scale,
            font,
        });
        if mute_clicked {
            toggle_mute_requested = true;
            sound_trigger = Some(SoundTrigger::ButtonClick);
        }

        // 3. Turn Counter (Center - safely positioned without overlapping badge or buttons)
        let turn_str = state.locales.hud.format_turn(state.turn_count);
        let turn_font = (14.0 * scale) as u16;
        let turn_dims = measure_text_styled(&turn_str, turn_font, font);
        let left_edge = badge_x + badge_w + 10.0 * scale;
        let right_edge = mute_x - 10.0 * scale;

        if right_edge > left_edge + turn_dims.width {
            let ideal_x = (screen_w - turn_dims.width) / 2.0;
            let clamped_x = ideal_x.clamp(left_edge, right_edge - turn_dims.width);
            draw_text_styled(
                &turn_str,
                clamped_x,
                hud_h / 2.0 + turn_dims.height / 3.0,
                turn_font,
                Color::from_rgba(207, 216, 220, 255),
                font,
            );
        }

        (sound_trigger, toggle_mute_requested)
    }

    pub fn draw_game_over_modal(
        state: &mut GameState,
        screen_w: f32,
        screen_h: f32,
        scale: f32,
        font: Option<&Font>,
    ) -> Option<SoundTrigger> {
        let mut sound_trigger = None;

        // Dim background overlay
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::from_rgba(0, 0, 0, 185));

        let layout = GameOverModalLayout::compute(screen_w, screen_h, scale);
        let modal = layout.modal_bounds;
        let center_x = screen_w / 2.0;

        // Modal Box
        draw_rectangle(
            modal.x,
            modal.y,
            modal.w,
            modal.h,
            Color::from_rgba(18, 28, 42, 250),
        );
        draw_rectangle_lines(
            modal.x,
            modal.y,
            modal.w,
            modal.h,
            2.0 * scale,
            Color::from_rgba(255, 255, 255, 50),
        );

        let mut curr_y = modal.y + (28.0 * scale).min(modal.h * 0.1);

        let player_won = matches!(
            (state.result, state.player_faction),
            (GameResult::FoxWon, Faction::Fox) | (GameResult::HoundsWon, Faction::Hounds)
        );

        // Header Title
        let header_text = if player_won {
            &state.locales.game_over.victory
        } else {
            &state.locales.game_over.defeat
        };
        let header_color = if player_won {
            Color::from_rgba(76, 175, 80, 255)
        } else {
            Color::from_rgba(229, 57, 53, 255)
        };
        let header_font = (26.0 * scale) as u16;
        let header_dims = measure_text_styled(header_text, header_font, font);
        draw_text_styled(
            header_text,
            center_x - header_dims.width / 2.0,
            curr_y + header_dims.height / 1.2,
            header_font,
            header_color,
            font,
        );
        curr_y += 30.0 * scale;

        // Detail Message
        let msg = match state.result {
            GameResult::FoxWon => &state.locales.game_over.fox_won_msg,
            GameResult::HoundsWon => &state.locales.game_over.hounds_won_msg,
            GameResult::Ongoing => "",
        };
        let msg_font = (13.0 * scale) as u16;
        let msg_dims = measure_text_styled(msg, msg_font, font);
        draw_text_styled(
            msg,
            center_x - msg_dims.width / 2.0,
            curr_y + msg_dims.height / 1.2,
            msg_font,
            Color::from_rgba(224, 224, 224, 255),
            font,
        );
        curr_y += 24.0 * scale;

        // Stats line
        let stats = state.locales.game_over.format_stats(
            state.turn_count,
            state.difficulty.localized_name(state.locales),
        );
        let stats_font = (12.0 * scale) as u16;
        let stats_dims = measure_text_styled(&stats, stats_font, font);
        draw_text_styled(
            &stats,
            center_x - stats_dims.width / 2.0,
            curr_y + stats_dims.height / 1.2,
            stats_font,
            Color::from_rgba(158, 158, 158, 255),
            font,
        );

        // Play Again Button
        let rematch_clicked = Self::draw_action_button(&ActionButtonConfig {
            bounds: layout.rematch_btn_bounds,
            text: &state.locales.game_over.play_again,
            normal_color: Color::from_rgba(25, 118, 210, 255),
            hover_color: Color::from_rgba(66, 165, 245, 255),
            scale,
            font,
        });
        if rematch_clicked {
            state.reset_board();
            state.phase = GamePhase::Playing;
            sound_trigger = Some(SoundTrigger::ButtonClick);
        }

        // Main Menu Button
        let menu_clicked = Self::draw_action_button(&ActionButtonConfig {
            bounds: layout.menu_btn_bounds,
            text: &state.locales.game_over.main_menu,
            normal_color: Color::from_rgba(55, 71, 79, 255),
            hover_color: Color::from_rgba(96, 125, 139, 255),
            scale,
            font,
        });
        if menu_clicked {
            state.phase = GamePhase::TitleScreen;
            sound_trigger = Some(SoundTrigger::ButtonClick);
        }

        sound_trigger
    }

    fn draw_selectable_button(cfg: &SelectableButtonConfig) -> bool {
        let mouse_pos = Vec2::from(mouse_position());
        let is_hovered = cfg.bounds.contains(mouse_pos);
        let clicked = is_hovered && is_mouse_button_released(MouseButton::Left);

        let bg_color = if cfg.is_selected {
            cfg.accent_color
        } else if is_hovered {
            Color::from_rgba(38, 50, 68, 255)
        } else {
            Color::from_rgba(26, 35, 50, 255)
        };

        draw_rectangle(
            cfg.bounds.x,
            cfg.bounds.y,
            cfg.bounds.w,
            cfg.bounds.h,
            bg_color,
        );
        let border_color = if cfg.is_selected {
            Color::from_rgba(255, 255, 255, 200)
        } else if is_hovered {
            Color::from_rgba(255, 255, 255, 80)
        } else {
            Color::from_rgba(255, 255, 255, 25)
        };
        draw_rectangle_lines(
            cfg.bounds.x,
            cfg.bounds.y,
            cfg.bounds.w,
            cfg.bounds.h,
            1.5 * cfg.scale,
            border_color,
        );

        let title_size = (14.0 * cfg.scale) as u16;
        let title_dims = measure_text_styled(cfg.title, title_size, cfg.font);
        let text_y = if cfg.subtitle.is_empty() {
            cfg.bounds.y + cfg.bounds.h / 2.0 + title_dims.height / 3.0
        } else {
            cfg.bounds.y + cfg.bounds.h / 2.0 - 2.0 * cfg.scale
        };

        draw_text_styled(
            cfg.title,
            cfg.bounds.x + (cfg.bounds.w - title_dims.width) / 2.0,
            text_y,
            title_size,
            WHITE,
            cfg.font,
        );

        if !cfg.subtitle.is_empty() {
            let sub_size = (11.0 * cfg.scale) as u16;
            let sub_dims = measure_text_styled(cfg.subtitle, sub_size, cfg.font);
            draw_text_styled(
                cfg.subtitle,
                cfg.bounds.x + (cfg.bounds.w - sub_dims.width) / 2.0,
                text_y + 16.0 * cfg.scale,
                sub_size,
                Color::from_rgba(207, 216, 220, 220),
                cfg.font,
            );
        }

        clicked
    }

    fn draw_action_button(cfg: &ActionButtonConfig) -> bool {
        let mouse_pos = Vec2::from(mouse_position());
        let is_hovered = cfg.bounds.contains(mouse_pos);
        let clicked = is_hovered && is_mouse_button_released(MouseButton::Left);

        let bg_color = if is_hovered {
            cfg.hover_color
        } else {
            cfg.normal_color
        };
        draw_rectangle(
            cfg.bounds.x,
            cfg.bounds.y,
            cfg.bounds.w,
            cfg.bounds.h,
            bg_color,
        );
        draw_rectangle_lines(
            cfg.bounds.x,
            cfg.bounds.y,
            cfg.bounds.w,
            cfg.bounds.h,
            1.5 * cfg.scale,
            Color::from_rgba(255, 255, 255, 80),
        );

        let font_size = (15.0 * cfg.scale) as u16;
        let dims = measure_text_styled(cfg.text, font_size, cfg.font);
        draw_text_styled(
            cfg.text,
            cfg.bounds.x + (cfg.bounds.w - dims.width) / 2.0,
            cfg.bounds.y + cfg.bounds.h / 2.0 + dims.height / 3.0,
            font_size,
            WHITE,
            cfg.font,
        );

        clicked
    }

    fn draw_icon_button(cfg: &IconButtonConfig) -> bool {
        let mouse_pos = Vec2::from(mouse_position());
        let is_hovered = cfg.bounds.contains(mouse_pos);
        let clicked = is_hovered && is_mouse_button_released(MouseButton::Left);

        let bg_color = if is_hovered {
            Color::from_rgba(45, 60, 80, 255)
        } else {
            Color::from_rgba(26, 35, 50, 220)
        };
        draw_rectangle(
            cfg.bounds.x,
            cfg.bounds.y,
            cfg.bounds.w,
            cfg.bounds.h,
            bg_color,
        );
        draw_rectangle_lines(
            cfg.bounds.x,
            cfg.bounds.y,
            cfg.bounds.w,
            cfg.bounds.h,
            1.0 * cfg.scale,
            Color::from_rgba(255, 255, 255, 40),
        );

        let font_size = (16.0 * cfg.scale) as u16;
        let dims = measure_text_styled(cfg.icon, font_size, cfg.font);
        draw_text_styled(
            cfg.icon,
            cfg.bounds.x + (cfg.bounds.w - dims.width) / 2.0,
            cfg.bounds.y + cfg.bounds.h / 2.0 + dims.height / 3.0,
            font_size,
            WHITE,
            cfg.font,
        );

        clicked
    }
}
