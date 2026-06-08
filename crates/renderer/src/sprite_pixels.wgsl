// sprite_pixels.wgsl - resolved SNES OBJ buffer renderer.
//
// SpriteRenderer resolves OAM into a 256x224 Rgba8Uint texture on the CPU:
//   r = CGRAM index
//   g = OAM priority 0..3
//   b = layer bit position for color math (4=OBJ, 6=no-math OBJ sentinel)
//   a = occupied flag
//
// This shader draws that resolved OBJ buffer through the same priority-filtered
// pass schedule as BG compositing.

struct SpriteUniforms {
    has_pixels:      u32,
    math_bit_pos:    u32,
    priority_filter: u32,
    window_flags:    u32,
    windowed:        u32,
    _pad0:           u32,
    _pad1:           u32,
    _pad2:           u32,
    scanline_tm:     array<vec4<u32>, 56>,
    scanline_window: array<vec4<u32>, 56>,
}

@group(0) @binding(0) var sprite_pixels: texture_2d<u32>;
@group(0) @binding(1) var cgram_palette: texture_2d<f32>;
@group(0) @binding(2) var<uniform> uni: SpriteUniforms;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let x = f32(vi & 1u) * 4.0 - 1.0;
    let y = 1.0 - f32((vi >> 1u) & 1u) * 4.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

fn obj_window_active(sx: u32, sy: u32) -> bool {
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
    if uni.has_pixels == 0u {
        discard;
    }

    let sx = u32(frag_pos.x);
    let sy = u32(frag_pos.y);

    if uni.math_bit_pos != 255u {
        let tm_val = uni.scanline_tm[sy / 4u][sy % 4u] & 0xffu;
        if (tm_val & 0x10u) == 0u {
            discard;
        }
    }

    if obj_window_active(sx, sy) {
        discard;
    }

    let p = textureLoad(sprite_pixels, vec2i(i32(sx), i32(sy)), 0);
    if p.a == 0u {
        discard;
    }

    let priority = p.g;
    if uni.priority_filter != 255u && priority != uni.priority_filter {
        discard;
    }

    let color = textureLoad(cgram_palette, vec2i(i32(p.r), 0), 0);
    let layer_bit_pos = select(p.b, uni.math_bit_pos, uni.math_bit_pos == 255u);
    let out_alpha = select(f32(layer_bit_pos) / 255.0, 1.0, uni.math_bit_pos >= 255u);
    return vec4<f32>(color.rgb, out_alpha);
}
