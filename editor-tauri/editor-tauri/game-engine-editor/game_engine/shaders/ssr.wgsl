// 屏幕空间反射着色器
// 提供高性能的屏幕空间反射

struct SSRParams {
    max_step_distance: f32,
    step_count: u32,
    binary_search_iterations: u32,
    roughness_threshold: f32,
    blend_factor: f32,
};

@group(0) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var normal_texture: texture_2d<f32>;
@group(0) @binding(3) var color_texture: texture_2d<f32>;
@group(0) @binding(4) var roughness_texture: texture_2d<f32>;
@group(0) @binding(5) var sampler_obj: sampler;
@group(0) @binding(6) var<uniform> params: SSRParams;

// 将深度转换为世界空间位置
fn depth_to_world_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    // 简化的深度转换
    // 实际应用中需要完整的逆投影矩阵
    let clip_pos = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    let world_pos = clip_pos.xyz / clip_pos.w;
    return world_pos;
}

// 屏幕空间光线行进
fn screen_space_ray_march(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    resolution: vec2<u32>,
) -> vec2<f32> {
    let res_f = vec2<f32>(resolution);

    // 步进
    var ray_pos = ray_origin;
    var step_size = params.max_step_distance / f32(params.step_count);

    for (var i = 0u; i < params.step_count; i++) {
        ray_pos += ray_dir * step_size;

        // 检查是否超出屏幕
        if (ray_pos.x < 0.0 || ray_pos.x > 1.0 ||
            ray_pos.y < 0.0 || ray_pos.y > 1.0 ||
            ray_pos.z < 0.0 || ray_pos.z > 1.0) {
            break;
        }

        // 采样深度
        let depth = textureLoad(depth_texture, vec2<i32>(ray_pos.xy * res_f), 0);

        // 检查相交
        if (ray_pos.z > depth) {
            // 二分搜索精化
            return binary_search(ray_origin, ray_dir, i, resolution);
        }
    }

    return vec2<f32>(-1.0);
}

// 二分搜索
fn binary_search(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    iteration: u32,
    resolution: vec2<u32>,
) -> vec2<f32> {
    var start = ray_origin;
    var end = start + ray_dir * (params.max_step_distance / f32(params.step_count)) * f32(iteration);

    for (var i = 0u; i < params.binary_search_iterations; i++) {
        let mid = (start + end) * 0.5;

        let res_f = vec2<f32>(resolution);
        let depth = textureLoad(depth_texture, vec2<i32>(mid.xy * res_f), 0);

        if (mid.z > depth) {
            end = mid;
        } else {
            start = mid;
        }
    }

    return end.xy;
}

// SSR主函数
@compute @workgroup_size(8, 8, 1)
fn ssr_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let resolution = textureDimensions(output_texture);
    let pixel = global_id.xy;

    if (pixel.x >= resolution.x || pixel.y >= resolution.y) {
        return;
    }

    let uv = vec2<f32>(pixel) / vec2<f32>(resolution);

    // 采样输入
    let depth = textureLoad(depth_texture, vec2<i32>(pixel), 0);
    let normal = textureLoad(normal_texture, vec2<i32>(pixel), 0).rgb;
    let color = textureLoad(color_texture, vec2<i32>(pixel), 0).rgb;
    let roughness = textureLoad(roughness_texture, vec2<i32>(pixel), 0).r;

    // 粗糙度检查
    if (roughness > params.roughness_threshold) {
        textureStore(output_texture, vec2<i32>(pixel), vec4<f32>(color, 1.0));
        return;
    }

    // 重建世界空间位置
    let world_pos = depth_to_world_position(uv, depth);

    // 生成反射方向
    let view_dir = normalize(-world_pos);
    let reflect_dir = reflect(view_dir, normal);

    // 屏幕空间光线行进
    let hit_uv = screen_space_ray_march(
        vec3<f32>(uv, depth),
        vec3<f32>(reflect_dir.xy, reflect_dir.z * 0.5), // 简化的投影
        resolution
    );

    var reflection_color = color;

    if (hit_uv.x >= 0.0) {
        // 采样反射颜色
        let res_f = vec2<f32>(resolution);
        let hit_pixel = vec2<i32>(hit_uv * res_f);

        // 边界检查
        if (hit_pixel.x >= 0 && hit_pixel.x < i32(resolution.x) &&
            hit_pixel.y >= 0 && hit_pixel.y < i32(resolution.y)) {
            reflection_color = textureLoad(color_texture, hit_pixel, 0).rgb;
        }
    }

    // 混合原始颜色和反射
    let final_color = mix(color, reflection_color, params.blend_factor * (1.0 - roughness));

    textureStore(output_texture, vec2<i32>(pixel), vec4<f32>(final_color, 1.0));
}
