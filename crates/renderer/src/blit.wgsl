// Fullscreen triangle blit: samples the game texture and outputs it scaled/centered.
//
// Three vertices cover the full NDC space without a vertex buffer. The rasterizer
// clips the oversized triangle to [-1,1] x [-1,1]; set_viewport then constrains
// output to the letterbox rect before this shader runs.

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOut {
    // vi=0 -> (-1, 1, uv 0,0)  top-left
    // vi=1 -> ( 3, 1, uv 2,0)  far top-right (clipped)
    // vi=2 -> (-1,-3, uv 0,2)  far bottom-left (clipped)
    let x = f32(vi & 1u) * 4.0 - 1.0;
    let y = 1.0 - f32((vi >> 1u) & 1u) * 4.0;
    var out: VertexOut;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv  = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@group(0) @binding(0) var t_game: texture_2d<f32>;
@group(0) @binding(1) var s_nearest: sampler;

struct PresentationParams {
    presentation: u32,
    lighting: u32,
    shadows: u32,
    light_count: u32,
    scene_flags: u32,
    notice_code: u32,
    notice_frames: u32,
    _pad2: u32,
    lights: array<vec4<f32>, 8>,
    occluders: array<vec4<u32>, 2>,
    light_mask: array<vec4<f32>, 56>,
}

@group(0) @binding(2) var<uniform> params: PresentationParams;

fn sample_nearest(uv: vec2<f32>) -> vec3<f32> {
    let dims = textureDimensions(t_game);
    let texel = clamp(vec2i(vec2<f32>(dims) * uv), vec2i(0, 0), vec2i(dims) - vec2i(1, 1));
    return textureLoad(t_game, texel, 0).rgb;
}

fn bright_pass(color: vec3<f32>) -> vec3<f32> {
    return max(color - vec3<f32>(0.62), vec3<f32>(0.0));
}

fn apply_bloom_color_grade(color: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let dims = textureDimensions(t_game);
    let texel_size = 1.0 / vec2<f32>(dims);
    var bloom = bright_pass(color) * 0.20;
    bloom += bright_pass(textureSample(t_game, s_nearest, uv + vec2<f32>( texel_size.x, 0.0)).rgb) * 0.08;
    bloom += bright_pass(textureSample(t_game, s_nearest, uv + vec2<f32>(-texel_size.x, 0.0)).rgb) * 0.08;
    bloom += bright_pass(textureSample(t_game, s_nearest, uv + vec2<f32>(0.0,  texel_size.y)).rgb) * 0.08;
    bloom += bright_pass(textureSample(t_game, s_nearest, uv + vec2<f32>(0.0, -texel_size.y)).rgb) * 0.08;

    var graded = color + bloom;
    graded = clamp((graded - 0.5) * 1.04 + 0.5, vec3<f32>(0.0), vec3<f32>(1.0));
    let luma = dot(graded, vec3<f32>(0.2126, 0.7152, 0.0722));
    graded = mix(vec3<f32>(luma), graded, 1.08);
    graded *= vec3<f32>(1.03, 1.01, 0.96);
    return clamp(graded, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn apply_crt(color: vec3<f32>, y: f32) -> vec3<f32> {
    let scanline = select(0.78, 1.0, (u32(y) & 1u) == 0u);
    let mask_phase = u32(y) % 3u;
    let mask = select(
        vec3<f32>(0.95, 1.0, 0.95),
        vec3<f32>(1.0, 0.95, 0.95),
        mask_phase == 0u,
    );
    return color * scanline * mask;
}

fn apply_lighting(color: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    if params.lighting == 0u {
        return color;
    }
    let in_dungeon = (params.scene_flags & 1u) != 0u;
    let base_dynamic = select(0.82, 0.70, in_dungeon);
    let ambient = select(0.88, base_dynamic, params.lighting == 2u);
    if params.lighting == 2u {
        let lift = sample_light_mask(uv);
        let warm_lift = vec3<f32>(1.0, 0.86, 0.58) * lift * 0.18;
        return color * (ambient + lift * 0.20) + warm_lift;
    }
    return color * ambient;
}

fn light_mask_cell(cell: vec2u) -> f32 {
    let clamped = min(cell, vec2u(15u, 13u));
    let idx = clamped.y * 16u + clamped.x;
    let lane = idx % 4u;
    return params.light_mask[idx / 4u][lane];
}

fn sample_light_mask(uv: vec2<f32>) -> f32 {
    if uv.x < 0.0 || uv.x >= 1.0 || uv.y < 0.0 || uv.y >= 1.0 {
        return 0.0;
    }
    let grid = uv * vec2<f32>(16.0, 14.0) - vec2<f32>(0.5);
    let base = vec2u(max(vec2<f32>(0.0), floor(grid)));
    let frac = clamp(fract(grid), vec2<f32>(0.0), vec2<f32>(1.0));
    let a = light_mask_cell(base);
    let b = light_mask_cell(base + vec2u(1u, 0u));
    let c = light_mask_cell(base + vec2u(0u, 1u));
    let d = light_mask_cell(base + vec2u(1u, 1u));
    return mix(mix(a, b, frac.x), mix(c, d, frac.x), frac.y);
}

fn occluder_at(uv: vec2<f32>) -> bool {
    if uv.x < 0.0 || uv.x >= 1.0 || uv.y < 0.0 || uv.y >= 1.0 {
        return false;
    }
    let cell = vec2u(min(vec2<f32>(15.0, 13.0), floor(uv * vec2<f32>(16.0, 14.0))));
    let bit = cell.y * 16u + cell.x;
    let word = bit / 32u;
    let lane = word % 4u;
    let packed = params.occluders[word / 4u][lane];
    return ((packed >> (bit % 32u)) & 1u) != 0u;
}

fn ray_shadow(light: vec4<f32>, uv: vec2<f32>) -> f32 {
    let to_pixel = uv - light.xy;
    let dist_to_pixel = length(to_pixel);
    if dist_to_pixel <= 0.001 || dist_to_pixel > light.z * 1.35 {
        return 0.0;
    }
    let dir = to_pixel / dist_to_pixel;
    var blocked = 0.0;
    for (var step = 1u; step <= 6u; step = step + 1u) {
        let t = (f32(step) / 7.0) * dist_to_pixel;
        let sample_uv = light.xy + dir * t;
        if occluder_at(sample_uv) {
            blocked = 1.0;
            break;
        }
    }
    let shadow_tail = smoothstep(light.z * 1.35, light.z * 0.25, dist_to_pixel);
    return blocked * shadow_tail;
}

fn soft_ray_shadow(light: vec4<f32>, uv: vec2<f32>) -> f32 {
    let to_pixel = uv - light.xy;
    let dist_to_pixel = max(length(to_pixel), 0.001);
    let dir = to_pixel / dist_to_pixel;
    let normal = vec2<f32>(-dir.y, dir.x);
    var shadow = ray_shadow(light, uv) * 0.50;
    shadow += ray_shadow(light, uv + normal * 0.004) * 0.20;
    shadow += ray_shadow(light, uv - normal * 0.004) * 0.20;
    shadow += ray_shadow(light, uv + normal * 0.010) * 0.05;
    shadow += ray_shadow(light, uv - normal * 0.010) * 0.05;
    return shadow;
}

fn apply_shadows(color: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    if params.shadows == 0u {
        return color;
    }
    let vignette = smoothstep(0.28, 0.80, distance(uv, vec2<f32>(0.5, 0.5)));
    let strength = select(0.22, 0.34, params.shadows == 2u);
    var ray_shadow_amount = 0.0;
    if params.shadows == 2u && params.lighting == 2u {
        for (var i = 0u; i < 8u; i = i + 1u) {
            if i >= params.light_count {
                break;
            }
            ray_shadow_amount = max(ray_shadow_amount, soft_ray_shadow(params.lights[i], uv));
        }
    }
    let ray_strength = ray_shadow_amount * 0.48;
    return color * clamp(1.0 - vignette * strength - ray_strength, 0.42, 1.0);
}

fn notice_label_len() -> u32 {
    switch params.notice_code {
        case 2u: { return 7u; }
        case 11u, 12u, 22u: { return 9u; }
        case 21u: { return 6u; }
        case 1u, 3u, 10u, 20u: { return 5u; }
        default: { return 0u; }
    }
}

fn notice_char_code(index: u32) -> u32 {
    if params.notice_code == 1u {
        return array<u32, 9>(80u, 58u, 79u, 70u, 70u, 0u, 0u, 0u, 0u)[index];
    }
    if params.notice_code == 2u {
        return array<u32, 9>(80u, 58u, 83u, 72u, 65u, 82u, 80u, 0u, 0u)[index];
    }
    if params.notice_code == 3u {
        return array<u32, 9>(80u, 58u, 67u, 82u, 84u, 0u, 0u, 0u, 0u)[index];
    }
    if params.notice_code == 10u {
        return array<u32, 9>(76u, 58u, 79u, 70u, 70u, 0u, 0u, 0u, 0u)[index];
    }
    if params.notice_code == 11u {
        return array<u32, 9>(76u, 58u, 65u, 77u, 66u, 73u, 69u, 78u, 84u)[index];
    }
    if params.notice_code == 12u {
        return array<u32, 9>(76u, 58u, 68u, 89u, 78u, 65u, 77u, 73u, 67u)[index];
    }
    if params.notice_code == 20u {
        return array<u32, 9>(83u, 58u, 79u, 70u, 70u, 0u, 0u, 0u, 0u)[index];
    }
    if params.notice_code == 21u {
        return array<u32, 9>(83u, 58u, 83u, 79u, 70u, 84u, 0u, 0u, 0u)[index];
    }
    if params.notice_code == 22u {
        return array<u32, 9>(83u, 58u, 82u, 65u, 89u, 67u, 65u, 83u, 84u)[index];
    }
    return 0u;
}

fn glyph_row_bits(code: u32, row: u32) -> u32 {
    var rows = array<u32, 7>(0u, 0u, 0u, 0u, 0u, 0u, 0u);
    switch code {
        case 58u: { rows = array<u32, 7>(0u, 4u, 4u, 0u, 4u, 4u, 0u); }
        case 65u: { rows = array<u32, 7>(14u, 17u, 17u, 31u, 17u, 17u, 17u); }
        case 66u: { rows = array<u32, 7>(30u, 17u, 17u, 30u, 17u, 17u, 30u); }
        case 67u: { rows = array<u32, 7>(15u, 16u, 16u, 16u, 16u, 16u, 15u); }
        case 68u: { rows = array<u32, 7>(30u, 17u, 17u, 17u, 17u, 17u, 30u); }
        case 69u: { rows = array<u32, 7>(31u, 16u, 16u, 30u, 16u, 16u, 31u); }
        case 70u: { rows = array<u32, 7>(31u, 16u, 16u, 30u, 16u, 16u, 16u); }
        case 72u: { rows = array<u32, 7>(17u, 17u, 17u, 31u, 17u, 17u, 17u); }
        case 73u: { rows = array<u32, 7>(14u, 4u, 4u, 4u, 4u, 4u, 14u); }
        case 76u: { rows = array<u32, 7>(16u, 16u, 16u, 16u, 16u, 16u, 31u); }
        case 77u: { rows = array<u32, 7>(17u, 27u, 21u, 21u, 17u, 17u, 17u); }
        case 78u: { rows = array<u32, 7>(17u, 25u, 21u, 19u, 17u, 17u, 17u); }
        case 79u: { rows = array<u32, 7>(14u, 17u, 17u, 17u, 17u, 17u, 14u); }
        case 80u: { rows = array<u32, 7>(30u, 17u, 17u, 30u, 16u, 16u, 16u); }
        case 82u: { rows = array<u32, 7>(30u, 17u, 17u, 30u, 20u, 18u, 17u); }
        case 83u: { rows = array<u32, 7>(15u, 16u, 16u, 14u, 1u, 1u, 30u); }
        case 84u: { rows = array<u32, 7>(31u, 4u, 4u, 4u, 4u, 4u, 4u); }
        case 89u: { rows = array<u32, 7>(17u, 17u, 10u, 4u, 4u, 4u, 4u); }
        default: {}
    }
    return rows[row];
}

fn notice_glyph_on(local: vec2u) -> bool {
    let char_index = local.x / 6u;
    let glyph_x = local.x % 6u;
    if glyph_x >= 5u || local.y >= 7u || char_index >= notice_label_len() {
        return false;
    }
    let code = notice_char_code(char_index);
    let row_bits = glyph_row_bits(code, local.y);
    return ((row_bits >> (4u - glyph_x)) & 1u) != 0u;
}

fn apply_notice_overlay(color: vec3<f32>, pos: vec2<f32>) -> vec3<f32> {
    if params.notice_code == 0u || params.notice_frames == 0u {
        return color;
    }
    let scale = 2.0;
    let origin = vec2<f32>(8.0, 8.0);
    let label_len = f32(notice_label_len());
    let panel_size = vec2<f32>(label_len * 12.0 + 12.0, 24.0);
    let panel_pos = pos - origin;
    if panel_pos.x < 0.0 || panel_pos.y < 0.0 || panel_pos.x >= panel_size.x || panel_pos.y >= panel_size.y {
        return color;
    }
    let fade = clamp(f32(params.notice_frames) / 12.0, 0.0, 1.0);
    var out_color = mix(color, vec3<f32>(0.02, 0.025, 0.03), 0.72 * fade);
    let text_local = vec2u(max(vec2<f32>(0.0), floor((panel_pos - vec2<f32>(6.0, 5.0)) / scale)));
    if notice_glyph_on(text_local) {
        out_color = mix(out_color, vec3<f32>(0.95, 0.92, 0.78), fade);
    }
    return out_color;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var color: vec3<f32>;
    if params.presentation == 1u {
        color = textureSample(t_game, s_nearest, in.uv).rgb;
        color = clamp((color - 0.5) * 1.08 + 0.5, vec3<f32>(0.0), vec3<f32>(1.0));
    } else if params.presentation == 2u {
        color = apply_crt(sample_nearest(in.uv), in.pos.y);
    } else {
        color = textureSample(t_game, s_nearest, in.uv).rgb;
    }
    if params.presentation != 0u {
        color = apply_bloom_color_grade(color, in.uv);
    }
    color = apply_lighting(color, in.uv);
    color = apply_shadows(color, in.uv);
    color = apply_notice_overlay(color, in.pos.xy);
    // Force alpha=1.0: the CPU PPU BGRA buffer has A=0, which would produce
    // fully transparent PNG output if the alpha channel is passed through.
    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
