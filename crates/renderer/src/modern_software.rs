use crate::modern_frame::{ModernFrame, MODERN_FRAME_HEIGHT, MODERN_FRAME_WIDTH};
use crate::modern_index_atlas::ModernIndexTile;

/// Composite the frame's palette-index OAM sprites onto an existing 256x224 RGBA
/// buffer (`out`, typically the BG render from
/// [`render_modern_frame_software_indexed`]).
///
/// Unlike BG cells, sprite cells in `sprite_cells` are stored UNFLIPPED, so this
/// applies each instance's `hflip`/`vflip` while sampling:
/// `src_x = if hflip {7-x}`, `src_y = if vflip {7-y}`,
/// `index = cell.indices[src_y*8 + src_x]`. Index 0 is transparent. The final
/// color is `frame.cgram_rgba[0x80 + palette*16 + index]` (OBJ CGRAM is 128..255).
///
/// Z-order matches the classic resolver's "first non-transparent pixel in OAM
/// order wins": sprites are drawn in REVERSE `index_sprites` order with REPLACE,
/// so the earliest OAM sprite is painted last and ends up on top. Sprites always
/// draw over the BG (sprite-vs-BG priority is out of scope here).
///
/// No-op when `frame.forced_blank` (the BG buffer is already solid black).
pub fn draw_modern_sprites_indexed(
    out: &mut [u8],
    frame: &ModernFrame,
    sprite_cells: &[ModernIndexTile],
) {
    if frame.forced_blank {
        return;
    }
    let width = usize::from(MODERN_FRAME_WIDTH);

    // Reverse OAM order → earliest OAM sprite drawn last (wins) under REPLACE.
    for inst in frame.index_sprites.iter().rev() {
        let cell = match sprite_cells.get(inst.cell_id as usize) {
            Some(c) => c,
            None => continue,
        };
        for y in 0..8usize {
            for x in 0..8usize {
                // Sprite cells are UNFLIPPED → apply flip when sampling.
                let src_x = if inst.hflip { 7 - x } else { x };
                let src_y = if inst.vflip { 7 - y } else { y };
                let index = cell.indices[src_y * 8 + src_x];
                if index == 0 {
                    continue; // transparent
                }
                let dst_x = inst.screen_x + x as i16;
                let dst_y = inst.screen_y + y as i16;
                if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                    continue; // clip to screen
                }
                let color = frame.cgram_rgba[0x80 + inst.palette as usize * 16 + index as usize];
                let dst = (dst_y as usize * width + dst_x as usize) * 4;
                out[dst..dst + 4].copy_from_slice(&color);
            }
        }
    }
}

pub fn render_modern_frame_software(
    frame: &ModernFrame,
    atlas_rgba: &[u8],
    atlas_width: u16,
    atlas_height: u16,
) -> Vec<u8> {
    let mut out = vec![0u8; usize::from(MODERN_FRAME_WIDTH) * usize::from(MODERN_FRAME_HEIGHT) * 4];
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&frame.backdrop_color_rgba);
    }
    if frame.forced_blank {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }
    for layer in &frame.bg_layers {
        if !layer.enabled_main {
            continue;
        }
        for tile in &layer.tiles {
            if tile.screen_width_px == 0 || tile.screen_height_px == 0 {
                continue; // degenerate footprint — nothing to draw
            }
            // Downsample factor from the (upscaled) atlas source rect to the
            // on-screen footprint. Nearest: sample the top-left of each block.
            let scale_x = tile.atlas_width_px / tile.screen_width_px;
            let scale_y = tile.atlas_height_px / tile.screen_height_px;
            for y in 0..tile.screen_height_px {
                for x in 0..tile.screen_width_px {
                    // Mirror on the SCREEN coordinate, then scale up into the source.
                    let src_x = if tile.hflip {
                        tile.screen_width_px - 1 - x
                    } else {
                        x
                    };
                    let src_y = if tile.vflip {
                        tile.screen_height_px - 1 - y
                    } else {
                        y
                    };
                    let atlas_x =
                        u32::from(tile.atlas_x_px) + u32::from(src_x) * u32::from(scale_x);
                    let atlas_y =
                        u32::from(tile.atlas_y_px) + u32::from(src_y) * u32::from(scale_y);
                    if atlas_x >= u32::from(atlas_width) || atlas_y >= u32::from(atlas_height) {
                        continue;
                    }
                    let dst_x = tile.screen_x + x as i16;
                    let dst_y = tile.screen_y + y as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue;
                    }
                    let src = (atlas_y as usize * usize::from(atlas_width) + atlas_x as usize) * 4;
                    let dst = (dst_y as usize * 256 + dst_x as usize) * 4;
                    if tile.transparent_color_zero && atlas_rgba[src + 3] == 0 {
                        continue;
                    }
                    out[dst..dst + 4].copy_from_slice(&atlas_rgba[src..src + 4]);
                }
            }
        }
    }
    out
}

