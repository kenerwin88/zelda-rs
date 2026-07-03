// Modern BG tile renderer: instanced atlas-sampled quads.
//
// Each instance is one ModernTileInstance. The quad covers the screen pixels
// [screen_x, screen_x+w) x [screen_y, screen_y+h). The fragment for screen pixel
// p (center p+0.5) maps to atlas texel floor(local) via an integer `textureLoad`,
// which avoids all filtering/half-texel ambiguity so the GPU output is byte-exact
// with `render_modern_frame_software` (nearest, REPLACE, no gamma).

@group(0) @binding(0) var atlas_tex: texture_2d<f32>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    // Local SCREEN pixel coordinate within the tile footprint (0..sw, 0..sh).
    @location(0) local: vec2<f32>,
    // Atlas SOURCE rect x,y,w,h for this instance (flat — same across the quad).
    @location(1) @interpolate(flat) atlas: vec4<u32>,
    // On-screen footprint w,h (flat).
    @location(2) @interpolate(flat) screen_wh: vec2<u32>,
    // bit0 = hflip, bit1 = vflip, bit2 = transparent_color_zero.
    @location(3) @interpolate(flat) flags: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) atlas_xywh: vec4<u32>,
    @location(1) screen_xy: vec2<i32>,
    @location(2) screen_wh: vec2<u32>,
    @location(3) flags: u32,
) -> VsOut {
    // Two triangles forming the unit quad.
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let c = corners[vi];
    // The quad's footprint is the on-screen size, not the (upscaled) source rect.
    let swh = vec2<f32>(f32(screen_wh.x), f32(screen_wh.y));
    let local = c * swh; // screen-local pixel coords at the quad corners
    let screen = vec2<f32>(f32(screen_xy.x), f32(screen_xy.y)) + local;

    // Map screen-pixel edge coords to clip space over the 256x224 viewport.
    let ndc_x = screen.x / 256.0 * 2.0 - 1.0;
    let ndc_y = 1.0 - screen.y / 224.0 * 2.0;

    var out: VsOut;
    out.pos = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local = local;
    out.atlas = atlas_xywh;
    out.screen_wh = screen_wh;
    out.flags = flags;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let sw = in.screen_wh.x;
    let sh = in.screen_wh.y;

    // Integer screen-local index for this fragment's pixel within the footprint.
    var lx = u32(floor(in.local.x));
    var ly = u32(floor(in.local.y));
    if (lx >= sw) { lx = sw - 1u; }
    if (ly >= sh) { ly = sh - 1u; }

    // Mirror on the SCREEN coordinate first.
    var slx = lx;
    var sly = ly;
    if ((in.flags & 1u) != 0u) { slx = sw - 1u - lx; }
    if ((in.flags & 2u) != 0u) { sly = sh - 1u - ly; }

    // Downsample factor source -> screen; sample each block's top-left texel.
    let scale_x = in.atlas.z / sw;
    let scale_y = in.atlas.w / sh;

    let ax = i32(in.atlas.x + slx * scale_x);
    let ay = i32(in.atlas.y + sly * scale_y);

    // Atlas out-of-bounds: leave the existing pixel (software does `continue`).
    let dim = textureDimensions(atlas_tex);
    if (ax >= i32(dim.x) || ay >= i32(dim.y)) {
        discard;
    }

    let color = textureLoad(atlas_tex, vec2<i32>(ax, ay), 0);

    // transparent_color_zero: skip alpha-zero texels.
    if ((in.flags & 4u) != 0u && color.a == 0.0) {
        discard;
    }

    return color;
}

// ── Palette-index path ───────────────────────────────────────────────────────
//
// Tiles are stored as 8x8 grids of 4-bit palette INDICES (0..15) in an R8Uint
// atlas (one 8x8 cell per grid slot). The live CGRAM is a 256x1 Rgba8Unorm
// texture. For each fragment: fetch the palette index via integer textureLoad,
// discard index 0 (transparent), then look up the final color at
// `cgram[palette*16 + index]`. No hflip/vflip is applied here — the flip is
// already baked into the atlas index pattern. This matches
// `render_modern_frame_software_indexed` byte-for-byte.

@group(0) @binding(2) var index_atlas: texture_2d<u32>;
@group(0) @binding(3) var cgram_tex: texture_2d<f32>;

struct VsIndexOut {
    @builtin(position) pos: vec4<f32>,
    // Local SCREEN pixel coordinate within the 8x8 tile (0..8).
    @location(0) local: vec2<f32>,
    // Top-left texel of this cell's 8x8 region in the index atlas (flat).
    @location(1) @interpolate(flat) cell_origin: vec2<u32>,
    // Palette number 0..7 (flat).
    @location(2) @interpolate(flat) palette: u32,
};

@vertex
fn vs_index(
    @builtin(vertex_index) vi: u32,
    @location(0) cell_origin: vec2<u32>,
    @location(1) screen_xy: vec2<i32>,
    @location(2) palette: u32,
) -> VsIndexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let c = corners[vi];
    // Index tiles always have an 8x8 on-screen footprint (no atlas scaling).
    let local = c * 8.0;
    let screen = vec2<f32>(f32(screen_xy.x), f32(screen_xy.y)) + local;

    let ndc_x = screen.x / 256.0 * 2.0 - 1.0;
    let ndc_y = 1.0 - screen.y / 224.0 * 2.0;

    var out: VsIndexOut;
    out.pos = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local = local;
    out.cell_origin = cell_origin;
    out.palette = palette;
    return out;
}

