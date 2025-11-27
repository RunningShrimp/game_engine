//! 粒子渲染 Shader
//!
//! 使用 Billboard 技术渲染粒子，支持软粒子和深度淡出。

// ============================================================================
// 数据结构
// ============================================================================

struct Particle {
    position: vec3<f32>,
    lifetime: f32,
    velocity: vec3<f32>,
    age: f32,
    color: vec4<f32>,
    size: f32,
    rotation: f32,
    rotation_speed: f32,
    alive: f32,
};

struct CameraUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    camera_position: vec3<f32>,
    _padding: f32,
    camera_right: vec3<f32>,
    _padding2: f32,
    camera_up: vec3<f32>,
    near: f32,
    far: f32,
    _padding3: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec3<f32>,
};

// ============================================================================
// Bindings
// ============================================================================

@group(0) @binding(0) var<storage, read> particles: array<Particle>;
@group(0) @binding(1) var<storage, read> alive_list: array<u32>;
@group(0) @binding(2) var<uniform> camera: CameraUniforms;

@group(1) @binding(0) var particle_texture: texture_2d<f32>;
@group(1) @binding(1) var particle_sampler: sampler;

// 深度纹理（用于软粒子）
@group(2) @binding(0) var depth_texture: texture_depth_2d;

// ============================================================================
// Vertex Shader
// ============================================================================

// 四边形顶点偏移（逆时针）
const QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-0.5, -0.5), // 左下
    vec2<f32>( 0.5, -0.5), // 右下
    vec2<f32>( 0.5,  0.5), // 右上
    vec2<f32>(-0.5, -0.5), // 左下
    vec2<f32>( 0.5,  0.5), // 右上
    vec2<f32>(-0.5,  0.5), // 左上
);

const QUAD_UVS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 0.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    // 获取粒子
    let particle_idx = alive_list[instance_index];
    let p = particles[particle_idx];
    
    var out: VertexOutput;
    
    // 死亡粒子不渲染
    if (p.alive < 0.5) {
        out.position = vec4<f32>(0.0, 0.0, -1000.0, 1.0);
        out.uv = vec2<f32>(0.0);
        out.color = vec4<f32>(0.0);
        out.world_position = vec3<f32>(0.0);
        return out;
    }
    
    // 获取四边形顶点
    let quad_vertex = QUAD_VERTICES[vertex_index % 6u];
    let quad_uv = QUAD_UVS[vertex_index % 6u];
    
    // 应用旋转
    let cos_r = cos(p.rotation);
    let sin_r = sin(p.rotation);
    let rotated = vec2<f32>(
        quad_vertex.x * cos_r - quad_vertex.y * sin_r,
        quad_vertex.x * sin_r + quad_vertex.y * cos_r
    );
    
    // Billboard: 始终面向相机
    let right = camera.camera_right;
    let up = camera.camera_up;
    
    // 计算世界空间位置
    let world_offset = (right * rotated.x + up * rotated.y) * p.size;
    let world_pos = p.position + world_offset;
    
    // 变换到裁剪空间
    out.position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = quad_uv;
    out.color = p.color;
    out.world_position = world_pos;
    
    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样纹理
    let tex_color = textureSample(particle_texture, particle_sampler, in.uv);
    
    // 混合颜色
    var color = in.color * tex_color;
    
    // 软粒子（基于深度的淡出）
    // 获取屏幕空间坐标
    let screen_pos = in.position.xy;
    let particle_depth = in.position.z;
    
    // 采样场景深度
    let scene_depth = textureLoad(depth_texture, vec2<i32>(screen_pos), 0);
    
    // 计算深度差异
    let depth_diff = scene_depth - particle_depth;
    let soft_factor = saturate(depth_diff * 10.0); // 调整软粒子范围
    
    color.a *= soft_factor;
    
    // 丢弃完全透明的像素
    if (color.a < 0.01) {
        discard;
    }
    
    return color;
}

// ============================================================================
// 简化版 Fragment Shader（无软粒子）
// ============================================================================

@fragment
fn fs_simple(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样纹理
    let tex_color = textureSample(particle_texture, particle_sampler, in.uv);
    
    // 混合颜色
    var color = in.color * tex_color;
    
    // 圆形粒子（如果没有纹理）
    let dist = length(in.uv - vec2<f32>(0.5));
    let circle_alpha = 1.0 - smoothstep(0.4, 0.5, dist);
    color.a *= circle_alpha;
    
    // 丢弃完全透明的像素
    if (color.a < 0.01) {
        discard;
    }
    
    return color;
}

// ============================================================================
// Point Sprite 版本
// ============================================================================

struct PointVertexOutput {
    @builtin(position) position: vec4<f32>,
    @builtin(point_size) point_size: f32,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_point(
    @builtin(instance_index) instance_index: u32,
) -> PointVertexOutput {
    let particle_idx = alive_list[instance_index];
    let p = particles[particle_idx];
    
    var out: PointVertexOutput;
    
    if (p.alive < 0.5) {
        out.position = vec4<f32>(0.0, 0.0, -1000.0, 1.0);
        out.point_size = 0.0;
        out.color = vec4<f32>(0.0);
        return out;
    }
    
    out.position = camera.view_proj * vec4<f32>(p.position, 1.0);
    
    // 根据距离计算点大小
    let dist = length(camera.camera_position - p.position);
    out.point_size = p.size * 500.0 / max(dist, 1.0);
    
    out.color = p.color;
    
    return out;
}

@fragment
fn fs_point(
    in: PointVertexOutput,
    @builtin(point_coord) point_coord: vec2<f32>,
) -> @location(0) vec4<f32> {
    // 圆形点
    let dist = length(point_coord - vec2<f32>(0.5));
    if (dist > 0.5) {
        discard;
    }
    
    let alpha = 1.0 - smoothstep(0.3, 0.5, dist);
    var color = in.color;
    color.a *= alpha;
    
    return color;
}

// ============================================================================
// Additive Blend 版本
// ============================================================================

@fragment
fn fs_additive(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(particle_texture, particle_sampler, in.uv);
    var color = in.color * tex_color;
    
    // Additive blending: 输出 RGB 作为加法混合
    // 需要在 Pipeline 中设置 blend mode 为 Additive
    color.rgb *= color.a; // 预乘 alpha
    
    return color;
}
