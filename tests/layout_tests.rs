use fox_and_hounds::game::level::BoardVariant;
use fox_and_hounds::ui::board_view::{
    special_rule_notice_center, SPECIAL_RULE_NOTICE_BASE_FONT_SIZE,
    SPECIAL_RULE_NOTICE_MAX_WIDTH_RATIO, SPECIAL_RULE_NOTICE_PIECE_CLEARANCE,
    SPECIAL_RULE_NOTICE_PLATE_PADDING,
};
use fox_and_hounds::ui::camera::ViewportCamera;
use fox_and_hounds::ui::screens::{GameOverModalLayout, TitleScreenLayout};
use macroquad::prelude::{Rect, Vec2};

fn assert_rect_inside(inner: Rect, outer: Rect, name: &str) {
    assert!(inner.w > 0.0, "{name}: inner.w ({}) <= 0", inner.w);
    assert!(inner.h > 0.0, "{name}: inner.h ({}) <= 0", inner.h);
    assert!(
        inner.x >= outer.x - 0.1,
        "{name}: inner.x ({}) < outer.x ({})",
        inner.x,
        outer.x
    );
    assert!(
        inner.y >= outer.y - 0.1,
        "{name}: inner.y ({}) < outer.y ({})",
        inner.y,
        outer.y
    );
    assert!(
        inner.x + inner.w <= outer.x + outer.w + 0.1,
        "{name}: inner right ({}) > outer right ({})",
        inner.x + inner.w,
        outer.x + outer.w
    );
    assert!(
        inner.y + inner.h <= outer.y + outer.h + 0.1,
        "{name}: inner bottom ({}) > outer bottom ({})",
        inner.y + inner.h,
        outer.y + outer.h
    );
}

#[test]
fn test_title_screen_responsive_fit() {
    let test_resolutions: [(f32, f32, f32, bool); 10] = [
        // Landscape (screen_w, screen_h, scale, expected_landscape)
        (2400.0, 1080.0, 2.07, true),
        (1920.0, 1080.0, 2.07, true),
        (1280.0, 720.0, 1.38, true),
        (960.0, 540.0, 1.03, true),
        (800.0, 480.0, 0.92, true),
        (640.0, 360.0, 0.69, true),
        // Portrait
        (1080.0, 2400.0, 2.84, false),
        (960.0, 1360.0, 1.60, false),
        (720.0, 1280.0, 1.89, false),
        (600.0, 800.0, 0.94, false),
    ];

    for (w, h, scale, expected_landscape) in test_resolutions {
        let screen_bounds = Rect::new(0.0, 0.0, w, h);

        // 1. With hero texture
        let layout = TitleScreenLayout::compute(w, h, scale, true, 16.0 / 9.0);
        assert_eq!(
            layout.is_landscape, expected_landscape,
            "{w}x{h} orientation"
        );
        assert_rect_inside(layout.card_bounds, screen_bounds, "card_bounds");
        assert_rect_inside(layout.start_btn_bounds, layout.card_bounds, "start_btn");
        assert_rect_inside(
            layout.variant_card_bounds,
            layout.card_bounds,
            "variant_card",
        );
        assert_rect_inside(
            layout.carousel_clip_bounds,
            layout.card_bounds,
            "carousel_clip",
        );
        assert_rect_inside(
            layout.variant_dots_bounds,
            layout.card_bounds,
            "variant_dots",
        );
        assert_rect_inside(layout.fox_btn_bounds, layout.card_bounds, "fox_btn");
        assert_rect_inside(layout.hounds_btn_bounds, layout.card_bounds, "hounds_btn");

        assert!(
            layout.variant_card_bounds.y + layout.variant_card_bounds.h
                <= layout.variant_dots_bounds.y + 0.1
        );

        for (idx, &db) in layout.difficulty_btn_bounds.iter().enumerate() {
            assert_rect_inside(db, layout.card_bounds, &format!("diff_btn_{idx}"));
        }

        // 2. Without hero texture fallback
        let no_hero = TitleScreenLayout::compute(w, h, scale, false, 16.0 / 9.0);
        assert!(no_hero.hero_bounds.is_none());
        assert_rect_inside(no_hero.card_bounds, screen_bounds, "no_hero_card_bounds");
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

/// The Classic special rule notice ("hounds cannot retreat on this board") opens a match, so its
/// plate has to land inside the viewport on every supported resolution: below the hound
/// line (sprite box plus the clearance kept for their idle sway) and above the bottom edge
/// of the framed field.
#[test]
fn test_classic_special_rule_notice_stays_on_screen() {
    let variant = BoardVariant::Classic;
    let config = variant.config();
    let dims = config.dimensions;

    let resolutions: [(f32, f32); 8] = [
        (2400.0, 1080.0),
        (1920.0, 1080.0),
        (1280.0, 720.0),
        (960.0, 540.0),
        (1080.0, 2400.0),
        (960.0, 1360.0),
        (720.0, 1280.0),
        (600.0, 800.0),
    ];

    let lowest_node_y = (config.build_graph)()
        .nodes
        .iter()
        .fold(f32::MIN, |lowest, node| lowest.max(node.visual_pos.y));
    let notice =
        special_rule_notice_center(config.intro_framing, lowest_node_y, config.piece_base_size);

    for (screen_w, screen_h) in resolutions {
        let viewport = Rect::new(0.0, 0.0, screen_w, screen_h);
        let board_scale = (viewport.h / dims.image_height).max(0.1);
        let board_size = Vec2::new(
            dims.image_width * board_scale,
            dims.image_height * board_scale,
        );

        // The framing the intro zoom settles on
        let mut camera = ViewportCamera::new();
        camera.start_intro(variant, viewport, board_size, board_scale, 2.0);
        let (zoom, pan) = match camera.anim {
            Some(anim) => (anim.target_zoom, anim.target_pan),
            None => (camera.zoom, camera.pan_offset),
        };
        let scale = board_scale * zoom;

        // Worst case of the plate: the base font at its tallest, and the widest the fit
        // allows - which is the framed field, or the visible width on boards wider than it
        let plate_w = (config.intro_framing.playable_size.x * scale).min(viewport.w)
            * SPECIAL_RULE_NOTICE_MAX_WIDTH_RATIO;
        let plate_h = (SPECIAL_RULE_NOTICE_BASE_FONT_SIZE * 1.3
            + 2.0 * SPECIAL_RULE_NOTICE_PLATE_PADDING)
            * scale;
        let center = pan + notice * scale;
        let plate = Rect::new(
            center.x - plate_w / 2.0,
            center.y - plate_h / 2.0,
            plate_w,
            plate_h,
        );

        assert_rect_inside(
            plate,
            viewport,
            &format!("Classic special rule notice on {screen_w}x{screen_h}"),
        );

        let piece_bottom = pan.y
            + (lowest_node_y + config.piece_base_size * 0.5 + SPECIAL_RULE_NOTICE_PIECE_CLEARANCE)
                * scale;
        assert!(
            plate.y >= piece_bottom - 0.1,
            "Classic special rule notice covers the hounds on {screen_w}x{screen_h}: \
             plate top {} < piece bottom {piece_bottom}",
            plate.y
        );
    }
}
