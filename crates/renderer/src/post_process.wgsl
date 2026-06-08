// Post-process pass: applies SNES color math and brightness to the composited
// BG+sprite intermediate texture.
//
// Uniform layout (960 bytes total):
//   bytes 0-47:  scalar params (12 u32s)
//   bytes 48-63: padding
//   bytes 64-959: scanline window data (56 vec4<u32>, 4 scanlines per vec4)
//
// Each scanline entry packs window1_left, window1_right, window2_left, and
// window2_right into one u32.
//
// CW_BITS_MOD table used for clip/math mode decoding:
//   index: [0]=0x00  [1]=0xff  [2]=0xff  [3]=0x00  [4]=0xff  [5]=0x00  [6]=0xff  [7]=0x00

struct Params {
    brightness: u32,
    forced_blank: u32,
    math_enabled: u32,
    subtract_color: u32,
    half_color: u32,
    fixed_color_r: u32,
    fixed_color_g: u32,
    fixed_color_b: u32,
    add_subscreen: u32,
    clip_mode: u32,
    prevent_math_mode: u32,
    windowsel_cm: u32,
    rendered_subscreen: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    // 56 vec4<u32>; each component holds one scanline as
    // (w1l | w1r<<8 | w2l<<16 | w2r<<24)
    scanline_wins: array<vec4<u32>, 56>,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var comp_tex: texture_2d<f32>;
@group(0) @binding(2) var comp_samp: sampler;
@group(0) @binding(3) var sub_comp_tex: texture_2d<f32>;

// CW_BITS_MOD[mode] masks whether the window state contributes to clip/math.
// CW_BITS_MOD[mode+4] is the base value XOR'd with the masked window state.
// Mode 0: (w & 0x00) ^ 0xff = 0xff  → always 1 (never clip / always math)
// Mode 1: (w & 0xff) ^ 0x00 = w     → follows window (clip outside / math inside)
// Mode 2: (w & 0xff) ^ 0xff = !w    → inverted (clip inside / math outside)
// Mode 3: (w & 0x00) ^ 0x00 = 0x00  → always 0 (always clip / never math)
fn cw_bit(in_window: bool, mode: u32) -> bool {
    let w = select(0u, 0xffu, in_window);
    let masks = array<u32, 8>(0x00u, 0xffu, 0xffu, 0x00u, 0xffu, 0x00u, 0xffu, 0x00u);
    return ((w & masks[mode]) ^ masks[mode + 4u]) != 0u;
}

// Expand a 5-bit component and scale by brightness/15.
fn apply_brightness(v5: u32, brightness: u32) -> f32 {
    // Same expansion as the CPU: (v5 << 3) | (v5 >> 2)
    let v8 = (v5 << 3u) | (v5 >> 2u);
    let scaled = (v8 * brightness) / 15u;
    return f32(scaled) / 255.0;
}

struct VertOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertOut {
    // Full-screen triangle: vertices at (-1,-1), (3,-1), (-1,3) in clip space
    let x = select(-1.0, 3.0, vi == 1u);
    let y = select(-1.0, 3.0, vi == 2u);
    var out: VertOut;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertOut) -> @location(0) vec4<f32> {
    let sx = u32(in.pos.x);
    let sy = u32(in.pos.y);

    if p.forced_blank != 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Read per-scanline color-window boundaries.
    let vec_idx = sy / 4u;
    let comp_idx = sy % 4u;
    let packed = p.scanline_wins[vec_idx][comp_idx];
    let w1l = packed & 0xffu;
    let w1r = (packed >> 8u) & 0xffu;
    let w2l = (packed >> 16u) & 0xffu;
    let w2r = (packed >> 24u) & 0xffu;

    // Determine color-math window membership. The SNES combines enabled W1/W2
    // masks with OR after applying each window's inversion flag.
    // windowsel_cm bits: 0=W1inv, 1=W1en, 2=W2inv, 3=W2en
    var in_cm_window = false;
    if (p.windowsel_cm & 0x2u) != 0u {
        var in_w1 = w1l <= w1r && sx >= w1l && sx <= w1r;
        if (p.windowsel_cm & 0x1u) != 0u { in_w1 = !in_w1; }
        in_cm_window = in_cm_window || in_w1;
    }
    if (p.windowsel_cm & 0x8u) != 0u {
        var in_w2 = w2l <= w2r && sx >= w2l && sx <= w2r;
        if (p.windowsel_cm & 0x4u) != 0u { in_w2 = !in_w2; }
        in_cm_window = in_cm_window || in_w2;
    }

    // Clip mode: if clip bit is 0 (clipped), output black.
    let not_clipped = cw_bit(in_cm_window, p.clip_mode);
    if !not_clipped {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Exact SNES-pixel readback: the intermediate textures are 256x224, so
    // integer texel loads avoid backend-specific normalized-sampling rounding.
    let pixel = vec2i(i32(sx), i32(sy));
    let sample = textureLoad(comp_tex, pixel, 0);

    // Convert float RGB to 5-bit (CGRAM stores (v5 << 3), lower 3 bits = 0).
    var r5 = i32(u32(sample.r * 255.0 + 0.5) >> 3u);
    var g5 = i32(u32(sample.g * 255.0 + 0.5) >> 3u);
    var b5 = i32(u32(sample.b * 255.0 + 0.5) >> 3u);

    // Color math: apply only if this layer has math enabled and window allows it.
    // The comp_tex alpha encodes the math_enabled bit position for the layer that
    // painted this pixel: 0=BG1, 1=BG2, 2=BG3, 3=BG4, 4=OBJ, 5=backdrop.
    // This matches the CPU PPU which checks the winning layer's math_enabled bit.
    let layer_bit_pos = u32(sample.a * 255.0 + 0.5);
    let layer_math_on = ((p.math_enabled >> layer_bit_pos) & 1u) != 0u;
    let no_effect_math = p.fixed_color_r == 0u
        && p.fixed_color_g == 0u
        && p.fixed_color_b == 0u
        && p.half_color == 0u
        && p.rendered_subscreen == 0u;
    let do_math = !no_effect_math && cw_bit(in_cm_window, p.prevent_math_mode) && layer_math_on;
    if do_math {
        // Second operand: sub-screen pixel (when add_subscreen=true) or fixed color.
        var r2: i32;
        var g2: i32;
        var b2: i32;
        var second_layer_real = false;
        if p.add_subscreen != 0u {
            // sub_comp_tex alpha: 0 = backdrop (no BG/sprite covered this pixel),
            // 1 = real sub-screen pixel. When backdrop, the SNES PPU falls back to
            // fixed_color (as if second_layer == 5), matching hardware behavior.
            let sub_sample = textureLoad(sub_comp_tex, pixel, 0);
            if sub_sample.a > 0.5 {
                second_layer_real = true;
                r2 = i32(u32(sub_sample.r * 255.0 + 0.5) >> 3u);
                g2 = i32(u32(sub_sample.g * 255.0 + 0.5) >> 3u);
                b2 = i32(u32(sub_sample.b * 255.0 + 0.5) >> 3u);
            } else {
                r2 = i32(p.fixed_color_r);
                g2 = i32(p.fixed_color_g);
                b2 = i32(p.fixed_color_b);
            }
        } else {
            r2 = i32(p.fixed_color_r);
            g2 = i32(p.fixed_color_g);
            b2 = i32(p.fixed_color_b);
        }
        if p.subtract_color != 0u {
            r5 = r5 - r2;
            g5 = g5 - g2;
            b5 = b5 - b2;
        } else {
            r5 = r5 + r2;
            g5 = g5 + g2;
            b5 = b5 + b2;
        }
        if p.half_color != 0u {
            if second_layer_real || p.add_subscreen == 0u {
                r5 = r5 >> 1u;
                g5 = g5 >> 1u;
                b5 = b5 >> 1u;
            }
        }
        r5 = clamp(r5, 0, 31);
        g5 = clamp(g5, 0, 31);
        b5 = clamp(b5, 0, 31);
    }

    let r = apply_brightness(u32(r5), p.brightness);
    let g = apply_brightness(u32(g5), p.brightness);
    let b = apply_brightness(u32(b5), p.brightness);
    return vec4<f32>(r, g, b, 1.0);
}
