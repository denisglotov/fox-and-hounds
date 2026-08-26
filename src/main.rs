use fox_and_hounds::audio::SoundManager;
use fox_and_hounds::game::level::{BOARD_IMAGE_HEIGHT, BOARD_IMAGE_WIDTH};
use fox_and_hounds::game::state::{GamePhase, GameResult, GameState};
use fox_and_hounds::ui::board_view::BoardView;
use fox_and_hounds::ui::camera::ViewportCamera;
use fox_and_hounds::ui::fx::FxManager;
use fox_and_hounds::ui::screens::Screens;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Fox and Hounds - Tactical Graph Strategy".to_string(),
        window_width: 960,
        window_height: 1360,
        high_dpi: true,
        window_resizable: true,
        fullscreen: false,
        sample_count: if cfg!(target_os = "android") { 0 } else { 4 },
        ..Default::default()
    }
}

fn compute_board_layout(screen_w: f32, screen_h: f32, _scale: f32) -> (Rect, f32, Vec2) {
    let viewport_rect = Rect::new(0.0, 0.0, screen_w, screen_h.max(1.0));

    let board_scale = (viewport_rect.h / BOARD_IMAGE_HEIGHT).max(0.1);

    let board_size = Vec2::new(
        BOARD_IMAGE_WIDTH * board_scale,
        BOARD_IMAGE_HEIGHT * board_scale,
    );

    (viewport_rect, board_scale, board_size)
}

#[macroquad::main(window_conf)]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/NotoSansEmoji.ttf")).ok();
    let character_texture = {
        let tex = Texture2D::from_file_with_format(
            include_bytes!("../assets/fox_and_hounds.png"),
            Some(ImageFormat::Png),
        );
        tex.set_filter(FilterMode::Linear);
        Some(tex)
    };
    let mut sound_manager = SoundManager::new().await;
    let mut state = GameState::new();
    let mut board_view = BoardView::new(font.clone()).await;
    let mut camera = ViewportCamera::new();
    let mut fx_manager = FxManager::new();

    let mut last_result = GameResult::Ongoing;

    loop {
        let dt = get_frame_time().min(0.04);
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Responsive UI scale factor (buttons, HUD, text, cards)
        #[cfg(target_os = "android")]
        let scale = {
            // Android uses physical pixels with high_dpi: true (typically 1080x2400 on phones)
            let portrait_w = screen_w.min(screen_h);
            (portrait_w / 380.0).clamp(1.0, 4.0)
        };
        #[cfg(not(target_os = "android"))]
        let scale = {
            let base_scale = if screen_w > screen_h {
                (screen_w / 850.0).min(screen_h / 520.0)
            } else {
                (screen_w / 520.0).min(screen_h / 850.0)
            };
            base_scale.clamp(0.65, 2.5)
        };

        // 1. Update Game State (animations, AI thinking)
        let state_sound = state.update(dt);
        if let Some(snd) = state_sound {
            sound_manager.play(snd);
        }

        // Spawn confetti when match concludes with player victory
        if state.result != last_result {
            if state.result != GameResult::Ongoing {
                fx_manager.spawn_victory_burst(Vec2::new(screen_w / 2.0, screen_h / 3.0), 70);
            }
            last_result = state.result;
        }

        // 2. Render Current Game Phase
        match state.phase {
            GamePhase::TitleScreen => {
                camera.reset_pan();
                let title_sound = Screens::draw_title_screen(
                    &mut state,
                    screen_w,
                    screen_h,
                    scale,
                    font.as_ref(),
                    character_texture.as_ref(),
                );
                if let Some(snd) = title_sound {
                    sound_manager.play(snd);
                }
                if state.phase == GamePhase::Playing {
                    let (viewport_rect, board_scale, board_size) =
                        compute_board_layout(screen_w, screen_h, scale);
                    camera.start_coop_fox_intro(viewport_rect, board_size, board_scale, scale, 2.0);
                }
            }
            GamePhase::Playing | GamePhase::GameOver => {
                let (viewport_rect, board_scale, board_size) =
                    compute_board_layout(screen_w, screen_h, scale);

                // Viewport Camera & Render Target setup for smooth subpixel scrolling & zooming
                let (rt, pan_offset, effective_scale, was_dragging) =
                    camera.update_and_begin(viewport_rect, board_size, board_scale, scale, dt);

                let mouse_pos = Vec2::from(mouse_position());
                let viewport_mouse = mouse_pos - Vec2::new(viewport_rect.x, viewport_rect.y);

                board_view.draw_and_handle_input(
                    &mut state,
                    pan_offset,
                    effective_scale,
                    viewport_mouse,
                    was_dragging,
                    &sound_manager,
                );

                camera.end_camera(viewport_rect, rt);

                // Draw Top Minimal In-Game HUD
                let (hud_sound, toggle_mute) = Screens::draw_ingame_hud(
                    &mut state,
                    screen_w,
                    screen_h,
                    scale,
                    sound_manager.is_muted(),
                    font.as_ref(),
                );
                if toggle_mute {
                    sound_manager.toggle_mute();
                }
                if let Some(snd) = hud_sound {
                    sound_manager.play(snd);
                    if state.turn_count == 1 && state.phase == GamePhase::Playing {
                        camera.start_coop_fox_intro(
                            viewport_rect,
                            board_size,
                            board_scale,
                            scale,
                            2.0,
                        );
                    }
                }

                // Update & Render Confetti Particles
                fx_manager.update(dt);
                fx_manager.draw();

                // Draw Game Over Modal if match ended
                if state.phase == GamePhase::GameOver {
                    let modal_sound = Screens::draw_game_over_modal(
                        &mut state,
                        screen_w,
                        screen_h,
                        scale,
                        font.as_ref(),
                    );
                    if let Some(snd) = modal_sound {
                        sound_manager.play(snd);
                        if state.phase == GamePhase::Playing {
                            camera.start_coop_fox_intro(
                                viewport_rect,
                                board_size,
                                board_scale,
                                scale,
                                2.0,
                            );
                        }
                    }
                }
            }
        }

        next_frame().await;
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_plugin_exports {
    #[no_mangle]
    pub extern "C" fn game_audio_crate_version() -> u32 {
        1
    }

    #[no_mangle]
    pub extern "C" fn macroquad_audio_crate_version() -> u32 {
        1
    }

    #[no_mangle]
    pub extern "C" fn sapp_jsutils_crate_version() -> u32 {
        1
    }

    #[no_mangle]
    pub extern "C" fn quad_net_crate_version() -> u32 {
        1
    }

    #[no_mangle]
    pub extern "C" fn game_locale_crate_version() -> u32 {
        1
    }
}