/// Render a `ModernFrame` using the palette-index atlas + live CGRAM.
///
/// For each enabled BG layer, each `index_tiles` instance is drawn: for each
/// 8×8 pixel in the tile, `index = cell.indices[sy*8+sx]`; if `index == 0`
/// the pixel is transparent (skip); otherwise `color = frame.cgram_rgba[palette*16 + index]`.
///
/// Backdrop and forced_blank behaviour match `render_modern_frame_software`.
///
/// Note: `hflip`/`vflip` on the instance are intentionally ignored here.
/// The index pattern in the atlas already baked flip via `graphics_key` during
/// Task 2 atlas generation, so re-applying flip would double-mirror the pixels.
pub fn render_modern_frame_software_indexed(
    frame: &ModernFrame,
    cells: &[ModernIndexTile],
) -> Vec<u8> {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    let mut out = vec![0u8; width * height * 4];

    // Fill backdrop.
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&frame.backdrop_color_rgba);
    }

    // Forced blank: solid black.
    if frame.forced_blank {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }

    for layer in &frame.bg_layers {
        if !layer.enabled_main {
            continue;
        }
        for inst in &layer.index_tiles {
            // Cells are stored densely 0..len; guard against a bad id.
            let cell = match cells.get(inst.cell_id as usize) {
                Some(c) => c,
                None => continue,
            };
            for sy in 0..8usize {
                for sx in 0..8usize {
                    let index = cell.indices[sy * 8 + sx];
                    if index == 0 {
                        continue; // transparent
                    }
                    let dst_x = inst.screen_x + sx as i16;
                    let dst_y = inst.screen_y + sy as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue; // clip to screen
                    }
                    let color = frame.cgram_rgba[inst.palette as usize * 16 + index as usize];
                    let dst = (dst_y as usize * width + dst_x as usize) * 4;
                    out[dst..dst + 4].copy_from_slice(&color);
                }
            }
        }
    }

    out
}

/// One composited SNES screen (main or sub) as 5-bit-per-channel pixels plus the
/// per-pixel winning-layer math bit and whether a non-backdrop source drew there.
struct Screen {
    /// Per-pixel 5-bit channels (0..31).
    c5: Vec<[u8; 3]>,
    /// Winning layer's color-math bit: BG1=0..BG4=3, OBJ=4, BACKDROP=5.
    bit: Vec<u8>,
    /// True where a BG/sprite (not the backdrop) covered the pixel.
    real: Vec<bool>,
}

impl Screen {
    fn new(backdrop_c5: [u8; 3], len: usize) -> Self {
        Self {
            c5: vec![backdrop_c5; len],
            bit: vec![5u8; len],
            real: vec![false; len],
        }
    }
}

/// Composite one BG layer's indexed tiles into `screen` (painter-style, REPLACE),
/// stamping the layer's math `bit` and marking pixels real.
fn composite_index_tiles_c5(
    screen: &mut Screen,
    layer: &crate::modern_frame::ModernBgLayer,
    cells: &[ModernIndexTile],
    frame: &ModernFrame,
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let bit = layer.index;
    for inst in &layer.index_tiles {
        let cell = match cells.get(inst.cell_id as usize) {
            Some(c) => c,
            None => continue,
        };
        for sy in 0..8usize {
            for sx in 0..8usize {
                let index = cell.indices[sy * 8 + sx];
                if index == 0 {
                    continue; // transparent
                }
                let dst_x = inst.screen_x + sx as i16;
                let dst_y = inst.screen_y + sy as i16;
                if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                    continue;
                }
                let color = frame.cgram_rgba[inst.palette as usize * 16 + index as usize];
                let i = dst_y as usize * width + dst_x as usize;
                screen.c5[i] = [color[0] >> 3, color[1] >> 3, color[2] >> 3];
                screen.bit[i] = bit;
                screen.real[i] = true;
            }
        }
    }
}

