struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct DemoSettingsStable {
    color_attachment_size: vec2<i32>,
    aspect_ratio: f32,
    is_debug: f32,
}

struct DemoSettingsDynamic {
    mouse_position: vec2<f32>,
    padding__: vec2<i32>,
}

@group(0) @binding(0) var<uniform> demo: DemoSettingsStable;
@group(0) @binding(1) var<uniform> demo_dyn: DemoSettingsDynamic;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = 1.0 * (2.0 * in.uv - 1.0);
    uv = vec2(uv.x * demo.aspect_ratio, uv.y);
    uv = uv % 1.0;
    var shade = vec4(uv, 0.0, 1.0);
    // var mouse_position = vec2(demo.mouse_position.x * demo.aspect_ratio, demo.mouse_position.y);
    // shade += demo.is_debug * step(length(uv - mouse_position), 0.01) * vec4(1.0, 0.0, 0.0, 0.0);
    return shade;
}