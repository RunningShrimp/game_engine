// GPU粒子更新计算着色器

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
    let dt = params.delta_time;
    
    // 更新速度（使用Verlet积分或Euler积分）
    // 简化的Euler积分
    if (p.mass > 0.0) {
        let acceleration = p.force / p.mass;
        p.velocity += acceleration * dt;
        
        // 应用阻尼
        let damping = 0.99;
        p.velocity *= damping;
    }
    
    // 更新位置
    p.position += p.velocity * dt;
    
    // 边界碰撞（可选）
    let boundary_min = vec3<f32>(-10.0, -10.0, -10.0);
    let boundary_max = vec3<f32>(10.0, 10.0, 10.0);
    
    if (p.position.x < boundary_min.x) {
        p.position.x = boundary_min.x;
        p.velocity.x *= -0.8;
    } else if (p.position.x > boundary_max.x) {
        p.position.x = boundary_max.x;
        p.velocity.x *= -0.8;
    }
    
    if (p.position.y < boundary_min.y) {
        p.position.y = boundary_min.y;
        p.velocity.y *= -0.8;
    } else if (p.position.y > boundary_max.y) {
        p.position.y = boundary_max.y;
        p.velocity.y *= -0.8;
    }
    
    if (p.position.z < boundary_min.z) {
        p.position.z = boundary_min.z;
        p.velocity.z *= -0.8;
    } else if (p.position.z > boundary_max.z) {
        p.position.z = boundary_max.z;
        p.velocity.z *= -0.8;
    }
    
    // 重置力
    p.force = vec3<f32>(0.0);
    
    particles[idx] = p;
}