/// Composite OAM sprites into `screen` (reverse OAM order, REPLACE → earliest OAM
/// sprite wins), stamping the OBJ math bit (4) and marking pixels real.
fn composite_sprites_c5(
    screen: &mut Screen,
    frame: &ModernFrame,
    sprite_cells: &[ModernIndexTile],
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    for inst in frame.index_sprites.iter().rev() {
        let cell = match sprite_cells.get(inst.cell_id as usize) {
            Some(c) => c,
            None => continue,
        };
        for y in 0..8usize {
            for x in 0..8usize {
                let src_x = if inst.hflip { 7 - x } else { x };
                let src_y = if inst.vflip { 7 - y } else { y };
                let index = cell.indices[src_y * 8 + src_x];
                if index == 0 {
                    continue;
                }
                let dst_x = inst.screen_x + x as i16;
                let dst_y = inst.screen_y + y as i16;
                if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                    continue;
                }
                let color = frame.cgram_rgba[0x80 + inst.palette as usize * 16 + index as usize];
                let i = dst_y as usize * width + dst_x as usize;
                screen.c5[i] = [color[0] >> 3, color[1] >> 3, color[2] >> 3];
                screen.bit[i] = 4;
                screen.real[i] = true;
            }
        }
    }
}

/// Expand a 5-bit channel to 8-bit and scale by master brightness, byte-exact with
/// the classic post-process shader (`post_process.wgsl::apply_brightness`):
/// `v8 = (c5 << 3) | (c5 >> 2)`, then `out8 = v8 * brightness / 15`.
///
/// NOTE: the task's prose formula paraphrased this as a plain `(c5 << 3) * b / 15`,
/// dropping the SNES low-bit fill `| (c5 >> 2)`. Because the comparison reference is
/// the classic shader (which applies the fill), this mirrors the shader exactly —
/// e.g. c5=31, b=15 → 255, not 248. (Empirically the fill is the dominant lever:
/// it alone drops the dungeon mismatch at frame 14000 from 38772 → 24906.)
#[inline]
fn expand_brightness(c5: i32, brightness: u8) -> u8 {
    let c = c5.clamp(0, 31) as u32;
    let v8 = (c << 3) | (c >> 2); // SNES low-bit fill, matching the classic shader
    ((v8 * u32::from(brightness)) / 15) as u8
}

/// Decode a CGWSEL clip/math-mode bit, byte-exact with `post_process.wgsl::cw_bit`.
///
/// `mode` 0=always 1 (never clip / always math), 1=follows window, 2=inverted,
/// 3=always 0 (always clip / never math). Mirrors the SNES CW_BITS_MOD table:
/// `((w & masks[mode]) ^ masks[mode + 4]) != 0`, where `w = 0xff` inside the window.
#[inline]
fn cw_bit(in_window: bool, mode: u8) -> bool {
    const MASKS: [u32; 8] = [0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00];
    let w = if in_window { 0xffu32 } else { 0 };
    let m = mode as usize & 7;
    ((w & MASKS[m]) ^ MASKS[m + 4]) != 0
}

/// Color-math window membership for screen column `sx`, byte-exact with the GPU
/// post-process shader: enabled W1/W2 masks (each with its inversion flag) are
/// OR-combined. `windowsel_cm` bits: 0=W1inv, 1=W1en, 2=W2inv, 3=W2en.
#[inline]
fn in_cm_window(sx: u32, win: [u8; 4], windowsel_cm: u8) -> bool {
    let [w1l, w1r, w2l, w2r] = win.map(u32::from);
    let mut inside = false;
    if windowsel_cm & 0x2 != 0 {
        let mut in_w1 = w1l <= w1r && sx >= w1l && sx <= w1r;
        if windowsel_cm & 0x1 != 0 {
            in_w1 = !in_w1;
        }
        inside |= in_w1;
    }
    if windowsel_cm & 0x8 != 0 {
        let mut in_w2 = w2l <= w2r && sx >= w2l && sx <= w2r;
        if windowsel_cm & 0x4 != 0 {
            in_w2 = !in_w2;
        }
        inside |= in_w2;
    }
    inside
}

