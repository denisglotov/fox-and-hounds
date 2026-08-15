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
        window_width: 1000,
        window_height: 960,
        high_dpi: true,
        window_resizable: true,
        fullscreen: false,
        sample_count: if cfg!(target_os = "android") { 0 } else { 4 },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut sound_manager = SoundManager::new().await;
    let mut state = GameState::new();
    let mut board_view = BoardView::new().await;
    let mut camera = ViewportCamera::new();
    let mut fx_manager = FxManager::new();

    let mut last_result = GameResult::Ongoing;

    loop {
        let dt = get_frame_time();
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Responsive UI scale factor
        let base_scale = (screen_w / 900.0).min(screen_h / 860.0);
        let scale = base_scale.clamp(0.65, 1.4);

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
                let title_sound = Screens::draw_title_screen(&mut state, screen_w, screen_h, scale);
                if let Some(snd) = title_sound {
                    sound_manager.play(snd);
                }
            }
            GamePhase::Playing | GamePhase::GameOver => {
                let hud_h = 56.0 * scale;
                let viewport_rect = Rect::new(0.0, hud_h, screen_w, (screen_h - hud_h).max(1.0));

                // Board scale calculation:
                // - Fit neatly inside available viewport on large desktop displays
                // - Enforce minimum board scale so nodes & pieces remain easily touchable/visible on mobile / small windows, enabling scrolling.
                #[cfg(target_os = "android")]
                let min_board_scale = 0.55 * scale;
                #[cfg(not(target_os = "android"))]
                let min_board_scale = 0.45 * scale;

                let fit_scale_w = (viewport_rect.w - 24.0 * scale) / BOARD_IMAGE_WIDTH;
                let fit_scale_h = (viewport_rect.h - 24.0 * scale) / BOARD_IMAGE_HEIGHT;
                let board_scale = fit_scale_w.min(fit_scale_h).max(min_board_scale);

                let board_size = Vec2::new(
                    BOARD_IMAGE_WIDTH * board_scale,
                    BOARD_IMAGE_HEIGHT * board_scale,
                );

                // Viewport Camera & Render Target setup for smooth subpixel scrolling
                let (rt, pan_offset, was_dragging) =
                    camera.update_and_begin(viewport_rect, board_size, scale);

                let mouse_pos = Vec2::from(mouse_position());
                let viewport_mouse = mouse_pos - Vec2::new(viewport_rect.x, viewport_rect.y);

                let board_sound = board_view.draw_and_handle_input(
                    &mut state,
                    pan_offset,
                    board_scale,
                    viewport_mouse,
                    was_dragging,
                );
                if let Some(snd) = board_sound {
                    sound_manager.play(snd);
                }

                camera.end_camera(viewport_rect, rt);

                // Draw Top Minimal In-Game HUD
                let (hud_sound, toggle_mute) = Screens::draw_ingame_hud(
                    &mut state,
                    screen_w,
                    screen_h,
                    scale,
                    sound_manager.is_muted(),
                );
                if toggle_mute {
                    sound_manager.toggle_mute();
                }
                if let Some(snd) = hud_sound {
                    sound_manager.play(snd);
                }

                // Update & Render Confetti Particles
                fx_manager.update(dt);
                fx_manager.draw();

                // Draw Game Over Modal if match ended
                if state.phase == GamePhase::GameOver {
                    let modal_sound =
                        Screens::draw_game_over_modal(&mut state, screen_w, screen_h, scale);
                    if let Some(snd) = modal_sound {
                        sound_manager.play(snd);
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
}
