struct Params {
    decay_rate: f32,
    diffuse_radius: u32,
};

@group(0) @binding(0) var<uniform> params: Params;

@group(1) @binding(0) var canvas_in : texture_2d<f32>;
@group(1) @binding(1) var canvas_out : texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>
) {
    let position = vec2<i32>(global_invocation_id.xy);
    let dimensions = vec2<i32>(textureDimensions(canvas_in));
    let br = i32(params.diffuse_radius);

    if (position.x >= dimensions.x || position.y >= dimensions.y) { return; }

    let kernel_width = 2 * br + 1;
    let kernel_elem_count = kernel_width * kernel_width;
    let kernel_elem_weight = 1.0 / f32(kernel_elem_count);

    var value = vec4<f32>(0);
    for (var x: i32 = position.x - br; x <= position.x + br; x += 1) {
        for (var y: i32 = position.y - br; y <= position.y + br; y += 1) {
            value += kernel_elem_weight * textureLoad(canvas_in, vec2<i32>(x, y), 0);
        }
    }
    value = max(vec4<f32>(0), value - params.decay_rate);

    textureStore(canvas_out, position, value);
}
