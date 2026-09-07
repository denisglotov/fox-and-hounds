#!/usr/bin/env python3
"""
Font subsetting script for Fox & Hounds.
Extracts all needed glyphs from Arial Unicode MS, merges game HUD emojis,
and outputs a compact assets/NotoSansEmoji.ttf.
"""

import copy
import glob
import json
import os
import sys

try:
    from fontTools import subset
    from fontTools.ttLib import TTFont
    from fontTools.ttLib.tables._c_m_a_p import CmapSubtable
except ImportError:
    print("Error: fontTools is required. Install via `pip install fonttools`.")
    sys.exit(1)


def collect_characters(locales_dir: str) -> set[str]:
    chars = set()
    # 1. Collect all characters across JSON translation files
    for fpath in glob.glob(os.path.join(locales_dir, "*.json")):
        with open(fpath, "r", encoding="utf-8") as f:
            data = json.load(f)

            def recurse(v):
                if isinstance(v, str):
                    chars.update(v)
                elif isinstance(v, dict):
                    for k, child in v.items():
                        if not k.startswith("@"):  # ignore ARB metadata
                            recurse(child)

            recurse(data)

    # 2. Standard ASCII printable set (U+0020 .. U+007E)
    for code in range(32, 127):
        chars.add(chr(code))

    return chars


def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    locales_dir = os.path.join(base_dir, "assets", "locales")
    output_path = os.path.join(base_dir, "assets", "NotoSansEmoji.ttf")

    source_candidates = [
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode.ttf",
    ]
    arial_path = next(
        (p for p in source_candidates if os.path.exists(p)), None)
    if not arial_path:
        print(f"Error: Arial Unicode MS not found in {source_candidates}")
        sys.exit(1)

    chars = collect_characters(locales_dir)
    emojis = ["🦊", "🐶", "🏠", "🔄", "🔇", "🔊"]
    text_to_subset = "".join(sorted(c for c in chars if c not in emojis))

    print(f"Subsetting {len(chars)} characters from {arial_path}...")

    # 1. Subset Arial Unicode MS
    options = subset.Options()
    options.name_IDs = ["*"]
    options.name_legacy = True
    options.name_languages = ["*"]
    options.layout_features = ["*"]
    options.glyph_names = True
    options.notdef_glyph = True
    options.notdef_outline = True
    options.recommended_glyphs = True

    font_src = TTFont(arial_path)
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(text=text_to_subset)
    subsetter.subset(font_src)

    # 2. Extract emojis from current font and append them
    curr_font = TTFont(output_path)
    curr_cmap = curr_font.getBestCmap()
    glyph_order = list(font_src.getGlyphOrder())

    emoji_gname_map = {}
    for e in emojis:
        code = ord(e)
        if code in curr_cmap:
            gname = curr_cmap[code]
            glyph = curr_font["glyf"][gname]
            width, lsb = curr_font["hmtx"][gname]

            dest_gname = f"u{code:05X}"
            font_src["glyf"].glyphs[dest_gname] = copy.deepcopy(glyph)
            font_src["hmtx"][dest_gname] = (width, lsb)
            glyph_order.append(dest_gname)
            emoji_gname_map[code] = dest_gname

    font_src.setGlyphOrder(glyph_order)
    font_src["glyf"].glyphOrder = glyph_order
    font_src["maxp"].numGlyphs = len(glyph_order)

    # 3. Create Format 12 subtable for cmap (to support emoji > 0xFFFF)
    cmap12 = CmapSubtable.newSubtable(12)
    cmap12.platformID = 3
    cmap12.platEncID = 10
    cmap12.language = 0
    cmap12.cmap = {}
    for st in font_src["cmap"].tables:
        if st.format == 4:
            cmap12.cmap.update(st.cmap)
    for code, gname in emoji_gname_map.items():
        cmap12.cmap[code] = gname

    font_src["cmap"].tables.append(cmap12)

    # Drop legacy device metrics tables
    for tag in ["hdmx", "LTSH", "VDMX"]:
        if tag in font_src:
            del font_src[tag]

    font_src.save(output_path)
    size_kb = os.path.getsize(output_path) / 1024.0
    print(f"Successfully generated {output_path} ({size_kb:.1f} KB)")


if __name__ == "__main__":
    main()
