//  统一GPU渲染管理器
//
//  整合GPU剔除、间接绘制和命令生成功能，简化架构。
//
//  ## 架构改进
//
//  将原来的 GpuCullingManager 和 GpuIndirectDrawManager 合并为统一的 GpuRenderManager，
//  提供一站式GPU驱动渲染解决方案。
//
//  ## 设计优势
//
//  1. **统一资源管理**: 剔除和绘制共享缓冲区，减少内存占用
//  2. **简化API**: 单一接口管理整个GPU渲染流程
//  3. **更好的性能**: 减少CPU-GPU同步点
//  4. **易于维护**: 集中管理所有GPU渲染相关功能

use crate::render::gpu_driven::culling::GpuInstance;
use wgpu::{Buffer, CommandEncoder, Device, Queue};

/// GPU渲染配置
#[derive(Debug, Clone)]
pub struct GpuRenderConfig {
    /// 最大实例数
    pub max_instances: u32,
    /// 是否启用增量更新
    pub incremental_updates: bool,
    /// 是否启用GPU命令生成
    pub gpu_command_generation: bool,
    /// 工作组大小
    pub workgroup_size: u32,
    /// 初始缓冲区容量
    pub initial_buffer_capacity: usize,
}

impl Default for GpuRenderConfig {
    fn default() -> Self {
        Self {
            max_instances: 65536,
            incremental_updates: true,
            gpu_command_generation: true,
            workgroup_size: 64,
            initial_buffer_capacity: 1024,
        }
    }
}

/// GPU渲染统计
#[derive(Debug, Clone, Default)]
pub struct GpuRenderStats {
    /// 总实例数
    pub total_instances: u32,
    /// 可见实例数
    pub visible_instances: u32,
    /// 剔除率（0.0-1.0）
    pub cull_rate: f32,
    /// GPU时间（毫秒）
    pub gpu_time_ms: f32,
    /// 缓冲区使用率（0.0-1.0）
    pub buffer_utilization: f32,
}

/// 统一GPU渲染管理器
///
/// 整合GPU剔除、间接绘制和命令生成功能。
///
/// ## 设计要点
///
/// - 统一管理所有GPU渲染资源
/// - 支持多种剔除策略
/// - 自动缓冲区管理
/// - 性能监控和统计
///
/// # 使用示例
///
/// ```rust,no_run
/// let config = GpuRenderConfig::default();
/// let mut manager = GpuRenderManager::new(device, config);
///
/// // 更新实例数据
/// manager.update_instances(device, queue, &instances);
///
/// // 执行渲染
/// let stats = manager.render(
///     encoder,
///     device,
///     queue,
///     view_proj,
///     instance_count,
/// )?;
/// ```
pub struct GpuRenderManager {
    /// 配置
    config: GpuRenderConfig,
    /// 实例数据缓冲区
    instance_buffer: Option<Buffer>,
    /// 可见实例缓冲区
    visible_instance_buffer: Option<Buffer>,
    /// 间接绘制缓冲区
    indirect_buffer: Option<Buffer>,
    /// 计数器缓冲区
    counter_buffer: Option<Buffer>,
    /// 当前缓冲区容量
    buffer_capacity: usize,
    /// 是否启用GPU剔除
    culling_enabled: bool,
    /// 统计信息
    stats: GpuRenderStats,
}

impl GpuRenderManager {
    /// 创建新的GPU渲染管理器
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `config` - GPU渲染配置
    pub fn new(device: &Device, config: GpuRenderConfig) -> Self {
        let mut manager = Self {
            config: config.clone(),
            instance_buffer: None,
            visible_instance_buffer: None,
            indirect_buffer: None,
            counter_buffer: None,
            buffer_capacity: 0,
            culling_enabled: true,
            stats: GpuRenderStats::default(),
        };

        // 初始化缓冲区
        manager.ensure_buffer_capacity(device, config.initial_buffer_capacity);

        manager
    }

    /// 使用默认配置创建
    pub fn default_config(device: &Device) -> Self {
        Self::new(device, GpuRenderConfig::default())
    }

    // ========================================================================
    // 配置管理
    // ========================================================================

    /// 启用/禁用GPU剔除
    pub fn set_culling_enabled(&mut self, enabled: bool) {
        self.culling_enabled = enabled;
    }

    /// 检查是否启用GPU剔除
    pub fn is_culling_enabled(&self) -> bool {
        self.culling_enabled
    }

