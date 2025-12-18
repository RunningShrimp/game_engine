// ============================================================================
// CSM (Cascaded Shadow Maps) 级联阴影着色器
// 支持最多 4 级级联，带 PCF 软阴影
// ============================================================================

// 级联数据
const MAX_CASCADES: u32 = 4u;

struct Cascade {
    view_proj: mat4x4<f32>,
    split_depth: f32,
    _padding: vec3<f32>,
};

struct CascadesUniform {
    cascades: array<Cascade, MAX_CASCADES>,
    cascade_count: u32,
    shadow_bias: f32,
    normal_bias: f32,
    pcf_radius: f32,
    light_direction: vec3<f32>,
    _padding: f32,
};

@group(3) @binding(0) var<uniform> csm: CascadesUniform;
@group(3) @binding(1) var shadow_map: texture_depth_2d_array;
@group(3) @binding(2) var shadow_sampler: sampler_comparison;

// ============================================================================
// 阴影深度渲染 (Shadow Pass)
// ============================================================================

struct ShadowVertexInput {
    @location(0) position: vec3<f32>,
};

struct ShadowInstanceInput {
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
};

struct ShadowVertexOutput {
    @builtin(position) position: vec4<f32>,
};

// Push constant for cascade index (使用单独的 uniform)
struct ShadowPushConstant {
    cascade_index: u32,
};

@group(0) @binding(1) var<uniform> shadow_push: ShadowPushConstant;

@vertex
fn vs_shadow(vertex: ShadowVertexInput, instance: ShadowInstanceInput) -> ShadowVertexOutput {
    let model = mat4x4<f32>(
        instance.model_0,
        instance.model_1,
        instance.model_2,
        instance.model_3,
    );
    
    let world_position = model * vec4<f32>(vertex.position, 1.0);
    let cascade = csm.cascades[shadow_push.cascade_index];
    
    var out: ShadowVertexOutput;
    out.position = cascade.view_proj * world_position;
    return out;
}

@fragment
fn fs_shadow() {
    // 深度写入自动完成，无需输出
}

// ============================================================================
// 阴影采样函数 (在主渲染 Pass 中使用)
// ============================================================================

/// 选择合适的级联层级
fn select_cascade(view_depth: f32) -> u32 {
    var cascade_index = csm.cascade_count - 1u;
    for (var i = 0u; i < csm.cascade_count - 1u; i++) {
        if view_depth < csm.cascades[i].split_depth {
            cascade_index = i;
            break;
        }
    }
    return cascade_index;
}

/// 计算阴影坐标
fn get_shadow_coords(world_pos: vec3<f32>, cascade_index: u32) -> vec3<f32> {
    let cascade = csm.cascades[cascade_index];
    let light_space_pos = cascade.view_proj * vec4<f32>(world_pos, 1.0);
    
    // 透视除法
    var proj_coords = light_space_pos.xyz / light_space_pos.w;
    
    // 转换到 [0, 1] UV 空间
    proj_coords.x = proj_coords.x * 0.5 + 0.5;
    proj_coords.y = proj_coords.y * -0.5 + 0.5; // Y 翻转 (Vulkan/wgpu 坐标系)
    
    return proj_coords;
}

/// PCF 软阴影采样
fn sample_shadow_pcf(world_pos: vec3<f32>, world_normal: vec3<f32>, view_depth: f32) -> f32 {
    let cascade_index = select_cascade(view_depth);
    
    // 应用法线偏移 (减少阴影痤疮)
    let bias_offset = world_normal * csm.normal_bias;
    let biased_pos = world_pos + bias_offset;
    
    let shadow_coords = get_shadow_coords(biased_pos, cascade_index);
    
    // 边界检查
    if shadow_coords.x < 0.0 || shadow_coords.x > 1.0 ||
       shadow_coords.y < 0.0 || shadow_coords.y > 1.0 ||
       shadow_coords.z < 0.0 || shadow_coords.z > 1.0 {
        return 1.0; // 在阴影区域外，完全照亮
    }
    
    let current_depth = shadow_coords.z - csm.shadow_bias;
    
    // PCF 采样 (3x3 kernel)
    var shadow = 0.0;
    let texel_size = 1.0 / vec2<f32>(textureDimensions(shadow_map));
    let pcf_radius = csm.pcf_radius;
    
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size * pcf_radius;
            let sample_uv = shadow_coords.xy + offset;
            
            shadow += textureSampleCompare(
                shadow_map,
                shadow_sampler,
                sample_uv,
                i32(cascade_index),
                current_depth
            );
        }
    }
    
    return shadow / 9.0;
}

