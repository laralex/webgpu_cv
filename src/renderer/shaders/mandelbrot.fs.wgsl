struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

const NUM_ITERS = 500;

struct Settings {
    center: vec2<f32>,
    zoom: f32,
    aspect_ratio: f32,
}

@group(0) @binding(0) var<uniform> settings: Settings;
// @layout(push_constant) var<uniform> settings: Settings;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = vec2(in.uv.x * settings.aspect_ratio, in.uv.y);
    var offset = settings.zoom * (2.0 * uv - 1.0);
    var diverge_iteration = mandelbrot_diverge_iteration(settings.center + offset);
    var zoom_compensation = pow(settings.zoom,0.22);
    var shade = step(-diverge_iteration, -1e-6) * (0.5 + 0.5*cos(diverge_iteration*0.08 + vec3(3.5,4.0,4.5)));
    return vec4<f32>(shade, 1.0);
}

fn mandelbrot_diverge_iteration(center: vec2<f32>) -> f32 {
    var z = vec2<f32>(0.0);
    var diverge_iteration = -1.0;
    for (var i = 0; i < NUM_ITERS; i++) {
        z = cmul(z,z) + center;
        if (dot(z, z) > 4.0) {
            diverge_iteration = f32(i);
            break;
        }
    }
    return diverge_iteration;
}

fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x*b.x-a.y*b.y, a.x*b.y+a.y*b.x);
}
