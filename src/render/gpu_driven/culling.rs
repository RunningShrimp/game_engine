//! GPU 剔除模块
//!
//! 实现基于计算着色器的视锥剔除。

/// 剔除 Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CullingUniforms {
    /// 视图投影矩阵
    pub view_proj: [[f32; 4]; 4],
    /// 视锥平面 (6个平面，每个4个float: nx, ny, nz, d)
    pub frustum_planes: [[f32; 4]; 6],
    /// 实例总数
    pub instance_count: u32,
    /// 填充
    pub _pad: [u32; 3],
}

impl Default for CullingUniforms {
    fn default() -> Self {
        Self {
            view_proj: [[0.0; 4]; 4],
            frustum_planes: [[0.0; 4]; 6],
            instance_count: 0,
            _pad: [0; 3],
        }
    }
}

impl CullingUniforms {
    /// 从视图投影矩阵提取视锥平面
    pub fn from_view_proj(view_proj: [[f32; 4]; 4], instance_count: u32) -> Self {
        let m = view_proj;
        
        // 提取视锥平面 (Gribb/Hartmann method)
        let planes = [
            // Left
            [m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0], m[3][3] + m[3][0]],
            // Right
            [m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0], m[3][3] - m[3][0]],
            // Bottom
            [m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1], m[3][3] + m[3][1]],
            // Top
            [m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1], m[3][3] - m[3][1]],
            // Near
            [m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2], m[3][3] + m[3][2]],
            // Far
            [m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2], m[3][3] - m[3][2]],
        ];
        
        // 归一化平面
        let mut frustum_planes = [[0.0f32; 4]; 6];
        for (i, plane) in planes.iter().enumerate() {
            let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            if len > 0.0001 {
                frustum_planes[i] = [
                    plane[0] / len,
                    plane[1] / len,
                    plane[2] / len,
                    plane[3] / len,
                ];
            }
        }
        
        Self {
            view_proj,
            frustum_planes,
            instance_count,
            _pad: [0; 3],
        }
    }
}

/// GPU 剔除器
pub struct GpuCuller {
    /// 剔除计算管线
    pipeline: wgpu::ComputePipeline,
    /// 绑定组布局
    bind_group_layout: wgpu::BindGroupLayout,
    /// Uniform 缓冲区
    uniform_buffer: wgpu::Buffer,
    /// 工作组大小
    workgroup_size: u32,
    /// 最大实例数
    max_instances: u32,
}

impl GpuCuller {
    /// 创建 GPU 剔除器
    pub fn new(device: &wgpu::Device, max_instances: u32, workgroup_size: u32) -> Self {
        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Culling BGL"),
            entries: &[
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 输入实例
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 输出可见实例
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 计数器
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // 创建计算着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Culling Shader"),
            source: wgpu::ShaderSource::Wgsl(CULLING_SHADER.into()),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Culling Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Culling Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cull_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });
        
        // 创建 Uniform 缓冲区
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Culling Uniforms"),
            size: std::mem::size_of::<CullingUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        Self {
            pipeline,
            bind_group_layout,
            uniform_buffer,
            workgroup_size,
            max_instances,
        }
    }
    
    /// 执行剔除
    pub fn cull(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input_buffer: &wgpu::Buffer,
        output_buffer: &wgpu::Buffer,
        counter_buffer: &wgpu::Buffer,
        view_proj: [[f32; 4]; 4],
        instance_count: u32,
    ) {
        let uniforms = CullingUniforms::from_view_proj(view_proj, instance_count);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        
        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Culling BG"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: counter_buffer.as_entire_binding(),
                },
            ],
        });
        
        // 执行计算着色器
        let workgroup_count = (instance_count + self.workgroup_size - 1) / self.workgroup_size;
        
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Culling Pass"),
            timestamp_writes: None,
        });
        
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(workgroup_count, 1, 1);
    }
}

/// 剔除计算着色器
const CULLING_SHADER: &str = r#"
struct CullingUniforms {
    view_proj: mat4x4<f32>,
    frustum_planes: array<vec4<f32>, 6>,
    instance_count: u32,
    _pad: vec3<u32>,
};

struct GpuInstance {
    model: mat4x4<f32>,
    aabb_min: vec3<f32>,
    instance_id: u32,
    aabb_max: vec3<f32>,
    flags: u32,
};

@group(0) @binding(0) var<uniform> uniforms: CullingUniforms;
@group(0) @binding(1) var<storage, read> input_instances: array<GpuInstance>;
@group(0) @binding(2) var<storage, read_write> output_instances: array<GpuInstance>;
@group(0) @binding(3) var<storage, read_write> counter: atomic<u32>;

// 检测 AABB 是否与视锥平面相交
fn aabb_vs_plane(aabb_min: vec3<f32>, aabb_max: vec3<f32>, plane: vec4<f32>) -> bool {
    // 计算 AABB 在平面法向量方向上的正顶点
    var p_vertex: vec3<f32>;
    if (plane.x >= 0.0) { p_vertex.x = aabb_max.x; } else { p_vertex.x = aabb_min.x; }
    if (plane.y >= 0.0) { p_vertex.y = aabb_max.y; } else { p_vertex.y = aabb_min.y; }
    if (plane.z >= 0.0) { p_vertex.z = aabb_max.z; } else { p_vertex.z = aabb_min.z; }
    
    // 检测正顶点是否在平面外侧
    return dot(vec3<f32>(plane.x, plane.y, plane.z), p_vertex) + plane.w >= 0.0;
}

// 检测 AABB 是否在视锥内
fn is_visible(aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> bool {
    for (var i = 0u; i < 6u; i++) {
        if (!aabb_vs_plane(aabb_min, aabb_max, uniforms.frustum_planes[i])) {
            return false;
        }
    }
    return true;
}

@compute @workgroup_size(64)
fn cull_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    if (idx >= uniforms.instance_count) {
        return;
    }
    
    let instance = input_instances[idx];
    
    // 将 AABB 变换到世界空间
    let world_min = (instance.model * vec4<f32>(instance.aabb_min, 1.0)).xyz;
    let world_max = (instance.model * vec4<f32>(instance.aabb_max, 1.0)).xyz;
    
    // 确保 min < max
    let actual_min = min(world_min, world_max);
    let actual_max = max(world_min, world_max);
    
    // 视锥剔除
    if (is_visible(actual_min, actual_max)) {
        // 原子增加计数器并获取输出索引
        let output_idx = atomicAdd(&counter, 1u);
        output_instances[output_idx] = instance;
    }
}
"#;
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuInstance {
    pub model: [[f32; 4]; 4],
    pub aabb_min: [f32; 3],
    pub instance_id: u32,
    pub aabb_max: [f32; 3],
    pub flags: u32,
}
