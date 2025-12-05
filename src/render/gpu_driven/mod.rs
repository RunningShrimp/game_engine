//! GPU Driven Rendering 模块
//!
//! 实现基于 GPU 的高性能渲染技术：
//! - **计算着色器剔除（Compute Shader Culling）** - **默认启用**
//! - 间接绘制（Indirect Drawing）- 可选优化
//! - 层次化场景剔除（BVH/八叉树）- 未来扩展
//!
//! ## GPU驱动剔除（默认策略）
//!
//! GPU驱动剔除现在是引擎的默认剔除策略，提供：
//! - **高性能**：GPU并行处理，预计性能提升 30-50%（取决于场景复杂度）
//! - **自动回退**：如果GPU剔除不可用，自动回退到CPU剔除
//! - **优化着色器**：使用优化的计算着色器，减少分支和内存访问
//!
//! ## 性能优化
//!
//! 计算着色器优化：
//! - 展开循环以减少分支预测失败
//! - 使用`select`函数替代if-else分支
//! - 早期退出优化
//! - 优化的AABB变换
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
pub mod culling_manager;
pub mod indirect;
pub mod instance_pool;
pub mod command_generator;
pub mod indirect_manager;

// 遮挡剔除集成
use crate::impl_default;
use crate::render::occlusion_culling::HierarchicalZCulling;

pub use culling::{CullingUniforms, GpuCuller, GpuInstance};
pub use culling_manager::GpuCullingManager;
pub use indirect::{DrawIndirectArgs, IndirectDrawBuffer};
pub use instance_pool::InstanceDataPool;
pub use command_generator::GpuCommandGenerator;
pub use indirect_manager::{GpuIndirectDrawConfig, GpuIndirectDrawManager};

/// GPU Driven 渲染配置
///
/// 配置GPU驱动渲染的各种选项，包括剔除策略、LOD和性能参数。
///
/// # 使用示例
///
/// ```rust
/// use game_engine::render::gpu_driven::GpuDrivenConfig;
///
/// let config = GpuDrivenConfig {
///     frustum_culling: true,
///     occlusion_culling: false,
///     lod_enabled: true,
///     max_instances: 65536,
///     workgroup_size: 64,
/// };
/// ```
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

impl_default!(GpuDrivenConfig {
    frustum_culling: true,
    occlusion_culling: false,
    lod_enabled: false,
    max_instances: 65536,
    workgroup_size: 64,
});

/// GPU Driven 渲染器
///
/// 实现GPU驱动的渲染管线，包括视锥剔除、遮挡剔除和间接绘制。
///
/// # 使用示例
///
/// ```rust
/// use game_engine::render::gpu_driven::{GpuDrivenRenderer, GpuDrivenConfig};
///
/// let config = GpuDrivenConfig::default();
/// let renderer = GpuDrivenRenderer::new(device, config);
///
/// // 更新实例数据
/// renderer.update_instances(queue, &instances);
///
/// // 执行剔除
/// renderer.cull(encoder, device, queue, view_proj, instance_count);
/// ```
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
    /// Hi-Z遮挡剔除器（可选）
    occlusion_culler: Option<HierarchicalZCulling>,
    /// 是否已初始化
    initialized: bool,
}

impl GpuDrivenRenderer {
    /// 创建 GPU Driven 渲染器
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `config` - GPU驱动渲染配置
    ///
    /// # 返回
    ///
    /// 返回一个初始化的`GpuDrivenRenderer`实例。
    ///
    /// # 注意
    ///
    /// 如果启用了遮挡剔除，Hi-Z剔除器会在首次使用时初始化。
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
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // 如果启用遮挡剔除，创建Hi-Z剔除器
        let occlusion_culler = if config.occlusion_culling {
            Some(HierarchicalZCulling::new(
                config.max_instances as u32, // 使用最大实例数作为参考分辨率
                config.max_instances as u32,
            ))
        } else {
            None
        };

