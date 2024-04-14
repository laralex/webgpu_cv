struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct DemoSettings {
    mouse_position: vec2<f32>,
    aspect_ratio: f32,
    is_debug: f32,
}
@group(0) @binding(0) var<uniform> demo: DemoSettings;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = vec2(in.uv.x * demo.aspect_ratio, in.uv.y);
    var shade = vec4(uv.x, uv.y, 0.0, 1.0);
    // var mouse_position = vec2(demo.mouse_position.x * demo.aspect_ratio, demo.mouse_position.y);
    // shade += demo.is_debug * step(length(uv - mouse_position), 0.01) * vec4(1.0, 0.0, 0.0, 0.0);
    return shade;
}