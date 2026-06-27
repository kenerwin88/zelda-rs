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
    // Local pixel coordinate within the tile (0..w, 0..h), interpolated.
    @location(0) local: vec2<f32>,
    // Atlas x,y,w,h for this instance (flat — same across the quad).
    @location(1) @interpolate(flat) atlas: vec4<u32>,
    // bit0 = hflip, bit1 = vflip, bit2 = transparent_color_zero.
    @location(2) @interpolate(flat) flags: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) atlas_xywh: vec4<u32>,
    @location(1) screen_xy: vec2<i32>,
    @location(2) flags: u32,
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
    let wh = vec2<f32>(f32(atlas_xywh.z), f32(atlas_xywh.w));
    let local = c * wh; // tile-local pixel coords at the quad corners
    let screen = vec2<f32>(f32(screen_xy.x), f32(screen_xy.y)) + local;

    // Map screen-pixel edge coords to clip space over the 256x224 viewport.
    let ndc_x = screen.x / 256.0 * 2.0 - 1.0;
    let ndc_y = 1.0 - screen.y / 224.0 * 2.0;

    var out: VsOut;
    out.pos = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local = local;
    out.atlas = atlas_xywh;
    out.flags = flags;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let w = in.atlas.z;
    let h = in.atlas.w;

    // Integer tile-local index for this fragment's screen pixel.
    var lx = u32(floor(in.local.x));
    var ly = u32(floor(in.local.y));
    if (lx >= w) { lx = w - 1u; }
    if (ly >= h) { ly = h - 1u; }

    var sx = lx;
    var sy = ly;
    if ((in.flags & 1u) != 0u) { sx = w - 1u - lx; }
    if ((in.flags & 2u) != 0u) { sy = h - 1u - ly; }

    let ax = i32(in.atlas.x + sx);
    let ay = i32(in.atlas.y + sy);

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