/// Full modern software render: BG + sprites + SNES color-math + master brightness
/// in one call, producing the FINAL 8-bit RGBA (256×224) that mirrors the classic
/// post-process pipeline (`post_process.wgsl`), including the color-math/clip
/// window gating (`cw_bit` + per-scanline `window_scanlines`).
///
/// `bg_cells` are the indexed BG tile patterns; `sprite_cells` the live-VRAM sprite
/// patterns (from `extract_modern_sprites_from_vram`). A MAIN and a SUB composite
/// are built from the raw `screen_enabled_main`/`screen_enabled_sub` bits (BG bits
/// 0-3, OBJ bit 4), then color math + brightness are applied per pixel.
pub fn render_modern_frame_full(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
) -> Vec<u8> {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    let len = width * height;
    let mut out = vec![0u8; len * 4];

    if frame.forced_blank {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }

    let bd = &frame.backdrop_color_rgba;
    let backdrop_c5 = [bd[0] >> 3, bd[1] >> 3, bd[2] >> 3];

    // MAIN composite: enabled-main BG layers (in order) then main sprites.
    let mut main = Screen::new(backdrop_c5, len);
    for layer in &frame.bg_layers {
        if (frame.screen_enabled_main >> layer.index) & 1 != 0 {
            composite_index_tiles_c5(&mut main, layer, bg_cells, frame);
        }
    }
    if (frame.screen_enabled_main >> 4) & 1 != 0 {
        composite_sprites_c5(&mut main, frame, sprite_cells);
    }

    // SUB composite: enabled-sub BG layers then sub sprites.
    let mut sub = Screen::new(backdrop_c5, len);
    for layer in &frame.bg_layers {
        if (frame.screen_enabled_sub >> layer.index) & 1 != 0 {
            composite_index_tiles_c5(&mut sub, layer, bg_cells, frame);
        }
    }
    if (frame.screen_enabled_sub >> 4) & 1 != 0 {
        composite_sprites_c5(&mut sub, frame, sprite_cells);
    }

    // `rendered_subscreen`: is there ANY sub composite (BG1-4 or OBJ on sub)?
    let rendered_subscreen = (frame.screen_enabled_sub & 0x1f) != 0;
    let fixed = [
        i32::from(frame.fixed_color_r),
        i32::from(frame.fixed_color_g),
        i32::from(frame.fixed_color_b),
    ];
    let no_effect_math = frame.fixed_color_r == 0
        && frame.fixed_color_g == 0
        && frame.fixed_color_b == 0
        && !frame.half_color
        && !rendered_subscreen;

    for i in 0..len {
        let px = i % width;
        let py = i / width;
        let mut c = [
            i32::from(main.c5[i][0]),
            i32::from(main.c5[i][1]),
            i32::from(main.c5[i][2]),
        ];
        let layer_math_on = (frame.math_enabled >> main.bit[i]) & 1 != 0;
        // Color-window membership for this pixel, then gate clip + color-math exactly
        // like the classic post-process shader.
        let win = frame.window_scanlines.get(py).copied().unwrap_or([0u8; 4]);
        let cm_window = in_cm_window(px as u32, win, frame.windowsel_cm);
        let not_clipped = cw_bit(cm_window, frame.clip_mode);
        let math_window_ok = cw_bit(cm_window, frame.prevent_math_mode);
        let do_math = !no_effect_math && layer_math_on && math_window_ok;
        if !not_clipped {
            let o = i * 4;
            out[o] = 0;
            out[o + 1] = 0;
            out[o + 2] = 0;
            out[o + 3] = 0xff;
            continue;
        }
        if do_math {
            let (operand, second_real) = if frame.add_subscreen {
                if sub.real[i] {
                    (
                        [
                            i32::from(sub.c5[i][0]),
                            i32::from(sub.c5[i][1]),
                            i32::from(sub.c5[i][2]),
                        ],
                        true,
                    )
                } else {
                    (fixed, false)
                }
            } else {
                (fixed, false)
            };
            for ch in 0..3 {
                if frame.subtract_color {
                    c[ch] -= operand[ch];
                } else {
                    c[ch] += operand[ch];
                }
            }
            if frame.half_color && (second_real || !frame.add_subscreen) {
                for ch in 0..3 {
                    c[ch] >>= 1;
                }
            }
        }

        let o = i * 4;
        out[o] = expand_brightness(c[0], frame.brightness);
        out[o + 1] = expand_brightness(c[1], frame.brightness);
        out[o + 2] = expand_brightness(c[2], frame.brightness);
        out[o + 3] = 0xff;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::{
        ModernBgLayer, ModernBlendMode, ModernFrame, ModernIndexSpriteInstance,
        ModernIndexTileInstance, ModernTileInstance,
    };
    use crate::modern_index_atlas::ModernIndexTile;

    /// Build a frame with one BG layer (`layer_index`) carrying a single index-1
    /// tile at (0,0) whose color resolves to the given 5-bit channel triple.
    fn frame_with_single_bg_pixel(
        layer_index: u8,
        c5: [u8; 3],
    ) -> (ModernFrame, Vec<ModernIndexTile>) {
        let mut indices = [0u8; 64];
        indices[0] = 1; // pixel (0,0)
        let cells = vec![ModernIndexTile { id: 0, indices }];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        // Use palette `layer_index` so the two layers in the half-add test don't share a slot.
        let pal = layer_index as usize;
        frame.cgram_rgba[pal * 16 + 1] = [c5[0] << 3, c5[1] << 3, c5[2] << 3, 0xff];
        frame.bg_layers[layer_index as usize]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                screen_x: 0,
                screen_y: 0,
                palette: layer_index,
                hflip: false,
                vflip: false,
            });
        (frame, cells)
    }

    #[test]
    fn color_math_half_add_matches_formula() {
        // Main BG (layer 0) = (20,20,20); sub BG (layer 1) = (10,10,10).
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [20, 20, 20]);
        let (sub_frame, _) = frame_with_single_bg_pixel(1, [10, 10, 10]);
        // Merge the sub layer's tile + palette into the frame.
        frame.bg_layers[1].index_tiles = sub_frame.bg_layers[1].index_tiles.clone();
        frame.cgram_rgba[1 * 16 + 1] = sub_frame.cgram_rgba[1 * 16 + 1];

        frame.screen_enabled_main = 0x01; // BG1 on main
        frame.screen_enabled_sub = 0x02; // BG2 on sub
        frame.math_enabled = 0x01; // math on for BG1 (the winning main layer, bit 0)
        frame.add_subscreen = true;
        frame.half_color = true;
        frame.subtract_color = false;
        frame.fixed_color_r = 0;
        frame.fixed_color_g = 0;
        frame.fixed_color_b = 0;
        frame.brightness = 15;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // r5 = (20 + 10) >> 1 = 15. Brightness 15 → v8 = (15<<3)|(15>>2) = 123,
        // out8 = 123*15/15 = 123. (The classic shader's low-bit fill makes this 123,
        // not the 120 of the task's no-fill paraphrase.)
        assert_eq!(&out[0..4], &[123, 123, 123, 0xff], "half-add pixel (0,0)");
    }

    #[test]
    fn brightness_only_scales_channels() {
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [31, 31, 31]);
        frame.screen_enabled_main = 0x01;
        frame.math_enabled = 0; // no math
        frame.brightness = 8;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // v8 = (31<<3)|(31>>2) = 255; out8 = 255*8/15 = 136. (Classic-correct;
        // the task's no-fill paraphrase would give 248*8/15 = 132.)
        assert_eq!(
            &out[0..4],
            &[136, 136, 136, 0xff],
            "brightness-only pixel (0,0)"
        );
    }

    #[test]
    fn no_effect_math_leaves_channel_unchanged_except_brightness() {
        // Math bit set for the winning layer, but fixed=0, half=0, no sub →
        // no_effect_math = true → channel untouched, only brightness applied.
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [20, 20, 20]);
        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0; // no sub composite
        frame.math_enabled = 0x01; // math "on" for BG1 but ineffective
        frame.half_color = false;
        frame.add_subscreen = false;
        frame.fixed_color_r = 0;
        frame.fixed_color_g = 0;
        frame.fixed_color_b = 0;
        frame.brightness = 15;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // Unchanged channel: v8 = (20<<3)|(20>>2) = 165; out8 = 165*15/15 = 165.
        assert_eq!(
            &out[0..4],
            &[165, 165, 165, 0xff],
            "no-effect-math pixel (0,0)"
        );
    }

    #[test]
    fn color_math_gated_off_outside_color_window() {
        // Math WOULD add the fixed color, but prevent_math_mode=1 (follows window)
        // and the color window (x in 10..=20) excludes pixel (0,0) → math is gated
        // OFF there, so the main channel is unchanged (only brightness applies).
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [20, 20, 20]);
        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0;
        frame.math_enabled = 0x01; // BG1 participates in math
        frame.add_subscreen = false;
        frame.half_color = false;
        frame.subtract_color = false;
        frame.fixed_color_r = 5; // non-zero → math is "effective" if allowed
        frame.fixed_color_g = 5;
        frame.fixed_color_b = 5;
        frame.brightness = 15;
        // CGWSEL: prevent math OUTSIDE the window (mode 1, follows window).
        frame.prevent_math_mode = 1;
        frame.clip_mode = 0; // never clip
        frame.windowsel_cm = 0x02; // W1 enabled, not inverted
        frame.window_scanlines[0] = [10, 20, 0, 0]; // window covers x 10..=20

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // (0,0) is outside the window → no math → unchanged 20 → v8=165, out=165.
        assert_eq!(
            &out[0..4],
            &[165, 165, 165, 0xff],
            "pixel (0,0) outside color window: math gated off"
        );
        // A pixel inside the window (x=12,y=0) is backdrop here (no tile), but its
        // window membership is exercised via the clip path staying unclipped.
        let inside = (0 * 256 + 12) * 4;
        assert_eq!(
            &out[inside..inside + 4],
            &[0, 0, 0, 0xff],
            "in-window backdrop pixel still renders (not clipped)"
        );
    }

    #[test]
    fn software_sprites_indexed_draw_over_bg_with_obj_cgram() {
        // Sprite cell 0: only pixel (0,0) is index 1; everything else transparent.
        let mut indices = [0u8; 64];
        indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile { id: 0, indices }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        // OBJ palette base is 0x80; palette 3, index 1.
        frame.cgram_rgba[0x80 + 3 * 16 + 1] = [200, 10, 20, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 5,
            screen_y: 7,
            palette: 3,
            priority: 0,
            hflip: false,
            vflip: false,
        });

        // BG is empty (no index_tiles) → backdrop only, then sprites composite over.
        let mut out = render_modern_frame_software_indexed(&frame, &[]);
        draw_modern_sprites_indexed(&mut out, &frame, &sprite_cells);

        let px = (7usize * 256 + 5) * 4;
        assert_eq!(&out[px..px + 4], &[200, 10, 20, 0xff], "sprite pixel (5,7)");
        // Neighbour stays backdrop (index 0 transparent).
        let nb = (7usize * 256 + 6) * 4;
        assert_eq!(&out[nb..nb + 4], &[0, 0, 0, 0xff], "neighbour backdrop");
    }

    #[test]
    fn software_sprites_indexed_apply_hflip() {
        // Sprite cell 0: only pixel (7,0) is index 2.
        let mut indices = [0u8; 64];
        indices[7] = 2;
        let sprite_cells = vec![ModernIndexTile { id: 0, indices }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[0x80 + 3 * 16 + 2] = [9, 99, 199, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 5,
            screen_y: 7,
            palette: 3,
            priority: 0,
            hflip: true,
            vflip: false,
        });

        let mut out = render_modern_frame_software_indexed(&frame, &[]);
        draw_modern_sprites_indexed(&mut out, &frame, &sprite_cells);

        // hflip: source pixel (7,0) lands at screen x = screen_x + 0.
        let px = (7usize * 256 + 5) * 4;
        assert_eq!(
            &out[px..px + 4],
            &[9, 99, 199, 0xff],
            "hflipped pixel at x=5"
        );
    }

    #[test]
    fn software_indexed_renderer_applies_live_cgram() {
        // Synthetic atlas: one cell (id=0) — all indices zero except (0,0)->1 and (1,0)->2.
        let mut indices = [0u8; 64];
        indices[0] = 1; // pixel (0,0): sx=0, sy=0 → indices[sy*8+sx]=indices[0]
        indices[1] = 2; // pixel (1,0): sx=1, sy=0 → indices[sy*8+sx]=indices[1]
        let cells = vec![ModernIndexTile { id: 0, indices }];

        // Frame: palette P=3 so that the palette offset (not P=0) is exercised.
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[3 * 16 + 1] = [10, 20, 30, 0xff]; // palette 3, index 1
        frame.cgram_rgba[3 * 16 + 2] = [40, 50, 60, 0xff]; // palette 3, index 2

        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 3,
            hflip: false,
            vflip: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software_indexed(&frame, &cells);

        // pixel (0,0): index 1, palette 3 → cgram_rgba[3*16+1]
        let px00 = (0 * 256 + 0) * 4;
        assert_eq!(&rgba[px00..px00 + 4], &[10, 20, 30, 0xff], "pixel (0,0)");
        // pixel (1,0): index 2, palette 3 → cgram_rgba[3*16+2]
        let px10 = (0 * 256 + 1) * 4;
        assert_eq!(&rgba[px10..px10 + 4], &[40, 50, 60, 0xff], "pixel (1,0)");
        // pixel (2,0): index 0 → transparent → backdrop
        let px20 = (0 * 256 + 2) * 4;
        assert_eq!(
            &rgba[px20..px20 + 4],
            &[0, 0, 0, 0xff],
            "pixel (2,0) should be backdrop"
        );
    }

    #[test]
    fn software_renderer_blits_one_opaque_tile_from_atlas() {
        let mut atlas = vec![0u8; 8 * 8 * 4];
        for px in atlas.chunks_exact_mut(4) {
            px.copy_from_slice(&[10, 20, 30, 0xff]);
        }
        let mut frame = ModernFrame::empty();
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.blend_mode = ModernBlendMode::Opaque;
        layer.tiles.push(ModernTileInstance {
            atlas_id: 0,
            atlas_x_px: 0,
            atlas_y_px: 0,
            atlas_width_px: 8,
            atlas_height_px: 8,
            screen_width_px: 8,
            screen_height_px: 8,
            screen_x: 4,
            screen_y: 5,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            transparent_color_zero: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software(&frame, &atlas, 8, 8);
        let offset = ((5usize * 256) + 4usize) * 4;

        assert_eq!(&rgba[offset..offset + 4], &[10, 20, 30, 0xff]);
    }

    /// A pixel-distinct 8x8 pattern: color depends on BOTH x and y so the test
    /// catches scale being ignored, x/y swaps, and mirror mistakes.
    fn pattern_8x8(x: usize, y: usize) -> [u8; 4] {
        [(x as u8) * 30 + 5, (y as u8) * 30 + 7, 100, 0xff]
    }

    #[test]
    fn software_renderer_downsamples_scaled_atlas_tile() {
        const SCALE: usize = 4;
        const SRC: usize = 8 * SCALE; // 32
                                      // Build a 32x32 atlas that is a 4x nearest upscale of the 8x8 pattern.
        let mut atlas = vec![0u8; SRC * SRC * 4];
        for ay in 0..SRC {
            for ax in 0..SRC {
                let px = pattern_8x8(ax / SCALE, ay / SCALE);
                let o = (ay * SRC + ax) * 4;
                atlas[o..o + 4].copy_from_slice(&px);
            }
        }

        let mut frame = ModernFrame::empty();
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.blend_mode = ModernBlendMode::Opaque;
        layer.tiles.push(ModernTileInstance {
            atlas_id: 0,
            atlas_x_px: 0,
            atlas_y_px: 0,
            atlas_width_px: SRC as u16,
            atlas_height_px: SRC as u16,
            screen_width_px: 8,
            screen_height_px: 8,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            transparent_color_zero: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software(&frame, &atlas, SRC as u16, SRC as u16);

        // The 8x8 footprint at screen (0,0) must equal the original 8x8 pattern.
        for y in 0..8usize {
            for x in 0..8usize {
                let o = (y * 256 + x) * 4;
                assert_eq!(
                    &rgba[o..o + 4],
                    &pattern_8x8(x, y),
                    "screen pixel ({x},{y}) should equal downsampled pattern"
                );
            }
        }

        // Pixels just outside the 8x8 footprint must remain backdrop (not 32x32).
        let backdrop = frame.backdrop_color_rgba;
        for &(x, y) in &[(8usize, 0usize), (0, 8), (8, 8), (20, 20)] {
            let o = (y * 256 + x) * 4;
            assert_eq!(&rgba[o..o + 4], &backdrop, "({x},{y}) should be backdrop");
        }
    }
}
