// 延迟渲染 - 光照阶段着色器

@group(0) @binding(0) var g_position: texture_2d<f32>;
@group(0) @binding(1) var g_normal: texture_2d<f32>;
@group(0) @binding(2) var g_albedo: texture_2d<f32>;
@group(0) @binding(3) var g_sampler: sampler;

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

// PBR辅助函数 (与前向渲染相同)
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
    // 从G-Buffer读取数据
    let tex_coords = vec2<i32>(in.uv * vec2<f32>(textureDimensions(g_position)));
    let position_data = textureLoad(g_position, tex_coords, 0);
    let normal_data = textureLoad(g_normal, tex_coords, 0);
    let albedo_data = textureLoad(g_albedo, tex_coords, 0);
    
    let world_pos = position_data.xyz;
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
    
    // 简单的方向光
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
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
    let Lo = (kD * albedo / PI + specular) * radiance * NdotL;
    
    // 环境光
    let ambient = vec3<f32>(0.03) * albedo;
    var color = ambient + Lo;
    
    // HDR色调映射
    color = color / (color + vec3<f32>(1.0));
    // Gamma校正
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, 1.0);
}
