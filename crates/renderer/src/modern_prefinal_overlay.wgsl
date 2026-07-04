@group(0) @binding(0) var<storage, read_write> main_screen: array<u32>;
@group(0) @binding(1) var<storage, read> data: array<u32>;

struct Params {
    p0: vec4<u32>,
    p1: vec4<u32>,
};
@group(0) @binding(2) var<uniform> params: Params;

fn transparent() -> u32 {
    return 0xffffffffu;
}

fn packet_pixel(cell_offset: u32, local_x: u32, local_y: u32) -> u32 {
    return data[cell_offset + local_y * 8u + local_x];
}

fn overlay_bg_pixel(sx: u32, sy: u32) -> vec2<u32> {
    var px = transparent();
    var rank = 255u;
    for (var i = 0u; i < params.p0.y; i = i + 1u) {
        let base = params.p1.z + i * 4u;
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
        let base = params.p1.w + i * 4u;
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
