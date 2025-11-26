// CSM阴影渲染着色器

struct VertexInput {
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(vertex: VertexInput) -> @builtin(position) vec4<f32> {
    // 简化版本:假设光源视图投影矩阵是单位矩阵
    return vec4<f32>(vertex.position, 1.0);
}

// 深度渲染不需要片段着色器