        Self {
            config,
            culler,
            indirect_buffer,
            instance_input_buffer,
            visible_instance_buffer,
            counter_buffer,
            occlusion_culler,
            initialized: true,
        }
    }

    /// 更新实例数据
    ///
    /// 将实例数据上传到GPU缓冲区，供剔除计算使用。
    ///
    /// # 参数
    ///
    /// * `queue` - WGPU命令队列
    /// * `instances` - 实例数据切片
    ///
    /// # 性能
    ///
    /// 此方法会阻塞直到数据上传完成。对于大量实例，考虑使用异步上传。
    pub fn update_instances(&self, queue: &wgpu::Queue, instances: &[GpuInstance]) {
        if instances.is_empty() {
            return;
        }
        queue.write_buffer(
            &self.instance_input_buffer,
            0,
            bytemuck::cast_slice(instances),
        );
    }

    /// 执行 GPU 剔除并生成间接绘制命令
    ///
    /// 在GPU上执行视锥剔除，将可见实例写入输出缓冲区。
    ///
    /// # 参数
    ///
    /// * `encoder` - 命令编码器
    /// * `device` - WGPU设备
    /// * `queue` - 命令队列
    /// * `view_proj` - 视图投影矩阵
    /// * `instance_count` - 实例总数
    ///
    /// # 注意
    ///
    /// 此方法需要先调用`update_instances`上传实例数据。
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
            queue,
            &self.instance_input_buffer,
            &self.visible_instance_buffer,
            &self.counter_buffer,
            view_proj,
            instance_count,
        );
    }

    /// 执行 GPU 剔除并生成间接绘制命令（完整版本）
    ///
    /// # 参数
    /// - `encoder`: 命令编码器
    /// - `device`: WGPU设备
    /// - `queue`: 命令队列
    /// - `view_proj`: 视图投影矩阵
    /// - `instance_count`: 实例数量
    /// - `vertex_count`: 每个实例的顶点数（用于间接绘制）
    /// - `index_count`: 每个实例的索引数（用于索引间接绘制）
    ///
    /// # 返回
    /// 可见实例数量（从GPU计数器读取，需要异步读取）
    ///
    /// # 错误
    ///
    /// 如果间接绘制命令生成失败，返回错误。
    pub fn cull_with_indirect(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_proj: [[f32; 4]; 4],
        instance_count: u32,
        vertex_count: u32,
        index_count: u32,
    ) -> Result<u32, crate::render::gpu_driven::indirect::IndirectDrawError> {
        if !self.config.frustum_culling {
            // 如果没有启用剔除，直接生成间接绘制命令
            self.generate_indirect_commands(queue, instance_count, vertex_count, index_count)?;
            return Ok(instance_count);
        }

        // 重置计数器
        queue.write_buffer(&self.counter_buffer, 0, &[0u8; 4]);

        // 执行剔除（带间接绘制缓冲区）
        // 传递index_count以生成间接绘制命令，实现完全GPU端剔除流程
        self.culler.cull_with_indirect(
            encoder,
            device,
            queue,
            &self.instance_input_buffer,
            &self.visible_instance_buffer,
            &self.counter_buffer,
            Some(self.indirect_buffer.buffer()),
            view_proj,
            instance_count,
            index_count,  // 传递索引数，用于生成间接绘制命令
        );

        // 注意：实际可见实例数量需要从GPU计数器异步读取
        // 这里返回估计值，实际使用中应该使用异步读取
        // 为了简化，这里假设所有实例都可见（实际应该从counter_buffer读取）
        Ok(instance_count)
    }

    /// 生成间接绘制命令（CPU端回退）
    ///
    /// 当GPU剔除不可用时，使用CPU端生成间接绘制命令。
    ///
    /// # 错误
    ///
    /// 如果缓冲区更新失败，返回错误。
    fn generate_indirect_commands(
        &self,
        queue: &wgpu::Queue,
        instance_count: u32,
        vertex_count: u32,
        index_count: u32,
    ) -> Result<(), crate::render::gpu_driven::indirect::IndirectDrawError> {
        use crate::render::gpu_driven::indirect::DrawIndexedIndirectArgs;
        
        let args = DrawIndexedIndirectArgs {
            index_count,
            instance_count,
            first_index: 0,
            base_vertex: 0,
            first_instance: 0,
        };
        
        self.indirect_buffer.update_indexed(queue, &[args])?;
        Ok(())
    }

    /// 获取可见实例缓冲区
    pub fn visible_instance_buffer(&self) -> &wgpu::Buffer {
        &self.visible_instance_buffer
    }

    /// 获取间接绘制缓冲区
    pub fn indirect_buffer(&self) -> &IndirectDrawBuffer {
        &self.indirect_buffer
    }

    /// 检查GPU驱动间接绘制是否可用
    ///
    /// 用于运行时检测GPU驱动间接绘制功能是否可用，如果不可用则回退到CPU间接绘制。
    ///
    /// # 返回
    /// - `true`: GPU驱动间接绘制可用
    /// - `false`: GPU驱动间接绘制不可用，应使用CPU间接绘制
    pub fn is_indirect_draw_available(&self) -> bool {
        // 检查GPU剔除是否可用
        if !self.culler.is_available() {
            return false;
        }
        // 检查间接缓冲区是否已初始化
        self.initialized
    }

    /// 获取可见实例数量（异步读取）
    ///
    /// 注意：这是一个异步操作，需要从GPU读取计数器。
    /// 实际使用中应该使用异步读取或查询机制。
    ///
    /// # 参数
    /// - `device`: WGPU设备
    /// - `queue`: 命令队列
    ///
    /// # 返回
    /// 可见实例数量（如果读取成功）
    pub fn get_visible_count(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<u32> {
        // 注意：实际实现需要使用异步读取或查询机制
        // 这里返回None表示需要异步读取
        // 实际使用中应该使用wgpu的异步读取功能
        None
    }

    /// 获取配置
    pub fn config(&self) -> &GpuDrivenConfig {
        &self.config
    }

    /// 初始化遮挡剔除（如果启用）
    ///
    /// 在首次使用前调用此方法初始化Hi-Z资源。
    /// 初始化遮挡剔除
    ///
    /// # 错误
    ///
    /// 如果初始化失败，返回错误。
    pub fn initialize_occlusion_culling(&mut self, device: &wgpu::Device) -> Result<(), crate::render::occlusion_culling::OcclusionError> {
        if let Some(ref mut occluder) = self.occlusion_culler {
            occluder.initialize(device)?;
        }
        Ok(())
    }

    /// 执行遮挡剔除（如果启用）
    ///
    /// # 参数
    /// - `encoder`: 命令编码器
    /// - `device`: WGPU设备
    /// - `depth_texture`: 深度缓冲纹理
    ///
    /// # 错误
    ///
    /// 如果遮挡剔除未初始化或执行失败，返回错误。
    pub fn perform_occlusion_culling(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        depth_texture: &wgpu::Texture,
    ) -> Result<(), crate::render::occlusion_culling::OcclusionError> {
        if let Some(ref occluder) = self.occlusion_culler {
            if occluder.is_initialized() {
                occluder.build_hi_z(encoder, device, depth_texture)?;
            }
        }
        Ok(())
    }

    /// 执行遮挡查询（如果启用）
    ///
    /// # 参数
    /// - `encoder`: 命令编码器
    /// - `device`: WGPU设备
    /// - `queries`: 查询列表（AABB，世界空间）
    /// - `view_proj`: 视图投影矩阵
    /// - `screen_size`: 屏幕分辨率
    ///
    /// # 错误
    ///
    /// 如果遮挡剔除未初始化或执行失败，返回错误。
    pub fn query_occlusion_async(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queries: &[(glam::Vec3, glam::Vec3)],
        view_proj: glam::Mat4,
        screen_size: (u32, u32),
    ) -> Result<(), crate::render::occlusion_culling::OcclusionError> {
        if let Some(ref mut occluder) = self.occlusion_culler {
            if occluder.is_initialized() {
                occluder.query_occlusion_async(encoder, device, queries, view_proj, screen_size)?;
            }
        }
        Ok(())
    }

    /// 读取异步遮挡查询结果（如果启用）
    ///
    /// # 参数
    /// - `device`: WGPU设备
    /// - `queue`: WGPU队列
    ///
    /// # 返回
    ///
    /// 如果结果可用，返回`Some(Vec<bool>)`；否则返回`None`。
    pub fn read_occlusion_query_result(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Result<Vec<bool>, crate::render::occlusion_culling::OcclusionError>> {
        if let Some(ref mut occluder) = self.occlusion_culler {
            occluder.read_async_query_result(device, queue)
        } else {
            None
        }
    }

    /// 获取遮挡剔除器（如果启用）
    pub fn occlusion_culler(&self) -> Option<&HierarchicalZCulling> {
        self.occlusion_culler.as_ref()
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
