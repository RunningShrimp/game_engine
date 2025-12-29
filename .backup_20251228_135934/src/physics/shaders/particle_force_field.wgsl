// GPU粒子力场计算着色器

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    force: vec3<f32>,
    mass: f32,
    radius: f32,
    _padding: vec2<f32>,
}

struct ForceField {
    position: vec3<f32>,
    strength: f32,
    radius: f32,
    field_type: u32, // 0 = 引力, 1 = 斥力, 2 = 涡流
    direction: vec3<f32>,
    _padding: f32,
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
var<storage, read> force_fields: array<ForceField>;

@group(0) @binding(2)
var<uniform> params: ParticlePhysicsParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.particle_count) {
        return;
    }

    var p = particles[idx];
    var total_force = vec3<f32>(0.0);
    
    // 应用重力
    total_force += params.gravity * p.mass;
    
    // 计算力场影响
    let num_fields = arrayLength(&force_fields);
    for (var i = 0u; i < num_fields; i++) {
        let field = force_fields[i];
        let delta = field.position - p.position;
        let dist_sq = dot(delta, delta);
        let dist = sqrt(dist_sq);
        
        if (dist < field.radius && dist > 0.001) {
            let normalized_delta = delta / dist;
            var force = vec3<f32>(0.0);
            
            if (field.field_type == 0u) {
                // 引力
                let force_magnitude = field.strength / (dist_sq + 0.01);
                force = normalized_delta * force_magnitude;
            } else if (field.field_type == 1u) {
                // 斥力
                let force_magnitude = -field.strength / (dist_sq + 0.01);
                force = normalized_delta * force_magnitude;
            } else if (field.field_type == 2u) {
                // 涡流
                let tangent = cross(normalized_delta, field.direction);
                let force_magnitude = field.strength / (dist + 0.1);
                force = tangent * force_magnitude;
            }
            
            total_force += force * p.mass;
        }
    }
    
    // 粒子间相互作用（简化实现）
    for (var i = 0u; i < params.particle_count; i++) {
        if (i == idx) {
            continue;
        }
        
        let other = particles[i];
        let delta = p.position - other.position;
        let dist_sq = dot(delta, delta);
        
        if (dist_sq < params.interaction_radius * params.interaction_radius && dist_sq > 0.001) {
            let dist = sqrt(dist_sq);
            let normalized_delta = delta / dist;
            
            // 简化的相互作用力（斥力）
            let interaction_strength = 0.1;
            let force_magnitude = interaction_strength / (dist_sq + 0.01);
            total_force += normalized_delta * force_magnitude * p.mass;
        }
    }
    
    // 更新力
    p.force = total_force;
    particles[idx] = p;
}

