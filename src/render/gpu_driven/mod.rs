//! GPU Driven Rendering 模块
//!
//! 实现基于 GPU 的高性能渲染技术：
//! - 计算着色器剔除（Compute Shader Culling）
//! - 间接绘制（Indirect Drawing）
//! - 层次化场景剔除（BVH/八叉树）
//!
//! ## 性能预期
//! - 预计性能提升 30-50%（取决于场景复杂度）
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    GPU Driven Pipeline                   │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. Upload Instance Data                                 │
//! │     - 所有实例数据上传到 GPU 存储缓冲区                    │
//! │                                                          │
//! │  2. Frustum Culling (Compute Shader)                     │
//! │     - 计算着色器并行检测每个实例的视锥剔除                  │
//! │     - 输出可见实例索引到间接缓冲区                         │
//! │                                                          │
//! │  3. Indirect Draw                                        │
//! │     - 使用 DrawIndirect 命令                              │
//! │     - GPU 自动确定绘制数量                                 │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod culling;
pub mod indirect;

pub use culling::{GpuCuller, CullingUniforms};
pub use indirect::{IndirectDrawBuffer, DrawIndirectArgs};

use wgpu::util::DeviceExt;

/// GPU Driven 渲染配置
#[derive(Debug, Clone)]
pub struct GpuDrivenConfig {
    /// 是否启用视锥剔除
    pub frustum_culling: bool,
    /// 是否启用遮挡剔除
    pub occlusion_culling: bool,
    /// 是否启用 LOD
    pub lod_enabled: bool,
    /// 最大实例数
    pub max_instances: u32,
    /// 计算着色器工作组大小
    pub workgroup_size: u32,
}

impl Default for GpuDrivenConfig {
    fn default() -> Self {
        Self {
            frustum_culling: true,
            occlusion_culling: false,
            lod_enabled: false,
            max_instances: 65536,
            workgroup_size: 64,
        }
    }
}

/// GPU Driven 渲染器
pub struct GpuDrivenRenderer {
    /// 配置
    config: GpuDrivenConfig,
    /// GPU 剔除器
    culler: GpuCuller,
    /// 间接绘制缓冲区
    indirect_buffer: IndirectDrawBuffer,
    /// 实例输入缓冲区
    instance_input_buffer: wgpu::Buffer,
    /// 可见实例输出缓冲区
    visible_instance_buffer: wgpu::Buffer,
    /// 计数器缓冲区
    counter_buffer: wgpu::Buffer,
    /// 是否已初始化
    initialized: bool,
}

impl GpuDrivenRenderer {
    /// 创建 GPU Driven 渲染器
    pub fn new(device: &wgpu::Device, config: GpuDrivenConfig) -> Self {
        let culler = GpuCuller::new(device, config.max_instances, config.workgroup_size);
        let indirect_buffer = IndirectDrawBuffer::new(device, config.max_instances);
        
        // 创建实例缓冲区
        let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;
        let instance_input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Driven Instance Input"),
            size: instance_size * config.max_instances as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let visible_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Driven Visible Instances"),
            size: instance_size * config.max_instances as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let counter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Driven Counter"),
            size: 4 as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        Self {
            config,
            culler,
            indirect_buffer,
            instance_input_buffer,
            visible_instance_buffer,
            counter_buffer,
            initialized: true,
        }
    }
    
    /// 更新实例数据
    pub fn update_instances(&self, queue: &wgpu::Queue, instances: &[GpuInstance]) {
        if instances.is_empty() {
            return;
        }
        queue.write_buffer(&self.instance_input_buffer, 0, bytemuck::cast_slice(instances));
    }
    
    /// 执行 GPU 剔除
    pub fn cull(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_proj: [[f32; 4]; 4],
        instance_count: u32,
    ) {
        if !self.config.frustum_culling {
            return;
        }
        
        // 重置计数器
        queue.write_buffer(&self.counter_buffer, 0, &[0u8; 4]);
        
        // 执行剔除
        self.culler.cull(
            encoder,
            device,
            &self.instance_input_buffer,
            &self.visible_instance_buffer,
            &self.counter_buffer,
            view_proj,
            instance_count,
        );
    }
    
    /// 获取可见实例缓冲区
    pub fn visible_instance_buffer(&self) -> &wgpu::Buffer {
        &self.visible_instance_buffer
    }
    
    /// 获取间接绘制缓冲区
    pub fn indirect_buffer(&self) -> &IndirectDrawBuffer {
        &self.indirect_buffer
    }
    
    /// 获取配置
    pub fn config(&self) -> &GpuDrivenConfig {
        &self.config
    }
}

/// GPU 实例数据（用于剔除）
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuInstance {
    /// 模型矩阵
    pub model: [[f32; 4]; 4],
    /// 包围盒最小点
    pub aabb_min: [f32; 3],
    /// 实例 ID
    pub instance_id: u32,
    /// 包围盒最大点
    pub aabb_max: [f32; 3],
    /// 标志位
    pub flags: u32,
}

impl Default for GpuInstance {
    fn default() -> Self {
        Self {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            aabb_min: [-0.5, -0.5, -0.5],
            instance_id: 0,
            aabb_max: [0.5, 0.5, 0.5],
            flags: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_driven_config_default() {
        let config = GpuDrivenConfig::default();
        assert!(config.frustum_culling);
        assert!(!config.occlusion_culling);
        assert_eq!(config.max_instances, 65536);
    }
    
    #[test]
    fn test_gpu_instance_default() {
        let instance = GpuInstance::default();
        assert_eq!(instance.instance_id, 0);
        assert_eq!(instance.aabb_min, [-0.5, -0.5, -0.5]);
        assert_eq!(instance.aabb_max, [0.5, 0.5, 0.5]);
    }
}

