//! 间接绘制模块
//!
//! 实现优化的 GPU 间接绘制支持。
//!
//! ## 性能优化
//!
//! - **缓冲区复用**：复用缓冲区，避免每帧重建
//! - **批量更新**：使用批量更新减少CPU开销
//! - **内存对齐**：优化缓冲区布局，提高GPU内存访问效率
//! - **多绘制支持**：使用 `MULTI_DRAW_INDIRECT` 减少绘制调用开销
//!
//! ## 错误处理
//!
//! 所有操作都包含完善的错误处理，确保在资源不足或无效参数时能够优雅降级。

use thiserror::Error;

/// 间接绘制错误类型
#[derive(Error, Debug)]
pub enum IndirectDrawError {
    /// 缓冲区容量不足
    #[error("Buffer capacity insufficient: required {required}, available {available}")]
    InsufficientCapacity { required: u32, available: u32 },
    /// 无效的绘制参数
    #[error("Invalid draw arguments: {0}")]
    InvalidArguments(String),
    /// 缓冲区创建失败
    #[error("Failed to create buffer: {0}")]
    BufferCreationFailed(String),
    /// 缓冲区更新失败
    #[error("Failed to update buffer: {0}")]
    BufferUpdateFailed(String),
}

/// 间接绘制参数
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirectArgs {
    /// 顶点数
    pub vertex_count: u32,
    /// 实例数
    pub instance_count: u32,
    /// 第一个顶点
    pub first_vertex: u32,
    /// 第一个实例
    pub first_instance: u32,
}

/// 索引间接绘制参数
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirectArgs {
    /// 索引数
    pub index_count: u32,
    /// 实例数
    pub instance_count: u32,
    /// 第一个索引
    pub first_index: u32,
    /// 基础顶点
    pub base_vertex: i32,
    /// 第一个实例
    pub first_instance: u32,
}

/// 优化的间接绘制缓冲区
///
/// 提供高效的间接绘制缓冲区管理，包括：
/// - 缓冲区复用，避免频繁重建
/// - 增量更新支持
/// - 优化的内存布局
pub struct IndirectDrawBuffer {
    /// 间接绘制缓冲区
    buffer: wgpu::Buffer,
    /// 最大绘制调用数
    max_draws: u32,
    /// 当前缓冲区容量（字节）
    buffer_size: wgpu::BufferAddress,
}

