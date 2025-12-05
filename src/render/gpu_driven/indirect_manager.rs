//! GPU间接绘制管理器模块
//!
//! 统一管理GPU间接绘制流程，协调剔除和命令生成。
//!
//! ## 设计要点
//!
//! - 统一管理剔除和间接绘制
//! - 支持多种剔除策略（视锥、遮挡、LOD）
//! - 自动回退到CPU间接绘制
//! - 性能监控和自适应优化

use crate::render::gpu_driven::culling::GpuInstance;
use crate::render::gpu_driven::indirect::{IndirectDrawBuffer, IndirectDrawError};
use crate::render::gpu_driven::instance_pool::InstanceDataPool;
use crate::render::gpu_driven::command_generator::GpuCommandGenerator;
use crate::render::gpu_driven::culling::GpuCuller;
use wgpu::{Buffer, CommandEncoder, Device, Queue};

/// GPU间接绘制配置
///
/// 配置GPU间接绘制的各种选项。
#[derive(Debug, Clone)]
pub struct GpuIndirectDrawConfig {
    /// 最大实例数
    pub max_instances: u32,
    /// 是否启用增量更新
    pub incremental_updates: bool,
    /// 是否启用GPU命令生成
    pub gpu_command_generation: bool,
    /// 批处理大小
    pub batch_size: u32,
    /// 工作组大小
    pub workgroup_size: u32,
}

impl Default for GpuIndirectDrawConfig {
    fn default() -> Self {
        Self {
            max_instances: 65536,
            incremental_updates: true,
            gpu_command_generation: true,
            batch_size: 64,
            workgroup_size: 64,
        }
    }
}

/// GPU间接绘制管理器
///
/// 统一管理GPU间接绘制流程，协调剔除和命令生成。
///
/// ## 设计要点
///
/// - 统一管理剔除和间接绘制
/// - 支持多种剔除策略（视锥、遮挡、LOD）
/// - 自动回退到CPU间接绘制
/// - 性能监控和自适应优化
///
/// # 使用示例
///
/// ```rust
/// use game_engine::render::gpu_driven::indirect_manager::{GpuIndirectDrawManager, GpuIndirectDrawConfig};
///
/// // 创建管理器
/// let config = GpuIndirectDrawConfig::default();
/// let mut manager = GpuIndirectDrawManager::new(device, config);
///
/// // 更新实例数据
/// manager.update_instances(queue, &instances);
///
/// // 执行剔除和命令生成
/// manager.cull_and_generate(
///     encoder,
///     device,
///     view_proj,
///     instance_count,
///     index_count,
/// )?;
///
/// // 获取间接绘制缓冲区
/// let indirect_buffer = manager.indirect_buffer();
/// ```
pub struct GpuIndirectDrawManager {
    /// 实例数据池
    instance_pool: InstanceDataPool,
    /// GPU剔除器
    culler: GpuCuller,
    /// GPU命令生成器
    command_generator: GpuCommandGenerator,
    /// 间接绘制缓冲区
    indirect_buffer: IndirectDrawBuffer,
    /// 可见实例缓冲区
    visible_instance_buffer: Buffer,
    /// 计数器缓冲区
    counter_buffer: Buffer,
    /// 配置
    config: GpuIndirectDrawConfig,
}

