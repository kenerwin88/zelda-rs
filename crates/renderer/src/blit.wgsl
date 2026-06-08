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

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // Force alpha=1.0: the CPU PPU BGRA buffer has A=0, which would produce
    // fully transparent PNG output if the alpha channel is passed through.
    return vec4<f32>(textureSample(t_game, s_nearest, in.uv).rgb, 1.0);
}
