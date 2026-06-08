// bg_layer.wgsl — SNES BG layer renderer (supports 4bpp and 2bpp, priority filtering)
//
// Renders one BG layer to a 256×224 intermediate texture. Transparent pixels
// (palette index 0) are discarded. Priority filtering discards tiles that don't
// match the current pass (lo or hi priority). Call twice per layer to implement
// SNES priority order: lo-pass then hi-pass with other layers interleaved.
//
// Per-scanline TM check: each pixel is discarded if the main-screen TM register
// (screen_enabled[0]) does not have this layer's bit set for that scanline.
// This matches the CPU PPU which uses HDMA-updated TM values per row.
// Set layer_bit=0 to skip the TM check (e.g. when rendering to the sub-screen).
//
// Alpha encoding for color math: the output alpha encodes the SNES math_enabled
// bit position for this layer (0=BG1, 1=BG2, 2=BG3, 3=BG4). The post-process
// shader reads this to apply color math only to layers that have it enabled.
// Set math_bit_pos=255 to output alpha=1.0 (used for sub-screen renders where
// alpha=1 means "real pixel" and alpha=0 means "backdrop").
//
// Uniform buffer layout (10944 bytes total):
//   bytes 0–3:    h_scroll
//   bytes 4–7:    v_scroll
//   bytes 8–11:   atlas_slot_base
//   bytes 12–15:  tilemap_width
//   bytes 16–19:  tilemap_height
//   bytes 20–23:  is_2bpp
//   bytes 24–27:  hi_priority_only
//   bytes 28–31:  layer_bit  (main-screen TM bit for this layer; 0 = skip TM check)
//   bytes 32–35:  math_bit_pos  (SNES math_enabled bit position, or 255 for sub-screen)
//   bytes 36–39:  window_flags (W1inv, W1en, W2inv, W2en)
//   bytes 40–43:  windowed (whether this layer uses window masking)
//   bytes 44–47:  mosaic_enabled
//   bytes 48–51:  mosaic_size
//   bytes 52–63:  padding
//   bytes 64–959: scanline_tm (56 vec4<u32>; low byte of each u32 = screen_enabled[0])
//   bytes 960–1855: scanline_scroll (56 vec4<u32>; h_scroll | v_scroll<<16)
//   bytes 1856–2751: scanline_window (56 vec4<u32>; w1l | w1r<<8 | w2l<<16 | w2r<<24)
//   bytes 2752–10943: tilemap_data (array<vec4<u32>, 512>)

struct BgUniforms {
    h_scroll:         u32,  // 0
    v_scroll:         u32,  // 4
    atlas_slot_base:  u32,  // 8
    tilemap_width:    u32,  // 12
    tilemap_height:   u32,  // 16
    is_2bpp:          u32,  // 20
    hi_priority_only: u32,  // 24
    layer_bit:        u32,  // 28 — TM bit for this layer (0 = skip check)
    math_bit_pos:     u32,  // 32 — bit position in math_enabled, or 255 for sub-screen
    window_flags:     u32,  // 36 — W1inv, W1en, W2inv, W2en
    windowed:         u32,  // 40 — screen_windowed bit for this layer
    mosaic_enabled:   u32,  // 44
    mosaic_size:      u32,  // 48
    _pad2:            u32,  // 52
    _pad3:            u32,  // 56
    _pad4:            u32,  // 60
    // 224 scanlines of screen_enabled[0], 4 per vec4 (low byte of each component).
    scanline_tm:      array<vec4<u32>, 56>,   // 64..959
    // 224 scanlines of scroll for this BG layer: h_scroll | (v_scroll << 16).
    scanline_scroll:  array<vec4<u32>, 56>,   // 960..1855
    // 224 scanlines of window bounds: w1l | w1r<<8 | w2l<<16 | w2r<<24.
    scanline_window:  array<vec4<u32>, 56>,   // 1856..2751
    // Up to 4096 u16 tilemap entries (64×64 tiles).
    tilemap_data:     array<vec4<u32>, 512>,  // 2752..10943
}

@group(0) @binding(0) var tile_atlas:    texture_2d<u32>;  // R8Uint  512×256
@group(0) @binding(1) var cgram_palette: texture_2d<f32>;  // Rgba8Unorm 256×1
@group(0) @binding(2) var<uniform>       uni: BgUniforms;

// Fullscreen triangle.
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let x = f32(vi & 1u) * 4.0 - 1.0;
    let y = 1.0 - f32((vi >> 1u) & 1u) * 4.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

fn layer_window_active(sx: u32, sy: u32) -> bool {
    if uni.windowed == 0u {
        return false;
    }

    let w1_enabled = (uni.window_flags & 0x2u) != 0u;
    let w2_enabled = (uni.window_flags & 0x8u) != 0u;
    if !w1_enabled && !w2_enabled {
        return false;
    }

    let packed = uni.scanline_window[sy / 4u][sy % 4u];
    let w1l = packed & 0xffu;
    let w1r = (packed >> 8u) & 0xffu;
    let w2l = (packed >> 16u) & 0xffu;
    let w2r = (packed >> 24u) & 0xffu;

    var test1 = sx >= w1l && sx <= w1r;
    var test2 = sx >= w2l && sx <= w2r;
    if (uni.window_flags & 0x1u) != 0u {
        test1 = !test1;
    }
    if (uni.window_flags & 0x4u) != 0u {
        test2 = !test2;
    }

    if w1_enabled && !w2_enabled {
        return test1;
    }
    if !w1_enabled && w2_enabled {
        return test2;
    }
    return test1 || test2;
}

