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

pub struct Screens;

impl Screens {
    pub fn draw_title_screen(
        state: &mut GameState,
        screen_w: f32,
        screen_h: f32,
        scale: f32,
        font: Option<&Font>,
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

        // Subtle ambient radial glow
        let pulse = (t * 2.0).sin() * 0.5 + 0.5;
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;
        draw_circle(
            center_x,
            center_y - 60.0 * scale,
            220.0 * scale,
            Color::from_rgba(230, 81, 0, 15 + (pulse * 10.0) as u8),
        );

        // Card Container Dimensions
        let card_w = (screen_w - 32.0 * scale)
            .min(440.0 * scale)
            .max(280.0 * scale);
        let card_h = (screen_h - 48.0 * scale)
            .min(560.0 * scale)
            .max(360.0 * scale);
        let card_x = (screen_w - card_w) / 2.0;
        let card_y = (screen_h - card_h) / 2.0;

        // Card Background with Sleek Border
        draw_rectangle(
            card_x,
            card_y,
            card_w,
            card_h,
            Color::from_rgba(18, 28, 42, 235),
        );
        draw_rectangle_lines(
            card_x,
            card_y,
            card_w,
            card_h,
            2.0 * scale,
            Color::from_rgba(255, 255, 255, 30),
        );

        let mut curr_y = card_y + 32.0 * scale;

        // 1. Game Title & Subtitle
        let title_text = "FOX & HOUNDS";
        let title_font_size = (34.0 * scale) as u16;
        let title_dims = measure_text_styled(title_text, title_font_size, font);
        draw_text_styled(
            title_text,
            center_x - title_dims.width / 2.0,
            curr_y,
            title_font_size,
            Color::from_rgba(255, 224, 130, 255),
            font,
        );
        curr_y += 24.0 * scale;

        let subtitle_text = "Лиса и Гончие • Graph Strategy";
        let sub_font_size = (15.0 * scale) as u16;
        let sub_dims = measure_text_styled(subtitle_text, sub_font_size, font);
        draw_text_styled(
            subtitle_text,
            center_x - sub_dims.width / 2.0,
            curr_y,
            sub_font_size,
            Color::from_rgba(176, 190, 197, 255),
            font,
        );
        curr_y += 38.0 * scale;

        // Divider
        draw_line(
            card_x + 30.0 * scale,
            curr_y,
            card_x + card_w - 30.0 * scale,
            curr_y,
            1.0 * scale,
            Color::from_rgba(255, 255, 255, 25),
        );
        curr_y += 28.0 * scale;

        // 2. Select Faction Header
        let role_label = "CHOOSE YOUR FACTION";
        let label_size = (13.0 * scale) as u16;
        let label_dims = measure_text_styled(role_label, label_size, font);
        draw_text_styled(
            role_label,
            center_x - label_dims.width / 2.0,
            curr_y,
            label_size,
            Color::from_rgba(144, 164, 174, 255),
            font,
        );
        curr_y += 18.0 * scale;

        // Faction Buttons (Fox / Hounds)
        let btn_w = (card_w - 50.0 * scale) / 2.0;
        let btn_h = 56.0 * scale;
        let fox_btn_x = card_x + 20.0 * scale;
        let hound_btn_x = fox_btn_x + btn_w + 10.0 * scale;

        // Fox Button
        let is_fox = state.player_faction == Faction::Fox;
        let (fox_clicked, _) = Self::draw_selectable_button(&SelectableButtonConfig {
            bounds: Rect::new(fox_btn_x, curr_y, btn_w, btn_h),
            title: "🦊 The Fox",
            subtitle: "Infiltrate Coop",
            is_selected: is_fox,
            accent_color: Color::from_rgba(230, 81, 0, 255),
            scale,
            font,
        });
        if fox_clicked {
            state.player_faction = Faction::Fox;
            sound_trigger = Some(SoundTrigger::ButtonClick);
        }

        // Hounds Button
        let is_hound = state.player_faction == Faction::Hounds;
        let (hound_clicked, _) = Self::draw_selectable_button(&SelectableButtonConfig {
            bounds: Rect::new(hound_btn_x, curr_y, btn_w, btn_h),
            title: "🐶 The Hounds",
            subtitle: "Trap the Fox",
            is_selected: is_hound,
            accent_color: Color::from_rgba(25, 118, 210, 255),
            scale,
            font,
        });
        if hound_clicked {
            state.player_faction = Faction::Hounds;
            sound_trigger = Some(SoundTrigger::ButtonClick);
        }
        curr_y += btn_h + 30.0 * scale;

        // 3. AI Difficulty Selector
        let diff_label = "AI DIFFICULTY";
        let diff_label_dims = measure_text_styled(diff_label, label_size, font);
        draw_text_styled(
            diff_label,
            center_x - diff_label_dims.width / 2.0,
            curr_y,
            label_size,
            Color::from_rgba(144, 164, 174, 255),
            font,
        );
        curr_y += 18.0 * scale;

        let diff_btn_w = (card_w - 60.0 * scale) / 3.0;
        let diff_btn_h = 42.0 * scale;
        let difficulties = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];

