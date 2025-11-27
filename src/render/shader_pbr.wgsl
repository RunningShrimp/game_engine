// PBR 3D Shader

struct Uniforms3D {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
};

struct MaterialUniform {
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    normal_scale: f32,
    emissive: vec3<f32>,
};

struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    radius: f32,
};

struct DirectionalLight {
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms3D;
@group(1) @binding(0) var<uniform> material: MaterialUniform;
@group(2) @binding(0) var<storage, read> point_lights: array<PointLight>;
@group(2) @binding(1) var<storage, read> dir_lights: array<DirectionalLight>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let model_matrix = mat4x4<f32>(
        vertex.model_matrix_0,
        vertex.model_matrix_1,
        vertex.model_matrix_2,
        vertex.model_matrix_3
    );
    let world_pos = model_matrix * vec4<f32>(vertex.position, 1.0);
    out.world_position = world_pos.xyz;
    out.clip_position = uniforms.view_proj * world_pos;
    out.world_normal = normalize((model_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.uv = vertex.uv;
    return out;
}

// PBR 辅助函数

const PI: f32 = 3.14159265359;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let V = normalize(uniforms.camera_pos - in.world_position);
    
    // 基础颜色和材质参数
    let albedo = material.base_color.rgb;
    let metallic = material.metallic;
    let roughness = material.roughness;
    let ao = material.ao;
    
    // 计算F0 (表面反射率)
    var F0 = vec3<f32>(0.04); // 非金属的默认值
    F0 = mix(F0, albedo, metallic);
    
    // 反射率方程
    var Lo = vec3<f32>(0.0);
    
    // 点光源
    let num_point_lights = arrayLength(&point_lights);
    for (var i = 0u; i < num_point_lights; i++) {
        let light = point_lights[i];
        let L = normalize(light.position - in.world_position);
        let H = normalize(V + L);
        let distance = length(light.position - in.world_position);
        let attenuation = 1.0 / (distance * distance);
        let radiance = light.color * light.intensity * attenuation;
        
        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
        
        let kS = F;
        var kD = vec3<f32>(1.0) - kS;
        kD *= 1.0 - metallic;
        
        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;
        
        let NdotL = max(dot(N, L), 0.0);
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }
    
    // 方向光
    let num_dir_lights = arrayLength(&dir_lights);
    for (var i = 0u; i < num_dir_lights; i++) {
        let light = dir_lights[i];
        let L = normalize(-light.direction);
        let H = normalize(V + L);
        let radiance = light.color * light.intensity;
        
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
        
        let kS = F;
        var kD = vec3<f32>(1.0) - kS;
        kD *= 1.0 - metallic;
        
        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;
        
        let NdotL = max(dot(N, L), 0.0);
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }
    
    // 环境光
    let ambient = vec3<f32>(0.03) * albedo * ao;
    var color = ambient + Lo + material.emissive;
    
    // HDR色调映射
    color = color / (color + vec3<f32>(1.0));
    // Gamma校正
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, material.base_color.a);
}