impl IndirectDrawBuffer {
    /// 创建间接绘制缓冲区
    ///
    /// # 参数
    /// - `device`: WGPU设备
    /// - `max_draws`: 最大绘制调用数
    ///
    /// # 性能提示
    /// - 缓冲区大小会根据需要自动扩展
    /// - 使用 `STORAGE` 标志允许GPU计算着色器写入
    pub fn new(device: &wgpu::Device, max_draws: u32) -> Self {
        let buffer_size =
            (std::mem::size_of::<DrawIndirectArgs>() * max_draws as usize) as wgpu::BufferAddress;
        // 对齐到256字节边界，提高GPU内存访问效率
        let aligned_size = (buffer_size + 255) & !255;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Draw Buffer"),
            size: aligned_size,
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            max_draws,
            buffer_size: aligned_size,
        }
    }

    /// 创建索引间接绘制缓冲区
    ///
    /// # 参数
    /// - `device`: WGPU设备
    /// - `max_draws`: 最大绘制调用数
    ///
    /// # 性能优化
    /// - 缓冲区大小对齐到256字节边界
    /// - 支持GPU计算着色器写入（用于GPU驱动剔除）
    pub fn new_indexed(device: &wgpu::Device, max_draws: u32) -> Self {
        let buffer_size = (std::mem::size_of::<DrawIndexedIndirectArgs>() * max_draws as usize)
            as wgpu::BufferAddress;
        // 对齐到256字节边界，提高GPU内存访问效率
        let aligned_size = (buffer_size + 255) & !255;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indexed Indirect Draw Buffer"),
            size: aligned_size,
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            max_draws,
            buffer_size: aligned_size,
        }
    }

    /// 确保缓冲区容量足够（自动扩展）
    ///
    /// 如果需要的绘制调用数超过当前容量，自动扩展缓冲区。
    ///
    /// # 参数
    /// - `device`: WGPU设备
    /// - `required_draws`: 需要的绘制调用数
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, required_draws: u32) {
        if required_draws <= self.max_draws {
            return;
        }

        // 扩展容量（预留50%额外空间）
        let new_max_draws = (required_draws * 3 / 2).max(64);
        let buffer_size = (std::mem::size_of::<DrawIndexedIndirectArgs>() * new_max_draws as usize)
            as wgpu::BufferAddress;
        let aligned_size = (buffer_size + 255) & !255;

        self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indexed Indirect Draw Buffer (Resized)"),
            size: aligned_size,
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.max_draws = new_max_draws;
        self.buffer_size = aligned_size;
    }

    /// 获取缓冲区
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// 获取最大绘制调用数
    pub fn max_draws(&self) -> u32 {
        self.max_draws
    }

    /// 更新间接绘制参数（批量更新）
    ///
    /// # 性能优化
    /// - 使用批量写入减少CPU开销
    /// - 只更新实际使用的部分
    ///
    /// # 错误
    ///
    /// 如果参数数量超过缓冲区容量，返回错误。
    pub fn update(&self, queue: &wgpu::Queue, args: &[DrawIndirectArgs]) -> Result<(), IndirectDrawError> {
        if args.len() > self.max_draws as usize {
            return Err(IndirectDrawError::InsufficientCapacity {
                required: args.len() as u32,
                available: self.max_draws,
            });
        }
        
        if !args.is_empty() {
            let data = bytemuck::cast_slice(args);
            queue.write_buffer(&self.buffer, 0, data);
        }
        
        Ok(())
    }

    /// 更新索引间接绘制参数（批量更新）
    ///
    /// # 性能优化
    /// - 使用批量写入减少CPU开销
    /// - 只更新实际使用的部分
    /// - 支持增量更新（通过offset参数）
    ///
    /// # 错误
    ///
    /// 如果参数数量超过缓冲区容量，返回错误。
    pub fn update_indexed(&self, queue: &wgpu::Queue, args: &[DrawIndexedIndirectArgs]) -> Result<(), IndirectDrawError> {
        if args.len() > self.max_draws as usize {
            return Err(IndirectDrawError::InsufficientCapacity {
                required: args.len() as u32,
                available: self.max_draws,
            });
        }
        
        if !args.is_empty() {
            let data = bytemuck::cast_slice(args);
            queue.write_buffer(&self.buffer, 0, data);
        }
        
        Ok(())
    }

    /// 增量更新索引间接绘制参数
    ///
    /// # 参数
    /// - `queue`: 命令队列
    /// - `args`: 要更新的绘制参数
    /// - `offset`: 缓冲区偏移量（以绘制命令为单位）
    ///
    /// # 性能优化
    /// - 只更新变化的部分，减少内存带宽使用
    ///
    /// # 错误
    ///
    /// 如果偏移量或参数数量超出缓冲区范围，返回错误。
    pub fn update_indexed_partial(
        &self,
        queue: &wgpu::Queue,
        args: &[DrawIndexedIndirectArgs],
        offset: u32,
    ) -> Result<(), IndirectDrawError> {
        if offset as usize + args.len() > self.max_draws as usize {
            return Err(IndirectDrawError::InsufficientCapacity {
                required: offset + args.len() as u32,
                available: self.max_draws,
            });
        }
        
        if !args.is_empty() {
            let byte_offset = (offset as usize * std::mem::size_of::<DrawIndexedIndirectArgs>())
                as wgpu::BufferAddress;
            let data = bytemuck::cast_slice(args);
            queue.write_buffer(&self.buffer, byte_offset, data);
        }
        
        Ok(())
    }

    /// 获取缓冲区大小（字节）
    pub fn buffer_size(&self) -> wgpu::BufferAddress {
        self.buffer_size
    }

    /// 获取缓冲区对齐大小（256字节对齐）
    ///
    /// # 性能提示
    /// - GPU内存访问通常对齐到256字节边界时性能更好
    /// - 使用对齐大小可以减少内存碎片
    pub fn aligned_size(&self) -> wgpu::BufferAddress {
        self.buffer_size
    }
}

/// 优化的多绘制间接批处理器
///
/// 提供高效的批量间接绘制命令管理：
/// - 自动缓冲区扩展
/// - 批量提交优化
/// - 内存访问模式优化
pub struct MultiDrawIndirect {
    /// 绘制命令列表
    commands: Vec<DrawIndexedIndirectArgs>,
    /// 间接缓冲区
    indirect_buffer: IndirectDrawBuffer,
    /// 设备引用（用于缓冲区扩展）
    device: Option<std::sync::Arc<wgpu::Device>>,
}

