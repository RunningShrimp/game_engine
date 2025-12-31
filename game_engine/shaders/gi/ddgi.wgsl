// DDGI (Dynamic Diffuse Global Illumination) Shaders
// 用于实现动态漫反射全局光照

// ============================================
// DDGI Uniforms
// ============================================

struct DDGIUniforms {
    volume_origin: vec3<f32>,
    volume_size: vec3<f32>,
    probe_counts: vec3<u32>,
    probe_spacing: f32,
    max_depth: f32,
    normal_bias: f32,
    padding: f32,
};

@group(0) @binding(0)
var<uniform> ddgi: DDGIUniforms;

@group(0) @binding(1)
var irradiance_texture: texture_storage_2d_array<rgba32float, read_write>;

@group(0) @binding(2)
var depth_texture: texture_storage_2d_array<r32float, read_write>;

@group(0) @binding(3)
var offset_texture: texture_storage_2d_array<rg32float, read>;

// ============================================
// 辅助函数
// ============================================

// 计算探针索引
fn get_probe_index(world_pos: vec3<f32>) -> vec3<u32> {
    let local_pos = world_pos - ddgi.volume_origin;
    let probe_index = vec3<u32>(
        u32(local_pos.x / ddgi.probe_spacing),
        u32(local_pos.y / ddgi.probe_spacing),
        u32(local_pos.z / ddgi.probe_spacing),
    );
    return clamp(probe_index, vec3<u32>(0u), ddgi.probe_counts - vec3<u32>(1u));
}

// 计算探针的线性索引
fn get_probe_linear_index(grid_index: vec3<u32>) -> u32 {
    return grid_index.z * ddgi.probe_counts.x * ddgi.probe_counts.y +
           grid_index.y * ddgi.probe_counts.x +
           grid_index.x;
}

// 获取探针世界位置
fn get_probe_position(grid_index: vec3<u32>) -> vec3<f32> {
    return ddgi.volume_origin + vec3<f32>(grid_index) * ddgi.probe_spacing;
}

// 立方体面方向（用于探针渲染）
const CUBE_FACES: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3<f32>(1.0, 0.0, 0.0),   // +X
    vec3<f32>(-1.0, 0.0, 0.0),  // -X
    vec3<f32>(0.0, 1.0, 0.0),   // +Y
    vec3<f32>(0.0, -1.0, 0.0),  // -Y
    vec3<f32>(0.0, 0.0, 1.0),   // +Z
    vec3<f32>(0.0, 0.0, -1.0),  // -Z
);

// ============================================
// 辐照度采样
// ============================================

// 采样辐照度纹理
fn sample_irradiance(probe_index: u32, face: u32, tex_coord: vec2<f32>) -> vec3<f32> {
    let texture_size = vec2<i32>(textureDimensions(irradiance_texture).xy);
    let coord = vec2<i32>(tex_coord * vec2<f32>(texture_size));

    let irradiance = textureLoad(irradiance_texture, coord, i32(probe_index * 6u + face));
    return irradiance.rgb;
}

// 三线性插值采样
fn sample_irradiance_trilinear(world_pos: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let grid_index = get_probe_index(world_pos);
    let local_pos = (world_pos - ddgi.volume_origin) / ddgi.probe_spacing;
    let base_pos = floor(local_pos);
    let fract_pos = fract(local_pos);

    let mut result = vec3<f32>(0.0);
    var weight_sum = 0.0;

    // 8个邻居探针
    for (var dz: u32 = 0u; dz < 2u; dz++) {
        for (var dy: u32 = 0u; dy < 2u; dy++) {
            for (var dx: u32 = 0u; dx < 2u; dx++) {
                let offset = vec3<u32>(dx, dy, dz);
                let neighbor_index = clamp(grid_index + offset, vec3<u32>(0u), ddgi.probe_counts - vec3<u32>(1u));
                let probe_idx = get_probe_linear_index(neighbor_index);
                let probe_pos = get_probe_position(neighbor_index);

                // 计算权重（距离衰减）
                let dist = length(world_pos - probe_pos);
                let weight = max(0.0, 1.0 - dist / (ddgi.probe_spacing * 2.0));

                // 采样6个方向的辐照度
                let mut probe_irradiance = vec3<f32>(0.0);
                for (var face: u32 = 0u; face < 6u; face++) {
                    let face_dir = CUBE_FACES[face];
                    let n_dot_l = max(0.0, dot(normal, face_dir));
                    let face_irradiance = sample_irradiance(probe_idx, face, vec2<f32>(0.5, 0.5));
                    probe_irradiance += face_irradiance * n_dot_l;
                }

                result += probe_irradiance * weight;
                weight_sum += weight;
            }
        }
    }

    if (weight_sum > 0.0) {
        return result / weight_sum;
    }
    return vec3<f32>(0.0);
}

// ============================================
// 深度采样
// ============================================

