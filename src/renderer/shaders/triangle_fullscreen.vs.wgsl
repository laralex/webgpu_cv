struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2(f32((in_vertex_index << 1) & 2), f32(in_vertex_index & 2));
    out.clip_position = vec4(out.uv * 2.0 + -1.0, 0.0, 1.0f);
    return out;
}