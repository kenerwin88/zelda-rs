// sprite.wgsl — SNES OBJ (sprite) renderer
//
// Renders all 8×8 sprite sub-tiles uploaded by SpriteRenderer::prepare().
// Each tile is drawn as a 2-triangle quad using a uniform-buffer-driven vertex
// shader. Transparent pixels (palette index 0) are discarded.
//
// Per-scanline TM check: pixels are discarded when bit 4 (OBJ) of the main-screen
// TM register is clear for that scanline. Set math_bit_pos=255 to skip the check
// (used for sub-screen renders).
//
// Alpha encoding: outputs alpha = layer_bit_pos/255.0 so the post-process shader
// can selectively apply color math. Main-screen sprites usually use bit 4 (OBJ).
// Sprites from OBJ palettes 0-3 use layer 6, matching the CPU renderer's
// no-color-math sentinel. For sub-screen renders (math_bit_pos=255), outputs
// alpha=1.0.
//
// Bindings:
//   0 — tile_atlas      R8Uint 512×256   decoded 4bpp palette indices
//   1 — cgram_palette   Rgba8Unorm 256×1 decoded CGRAM colours
//   2 — SpriteUniforms  uniform buffer   tile count + TM data + per-tile data

struct SpriteUniforms {
    tile_count:      u32,   // 0
    math_bit_pos:    u32,   // 4  — bit pos in math_enabled (4=OBJ), or 255=sub-screen
    priority_filter: u32,   // 8  — OAM priority 0..3, or 255=all
    _pad0:           u32,   // 12
    // 224 scanlines of screen_enabled[0], packed 4 per vec4 (low byte each).
    scanline_tm:  array<vec4<u32>, 56>,   // 16..911
    // Per-tile data.
    tiles:        array<vec4<u32>, 512>,  // 912..9103
}

@group(0) @binding(0) var tile_atlas:    texture_2d<u32>;  // R8Uint 512×256
@group(0) @binding(1) var cgram_palette: texture_2d<f32>;  // Rgba8Unorm 256×1
@group(0) @binding(2) var<uniform>       uni: SpriteUniforms;

struct VertexOut {
    @builtin(position)                 pos:      vec4<f32>,
    @location(0)                       uv:       vec2<f32>,
    @location(1) @interpolate(flat)    tile_idx: u32,
}

// Quad vertex table: 2 triangles, 6 vertices, CCW winding.
fn quad_uv(vi: u32) -> vec2<f32> {
    let xs = array<f32, 6>(0.0, 1.0, 1.0, 0.0, 1.0, 0.0);
    let ys = array<f32, 6>(0.0, 0.0, 1.0, 0.0, 1.0, 1.0);
    return vec2<f32>(xs[vi], ys[vi]);
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOut {
    let tile_idx = vi / 6u;
    let vert    = vi % 6u;

    let td  = uni.tiles[tile_idx];
    let sx  = bitcast<i32>(td.x);
    let sy  = bitcast<i32>(td.y);

    let uv = quad_uv(vert);
    let px = f32(sx) + uv.x * 8.0;
    let py = f32(sy) + uv.y * 8.0;

    let clip_x = px / 128.0 - 1.0;
    let clip_y = 1.0 - py / 112.0;

    var out: VertexOut;
    out.pos      = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.uv       = uv;
    out.tile_idx = tile_idx;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // Per-scanline TM check for main-screen renders.
    // math_bit_pos=255 means sub-screen; skip the check.
    if uni.math_bit_pos != 255u {
        let sy = u32(in.pos.y);
        let tm_val = uni.scanline_tm[sy / 4u][sy % 4u] & 0xFFu;
        if (tm_val & 0x10u) == 0u { discard; }  // bit4 = OBJ
    }

    let td          = uni.tiles[in.tile_idx];
    let atlas_slot  = td.z;
    let palette_base = td.w & 0xFFFFu;
    let flags       = td.w >> 16u;
    let hflip       = (flags & 1u) != 0u;
    let vflip       = (flags >> 1u & 1u) != 0u;
    let no_color_math = (flags >> 2u & 1u) != 0u;
    let priority = (flags >> 3u) & 3u;
    let row_mask = (flags >> 5u) & 0xffu;

    if uni.priority_filter != 255u && priority != uni.priority_filter {
        discard;
    }

    var tx = u32(in.uv.x * 8.0);
    var ty = u32(in.uv.y * 8.0);
    tx = min(tx, 7u);
    ty = min(ty, 7u);

    // TODO: Re-enable once the mask is validated against the old-renderer
    // scanline numbering. The current mask drops valid OBJ pixels on frames 900+.
    _ = row_mask;

    tx = select(tx, 7u - tx, hflip);
    ty = select(ty, 7u - ty, vflip);

    let atlas_x = i32((atlas_slot % 64u) * 8u + tx);
    let atlas_y = i32((atlas_slot / 64u) * 8u + ty);
    let pal_idx = textureLoad(tile_atlas, vec2i(atlas_x, atlas_y), 0).r;

    if pal_idx == 0u { discard; }

    let cgram_idx = palette_base + pal_idx;
    let color = textureLoad(cgram_palette, vec2i(i32(cgram_idx), 0), 0);

    let layer_bit_pos = select(uni.math_bit_pos, 6u, no_color_math && uni.math_bit_pos != 255u);
    let out_alpha = select(f32(layer_bit_pos) / 255.0, 1.0, uni.math_bit_pos >= 255u);
    return vec4<f32>(color.rgb, out_alpha);
}
