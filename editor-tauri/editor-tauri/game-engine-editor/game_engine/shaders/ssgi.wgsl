// 屏幕空间全局光照着色器
// 提供屏幕空间间接光照

struct SSGIParams {
    sample_radius: f32,
    sample_count: u32,
    intensity: f32,
};

@group(0) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var normal_texture: texture_2d<f32>;
@group(0) @binding(3) var color_texture: texture_2d<f32>;
@group(0) @binding(5) var sampler_obj: sampler;
@group(0) @binding(6) var<uniform> params: SSGIParams;

// 将深度转换为世界空间位置
fn depth_to_world_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let clip_pos = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    let world_pos = clip_pos.xyz / clip_pos.w;
    return world_pos;
}

// SSGI采样
fn sample_ssgi(
    center_pos: vec3<f32>,
    center_normal: vec3<f32>,
    pixel: vec2<i32>,
    resolution: vec2<u32>,
) -> vec3<f32> {
    var accumulated = vec3<f32>(0.0);
    var valid_samples = 0u;

    let res_f = vec2<f32>(resolution);

    for (var i = 0u; i < params.sample_count; i++) {
        // 生成采样方向
        let angle = 2.0 * 3.14159 * f32(i) / f32(params.sample_count);
        let sample_dir_2d = vec2<f32>(cos(angle), sin(angle));

        // 沿法线半球采样
        let sample_offset = sample_dir_2d * params.sample_radius;
        let sample_uv = (vec2<f32>(pixel) + sample_offset) / res_f;

        // 边界检查
        if (sample_uv.x < 0.0 || sample_uv.x > 1.0 ||
            sample_uv.y < 0.0 || sample_uv.y > 1.0) {
            continue;
        }

        let sample_pixel = vec2<i32>(sample_uv * res_f);

        // 采样邻居
        let sample_depth = textureLoad(depth_texture, sample_pixel, 0);
        let sample_normal = textureLoad(normal_texture, sample_pixel, 0).rgb;
        let sample_color = textureLoad(color_texture, sample_pixel, 0).rgb;

        // 计算世界空间位置
        let sample_pos = depth_to_world_position(sample_uv, sample_depth);

        // 法线和距离检查
        let normal_similarity = dot(center_normal, sample_normal);
        let distance = length(sample_pos - center_pos);

        if (normal_similarity > 0.5 && distance < params.sample_radius) {
            // 累积间接光照
            accumulated += sample_color * normal_similarity;
            valid_samples++;
        }
    }

    if (valid_samples > 0u) {
        return accumulated / f32(valid_samples);
    }

    return vec3<f32>(0.0);
}

// SSGI主函数
@compute @workgroup_size(8, 8, 1)
fn ssgi_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let resolution = textureDimensions(output_texture);
    let pixel = global_id.xy;

    if (pixel.x >= resolution.x || pixel.y >= resolution.y) {
        return;
    }

    // 采样输入
    let depth = textureLoad(depth_texture, vec2<i32>(pixel), 0);
    let normal = textureLoad(normal_texture, vec2<i32>(pixel), 0).rgb;
    let color = textureLoad(color_texture, vec2<i32>(pixel), 0).rgb;

    // 重建世界空间位置
    let uv = vec2<f32>(pixel) / vec2<f32>(resolution);
    let world_pos = depth_to_world_position(uv, depth);

    // 采样间接光照
    let indirect = sample_ssgi(world_pos, normal, vec2<i32>(pixel), resolution);

    // 混合直接和间接光照
    let final_color = color + indirect * params.intensity;

    textureStore(output_texture, vec2<i32>(pixel), vec4<f32>(final_color, 1.0));
}
