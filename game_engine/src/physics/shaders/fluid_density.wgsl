// GPU流体密度计算着色器（SPH）

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

// SPH平滑核函数（Poly6）
fn poly6_kernel(r_sq: f32, h: f32) -> f32 {
    let h_sq = h * h;
    if (r_sq >= h_sq || r_sq < 0.0) {
        return 0.0;
    }
    let term = h_sq - r_sq;
    return 315.0 / (64.0 * 3.14159 * pow(h, 9.0)) * term * term * term;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.particle_count) {
        return;
    }

    var p = particles[idx];
    var density = 0.0;
    let h = params.smoothing_radius;
    
    // 计算密度（SPH密度公式）
    for (var i = 0u; i < params.particle_count; i++) {
        let other = particles[i];
        let delta = p.position - other.position;
        let r_sq = dot(delta, delta);
        
        if (r_sq < h * h) {
            density += other.mass * poly6_kernel(r_sq, h);
        }
    }
    
    p.density = density;
    particles[idx] = p;
}

