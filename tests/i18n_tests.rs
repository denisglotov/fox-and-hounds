//! Locale data integrity: every embedded translation must be complete, resolvable and drawable
//! with the subset font that ships in `assets/`.

use fox_and_hounds::game::i18n::{
    get_locales_list, normalize_locale_tag, resolve_locale, LocaleStrings, TitleScreenStrings,
};
use fox_and_hounds::game::state::Difficulty;

/// Every user-visible string of a locale, named by its JSON path so a failure points at the file
/// to fix. Used both as the completeness checklist and as the glyph coverage corpus.
fn required_strings(loc: &LocaleStrings) -> Vec<(&'static str, &str)> {
    vec![
        ("locale", &loc.locale),
        ("language_name", &loc.language_name),
        ("title_screen.title", &loc.title_screen.title),
        ("title_screen.subtitle", &loc.title_screen.subtitle),
        (
            "title_screen.board_variant",
            &loc.title_screen.board_variant,
        ),
        (
            "title_screen.variant_classic",
            &loc.title_screen.variant_classic,
        ),
        (
            "title_screen.variant_classic_sub",
            &loc.title_screen.variant_classic_sub,
        ),
        (
            "title_screen.variant_river_crossing",
            &loc.title_screen.variant_river_crossing,
        ),
        (
            "title_screen.variant_river_crossing_sub",
            &loc.title_screen.variant_river_crossing_sub,
        ),
        (
            "title_screen.variant_fox_and_dogs",
            &loc.title_screen.variant_fox_and_dogs,
        ),
        (
            "title_screen.variant_fox_and_dogs_sub",
            &loc.title_screen.variant_fox_and_dogs_sub,
        ),
        (
            "title_screen.variant_fox_and_dogs_maze",
            &loc.title_screen.variant_fox_and_dogs_maze,
        ),
        (
            "title_screen.variant_fox_and_dogs_maze_sub",
            &loc.title_screen.variant_fox_and_dogs_maze_sub,
        ),
        (
            "title_screen.variant_the_red_hunt",
            &loc.title_screen.variant_the_red_hunt,
        ),
        (
            "title_screen.variant_the_red_hunt_sub",
            &loc.title_screen.variant_the_red_hunt_sub,
        ),
        (
            "title_screen.choose_faction",
            &loc.title_screen.choose_faction,
        ),
        ("title_screen.fox_title", &loc.title_screen.fox_title),
        ("title_screen.fox_subtitle", &loc.title_screen.fox_subtitle),
        ("title_screen.hounds_title", &loc.title_screen.hounds_title),
        (
            "title_screen.hounds_subtitle",
            &loc.title_screen.hounds_subtitle,
        ),
        (
            "title_screen.ai_difficulty",
            &loc.title_screen.ai_difficulty,
        ),
        (
            "title_screen.difficulty_easy",
            &loc.title_screen.difficulty_easy,
        ),
        (
            "title_screen.difficulty_medium",
            &loc.title_screen.difficulty_medium,
        ),
        (
            "title_screen.difficulty_hard",
            &loc.title_screen.difficulty_hard,
        ),
        ("title_screen.start_match", &loc.title_screen.start_match),
        ("hud.fox_turn", &loc.hud.fox_turn),
        ("hud.hounds_turn", &loc.hud.hounds_turn),
        ("hud.thinking", &loc.hud.thinking),
        ("hud.special_rule_notice", &loc.hud.special_rule_notice),
        ("game_over.victory", &loc.game_over.victory),
        ("game_over.defeat", &loc.game_over.defeat),
        ("game_over.fox_won_msg", &loc.game_over.fox_won_msg),
        ("game_over.hounds_won_msg", &loc.game_over.hounds_won_msg),
        ("game_over.stats_template", &loc.game_over.stats_template),
        ("game_over.play_again", &loc.game_over.play_again),
        ("game_over.main_menu", &loc.game_over.main_menu),
    ]
}

#[test]
fn test_all_locales_load_and_contain_required_strings() {
    let locales = get_locales_list();
    assert_eq!(locales.len(), 10);

    for loc in locales {
        // Every required string is present and carries something other than whitespace
        for (field, value) in required_strings(loc) {
            assert!(!value.trim().is_empty(), "{}: {field} is empty", loc.locale);
        }

        // The stats template fills every placeholder it promises
        let stats_text = loc.game_over.format_stats(12, "Medium", "Classic");
        assert!(
            stats_text.contains("12")
                && stats_text.contains("Medium")
                && stats_text.contains("Classic"),
            "Stats string missing placeholders in {}",
            loc.locale
        );

        // The difficulty accessor reads back the names the title screen offers
        let difficulty_names = [
            (Difficulty::Easy, &loc.title_screen.difficulty_easy),
            (Difficulty::Medium, &loc.title_screen.difficulty_medium),
            (Difficulty::Hard, &loc.title_screen.difficulty_hard),
        ];
        for (difficulty, expected) in difficulty_names {
            assert_eq!(loc.difficulty_name(difficulty), expected);
        }
    }
}

