struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2<f32>(f32((in_vertex_index << 1) & 2), f32(in_vertex_index & 2));
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var canvas_sampler: sampler;

@group(1) @binding(0) var canvas_texture: texture_2d<f32>;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    return textureSample(canvas_texture, canvas_sampler, in.uv) * vec4<f32>(in.uv, 1.0, 1.0);
}
