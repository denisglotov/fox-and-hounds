use fox_and_hounds::ui::screens::{GameOverModalLayout, TitleScreenLayout};
use macroquad::prelude::Rect;

fn assert_rect_inside(inner: Rect, outer: Rect, name: &str) {
    assert!(
        inner.x >= outer.x - 0.1,
        "{}: inner.x ({}) < outer.x ({})",
        name,
        inner.x,
        outer.x
    );
    assert!(
        inner.y >= outer.y - 0.1,
        "{}: inner.y ({}) < outer.y ({})",
        name,
        inner.y,
        outer.y
    );
    assert!(
        inner.x + inner.w <= outer.x + outer.w + 0.1,
        "{}: inner right ({}) > outer right ({})",
        name,
        inner.x + inner.w,
        outer.x + outer.w
    );
    assert!(
        inner.y + inner.h <= outer.y + outer.h + 0.1,
        "{}: inner bottom ({}) > outer bottom ({})",
        name,
        inner.y + inner.h,
        outer.y + outer.h
    );
}

#[test]
fn test_title_screen_landscape_android_fit() {
    let screen_w: f32 = 2400.0;
    let screen_h: f32 = 1080.0;
    let scale: f32 = (1080.0f32 / 380.0f32).clamp(1.0, 4.0); // ~2.842

    let layout = TitleScreenLayout::compute(screen_w, screen_h, scale, true, 16.0 / 9.0);

    assert!(layout.is_landscape);

    let screen_bounds = Rect::new(0.0, 0.0, screen_w, screen_h);
    assert_rect_inside(layout.card_bounds, screen_bounds, "card_bounds");
    assert_rect_inside(layout.left_column, layout.card_bounds, "left_column");
    assert_rect_inside(layout.right_column, layout.card_bounds, "right_column");

    if let Some(hb) = layout.hero_bounds {
        assert_rect_inside(hb, layout.left_column, "hero_bounds");
    }

    assert_rect_inside(layout.fox_btn_bounds, layout.right_column, "fox_btn_bounds");
    assert_rect_inside(
        layout.hounds_btn_bounds,
        layout.right_column,
        "hounds_btn_bounds",
    );
    assert!(
        layout.fox_btn_bounds.x + layout.fox_btn_bounds.w <= layout.hounds_btn_bounds.x + 0.1,
        "Fox and Hounds buttons must not overlap horizontally"
    );

    for (idx, &db) in layout.difficulty_btn_bounds.iter().enumerate() {
        assert_rect_inside(db, layout.right_column, &format!("diff_btn_{}", idx));
    }
    assert!(
        layout.difficulty_btn_bounds[0].x + layout.difficulty_btn_bounds[0].w
            <= layout.difficulty_btn_bounds[1].x + 0.1
    );
    assert!(
        layout.difficulty_btn_bounds[1].x + layout.difficulty_btn_bounds[1].w
            <= layout.difficulty_btn_bounds[2].x + 0.1
    );

    assert_rect_inside(
        layout.start_btn_bounds,
        layout.right_column,
        "start_btn_bounds",
    );

    assert!(
        layout.fox_btn_bounds.y + layout.fox_btn_bounds.h
            <= layout.difficulty_btn_bounds[0].y + 0.1,
        "Faction buttons must precede difficulty buttons vertically"
    );
    assert!(
        layout.difficulty_btn_bounds[0].y + layout.difficulty_btn_bounds[0].h
            <= layout.start_btn_bounds.y + 0.1,
        "Difficulty buttons must precede Start button vertically"
    );
}

#[test]
fn test_title_screen_landscape_desktop_and_web_fit() {
    let test_resolutions: [(f32, f32); 5] = [
        (1920.0, 1080.0), // 1080p
        (1280.0, 720.0),  // 720p
        (960.0, 540.0),   // qHD
        (800.0, 480.0),   // WVGA
        (640.0, 360.0),   // 360p
    ];

    for (screen_w, screen_h) in test_resolutions {
        let base_scale = (screen_w / 850.0f32).min(screen_h / 520.0f32);
        let scale = base_scale.clamp(0.65, 2.5);

        let layout = TitleScreenLayout::compute(screen_w, screen_h, scale, true, 16.0 / 9.0);
        assert!(
            layout.is_landscape,
            "Resolution {}x{} should be landscape",
            screen_w, screen_h
        );

        let screen_bounds = Rect::new(0.0, 0.0, screen_w, screen_h);
        assert_rect_inside(layout.card_bounds, screen_bounds, "card_bounds");
        assert_rect_inside(
            layout.start_btn_bounds,
            layout.card_bounds,
            "start_btn_bounds",
        );
        assert_rect_inside(layout.fox_btn_bounds, layout.card_bounds, "fox_btn_bounds");
    }
}

#[test]
fn test_title_screen_portrait_fit() {
    let test_portrait_resolutions: [(f32, f32, f32); 4] = [
        (1080.0, 2400.0, 2.842), // Android portrait
        (960.0, 1360.0, 1.6),    // Desktop portrait
        (720.0, 1280.0, 1.89),   // HD portrait
        (600.0, 800.0, 0.94),    // 3:4 portrait
    ];

    for (screen_w, screen_h, scale) in test_portrait_resolutions {
        let layout = TitleScreenLayout::compute(screen_w, screen_h, scale, true, 16.0 / 9.0);
        assert!(
            !layout.is_landscape,
            "Resolution {}x{} should be portrait",
            screen_w, screen_h
        );

        let screen_bounds = Rect::new(0.0, 0.0, screen_w, screen_h);
        assert_rect_inside(layout.card_bounds, screen_bounds, "card_bounds");
        assert_rect_inside(
            layout.start_btn_bounds,
            layout.card_bounds,
            "start_btn_bounds",
        );
        assert_rect_inside(layout.fox_btn_bounds, layout.card_bounds, "fox_btn_bounds");
        assert_rect_inside(
            layout.hounds_btn_bounds,
            layout.card_bounds,
            "hounds_btn_bounds",
        );

        assert!(
            layout.fox_btn_bounds.y + layout.fox_btn_bounds.h
                <= layout.difficulty_btn_bounds[0].y + 0.1
        );
        assert!(
            layout.difficulty_btn_bounds[0].y + layout.difficulty_btn_bounds[0].h
                <= layout.start_btn_bounds.y + 0.1
        );
    }
}

#[test]
fn test_game_over_modal_layout_fit() {
    let test_screens: [(f32, f32, f32); 5] = [
        (2400.0, 1080.0, 2.842),
        (1920.0, 1080.0, 2.0),
        (1280.0, 720.0, 1.38),
        (1080.0, 2400.0, 2.842),
        (960.0, 1360.0, 1.6),
    ];

    for (screen_w, screen_h, scale) in test_screens {
        let layout = GameOverModalLayout::compute(screen_w, screen_h, scale);
        let screen_bounds = Rect::new(0.0, 0.0, screen_w, screen_h);

        assert_rect_inside(layout.modal_bounds, screen_bounds, "modal_bounds");
        assert_rect_inside(
            layout.rematch_btn_bounds,
            layout.modal_bounds,
            "rematch_btn_bounds",
        );
        assert_rect_inside(
            layout.menu_btn_bounds,
            layout.modal_bounds,
            "menu_btn_bounds",
        );
        assert!(
            layout.rematch_btn_bounds.y + layout.rematch_btn_bounds.h
                <= layout.menu_btn_bounds.y + 0.1,
            "Rematch button must be above Menu button without overlap"
        );
    }
}
