struct Mode7Uniforms {
    m0: u32,
    m1: u32,
    m2: u32,
    m3: u32,
    m4: u32,
    m5: u32,
    m6: u32,
    m7: u32,
    flags: u32,
    layer_bit: u32,
    math_bit_pos: u32,
    window_flags: u32,
    windowed: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    scanline_tm: array<vec4<u32>, 56>,
    scanline_window: array<vec4<u32>, 56>,
    scanline_m7: array<vec4<u32>, 448>,
}

@group(0) @binding(0) var cgram_palette: texture_2d<f32>;
@group(0) @binding(1) var<storage, read> vram: array<u32>;
@group(0) @binding(2) var<uniform> uni: Mode7Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let x = f32(vi & 1u) * 4.0 - 1.0;
    let y = 1.0 - f32((vi >> 1u) & 1u) * 4.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

fn i32_from_u32_bits(value: u32) -> i32 {
    if (value & 0x80000000u) != 0u {
        return -i32((~value + 1u) & 0x7fffffffu);
    }
    return i32(value);
}

fn expand_m7_13(value: i32) -> i32 {
    let masked = value & 0x1fff;
    if (masked & 0x1000) != 0 {
        return masked | -8192;
    }
    return masked;
}

fn clip_m7_offset(value: i32) -> i32 {
    if (value & 0x2000) != 0 {
        return value | -1024;
    }
    return value & 1023;
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

    if uni.layer_bit != 0u {
        let tm_val = uni.scanline_tm[sy / 4u][sy % 4u] & 0xffu;
        if (tm_val & uni.layer_bit) == 0u {
            discard;
        }
    }
    if layer_window_active(sx, sy) {
        discard;
    }

    let m_base = sy * 2u;
    let m_lo = uni.scanline_m7[m_base];
    let m_hi = uni.scanline_m7[m_base + 1u];
    let m0 = i32_from_u32_bits(m_lo.x);
    let m1 = i32_from_u32_bits(m_lo.y);
    let m2 = i32_from_u32_bits(m_lo.z);
    let m3 = i32_from_u32_bits(m_lo.w);
    let x_center = expand_m7_13(i32_from_u32_bits(m_hi.x));
    let y_center = expand_m7_13(i32_from_u32_bits(m_hi.y));
    let h_scroll = expand_m7_13(i32_from_u32_bits(m_hi.z));
    let v_scroll = expand_m7_13(i32_from_u32_bits(m_hi.w));
    let clipped_h = clip_m7_offset(h_scroll - x_center);
    let clipped_v = clip_m7_offset(v_scroll - y_center);

    let y = i32(sy) + 1;
    let ry = select(y, 255 - y, (uni.flags & 0x8u) != 0u);
    let start_x = (m0 * clipped_h & -64) + (m1 * ry & -64) + (m1 * clipped_v & -64) + (x_center << 8);
    let start_y = (m2 * clipped_h & -64) + (m3 * ry & -64) + (m3 * clipped_v & -64) + (y_center << 8);
    let rx = select(i32(sx), 255 - i32(sx), (uni.flags & 0x4u) != 0u);

    var x_pos = (start_x + m0 * rx) >> 8;
    var y_pos = (start_y + m2 * rx) >> 8;
    var outside = x_pos < 0 || x_pos >= 1024 || y_pos < 0 || y_pos >= 1024;
    x_pos = x_pos & 0x3ff;
    y_pos = y_pos & 0x3ff;
    if (uni.flags & 0x1u) == 0u {
        outside = false;
    }

    var tile: u32;
    if outside {
        tile = 0u;
    } else {
        tile = vram[u32((y_pos >> 3) * 128 + (x_pos >> 3))] & 0xffu;
    }

    var pixel: u32;
    if outside && (uni.flags & 0x2u) == 0u {
        pixel = 0u;
    } else {
        pixel = (vram[tile * 64u + u32((y_pos & 7) * 8 + (x_pos & 7))] >> 8u) & 0xffu;
    }
    if pixel == 0u {
        discard;
    }

    let color = textureLoad(cgram_palette, vec2i(i32(pixel), 0), 0);
    let out_alpha = select(f32(uni.math_bit_pos) / 255.0, 1.0, uni.math_bit_pos >= 255u);
    return vec4<f32>(color.rgb, out_alpha);
}