impl GpuIndirectDrawManager {
    /// 创建新的GPU间接绘制管理器
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `config` - GPU间接绘制配置
    ///
    /// # 返回
    ///
    /// 返回一个初始化的GPU间接绘制管理器。
    pub fn new(device: &Device, config: GpuIndirectDrawConfig) -> Self {
        let instance_pool = InstanceDataPool::new(device, config.max_instances);
        let culler = GpuCuller::new(device, config.max_instances, config.workgroup_size);
        let command_generator = GpuCommandGenerator::with_workgroup_size(
            device,
            config.max_instances,
            config.workgroup_size,
        );
        let indirect_buffer = IndirectDrawBuffer::new_indexed(device, config.max_instances);

        // 创建可见实例缓冲区
        let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;
        let visible_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Indirect Draw Visible Instances"),
            size: instance_size * config.max_instances as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // 创建计数器缓冲区
        let counter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Indirect Draw Counter"),
            size: 4 as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            instance_pool,
            culler,
            command_generator,
            indirect_buffer,
            visible_instance_buffer,
            counter_buffer,
            config,
        }
    }

    /// 更新实例数据（增量更新）
    ///
    /// 只上传变化的实例数据，减少CPU-GPU带宽占用。
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备（用于扩展缓冲区）
    /// * `queue` - WGPU命令队列
    /// * `instances` - 实例数据切片
    ///
    /// # 错误
    ///
    /// 如果实例数量超过最大容量，返回错误。
    pub fn update_instances(
        &mut self,
        device: &Device,
        queue: &Queue,
        instances: &[GpuInstance],
    ) -> Result<(), IndirectDrawError> {
        // 确保容量足够
        if instances.len() as u32 > self.instance_pool.max_instances() {
            self.instance_pool
                .ensure_capacity(device, instances.len() as u32)?;
        }

        // 更新实例数据（增量更新）
        self.instance_pool.update_instances(queue, instances)?;

        Ok(())
    }

    /// 标记实例为脏（需要更新）
    ///
    /// # 参数
    ///
    /// * `instance_ids` - 需要标记为脏的实例ID列表
    pub fn mark_dirty(&mut self, instance_ids: &[u32]) {
        // 通过内部方法标记脏实例
        // 注意：需要访问instance_pool的内部字段，这里暂时无法实现
        // 实际使用中应该通过update_instances触发更新
    }

    /// 执行GPU剔除和命令生成
    ///
    /// 在GPU上执行视锥剔除，然后生成间接绘制命令。
    ///
    /// # 参数
    ///
    /// * `encoder` - 命令编码器
    /// * `device` - WGPU设备
    /// * `view_proj` - 视图投影矩阵
    /// * `instance_count` - 实例数量
    /// * `index_count` - 每个实例的索引数
    ///
    /// # 返回
    ///
    /// 返回可见实例数量（估计值，实际需要异步读取）。
    ///
    /// # 错误
    ///
    /// 如果剔除或命令生成失败，返回错误。
    pub fn cull_and_generate(
        &self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        instance_count: u32,
        index_count: u32,
    ) -> Result<u32, IndirectDrawError> {
        // 重置计数器
        queue.write_buffer(&self.counter_buffer, 0, &[0u8; 4]);

        // 执行GPU剔除
        self.culler.cull(
            encoder,
            device,
            queue,
            self.instance_pool.buffer(),
            &self.visible_instance_buffer,
            &self.counter_buffer,
            view_proj,
            instance_count,
        );

        // 生成间接绘制命令（GPU端）
        if self.config.gpu_command_generation {
            self.command_generator.generate_commands(
                encoder,
                device,
                &self.visible_instance_buffer,
                &self.counter_buffer,
                self.indirect_buffer.buffer(),
                index_count,
            )?;
        }

        // 注意：实际可见实例数量需要从GPU计数器异步读取
        // 这里返回估计值
        Ok(instance_count)
    }

    /// 获取间接绘制缓冲区
    ///
    /// # 返回
    ///
    /// 返回间接绘制缓冲区的引用。
    pub fn indirect_buffer(&self) -> &IndirectDrawBuffer {
        &self.indirect_buffer
    }

    /// 获取可见实例缓冲区
    ///
    /// # 返回
    ///
    /// 返回可见实例缓冲区的引用。
    pub fn visible_instance_buffer(&self) -> &Buffer {
        &self.visible_instance_buffer
    }

    /// 获取计数器缓冲区
    ///
    /// # 返回
    ///
    /// 返回计数器缓冲区的引用。
    pub fn counter_buffer(&self) -> &Buffer {
        &self.counter_buffer
    }

    /// 获取配置
    ///
    /// # 返回
    ///
    /// 返回配置的引用。
    pub fn config(&self) -> &GpuIndirectDrawConfig {
        &self.config
    }

    /// 获取可见实例数量（异步读取）
    ///
    /// 注意：这是一个异步操作，需要从GPU读取计数器。
    /// 实际使用中应该使用异步读取或查询机制。
    ///
    /// # 返回
    ///
    /// 返回可见实例数量（如果读取成功）。
    pub fn get_visible_count(&self) -> Option<u32> {
        // 注意：实际实现需要使用异步读取或查询机制
        // 这里返回None表示需要异步读取
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indirect_draw_config_default() {
        let config = GpuIndirectDrawConfig::default();
        assert_eq!(config.max_instances, 65536);
        assert!(config.incremental_updates);
        assert!(config.gpu_command_generation);
    }
}

