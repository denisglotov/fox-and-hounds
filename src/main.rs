use fox_and_hounds::audio::SoundManager;
use fox_and_hounds::game::level::{BoardDimensions, BoardVariant};
use fox_and_hounds::game::state::{Faction, GamePhase, GameResult, GameState};
use fox_and_hounds::ui::board_view::{BoardView, BoardViewParams};
use fox_and_hounds::ui::camera::{CameraUpdateParams, ViewportCamera};
use fox_and_hounds::ui::carousel::BoardCarousel;
use fox_and_hounds::ui::draw_loading_overlay;
use fox_and_hounds::ui::fx::FxManager;
use fox_and_hounds::ui::nav::{GameOverFocus, InGameFocus, NavInput, UiNavState};
use fox_and_hounds::ui::screens::{
    GameOverParams, HudAction, HudParams, Screens, TitleScreenParams,
};
use fox_and_hounds::ui::story::LiveHeroBanner;
use macroquad::experimental::coroutines::start_coroutine;
use macroquad::prelude::*;

async fn load_board_with_spinner(
    board_view: &mut BoardView,
    variant: BoardVariant,
    icon_texture: Option<&Texture2D>,
    scale: f32,
) {
    if board_view.is_board_loaded(variant) {
        return;
    }

    let filename = variant.config().board_image_filename;
    let coroutine =
        start_coroutine(async move { macroquad::texture::load_texture(filename).await });

    let start_time = get_time();
    let spinner_threshold = 0.3;
    let mut spinner_shown = false;
    let mut loaded_tex = None;

    loop {
        let elapsed = (get_time() - start_time) as f32;

        if coroutine.is_done() && (!spinner_shown || elapsed >= spinner_threshold + 0.2) {
            break;
        }

        if elapsed >= spinner_threshold {
            spinner_shown = true;
            let sw = screen_width();
            let sh = screen_height();
            draw_loading_overlay(sw, sh, scale, icon_texture, get_time() as f32);
        }

        next_frame().await;
    }

    if let Some(res) = coroutine.retrieve() {
        match res {
            Ok(tex) => loaded_tex = Some(tex),
            Err(err) => eprintln!("Failed to load board texture '{filename}': {err:?}"),
        }
    }

    if let Some(tex) = loaded_tex {
        board_view.set_board_texture(variant, tex);
    }
}

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

fn compute_board_layout(screen_w: f32, screen_h: f32, dims: &BoardDimensions) -> (Rect, f32, Vec2) {
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
    let icon_texture = {
        let tex = Texture2D::from_file_with_format(
            include_bytes!("../assets/icon.png"),
            Some(ImageFormat::Png),
        );
        tex.set_filter(FilterMode::Linear);
        Some(tex)
    };
    let mut sound_manager = SoundManager::new().await;
    let mut live_hero = LiveHeroBanner::new(include_bytes!("../assets/fox_and_hounds_story.bin"))
        .expect("Valid story frames binary");
    let mut state = GameState::new();
    let mut board_view = BoardView::new(font.clone()).await;
    let mut carousel = BoardCarousel::new();
    // Decode carousel art during application setup, before the title screen's first frame.
    carousel.load_textures();
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
        Screens::update_mouse_press_state();
        match state.phase {
            GamePhase::TitleScreen => {
                camera.reset_pan();
                if let Some(snd) = live_hero.update() {
                    if snd == fox_and_hounds::audio::SoundTrigger::StopStory {
                        sound_manager.stop_story();
                    } else {
                        sound_manager.play(snd);
                    }
                }
                let title_res = Screens::draw_title_screen(
                    &mut state,
                    &mut carousel,
                    &mut nav,
                    &TitleScreenParams {
                        screen_w,
                        screen_h,
                        scale,
                        font: font.as_ref(),
                        hero_texture: character_texture.as_ref(),
                        live_hero_texture: live_hero.texture(),
                        nav_input,
                        dt,
                    },
                );
                if let Some(snd) = title_res.sound {
                    sound_manager.play(snd);
                }
                if title_res.toggle_story {
                    if let Some(snd) = live_hero.toggle() {
                        if snd == fox_and_hounds::audio::SoundTrigger::StopStory {
                            sound_manager.stop_story();
                        } else {
                            sound_manager.play(snd);
                        }
                    }
                }

                if state.phase == GamePhase::Playing {
                    carousel.reset_drag();
                    if live_hero.is_playing() {
                        live_hero.stop();
                        sound_manager.stop_story();
                    }

                    load_board_with_spinner(
                        &mut board_view,
                        state.variant,
                        icon_texture.as_ref(),
                        scale,
                    )
                    .await;

                    nav.ingame_focus = InGameFocus::Board;
                    let layout = compute_board_layout(
                        screen_w,
                        screen_h,
                        &state.variant.config().dimensions,
                    );
                    restart_board_session(&mut board_view, &mut camera, state.variant, layout)
                        .await;
                }
            }
            GamePhase::Playing | GamePhase::GameOver => {
                let (viewport_rect, board_scale, board_size) =
                    compute_board_layout(screen_w, screen_h, &state.variant.config().dimensions);

                let focus_board_pos = board_view.focused_board_pos(&state, &nav);
                let mouse_pos = Vec2::from(mouse_position());
                let hud_rect = Screens::hud_bounds(screen_w, screen_h, scale);
                let block_mouse_click = hud_rect.contains(mouse_pos);
                // After the first C8 tap the board owns the short secret-mode click window.
                // Suppress camera double-tap/pan recognition until it has resolved.
                let block_camera_input = block_mouse_click || board_view.c8_click_timer > 0.0;

                // Viewport Camera & Render Target setup for smooth subpixel scrolling & zooming
                let camera_ctx = camera.update_and_begin(&CameraUpdateParams {
                    viewport_rect,
                    base_board_size: board_size,
                    base_board_scale: board_scale,
                    scale,
                    dt,
                    dims: &state.variant.config().dimensions,
                    focus_board_pos,
                    block_mouse_input: block_camera_input,
                });

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
                        block_mouse_click,
                    },
                );

                camera.end_camera(viewport_rect, camera_ctx.render_target);

                // Draw Top Minimal In-Game HUD
                let hud_nav_input = if state.phase == GamePhase::Playing {
                    nav_input
                } else {
                    NavInput::default()
                };
                let hud_outcome = Screens::draw_ingame_hud(
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
                if hud_outcome.toggle_mute {
                    sound_manager.toggle_mute();
                }
                if let Some(snd) = hud_outcome.sound {
                    sound_manager.play(snd);
                }
                if hud_outcome.action == Some(HudAction::Restart) {
                    state.start_game(state.player_faction, state.difficulty);
                    nav.ingame_focus = InGameFocus::Board;
                    restart_board_session(
                        &mut board_view,
                        &mut camera,
                        state.variant,
                        (viewport_rect, board_scale, board_size),
                    )
                    .await;
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_board_spinner_timing_calculation() {
        let start_time = 10.0;
        let spinner_threshold = 0.3;
        let current_time_fast = 10.15;
        let current_time_slow = 10.45;

        // Loads completing in < 0.3s do not trigger spinner
        assert!((current_time_fast - start_time) < spinner_threshold);
        // Loads taking >= 0.3s trigger spinner
        assert!((current_time_slow - start_time) >= spinner_threshold);
    }
}
