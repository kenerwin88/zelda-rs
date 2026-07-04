@group(0) @binding(0) var<storage, read_write> main_screen: array<u32>;
@group(0) @binding(1) var<storage, read_write> sub_screen: array<u32>;
@group(0) @binding(2) var<storage, read> data: array<u32>;

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
@group(0) @binding(3) var<uniform> params: Params;

fn pack_pixel(c5: vec3<u32>, bit: u32, real: bool) -> u32 {
    return (c5.x & 31u)
        | ((c5.y & 31u) << 5u)
        | ((c5.z & 31u) << 10u)
        | ((bit & 7u) << 15u)
        | (select(0u, 1u, real) << 18u);
}

fn cgram_c5(index: u32) -> vec3<u32> {
    let rgba = data[params.p5.w + (index & 255u)];
    return vec3<u32>((rgba & 0xffu) >> 3u, ((rgba >> 8u) & 0xffu) >> 3u, ((rgba >> 16u) & 0xffu) >> 3u);
}

fn wrap_i32(value: i32, period: i32) -> i32 {
    let r = value % period;
    return select(r + period, r, r >= 0);
}

fn mosaic_active() -> bool {
    return params.p7.w > 1u && (params.p7.z & 0x07u) != 0u;
}

fn layer_mosaic_enabled(layer: u32) -> bool {
    return params.p7.w > 1u && ((params.p7.z & (1u << layer)) != 0u);
}

