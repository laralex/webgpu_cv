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

struct FractalSettings {
    center: vec2<f32>,
    zoom: f32,
    num_iterations: i32,
    color_bias: vec3<f32>,
    color_power: f32,
}
@group(1) @binding(0) var<uniform> fractal: FractalSettings;

const AA : i32 = 2;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = 2.0 * in.uv - 1.0;
    uv = vec2(uv.x * demo.aspect_ratio, uv.y);
    var delta_center = fractal.zoom * uv;
    var center = fractal.center + delta_center;

#ifdef USE_ANTIALIASING
    var AA_norm = fractal.zoom / vec2<f32>(demo.color_attachment_size);
    var shade = vec3(0.0);
    for (var re = 0; re < AA; re++) {
        for (var im = 0; im < AA; im++) {
            var diverge_iteration = mandelbrot_diverge_iteration(center + vec2(f32(re), f32(im)) * AA_norm, fractal.num_iterations);
            shade += step(1e-6, diverge_iteration) * (0.5 + 0.5*cos(pow(fractal.zoom, fractal.color_power)*diverge_iteration*0.08 + fractal.color_bias));
        }
    }
    shade /= f32(AA*AA);
#else
    var diverge_iteration = mandelbrot_diverge_iteration(center, fractal.num_iterations);
    var shade = step(1e-6, diverge_iteration) * (0.5 + 0.5*cos(pow(fractal.zoom, fractal.color_power)*diverge_iteration*0.08 + fractal.color_bias));
#endif
    // red cursor overlay
    shade = mix(shade, vec3(1.0, 0.0, 0.0), step(length(in.uv - demo_dyn.mouse_position), 0.01) * step(0.5, demo.is_debug));
    return vec4<f32>(shade, 1.0);
}

fn mandelbrot_diverge_iteration(center: vec2<f32>, num_iterations: i32) -> f32 {
    var z = vec2<f32>(0.0);
    var diverge_iteration = -1.0;
    for (var i = 0; i < num_iterations; i++) {
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
