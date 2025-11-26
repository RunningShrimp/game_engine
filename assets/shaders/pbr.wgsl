// ============================================================================
// PBR (Physically Based Rendering) 材质着色器
// 基于 Cook-Torrance BRDF 模型
// ============================================================================

// 顶点输入
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>, // w = bitangent sign
};

// 实例数据
struct InstanceInput {
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
};

// 顶点输出 / 片段输入
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
    @location(5) view_depth: f32,
};

// 相机 Uniform
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    near: f32,
    far: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// 材质参数
struct MaterialUniform {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    emissive_strength: f32,
    emissive_color: vec3<f32>,
    _padding: f32,
};

@group(1) @binding(0) var<uniform> material: MaterialUniform;
@group(1) @binding(1) var albedo_texture: texture_2d<f32>;
@group(1) @binding(2) var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(3) var normal_texture: texture_2d<f32>;
@group(1) @binding(4) var ao_texture: texture_2d<f32>;
@group(1) @binding(5) var emissive_texture: texture_2d<f32>;
@group(1) @binding(6) var material_sampler: sampler;

// 光源数据
const MAX_LIGHTS: u32 = 16u;

struct PointLight {
    position: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    intensity: f32,
};

struct DirectionalLight {
    direction: vec3<f32>,
    _pad0: f32,
    color: vec3<f32>,
    intensity: f32,
};

struct LightsUniform {
    directional: DirectionalLight,
    point_lights: array<PointLight, MAX_LIGHTS>,
    point_light_count: u32,
    ambient_color: vec3<f32>,
    ambient_intensity: f32,
};

@group(2) @binding(0) var<uniform> lights: LightsUniform;

// ============================================================================
// 顶点着色器
// ============================================================================

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        instance.model_0,
        instance.model_1,
        instance.model_2,
        instance.model_3,
    );
    
    let world_position = model * vec4<f32>(vertex.position, 1.0);
    let normal_matrix = mat3x3<f32>(
        model[0].xyz,
        model[1].xyz,
        model[2].xyz,
    );
    
    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.world_position = world_position.xyz;
    out.world_normal = normalize(normal_matrix * vertex.normal);
    out.uv = vertex.uv;
    out.world_tangent = normalize(normal_matrix * vertex.tangent.xyz);
    out.world_bitangent = cross(out.world_normal, out.world_tangent) * vertex.tangent.w;
    
    // 计算视图空间深度 (用于 CSM 级联选择)
    let view_pos = camera.view * world_position;
    out.view_depth = -view_pos.z;
    
    return out;
}

// ============================================================================
// PBR 核心函数
// ============================================================================

const PI: f32 = 3.14159265359;

/// GGX/Trowbridge-Reitz 法线分布函数
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    
    let num = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return num / (PI * denom * denom);
}

/// Schlick-GGX 几何遮蔽函数
fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k);
}

/// Smith 几何函数
fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);
    return ggx1 * ggx2;
}

/// Fresnel-Schlick 菲涅尔近似
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

/// Fresnel-Schlick 带粗糙度 (用于环境光)
fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

/// PBR 光照计算
struct PbrInput {
    albedo: vec3<f32>,
    roughness: f32,
    metallic: f32,
    N: vec3<f32>,
    V: vec3<f32>,
    F0: vec3<f32>,
};

fn pbr_direct_lighting(input: PbrInput, L: vec3<f32>, radiance: vec3<f32>) -> vec3<f32> {
    let H = normalize(input.V + L);
    
    // Cook-Torrance BRDF
    let NDF = distribution_ggx(input.N, H, input.roughness);
    let G = geometry_smith(input.N, input.V, L, input.roughness);
    let F = fresnel_schlick(max(dot(H, input.V), 0.0), input.F0);
    
    let numerator = NDF * G * F;
    let denominator = 4.0 * max(dot(input.N, input.V), 0.0) * max(dot(input.N, L), 0.0) + 0.0001;
    let specular = numerator / denominator;
    
    // 能量守恒
    let kS = F;
    var kD = vec3<f32>(1.0) - kS;
    kD *= 1.0 - input.metallic; // 金属没有漫反射
    
    let NdotL = max(dot(input.N, L), 0.0);
    return (kD * input.albedo / PI + specular) * radiance * NdotL;
}

