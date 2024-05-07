struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in_vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = in_vertex.uv;
    out.clip_position = vec4(in_vertex.position, 1.0);
    return out;
}