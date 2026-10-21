use fox_and_hounds::audio::SoundManager;
use fox_and_hounds::game::level::BoardDimensions;
use fox_and_hounds::game::state::{Faction, GamePhase, GameResult, GameState};
use fox_and_hounds::ui::board_view::{BoardView, BoardViewParams};
use fox_and_hounds::ui::camera::{CameraUpdateParams, ViewportCamera};
use fox_and_hounds::ui::carousel::BoardCarousel;
use fox_and_hounds::ui::fx::FxManager;
use fox_and_hounds::ui::nav::{GameOverFocus, InGameFocus, NavInput, UiNavState};
use fox_and_hounds::ui::screens::{GameOverParams, HudParams, Screens, TitleScreenParams};
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

fn compute_ui_scale(screen_w: f32, screen_h: f32) -> f32 {
    #[cfg(target_os = "android")]
    {
        if screen_w > screen_h {
            ((screen_w / 850.0).min(screen_h / 520.0)).clamp(1.0, 4.0)
        } else {
            (screen_w / 380.0).clamp(1.0, 4.0)
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        let base_scale = if screen_w > screen_h {
            (screen_w / 850.0).min(screen_h / 520.0)
        } else {
            (screen_w / 520.0).min(screen_h / 850.0)
        };
        base_scale.clamp(0.65, 2.5)
    }
}

fn compute_board_layout(
    screen_w: f32,
    screen_h: f32,
    _scale: f32,
    dims: &BoardDimensions,
) -> (Rect, f32, Vec2) {
    let viewport_rect = Rect::new(0.0, 0.0, screen_w, screen_h.max(1.0));

    let board_scale = (viewport_rect.h / dims.image_height).max(0.1);

    let board_size = Vec2::new(
        dims.image_width * board_scale,
        dims.image_height * board_scale,
    );

    (viewport_rect, board_scale, board_size)
}

async fn restart_board_session(
    board_view: &mut BoardView,
    camera: &mut ViewportCamera,
    variant: fox_and_hounds::game::level::BoardVariant,
    (viewport_rect, board_scale, board_size): (Rect, f32, Vec2),
) {
    board_view.ensure_board_loaded(variant).await;
    board_view.reset_simulations();
    camera.start_intro(variant, viewport_rect, board_size, board_scale, 2.0);
}

#[macroquad::main(window_conf)]
async fn main() {
    macroquad::file::set_pc_assets_folder("assets");
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
    let mut carousel = BoardCarousel::new();
    let mut camera = ViewportCamera::new();
    let mut fx_manager = FxManager::new();
    let mut nav = UiNavState::new();

    let mut last_result = GameResult::Ongoing;

    loop {
        let dt = get_frame_time().min(0.04);
        let screen_w = screen_width();
        let screen_h = screen_height();
        let mut nav_input = nav.poll_input();
        let scale = compute_ui_scale(screen_w, screen_h);

        // 1. Update Game State (animations, AI thinking)
        let state_sound = state.update(dt);
        if let Some(snd) = state_sound {
            sound_manager.play(snd);
        }

        // Spawn confetti when match concludes with player victory
        if state.result != last_result {
            if state.result != GameResult::Ongoing {
                nav.game_over_focus = GameOverFocus::PlayAgain;
            }
            let player_won = matches!(
                (state.result, state.player_faction),
                (GameResult::FoxWon, Faction::Fox) | (GameResult::HoundsWon, Faction::Hounds)
            );
            if player_won {
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
                    &mut carousel,
                    &mut nav,
                    &TitleScreenParams {
                        screen_w,
                        screen_h,
                        scale,
                        font: font.as_ref(),
                        hero_texture: character_texture.as_ref(),
                        nav_input,
                    },
                );
                if let Some(snd) = title_sound {
                    sound_manager.play(snd);
                }
                if state.phase == GamePhase::Playing {
                    nav.ingame_focus = InGameFocus::Board;
                    let layout = compute_board_layout(
                        screen_w,
                        screen_h,
                        scale,
                        &state.variant.config().dimensions,
                    );
                    restart_board_session(&mut board_view, &mut camera, state.variant, layout)
                        .await;
                }
            }
            GamePhase::Playing | GamePhase::GameOver => {
                let (viewport_rect, board_scale, board_size) = compute_board_layout(
                    screen_w,
                    screen_h,
                    scale,
                    &state.variant.config().dimensions,
                );

                let focus_board_pos = board_view.focused_board_pos(&state, &nav);

                // Viewport Camera & Render Target setup for smooth subpixel scrolling & zooming
                let camera_ctx = camera.update_and_begin(&CameraUpdateParams {
                    viewport_rect,
                    base_board_size: board_size,
                    base_board_scale: board_scale,
                    scale,
                    dt,
                    dims: &state.variant.config().dimensions,
                    focus_board_pos,
                });

                let mouse_pos = Vec2::from(mouse_position());
                let viewport_mouse = mouse_pos - Vec2::new(viewport_rect.x, viewport_rect.y);

                board_view.draw_and_handle_input(
                    &mut state,
                    &mut BoardViewParams {
                        origin: camera_ctx.pan_offset,
                        scale: camera_ctx.effective_scale,
                        viewport_mouse_pos: viewport_mouse,
                        viewport_size: Vec2::new(viewport_rect.w, viewport_rect.h),
                        was_dragging: camera_ctx.was_dragging,
                        intro_settled: camera_ctx.intro_settled,
                        sound_manager: &sound_manager,
                        dt,
                        nav: &mut nav,
                        nav_input: &mut nav_input,
                    },
                );

                camera.end_camera(viewport_rect, camera_ctx.render_target);

                // Draw Top Minimal In-Game HUD
                let hud_nav_input = if state.phase == GamePhase::Playing {
                    nav_input
                } else {
                    NavInput::default()
                };
                let (hud_sound, toggle_mute) = Screens::draw_ingame_hud(
                    &mut state,
                    &mut nav,
                    &HudParams {
                        screen_w,
                        screen_h,
                        scale,
                        is_muted: sound_manager.is_muted(),
                        font: font.as_ref(),
                        nav_input: hud_nav_input,
                    },
                );
                if toggle_mute {
                    sound_manager.toggle_mute();
                }
                if let Some(snd) = hud_sound {
                    sound_manager.play(snd);
                    if state.turn_count == 1 && state.phase == GamePhase::Playing {
                        nav.ingame_focus = InGameFocus::Board;
                        restart_board_session(
                            &mut board_view,
                            &mut camera,
                            state.variant,
                            (viewport_rect, board_scale, board_size),
                        )
                        .await;
                    }
                }

                // Update & Render Confetti Particles
                fx_manager.update(dt);
                fx_manager.draw();

                // Draw Game Over Modal if match ended
                if state.phase == GamePhase::GameOver {
                    let modal_sound = Screens::draw_game_over_modal(
                        &mut state,
                        &mut nav,
                        &GameOverParams {
                            screen_w,
                            screen_h,
                            scale,
                            font: font.as_ref(),
                            nav_input,
                        },
                    );
                    if let Some(snd) = modal_sound {
                        sound_manager.play(snd);
                        if state.phase == GamePhase::Playing {
                            nav.ingame_focus = InGameFocus::Board;
                            restart_board_session(
                                &mut board_view,
                                &mut camera,
                                state.variant,
                                (viewport_rect, board_scale, board_size),
                            )
                            .await;
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