#[test]
fn test_missing_field_fails_deserialization() {
    // Deserializing partial TitleScreenStrings missing required board variants must fail
    let partial_json = r#"{
            "title":"T","subtitle":"S","board_variant":"B",
            "variant_classic":"Classic","variant_classic_sub":"Sub",
            "variant_river_crossing":"River","variant_river_crossing_sub":"RiverSub",
            "choose_faction":"C","fox_title":"F","fox_subtitle":"FS",
            "hounds_title":"H","hounds_subtitle":"HS",
            "ai_difficulty":"A","difficulty_easy":"E","difficulty_medium":"M",
            "difficulty_hard":"HD","start_match":"SM"
        }"#;
    let res: Result<TitleScreenStrings, _> = serde_json::from_str(partial_json);
    assert!(res.is_err());
}

#[test]
fn test_locale_normalization_and_resolution() {
    // Tags are normalized, separators and case alike, before they are matched
    assert_eq!(normalize_locale_tag("  ru_RU "), "ru-ru");
    assert_eq!(normalize_locale_tag("ru+RU"), "ru-ru");

    assert_eq!(resolve_locale("ru-RU").locale, "ru-RU");
    assert_eq!(resolve_locale("ru_RU").locale, "ru-RU");
    assert_eq!(resolve_locale("ru+RU").locale, "ru-RU");
    assert_eq!(resolve_locale("RU").locale, "ru-RU");
    assert_eq!(resolve_locale("ru_KZ").locale, "ru-RU");

    assert_eq!(resolve_locale("es-ES").locale, "es-ES");
    assert_eq!(resolve_locale("es_MX").locale, "es-ES");
    assert_eq!(resolve_locale("es").locale, "es-ES");

    assert_eq!(resolve_locale("de-DE").locale, "de-DE");
    assert_eq!(resolve_locale("de_AT").locale, "de-DE");
    assert_eq!(resolve_locale("de").locale, "de-DE");

    assert_eq!(resolve_locale("fr-FR").locale, "fr-FR");
    assert_eq!(resolve_locale("fr_CA").locale, "fr-FR");
    assert_eq!(resolve_locale("fr").locale, "fr-FR");

    assert_eq!(resolve_locale("ja-JP").locale, "ja-JP");
    assert_eq!(resolve_locale("ja_JP").locale, "ja-JP");
    assert_eq!(resolve_locale("ja").locale, "ja-JP");
    assert_eq!(resolve_locale("jp").locale, "ja-JP");
    assert_eq!(resolve_locale("jp-JP").locale, "ja-JP");

    assert_eq!(resolve_locale("zh-CN").locale, "zh-CN");
    assert_eq!(resolve_locale("zh_CN").locale, "zh-CN");
    assert_eq!(resolve_locale("zh").locale, "zh-CN");
    assert_eq!(resolve_locale("cn").locale, "zh-CN");
    assert_eq!(resolve_locale("zh-hans").locale, "zh-CN");

    assert_eq!(resolve_locale("ko-KR").locale, "ko-KR");
    assert_eq!(resolve_locale("ko_KR").locale, "ko-KR");
    assert_eq!(resolve_locale("ko").locale, "ko-KR");
    assert_eq!(resolve_locale("kr").locale, "ko-KR");

    assert_eq!(resolve_locale("en-US").locale, "en-US");
    assert_eq!(resolve_locale("en_GB").locale, "en-US");
    assert_eq!(resolve_locale("en").locale, "en-US");

    assert_eq!(resolve_locale("it-IT").locale, "it-IT");
    assert_eq!(resolve_locale("it_CH").locale, "it-IT");
    assert_eq!(resolve_locale("it").locale, "it-IT");

    assert_eq!(resolve_locale("pt-BR").locale, "pt-BR");
    assert_eq!(resolve_locale("pt_PT").locale, "pt-BR");
    assert_eq!(resolve_locale("pt").locale, "pt-BR");
    assert_eq!(resolve_locale("br").locale, "pt-BR");

    // Unknown fallback to en-US
    assert_eq!(resolve_locale("ar-SA").locale, "en-US");
    assert_eq!(resolve_locale("unknown").locale, "en-US");
}