        for (idx, &diff) in difficulties.iter().enumerate() {
            let bx = card_x + 20.0 * scale + (diff_btn_w + 10.0 * scale) * idx as f32;
            let is_sel = state.difficulty == diff;
            let (clicked, _) = Self::draw_selectable_button(&SelectableButtonConfig {
                bounds: Rect::new(bx, curr_y, diff_btn_w, diff_btn_h),
                title: diff.name(),
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
        curr_y += diff_btn_h + 36.0 * scale;

        // 4. Start Match Button
        let start_w = card_w - 40.0 * scale;
        let start_h = 52.0 * scale;
        let start_x = card_x + 20.0 * scale;
        let (start_clicked, _) = Self::draw_action_button(&ActionButtonConfig {
            bounds: Rect::new(start_x, curr_y, start_w, start_h),
            text: "START MATCH",
            normal_color: Color::from_rgba(46, 125, 50, 255),
            hover_color: Color::from_rgba(76, 175, 80, 255),
            scale,
            font,
        });
        if start_clicked {
            state.start_game(state.player_faction, state.difficulty);
            sound_trigger = Some(SoundTrigger::ButtonClick);
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
                if is_ai { "Thinking..." } else { "Fox Turn" },
                Color::from_rgba(230, 81, 0, 220),
            ),
            Faction::Hounds => (
                "🐶",
                if is_ai { "Thinking..." } else { "Hounds Turn" },
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
        let (menu_clicked, _) = Self::draw_icon_button(&IconButtonConfig {
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
        let (restart_clicked, _) = Self::draw_icon_button(&IconButtonConfig {
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
        let (mute_clicked, _) = Self::draw_icon_button(&IconButtonConfig {
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
        let turn_str = format!("Turn {}", state.turn_count);
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

        let modal_w = (screen_w - 32.0 * scale)
            .min(380.0 * scale)
            .max(280.0 * scale);
        let modal_h = (screen_h - 48.0 * scale)
            .min(340.0 * scale)
            .max(260.0 * scale);
        let modal_x = (screen_w - modal_w) / 2.0;
        let modal_y = (screen_h - modal_h) / 2.0;
        let center_x = screen_w / 2.0;

        // Modal Box
        draw_rectangle(
            modal_x,
            modal_y,
            modal_w,
            modal_h,
            Color::from_rgba(18, 28, 42, 250),
        );
        draw_rectangle_lines(
            modal_x,
            modal_y,
            modal_w,
            modal_h,
            2.0 * scale,
            Color::from_rgba(255, 255, 255, 50),
        );

        let mut curr_y = modal_y + 40.0 * scale;

        let player_won = matches!(
            (state.result, state.player_faction),
            (GameResult::FoxWon, Faction::Fox) | (GameResult::HoundsWon, Faction::Hounds)
        );

        // Header Title
        let header_text = if player_won { "VICTORY!" } else { "DEFEAT!" };
        let header_color = if player_won {
            Color::from_rgba(76, 175, 80, 255)
        } else {
            Color::from_rgba(229, 57, 53, 255)
        };
        let header_font = (28.0 * scale) as u16;
        let header_dims = measure_text_styled(header_text, header_font, font);
        draw_text_styled(
            header_text,
            center_x - header_dims.width / 2.0,
            curr_y,
            header_font,
            header_color,
            font,
        );
        curr_y += 32.0 * scale;

        // Detail Message
        let msg = match state.result {
            GameResult::FoxWon => "🦊 The Fox reached the Chicken Coop!",
            GameResult::HoundsWon => "🐶 The Hounds encircled and trapped the Fox!",
            GameResult::Ongoing => "",
        };
        let msg_font = (14.0 * scale) as u16;
        let msg_dims = measure_text_styled(msg, msg_font, font);
        draw_text_styled(
            msg,
            center_x - msg_dims.width / 2.0,
            curr_y,
            msg_font,
            Color::from_rgba(224, 224, 224, 255),
            font,
        );
        curr_y += 26.0 * scale;

        // Stats line
        let stats = format!(
            "Completed in {} turns • {} AI",
            state.turn_count,
            state.difficulty.name()
        );
        let stats_font = (13.0 * scale) as u16;
        let stats_dims = measure_text_styled(&stats, stats_font, font);
        draw_text_styled(
            &stats,
            center_x - stats_dims.width / 2.0,
            curr_y,
            stats_font,
            Color::from_rgba(158, 158, 158, 255),
            font,
        );
        curr_y += 42.0 * scale;

        // Play Again Button
        let btn_w = modal_w - 40.0 * scale;
        let btn_h = 48.0 * scale;
        let btn_x = modal_x + 20.0 * scale;

        let (rematch_clicked, _) = Self::draw_action_button(&ActionButtonConfig {
            bounds: Rect::new(btn_x, curr_y, btn_w, btn_h),
            text: "PLAY AGAIN",
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
        curr_y += btn_h + 12.0 * scale;

        // Main Menu Button
        let (menu_clicked, _) = Self::draw_action_button(&ActionButtonConfig {
            bounds: Rect::new(btn_x, curr_y, btn_w, btn_h),
            text: "MAIN MENU",
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

    fn draw_selectable_button(cfg: &SelectableButtonConfig) -> (bool, bool) {
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

        (clicked, is_hovered)
    }

    fn draw_action_button(cfg: &ActionButtonConfig) -> (bool, bool) {
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

        (clicked, is_hovered)
    }

    fn draw_icon_button(cfg: &IconButtonConfig) -> (bool, bool) {
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

        (clicked, is_hovered)
    }
}