// 采样深度纹理
fn sample_depth(probe_index: u32, face: u32, tex_coord: vec2<f32>) -> f32 {
    let texture_size = vec2<i32>(textureDimensions(depth_texture).xy);
    let coord = vec2<i32>(tex_coord * vec2<f32>(texture_size));

    let depth = textureLoad(depth_texture, coord, i32(probe_index * 6u + face));
    return depth.r;
}

// 计算可见性
fn compute_visibility(world_pos: vec3<f32>, probe_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let to_probe = normalize(probe_pos - world_pos);
    let n_dot_l = dot(normal, to_probe);

    if (n_dot_l <= 0.0) {
        return 0.0;
    }

    let biased_pos = world_pos + normal * ddgi.normal_bias;
    let dist = length(probe_pos - biased_pos);

    // 简化可见性计算
    // 实际实现中应该采样深度纹理
    return 1.0;
}

// ============================================
// 辐照度更新（计算着色器）
// ============================================

@compute @workgroup_size(16, 16, 1)
fn update_irradiance(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let probe_index = global_id.z;
    let face = global_id.y / 16u;
    let texel_x = global_id.x % 16u;
    let texel_y = global_id.y % 16u;

    let total_probes = ddgi.probe_counts.x * ddgi.probe_counts.y * ddgi.probe_counts.z;

    if (probe_index >= total_probes || face >= 6u) {
        return;
    }

    // 获取探针位置
    let grid_index = vec3<u32>(
        probe_index % ddgi.probe_counts.x,
        (probe_index / ddgi.probe_counts.x) % ddgi.probe_counts.y,
        probe_index / (ddgi.probe_counts.x * ddgi.probe_counts.y)
    );
    let probe_pos = get_probe_position(grid_index);

    // 计算纹理坐标对应的方向
    let tex_size = f32(16u);
    let u = (f32(texel_x) + 0.5) / tex_size;
    let v = (f32(texel_y) + 0.5) / tex_size;

    // 将UV转换为立方体方向
    let face_dir = CUBE_FACES[face];
    let mut dir = vec3<f32>(0.0);

    if (face == 0u) { // +X
        dir = normalize(vec3<f32>(1.0, 2.0 * (v - 0.5), 2.0 * (u - 0.5)));
    } else if (face == 1u) { // -X
        dir = normalize(vec3<f32>(-1.0, 2.0 * (v - 0.5), 2.0 * (0.5 - u)));
    } else if (face == 2u) { // +Y
        dir = normalize(vec3<f32>(2.0 * (u - 0.5), 1.0, 2.0 * (v - 0.5)));
    } else if (face == 3u) { // -Y
        dir = normalize(vec3<f32>(2.0 * (u - 0.5), -1.0, 2.0 * (0.5 - v)));
    } else if (face == 4u) { // +Z
        dir = normalize(vec3<f32>(2.0 * (u - 0.5), 2.0 * (v - 0.5), 1.0));
    } else { // -Z
        dir = normalize(vec3<f32>(2.0 * (0.5 - u), 2.0 * (v - 0.5), -1.0));
    }

    // 从深度纹理采样并计算辐照度
    let depth = sample_depth(probe_index, face, vec2<f32>(u, v));
    let visibility = if (depth > 0.0) { 1.0 } else { 0.0 };

    // 简化辐照度计算（实际应该积分环境光照）
    let irradiance = vec3<f32>(0.5) * visibility;

    // 存储结果
    let coord = vec2<i32>(i32(texel_x), i32(texel_y));
    let layer = i32(probe_index * 6u + face);
    textureStore(irradiance_texture, coord, layer, vec4<f32>(irradiance, 1.0));
}

// ============================================
// 光照传播（计算着色器）
// ============================================

@compute @workgroup_size(16, 16, 1)
fn propagate_lighting(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let probe_index = global_id.z;
    let face = global_id.y / 16u;
    let texel_x = global_id.x % 16u;
    let texel_y = global_id.y % 16u;

    let total_probes = ddgi.probe_counts.x * ddgi.probe_counts.y * ddgi.probe_counts.z;

    if (probe_index >= total_probes || face >= 6u) {
        return;
    }

    // 读取当前辐照度
    let coord = vec2<i32>(i32(texel_x), i32(texel_y));
    let layer = i32(probe_index * 6u + face);
    let current_irradiance = textureLoad(irradiance_texture, coord, layer);

    // 从邻居探针传播光照
    let mut propagated_irradiance = vec3<f32>(0.0);

    // 简化实现：当前跳过
    // 实际应该从相邻探针采样并加权平均

    // 存储结果
    textureStore(irradiance_texture, coord, layer, vec4<f32>(propagated_irradiance, 1.0));
}

// ============================================
// 片段着色器 - 采样DDGI光照
// ============================================

@fragment
fn sample_ddgi(
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>
) -> vec3<f32> {
    // 使用DDGI计算全局光照
    let gi = sample_irradiance_trilinear(world_pos, normal);

    return gi;
}
