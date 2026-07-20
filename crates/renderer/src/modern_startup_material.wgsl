struct Params {
    p0: vec4<u32>,
    p1: vec4<u32>,
    p2: vec4<u32>,
    p3: vec4<u32>,
};

@group(0) @binding(0) var<storage, read_write> out_rgba: array<u32>;
@group(0) @binding(1) var<storage, read> direct_pixels: array<vec4<u32>, 512>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let material_index = gid.x;
    let count = params.p3.y;
    if (material_index >= count) {
        return;
    }
    let pixel = direct_pixels[material_index];
    if ((pixel.z >> 24u) == 0u || pixel.x >= params.p0.y || pixel.y >= params.p0.x / params.p0.y) {
        return;
    }
    out_rgba[pixel.y * params.p0.y + pixel.x] = pixel.z;
}
