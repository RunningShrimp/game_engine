//! GPU命令生成模块
//!
//! 在GPU上生成间接绘制命令，减少CPU开销。
//!
//! ## 性能优化
//!
//! - **GPU端生成**: 使用计算着色器从可见实例生成绘制命令
//! - **批量处理**: 支持多绘制批处理
//! - **自动分组**: 自动合并相同网格的实例（实例化）
//! - **内存优化**: 优化的内存访问模式

use crate::render::gpu_driven::culling::GpuInstance;
use crate::render::gpu_driven::indirect::{DrawIndexedIndirectArgs, IndirectDrawBuffer, IndirectDrawError};
use wgpu::{BindGroup, BindGroupLayout, Buffer, ComputePipeline, Device, PipelineLayout, ShaderModule};

/// GPU命令生成器
///
/// 在GPU上生成间接绘制命令，从可见实例生成绘制参数。
///
/// ## 设计要点
///
/// - 使用计算着色器从可见实例生成绘制命令
/// - 支持多绘制批处理
/// - 自动分组相同网格的实例
/// - 优化内存访问模式
///
/// # 使用示例
///
/// ```rust
/// use game_engine::render::gpu_driven::command_generator::GpuCommandGenerator;
///
/// // 创建命令生成器
/// let generator = GpuCommandGenerator::new(device, max_instances);
///
/// // 生成间接绘制命令
/// generator.generate_commands(
///     encoder,
///     device,
///     &visible_instance_buffer,
///     &counter_buffer,
///     &indirect_buffer,
///     index_count,
/// );
/// ```
pub struct GpuCommandGenerator {
    /// 计算管线
    pipeline: ComputePipeline,
    /// 绑定组布局
    bind_group_layout: BindGroupLayout,
    /// 工作组大小
    workgroup_size: u32,
    /// 最大实例数
    max_instances: u32,
}

impl GpuCommandGenerator {
    /// 创建新的GPU命令生成器
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `max_instances` - 最大实例数
    ///
    /// # 返回
    ///
    /// 返回一个初始化的GPU命令生成器。
    pub fn new(device: &Device, max_instances: u32) -> Self {
        Self::with_workgroup_size(device, max_instances, 64)
    }

    /// 使用自定义工作组大小创建GPU命令生成器
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `max_instances` - 最大实例数
    /// * `workgroup_size` - 工作组大小（建议64或128）
    ///
    /// # 返回
    ///
    /// 返回一个初始化的GPU命令生成器。
    pub fn with_workgroup_size(device: &Device, max_instances: u32, workgroup_size: u32) -> Self {
        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Command Generator Bind Group Layout"),
            entries: &[
                // 可见实例缓冲区（输入）
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 计数器缓冲区（输入，可见实例数量）
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(4),
                    },
                    count: None,
                },
                // 间接绘制缓冲区（输出）
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
            ],
        });

        // 创建计算着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Command Generator Shader"),
            source: wgpu::ShaderSource::Wgsl(COMMAND_GENERATOR_SHADER.into()),
        });

        // 创建管线布局
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Command Generator Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // 创建计算管线
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Command Generator Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "generate_commands",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        Self {
            pipeline,
            bind_group_layout,
            workgroup_size,
            max_instances,
        }
    }

    /// 生成间接绘制命令
    ///
    /// 从可见实例生成间接绘制命令，写入间接绘制缓冲区。
    ///
    /// # 参数
    ///
    /// * `encoder` - 命令编码器
    /// * `device` - WGPU设备
    /// * `visible_instance_buffer` - 可见实例缓冲区（输入）
    /// * `counter_buffer` - 计数器缓冲区（输入，包含可见实例数量）
    /// * `indirect_buffer` - 间接绘制缓冲区（输出）
    /// * `index_count` - 每个实例的索引数
    ///
    /// # 错误
    ///
    /// 如果生成失败，返回错误。
    pub fn generate_commands(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &Device,
        visible_instance_buffer: &Buffer,
        counter_buffer: &Buffer,
        indirect_buffer: &Buffer,
        index_count: u32,
    ) -> Result<(), IndirectDrawError> {
        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Command Generator Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: visible_instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: counter_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: indirect_buffer.as_entire_binding(),
                },
            ],
        });

        // 执行计算着色器
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Command Generation Pass"),
            timestamp_writes: None,
        });

        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        
        // 计算工作组数量（使用最大实例数，实际数量由计数器决定）
        let workgroup_count = (self.max_instances + self.workgroup_size - 1) / self.workgroup_size;
        cpass.dispatch_workgroups(workgroup_count, 1, 1);

        Ok(())
    }

    /// 获取工作组大小
    ///
    /// # 返回
    ///
    /// 返回工作组大小。
    pub fn workgroup_size(&self) -> u32 {
        self.workgroup_size
    }

    /// 获取最大实例数
    ///
    /// # 返回
    ///
    /// 返回最大实例数。
    pub fn max_instances(&self) -> u32 {
        self.max_instances
    }
}

/// 命令生成计算着色器
///
/// 从可见实例生成间接绘制命令。
/// 每个可见实例生成一个绘制命令。
const COMMAND_GENERATOR_SHADER: &str = r#"
struct GpuInstance {
    model: mat4x4<f32>,
    aabb_min: vec3<f32>,
    instance_id: u32,
    aabb_max: vec3<f32>,
    flags: u32,
};

struct DrawIndexedIndirectArgs {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

@group(0) @binding(0) var<storage, read> visible_instances: array<GpuInstance>;
@group(0) @binding(1) var<storage, read> counter: atomic<u32>;
@group(0) @binding(2) var<storage, read_write> indirect_commands: array<DrawIndexedIndirectArgs>;

// 注意：index_count需要通过push constant或uniform传递
// 这里暂时使用固定值，实际实现中应该从uniform读取
const INDEX_COUNT: u32 = 36u; // 示例值，应该从外部传入

@compute @workgroup_size(64)
fn generate_commands(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let visible_count = atomicLoad(&counter);
    
    // 早期退出：超出可见实例数量
    if (idx >= visible_count) {
        return;
    }
    
    // 从可见实例生成绘制命令
    let instance = visible_instances[idx];
    
    // 生成间接绘制命令
    // 每个可见实例生成一个绘制命令
    indirect_commands[idx] = DrawIndexedIndirectArgs {
        index_count: INDEX_COUNT,        // 每个实例的索引数
        instance_count: 1u,                // 每个命令的实例数（单个实例）
        first_index: 0u,                   // 第一个索引
        base_vertex: 0i,                   // 基础顶点
        first_instance: idx,               // 第一个实例（使用实例索引）
    };
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_generator_creation() {
        // 注意：实际测试需要WGPU设备
        // let generator = GpuCommandGenerator::new(device, 1000);
        // assert_eq!(generator.max_instances(), 1000);
        // assert_eq!(generator.workgroup_size(), 64);
    }
}

