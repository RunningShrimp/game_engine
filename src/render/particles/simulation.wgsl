//! 粒子模拟 Compute Shader
//!
//! 在 GPU 上执行粒子物理模拟，支持百万级粒子实时更新。

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

struct Uniforms {
    emitter_position: vec3<f32>,
    delta_time: f32,
    gravity: vec3<f32>,
    drag: f32,
    emit_count: u32,
    time: f32,
    random_seed: f32,
    _padding: f32,
};

struct Counters {
    alive_count: atomic<u32>,
    dead_count: atomic<u32>,
    emit_count: atomic<u32>,
    dispatch_count: atomic<u32>,
};

// ============================================================================
// Bindings
// ============================================================================

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> alive_list: array<u32>;
@group(0) @binding(2) var<storage, read_write> dead_list: array<u32>;
@group(0) @binding(3) var<storage, read_write> counters: Counters;
@group(0) @binding(4) var<uniform> uniforms: Uniforms;

// ============================================================================
// 随机数生成
// ============================================================================

// PCG 随机数生成器状态
var<private> rng_state: u32;

fn pcg_hash(input: u32) -> u32 {
    let state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn init_random(seed: u32, id: u32) {
    rng_state = pcg_hash(seed + id * 1000000u);
}

fn random() -> f32 {
    rng_state = pcg_hash(rng_state);
    return f32(rng_state) / 4294967295.0;
}

fn random_range(min_val: f32, max_val: f32) -> f32 {
    return min_val + random() * (max_val - min_val);
}

fn random_vec3(min_val: vec3<f32>, max_val: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        random_range(min_val.x, max_val.x),
        random_range(min_val.y, max_val.y),
        random_range(min_val.z, max_val.z)
    );
}

// 生成球面上的随机点
fn random_on_sphere() -> vec3<f32> {
    let theta = random() * 6.28318530718;
    let phi = acos(2.0 * random() - 1.0);
    let sin_phi = sin(phi);
    return vec3<f32>(
        sin_phi * cos(theta),
        sin_phi * sin(theta),
        cos(phi)
    );
}

// 生成圆锥内的随机方向
fn random_in_cone(direction: vec3<f32>, angle: f32) -> vec3<f32> {
    let cos_angle = cos(angle);
    let z = random_range(cos_angle, 1.0);
    let phi = random() * 6.28318530718;
    let sin_theta = sqrt(1.0 - z * z);
    
    let local_dir = vec3<f32>(
        sin_theta * cos(phi),
        sin_theta * sin(phi),
        z
    );
    
    // NOTE: 旋转到目标方向的逻辑待实现，当前返回局部方向
    return local_dir;
}

// ============================================================================
// 颜色插值
// ============================================================================

fn lerp_color(start: vec4<f32>, end: vec4<f32>, t: f32) -> vec4<f32> {
    return mix(start, end, t);
}

// ============================================================================
// 发射 Shader
// ============================================================================

@compute @workgroup_size(64)
fn emit(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    // 检查是否需要发射
    if (idx >= uniforms.emit_count) {
        return;
    }
    
    // 从死亡列表获取粒子索引
    let dead_count = atomicLoad(&counters.dead_count);
    if (dead_count == 0u) {
        return;
    }
    
    // 原子减少死亡计数并获取索引
    let dead_idx = atomicSub(&counters.dead_count, 1u) - 1u;
    if (dead_idx >= arrayLength(&dead_list)) {
        atomicAdd(&counters.dead_count, 1u);
        return;
    }
    
    let particle_idx = dead_list[dead_idx];
    
    // 初始化随机数
    init_random(bitcast<u32>(uniforms.random_seed * 1000000.0), idx);
    
    // 初始化粒子
    var p: Particle;
    p.position = uniforms.emitter_position + random_vec3(vec3<f32>(-0.1), vec3<f32>(0.1));
    p.velocity = random_vec3(vec3<f32>(-1.0, 2.0, -1.0), vec3<f32>(1.0, 5.0, 1.0));
    p.lifetime = random_range(1.0, 3.0);
    p.age = 0.0;
    p.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    p.size = random_range(0.1, 0.3);
    p.rotation = random() * 6.28318530718;
    p.rotation_speed = random_range(-1.0, 1.0);
    p.alive = 1.0;
    
    particles[particle_idx] = p;
    
    // 添加到存活列表
    let alive_idx = atomicAdd(&counters.alive_count, 1u);
    alive_list[alive_idx] = particle_idx;
}

// ============================================================================
// 更新 Shader
// ============================================================================

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let alive_count = atomicLoad(&counters.alive_count);
    
    if (idx >= alive_count) {
        return;
    }
    
    let particle_idx = alive_list[idx];
    var p = particles[particle_idx];
    
    // 跳过死亡粒子
    if (p.alive < 0.5) {
        return;
    }
    
    // 更新年龄
    p.age += uniforms.delta_time;
    
    // 检查是否死亡
    if (p.age >= p.lifetime) {
        p.alive = 0.0;
        particles[particle_idx] = p;
        
        // 添加到死亡列表
        let dead_idx = atomicAdd(&counters.dead_count, 1u);
        dead_list[dead_idx] = particle_idx;
        
        return;
    }
    
    // 物理更新
    let dt = uniforms.delta_time;
    
    // 重力
    p.velocity += uniforms.gravity * dt;
    
    // 阻力
    p.velocity *= 1.0 - uniforms.drag * dt;
    
    // 位置更新
    p.position += p.velocity * dt;
    
    // 旋转更新
    p.rotation += p.rotation_speed * dt;
    
    // 计算生命周期比例
    let life_ratio = p.age / p.lifetime;
    
    // 颜色随生命周期变化（从白色到透明）
    p.color = lerp_color(
        vec4<f32>(1.0, 1.0, 1.0, 1.0),
        vec4<f32>(1.0, 0.5, 0.0, 0.0),
        life_ratio
    );
    
    // 大小随生命周期变化
    p.size = mix(0.3, 0.05, life_ratio);
    
    particles[particle_idx] = p;
}

// ============================================================================
// 紧凑存活列表（可选，用于优化渲染）
// ============================================================================

@compute @workgroup_size(64)
fn compact_alive_list(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let alive_count = atomicLoad(&counters.alive_count);
    
    if (idx >= alive_count) {
        return;
    }
    
    let particle_idx = alive_list[idx];
    let p = particles[particle_idx];
    
    // 如果粒子已死亡，需要从存活列表移除
    // 注：这是一个简化版本，完整实现需要 prefix sum
    if (p.alive < 0.5) {
        // 原子减少存活计数
        atomicSub(&counters.alive_count, 1u);
    }
}
