# Android 15 edge-to-edge compatibility patch

Macroquad's Android activity comes from Miniquad. Miniquad 0.4.11 sets
`LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES` whenever fullscreen is enabled.
Android 15 deprecates that mode for edge-to-edge windows and treats it as
`LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS`.

This vendored copy uses `LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS` on Android 11
(API 30) and later, where that constant is available. Android 9 and 10 retain
their framework default cutout behavior. Remove this patch once the resolved
upstream Miniquad release no longer references `SHORT_EDGES` in
`java/MainActivity.java`.