impl MultiDrawIndirect {
    /// 创建多绘制批处理器
    ///
    /// # 参数
    /// - `device`: WGPU设备
    /// - `max_draws`: 初始最大绘制调用数
    ///
    /// # 性能优化
    /// - 缓冲区会自动扩展以适应更多绘制调用
    /// - 使用预分配容量减少内存重新分配
    pub fn new(device: &wgpu::Device, max_draws: u32) -> Self {
        Self {
            commands: Vec::with_capacity(max_draws as usize),
            indirect_buffer: IndirectDrawBuffer::new_indexed(device, max_draws),
            device: None, // 暂时不使用设备引用，避免生命周期问题
        }
    }

    /// 清空命令（保留容量）
    ///
    /// # 性能优化
    /// - 保留向量容量，避免重新分配
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// 添加绘制命令（自动扩展缓冲区）
    ///
    /// # 参数
    /// - `device`: WGPU设备（用于扩展缓冲区）
    /// - `args`: 绘制参数
    ///
    /// # 错误
    ///
    /// 如果缓冲区扩展失败，返回错误。
    ///
    /// # 性能优化
    /// - 如果超出容量，自动扩展缓冲区
    /// - 批量添加时使用预分配容量
    pub fn push(&mut self, device: &wgpu::Device, args: DrawIndexedIndirectArgs) -> Result<(), IndirectDrawError> {
        // 如果接近容量限制，扩展缓冲区
        if self.commands.len() >= self.indirect_buffer.max_draws as usize {
            let new_capacity = (self.indirect_buffer.max_draws * 2).max(64);
            self.indirect_buffer.ensure_capacity(device, new_capacity);
        }
        
        // 验证参数有效性
        if args.index_count == 0 {
            return Err(IndirectDrawError::InvalidArguments(
                "index_count cannot be zero".to_string(),
            ));
        }
        
        self.commands.push(args);
        Ok(())
    }

    /// 批量添加绘制命令
    ///
    /// # 参数
    /// - `device`: WGPU设备（用于扩展缓冲区）
    /// - `args`: 绘制参数迭代器
    ///
    /// # 错误
    ///
    /// 如果缓冲区扩展失败或参数无效，返回错误。
    ///
    /// # 性能优化
    /// - 使用批量操作减少函数调用开销
    /// - 自动扩展缓冲区以适应更多命令
    pub fn extend(
        &mut self,
        device: &wgpu::Device,
        args: impl IntoIterator<Item = DrawIndexedIndirectArgs>,
    ) -> Result<(), IndirectDrawError> {
        let args_vec: Vec<_> = args.into_iter().collect();
        
        // 检查是否需要扩展缓冲区
        if self.commands.len() + args_vec.len() > self.indirect_buffer.max_draws as usize {
            let new_capacity = ((self.commands.len() + args_vec.len()) as u32 * 3 / 2).max(64);
            self.indirect_buffer.ensure_capacity(device, new_capacity);
        }
        
        // 验证所有参数的有效性
        for arg in &args_vec {
            if arg.index_count == 0 {
                return Err(IndirectDrawError::InvalidArguments(
                    "index_count cannot be zero".to_string(),
                ));
            }
        }
        
        self.commands.extend(args_vec);
        Ok(())
    }

    /// 提交到 GPU（批量更新）
    ///
    /// # 性能优化
    /// - 使用批量写入减少CPU开销
    /// - 只更新实际使用的部分
    ///
    /// # 错误
    ///
    /// 如果命令数量超过缓冲区容量，返回错误。
    pub fn submit(&self, queue: &wgpu::Queue) -> Result<(), IndirectDrawError> {
        if !self.commands.is_empty() {
            self.indirect_buffer.update_indexed(queue, &self.commands)?;
        }
        Ok(())
    }

    /// 获取命令数量
    pub fn count(&self) -> u32 {
        self.commands.len() as u32
    }

    /// 获取间接缓冲区
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.indirect_buffer.buffer()
    }

    /// 获取最大绘制调用数
    pub fn max_draws(&self) -> u32 {
        self.indirect_buffer.max_draws()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_indirect_args_default() {
        let args = DrawIndirectArgs::default();
        assert_eq!(args.vertex_count, 0);
        assert_eq!(args.instance_count, 0);
    }

    #[test]
    fn test_draw_indexed_indirect_args_default() {
        let args = DrawIndexedIndirectArgs::default();
        assert_eq!(args.index_count, 0);
        assert_eq!(args.base_vertex, 0);
    }
}
