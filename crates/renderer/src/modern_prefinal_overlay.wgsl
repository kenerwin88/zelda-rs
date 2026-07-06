@group(0) @binding(0) var<storage, read_write> main_screen: array<u32>;
@group(0) @binding(1) var<storage, read> data: array<u32>;

struct Params {
    p0: vec4<u32>,
    p1: vec4<u32>,
    p2: vec4<u32>,
    p3: vec4<u32>,
    p4: vec4<u32>,
    p5: vec4<u32>,
    p6: vec4<u32>,
    p7: vec4<u32>,
};
@group(0) @binding(2) var<uniform> params: Params;

fn transparent() -> u32 {
    return 0xffffffffu;
}

fn packet_pixel(cell_offset: u32, local_x: u32, local_y: u32) -> u32 {
    return data[cell_offset + local_y * 8u + local_x];
}

fn wrap_i32(value: i32, period: i32) -> i32 {
    let r = value % period;
    return select(r + period, r, r >= 0);
}

fn layer_param(layer: u32) -> vec4<u32> {
    if (layer == 0u) {
        return params.p2;
    }
    if (layer == 1u) {
        return params.p3;
    }
    return params.p4;
}

fn layer_window_masks(layer: u32, sx: u32, sy: u32) -> bool {
    if ((params.p6.x & (1u << layer)) == 0u) {
        return false;
    }
    let window_flags = (params.p5.z >> (layer * 4u)) & 0x0fu;
    let w1_enabled = (window_flags & 0x2u) != 0u;
    let w2_enabled = (window_flags & 0x8u) != 0u;
    if (!w1_enabled && !w2_enabled) {
        return false;
    }

    let window_base = params.p5.x + sy * 4u;
    let w1l = data[window_base + 0u];
    let w1r = data[window_base + 1u];
    let w2l = data[window_base + 2u];
    let w2r = data[window_base + 3u];
    var test1 = sx >= w1l && sx <= w1r;
    var test2 = sx >= w2l && sx <= w2r;
    if ((window_flags & 0x1u) != 0u) {
        test1 = !test1;
    }
    if ((window_flags & 0x4u) != 0u) {
        test2 = !test2;
    }
    if (w1_enabled && !w2_enabled) {
        return test1;
    }
    if (!w1_enabled && w2_enabled) {
        return test2;
    }
    return test1 || test2;
}

fn bg_packet_pixel(base: u32, sx: u32, sy: u32) -> u32 {
    let layer = data[base + 4u];
    if (layer >= 3u) {
        return transparent();
    }
    let layer_bit = 1u << layer;
    if ((params.p5.w & layer_bit) == 0u) {
        return transparent();
    }
    if ((data[params.p1.w + sy] & layer_bit) == 0u) {
        return transparent();
    }
    if (layer_window_masks(layer, sx, sy)) {
        return transparent();
    }

    let inst_x = bitcast<i32>(data[base + 0u]);
    let inst_y = bitcast<i32>(data[base + 1u]);
    var local_x: i32;
    var local_y: i32;
    if (((params.p5.y >> layer) & 1u) != 0u) {
        let lp = layer_param(layer);
        let base_h = i32(lp.x);
        let base_v = i32(lp.y);
        let bg_w = i32(max(lp.z, 256u));
        let bg_h = i32(max(lp.w, 224u));
        let off_x = bg_w - 256;
        let off_y = bg_h - 224;
        let scroll_base = params.p1.z + (sy * 8u) + (layer * 2u);
        let dh = i32(data[scroll_base]) - base_h;
        let dv = i32(data[scroll_base + 1u]) - base_v;
        let bx = wrap_i32(i32(sx) + dh + off_x, bg_w);
        let by = wrap_i32(i32(sy) + dv + off_y, bg_h);
        let bx0 = inst_x + off_x;
        let by0 = inst_y + off_y;
        local_x = wrap_i32(bx - bx0, bg_w);
        local_y = wrap_i32(by - by0, bg_h);
    } else {
        local_x = i32(sx) - inst_x;
        local_y = i32(sy) - inst_y;
    }
    if (local_x < 0 || local_y < 0 || local_x >= 8 || local_y >= 8) {
        return transparent();
    }
    return packet_pixel(data[base + 3u], u32(local_x), u32(local_y));
}

fn overlay_bg_pixel(sx: u32, sy: u32) -> vec2<u32> {
    var px = transparent();
    var rank = 255u;
    for (var i = 0u; i < params.p0.y; i = i + 1u) {
        let base = params.p1.x + i * 8u;
        let candidate = bg_packet_pixel(base, sx, sy);
        if (candidate == transparent()) {
            continue;
        }
        px = candidate;
        rank = data[base + 2u];
    }
    return vec2<u32>(px, rank);
}

fn overlay_sprite_pixel(current: u32, bg_rank: u32, sx: u32, sy: u32) -> u32 {
    if (bg_rank == 255u) {
        return current;
    }
    var out = current;
    for (var i = 0u; i < params.p0.z; i = i + 1u) {
        let base = params.p1.y + i * 4u;
        let sprite_rank = data[base + 2u];
        if (sprite_rank < bg_rank) {
            continue;
        }
        let inst_x = bitcast<i32>(data[base + 0u]);
        let inst_y = bitcast<i32>(data[base + 1u]);
        let local_x = i32(sx) - inst_x;
        let local_y = i32(sy) - inst_y;
        if (local_x < 0 || local_y < 0 || local_x >= 8 || local_y >= 8) {
            continue;
        }
        let candidate = packet_pixel(data[base + 3u], u32(local_x), u32(local_y));
        if (candidate == transparent()) {
            continue;
        }
        out = candidate;
    }
    return out;
}

fn math_bit(pixel: u32) -> u32 {
    return (pixel >> 15u) & 7u;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.p0.x) {
        return;
    }
    let sx = i % 256u;
    let sy = i / 256u;
    let bg = overlay_bg_pixel(sx, sy);
    var px = main_screen[i];
    if (bg.x != transparent() && math_bit(px) == math_bit(bg.x)) {
        px = bg.x;
    }
    main_screen[i] = overlay_sprite_pixel(px, bg.y, sx, sy);
}
