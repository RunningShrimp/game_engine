// GPU流体压力计算着色器（SPH）

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
    
    // 计算压力（使用状态方程：P = k * (ρ - ρ0)）
    // 其中 k 是压力常数，ρ 是密度，ρ0 是静止密度
    let density_diff = p.density - params.rest_density;
    p.pressure = params.pressure_constant * max(density_diff, 0.0);
    
    particles[idx] = p;
}

