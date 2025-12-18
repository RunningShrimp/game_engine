// 延迟渲染 - 几何阶段着色器

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) albedo: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // 简化版本:假设模型矩阵是单位矩阵
    out.world_position = vertex.position;
    out.clip_position = vec4<f32>(vertex.position, 1.0);
    out.world_normal = normalize(vertex.normal);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    
    // 位置 + 深度
    out.position = vec4<f32>(in.world_position, in.clip_position.z);
    
    // 法线 + 粗糙度 (暂时硬编码粗糙度为0.5)
    out.normal = vec4<f32>(normalize(in.world_normal), 0.5);
    
    // 反照率 + 金属度 (暂时硬编码为白色,非金属)
    out.albedo = vec4<f32>(1.0, 1.0, 1.0, 0.0);
    
    return out;
}
