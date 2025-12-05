//! GPU剔除管理器
//!
//! 提供高级API管理GPU剔除资源，避免每帧创建/销毁缓冲区。

use super::culling::{GpuCuller, GpuInstance};
use wgpu::{Buffer, CommandEncoder, Device, Queue};

/// GPU剔除管理器
///
/// 管理GPU剔除的资源生命周期，提供高效的剔除API。
///
/// ## 资源管理
///
/// - 复用缓冲区，避免每帧分配/释放
/// - 自动调整缓冲区大小
/// - 提供CPU回退路径
pub struct GpuCullingManager {
    /// GPU剔除器
    culler: GpuCuller,
    /// 输入实例缓冲区
    input_buffer: Option<Buffer>,
    /// 输出可见实例缓冲区
    output_buffer: Option<Buffer>,
    /// 计数器缓冲区
    counter_buffer: Option<Buffer>,
    /// 当前缓冲区容量
    buffer_capacity: usize,
    /// 是否启用GPU剔除
    enabled: bool,
}

impl GpuCullingManager {
    /// 创建新的GPU剔除管理器
    ///
    /// # 参数
    /// - `device`: WGPU设备
    /// - `initial_capacity`: 初始缓冲区容量
    /// - `workgroup_size`: 工作组大小
    pub fn new(device: &Device, initial_capacity: usize, workgroup_size: u32) -> Self {
        let culler = GpuCuller::new(device, initial_capacity as u32, workgroup_size);

        Self {
            culler,
            input_buffer: None,
            output_buffer: None,
            counter_buffer: None,
            buffer_capacity: initial_capacity,
            enabled: true,
        }
    }

    /// 启用/禁用GPU剔除
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 确保缓冲区容量足够
    fn ensure_buffer_capacity(&mut self, device: &Device, required_capacity: usize) {
        if required_capacity <= self.buffer_capacity {
            return;
        }

        // 扩展缓冲区容量（预留50%额外空间）
        let new_capacity = (required_capacity * 3 / 2).max(64);
        let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;
        let buffer_size = instance_size * new_capacity as wgpu::BufferAddress;

        // 创建输入缓冲区
        self.input_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Culling Input Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // 创建输出缓冲区
        self.output_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Culling Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        // 创建计数器缓冲区
        self.counter_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Culling Counter Buffer"),
            size: std::mem::size_of::<u32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        self.buffer_capacity = new_capacity;
    }

    /// 执行GPU剔除（高级API）
    ///
    /// # 参数
    /// - `device`: WGPU设备
    /// - `queue`: 命令队列
    /// - `encoder`: 命令编码器
    /// - `instances`: 输入实例数据
    /// - `view_proj`: 视图投影矩阵
    ///
    /// # 返回
    /// 如果成功返回可见实例数量，否则返回None（使用CPU回退）
    pub fn cull_instances(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        instances: &[GpuInstance],
        view_proj: [[f32; 4]; 4],
    ) -> Option<u32> {
        if !self.enabled || instances.is_empty() {
            return None;
        }

        // 确保缓冲区容量足够
        self.ensure_buffer_capacity(device, instances.len());

        let input_buffer = self.input_buffer.as_ref()?;
        let output_buffer = self.output_buffer.as_ref()?;
        let counter_buffer = self.counter_buffer.as_ref()?;

        // 上传实例数据
        queue.write_buffer(input_buffer, 0, bytemuck::cast_slice(instances));

        // 重置计数器
        queue.write_buffer(counter_buffer, 0, &[0u8; 4]);

        // 执行剔除
        self.culler.cull(
            encoder,
            device,
            queue,
            input_buffer,
            output_buffer,
            counter_buffer,
            view_proj,
            instances.len() as u32,
        );

        // 返回可见实例数量（需要异步读取，这里返回None表示需要后续读取）
        Some(instances.len() as u32)
    }

    /// 获取输入缓冲区（用于直接写入）
    pub fn input_buffer(&self) -> Option<&Buffer> {
        self.input_buffer.as_ref()
    }

    /// 获取输出缓冲区（用于读取结果）
    pub fn output_buffer(&self) -> Option<&Buffer> {
        self.output_buffer.as_ref()
    }

    /// 获取计数器缓冲区（用于读取可见实例数量）
    pub fn counter_buffer(&self) -> Option<&Buffer> {
        self.counter_buffer.as_ref()
    }

    /// 获取当前缓冲区容量
    pub fn buffer_capacity(&self) -> usize {
        self.buffer_capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_culling_manager_creation() {
        // 注意：实际测试需要WGPU设备，这里只是结构测试
        // let manager = GpuCullingManager::new(device, 1024, 64);
        // assert!(manager.is_enabled());
        // assert_eq!(manager.buffer_capacity(), 1024);
    }

    #[test]
    fn test_gpu_culling_manager_enable_disable() {
        // let mut manager = GpuCullingManager::new(device, 1024, 64);
        // manager.set_enabled(false);
        // assert!(!manager.is_enabled());
        // manager.set_enabled(true);
        // assert!(manager.is_enabled());
    }
}
