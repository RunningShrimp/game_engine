// 延迟渲染 - 光照阶段着色器 (带CSM阴影)

@group(0) @binding(0) var g_position: texture_2d<f32>;
@group(0) @binding(1) var g_normal: texture_2d<f32>;
@group(0) @binding(2) var g_albedo: texture_2d<f32>;
@group(0) @binding(3) var g_sampler: sampler;

// CSM阴影贴图
@group(1) @binding(0) var shadow_sampler: sampler_comparison;
@group(1) @binding(1) var shadow_map_0: texture_depth_2d;
@group(1) @binding(2) var shadow_map_1: texture_depth_2d;
@group(1) @binding(3) var shadow_map_2: texture_depth_2d;
@group(1) @binding(4) var shadow_map_3: texture_depth_2d;

// CSM Uniform
struct CsmUniforms {
    light_view_proj_0: mat4x4<f32>,
    light_view_proj_1: mat4x4<f32>,
    light_view_proj_2: mat4x4<f32>,
    light_view_proj_3: mat4x4<f32>,
    cascade_distances: vec4<f32>,
    light_direction: vec3<f32>,
    _pad: f32,
};

@group(2) @binding(0) var<uniform> csm: CsmUniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    // 将NDC坐标转换为UV坐标 ([-1,1] -> [0,1])
    out.uv = vertex.position * 0.5 + 0.5;
    // 翻转Y轴 (因为纹理坐标原点在左上角)
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

const PI: f32 = 3.14159265359;

// PBR辅助函数
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    
    let num = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return num / denom;
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    
    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;
    
    return num / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);
    
    return ggx1 * ggx2;
}

fn fresnel_schlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

// CSM阴影计算
fn calculate_shadow(world_pos: vec3<f32>, view_depth: f32) -> f32 {
    // 选择合适的级联
    var cascade_index = 0u;
    if view_depth < csm.cascade_distances.x {
        cascade_index = 0u;
    } else if view_depth < csm.cascade_distances.y {
        cascade_index = 1u;
    } else if view_depth < csm.cascade_distances.z {
        cascade_index = 2u;
    } else {
        cascade_index = 3u;
    }
    
    // 计算光源空间坐标
    var light_space_pos: vec4<f32>;
    if cascade_index == 0u {
        light_space_pos = csm.light_view_proj_0 * vec4<f32>(world_pos, 1.0);
    } else if cascade_index == 1u {
        light_space_pos = csm.light_view_proj_1 * vec4<f32>(world_pos, 1.0);
    } else if cascade_index == 2u {
        light_space_pos = csm.light_view_proj_2 * vec4<f32>(world_pos, 1.0);
    } else {
        light_space_pos = csm.light_view_proj_3 * vec4<f32>(world_pos, 1.0);
    }
    
    // 透视除法
    let proj_coords = light_space_pos.xyz / light_space_pos.w;
    
    // 转换到[0,1]范围
    let shadow_coords = proj_coords * 0.5 + 0.5;
    
    // 检查是否在阴影贴图范围内
    if shadow_coords.x < 0.0 || shadow_coords.x > 1.0 ||
       shadow_coords.y < 0.0 || shadow_coords.y > 1.0 ||
       shadow_coords.z < 0.0 || shadow_coords.z > 1.0 {
        return 1.0; // 不在阴影范围内
    }
    
    // PCF (Percentage Closer Filtering)
    var shadow = 0.0;
    let texel_size = 1.0 / 2048.0; // 阴影贴图分辨率
    
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_coords = shadow_coords.xy + offset;
            
            var depth: f32;
            if cascade_index == 0u {
                depth = textureSampleCompare(shadow_map_0, shadow_sampler, sample_coords, shadow_coords.z);
            } else if cascade_index == 1u {
                depth = textureSampleCompare(shadow_map_1, shadow_sampler, sample_coords, shadow_coords.z);
            } else if cascade_index == 2u {
                depth = textureSampleCompare(shadow_map_2, shadow_sampler, sample_coords, shadow_coords.z);
            } else {
                depth = textureSampleCompare(shadow_map_3, shadow_sampler, sample_coords, shadow_coords.z);
            }
            
            shadow += depth;
        }
    }
    
    shadow /= 9.0; // 9个采样点的平均值
    
    return shadow;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 从G-Buffer读取数据
    let tex_coords = vec2<i32>(in.uv * vec2<f32>(textureDimensions(g_position)));
    let position_data = textureLoad(g_position, tex_coords, 0);
    let normal_data = textureLoad(g_normal, tex_coords, 0);
    let albedo_data = textureLoad(g_albedo, tex_coords, 0);
    
    let world_pos = position_data.xyz;
    let view_depth = position_data.w;
    let normal = normalize(normal_data.xyz);
    let albedo = albedo_data.rgb;
    let roughness = normal_data.a;
    let metallic = albedo_data.a;
    
    // 相机位置 (暂时硬编码)
    let camera_pos = vec3<f32>(0.0, 0.0, 5.0);
    let V = normalize(camera_pos - world_pos);
    
    // 计算F0
    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo, metallic);
    
    // 计算阴影
    let shadow = calculate_shadow(world_pos, view_depth);
    
    // 方向光
    let light_dir = normalize(csm.light_direction);
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let light_intensity = 1.0;
    
    let L = light_dir;
    let H = normalize(V + L);
    let radiance = light_color * light_intensity;
    
    // Cook-Torrance BRDF
    let NDF = distribution_ggx(normal, H, roughness);
    let G = geometry_smith(normal, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
    
    let kS = F;
    var kD = vec3<f32>(1.0) - kS;
    kD *= 1.0 - metallic;
    
    let numerator = NDF * G * F;
    let denominator = 4.0 * max(dot(normal, V), 0.0) * max(dot(normal, L), 0.0) + 0.0001;
    let specular = numerator / denominator;
    
    let NdotL = max(dot(normal, L), 0.0);
    let Lo = (kD * albedo / PI + specular) * radiance * NdotL * shadow; // 应用阴影
    
    // 环境光
    let ambient = vec3<f32>(0.03) * albedo;
    var color = ambient + Lo;
    
    // HDR色调映射
    color = color / (color + vec3<f32>(1.0));
    // Gamma校正
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, 1.0);
}
