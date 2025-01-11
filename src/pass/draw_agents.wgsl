struct Params {
    scale: f32,
};

struct VertexOutput {
    @builtin(position) screen_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    @location(0) agent_pos: vec2<f32>,
    @location(1) agent_heading: f32,
    @location(2) vertex_pos: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let cs_heading = vec2<f32>(cos(agent_heading), sin(agent_heading));
    let vertex_position = vertex_pos * params.scale;
    let rotated_vertex_pos = vec2<f32>(
        vertex_position.x * cs_heading.x - vertex_position.y * cs_heading.y,
        vertex_position.x * cs_heading.y + vertex_position.y * cs_heading.x,
    );
    out.uv = (agent_pos + rotated_vertex_pos) * 0.5 + 0.5;
    out.screen_position = vec4<f32>(agent_pos + rotated_vertex_pos, 0, 1);
    return out;
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    return vec4<f32>(in.uv, 1, 1);
}
