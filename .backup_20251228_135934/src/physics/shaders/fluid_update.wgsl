// GPU流体更新着色器（SPH）

struct FluidParticle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    density: f32,
    pressure: f32,
    mass: f32,
    _padding: vec2<f32>,
}

struct FluidSimulationParams {
    particle_count: u32,
    delta_time: f32,
    smoothing_radius: f32,
    rest_density: f32,
    pressure_constant: f32,
    viscosity: f32,
    surface_tension: f32,
    gravity: vec3<f32>,
    _padding: vec2<u32>,
}

@group(0) @binding(0)
var<storage, read_write> particles: array<FluidParticle>;

@group(0) @binding(1)
var<uniform> params: FluidSimulationParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.particle_count) {
        return;
    }

    var p = particles[idx];
    let dt = params.delta_time;
    
    // 更新位置
    p.position += p.velocity * dt;
    
    // 边界碰撞（可选）
    let boundary_min = vec3<f32>(-10.0, -10.0, -10.0);
    let boundary_max = vec3<f32>(10.0, 10.0, 10.0);
    
    if (p.position.x < boundary_min.x) {
        p.position.x = boundary_min.x;
        p.velocity.x *= -0.5;
    } else if (p.position.x > boundary_max.x) {
        p.position.x = boundary_max.x;
        p.velocity.x *= -0.5;
    }
    
    if (p.position.y < boundary_min.y) {
        p.position.y = boundary_min.y;
        p.velocity.y *= -0.5;
    } else if (p.position.y > boundary_max.y) {
        p.position.y = boundary_max.y;
        p.velocity.y *= -0.5;
    }
    
    if (p.position.z < boundary_min.z) {
        p.position.z = boundary_min.z;
        p.velocity.z *= -0.5;
    } else if (p.position.z > boundary_max.z) {
        p.position.z = boundary_max.z;
        p.velocity.z *= -0.5;
    }
    
    particles[idx] = p;
}