/// 高质量 PCF (5x5 Poisson Disk)
fn sample_shadow_pcf_hq(world_pos: vec3<f32>, world_normal: vec3<f32>, view_depth: f32) -> f32 {
    let cascade_index = select_cascade(view_depth);
    
    let bias_offset = world_normal * csm.normal_bias;
    let biased_pos = world_pos + bias_offset;
    let shadow_coords = get_shadow_coords(biased_pos, cascade_index);
    
    if shadow_coords.x < 0.0 || shadow_coords.x > 1.0 ||
       shadow_coords.y < 0.0 || shadow_coords.y > 1.0 {
        return 1.0;
    }
    
    let current_depth = shadow_coords.z - csm.shadow_bias;
    let texel_size = 1.0 / vec2<f32>(textureDimensions(shadow_map));
    
    // Poisson Disk 采样点
    let poisson_disk = array<vec2<f32>, 16>(
        vec2<f32>(-0.94201624, -0.39906216),
        vec2<f32>(0.94558609, -0.76890725),
        vec2<f32>(-0.094184101, -0.92938870),
        vec2<f32>(0.34495938, 0.29387760),
        vec2<f32>(-0.91588581, 0.45771432),
        vec2<f32>(-0.81544232, -0.87912464),
        vec2<f32>(-0.38277543, 0.27676845),
        vec2<f32>(0.97484398, 0.75648379),
        vec2<f32>(0.44323325, -0.97511554),
        vec2<f32>(0.53742981, -0.47373420),
        vec2<f32>(-0.26496911, -0.41893023),
        vec2<f32>(0.79197514, 0.19090188),
        vec2<f32>(-0.24188840, 0.99706507),
        vec2<f32>(-0.81409955, 0.91437590),
        vec2<f32>(0.19984126, 0.78641367),
        vec2<f32>(0.14383161, -0.14100790)
    );
    
    var shadow = 0.0;
    let spread = csm.pcf_radius * 2.0;
    
    for (var i = 0u; i < 16u; i++) {
        let offset = poisson_disk[i] * texel_size * spread;
        let sample_uv = shadow_coords.xy + offset;
        
        shadow += textureSampleCompare(
            shadow_map,
            shadow_sampler,
            sample_uv,
            i32(cascade_index),
            current_depth
        );
    }
    
    return shadow / 16.0;
}

/// 级联调试可视化 (可选)
fn cascade_debug_color(view_depth: f32) -> vec3<f32> {
    let cascade_index = select_cascade(view_depth);
    
    switch cascade_index {
        case 0u: { return vec3<f32>(1.0, 0.0, 0.0); } // 红
        case 1u: { return vec3<f32>(0.0, 1.0, 0.0); } // 绿
        case 2u: { return vec3<f32>(0.0, 0.0, 1.0); } // 蓝
        case 3u: { return vec3<f32>(1.0, 1.0, 0.0); } // 黄
        default: { return vec3<f32>(1.0, 1.0, 1.0); }
    }
}

// ============================================================================
// 级联分割计算 (CPU 端伪代码，参考用)
// ============================================================================

// fn calculate_cascade_splits(near: f32, far: f32, cascade_count: u32, lambda: f32) -> array<f32, MAX_CASCADES> {
//     var splits: array<f32, MAX_CASCADES>;
//     
//     for i in 0..cascade_count {
//         let p = (i + 1) as f32 / cascade_count as f32;
//         
//         // 对数分割
//         let log_split = near * pow(far / near, p);
//         
//         // 均匀分割
//         let uniform_split = near + (far - near) * p;
//         
//         // 混合 (lambda = 0: 均匀, lambda = 1: 对数)
//         splits[i] = log_split * lambda + uniform_split * (1.0 - lambda);
//     }
//     
//     return splits;
// }

// ============================================================================
// VSM (Variance Shadow Maps) 可选实现
// ============================================================================

/// VSM 阴影采样 (更柔和的边缘，但需要额外的 blur pass)
fn sample_shadow_vsm(world_pos: vec3<f32>, view_depth: f32) -> f32 {
    let cascade_index = select_cascade(view_depth);
    let shadow_coords = get_shadow_coords(world_pos, cascade_index);
    
    // VSM 需要存储 depth 和 depth^2
    // 这里只是示意，实际需要使用 RG32F 纹理
    let current_depth = shadow_coords.z;
    
    // 假设 moments.x = E[depth], moments.y = E[depth^2]
    // let moments = textureSample(vsm_shadow_map, shadow_sampler, shadow_coords.xy, cascade_index).xy;
    
    // Chebyshev 不等式
    // let variance = max(moments.y - moments.x * moments.x, 0.0001);
    // let d = current_depth - moments.x;
    // let p_max = variance / (variance + d * d);
    
    // return select(1.0, p_max, current_depth > moments.x);
    
    return 1.0; // 占位
}