fn extract_ttf_cmap_codepoints(data: &[u8]) -> std::collections::HashSet<u32> {
    let mut chars = std::collections::HashSet::new();
    if data.len() < 12 {
        return chars;
    }

    let num_tables = u16::from_be_bytes([data[4], data[5]]) as usize;
    let mut cmap_offset = None;
    for i in 0..num_tables {
        let offset = 12 + i * 16;
        if offset + 16 > data.len() {
            break;
        }
        if &data[offset..offset + 4] == b"cmap" {
            cmap_offset = Some(u32::from_be_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]) as usize);
            break;
        }
    }

    let Some(cmap_pos) = cmap_offset else {
        return chars;
    };
    if cmap_pos + 4 > data.len() {
        return chars;
    }

    let num_subtables = u16::from_be_bytes([data[cmap_pos + 2], data[cmap_pos + 3]]) as usize;
    for i in 0..num_subtables {
        let sub_rec = cmap_pos + 4 + i * 8;
        if sub_rec + 8 > data.len() {
            break;
        }
        let sub_offset = u32::from_be_bytes([
            data[sub_rec + 4],
            data[sub_rec + 5],
            data[sub_rec + 6],
            data[sub_rec + 7],
        ]) as usize;
        let sub_pos = cmap_pos + sub_offset;
        if sub_pos + 2 > data.len() {
            continue;
        }
        let format = u16::from_be_bytes([data[sub_pos], data[sub_pos + 1]]);
        if format == 4 && sub_pos + 14 <= data.len() {
            let seg_count =
                (u16::from_be_bytes([data[sub_pos + 6], data[sub_pos + 7]]) / 2) as usize;
            let end_code_offset = sub_pos + 14;
            let start_code_offset = end_code_offset + seg_count * 2 + 2;
            if start_code_offset + seg_count * 2 <= data.len() {
                for seg in 0..seg_count {
                    let end_c = u16::from_be_bytes([
                        data[end_code_offset + seg * 2],
                        data[end_code_offset + seg * 2 + 1],
                    ]) as u32;
                    let start_c = u16::from_be_bytes([
                        data[start_code_offset + seg * 2],
                        data[start_code_offset + seg * 2 + 1],
                    ]) as u32;
                    if start_c != 0xFFFF && end_c != 0xFFFF {
                        for code in start_c..=end_c {
                            chars.insert(code);
                        }
                    }
                }
            }
        } else if format == 12 && sub_pos + 16 <= data.len() {
            let num_groups = u32::from_be_bytes([
                data[sub_pos + 12],
                data[sub_pos + 13],
                data[sub_pos + 14],
                data[sub_pos + 15],
            ]) as usize;
            let groups_pos = sub_pos + 16;
            if groups_pos + num_groups * 12 <= data.len() {
                for g in 0..num_groups {
                    let g_offset = groups_pos + g * 12;
                    let start_c = u32::from_be_bytes([
                        data[g_offset],
                        data[g_offset + 1],
                        data[g_offset + 2],
                        data[g_offset + 3],
                    ]);
                    let end_c = u32::from_be_bytes([
                        data[g_offset + 4],
                        data[g_offset + 5],
                        data[g_offset + 6],
                        data[g_offset + 7],
                    ]);
                    for code in start_c..=end_c {
                        chars.insert(code);
                    }
                }
            }
        }
    }
    chars
}

#[test]
fn test_font_contains_all_locale_glyphs() {
    let font_bytes = include_bytes!("../assets/NotoSansEmoji.ttf");
    let font_chars = extract_ttf_cmap_codepoints(font_bytes);

    for loc in get_locales_list() {
        for (field, text) in required_strings(loc) {
            for ch in text.chars() {
                if ch.is_control() || ch == ' ' {
                    continue;
                }
                assert!(
                    font_chars.contains(&(ch as u32)),
                    "Character '{}' (U+{:04X}) in {} ({field}) is missing from font",
                    ch,
                    ch as u32,
                    loc.locale
                );
            }
        }
    }

    // HUD emojis
    for emoji in [
        '\u{1f98a}',
        '\u{1f436}',
        '\u{1f3e0}',
        '\u{1f504}',
        '\u{1f507}',
        '\u{1f50a}',
    ] {
        assert!(
            font_chars.contains(&(emoji as u32)),
            "Emoji '{}' (U+{:04X}) is missing from font",
            emoji,
            emoji as u32
        );
    }
}