/// 从法线贴图采样获取世界空间法线
fn get_normal_from_map(uv: vec2<f32>, world_normal: vec3<f32>, world_tangent: vec3<f32>, world_bitangent: vec3<f32>) -> vec3<f32> {
    let tangent_normal = textureSample(normal_texture, material_sampler, uv).xyz * 2.0 - 1.0;
    
    let TBN = mat3x3<f32>(
        normalize(world_tangent),
        normalize(world_bitangent),
        normalize(world_normal),
    );
    
    return normalize(TBN * tangent_normal);
}

// ============================================================================
// 片段着色器
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样材质贴图
    let albedo_sample = textureSample(albedo_texture, material_sampler, in.uv);
    let albedo = albedo_sample.rgb * material.albedo.rgb;
    let alpha = albedo_sample.a * material.albedo.a;
    
    // Alpha 剔除
    if alpha < 0.1 {
        discard;
    }
    
    let metallic_roughness = textureSample(metallic_roughness_texture, material_sampler, in.uv);
    let metallic = metallic_roughness.b * material.metallic;
    let roughness = metallic_roughness.g * material.roughness;
    let ao = textureSample(ao_texture, material_sampler, in.uv).r * material.ao;
    let emissive = textureSample(emissive_texture, material_sampler, in.uv).rgb * material.emissive_color * material.emissive_strength;
    
    // 获取法线
    let N = get_normal_from_map(in.uv, in.world_normal, in.world_tangent, in.world_bitangent);
    let V = normalize(camera.camera_pos - in.world_position);
    
    // 计算 F0 (基础反射率)
    var F0 = vec3<f32>(0.04); // 非金属默认值
    F0 = mix(F0, albedo, metallic);
    
    let pbr_input = PbrInput(albedo, roughness, metallic, N, V, F0);
    
    // 累积光照
    var Lo = vec3<f32>(0.0);
    
    // 方向光
    if lights.directional.intensity > 0.0 {
        let L = normalize(-lights.directional.direction);
        let radiance = lights.directional.color * lights.directional.intensity;
        Lo += pbr_direct_lighting(pbr_input, L, radiance);
    }
    
    // 点光源
    for (var i = 0u; i < lights.point_light_count; i++) {
        let light = lights.point_lights[i];
        let L = normalize(light.position - in.world_position);
        let distance = length(light.position - in.world_position);
        
        // 衰减 (使用物理衰减 + 半径限制)
        let attenuation = 1.0 / (distance * distance + 1.0);
        let falloff = clamp(1.0 - pow(distance / light.radius, 4.0), 0.0, 1.0);
        let radiance = light.color * light.intensity * attenuation * falloff * falloff;
        
        Lo += pbr_direct_lighting(pbr_input, L, radiance);
    }
    
    // 环境光 (简化的 IBL)
    let ambient = lights.ambient_color * lights.ambient_intensity * albedo * ao;
    
    // 最终颜色
    var color = ambient + Lo + emissive;
    
    // HDR 色调映射 (ACES)
    color = aces_tonemap(color);
    
    // Gamma 校正
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, alpha);
}

/// ACES 色调映射
fn aces_tonemap(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// ============================================================================
// G-Buffer 输出 (延迟渲染路径)
// ============================================================================

struct GBufferOutput {
    @location(0) albedo_metallic: vec4<f32>,   // rgb = albedo, a = metallic
    @location(1) normal_roughness: vec4<f32>,  // rgb = normal (encoded), a = roughness
    @location(2) emissive_ao: vec4<f32>,       // rgb = emissive, a = ao
};

@fragment
fn fs_gbuffer(in: VertexOutput) -> GBufferOutput {
    let albedo_sample = textureSample(albedo_texture, material_sampler, in.uv);
    let albedo = albedo_sample.rgb * material.albedo.rgb;
    
    let metallic_roughness = textureSample(metallic_roughness_texture, material_sampler, in.uv);
    let metallic = metallic_roughness.b * material.metallic;
    let roughness = metallic_roughness.g * material.roughness;
    let ao = textureSample(ao_texture, material_sampler, in.uv).r * material.ao;
    let emissive = textureSample(emissive_texture, material_sampler, in.uv).rgb * material.emissive_color * material.emissive_strength;
    
    let N = get_normal_from_map(in.uv, in.world_normal, in.world_tangent, in.world_bitangent);
    
    // 编码法线到 [0, 1]
    let encoded_normal = N * 0.5 + 0.5;
    
    var output: GBufferOutput;
    output.albedo_metallic = vec4<f32>(albedo, metallic);
    output.normal_roughness = vec4<f32>(encoded_normal, roughness);
    output.emissive_ao = vec4<f32>(emissive, ao);
    
    return output;
}
