// GPU流体力计算着色器（SPH）

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

// SPH压力梯度核函数（Spiky）
fn spiky_gradient(r: vec3<f32>, h: f32) -> vec3<f32> {
    let r_len = length(r);
    if (r_len >= h || r_len < 0.001) {
        return vec3<f32>(0.0);
    }
    let term = h - r_len;
    let coeff = -45.0 / (3.14159 * pow(h, 6.0)) * term * term;
    return coeff * normalize(r);
}

// SPH粘性拉普拉斯核函数（Viscosity）
fn viscosity_laplacian(r: vec3<f32>, h: f32) -> f32 {
    let r_len = length(r);
    if (r_len >= h || r_len < 0.001) {
        return 0.0;
    }
    return 45.0 / (3.14159 * pow(h, 6.0)) * (h - r_len);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.particle_count) {
        return;
    }

    var p = particles[idx];
    var pressure_force = vec3<f32>(0.0);
    var viscosity_force = vec3<f32>(0.0);
    let h = params.smoothing_radius;
    
    // 计算压力力和粘性力
    for (var i = 0u; i < params.particle_count; i++) {
        if (i == idx) {
            continue;
        }
        
        let other = particles[i];
        let delta = p.position - other.position;
        let r_len = length(delta);
        
        if (r_len < h && r_len > 0.001) {
            // 压力力（压力梯度）
            let pressure_gradient = spiky_gradient(delta, h);
            let pressure_term = (p.pressure + other.pressure) / (2.0 * other.density);
            pressure_force -= pressure_gradient * pressure_term * other.mass;
            
            // 粘性力（拉普拉斯）
            let viscosity_laplacian = viscosity_laplacian(delta, h);
            let velocity_diff = other.velocity - p.velocity;
            viscosity_force += velocity_diff * viscosity_laplacian * other.mass / other.density * params.viscosity;
        }
    }
    
    // 总力 = 压力力 + 粘性力 + 重力
    var total_force = pressure_force + viscosity_force + params.gravity * p.mass;
    
    // 更新速度（临时存储，实际更新在update阶段）
    // 这里简化实现，实际应该存储力并在update阶段应用
    p.velocity += total_force / p.mass * params.delta_time;
    
    particles[idx] = p;
}