@fragment
fn fs_index(in: VsIndexOut) -> @location(0) vec4<f32> {
    var lx = u32(floor(in.local.x));
    var ly = u32(floor(in.local.y));
    if (lx >= 8u) { lx = 7u; }
    if (ly >= 8u) { ly = 7u; }

    let ax = i32(in.cell_origin.x + lx);
    let ay = i32(in.cell_origin.y + ly);
    let index = textureLoad(index_atlas, vec2<i32>(ax, ay), 0).r;

    // Index 0 is transparent — leave the backdrop / lower layer showing.
    if (index == 0u) {
        discard;
    }

    let cx = i32(in.palette) * 16 + i32(index);
    return textureLoad(cgram_tex, vec2<i32>(cx, 0), 0);
}

// ── Sprite (OBJ) index path ──────────────────────────────────────────────────
//
// Like the BG index path, but sprite cells are stored UNFLIPPED, so the fragment
// applies the instance's hflip/vflip when sampling the 8x8 cell. The final color
// comes from the OBJ half of CGRAM: `cgram[128 + palette*16 + index]` (palettes
// 8..15). Index 0 is transparent. Sprites are drawn over the BG (LoadOp::Load).
// Reuses @binding(2)=index_atlas (here the SPRITE atlas) and @binding(3)=cgram.
// Matches `draw_modern_sprites_indexed` byte-for-byte.

struct VsSpriteOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) cell_origin: vec2<u32>,
    @location(2) @interpolate(flat) palette: u32,
    // bit0 = hflip, bit1 = vflip.
    @location(3) @interpolate(flat) flags: u32,
};

@vertex
fn vs_sprite(
    @builtin(vertex_index) vi: u32,
    @location(0) cell_origin: vec2<u32>,
    @location(1) screen_xy: vec2<i32>,
    @location(2) palette: u32,
    @location(3) flags: u32,
) -> VsSpriteOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let c = corners[vi];
    let local = c * 8.0;
    let screen = vec2<f32>(f32(screen_xy.x), f32(screen_xy.y)) + local;

    let ndc_x = screen.x / 256.0 * 2.0 - 1.0;
    let ndc_y = 1.0 - screen.y / 224.0 * 2.0;

    var out: VsSpriteOut;
    out.pos = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local = local;
    out.cell_origin = cell_origin;
    out.palette = palette;
    out.flags = flags;
    return out;
}

@fragment
fn fs_sprite(in: VsSpriteOut) -> @location(0) vec4<f32> {
    var lx = u32(floor(in.local.x));
    var ly = u32(floor(in.local.y));
    if (lx >= 8u) { lx = 7u; }
    if (ly >= 8u) { ly = 7u; }

    // Sprite cells are UNFLIPPED — apply flip when sampling.
    var sx = lx;
    var sy = ly;
    if ((in.flags & 1u) != 0u) { sx = 7u - lx; }
    if ((in.flags & 2u) != 0u) { sy = 7u - ly; }

    let ax = i32(in.cell_origin.x + sx);
    let ay = i32(in.cell_origin.y + sy);
    let index = textureLoad(index_atlas, vec2<i32>(ax, ay), 0).r;

    if (index == 0u) {
        discard;
    }

    let cx = 128 + i32(in.palette) * 16 + i32(index);
    return textureLoad(cgram_tex, vec2<i32>(cx, 0), 0);
}

// ── Variant effect path ─────────────────────────────────────────────────────
//
// Source tile indices are sampled from an R8Uint atlas, then mapped through a
// compact palette/effect LUT texture. This is the GPU equivalent of
// `render_modern_frame_software_variant_atlas`'s stable `palette_lut` path.

@group(0) @binding(4) var effect_lut_tex: texture_2d<f32>;

struct VsEffectOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) cell_origin: vec2<u32>,
    // bit0 = hflip, bit1 = vflip, bits 8..15 = row mask.
    @location(2) @interpolate(flat) flags: u32,
    @location(3) @interpolate(flat) effect_row: u32,
};

@vertex
fn vs_effect(
    @builtin(vertex_index) vi: u32,
    @location(0) cell_origin: vec2<u32>,
    @location(1) screen_xy: vec2<i32>,
    @location(2) flags: u32,
    @location(3) effect_row: u32,
) -> VsEffectOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let c = corners[vi];
    let local = c * 8.0;
    let screen = vec2<f32>(f32(screen_xy.x), f32(screen_xy.y)) + local;

    let ndc_x = screen.x / 256.0 * 2.0 - 1.0;
    let ndc_y = 1.0 - screen.y / 224.0 * 2.0;

    var out: VsEffectOut;
    out.pos = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local = local;
    out.cell_origin = cell_origin;
    out.flags = flags;
    out.effect_row = effect_row;
    return out;
}

@fragment
fn fs_effect(in: VsEffectOut) -> @location(0) vec4<f32> {
    var lx = u32(floor(in.local.x));
    var ly = u32(floor(in.local.y));
    if (lx >= 8u) { lx = 7u; }
    if (ly >= 8u) { ly = 7u; }

    var sx = lx;
    var sy = ly;
    if ((in.flags & 1u) != 0u) { sx = 7u - lx; }
    if ((in.flags & 2u) != 0u) { sy = 7u - ly; }
    if (((in.flags >> (8u + ly)) & 1u) == 0u) {
        discard;
    }

    let ax = i32(in.cell_origin.x + sx);
    let ay = i32(in.cell_origin.y + sy);
    let index = textureLoad(index_atlas, vec2<i32>(ax, ay), 0).r;
    if (index == 0u) {
        discard;
    }

    return textureLoad(effect_lut_tex, vec2<i32>(i32(index), i32(in.effect_row)), 0);
}
