// GI合成着色器
// 合成多种GI技术

struct CompositeParams {
    ray_tracing_weight: f32,
    ssr_weight: f32,
    ssgi_weight: f32,
    ssdo_weight: f32,
    probe_weight: f32,
};

@group(0) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;
@group(0) @binding(1) var base_color: texture_2d<f32>;
@group(0) @binding(2) var ray_tracing_gi: texture_2d<f32>;
@group(0) @binding(3) var ssr_output: texture_2d<f32>;
@group(0) @binding(4) var ssgi_output: texture_2d<f32>;
@group(0) @binding(5) var ssdo_output: texture_2d<f32>;
@group(0) @binding(6) var probe_gi: texture_2d<f32>;
@group(0) @binding(7) var sampler_obj: sampler;
@group(0) @binding(8) var<uniform> params: CompositeParams;

// GI合成主函数
@compute @workgroup_size(8, 8, 1)
fn gi_composite_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let resolution = textureDimensions(output_texture);
    let pixel = global_id.xy;

    if (pixel.x >= resolution.x || pixel.y >= resolution.y) {
        return;
    }

    let uv = vec2<f32>(pixel) / vec2<f32>(resolution);

    // 采样各个GI结果
    let color = textureSample(base_color, sampler_obj, uv).rgb;
    let rt_gi = textureSample(ray_tracing_gi, sampler_obj, uv).rgb;
    let ssr = textureSample(ssr_output, sampler_obj, uv).rgb;
    let ssgi = textureSample(ssgi_output, sampler_obj, uv).rgb;
    let ssdo = textureSample(ssdo_output, sampler_obj, uv).rgb;
    let probe = textureSample(probe_gi, sampler_obj, uv).rgb;

    // 加权合成
    var final_color = color;

    // 光线追踪GI
    final_color = mix(final_color, rt_gi, params.ray_tracing_weight);

    // 屏幕空间反射
    final_color = mix(final_color, ssr, params.ssr_weight);

    // 屏幕空间GI
    final_color = mix(final_color, ssgi, params.ssgi_weight);

    // 屏幕空间方向遮蔽
    final_color *= 1.0 - ssdo * params.ssdo_weight;

    // 光照探针
    final_color = mix(final_color, probe, params.probe_weight);

    // 色调映射
    final_color = acsr_tone_map(final_color);

    // Gamma校正
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));

    textureStore(output_texture, vec2<i32>(pixel), vec4<f32>(final_color, 1.0));
}

// ACES色调映射
fn acsr_tone_map(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;

    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}
