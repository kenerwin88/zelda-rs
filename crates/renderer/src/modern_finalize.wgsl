@group(0) @binding(0) var<storage, read> main_screen: array<u32>;
@group(0) @binding(1) var<storage, read> sub_screen: array<u32>;
@group(0) @binding(2) var<storage, read> windows: array<u32>;
struct Params {
    p0: vec4<u32>,
    p1: vec4<u32>,
    p2: vec4<u32>,
};
@group(0) @binding(3) var<uniform> params: Params;
@group(0) @binding(4) var<storage, read_write> out_rgba: array<u32>;

fn unpack_r(p: u32) -> i32 {
    return i32(p & 31u);
}

fn unpack_g(p: u32) -> i32 {
    return i32((p >> 5u) & 31u);
}

fn unpack_b(p: u32) -> i32 {
    return i32((p >> 10u) & 31u);
}

fn unpack_bit(p: u32) -> u32 {
    return (p >> 15u) & 7u;
}

fn unpack_real(p: u32) -> bool {
    return ((p >> 18u) & 1u) != 0u;
}

fn cw_bit(in_window: bool, mode: u32) -> bool {
    let masks = array<u32, 8>(0x00u, 0xffu, 0xffu, 0x00u, 0xffu, 0x00u, 0xffu, 0x00u);
    let w = select(0u, 0xffu, in_window);
    let m = mode & 7u;
    return ((w & masks[m]) ^ masks[m + 4u]) != 0u;
}

fn in_cm_window(sx: u32, packed_win: u32, windowsel_cm: u32) -> bool {
    let w1l = packed_win & 0xffu;
    let w1r = (packed_win >> 8u) & 0xffu;
    let w2l = (packed_win >> 16u) & 0xffu;
    let w2r = (packed_win >> 24u) & 0xffu;
    var inside = false;
    if ((windowsel_cm & 0x2u) != 0u) {
        var in_w1 = w1l <= w1r && sx >= w1l && sx <= w1r;
        if ((windowsel_cm & 0x1u) != 0u) {
            in_w1 = !in_w1;
        }
        inside = inside || in_w1;
    }
    if ((windowsel_cm & 0x8u) != 0u) {
        var in_w2 = w2l <= w2r && sx >= w2l && sx <= w2r;
        if ((windowsel_cm & 0x4u) != 0u) {
            in_w2 = !in_w2;
        }
        inside = inside || in_w2;
    }
    return inside;
}

fn expand_brightness(c5: i32, brightness: u32) -> u32 {
    let clamped = u32(clamp(c5, 0, 31));
    let scaled5 = (clamped * min(brightness, 15u) + 7u) / 15u;
    return (scaled5 << 3u) | (scaled5 >> 2u);
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let len = params.p0.x;
    if (i >= len) {
        return;
    }

    let width = params.p0.y;
    let scale = params.p0.z;
    let brightness = params.p0.w;
    let math_enabled = params.p1.x;
    let flags = params.p1.y;
    let fixed_packed = params.p1.z;
    let clip_mode = params.p1.w;
    let prevent_math_mode = params.p2.x;
    let windowsel_cm = params.p2.y;
    let forced_blank_scanlines = params.p2.z;

    if ((flags & 0x10u) != 0u) {
        out_rgba[i] = 0xff000000u;
        return;
    }

    let main = main_screen[i];
    let sub = sub_screen[i];
    var r = unpack_r(main);
    var g = unpack_g(main);
    var b = unpack_b(main);

    let out_x = i % width;
    let out_y = i / width;
    let nrow = out_y / scale;
    let ncol = out_x / scale;

    if (nrow < forced_blank_scanlines) {
        out_rgba[i] = 0xff000000u;
        return;
    }

    let layer_math_on = ((math_enabled >> unpack_bit(main)) & 1u) != 0u;
    let cm_window = in_cm_window(ncol, windows[nrow], windowsel_cm);
    let not_clipped = cw_bit(cm_window, clip_mode);
    let math_window_ok = cw_bit(cm_window, prevent_math_mode);
    let no_effect_math = (flags & 0x8u) != 0u;
    let do_math = !no_effect_math && layer_math_on && math_window_ok;

    if (!not_clipped) {
        out_rgba[i] = 0xff000000u;
        return;
    }

    if (do_math) {
        let add_subscreen = (flags & 0x4u) != 0u;
        let sub_real = unpack_real(sub);
        var or = i32(fixed_packed & 0xffu);
        var og = i32((fixed_packed >> 8u) & 0xffu);
        var ob = i32((fixed_packed >> 16u) & 0xffu);
        var second_real = false;
        if (add_subscreen && sub_real) {
            or = unpack_r(sub);
            og = unpack_g(sub);
            ob = unpack_b(sub);
            second_real = true;
        }

        if ((flags & 0x1u) != 0u) {
            r = r - or;
            g = g - og;
            b = b - ob;
        } else {
            r = r + or;
            g = g + og;
            b = b + ob;
        }

        let half_color = (flags & 0x2u) != 0u;
        if (half_color && (second_real || !add_subscreen)) {
            r = r >> 1;
            g = g >> 1;
            b = b >> 1;
        }
    }

    let rr = expand_brightness(r, brightness);
    let gg = expand_brightness(g, brightness);
    let bb = expand_brightness(b, brightness);
    out_rgba[i] = rr | (gg << 8u) | (bb << 16u) | 0xff000000u;
}