@fragment
fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    let sx = u32(frag_pos.x);
    let sy = u32(frag_pos.y);

    // Per-scanline TM check: discard if this layer is disabled at this row.
    // layer_bit=0 skips the check (used for sub-screen renders).
    if uni.layer_bit != 0u {
        let tm_val = uni.scanline_tm[sy / 4u][sy % 4u] & 0xFFu;
        if (tm_val & uni.layer_bit) == 0u { discard; }
    }

    if layer_window_active(sx, sy) {
        discard;
    }

    // Scroll like the CPU old renderer: keep the 10-bit BG coordinate, then
    // use bit 8 only when the tilemap has the matching 32x32 page.
    let packed_scroll = uni.scanline_scroll[sy / 4u][sy % 4u];
    let h_scroll = packed_scroll & 0xffffu;
    let v_scroll = (packed_scroll >> 16u) & 0xffffu;

    var source_x = sx;
    // SNES PPU renders scanlines starting at y=1; output row 0 = scanline 1.
    var source_y = sy + 1u;
    if uni.mosaic_enabled != 0u && uni.mosaic_size > 1u {
        source_x = source_x - (source_x % uni.mosaic_size);
        source_y = source_y - ((source_y - 1u) % uni.mosaic_size);
    }
    let scroll_x = (source_x + h_scroll) & 0x3ffu;
    let scroll_y = (source_y + v_scroll) & 0x3ffu;

    // Tile grid index and sub-tile pixel offset.
    var tile_x = (scroll_x >> 3u) & 0x1fu;
    var tile_y = (scroll_y >> 3u) & 0x1fu;
    if uni.tilemap_width > 32u && (scroll_x & 0x100u) != 0u {
        tile_x = tile_x + 32u;
    }
    if uni.tilemap_height > 32u && (scroll_y & 0x100u) != 0u {
        tile_y = tile_y + 32u;
    }
    let pixel_x = scroll_x % 8u;
    let pixel_y = scroll_y % 8u;

    // Tilemap entry: [15:vflip][14:hflip][13:priority][12:10:palette][9:0:tile_num]
    let flat_idx   = tile_y * uni.tilemap_width + tile_x;
    let vec_idx    = flat_idx / 8u;
    let sub        = flat_idx % 8u;
    let u32_in_vec = sub / 2u;
    let half       = sub % 2u;
    let word       = uni.tilemap_data[vec_idx][u32_in_vec];
    let entry      = (word >> (half * 16u)) & 0xFFFFu;

    let tile_num    = entry & 0x3FFu;
    let palette_sub = (entry >> 10u) & 0x7u;
    let tile_priority = (entry >> 13u) & 1u;
    let hflip = (entry >> 14u) & 1u;
    let vflip = (entry >> 15u) & 1u;

    // Discard tiles that don't match the priority pass for this draw call.
    if tile_priority != uni.hi_priority_only {
        discard;
    }

    // Apply horizontal / vertical tile flip.
    let px = select(pixel_x, 7u - pixel_x, hflip != 0u);
    let py = select(pixel_y, 7u - pixel_y, vflip != 0u);

    var pal_idx:  u32;
    var cgram_idx: u32;

    if uni.is_2bpp != 0u {
        // 2bpp tile (BG3 in Mode 1): two 2bpp tiles share one 4bpp atlas slot.
        let slot    = uni.atlas_slot_base + tile_num / 2u;
        let atlas_x = i32((slot % 64u) * 8u + px);
        let atlas_y = i32((slot / 64u) * 8u + py);
        let raw     = textureLoad(tile_atlas, vec2i(atlas_x, atlas_y), 0).r;
        let shift   = (tile_num & 1u) * 2u;
        pal_idx     = (raw >> shift) & 3u;
        if pal_idx == 0u { discard; }
        cgram_idx = palette_sub * 4u + pal_idx;
    } else {
        // 4bpp tile: one atlas slot per tile.
        let slot    = uni.atlas_slot_base + tile_num;
        let atlas_x = i32((slot % 64u) * 8u + px);
        let atlas_y = i32((slot / 64u) * 8u + py);
        pal_idx     = textureLoad(tile_atlas, vec2i(atlas_x, atlas_y), 0).r;
        if pal_idx == 0u { discard; }
        cgram_idx = palette_sub * 16u + pal_idx;
    }

    let color = textureLoad(cgram_palette, vec2i(i32(cgram_idx), 0), 0);

    // Alpha encodes the SNES math_enabled bit position for this layer so the
    // post-process shader can apply color math only to eligible layers.
    // math_bit_pos=255 is the sub-screen sentinel: output alpha=1.0.
    let out_alpha = select(f32(uni.math_bit_pos) / 255.0, 1.0, uni.math_bit_pos >= 255u);
    return vec4<f32>(color.rgb, out_alpha);
}
