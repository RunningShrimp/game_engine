// GPU 蒙皮 PBR 着色器
// 支持骨骼动画的 PBR 渲染

// ============================================================================
// 绑定组定义
// ============================================================================

// @group(0) 相机
struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _pad: f32,
};
@group(0) @binding(0) var<uniform> camera: CameraUniforms;

// @group(1) 材质
struct MaterialUniforms {
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    normal_scale: f32,
    emissive: vec3<f32>,
    _pad: f32,
};
@group(1) @binding(0) var<uniform> material: MaterialUniforms;

// @group(2) 骨骼矩阵
@group(2) @binding(0) var<storage, read> bone_matrices: array<mat4x4<f32>>;

// ============================================================================
// 顶点输入/输出
// ============================================================================

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) bone_indices: vec4<u32>,
    @location(4) bone_weights: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

// ============================================================================
// 顶点着色器
// ============================================================================

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // GPU 蒙皮计算
    var skinned_position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var skinned_normal = vec3<f32>(0.0, 0.0, 0.0);
    
    // 遍历 4 个骨骼影响
    for (var i = 0u; i < 4u; i = i + 1u) {
        let bone_index = input.bone_indices[i];
        let weight = input.bone_weights[i];
        
        if (weight > 0.0001) {
            let bone_matrix = bone_matrices[bone_index];
            
            // 变换位置
            skinned_position += bone_matrix * vec4<f32>(input.position, 1.0) * weight;
            
            // 变换法线（只取旋转部分，假设无缩放或均匀缩放）
            let normal_matrix = mat3x3<f32>(
                bone_matrix[0].xyz,
                bone_matrix[1].xyz,
                bone_matrix[2].xyz
            );
            skinned_normal += normal_matrix * input.normal * weight;
        }
    }
    
    // 归一化法线
    skinned_normal = normalize(skinned_normal);
    
    // 应用相机变换
    output.clip_position = camera.view_proj * skinned_position;
    output.world_position = skinned_position.xyz;
    output.world_normal = skinned_normal;
    output.uv = input.uv;
    
    return output;
}

// ============================================================================
// 片段着色器（简化 PBR）
// ============================================================================

const PI: f32 = 3.14159265359;

// GGX/Trowbridge-Reitz 法线分布函数
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    
    let num = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return num / (PI * denom * denom);
}

// Schlick-GGX 几何遮蔽函数
fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k);
}

// Smith 几何遮蔽函数
fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx1 = geometry_schlick_ggx(NdotV, roughness);
    let ggx2 = geometry_schlick_ggx(NdotL, roughness);
    return ggx1 * ggx2;
}

// Fresnel-Schlick 近似
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 材质属性
    let albedo = material.base_color.rgb;
    let metallic = material.metallic;
    let roughness = material.roughness;
    let ao = material.ao;
    
    // 视线方向
    let N = normalize(input.world_normal);
    let V = normalize(camera.camera_pos - input.world_position);
    
    // 计算 F0（基础反射率）
    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo, metallic);
    
    // 简化光照：单个平行光
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let light_intensity = 2.0;
    
    let L = light_dir;
    let H = normalize(V + L);
    
    // Cook-Torrance BRDF
    let NDF = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
    
    let numerator = NDF * G * F;
    let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    let specular = numerator / denominator;
    
    // 漫反射与镜面反射比例
    var kS = F;
    var kD = vec3<f32>(1.0) - kS;
    kD *= 1.0 - metallic;
    
    // 最终光照
    let NdotL = max(dot(N, L), 0.0);
    let Lo = (kD * albedo / PI + specular) * light_color * light_intensity * NdotL;
    
    // 环境光
    let ambient = vec3<f32>(0.03) * albedo * ao;
    
    // 自发光
    let emissive = material.emissive;
    
    // 最终颜色
    var color = ambient + Lo + emissive;
    
    // HDR 色调映射 (Reinhard)
    color = color / (color + vec3<f32>(1.0));
    
    // Gamma 校正
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, 1.0);
}