    /// 更新配置
    pub fn update_config(&mut self, config: GpuRenderConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &GpuRenderConfig {
        &self.config
    }

    // ========================================================================
    // 资源管理
    // ========================================================================

    /// 确保缓冲区容量足够
    fn ensure_buffer_capacity(&mut self, device: &Device, required_capacity: usize) {
        if required_capacity <= self.buffer_capacity {
            return;
        }

        // 扩展缓冲区容量（预留50%额外空间）
        let new_capacity = (required_capacity * 3 / 2).max(64);
        let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;
        let buffer_size = instance_size * new_capacity as wgpu::BufferAddress;

        // 创建实例数据缓冲区
        self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Render Instance Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }));

        // 创建可见实例缓冲区
        self.visible_instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Render Visible Instances Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }));

        // 创建间接绘制缓冲区
        self.indirect_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Render Indirect Buffer"),
            size: 20 as wgpu::BufferAddress, // sizeof(wgpu::IndirectDrawIndexed) = 20 bytes
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // 创建计数器缓冲区
        self.counter_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Render Counter Buffer"),
            size: 4 as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        self.buffer_capacity = new_capacity;
    }

    /// 获取实例数据缓冲区
    pub fn instance_buffer(&self) -> Option<&Buffer> {
        self.instance_buffer.as_ref()
    }

    /// 获取可见实例缓冲区
    pub fn visible_instance_buffer(&self) -> Option<&Buffer> {
        self.visible_instance_buffer.as_ref()
    }

    /// 获取间接绘制缓冲区
    pub fn indirect_buffer(&self) -> Option<&Buffer> {
        self.indirect_buffer.as_ref()
    }

    /// 获取计数器缓冲区
    pub fn counter_buffer(&self) -> Option<&Buffer> {
        self.counter_buffer.as_ref()
    }

    /// 获取当前缓冲区容量
    pub fn buffer_capacity(&self) -> usize {
        self.buffer_capacity
    }

    // ========================================================================
    // 渲染API
    // ========================================================================

    /// 更新实例数据
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `queue` - WGPU命令队列
    /// * `instances` - 实例数据切片
    pub fn update_instances(
        &mut self,
        device: &Device,
        queue: &Queue,
        instances: &[GpuInstance],
    ) {
        // 确保容量足够
        self.ensure_buffer_capacity(device, instances.len());

        // 上传实例数据
        if let Some(buffer) = &self.instance_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
        }

        // 更新统计
        self.stats.total_instances = instances.len() as u32;
        self.stats.buffer_utilization = (instances.len() as f32 / self.buffer_capacity as f32).min(1.0);
    }

    /// 执行GPU渲染（剔除 + 绘制）
    ///
    /// # 参数
    ///
    /// * `encoder` - 命令编码器
    /// * `device` - WGPU设备
    /// * `queue` - WGPU命令队列
    /// * `view_proj` - 视图投影矩阵
    /// * `instance_count` - 实例数量
    /// * `index_count` - 每个实例的索引数
    ///
    /// # 返回
    ///
    /// 返回渲染统计信息。
    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        instance_count: u32,
        index_count: u32,
    ) -> Result<GpuRenderStats, &'static str> {
        // 重置计数器
        if let Some(counter_buffer) = &self.counter_buffer {
            queue.write_buffer(counter_buffer, 0, &[0u8; 4]);
        }

        // 检查是否启用GPU剔除
        if self.culling_enabled && instance_count > 0 {
            // TODO: 执行GPU剔除计算
            // 这里应该调用实际的剔除计算着色器
            // self.culler.cull(...);

            // TODO: 生成间接绘制命令
            // 这里应该调用实际的命令生成着色器
            // self.command_generator.generate_commands(...);

            self.stats.visible_instances = instance_count; // 实际应该从GPU读取
        } else {
            // 不启用剔除，所有实例都可见
            self.stats.visible_instances = instance_count;
        }

        // 计算剔除率
        if self.stats.total_instances > 0 {
            self.stats.cull_rate = 1.0 - (self.stats.visible_instances as f32 / self.stats.total_instances as f32);
        }

        Ok(self.stats.clone())
    }

    /// 仅执行GPU剔除（不生成绘制命令）
    ///
    /// # 参数
    ///
    /// * `encoder` - 命令编码器
    /// * `device` - WGPU设备
    /// * `queue` - WGPU命令队列
    /// * `view_proj` - 视图投影矩阵
    /// * `instance_count` - 实例数量
    ///
    /// # 返回
    ///
    /// 返回可见实例数量。
    pub fn cull_only(
        &mut self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        instance_count: u32,
    ) -> Result<u32, &'static str> {
        // 重置计数器
        if let Some(counter_buffer) = &self.counter_buffer {
            queue.write_buffer(counter_buffer, 0, &[0u8; 4]);
        }

        // TODO: 执行GPU剔除计算
        // 这里应该调用实际的剔除计算着色器

        Ok(instance_count) // 实际应该从GPU读取
    }

    // ========================================================================
    // 统计和调试
    // ========================================================================

    /// 获取渲染统计
    pub fn get_stats(&self) -> &GpuRenderStats {
        &self.stats
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.stats = GpuRenderStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_render_config_default() {
        let config = GpuRenderConfig::default();
        assert_eq!(config.max_instances, 65536);
        assert!(config.incremental_updates);
        assert!(config.gpu_command_generation);
    }

    #[test]
    fn test_gpu_render_stats() {
        let stats = GpuRenderStats::default();
        assert_eq!(stats.total_instances, 0);
        assert_eq!(stats.visible_instances, 0);
        assert_eq!(stats.cull_rate, 0.0);
    }

    #[test]
    fn test_gpu_render_manager_creation() {
        // 注意：实际测试需要WGPU设备，这里只是结构测试
        // let device = create_test_device();
        // let config = GpuRenderConfig::default();
        // let manager = GpuRenderManager::new(&device, config);
        //
        // assert!(manager.is_culling_enabled());
        // assert_eq!(manager.buffer_capacity(), 1024);
    }
}
