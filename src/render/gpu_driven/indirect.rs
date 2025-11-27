//! 间接绘制模块
//!
//! 实现 GPU 间接绘制支持。

/// 间接绘制参数
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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

impl Default for DrawIndirectArgs {
    fn default() -> Self {
        Self {
            vertex_count: 0,
            instance_count: 0,
            first_vertex: 0,
            first_instance: 0,
        }
    }
}

/// 索引间接绘制参数
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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

impl Default for DrawIndexedIndirectArgs {
    fn default() -> Self {
        Self {
            index_count: 0,
            instance_count: 0,
            first_index: 0,
            base_vertex: 0,
            first_instance: 0,
        }
    }
}

/// 间接绘制缓冲区
pub struct IndirectDrawBuffer {
    /// 间接绘制缓冲区
    buffer: wgpu::Buffer,
    /// 最大绘制调用数
    max_draws: u32,
}

impl IndirectDrawBuffer {
    /// 创建间接绘制缓冲区
    pub fn new(device: &wgpu::Device, max_draws: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Draw Buffer"),
            size: (std::mem::size_of::<DrawIndirectArgs>() * max_draws as usize) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        Self { buffer, max_draws }
    }
    
    /// 创建索引间接绘制缓冲区
    pub fn new_indexed(device: &wgpu::Device, max_draws: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indexed Indirect Draw Buffer"),
            size: (std::mem::size_of::<DrawIndexedIndirectArgs>() * max_draws as usize) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        Self { buffer, max_draws }
    }
    
    /// 获取缓冲区
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    
    /// 获取最大绘制调用数
    pub fn max_draws(&self) -> u32 {
        self.max_draws
    }
    
    /// 更新间接绘制参数
    pub fn update(&self, queue: &wgpu::Queue, args: &[DrawIndirectArgs]) {
        if !args.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(args));
        }
    }
    
    /// 更新索引间接绘制参数
    pub fn update_indexed(&self, queue: &wgpu::Queue, args: &[DrawIndexedIndirectArgs]) {
        if !args.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(args));
        }
    }
}

/// 多绘制间接批处理器
pub struct MultiDrawIndirect {
    /// 绘制命令列表
    commands: Vec<DrawIndexedIndirectArgs>,
    /// 间接缓冲区
    indirect_buffer: IndirectDrawBuffer,
}

impl MultiDrawIndirect {
    /// 创建多绘制批处理器
    pub fn new(device: &wgpu::Device, max_draws: u32) -> Self {
        Self {
            commands: Vec::with_capacity(max_draws as usize),
            indirect_buffer: IndirectDrawBuffer::new_indexed(device, max_draws),
        }
    }
    
    /// 清空命令
    pub fn clear(&mut self) {
        self.commands.clear();
    }
    
    /// 添加绘制命令
    pub fn push(&mut self, args: DrawIndexedIndirectArgs) {
        if self.commands.len() < self.indirect_buffer.max_draws as usize {
            self.commands.push(args);
        }
    }
    
    /// 提交到 GPU
    pub fn submit(&self, queue: &wgpu::Queue) {
        self.indirect_buffer.update_indexed(queue, &self.commands);
    }
    
    /// 获取命令数量
    pub fn count(&self) -> u32 {
        self.commands.len() as u32
    }
    
    /// 获取间接缓冲区
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.indirect_buffer.buffer()
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

