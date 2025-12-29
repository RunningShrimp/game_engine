// GPU粒子碰撞检测计算着色器

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    force: vec3<f32>,
    mass: f32,
    radius: f32,
    _padding: vec2<f32>,
}

struct ParticlePhysicsParams {
    particle_count: u32,
    delta_time: f32,
    collision_radius: f32,
    interaction_radius: f32,
    gravity: vec3<f32>,
    _padding: vec2<u32>,
}

@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> params: ParticlePhysicsParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.particle_count) {
        return;
    }

    var p = particles[idx];
    
    // 检查与其他粒子的碰撞
    for (var i = idx + 1u; i < params.particle_count; i++) {
        let other = particles[i];
        
        let delta = p.position - other.position;
        let dist_sq = dot(delta, delta);
        let min_dist = p.radius + other.radius;
        let min_dist_sq = min_dist * min_dist;
        
        if (dist_sq < min_dist_sq && dist_sq > 0.0) {
            let dist = sqrt(dist_sq);
            let normal = delta / dist;
            let overlap = min_dist - dist;
            
            // 碰撞响应（弹性碰撞）
            let relative_velocity = p.velocity - other.velocity;
            let velocity_along_normal = dot(relative_velocity, normal);
            
            // 不处理分离的粒子
            if (velocity_along_normal > 0.0) {
                continue;
            }
            
            // 计算恢复系数
            let restitution = 0.8;
            let impulse_magnitude = -(1.0 + restitution) * velocity_along_normal;
            impulse_magnitude /= p.mass + other.mass;
            
            let impulse = impulse_magnitude * normal;
            
            // 应用冲量
            p.velocity += impulse / p.mass;
            
            // 分离粒子
            p.position += normal * overlap * 0.5;
        }
    }
    
    particles[idx] = p;
}