fn mosaic_snap(value: u32) -> u32 {
    let size = params.p7.w;
    return value - (value % size);
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

fn layer_window_masks(layer: u32, sx: u32, sy: u32, is_main: bool) -> bool {
    let windowed = select(params.p7.y, params.p7.x, is_main);
    if ((windowed & (1u << layer)) == 0u) {
        return false;
    }
    let window_flags = (params.p6.w >> (layer * 4u)) & 0x0fu;
    let w1_enabled = (window_flags & 0x2u) != 0u;
    let w2_enabled = (window_flags & 0x8u) != 0u;
    if (!w1_enabled && !w2_enabled) {
        return false;
    }

    let window_base = params.p6.z + sy * 4u;
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

fn bg_instance_pixel(inst_base: u32, layer: u32, sx: u32, sy: u32, hi_priority: bool) -> u32 {
    let cell_id = data[inst_base + 0u];
    let inst_x = bitcast<i32>(data[inst_base + 1u]);
    let inst_y = bitcast<i32>(data[inst_base + 2u]);
    let palette = data[inst_base + 3u];
    let priority = data[inst_base + 4u] != 0u;
    let inst_layer = data[inst_base + 5u];
    if (inst_layer != layer || priority != hi_priority || cell_id >= params.p0.y) {
        return 0xffffffffu;
    }

    let lp = layer_param(layer);
    let scroll_mask = params.p1.w;
    var local_x: i32;
    var local_y: i32;
    if (!mosaic_active() && ((scroll_mask >> layer) & 1u) != 0u) {
        let base_h = i32(lp.x);
        let base_v = i32(lp.y);
        let bg_w = i32(max(lp.z, 256u));
        let bg_h = i32(max(lp.w, 224u));
        let off_x = bg_w - 256;
        let off_y = bg_h - 224;
        let scroll_base = params.p6.x + (sy * 8u) + (layer * 2u);
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
        return 0xffffffffu;
    }

    let cell_offset = cell_id * 64u + u32(local_y) * 8u + u32(local_x);
    let index = data[params.p5.x + cell_offset];
    if (index == 0u) {
        return 0xffffffffu;
    }
    let color = cgram_c5(palette * 16u + index);
    return pack_pixel(color, layer, true);
}

fn bg_pixel(layer: u32, sx: u32, sy: u32, hi_priority: bool, is_main: bool) -> u32 {
    if (layer >= 3u) {
        return 0xffffffffu;
    }
    if (is_main && ((data[params.p6.y + sy] & (1u << layer)) == 0u)) {
        return 0xffffffffu;
    }
    if (layer_window_masks(layer, sx, sy, is_main)) {
        return 0xffffffffu;
    }
    var sample_sx = sx;
    var sample_sy = sy;
    if (layer_mosaic_enabled(layer)) {
        sample_sx = mosaic_snap(sx);
        sample_sy = mosaic_snap(sy);
    }
    var out = 0xffffffffu;
    let count = params.p0.z;
    for (var i = 0u; i < count; i = i + 1u) {
        let px = bg_instance_pixel(params.p5.y + i * 8u, layer, sample_sx, sample_sy, hi_priority);
        if (px != 0xffffffffu) {
            out = px;
        }
    }
    return out;
}

fn sprite_instance_pixel(inst_base: u32, sx: u32, sy: u32, prio: u32) -> u32 {
    let cell_id = data[inst_base + 0u];
    let inst_x = bitcast<i32>(data[inst_base + 1u]);
    let inst_y = bitcast<i32>(data[inst_base + 2u]);
    let palette = data[inst_base + 3u];
    let inst_prio = data[inst_base + 4u];
    let flags = data[inst_base + 5u];
    let row_mask = data[inst_base + 6u];
    if (inst_prio != prio) {
        return 0xffffffffu;
    }
    let local_x = i32(sx) - inst_x;
    let local_y = i32(sy) - inst_y;
    if (local_x < 0 || local_y < 0 || local_x >= 8 || local_y >= 8) {
        return 0xffffffffu;
    }
    if ((row_mask & (1u << u32(local_y))) == 0u) {
        return 0xffffffffu;
    }
    let src_x = select(u32(local_x), 7u - u32(local_x), (flags & 1u) != 0u);
    let src_y = select(u32(local_y), 7u - u32(local_y), (flags & 2u) != 0u);
    let index = data[params.p5.x + (params.p0.y + cell_id) * 64u + src_y * 8u + src_x];
    if (index == 0u) {
        return 0xffffffffu;
    }
    let color = cgram_c5(128u + palette * 16u + index);
    let bit = select(4u, 6u, palette < 4u);
    return pack_pixel(color, bit, true);
}

fn obj_pixel(sx: u32, sy: u32, prio: u32, is_main: bool) -> u32 {
    if (is_main && ((data[params.p6.y + sy] & 0x10u) == 0u)) {
        return 0xffffffffu;
    }
    if (layer_window_masks(4u, sx, sy, is_main)) {
        return 0xffffffffu;
    }
    let count = params.p0.w;
    for (var i = 0u; i < count; i = i + 1u) {
        let px = sprite_instance_pixel(params.p5.z + i * 8u, sx, sy, prio);
        if (px != 0xffffffffu) {
            return px;
        }
    }
    return 0xffffffffu;
}

fn maybe_paint_bg(current: u32, enabled: u32, layer: u32, sx: u32, sy: u32, hi: bool, is_main: bool) -> u32 {
    if ((enabled & (1u << layer)) == 0u) {
        return current;
    }
    let px = bg_pixel(layer, sx, sy, hi, is_main);
    return select(current, px, px != 0xffffffffu);
}

fn maybe_paint_obj(current: u32, enabled: u32, prio: u32, sx: u32, sy: u32, is_main: bool) -> u32 {
    if ((enabled & 0x10u) == 0u) {
        return current;
    }
    let px = obj_pixel(sx, sy, prio, is_main);
    return select(current, px, px != 0xffffffffu);
}

fn composite_screen(sx: u32, sy: u32, enabled: u32, is_main: bool, backdrop: u32) -> u32 {
    var out = backdrop;
    out = maybe_paint_bg(out, enabled, 2u, sx, sy, false, is_main);
    out = maybe_paint_obj(out, enabled, 0u, sx, sy, is_main);
    out = maybe_paint_obj(out, enabled, 1u, sx, sy, is_main);
    out = maybe_paint_bg(out, enabled, 1u, sx, sy, false, is_main);
    out = maybe_paint_bg(out, enabled, 0u, sx, sy, false, is_main);
    out = maybe_paint_obj(out, enabled, 2u, sx, sy, is_main);
    out = maybe_paint_bg(out, enabled, 1u, sx, sy, true, is_main);
    out = maybe_paint_bg(out, enabled, 0u, sx, sy, true, is_main);
    out = maybe_paint_obj(out, enabled, 3u, sx, sy, is_main);
    out = maybe_paint_bg(out, enabled, 2u, sx, sy, true, is_main);
    return out;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.p0.x) {
        return;
    }
    let sx = i % 256u;
    let sy = i / 256u;
    let backdrop = params.p1.x;
    main_screen[i] = composite_screen(sx, sy, params.p1.y, true, backdrop);
    sub_screen[i] = composite_screen(sx, sy, params.p1.z, false, backdrop);
}
