use crate::gpu_frame::GpuFrame;
use crate::modern_assets::{atlas_entry_for_tilemap_entry, ModernTileAtlasAsset};
use crate::modern_frame::{
    ModernFrame, ModernIndexSpriteInstance, ModernIndexTileInstance, ModernTileInstance,
};
use crate::modern_index_atlas::{index_cell_for_tilemap_entry, ModernIndexAtlas, ModernIndexTile};
use crate::modern_sprite_atlas::{sprite_index_cell, ModernSpriteIndexAtlas};

// Per SNES PPU OBSEL: maps obj_size (3-bit index) to [small_px, large_px].
// Copied from sprite_renderer::SPRITE_SIZES so the modern OAM enumeration matches
// the classic resolver's size selection exactly.
const SPRITE_SIZES: [[u8; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

/// Screen Y of an OBJ 8×8 tile, honoring the SNES 256-line OBJ-Y wrap. The classic
/// `resolve_obj_pixels` visibility test is `row = (line - yy) & 0xff < size`, i.e. a
/// sprite whose unwrapped top sits at/below the bottom edge (`top_y + sty*8 >= 224`)
/// reappears at the TOP via the mod-256 wrap (high-Y sprites — e.g. statue tops that
/// straddle scanline 0). Remapping such a tile's screen Y by `-256` reproduces that
/// wrap exactly: tiles in 249..262 become -7..6 (visible at top); 224..248 become
/// -32..-8 (off-screen either way). The per-pixel clip then handles the boundary.
fn obj_tile_screen_y(top_y: i32, sty: i32) -> i16 {
    let y = top_y + sty * 8;
    (if y >= 224 { y - 256 } else { y }) as i16
}

/// Replicate the SNES per-scanline OBJ range/time-over limits (max 32 sprites and
/// 34 tile-columns per scanline) EXACTLY as `sprite_renderer::resolve_obj_pixels`
/// selects them, returning — per scanline 0..224 — the `(sprite_num, tile_left_x)`
/// tile-columns that survive the budget and are therefore actually drawn.
///
/// The modern OBJ compositor is instance-based (one instance per 8×8 tile) and
/// otherwise draws every sprite, so without this it renders sprites the real PPU
/// drops on crowded scanlines (e.g. frame 198300: 33 sprites on lines 118-119 →
/// classic drops sprite_num≥66, modern kept it → a 30px right-edge diff). Each
/// emitted instance is gated per-row against this table via `row_mask`.
fn compute_obj_drawn_tiles(frame: &GpuFrame<'_>) -> Vec<Vec<(u8, i16)>> {
    let oam = frame.oam;
    let obj = &frame.obj;
    let extra = frame.extra_left_right as i32;
    let mut drawn: Vec<Vec<(u8, i16)>> = vec![Vec::new(); 224];

    for line in 1..=224i32 {
        // +1 sentinels: 33 sprites / 35 tiles → the 33rd sprite and 35th tile are
        // the ones that make the counter hit 0 and are NOT drawn (range/time over).
        let mut sprites_left = 33i32;
        let mut tiles_left = 35i32;
        let mut sprites: Vec<(usize, i32, i32)> = Vec::with_capacity(34); // (sprite_num, x, size)

        for sprite_num in 0..128usize {
            let idx = sprite_num * 2;
            let oam0 = oam.get(idx).copied().unwrap_or(0);
            let yy = (((oam0 >> 8) as i32) + 1) & 0xff;
            if yy == 0xf0 {
                continue;
            }
            let row = (line - yy) & 0xff;
            let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
            let hi_bits = (hi_word >> (idx % 16)) as i32;
            let sprite_size =
                SPRITE_SIZES[(obj.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize] as i32;
            if row >= sprite_size {
                continue;
            }
            let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
            if object_x > 256 && object_x + sprite_size - 1 < 512 {
                continue;
            }
            let mut x = object_x;
            if x >= 256 + extra {
                x -= 512;
            }
            if x <= -(sprite_size + extra) {
                continue;
            }
            sprites_left -= 1;
            if sprites_left == 0 {
                break;
            }
            sprites.push((sprite_num, x, sprite_size));
        }

        let out_y = (line - 1) as usize;
        'tiles: for (sprite_num, sx, sprite_size) in sprites {
            let mut col = 0;
            while col < sprite_size {
                if col + sx <= -8 - extra || col + sx >= 256 + extra {
                    col += 8;
                    continue;
                }
                tiles_left -= 1;
                if tiles_left == 0 {
                    break 'tiles;
                }
                drawn[out_y].push((sprite_num as u8, (sx + col) as i16));
                col += 8;
            }
        }
    }

    drawn
}

/// Per-row visibility mask (bit r = output row `screen_y + r` survives the OBJ
/// per-scanline budget) for a tile of `sprite_num` whose left edge is `screen_x`.
fn obj_row_mask(drawn: &[Vec<(u8, i16)>], sprite_num: usize, screen_x: i16, screen_y: i16) -> u8 {
    let mut mask = 0u8;
    for r in 0..8i32 {
        let dy = screen_y as i32 + r;
        if (0..224).contains(&dy)
            && drawn[dy as usize]
                .iter()
                .any(|&(sn, tx)| sn as usize == sprite_num && tx == screen_x)
        {
            mask |= 1 << r;
        }
    }
    mask
}

/// Decode OAM into palette-index sprite-tile instances, mirroring the per-sprite,
/// per-8×8-tile ENUMERATION of `sprite_renderer::resolve_obj_pixels` (NOT its
/// per-pixel resolver).
///
/// For each of the 128 OAM sprites: skip the off-screen sentinel (`y == 0xf0`),
/// derive size from `obj.obj_size` + the OAM hi-table, cull horizontally exactly
/// like the reference, then emit one `ModernIndexSpriteInstance` per 8×8 tile whose
/// UNFLIPPED pattern is present in `atlas` for `(context, effective_tile)`. The
/// instance carries the OAM palette/priority/flip; the renderer (Task 5) applies
/// `hflip`/`vflip` to the cell's 8×8 pixels.
pub fn extract_modern_sprites(
    frame: &GpuFrame<'_>,
    atlas: &ModernSpriteIndexAtlas,
    context: u64,
) -> Vec<ModernIndexSpriteInstance> {
    let oam = frame.oam;
    let obj = &frame.obj;
    let drawn = compute_obj_drawn_tiles(frame);
    let mut out = Vec::new();

    for sprite_num in 0..128usize {
        let idx = sprite_num * 2;
        let oam0 = oam.get(idx).copied().unwrap_or(0);

        // Off-screen sentinel: the game parks hidden sprites at y == 0xf0.
        let y_byte = ((oam0 >> 8) & 0xff) as i32;
        if y_byte == 0xf0 {
            continue;
        }
        // On-screen top row = ((Y+1)&0xff) - 1 (= Y for Y < 0xff), matching the
        // reference's `yy = ((oam0>>8)+1)&0xff` then `out_y = line - 1`.
        let top_y = ((y_byte + 1) & 0xff) - 1;

        let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
        let hi_bits = (hi_word >> (idx % 16)) as i32;
        let size = SPRITE_SIZES[(obj.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize] as i32;

        let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
        // extra_left_right = 0: replicate the reference's horizontal cull.
        if object_x > 256 && object_x + size - 1 < 512 {
            continue;
        }
        let mut x = object_x;
        if x >= 256 {
            x -= 512;
        }
        if x <= -size {
            continue;
        }

        let oam1 = oam.get(idx + 1).copied().unwrap_or(0);
        let hflip = oam1 & 0x4000 != 0;
        let vflip = oam1 & 0x8000 != 0;
        let palette = ((oam1 & 0x0e00) >> 9) as u8;
        let priority = ((oam1 & 0x3000) >> 12) as u8;
        let bank: u16 = if oam1 & 0x0100 != 0 { 256 } else { 0 };
        let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
        let tile_col_base = (oam1 & 0x0f) as i32;

        let tiles_per_side = size / 8;
        for sty in 0..tiles_per_side {
            for stx in 0..tiles_per_side {
                // Source tile honoring flip at TILE granularity (matches the
                // reference's used_col = hflip ? size-1-col : col, taken >> 3).
                let src_col_tile = if hflip { tiles_per_side - 1 - stx } else { stx };
                let src_row_tile = if vflip { tiles_per_side - 1 - sty } else { sty };
                let used_tile = (((tile_row_base + src_row_tile) << 4)
                    | ((tile_col_base + src_col_tile) & 0x0f))
                    as u16;
                let effective_tile = bank + used_tile;

                let screen_x = (x + stx * 8) as i16;
                let screen_y = obj_tile_screen_y(top_y, sty);
                let row_mask = obj_row_mask(&drawn, sprite_num, screen_x, screen_y);
                if row_mask == 0 {
                    continue; // fully dropped by the per-scanline OBJ budget
                }
                let Some(cell) = sprite_index_cell(atlas, context, effective_tile) else {
                    continue;
                };
                out.push(ModernIndexSpriteInstance {
                    cell_id: cell.id,
                    screen_x,
                    screen_y,
                    palette,
                    priority,
                    hflip,
                    vflip,
                    row_mask,
                });
            }
        }
    }

    out
}

/// Decode one SNES 4bpp 8×8 OBJ/BG tile from live VRAM into 64 palette indices
/// (row-major, 0–15). `chr_base_words` is the VRAM word base of the CHR page;
/// `tilemap_entry` carries the tile number in bits [9:0] and flip in bits 14/15.
///
/// Mirrors `decode_snes_4bpp_tile_indices` in the binary: each 8×8 tile occupies
/// 16 words (two bitplane words per row, planes 0+1 then 2+3), pixel x=0 is the
/// MSB. Call with NO flip bits to obtain the UNFLIPPED pattern (the renderer
/// applies hflip/vflip when sampling the cell).
pub fn decode_snes_4bpp_tile_indices(
    vram: &[u16],
    chr_base_words: usize,
    tilemap_entry: u16,
) -> [u8; 64] {
    let tile_number = usize::from(tilemap_entry & 0x03ff);
    let hflip = tilemap_entry & 0x4000 != 0;
    let vflip = tilemap_entry & 0x8000 != 0;
    let tile_base = chr_base_words + tile_number * 16;
    let mut out = [0u8; 64];
    for y in 0..8usize {
        let source_y = if vflip { 7 - y } else { y };
        let w01 = vram.get(tile_base + source_y).copied().unwrap_or(0);
        let w23 = vram.get(tile_base + 8 + source_y).copied().unwrap_or(0);
        let (bp0, bp1) = ((w01 & 0xff) as u8, (w01 >> 8) as u8);
        let (bp2, bp3) = ((w23 & 0xff) as u8, (w23 >> 8) as u8);
        for x in 0..8usize {
            let source_x = if hflip { x } else { 7 - x };
            let bit = 1u8 << source_x;
            out[y * 8 + x] = ((bp0 & bit != 0) as u8)
                | (((bp1 & bit != 0) as u8) << 1)
                | (((bp2 & bit != 0) as u8) << 2)
                | (((bp3 & bit != 0) as u8) << 3);
        }
    }
    out
}

/// Decode one SNES 2bpp 8×8 tile from live VRAM into 64 palette indices (0–3),
/// applying `tilemap_entry` flip. Each tile occupies 8 words (one word = two
/// bitplanes per row); pixel x=0 is the MSB. Used for BG3 in PPU mode 1 (the
/// dungeon HUD/message layer), which is 2bpp — decoding it as 4bpp reads adjacent
/// tile data as planes 2/3 and produces garbage indices.
pub fn decode_snes_2bpp_tile_indices(
    vram: &[u16],
    chr_base_words: usize,
    tilemap_entry: u16,
) -> [u8; 64] {
    let tile_number = usize::from(tilemap_entry & 0x03ff);
    let hflip = tilemap_entry & 0x4000 != 0;
    let vflip = tilemap_entry & 0x8000 != 0;
    let tile_base = chr_base_words + tile_number * 8;
    let mut out = [0u8; 64];
    for y in 0..8usize {
        let source_y = if vflip { 7 - y } else { y };
        let w01 = vram.get(tile_base + source_y).copied().unwrap_or(0);
        let (bp0, bp1) = ((w01 & 0xff) as u8, (w01 >> 8) as u8);
        for x in 0..8usize {
            let source_x = if hflip { x } else { 7 - x };
            let bit = 1u8 << source_x;
            out[y * 8 + x] = ((bp0 & bit != 0) as u8) | (((bp1 & bit != 0) as u8) << 1);
        }
    }
    out
}

/// Extract a dungeon `ModernFrame` whose BG tile patterns are decoded from LIVE
/// VRAM (not the static `dungeon_index_tiles` atlas), returning the decoded cell
/// set alongside.
///
/// The static dungeon atlas bakes each `(theme, tile#)` pattern from a snapshot of
/// CHR, but several dungeon BG tiles are ANIMATED — their CHR is re-DMA'd into VRAM
/// each frame (e.g. the animated floor tile #255, water/lava). A fixed atlas cannot
/// follow that, so those tiles render with a stale palette index. Because the
/// dungeon floor is composited as backdrop + BG1-subscreen color-math, a stale BG1
/// index shows up as a ~1-LSB-low blended floor versus the classic renderer (which
/// reads live VRAM). Decoding the BG CHR straight from `frame.vram` — exactly as
/// `extract_modern_sprites_from_vram` does for OBJ — makes BG byte-exact with the
/// classic PPU.
///
/// BG1/BG2 (4bpp) and BG3 (the 2bpp HUD/message layer in mode 1) are all decoded.
/// BG3 is decoded as 2bpp (8 words/tile) and its classic CGRAM mapping
/// (`cgram_idx = palette*4 + pal_idx`, low CGRAM, 4 colors/palette — see
/// `bg_layer.wgsl`) is BAKED into the cell indices, with the instance palette set
/// to 0 so the compositor's `cgram_rgba[palette*16 + index]` resolves to
/// `cgram_rgba[cgram_idx]` exactly like the classic renderer. This makes the
/// dungeon HUD (LIFE/hearts/magic/items) render correctly.
/// Geometry (64×64 four-quadrant tilemap, main||sub visibility, scroll) matches
/// [`extract_modern_frame_with_dungeon_atlas`]. Cells are deduplicated by
/// `(layer CHR base, tile# + flip, bpp)` and store the FLIP-BAKED 8×8 pattern (the
/// dungeon composite samples cells without re-applying flip). The backdrop is set
/// from CGRAM[0] (the classic main color-math operand for backdrop pixels).
pub fn extract_modern_dungeon_frame_from_vram(
    frame: &GpuFrame<'_>,
) -> (ModernFrame, Vec<ModernIndexTile>) {
    use std::collections::HashMap;
    let mut modern = extract_modern_frame(frame);
    modern.cgram_rgba = crate::modern_palette::cgram_words_to_rgba256(frame.cgram);
    modern.backdrop_color_rgba =
        crate::modern_palette::snes_cgram_to_rgba(*frame.cgram.first().unwrap_or(&0));

    let mut cells: Vec<ModernIndexTile> = Vec::new();
    // key: (CHR word base, tilemap word masked to tile#+flip) -> cell id
    let mut cell_ids: HashMap<(usize, u16), u32> = HashMap::new();

    // BG1 (floor/statues, the subscreen color-math operand) and BG2 (walls) are
    // decoded as 4bpp; BG3 (the 2bpp HUD/message layer in mode 1) is decoded as
    // 2bpp with its CGRAM mapping baked into the cell (see below) so the dungeon
    // HUD renders correctly.
    for layer_index in 0..3usize {
        // BG3 in PPU mode 1 is 2bpp; BG1/BG2 are 4bpp.
        let is_2bpp = layer_index == 2;
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        let enabled_sub = frame.screen_enabled[1] & (1 << layer_index) != 0;
        let enabled = enabled_main || enabled_sub;
        modern.bg_layers[layer_index].enabled_main = enabled;
        modern.bg_layers[layer_index].enabled_sub = enabled_sub;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        let chr_base = frame.bg[layer_index].tile_adr as usize;
        let h_scroll = frame.bg[layer_index].h_scroll;
        let v_scroll = frame.bg[layer_index].v_scroll;
        let wide = frame.bg[layer_index].tilemap_wider;
        let tall = frame.bg[layer_index].tilemap_higher;
        let cols = if wide { 64usize } else { 32 };
        let rows = if tall { 64usize } else { 32 };
        for ty in 0..rows {
            for tx in 0..cols {
                let q = (if wide && tx >= 32 { 1 } else { 0 })
                    + (if tall && ty >= 32 {
                        if wide {
                            2
                        } else {
                            1
                        }
                    } else {
                        0
                    });
                let within = (ty % 32) * 32 + (tx % 32);
                let addr = base + q * 0x400 + within;
                let entry_word = *frame.vram.get(addr).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                // tile number (bits 0-9) + flip (bits 14/15); palette/priority dropped.
                let pattern_key = entry_word & 0xC3FF;
                let palette = ((entry_word >> 10) & 7) as u8;
                // For 2bpp BG3 the palette is BAKED into the cell (see below), so the
                // dedup key must include the palette bits; the 4bpp path keeps the
                // palette on the instance and keys on tile#+flip only. The chr_base is
                // tagged for 2bpp so a 2bpp and 4bpp cell at the same base never alias.
                let (map_base, map_key) = if is_2bpp {
                    (chr_base | (1usize << 28), entry_word & 0xDFFF)
                } else {
                    (chr_base, pattern_key)
                };
                let cell_id = *cell_ids.entry((map_base, map_key)).or_insert_with(|| {
                    // Bake flip into the cell: the dungeon composite samples without flip.
                    let indices = if is_2bpp {
                        // BG3 2bpp: decode 0..3, then bake the classic BG3→CGRAM
                        // mapping (cgram_idx = palette*4 + pal_idx, 4 colors/palette in
                        // low CGRAM; bg_layer.wgsl). pal_idx 0 stays transparent.
                        let raw = decode_snes_2bpp_tile_indices(frame.vram, chr_base, pattern_key);
                        let mut baked = [0u8; 64];
                        for (b, &p) in baked.iter_mut().zip(raw.iter()) {
                            *b = if p == 0 { 0 } else { palette * 4 + p };
                        }
                        baked
                    } else {
                        decode_snes_4bpp_tile_indices(frame.vram, chr_base, pattern_key)
                    };
                    let id = cells.len() as u32;
                    cells.push(ModernIndexTile {
                        id,
                        indices,
                        source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                        hflip: false,
                        vflip: false,
                    });
                    id
                });
                // The SNES BG is a torus: scroll wraps modulo the tilemap pixel
                // size (cols*8 / rows*8), so a tile scrolled past one edge reappears
                // on the opposite edge. Without this wrap, wide/tall (64-tile, 512px)
                // dungeon tilemaps with a large scroll (e.g. h_scroll=640 on a 512px
                // BG) push EVERY tile off-screen, leaving the room BG black. Bring the
                // wrapped position into the visible window [-(bg-screen)..screen).
                let bg_w = (cols * 8) as i32;
                let bg_h = (rows * 8) as i32;
                let mut sx = ((tx * 8) as i32 - h_scroll as i32).rem_euclid(bg_w);
                if sx >= 256 {
                    sx -= bg_w;
                }
                // The SNES PPU fetches BG row `sy + 1` for output scanline `sy`
                // (vertical +1 offset; see bg_layer.wgsl `source_y = sy + 1`). Apply
                // it before wrapping so the modern BG aligns with the classic render.
                let mut sy = ((ty * 8) as i32 - v_scroll as i32 - 1).rem_euclid(bg_h);
                if sy >= 224 {
                    sy -= bg_h;
                }
                modern.bg_layers[layer_index]
                    .index_tiles
                    .push(ModernIndexTileInstance {
                        cell_id,
                        screen_x: sx as i16,
                        screen_y: sy as i16,
                        // 2bpp BG3 bakes the CGRAM index into the cell → palette 0.
                        palette: if is_2bpp { 0 } else { palette },
                        hflip: false,
                        vflip: false,
                        priority: entry_word & 0x2000 != 0,
                    });
            }
        }
    }
    (modern, cells)
}

/// Live-VRAM BG decode for the OVERWORLD (PPU mode 1, main module 9).
///
/// The overworld uses the same SNES BG structure as the dungeon — BG1/BG2 are
/// 4bpp, BG3 is the 2bpp HUD/message layer, all addressed through
/// `frame.bg[layer].tilemap_adr` / `tile_adr` in `frame.vram` — so it shares the
/// dungeon's mode-agnostic live-VRAM decoder verbatim: live tilemap/CHR read, the
/// SNES +1 vertical fetch offset, scroll-wrap (torus) for wide/tall maps, the
/// `cgram_rgba` fill, BG3 2bpp CGRAM baking, and `enabled_main || enabled_sub`
/// visibility. This replaces the STATIC index atlas (which cannot follow the
/// overworld's per-frame animated CHR / palette), making the overworld BG
/// byte-exact with the classic PPU the same way the dungeon already is.
///
/// `extract_modern_dungeon_frame_from_vram` is genuinely mode-independent (it reads
/// only `frame.bg[]`/`frame.vram`/`frame.cgram`/`frame.screen_enabled`), so this is
/// a thin alias rather than a copy; both modes drive the same decode path.
pub fn extract_modern_frame_from_vram(frame: &GpuFrame<'_>) -> (ModernFrame, Vec<ModernIndexTile>) {
    extract_modern_dungeon_frame_from_vram(frame)
}

/// Decode OAM into palette-index sprite-tile instances AND their UNIQUE 8×8 tile
/// patterns decoded from LIVE VRAM, mirroring the per-sprite, per-8×8-tile
/// ENUMERATION of `extract_modern_sprites` (same culling, sizes, flip, bank, and
/// tile addressing).
///
/// Unlike the static-atlas variant, this resolves each sprite tile's pixels from
/// the current frame's sprite CHR (`frame.obj.tile_adr1` / `tile_adr2`, selected
/// by the OAM bank bit) in `frame.vram`. Sprite CHR is DMA'd per frame, so this is
/// the only way modern rendering can follow animations (Link's poses, dynamic
/// sprites) — a fixed `(context, tile)` atlas cannot.
///
/// Returns `(cells, instances)`: `cells` is the deduplicated set of UNFLIPPED 8×8
/// patterns (sequential ids, ready for `draw_modern_sprites_indexed`); each
/// instance references its pattern by `cell_id` and carries the OAM
/// palette/priority/flip and on-screen position. Fully transparent (all-zero)
/// patterns are skipped.
pub fn extract_modern_sprites_from_vram(
    frame: &GpuFrame<'_>,
) -> (Vec<ModernIndexTile>, Vec<ModernIndexSpriteInstance>) {
    use std::collections::HashMap;

    let oam = frame.oam;
    let obj = &frame.obj;
    let drawn = compute_obj_drawn_tiles(frame);
    let mut cells: Vec<ModernIndexTile> = Vec::new();
    let mut pattern_ids: HashMap<[u8; 64], u32> = HashMap::new();
    let mut out = Vec::new();

    for sprite_num in 0..128usize {
        let idx = sprite_num * 2;
        let oam0 = oam.get(idx).copied().unwrap_or(0);

        // Off-screen sentinel: the game parks hidden sprites at y == 0xf0.
        let y_byte = ((oam0 >> 8) & 0xff) as i32;
        if y_byte == 0xf0 {
            continue;
        }
        let top_y = ((y_byte + 1) & 0xff) - 1;

        let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
        let hi_bits = (hi_word >> (idx % 16)) as i32;
        let size = SPRITE_SIZES[(obj.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize] as i32;

        let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
        if object_x > 256 && object_x + size - 1 < 512 {
            continue;
        }
        let mut x = object_x;
        if x >= 256 {
            x -= 512;
        }
        if x <= -size {
            continue;
        }

        let oam1 = oam.get(idx + 1).copied().unwrap_or(0);
        let hflip = oam1 & 0x4000 != 0;
        let vflip = oam1 & 0x8000 != 0;
        let palette = ((oam1 & 0x0e00) >> 9) as u8;
        let priority = ((oam1 & 0x3000) >> 12) as u8;
        // Decode from the correct CHR page directly (the OAM bank bit selects the
        // word base), so `used_tile` stays 0..255 within that page.
        let bank_base = if oam1 & 0x0100 != 0 {
            obj.tile_adr2
        } else {
            obj.tile_adr1
        };
        let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
        let tile_col_base = (oam1 & 0x0f) as i32;

        let tiles_per_side = size / 8;
        for sty in 0..tiles_per_side {
            for stx in 0..tiles_per_side {
                let src_col_tile = if hflip { tiles_per_side - 1 - stx } else { stx };
                let src_row_tile = if vflip { tiles_per_side - 1 - sty } else { sty };
                let used_tile = (((tile_row_base + src_row_tile) << 4)
                    | ((tile_col_base + src_col_tile) & 0x0f))
                    as u16;

                // Decode the UNFLIPPED 8×8 pattern (no flip bits): the renderer
                // applies hflip/vflip per instance when sampling the cell.
                let indices =
                    decode_snes_4bpp_tile_indices(frame.vram, bank_base as usize, used_tile);

                // Skip fully transparent tiles to avoid cluttering the cell set.
                if indices.iter().all(|&i| i == 0) {
                    continue;
                }

                let screen_x = (x + stx * 8) as i16;
                let screen_y = obj_tile_screen_y(top_y, sty);
                let row_mask = obj_row_mask(&drawn, sprite_num, screen_x, screen_y);
                if row_mask == 0 {
                    continue; // fully dropped by the per-scanline OBJ budget
                }

                let cell_id = *pattern_ids.entry(indices).or_insert_with(|| {
                    let id = cells.len() as u32;
                    cells.push(ModernIndexTile {
                        id,
                        indices,
                        source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                        hflip: false,
                        vflip: false,
                    });
                    id
                });

                out.push(ModernIndexSpriteInstance {
                    cell_id,
                    screen_x,
                    screen_y,
                    palette,
                    priority,
                    hflip,
                    vflip,
                    row_mask,
                });
            }
        }
    }

    (cells, out)
}

/// Decoded visual fields from a single SNES BG tilemap entry (u16).
///
/// Bits [9:0]  → tile_number
/// Bits [12:10] → palette
/// Bit  13     → priority
/// Bit  14     → hflip
/// Bit  15     → vflip
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernTileFields {
    pub tile_number: u16,
    pub palette: u8,
    pub priority: bool,
    pub hflip: bool,
    pub vflip: bool,
}

/// Decode one SNES BG tilemap entry word into its visual sub-fields.
pub fn decode_snes_tilemap_entry(entry: u16) -> ModernTileFields {
    ModernTileFields {
        tile_number: entry & 0x03ff,
        palette: ((entry >> 10) & 0x07) as u8,
        priority: entry & 0x2000 != 0,
        hflip: entry & 0x4000 != 0,
        vflip: entry & 0x8000 != 0,
    }
}

/// Extract frame-level visual state and BG tile instances from a `GpuFrame` into a `ModernFrame`,
/// mapping each tilemap entry to its atlas tile via `atlas_entry_for_tilemap_entry`.
///
/// Loops layers 0..3, gates on the main-screen enable bit, reads VRAM tilemap entries,
/// skips zeroes, looks up the atlas entry, and pushes a `ModernTileInstance` for each hit.
/// Screen position is `col*8 - h_scroll` / `row*8 - v_scroll`.
pub fn extract_modern_frame_with_atlas(
    frame: &GpuFrame<'_>,
    atlas: &ModernTileAtlasAsset,
) -> ModernFrame {
    let mut modern = extract_modern_frame(frame);
    for layer_index in 0..3usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].enabled_main = enabled_main;
        modern.bg_layers[layer_index].enabled_sub =
            frame.screen_enabled[1] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled_main {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        for row in 0..32usize {
            for col in 0..32usize {
                let entry_word = *frame.vram.get(base + row * 32 + col).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                let Some(atlas_entry) = atlas_entry_for_tilemap_entry(atlas, entry_word) else {
                    continue;
                };
                let fields = decode_snes_tilemap_entry(entry_word);
                let scale = atlas.atlas_scale.max(1);
                modern.bg_layers[layer_index]
                    .tiles
                    .push(ModernTileInstance {
                        atlas_id: atlas_entry.id,
                        atlas_x_px: atlas_entry.atlas_x_px,
                        atlas_y_px: atlas_entry.atlas_y_px,
                        atlas_width_px: atlas_entry.atlas_width_px,
                        atlas_height_px: atlas_entry.atlas_height_px,
                        screen_width_px: atlas_entry.atlas_width_px / scale,
                        screen_height_px: atlas_entry.atlas_height_px / scale,
                        screen_x: (col * 8) as i16 - frame.bg[layer_index].h_scroll as i16,
                        screen_y: (row * 8) as i16 - frame.bg[layer_index].v_scroll as i16,
                        palette: fields.palette,
                        priority: u8::from(fields.priority),
                        // atlas bakes the word's flip into the cell appearance; do not re-apply
                        // flip here — doing so would double-flip asymmetric tiles.
                        hflip: false,
                        vflip: false,
                        transparent_color_zero: true,
                    });
            }
        }
    }
    modern
}

/// Extract frame-level visual state and indexed BG tile instances from a `GpuFrame`
/// into a `ModernFrame`, using a palette-index atlas.
///
/// Sets `cgram_rgba` from `frame.cgram`. For layers 0..3 enabled on the main screen,
/// reads the 32×32 tilemap from `frame.bg[layer].tilemap_adr`, looks up each nonzero
/// word in the index atlas (by `word & 0xC3FF`), and pushes a `ModernIndexTileInstance`
/// with the per-word palette, screen position, and `hflip/vflip` fixed false (the atlas
/// index pattern already bakes flip).
pub fn extract_modern_frame_with_index_atlas(
    frame: &GpuFrame<'_>,
    atlas: &ModernIndexAtlas,
) -> ModernFrame {
    let mut modern = extract_modern_frame(frame);
    modern.cgram_rgba = crate::modern_palette::cgram_words_to_rgba256(frame.cgram);
    for layer_index in 0..3usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].enabled_main = enabled_main;
        modern.bg_layers[layer_index].enabled_sub =
            frame.screen_enabled[1] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled_main {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        let h_scroll = frame.bg[layer_index].h_scroll;
        let v_scroll = frame.bg[layer_index].v_scroll;
        for row in 0..32usize {
            for col in 0..32usize {
                let entry_word = *frame.vram.get(base + row * 32 + col).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                let Some(cell) = index_cell_for_tilemap_entry(atlas, entry_word) else {
                    continue;
                };
                modern.bg_layers[layer_index]
                    .index_tiles
                    .push(ModernIndexTileInstance {
                        cell_id: cell.id,
                        screen_x: (col * 8) as i16 - h_scroll as i16,
                        screen_y: (row * 8) as i16 - v_scroll as i16,
                        palette: ((entry_word >> 10) & 7) as u8,
                        hflip: false,
                        vflip: false,
                        priority: entry_word & 0x2000 != 0,
                    });
            }
        }
    }
    modern
}

/// Extract frame-level visual state and indexed BG tile instances from a `GpuFrame`
/// into a `ModernFrame`, using a dungeon palette-index atlas keyed by `(theme, graphics_key)`.
///
/// Sets `cgram_rgba` from `frame.cgram`. For layers 0..3 enabled on the main screen OR
/// subscreen (dungeon BG1 walls/statues use color-math / subscreen), reads the full SNES
/// tilemap honoring `tilemap_wider`/`tilemap_higher` (up to 64×64 tiles) with correct
/// four-quadrant VRAM addressing (each 32×32 block at base + quadrant × 0x400), looks up
/// each nonzero word in the dungeon atlas via `dungeon_index_cell(atlas, theme, word)`, and
/// pushes a `ModernIndexTileInstance` with the per-word palette, screen position, and
/// `hflip/vflip` fixed false (the atlas index pattern already bakes flip).
pub fn extract_modern_frame_with_dungeon_atlas(
    frame: &GpuFrame<'_>,
    atlas: &crate::modern_dungeon_atlas::ModernDungeonIndexAtlas,
    theme: u16,
) -> ModernFrame {
    let mut modern = extract_modern_frame(frame);
    modern.cgram_rgba = crate::modern_palette::cgram_words_to_rgba256(frame.cgram);
    for layer_index in 0..3usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        let enabled_sub = frame.screen_enabled[1] & (1 << layer_index) != 0;
        // Render if enabled on main OR sub — dungeon BG1 (the room floor/walls) is on the
        // subscreen for color-math and is NOT set on the main screen. The simplified indexed
        // renderer only draws layers whose `enabled_main` is true, so persist the combined
        // (main || sub) visibility into `enabled_main`; otherwise the subscreen floor is
        // extracted but never drawn, leaving the room black behind the HUD.
        let enabled = enabled_main || enabled_sub;
        modern.bg_layers[layer_index].enabled_main = enabled;
        modern.bg_layers[layer_index].enabled_sub = enabled_sub;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        let h_scroll = frame.bg[layer_index].h_scroll;
        let v_scroll = frame.bg[layer_index].v_scroll;
        // Dungeon BG tilemaps are 64×64 (four 32×32 quadrants at base+0x000/0x400/0x800/0xC00).
        // Quadrant index = (right_half ? 1 : 0) + (bottom_half ? (wide ? 2 : 1) : 0).
        let wide = frame.bg[layer_index].tilemap_wider;
        let tall = frame.bg[layer_index].tilemap_higher;
        let cols = if wide { 64usize } else { 32 };
        let rows = if tall { 64usize } else { 32 };
        for ty in 0..rows {
            for tx in 0..cols {
                let q = (if wide && tx >= 32 { 1 } else { 0 })
                    + (if tall && ty >= 32 {
                        if wide {
                            2
                        } else {
                            1
                        }
                    } else {
                        0
                    });
                let within = (ty % 32) * 32 + (tx % 32);
                let addr = base + q * 0x400 + within;
                let entry_word = *frame.vram.get(addr).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                let Some(cell) =
                    crate::modern_dungeon_atlas::dungeon_index_cell(atlas, theme, entry_word)
                else {
                    continue;
                };
                modern.bg_layers[layer_index]
                    .index_tiles
                    .push(ModernIndexTileInstance {
                        cell_id: cell.id,
                        screen_x: (tx * 8) as i16 - h_scroll as i16,
                        screen_y: (ty * 8) as i16 - v_scroll as i16,
                        palette: ((entry_word >> 10) & 7) as u8,
                        hflip: false,
                        vflip: false,
                        priority: entry_word & 0x2000 != 0,
                    });
            }
        }
    }
    modern
}

/// Extract frame-level visual state from a `GpuFrame` into a `ModernFrame`.
///
/// This function copies brightness and forced-blank from the GPU frame.
/// BG layer tile extraction will be added in a subsequent task.
pub fn extract_modern_frame(frame: &GpuFrame<'_>) -> ModernFrame {
    let mut modern = ModernFrame::empty();
    modern.brightness = frame.brightness;
    modern.forced_blank = frame.forced_blank;
    // Raw screen-enable bits + color-math registers for the full software path.
    modern.screen_enabled_main = frame.screen_enabled[0];
    modern.screen_enabled_sub = frame.screen_enabled[1];
    modern.math_enabled = frame.math_enabled;
    modern.subtract_color = frame.subtract_color;
    modern.half_color = frame.half_color;
    modern.fixed_color_r = frame.fixed_color_r;
    modern.fixed_color_g = frame.fixed_color_g;
    modern.fixed_color_b = frame.fixed_color_b;
    modern.add_subscreen = frame.add_subscreen;
    // SNES mosaic ($2106): per-BG enable bits + block size, for the mosaic path in
    // the Mode-1 compositor (matches the classic bg_layer.wgsl source-snap).
    modern.mosaic_enabled = frame.mosaic_enabled;
    modern.mosaic_size = frame.mosaic_size;
    // Color-window registers + per-scanline boundaries for the full software path
    // (mirrors the GPU post-process color-math window gating).
    modern.clip_mode = frame.clip_mode;
    modern.prevent_math_mode = frame.prevent_math_mode;
    modern.windowsel_cm = frame.windowsel_cm;
    // Full per-layer window-select + main-screen window-enable (TMW) for the
    // Mode-1 compositor's main-screen window mask (mirrors bg_layer.wgsl /
    // sprite_pixels.wgsl `*_window_active`).
    modern.windowsel = frame.windowsel;
    modern.screen_windowed_main = frame.screen_windowed[0];
    modern.screen_windowed_sub = frame.screen_windowed[1];
    for (dst, sl) in modern
        .window_scanlines
        .iter_mut()
        .zip(frame.scanlines.iter())
    {
        *dst = [
            sl.window1_left,
            sl.window1_right,
            sl.window2_left,
            sl.window2_right,
        ];
    }
    for (dst, sl) in modern
        .main_tm_scanlines
        .iter_mut()
        .zip(frame.scanlines.iter())
    {
        *dst = sl.screen_enabled_main;
    }
    // Per-scanline BG scroll (HDMA), mirroring the classic GPU's `scanline_scroll`
    // uniform. Drives the per-output-row re-sample in the Mode-1 compositor for BG
    // layers whose scroll varies across scanlines (e.g. the pyramid HDMA wave).
    modern.bg_scroll_scanlines = frame
        .scanlines
        .iter()
        .map(|sl| {
            let mut per_layer = [[0u16; 2]; 4];
            for (l, slot) in per_layer.iter_mut().enumerate() {
                *slot = [sl.bg_h_scroll[l], sl.bg_v_scroll[l]];
            }
            per_layer
        })
        .collect();
    // Scroll-torus period per BG layer (256 for a 32-tile map, 512 for wide/tall),
    // used by the per-scanline-scroll wrap-sampler.
    for (l, layer) in modern.bg_layers.iter_mut().enumerate() {
        layer.wrap_w = if frame.bg[l].tilemap_wider { 512 } else { 256 };
        layer.wrap_h = if frame.bg[l].tilemap_higher { 512 } else { 256 };
    }
    modern
}

/// Unified live-VRAM modern frame render entry point.
///
/// One committed orchestration for the modern (software) path: decode the BG from
/// live VRAM (mode-agnostic — `extract_modern_frame_from_vram` aliases the dungeon
/// decoder, which reads only `frame.bg[]`/`frame.vram`/`frame.cgram`/
/// `frame.screen_enabled`), decode sprites from live VRAM, then composite BG +
/// sprites + SNES color-math + master brightness in a single
/// [`crate::modern_software::render_modern_frame_full`] call (mirrors the classic
/// post-process pipeline).
///
/// Returns a `256 * 224 * 4` RGBA buffer in R,G,B,A byte order (the same layout the
/// offscreen compare hashes against the classic renderer).
pub fn render_modern_frame_full_from_vram(frame: &GpuFrame<'_>) -> Vec<u8> {
    // Mode 7 (affine BG, e.g. the map screen) isn't a Mode-1 tilemap; the dedicated
    // CPU Mode-7 compositor handles it. This is the single live entry point
    // (`Renderer::render_modern_frame`) as well as the compare's `via=vram` path, so
    // branching here fixes both.
    if frame.mode == 7 {
        return crate::modern_software::render_modern_mode7_frame(frame);
    }
    let (mut modern, bg_cells) = extract_modern_frame_from_vram(frame);
    let (sprite_cells, sprites) = extract_modern_sprites_from_vram(frame);
    modern.index_sprites = sprites;
    crate::modern_software::render_modern_frame_full(&modern, &bg_cells, &sprite_cells)
}

// ── Off-VRAM (logical-CHR-source) render path (Milestone 3) ──────────────────

use crate::modern_source_atlas::{source_cell, ModernSourceAtlas};

/// The one BG kind resolved via the assets atlas: `CHR_KIND_BG_STREAM` is content-
/// hashed (streamed dungeon BG + sprite CHR), keyed by the hash of its frame-end
/// pixels so the atlas is self-consistent. The other kinds are NOT atlas-resolved:
/// generic `CHR_KIND_BG` (= 1, non-injective 3bpp->4bpp conversion) and
/// `CHR_KIND_BG_ANIM` (= 5, in-place overworld animation) decode from LIVE VRAM (see
/// `extract_modern_frame_from_sources`); `CHR_KIND_LINK` decodes from live VRAM in the
/// sprite path.
const CHR_KIND_BG_STREAM: u8 = 6;

/// FNV-1a 32-bit hash of one 16-word (4bpp) CHR tile at `slot`, over its little-endian
/// bytes — byte-identical to `zelda3::chr_content_hash32`. Content-hashed source tags
/// (`CHR_KIND_BG_STREAM`) are re-derived from this at render time so they reflect the
/// FRAME-END pixels: sprite CHR is uploaded incrementally, so the tag the game wrote at
/// NMI/rehash time can desync from the pixels present when the frame is drawn. The
/// assets dump keys cells the same way, so the tag and the atlas cell stay consistent
/// (atlas[hash(W)] == decode(W) for the same 16 words W).
fn content_hash32_slot(vram: &[u16], slot: usize) -> u32 {
    let base = slot * 16;
    let mut h: u32 = 0x811c_9dc5;
    for off in 0..16 {
        let w = *vram.get(base + off).unwrap_or(&0);
        for b in [(w & 0xff) as u8, (w >> 8) as u8] {
            h ^= b as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    h
}

/// Link CHR (mirrors `zelda3::CHR_KIND_LINK`). Decoded from live VRAM in the sprite
/// path, not the atlas: Link's pose CHR is DMA'd per-frame from a source buffer whose
/// offsets are reused across poses, so the source-identity atlas key is non-injective
/// in practice (the dump captures a different pose than the live frame streams).
const CHR_KIND_LINK: u8 = 3;

/// A thin view over the M1 per-VRAM-slot logical CHR source table, returning
/// `(kind, pack, tile_off)` for a CHR tile slot (`word_addr / 16`). Defined in
/// the renderer crate so the off-VRAM path does not depend on the zelda3 crate;
/// the harness adapts `game.vram_chr_source()` into it (a copied slice or a
/// closure). Out-of-range slots return `(0, 0, 0)` = kind-none.
pub trait SourceTableView {
    fn get(&self, slot: usize) -> (u8, u16, u16);
}

impl<F: Fn(usize) -> (u8, u16, u16)> SourceTableView for F {
    fn get(&self, slot: usize) -> (u8, u16, u16) {
        self(slot)
    }
}

impl SourceTableView for [(u8, u16, u16)] {
    fn get(&self, slot: usize) -> (u8, u16, u16) {
        <[_]>::get(self, slot).copied().unwrap_or((0, 0, 0))
    }
}

/// Apply hflip/vflip to a 64-entry (8x8 row-major) index pattern.
fn flip_index_pattern(indices: &[u8; 64], hflip: bool, vflip: bool) -> [u8; 64] {
    if !hflip && !vflip {
        return *indices;
    }
    let mut out = [0u8; 64];
    for y in 0..8usize {
        let sy = if vflip { 7 - y } else { y };
        for x in 0..8usize {
            let sx = if hflip { 7 - x } else { x };
            out[y * 8 + x] = indices[sy * 8 + sx];
        }
    }
    out
}

/// Extract a BG `ModernFrame` whose tile patterns come ONLY from the asset atlas,
/// selected via the M1 logical CHR source table — never reading `frame.vram` CHR
/// pixel content. This is the off-VRAM animation-modeled render path.
///
/// For each enabled BG layer (main OR sub, mirroring the dungeon subscreen floor)
/// and each nonzero tilemap entry: the CHR tile slot is
/// `slot = tile_adr/16 + (entry & 0x3ff)`; its logical source `{kind, pack,
/// tile_off}` is read from `src_table`; the cell is resolved via `source_cell`.
/// Missing source (kind 0 / not in atlas) → the tile is skipped (a gap). The
/// atlas cells are UNFLIPPED (decoded with no flip in M2), and the index-tile
/// compositor ignores per-instance flip, so the tilemap word's flip bits are
/// BAKED into a per-(cell, flip) cell here; the emitted instance flip is false.
/// Geometry (64x64 four-quadrant tilemap, +1 vertical fetch offset, scroll-wrap
/// torus) matches [`extract_modern_dungeon_frame_from_vram`].
pub fn extract_modern_frame_from_sources<S: SourceTableView + ?Sized>(
    frame: &GpuFrame<'_>,
    src_table: &S,
    atlas: &ModernSourceAtlas,
) -> (ModernFrame, Vec<ModernIndexTile>) {
    use std::collections::HashMap;
    let mut modern = extract_modern_frame(frame);
    modern.cgram_rgba = crate::modern_palette::cgram_words_to_rgba256(frame.cgram);
    modern.backdrop_color_rgba =
        crate::modern_palette::snes_cgram_to_rgba(*frame.cgram.first().unwrap_or(&0));

    let mut cells: Vec<ModernIndexTile> = Vec::new();
    // Permanent env-gated diagnostic (ZELDA3_SRC_DEBUG): per BG1/BG2 tile, compare
    // the off-VRAM resolved atlas cell (unflipped) against the live-VRAM decode of
    // the same slot. wrong_cell>0 means a STALE/WRONG source tag; gap>0 means the
    // recorded key is absent from the atlas. Off when the var is unset (one getenv).
    let dbg = std::env::var("ZELDA3_SRC_DEBUG").is_ok();
    let mut dbg_total = 0usize;
    let mut dbg_mismatch = 0usize;
    let mut dbg_gap = 0usize;
    let mut dbg_stale = 0usize;
    let mut dbg_samples: Vec<(usize, usize, u8, u16, u16)> = Vec::new();
    // key: (atlas cell id, hflip, vflip) -> local flip-baked cell id
    let mut cell_ids: HashMap<(u32, bool, bool), u32> = HashMap::new();
    // BG3 (the HUD/message layer) is procedurally COMPOSED at runtime: digit and
    // item-icon glyphs are streamed into a small set of 2bpp CHR slots, so the same
    // `(tile_number, palette)` holds different pixels across the route (~46% of BG3
    // keys are non-injective). A static source atlas therefore cannot represent it
    // (the dump must drop the ambiguous keys -> HUD gaps / wrong digits). Decode BG3
    // from LIVE VRAM instead, keyed by `(chr_base, tile+flip+palette)`. Game art
    // (BG1/BG2/sprites/Link) stays off-VRAM via the source atlas; only this dynamic
    // UI layer reads VRAM pixels.
    let mut bg3_cell_ids: HashMap<(usize, u16), u32> = HashMap::new();

    for layer_index in 0..3usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        let enabled_sub = frame.screen_enabled[1] & (1 << layer_index) != 0;
        let enabled = enabled_main || enabled_sub;
        modern.bg_layers[layer_index].enabled_main = enabled;
        modern.bg_layers[layer_index].enabled_sub = enabled_sub;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        let chr_slot_base = frame.bg[layer_index].tile_adr as usize / 16;
        let h_scroll = frame.bg[layer_index].h_scroll;
        let v_scroll = frame.bg[layer_index].v_scroll;
        let wide = frame.bg[layer_index].tilemap_wider;
        let tall = frame.bg[layer_index].tilemap_higher;
        let cols = if wide { 64usize } else { 32 };
        let rows = if tall { 64usize } else { 32 };
        for ty in 0..rows {
            for tx in 0..cols {
                let q = (if wide && tx >= 32 { 1 } else { 0 })
                    + (if tall && ty >= 32 {
                        if wide {
                            2
                        } else {
                            1
                        }
                    } else {
                        0
                    });
                let within = (ty % 32) * 32 + (tx % 32);
                let addr = base + q * 0x400 + within;
                let entry_word = *frame.vram.get(addr).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                let tile_number = (entry_word & 0x03ff) as usize;
                let hflip = entry_word & 0x4000 != 0;
                let vflip = entry_word & 0x8000 != 0;
                // BG3 (2bpp HUD/message) is decoded from LIVE VRAM (procedurally
                // composed; not injective by tilemap key — see `bg3_cell_ids`). BG1/BG2
                // (4bpp) resolve via the per-slot CHR source table and keep the tilemap
                // palette. The BG3 cell bakes the BG3->CGRAM palette + flip, so its
                // instance renders with palette 0 and no flip.
                let is_bg3 = layer_index == 2;
                let (cell_id, palette) = if is_bg3 {
                    let pal = ((entry_word >> 10) & 7) as u8;
                    let chr_base = frame.bg[layer_index].tile_adr as usize;
                    // dedup key: chr_base + tile# + flip + palette (priority bit dropped)
                    let map_key = entry_word & 0xDFFF;
                    let id = *bg3_cell_ids.entry((chr_base, map_key)).or_insert_with(|| {
                        // 2bpp decode with flip baked, then bake the classic BG3->CGRAM
                        // palette (cgram_idx = palette*4 + pal_idx; pal_idx 0 transparent).
                        let raw = decode_snes_2bpp_tile_indices(
                            frame.vram,
                            chr_base,
                            entry_word & 0xc3ff,
                        );
                        let mut baked = [0u8; 64];
                        for (b, &p) in baked.iter_mut().zip(raw.iter()) {
                            *b = if p == 0 { 0 } else { pal * 4 + p };
                        }
                        let id = cells.len() as u32;
                        cells.push(ModernIndexTile {
                            id,
                            indices: baked,
                            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                            hflip: false,
                            vflip: false,
                        });
                        id
                    });
                    (id, 0u8)
                } else {
                    let slot = chr_slot_base + tile_number;
                    let (kind, mut pack, mut tile_off) = src_table.get(slot);
                    if kind == CHR_KIND_BG_STREAM {
                        // Re-derive the content-hash key from the FRAME-END pixels (see
                        // `content_hash32_slot`) so it matches how the assets dump keys.
                        let h = content_hash32_slot(frame.vram, slot);
                        pack = (h >> 16) as u16;
                        tile_off = (h & 0xffff) as u16;
                    }
                    // The generic-BG CHR source key `(CHR_KIND_BG, pack, tile_off)` is
                    // NOT injective: an area's BG graphics pack is 3bpp ROM data expanded
                    // to 4bpp VRAM via either the "high" or "low" conversion, chosen per
                    // (slot, palette theme) in `load_background_graphics`, and the
                    // conversion variant is NOT encoded in the key — so the SAME
                    // `(pack, tile_off)` resolves to different pixels in different
                    // themes/areas. The assets-by-source dump keeps the first occurrence
                    // (it is processed in frame order), so areas that reuse a key with the
                    // other conversion render a stale cell (whole-screen shade shift in the
                    // overworld). The injective BG kinds — `CHR_KIND_BG_ANIM` (keyed by
                    // absolute buffer position) and `CHR_KIND_BG_STREAM` (content-hashed) —
                    // do not have this problem, and sprites are content-hashed too. Until
                    // the generic-BG area-load is content-hashed at the source (the same fix
                    // already applied to OBJ CHR via `tag_stream_content_hash`), decode the
                    // non-injective generic-BG tiles (and any untagged/gap slot) from LIVE
                    // VRAM, exactly like the BG3 exception above and the from-VRAM oracle.
                    // NOTE: CHR_KIND_BG_ANIM (overworld water/flower tiles at VRAM
                    // 0x3c00) is intentionally NOT injective here — it decodes from LIVE
                    // VRAM below. Those animations rewrite the same 0xa680 buffer position
                    // in-place per phase, so the frame-end pixels the assets dump captures
                    // never match the pixels the live frame streams; no static key
                    // (neither (pack,position) nor content-hash) yields a byte-exact atlas
                    // cell (frame 250000 waterfall). Only CHR_KIND_BG_STREAM (content-
                    // hashed dungeon/streamed BG, verified byte-exact) resolves via the atlas.
                    let injective = kind == CHR_KIND_BG_STREAM;
                    if !injective {
                        let pal = ((entry_word >> 10) & 7) as u8;
                        let chr_base = frame.bg[layer_index].tile_adr as usize;
                        let pattern_key = entry_word & 0xC3FF;
                        let id = *cell_ids
                            .entry((0x8000_0000 | u32::from(pattern_key), false, false))
                            .or_insert_with(|| {
                                let indices = decode_snes_4bpp_tile_indices(
                                    frame.vram,
                                    chr_base,
                                    pattern_key,
                                );
                                let id = cells.len() as u32;
                                cells.push(ModernIndexTile {
                                    id,
                                    indices,
                                    source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                                    hflip: false,
                                    vflip: false,
                                });
                                id
                            });
                        (id, pal)
                    } else {
                        if dbg && layer_index < 2 {
                            dbg_total += 1;
                            let live = decode_snes_4bpp_tile_indices(
                                frame.vram,
                                frame.bg[layer_index].tile_adr as usize,
                                tile_number as u16,
                            );
                            match source_cell(atlas, kind, pack, tile_off) {
                                None => {
                                    dbg_gap += 1;
                                    if dbg_samples.len() < 24 {
                                        dbg_samples.push((slot, tile_number, kind, pack, tile_off));
                                    }
                                }
                                Some(s) if s.indices != live => {
                                    dbg_mismatch += 1;
                                    // Classify: STALE TAG (the source-table hash doesn't match
                                    // the LIVE pixels — the slot was re-loaded without re-tagging)
                                    // vs COLLISION (tag matches live, but the atlas stored another
                                    // area's cell under the same hash). Only meaningful for the
                                    // content-hash kind (6). FNV-1a 24-bit over the 16 live words.
                                    if kind == 6 {
                                        let mut h: u32 = 0x811c_9dc5;
                                        for off in 0..16 {
                                            let w = *frame.vram.get(slot * 16 + off).unwrap_or(&0);
                                            for b in [(w & 0xff) as u8, (w >> 8) as u8] {
                                                h ^= b as u32;
                                                h = h.wrapping_mul(0x0100_0193);
                                            }
                                        }
                                        let live_hash = h;
                                        let tag_hash = ((pack as u32) << 16) | (tile_off as u32);
                                        if live_hash != tag_hash {
                                            dbg_stale += 1;
                                        }
                                    }
                                    if dbg_samples.len() < 24 {
                                        dbg_samples.push((slot, tile_number, kind, pack, tile_off));
                                    }
                                }
                                _ => {}
                            }
                        }
                        match source_cell(atlas, kind, pack, tile_off) {
                            Some(src) => {
                                let id =
                                    *cell_ids.entry((src.id, hflip, vflip)).or_insert_with(|| {
                                        let indices =
                                            flip_index_pattern(&src.indices, hflip, vflip);
                                        let id = cells.len() as u32;
                                        cells.push(ModernIndexTile {
                                            id,
                                            indices,
                                            source_key:
                                                crate::modern_source_atlas::modern_source_key(
                                                    kind, pack, tile_off,
                                                ),
                                            hflip,
                                            vflip,
                                        });
                                        id
                                    });
                                (id, ((entry_word >> 10) & 7) as u8)
                            }
                            None => {
                                let pal = ((entry_word >> 10) & 7) as u8;
                                let chr_base = frame.bg[layer_index].tile_adr as usize;
                                let pattern_key = entry_word & 0xC3FF;
                                let id = *cell_ids
                                    .entry((0x9000_0000 | u32::from(pattern_key), false, false))
                                    .or_insert_with(|| {
                                        let indices = decode_snes_4bpp_tile_indices(
                                            frame.vram,
                                            chr_base,
                                            pattern_key,
                                        );
                                        let id = cells.len() as u32;
                                        cells.push(ModernIndexTile {
                                            id,
                                            indices,
                                            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                                            hflip: false,
                                            vflip: false,
                                        });
                                        id
                                    });
                                (id, pal)
                            }
                        }
                    }
                };

                let bg_w = (cols * 8) as i32;
                let bg_h = (rows * 8) as i32;
                let mut sx = ((tx * 8) as i32 - h_scroll as i32).rem_euclid(bg_w);
                if sx >= 256 {
                    sx -= bg_w;
                }
                let mut sy = ((ty * 8) as i32 - v_scroll as i32 - 1).rem_euclid(bg_h);
                if sy >= 224 {
                    sy -= bg_h;
                }
                modern.bg_layers[layer_index]
                    .index_tiles
                    .push(ModernIndexTileInstance {
                        cell_id,
                        screen_x: sx as i16,
                        screen_y: sy as i16,
                        palette,
                        // Flip is baked into the cell above; the compositor ignores
                        // per-instance BG flip.
                        hflip: false,
                        vflip: false,
                        priority: entry_word & 0x2000 != 0,
                    });
            }
        }
    }
    if dbg {
        eprintln!("[SRC_DEBUG] bg_tiles={dbg_total} wrong_cell={dbg_mismatch} gap={dbg_gap} stale_of_kind6={dbg_stale}");
        for (slot, tile, kind, pack, off) in &dbg_samples {
            eprintln!(
                "[SRC_DEBUG]   slot=0x{slot:03x} tile=0x{tile:03x} src=(kind={kind},pack=0x{pack:04x},off={off})"
            );
        }
    }
    (modern, cells)
}

/// Decode OAM into palette-index sprite-tile instances whose 8x8 patterns come
/// ONLY from the asset atlas via the M1 logical CHR source table — never reading
/// `frame.vram` CHR pixel content. Mirrors the per-sprite/per-8x8-tile
/// enumeration of [`extract_modern_sprites_from_vram`] (same culling, sizes,
/// flip-at-tile-granularity, bank selection, tile addressing).
///
/// The CHR tile slot is `slot = bank_base/16 + used_tile` (bank_base from the OAM
/// bank bit). Missing source (kind 0 / not in atlas) → the tile is skipped. The
/// sprite compositor applies per-instance hflip/vflip while sampling, so cells are
/// emitted UNFLIPPED and the instance carries the flip (matching the from-VRAM
/// variant). Returns `(cells, instances)` with cells re-indexed densely from 0.
pub fn extract_modern_sprites_from_sources<S: SourceTableView + ?Sized>(
    frame: &GpuFrame<'_>,
    src_table: &S,
    atlas: &ModernSourceAtlas,
) -> (Vec<ModernIndexTile>, Vec<ModernIndexSpriteInstance>) {
    use std::collections::HashMap;

    let oam = frame.oam;
    let obj = &frame.obj;
    let drawn = compute_obj_drawn_tiles(frame);
    let mut cells: Vec<ModernIndexTile> = Vec::new();
    // atlas cell id -> local dense cell id
    let mut cell_ids: HashMap<u32, u32> = HashMap::new();
    let mut out = Vec::new();

    for sprite_num in 0..128usize {
        let idx = sprite_num * 2;
        let oam0 = oam.get(idx).copied().unwrap_or(0);

        let y_byte = ((oam0 >> 8) & 0xff) as i32;
        if y_byte == 0xf0 {
            continue;
        }
        let top_y = ((y_byte + 1) & 0xff) - 1;

        let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
        let hi_bits = (hi_word >> (idx % 16)) as i32;
        let size = SPRITE_SIZES[(obj.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize] as i32;

        let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
        if object_x > 256 && object_x + size - 1 < 512 {
            continue;
        }
        let mut x = object_x;
        if x >= 256 {
            x -= 512;
        }
        if x <= -size {
            continue;
        }

        let oam1 = oam.get(idx + 1).copied().unwrap_or(0);
        let hflip = oam1 & 0x4000 != 0;
        let vflip = oam1 & 0x8000 != 0;
        let palette = ((oam1 & 0x0e00) >> 9) as u8;
        let priority = ((oam1 & 0x3000) >> 12) as u8;
        let bank_base = if oam1 & 0x0100 != 0 {
            obj.tile_adr2
        } else {
            obj.tile_adr1
        };
        let chr_slot_base = bank_base as usize / 16;
        let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
        let tile_col_base = (oam1 & 0x0f) as i32;

        let tiles_per_side = size / 8;
        for sty in 0..tiles_per_side {
            for stx in 0..tiles_per_side {
                let src_col_tile = if hflip { tiles_per_side - 1 - stx } else { stx };
                let src_row_tile = if vflip { tiles_per_side - 1 - sty } else { sty };
                let used_tile = (((tile_row_base + src_row_tile) << 4)
                    | ((tile_col_base + src_col_tile) & 0x0f))
                    as usize;

                let screen_x = (x + stx * 8) as i16;
                let screen_y = obj_tile_screen_y(top_y, sty);
                let row_mask = obj_row_mask(&drawn, sprite_num, screen_x, screen_y);
                if row_mask == 0 {
                    continue; // fully dropped by the per-scanline OBJ budget
                }

                let slot = chr_slot_base + used_tile;
                let (kind, mut pack, mut tile_off) = src_table.get(slot);
                if kind == CHR_KIND_BG_STREAM {
                    // Sprite CHR is content-hashed (kind BG_STREAM). Re-derive the key
                    // from the FRAME-END pixels so it matches the assets dump (the tag the
                    // game wrote at rehash time can desync from the drawn pixels because
                    // sprite CHR uploads incrementally). See `content_hash32_slot`.
                    let h = content_hash32_slot(frame.vram, slot);
                    pack = (h >> 16) as u16;
                    tile_off = (h & 0xffff) as u16;
                }
                // Link (kind=3) decodes from LIVE VRAM (unflipped; per-instance flip is
                // applied by the compositor). Its pose CHR is DMA'd per-frame from a
                // source buffer whose offsets are reused across poses, so the source-
                // identity atlas key resolves a stale pose (frame 253000 Link: 98px).
                // Cache the per-slot decode under a high-bit-tagged key disjoint from
                // atlas cell ids. All other sprites (kind=2, content-hashed) use the atlas.
                let cell_id = if kind == CHR_KIND_LINK {
                    *cell_ids
                        .entry(0x8000_0000 | slot as u32)
                        .or_insert_with(|| {
                            let indices = decode_snes_4bpp_tile_indices(frame.vram, slot * 16, 0);
                            let id = cells.len() as u32;
                            cells.push(ModernIndexTile {
                                id,
                                indices,
                                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                                hflip: false,
                                vflip: false,
                            });
                            id
                        })
                } else {
                    let Some(src) = source_cell(atlas, kind, pack, tile_off) else {
                        if std::env::var("ZELDA3_SPR_DEBUG").is_ok() {
                            eprintln!(
                                "[SPR_GAP] slot=0x{slot:03x} kind={kind} pack=0x{pack:04x} off=0x{tile_off:04x} x={screen_x} y={screen_y}"
                            );
                        }
                        continue;
                    };
                    *cell_ids.entry(src.id).or_insert_with(|| {
                        let id = cells.len() as u32;
                        cells.push(ModernIndexTile {
                            id,
                            indices: src.indices,
                            source_key: crate::modern_source_atlas::modern_source_key(
                                kind, pack, tile_off,
                            ),
                            hflip: false,
                            vflip: false,
                        });
                        id
                    })
                };

                out.push(ModernIndexSpriteInstance {
                    cell_id,
                    screen_x,
                    screen_y,
                    palette,
                    priority,
                    hflip,
                    vflip,
                    row_mask,
                });
            }
        }
    }

    (cells, out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_frame::{GpuFrame, ScanlineRegs};
    use crate::modern_assets::{ModernTileAtlasAsset, ModernTileAtlasEntry};
    use crate::modern_index_atlas::ModernIndexTile;
    use crate::modern_palette::snes_cgram_to_rgba;

    #[test]
    fn render_full_from_vram_returns_256x224x4_backdrop() {
        // The unified entry point must return a 256×224×4 RGBA buffer. With a
        // nonzero CGRAM[0] backdrop and no enabled layers, every pixel should be
        // the backdrop color (non-empty / not all-zero).
        let vram = vec![0u16; 0x8000];
        let mut cgram = vec![0u16; 0x100];
        cgram[0] = 0x7fff; // white backdrop (SNES BGR555)
        let oam = vec![0u16; 0x110];
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 1;
        frame.screen_enabled = [0, 0];

        let rgba = render_modern_frame_full_from_vram(&frame);
        assert_eq!(rgba.len(), 256 * 224 * 4);
        // Backdrop is non-black; buffer is not all-zero.
        assert!(rgba.iter().any(|&b| b != 0), "frame buffer is empty");
        // Alpha channel is opaque.
        assert!(rgba.chunks_exact(4).all(|px| px[3] == 0xff));
    }

    #[test]
    fn extract_from_sources_emits_atlas_cell_for_injective_bg_tile() {
        use crate::modern_source_atlas::ModernSourceAtlas;
        // Only CHR_KIND_BG_STREAM (=6, content-hashed) resolves via the atlas; generic
        // CHR_KIND_BG (=1) and CHR_KIND_BG_ANIM (=5) decode from live VRAM. The
        // BG_STREAM key is RE-DERIVED from the tile's frame-end VRAM pixels
        // (content_hash32_slot), NOT the source-table tag's pack/tile_off — so the
        // source table below can report an arbitrary (pack, tile_off); the render
        // ignores it and keys by the hash. The atlas cell must be keyed by that hash.
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];

        // Source-table tag: BG CHR base tile_adr=0x2000 → slot base 0x200. Tile# 4 →
        // slot 0x204. The tag's pack/tile_off (5, 3) are intentionally ignored by the
        // re-key; only kind=CHR_KIND_BG_STREAM matters.
        let table = |slot: usize| -> (u8, u16, u16) {
            if slot == 0x200 + 4 {
                (CHR_KIND_BG_STREAM, 5, 3)
            } else {
                (0, 0, 0)
            }
        };

        // Give slot 0x204's CHR tile (VRAM words 0x2040..0x2050) distinctive pixels so
        // its content hash is non-trivial; key the atlas cell by that hash.
        let mut vram = vec![0u16; 0x8000];
        for (i, w) in vram[0x2040..0x2050].iter_mut().enumerate() {
            *w = 0x1000u16.wrapping_add(i as u16);
        }
        vram[0] = 4; // tile# 4, palette 0, no flip, at row0 col0
        let h = content_hash32_slot(&vram, 0x204);
        let mut indices = [0u8; 64];
        indices[0] = 7;
        indices[63] = 9;
        let cell = ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        };
        let atlas = ModernSourceAtlas::from_keyed_cells_for_test(
            vec![cell],
            &[(CHR_KIND_BG_STREAM, (h >> 16) as u16, (h & 0xffff) as u16, 0)],
        );

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 1;
        frame.bg[0].tilemap_adr = 0;
        frame.bg[0].tile_adr = 0x2000;
        frame.screen_enabled = [0x01, 0x00];

        let (modern, cells) = extract_modern_frame_from_sources(&frame, &table, &atlas);
        assert_eq!(cells.len(), 1, "one cell emitted from the atlas source");
        assert_eq!(cells[0].indices[0], 7);
        assert_eq!(cells[0].indices[63], 9);
        let tiles = &modern.bg_layers[0].index_tiles;
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].cell_id, 0);
        assert_eq!(tiles[0].screen_x, 0);
        // +1 vertical fetch offset, no scroll: (0-0-1).rem_euclid(256)=255 → -=256 → -1.
        assert_eq!(tiles[0].screen_y, -1);
        assert_eq!(tiles[0].palette, 0);
        // The CELL pixels came from the atlas (7,9), not the VRAM tile pixels; only the
        // KEY was derived from VRAM (the frame-end content hash).

        // A BG_STREAM tile whose content hash is NOT in the atlas must remain visible
        // by falling back to a live-VRAM indexed cell. This keeps existing assets
        // correct while regenerated source atlases catch up to new hash keys.
        let table_miss = |slot: usize| -> (u8, u16, u16) {
            if slot == 0x200 + 9 {
                (CHR_KIND_BG_STREAM, 5, 99)
            } else {
                (0, 0, 0)
            }
        };
        let mut vram2 = vec![0u16; 0x8000];
        for (i, w) in vram2[0x2090..0x20a0].iter_mut().enumerate() {
            *w = 0x7abcu16.wrapping_sub(i as u16); // distinct content → different hash
        }
        vram2[0] = 9; // tile# 9 → slot 0x209
        let mut frame2 = test_gpu_frame(&vram2, &cgram, &oam, 15, false);
        frame2.mode = 1;
        frame2.bg[0].tile_adr = 0x2000;
        frame2.screen_enabled = [0x01, 0x00];
        let (modern2, cells2) = extract_modern_frame_from_sources(&frame2, &table_miss, &atlas);
        assert_eq!(cells2.len(), 1, "missing hash falls back to live VRAM");
        assert_eq!(
            cells2[0].source_key,
            crate::modern_hd_overrides::NO_SOURCE_KEY,
            "live fallback cells remain unkeyed until the source atlas is regenerated"
        );
        assert_eq!(modern2.bg_layers[0].index_tiles.len(), 1);
    }

    #[test]
    fn extract_from_sources_decodes_generic_bg_from_live_vram() {
        use crate::modern_source_atlas::ModernSourceAtlas;
        // Generic CHR_KIND_BG (=1) is NOT injective by `(pack, tile_off)` (the
        // theme-dependent high/low 3bpp→4bpp conversion is not encoded in the key),
        // so the off-VRAM path must decode it from LIVE VRAM to stay byte-exact with
        // the from-VRAM oracle — even when the source table reports a kind=1 tag and
        // the atlas holds a (stale) cell for that key. Here the atlas's kind=1 cell
        // (index 7) must be IGNORED in favour of the live VRAM pattern.
        let mut stale = [0u8; 64];
        stale[0] = 7;
        let cell = ModernIndexTile {
            id: 0,
            indices: stale,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        };
        let atlas = ModernSourceAtlas::from_keyed_cells_for_test(vec![cell], &[(1, 5, 3, 0)]);
        let table = |slot: usize| -> (u8, u16, u16) {
            if slot == 0x200 + 4 {
                (1, 5, 3)
            } else {
                (0, 0, 0)
            }
        };

        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = 4; // BG1 tilemap entry: tile# 4
                     // Live 4bpp CHR for tile#4 at chr_base 0x2000: tile_base = 0x2000 + 4*16 =
                     // 0x2040. Row 0 plane01 word: set pixel x=7 (bit 0x01) in plane0 → index 1.
        vram[0x2040] = 0x0001;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 1;
        frame.bg[0].tilemap_adr = 0;
        frame.bg[0].tile_adr = 0x2000;
        frame.screen_enabled = [0x01, 0x00];

        let (modern, cells) = extract_modern_frame_from_sources(&frame, &table, &atlas);
        assert_eq!(cells.len(), 1, "one cell decoded from live VRAM");
        // Live decode: pixel (row 0, x=7) = 1; the stale atlas index 7 is NOT used.
        assert_eq!(cells[0].indices[7], 1, "generic BG decoded from live VRAM");
        assert_eq!(cells[0].indices[0], 0);
        let tiles = &modern.bg_layers[0].index_tiles;
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].cell_id, 0);
    }

    #[test]
    fn extract_from_sources_decodes_bg3_from_live_vram_palette_zero() {
        use crate::modern_source_atlas::ModernSourceAtlas;
        // BG3 (layer 2, 2bpp HUD) is procedurally composed, so it is decoded from
        // LIVE VRAM (not the source atlas), with the BG3->CGRAM palette baked into
        // the cell and rendered at instance palette 0. The atlas is irrelevant here.
        let atlas = ModernSourceAtlas::from_keyed_cells_for_test(vec![], &[]);
        let table = |_slot: usize| -> (u8, u16, u16) { (0, 0, 0) };

        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        // BG3 tilemap at adr 0; entry = tile#7, palette 3 (bits 12:10 = 3 << 10).
        vram[0] = 7 | (3 << 10);
        // 2bpp CHR for tile#7 at chr_base 0x1000: tile_base = 0x1000 + 7*8 = 0x1038.
        // Row 0 word = bp0 | (bp1 << 8). Set pixel x=0 (bit 0x80) to pal_idx 2
        // (bp1 bit set, bp0 clear): word = 0x80 << 8 = 0x8000.
        vram[0x1038] = 0x8000;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 1;
        frame.bg[2].tilemap_adr = 0;
        frame.bg[2].tile_adr = 0x1000;
        frame.screen_enabled = [0x04, 0x00]; // BG3 main

        let (modern, cells) = extract_modern_frame_from_sources(&frame, &table, &atlas);
        assert_eq!(cells.len(), 1, "BG3 cell decoded from live VRAM");
        // Baked CGRAM index = palette*4 + pal_idx = 3*4 + 2 = 14.
        assert_eq!(cells[0].indices[0], 14);
        let tiles = &modern.bg_layers[2].index_tiles;
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].cell_id, 0);
        // BG3 bakes CGRAM into the cell, so the instance palette is 0.
        assert_eq!(tiles[0].palette, 0);
    }

    #[test]
    fn perf_render_modern_frame_full_from_vram() {
        // Microbenchmark: time the unified live-VRAM render on a synthetic,
        // fully-populated full-screen frame (BG1+BG2 4bpp, BG3 2bpp HUD, 128 OAM
        // sprites). Reports ms/frame and whether it clears the 16.6ms (60fps) bar.
        let mut vram = vec![0u16; 0x8000];
        let mut cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        // Fill CGRAM with a varied palette (avoid all-transparent).
        for (i, c) in cgram.iter_mut().enumerate() {
            *c = (i as u16).wrapping_mul(0x0421) | 0x0001;
        }
        // Fill CHR for a pool of 256 tiles with non-trivial bitplanes.
        for (i, w) in vram.iter_mut().enumerate() {
            *w = (i as u16).wrapping_mul(0x9E37) ^ 0x55AA;
        }
        // BG1 (4bpp) + BG2 (4bpp) + BG3 (2bpp) full 32×32 tilemaps, distinct CHR bases.
        for layer in 0..3usize {
            let base = layer * 0x400;
            for cell in 0..0x400usize {
                // nonzero tile# spread across the CHR pool, with a palette spread.
                vram[base + cell] = ((cell as u16 & 0xFF) | ((cell as u16 & 0x7) << 10)) | 0x0001;
            }
        }
        // 128 on-screen sprites (8×8) across the frame.
        for s in 0..128usize {
            let b = s * 2;
            let x = ((s * 2) % 248) as u16;
            let y = ((s * 13) % 216) as u16;
            oam[b] = (x & 0xFF) | (y << 8);
            oam[b + 1] = (s as u16 & 0xFF) | 0x0200; // tile# + palette/attr
        }

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 1;
        for layer in 0..3usize {
            frame.bg[layer].tilemap_adr = (layer * 0x400) as u16;
            frame.bg[layer].tile_adr = 0x1000;
        }
        frame.screen_enabled = [0x17, 0x00]; // BG1|BG2|BG3|OBJ on main
        frame.obj.tile_adr1 = 0x4000;
        frame.obj.tile_adr2 = 0x5000;

        // Warm up, then time.
        let out = render_modern_frame_full_from_vram(&frame);
        assert_eq!(out.len(), 256 * 224 * 4);
        let iters = 50u32;
        let t0 = std::time::Instant::now();
        for _ in 0..iters {
            let buf = render_modern_frame_full_from_vram(&frame);
            std::hint::black_box(&buf);
        }
        let elapsed = t0.elapsed();
        let ms = elapsed.as_secs_f64() * 1000.0 / f64::from(iters);
        println!(
            "perf render_modern_frame_full_from_vram: {ms:.3} ms/frame ({}) [{} iters]",
            if ms < 16.6 {
                "<16.6ms OK for 60fps"
            } else {
                "OVER 16.6ms budget"
            },
            iters
        );
    }

    /// Same synthetic fully-populated frame as `perf_render_modern_frame_full_from_vram`
    /// (BG1+BG2 4bpp, BG3 2bpp HUD, 128 OAM sprites), but pre-extracted to the
    /// `ModernFrame` + cells the modern software compositor entries take directly, so
    /// scaled-render perf tests don't pay VRAM-decode cost per iteration.
    fn perf_fixture() -> (ModernFrame, Vec<ModernIndexTile>, Vec<ModernIndexTile>) {
        let mut vram = vec![0u16; 0x8000];
        let mut cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        for (i, c) in cgram.iter_mut().enumerate() {
            *c = (i as u16).wrapping_mul(0x0421) | 0x0001;
        }
        for (i, w) in vram.iter_mut().enumerate() {
            *w = (i as u16).wrapping_mul(0x9E37) ^ 0x55AA;
        }
        for layer in 0..3usize {
            let base = layer * 0x400;
            for cell in 0..0x400usize {
                vram[base + cell] = ((cell as u16 & 0xFF) | ((cell as u16 & 0x7) << 10)) | 0x0001;
            }
        }
        for s in 0..128usize {
            let b = s * 2;
            let x = ((s * 2) % 248) as u16;
            let y = ((s * 13) % 216) as u16;
            oam[b] = (x & 0xFF) | (y << 8);
            oam[b + 1] = (s as u16 & 0xFF) | 0x0200;
        }

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 1;
        for layer in 0..3usize {
            frame.bg[layer].tilemap_adr = (layer * 0x400) as u16;
            frame.bg[layer].tile_adr = 0x1000;
        }
        frame.screen_enabled = [0x17, 0x00];
        frame.obj.tile_adr1 = 0x4000;
        frame.obj.tile_adr2 = 0x5000;

        let (mut modern, bg_cells) = extract_modern_frame_from_vram(&frame);
        let (sprite_cells, sprites) = extract_modern_sprites_from_vram(&frame);
        modern.index_sprites = sprites;
        (modern, bg_cells, sprite_cells)
    }

    #[test]
    fn perf_render_modern_frame_scaled() {
        // Reuse the same frame/cells construction as perf_render_modern_frame_full_from_vram.
        let (frame, bg_cells, sprite_cells) = perf_fixture();
        for scale in [2u32, 4] {
            let iters = 20;
            let start = std::time::Instant::now();
            let mut sink = 0usize;
            for _ in 0..iters {
                let out = crate::modern_software::render_modern_frame_full_scaled(
                    &frame,
                    &bg_cells,
                    &sprite_cells,
                    &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
                    scale,
                );
                sink = sink.wrapping_add(out.len());
            }
            let ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
            eprintln!("perf render_modern_frame_full_scaled x{scale}: {ms:.3} ms/frame ({sink})");
            assert!(ms < 16.6, "scale {scale} too slow: {ms:.3} ms");
        }
    }

    #[test]
    fn dungeon_from_vram_wraps_scroll_modulo_tilemap_size() {
        // Regression for the BLACK dungeon room: a 64×64 (512px) tilemap with a
        // large scroll (h_scroll=640) must wrap modulo the BG pixel size so tiles
        // reappear on-screen instead of being pushed off (the SNES BG is a torus).
        // Tile at column tx=16, row ty=12 with h_scroll=640 wraps to screen_x=0:
        //   (16*8 - 640).rem_euclid(512) = (-512).rem_euclid(512) = 0.
        // Vertical: (12*8 - 0 - 1).rem_euclid(512) = 95.
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        // within = (ty%32)*32 + (tx%32) = 12*32 + 16 = 400; nonzero entry → emitted.
        vram[400] = 1;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 1;
        frame.bg[0].tilemap_adr = 0;
        frame.bg[0].tile_adr = 0;
        frame.bg[0].tilemap_wider = true;
        frame.bg[0].tilemap_higher = true;
        frame.bg[0].h_scroll = 640;
        frame.bg[0].v_scroll = 0;
        frame.screen_enabled = [0x01, 0x00];

        let (modern, _cells) = extract_modern_dungeon_frame_from_vram(&frame);
        let tiles = &modern.bg_layers[0].index_tiles;
        assert_eq!(tiles.len(), 1, "exactly one nonzero tilemap entry emitted");
        assert_eq!(
            tiles[0].screen_x, 0,
            "h_scroll=640 on a 512px BG must wrap to screen_x=0, not -512"
        );
        assert_eq!(tiles[0].screen_y, 95, "vertical +1 offset, no wrap needed");
    }

    #[test]
    fn decode_snes_tilemap_entry_splits_visual_fields() {
        let fields = decode_snes_tilemap_entry(0xed23);

        assert_eq!(fields.tile_number, 0x0123);
        assert_eq!(fields.palette, 3);
        assert!(fields.priority);
        assert!(fields.hflip);
        assert!(fields.vflip);
    }

    #[test]
    fn extract_modern_frame_copies_frame_level_visual_state() {
        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let frame = test_gpu_frame(&vram, &cgram, &oam, 9, true);

        let modern = extract_modern_frame(&frame);

        assert_eq!(modern.width, 256);
        assert_eq!(modern.height, 224);
        assert_eq!(modern.brightness, 9);
        assert!(modern.forced_blank);
    }

    #[test]
    fn extract_modern_frame_maps_bg_tilemap_entry_to_atlas_tile() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = crate::modern_assets::load_modern_overworld_tile_atlas(&root)
            .expect("atlas should load");
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = 2218;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.bg[0].tile_adr = 0x2000;
        frame.screen_enabled = [0x01, 0x00];

        let modern = extract_modern_frame_with_atlas(&frame, &atlas);

        assert_eq!(modern.bg_layers[0].tiles.len(), 1);
        assert_eq!(modern.bg_layers[0].tiles[0].atlas_x_px, 1);
        // Real atlas entry is a 32px source cell at atlas_scale 4, so the on-screen
        // footprint downsamples to the true 8x8 tile size.
        assert_eq!(modern.bg_layers[0].tiles[0].atlas_width_px, 32);
        assert_eq!(modern.bg_layers[0].tiles[0].screen_width_px, 8);
        assert_eq!(modern.bg_layers[0].tiles[0].screen_height_px, 8);
        assert_eq!(modern.bg_layers[0].tiles[0].screen_x, 0);
        assert_eq!(modern.bg_layers[0].tiles[0].screen_y, 0);
    }

    /// Builds a minimal synthetic atlas with a single entry keyed by the given tilemap word.
    fn synthetic_atlas(tilemap_entry: u16) -> ModernTileAtlasAsset {
        ModernTileAtlasAsset {
            tile_width_px: 8,
            tile_height_px: 8,
            atlas_scale: 1,
            width_px: 8,
            height_px: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![ModernTileAtlasEntry {
                id: 0,
                atlas_x_px: 0,
                atlas_y_px: 0,
                atlas_width_px: 8,
                atlas_height_px: 8,
                tilemap_entry,
                tilemap_variants: vec![tilemap_entry],
            }],
        }
    }

    /// The atlas bakes flip into its cell pixels; re-applying the word's flip bits on the
    /// emitted ModernTileInstance would double-flip asymmetric tiles.  Expect hflip==false
    /// and vflip==false regardless of what the tilemap word's flip bits say.
    #[test]
    fn atlas_sourced_tile_does_not_re_apply_hflip() {
        let hflip_word: u16 = 0x4001; // bit 14 set = hflip
        let atlas = synthetic_atlas(hflip_word);
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = hflip_word;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00];

        let modern = extract_modern_frame_with_atlas(&frame, &atlas);

        assert_eq!(modern.bg_layers[0].tiles.len(), 1);
        assert!(
            !modern.bg_layers[0].tiles[0].hflip,
            "hflip must be false: atlas bakes flip, re-applying would double-flip"
        );
        assert!(
            !modern.bg_layers[0].tiles[0].vflip,
            "vflip must be false: atlas bakes flip"
        );
    }

    #[test]
    fn atlas_sourced_tile_does_not_re_apply_vflip() {
        let vflip_word: u16 = 0x8001; // bit 15 set = vflip
        let atlas = synthetic_atlas(vflip_word);
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = vflip_word;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00];

        let modern = extract_modern_frame_with_atlas(&frame, &atlas);

        assert_eq!(modern.bg_layers[0].tiles.len(), 1);
        assert!(
            !modern.bg_layers[0].tiles[0].hflip,
            "hflip must be false: atlas bakes flip"
        );
        assert!(
            !modern.bg_layers[0].tiles[0].vflip,
            "vflip must be false: atlas bakes flip, re-applying would double-flip"
        );
    }

    /// WORD = 0x0C01: palette=3 (bits [12:10] = 3 = 0b011), tile=1 (bit 0 set), no flip.
    /// graphics_key = 0x0C01 & 0xC3FF = 0x0001 (palette/priority bits stripped).
    #[test]
    fn extract_indexed_frame_emits_index_tile_and_populates_cgram_rgba() {
        // palette=3, tile=1 → word=0x0C01; graphics_key=0x0001
        const WORD: u16 = (3u16 << 10) | 1u16; // 0x0C01
        const GRAPHICS_KEY: u16 = WORD & 0xC3FF; // 0x0001

        let cell = ModernIndexTile {
            id: 0,
            indices: [0u8; 64],
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        };
        // Build an atlas that maps GRAPHICS_KEY → cell index 0
        let atlas = ModernIndexAtlas::from_keyed_cells_for_test(vec![cell], &[(GRAPHICS_KEY, 0)]);

        let mut vram = vec![0u16; 0x8000];
        // Set cgram[0]=0x001F (R=31 → [248,0,0,0xff]), cgram[1]=0x7C00 (B=31 → [0,0,248,0xff])
        let mut cgram = vec![0u16; 0x100];
        cgram[0] = 0x001F;
        cgram[1] = 0x7C00;
        let oam = vec![0u16; 0x110];

        vram[0] = WORD; // tilemap entry at row=0, col=0
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00]; // only BG0 enabled on main

        let modern = extract_modern_frame_with_index_atlas(&frame, &atlas);

        // One indexed tile on layer 0
        assert_eq!(modern.bg_layers[0].index_tiles.len(), 1);
        let inst = &modern.bg_layers[0].index_tiles[0];
        assert_eq!(inst.cell_id, 0);
        assert_eq!(inst.palette, 3);
        assert_eq!(inst.screen_x, 0);
        assert_eq!(inst.screen_y, 0);
        assert!(!inst.hflip);
        assert!(!inst.vflip);

        // No RGBA atlas tiles emitted
        assert!(modern.bg_layers[0].tiles.is_empty());

        // Layers 1 and 2 are disabled → no tiles
        assert!(modern.bg_layers[1].index_tiles.is_empty());
        assert!(modern.bg_layers[2].index_tiles.is_empty());

        // cgram_rgba populated from frame.cgram
        assert_eq!(modern.cgram_rgba[0], snes_cgram_to_rgba(0x001F)); // [248,0,0,0xff]
        assert_eq!(modern.cgram_rgba[1], snes_cgram_to_rgba(0x7C00)); // [0,0,248,0xff]
                                                                      // Slots beyond the supplied cgram default to black opaque
        assert_eq!(modern.cgram_rgba[255], [0, 0, 0, 0xff]);
    }

    /// WORD = (3<<10)|0x012 → palette=3, tile=0x12, no flip.
    /// graphics_key = WORD & 0xC3FF = 0x0012 (palette bits stripped).
    /// packed key  = ((THEME as u32)<<16) | 0x0012.
    #[test]
    fn extract_dungeon_frame_emits_index_tile_by_theme_and_populates_cgram_rgba() {
        const THEME: u16 = 4;
        const WORD: u16 = (3u16 << 10) | 0x012; // palette=3, tile=0x12 → 0x0C12
        const GKEY: u32 = ((THEME as u32) << 16) | ((WORD & 0xC3FF) as u32);

        let cell = ModernIndexTile {
            id: 42,
            indices: [0u8; 64],
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        };
        let atlas = crate::modern_dungeon_atlas::ModernDungeonIndexAtlas::from_keyed_cells_for_test(
            vec![cell],
            vec![(GKEY, 0)],
        );

        let mut vram = vec![0u16; 0x8000];
        let mut cgram = vec![0u16; 0x100];
        cgram[0] = 0x001F; // R=31 → [248, 0, 0, 255]
        cgram[1] = 0x7C00; // B=31 → [0, 0, 248, 255]
        let oam = vec![0u16; 0x110];

        vram[0] = WORD; // tilemap entry at row=0, col=0
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00]; // only BG0 on main

        let modern =
            crate::modern_extract::extract_modern_frame_with_dungeon_atlas(&frame, &atlas, THEME);

        // One indexed tile on layer 0
        assert_eq!(modern.bg_layers[0].index_tiles.len(), 1);
        let inst = &modern.bg_layers[0].index_tiles[0];
        assert_eq!(inst.cell_id, 42);
        assert_eq!(inst.palette, 3);
        assert_eq!(inst.screen_x, 0);
        assert_eq!(inst.screen_y, 0);
        assert!(!inst.hflip);
        assert!(!inst.vflip);

        // No RGBA atlas tiles emitted on any layer
        assert!(modern.bg_layers[0].tiles.is_empty());
        assert!(modern.bg_layers[1].index_tiles.is_empty());
        assert!(modern.bg_layers[2].index_tiles.is_empty());

        // cgram_rgba populated from frame.cgram
        assert_eq!(modern.cgram_rgba[0], snes_cgram_to_rgba(0x001F));
        assert_eq!(modern.cgram_rgba[1], snes_cgram_to_rgba(0x7C00));
        assert_eq!(modern.cgram_rgba[255], [0, 0, 0, 0xff]);

        // Wrong theme → zero tiles (key won't resolve)
        let modern_wrong_theme = crate::modern_extract::extract_modern_frame_with_dungeon_atlas(
            &frame,
            &atlas,
            THEME + 1,
        );
        assert!(
            modern_wrong_theme.bg_layers[0].index_tiles.is_empty(),
            "wrong theme must yield zero tiles"
        );
    }

    #[test]
    fn decode_2bpp_tile_reads_two_bitplanes() {
        // tile#0 at chr_base 0: row0 word = planes 0 (low) + 1 (high). Set both bit7
        // (pixel x=0) → index 3; leave the rest 0.
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x8080; // bp0 bit7 + bp1 bit7
        let out = decode_snes_2bpp_tile_indices(&vram, 0, 0);
        assert_eq!(out[0], 3, "pixel (0,0) = 2bpp index 3");
        assert_eq!(out[1], 0, "pixel (1,0) untouched");
        assert_eq!(out[8], 0, "pixel (0,1) untouched");
    }

    #[test]
    fn extract_dungeon_from_vram_decodes_live_chr_and_backdrop() {
        // CHR base 0, tile#1 → tile_base 16. Set pixel (0,0) to 4bpp index 10
        // (planes 1 and 3 → w01 high byte + w23 high byte have bit7).
        let mut vram = vec![0u16; 0x8000];
        vram[16] = 0x8000; // w01: bp1 bit7  (plane 1)
        vram[24] = 0x8000; // w23: bp3 bit7  (plane 3)  => index 0b1010 = 10
        vram[0x1000] = (3u16 << 10) | 1; // tilemap entry: palette 3, tile 1, no flip

        let mut cgram = vec![0u16; 0x100];
        cgram[0] = 0x001F; // backdrop R=31 → [248,0,0,255]
        let oam = vec![0u16; 0x110];

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0x1000;
        frame.bg[0].tile_adr = 0;
        frame.screen_enabled = [0x01, 0x00]; // BG1 on main only

        let (modern, cells) = crate::modern_extract::extract_modern_dungeon_frame_from_vram(&frame);

        assert_eq!(modern.bg_layers[0].index_tiles.len(), 1);
        let inst = &modern.bg_layers[0].index_tiles[0];
        assert_eq!(inst.palette, 3);
        assert_eq!(inst.screen_x, 0);
        // SNES vertical +1 fetch offset (output line sy shows BG row sy+1), so the
        // tile at tilemap row 0 with v_scroll=0 lands at screen_y = -1.
        assert_eq!(inst.screen_y, -1);
        let cell = &cells[inst.cell_id as usize];
        assert_eq!(cell.indices[0], 10, "live-VRAM decoded index at (0,0)");
        assert_eq!(cell.indices[1], 0, "neighbour pixel transparent");
        // Backdrop comes from CGRAM[0] (the classic main color-math operand).
        assert_eq!(modern.backdrop_color_rgba, snes_cgram_to_rgba(0x001F));
        // BG3 (layer 2) is not enabled here (screen_enabled bit 2 clear) → no tiles.
        assert!(modern.bg_layers[2].index_tiles.is_empty());
    }

    /// BG3 (layer 2) is the 2bpp HUD layer: it must decode 2bpp (8 words/tile) and
    /// bake the classic CGRAM mapping `palette*4 + pal_idx` into the cell, emitting
    /// instance palette 0 so the compositor resolves `cgram_rgba[palette*4+idx]`.
    #[test]
    fn extract_dungeon_from_vram_decodes_bg3_as_2bpp_with_baked_palette() {
        let mut vram = vec![0u16; 0x8000];
        // BG3 CHR at base 0x2000, tile#2 → tile_base 0x2000 + 2*8 = 0x2010.
        // Row 0: bp0 bit7 + bp1 bit7 → pixel (0,0) = 2bpp index 3.
        vram[0x2010] = 0x8080;
        // BG3 tilemap at 0x3000: palette 5, tile 2, no flip.
        vram[0x3000] = (5u16 << 10) | 2;

        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[2].tilemap_adr = 0x3000;
        frame.bg[2].tile_adr = 0x2000;
        frame.screen_enabled = [0x04, 0x00]; // BG3 (bit 2) on main only

        let (modern, cells) = crate::modern_extract::extract_modern_dungeon_frame_from_vram(&frame);

        assert_eq!(modern.bg_layers[2].index_tiles.len(), 1);
        let inst = &modern.bg_layers[2].index_tiles[0];
        // Palette baked into cell → instance palette is 0.
        assert_eq!(inst.palette, 0);
        assert_eq!(inst.screen_x, 0);
        assert_eq!(inst.screen_y, -1); // +1 vertical fetch offset
        let cell = &cells[inst.cell_id as usize];
        // pixel (0,0): 2bpp index 3, palette 5 → baked cgram idx = 5*4 + 3 = 23.
        assert_eq!(
            cell.indices[0], 23,
            "BG3 baked cgram index = palette*4 + pal_idx"
        );
        assert_eq!(cell.indices[1], 0, "transparent neighbour stays 0");
    }

    /// The SNES OBJ Y space wraps mod 256: a high-Y sprite's lower tiles reappear
    /// at the top of the screen. `obj_tile_screen_y` must reproduce that wrap so
    /// `resolve_obj_pixels`-matching placement is preserved (the statue-tops bug).
    #[test]
    fn obj_tile_screen_y_wraps_high_y_objects_to_top() {
        // Normal on-screen sprite: unchanged.
        assert_eq!(obj_tile_screen_y(10, 0), 10);
        assert_eq!(obj_tile_screen_y(10, 1), 18);
        // Top-straddling sprite (yy==0 -> top_y==-1): unchanged (small negative).
        assert_eq!(obj_tile_screen_y(-1, 0), -1);
        // High-Y wrap (yy==250 -> top_y==249, 16px sprite): tile0 -> -7, tile1 -> 1.
        assert_eq!(obj_tile_screen_y(249, 0), -7);
        assert_eq!(obj_tile_screen_y(249, 1), 1);
        // Boundary: exactly at the bottom edge wraps but stays off-screen (-32).
        assert_eq!(obj_tile_screen_y(224, 0), -32);
        // Last on-screen row is not wrapped.
        assert_eq!(obj_tile_screen_y(223, 0), 223);
    }

    /// Craft an OAM with ONE 8×8 sprite and assert `extract_modern_sprites`
    /// resolves it to a single instance with the right cell/palette/priority/pos.
    /// For an 8×8 sprite, `effective_tile == oam1 & 0xff` (bank 0, single tile).
    #[test]
    fn extract_modern_sprites_decodes_one_8x8_sprite() {
        const CONTEXT: u64 = 21;
        const TILE: u16 = 5;
        let palette: u16 = 2;
        let priority: u16 = 1;
        let x: u16 = 40;
        let y: u16 = 50;

        let cell = ModernIndexTile {
            id: 99,
            indices: [0u8; 64],
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        };
        let atlas = ModernSpriteIndexAtlas::from_keyed_cells_for_test(
            vec![cell],
            vec![((CONTEXT, TILE), 0)],
        );

        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        oam[0] = (y << 8) | x;
        oam[1] = (palette << 9) | (priority << 12) | TILE;
        // hi-word (oam[0x100]) left 0 → size bit 0 (small=8), x-hi 0.

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.obj.obj_size = 0; // SPRITE_SIZES[0] = [8, 16] → small = 8

        let sprites = extract_modern_sprites(&frame, &atlas, CONTEXT);

        assert_eq!(sprites.len(), 1);
        let s = &sprites[0];
        assert_eq!(s.cell_id, 99);
        assert_eq!(s.palette, 2);
        assert_eq!(s.priority, 1);
        assert_eq!(s.screen_x, 40);
        assert_eq!(s.screen_y, 50);
        assert!(!s.hflip);
        assert!(!s.vflip);

        // Wrong context → no resolution.
        assert!(extract_modern_sprites(&frame, &atlas, CONTEXT + 1).is_empty());
    }

    /// hflip on an 8×8 sprite resolves the same single tile but propagates the flag.
    #[test]
    fn extract_modern_sprites_propagates_hflip() {
        const CONTEXT: u64 = 21;
        const TILE: u16 = 5;

        let cell = ModernIndexTile {
            id: 7,
            indices: [0u8; 64],
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        };
        let atlas = ModernSpriteIndexAtlas::from_keyed_cells_for_test(
            vec![cell],
            vec![((CONTEXT, TILE), 0)],
        );

        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        oam[0] = (50u16 << 8) | 40u16;
        oam[1] = 0x4000 | TILE; // bit 14 = hflip

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.obj.obj_size = 0;

        let sprites = extract_modern_sprites(&frame, &atlas, CONTEXT);

        assert_eq!(sprites.len(), 1);
        assert_eq!(sprites[0].cell_id, 7);
        assert!(sprites[0].hflip);
        assert!(!sprites[0].vflip);
    }

    /// Live-VRAM sprite decode: an 8×8 sprite using tile T from CHR page
    /// `tile_adr1 = B` must yield a cell whose pattern equals
    /// `decode_snes_4bpp_tile_indices` for the same words, and one instance with
    /// the matching cell_id/palette/screen position.
    #[test]
    fn extract_modern_sprites_from_vram_decodes_live_chr() {
        const B: usize = 0x1000; // CHR page word base (tile_adr1)
        const TILE: u16 = 3;
        let palette: u16 = 4;
        let priority: u16 = 2;
        let x: u16 = 40;
        let y: u16 = 50;

        let mut vram = vec![0u16; 0x8000];
        // Encode a known pattern for tile T at base B:
        //   row 0: bp0 = 0xFF → all 8 pixels index bit0 set (index 1)
        //   row 3: bp0/bp1 = 0xFF/0xFF → all 8 pixels index 3
        let tile_base = B + (TILE as usize) * 16;
        vram[tile_base] = 0x00FF; // y=0: bp0=0xFF, bp1=0x00
        vram[tile_base + 3] = 0xFFFF; // y=3: bp0=0xFF, bp1=0xFF

        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        oam[0] = (y << 8) | x;
        oam[1] = (palette << 9) | (priority << 12) | TILE; // bank 0, no flip

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.obj.obj_size = 0; // small = 8×8
        frame.obj.tile_adr1 = B as u16;
        frame.obj.tile_adr2 = 0;

        let (cells, sprites) = extract_modern_sprites_from_vram(&frame);

        assert_eq!(sprites.len(), 1, "one visible 8×8 sprite tile");
        let s = &sprites[0];
        assert_eq!(s.palette, 4);
        assert_eq!(s.priority, 2);
        assert_eq!(s.screen_x, 40);
        assert_eq!(s.screen_y, 50);
        assert!(!s.hflip);
        assert!(!s.vflip);

        // The referenced cell's pattern equals the unflipped decode of the words.
        let expected = decode_snes_4bpp_tile_indices(&vram, B, TILE);
        assert_ne!(expected, [0u8; 64], "pattern must be non-trivial");
        let cell = &cells[s.cell_id as usize];
        assert_eq!(cell.indices, expected);
        // Spot-check the crafted rows.
        assert!(cell.indices[0..8].iter().all(|&i| i == 1));
        assert!(cell.indices[24..32].iter().all(|&i| i == 3));
    }

    /// hflip propagates as a flag; the decoded cell stays UNFLIPPED (so a second
    /// hflipped sprite using the same tile dedups to the same cell).
    #[test]
    fn extract_modern_sprites_from_vram_propagates_hflip_keeps_cell_unflipped() {
        const B: usize = 0x2000;
        const TILE: u16 = 7;

        let mut vram = vec![0u16; 0x8000];
        let tile_base = B + (TILE as usize) * 16;
        vram[tile_base] = 0x00FF; // row 0 index 1

        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        // Sprite 0: no flip. Sprite 1: hflip, same tile.
        oam[0] = (50u16 << 8) | 40u16;
        oam[1] = TILE;
        oam[2] = (60u16 << 8) | 80u16;
        oam[3] = 0x4000 | TILE; // bit 14 = hflip

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.obj.obj_size = 0;
        frame.obj.tile_adr1 = B as u16;

        let (cells, sprites) = extract_modern_sprites_from_vram(&frame);

        assert_eq!(sprites.len(), 2);
        // Both reference the SAME (unflipped) cell — hflip is a per-instance flag.
        assert_eq!(sprites[0].cell_id, sprites[1].cell_id);
        assert_eq!(cells.len(), 1, "identical tiles dedup to one cell");
        assert!(!sprites[0].hflip);
        assert!(sprites[1].hflip);

        let expected = decode_snes_4bpp_tile_indices(&vram, B, TILE);
        assert_eq!(cells[sprites[1].cell_id as usize].indices, expected);
    }

    fn test_gpu_frame<'a>(
        vram: &'a [u16],
        cgram: &'a [u16],
        oam: &'a [u16],
        brightness: u8,
        forced_blank: bool,
    ) -> GpuFrame<'a> {
        GpuFrame {
            vram,
            cgram,
            oam,
            mode: 1,
            bg: Default::default(),
            obj: Default::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Default::default(),
            screen_enabled: [0, 0],
            screen_windowed: [0, 0],
            brightness,
            forced_blank,
            math_enabled: 0,
            subtract_color: false,
            half_color: false,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            add_subscreen: false,
            clip_mode: 0,
            prevent_math_mode: 0,
            windowsel_cm: 0,
            windowsel: 0,
            scanlines: Box::new([ScanlineRegs::default(); 224]),
        }
    }

    #[test]
    fn obj_per_scanline_sprite_limit_drops_33rd_sprite() {
        // The SNES OBJ evaluator draws at most 32 sprites per scanline (range over);
        // `compute_obj_drawn_tiles` must reproduce that so the instance compositor
        // drops the same sprites the classic renderer does (frame 198300 regression).
        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        // 33 8×8 sprites, all overlapping scanline y=10 at distinct x.
        for s in 0..33usize {
            oam[s * 2] = (10u16 << 8) | (s as u16); // y=10, x=s
            oam[s * 2 + 1] = 0; // tile 0, palette 0, no flip
        }
        let frame = test_gpu_frame(&vram, &cgram, &oam, 15, false); // obj_size 0 → 8×8
        let drawn = compute_obj_drawn_tiles(&frame);
        // Line 11 (out_y=10) is where all 33 sprites are visible; only 32 survive.
        assert_eq!(
            drawn[10].len(),
            32,
            "per-scanline sprite limit should keep 32"
        );
        assert!(
            drawn[10].iter().all(|&(sn, _)| sn < 32),
            "the 33rd sprite (num 32) must be dropped"
        );
        // A scanline none of the sprites cover stays empty.
        assert!(drawn[50].is_empty());
    }

    #[test]
    fn obj_row_mask_reflects_per_scanline_budget() {
        // A single sprite on an uncrowded line is fully visible (row_mask covers its
        // 8 rows within the screen).
        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        oam[0] = (10u16 << 8) | 20u16; // one sprite at y=10, x=20
        let frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        let drawn = compute_obj_drawn_tiles(&frame);
        let mask = obj_row_mask(&drawn, 0, 20, 10);
        assert_eq!(mask, 0xff, "an uncontended 8×8 sprite draws all 8 rows");
    }

    #[test]
    fn mode7_bg_samples_affine_field_through_tilemap_and_palette() {
        // Identity Mode-7 transform: screen (0,0) samples texture (0,1) (the SNES
        // scanline+1 vertical offset). Prove the affine sample follows the tilemap
        // indirection (entry -> tile -> CHR texel) and resolves via CGRAM.
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x0002; // tilemap entry (0,0): low byte = tile number 2
        vram[2 * 64 + (1 * 8)] = 5u16 << 8; // tile 2, texel row1 col0: high byte = index 5
        let mut cgram = vec![0u16; 0x100];
        cgram[5] = 0x7C1F; // BGR555: R=31, G=0, B=31 (magenta)
        let oam = vec![0u16; 0x110];
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 7;
        frame.screen_enabled = [0x01, 0]; // BG1 on main
        for sl in frame.scanlines.iter_mut() {
            sl.mode7_matrix = [256, 0, 0, 256, 0, 0, 0, 0]; // identity (8.8 fixed)
            sl.screen_enabled_main = 0x01; // per-scanline BG1 enable (TM)
        }
        let rgba = crate::modern_software::render_modern_mode7_frame(&frame);
        // Pixel (0,0) resolves to CGRAM[5] (magenta), NOT the black backdrop.
        assert_eq!(rgba[1], 0, "green channel of magenta");
        assert!(
            rgba[0] > 200 && rgba[2] > 200,
            "mode7 pixel (0,0) should be CGRAM[5] magenta via tile-2 affine sample, got {:?}",
            &rgba[0..4]
        );
        assert_eq!(rgba[3], 0xff);
    }
}
