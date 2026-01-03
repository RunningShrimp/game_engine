// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    // Light
    light_direction: vec3<f32>,
    light_color: vec3<f32>,
    ambient_color: vec3<f32>,
    ambient_strength: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = uniforms.model * vec4<f32>(model.position, 1.0);
    output.world_pos = world_pos.xyz;
    output.clip_position = uniforms.proj * uniforms.view * world_pos;

    // Transform normal to world space
    output.normal = normalize((uniforms.model * vec4<f32>(model.normal, 0.0)).xyz);
    output.uv = model.uv;

    return output;
}

// Fragment shader
@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    // Base color (can be textured later)
    let base_color = vec3<f32>(0.8, 0.3, 0.3);

    // Ambient lighting
    let ambient = uniforms.ambient_color * uniforms.ambient_strength;

    // Diffuse lighting
    let light_dir = normalize(uniforms.light_direction);
    let diffuse_strength = max(dot(in.normal, light_dir), 0.0);
    let diffuse = diffuse_strength * uniforms.light_color;

    // Combine lighting
    let lighting = ambient + diffuse;

    // Final color
    let final_color = base_color * lighting;

    return vec4<f32>(final_color, 1.0);
}

// Grid shader (for visual reference)
struct GridVertexInput {
    @location(0) position: vec3<f32>,
}

struct GridVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

@vertex
fn grid_vs(
    model: GridVertexInput,
) -> GridVertexOutput {
    var output: GridVertexOutput;
    output.world_pos = model.position;
    output.clip_position = uniforms.proj * uniforms.view * vec4<f32>(model.position, 1.0);
    return output;
}

@fragment
fn grid_fs(
    in: GridVertexOutput,
) -> @location(0) vec4<f32> {
    // Create grid pattern
    let grid_size = 10.0;
    let grid_spacing = 1.0;

    let x = abs(in.world_pos.x);
    let z = abs(in.world_pos.z);

    let dx = abs(fract(x / grid_spacing - 0.5) - 0.5) / fwidth(x);
    let dz = abs(fract(z / grid_spacing - 0.5) - 0.5) / fwidth(z);

    let line_alpha = max(min(dx, dz), 0.0);
    let line_color = vec3<f32>(0.5, 0.5, 0.5);

    // Fade out at edges
    let dist = length(in.world_pos.xz);
    let fade = 1.0 - smoothstep(0.0, grid_size * 0.5, dist);

    let alpha = line_alpha * fade * 0.3;

    return vec4<f32>(line_color, alpha);
}
