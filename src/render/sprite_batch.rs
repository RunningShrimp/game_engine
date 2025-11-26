use glam::{Vec2, Vec3, Vec4, Mat4};
use wgpu::{Device, Queue, Buffer, BufferUsages};

/// 精灵实例数据
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    /// 变换矩阵 (4x4)
    pub transform: [[f32; 4]; 4],
    /// 纹理坐标 (x, y, width, height)
    pub tex_coords: [f32; 4],
    /// 颜色 (RGBA)
    pub color: [f32; 4],
}

impl SpriteInstance {
    /// 创建新的精灵实例
    pub fn new(position: Vec3, size: Vec2, tex_coords: Vec4, color: Vec4) -> Self {
        let transform = Mat4::from_scale_rotation_translation(
            Vec3::new(size.x, size.y, 1.0),
            glam::Quat::IDENTITY,
            position,
        );
        
        Self {
            transform: transform.to_cols_array_2d(),
            tex_coords: tex_coords.to_array(),
            color: color.to_array(),
        }
    }
}

/// 精灵批次
pub struct SpriteBatch {
    /// 精灵实例
    instances: Vec<SpriteInstance>,
    /// 实例缓冲区
    instance_buffer: Option<Buffer>,
    /// 最大批次大小
    max_batch_size: usize,
}

impl SpriteBatch {
    /// 创建新的精灵批次
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            instances: Vec::with_capacity(max_batch_size),
            instance_buffer: None,
            max_batch_size,
        }
    }
    
    /// 添加精灵实例
    pub fn add(&mut self, instance: SpriteInstance) -> bool {
        if self.instances.len() >= self.max_batch_size {
            return false;
        }
        
        self.instances.push(instance);
        true
    }
    
    /// 清空批次
    pub fn clear(&mut self) {
        self.instances.clear();
    }
    
    /// 获取实例数量
    pub fn len(&self) -> usize {
        self.instances.len()
    }
    
    /// 检查批次是否为空
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
    
    /// 检查批次是否已满
    pub fn is_full(&self) -> bool {
        self.instances.len() >= self.max_batch_size
    }
    
    /// 更新实例缓冲区
    pub fn update_buffer(&mut self, device: &Device, queue: &Queue) {
        if self.instances.is_empty() {
            return;
        }
        
        // 创建或更新缓冲区
        if self.instance_buffer.is_none() || 
           self.instance_buffer.as_ref().unwrap().size() < (self.instances.len() * std::mem::size_of::<SpriteInstance>()) as u64 {
            self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Sprite Instance Buffer"),
                size: (self.max_batch_size * std::mem::size_of::<SpriteInstance>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        
        // 写入数据
        if let Some(buffer) = &self.instance_buffer {
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&self.instances),
            );
        }
    }
    
    /// 获取实例缓冲区
    pub fn buffer(&self) -> Option<&Buffer> {
        self.instance_buffer.as_ref()
    }
}

/// 精灵批量渲染器
pub struct SpriteBatchRenderer {
    /// 当前批次
    current_batch: SpriteBatch,
    /// 已完成的批次
    completed_batches: Vec<SpriteBatch>,
    /// 最大批次大小
    max_batch_size: usize,
}

impl SpriteBatchRenderer {
    /// 创建新的批量渲染器
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            current_batch: SpriteBatch::new(max_batch_size),
            completed_batches: Vec::new(),
            max_batch_size,
        }
    }
    
    /// 添加精灵
    pub fn add_sprite(&mut self, instance: SpriteInstance) {
        if !self.current_batch.add(instance) {
            // 当前批次已满,创建新批次
            let mut new_batch = SpriteBatch::new(self.max_batch_size);
            new_batch.add(instance);
            
            let old_batch = std::mem::replace(&mut self.current_batch, new_batch);
            self.completed_batches.push(old_batch);
        }
    }
    
    /// 完成批次
    pub fn finish(&mut self) {
        if !self.current_batch.is_empty() {
            let new_batch = SpriteBatch::new(self.max_batch_size);
            let old_batch = std::mem::replace(&mut self.current_batch, new_batch);
            self.completed_batches.push(old_batch);
        }
    }
    
    /// 更新所有缓冲区
    pub fn update_buffers(&mut self, device: &Device, queue: &Queue) {
        for batch in &mut self.completed_batches {
            batch.update_buffer(device, queue);
        }
        
        if !self.current_batch.is_empty() {
            self.current_batch.update_buffer(device, queue);
        }
    }
    
    /// 获取所有批次
    pub fn batches(&self) -> impl Iterator<Item = &SpriteBatch> {
        self.completed_batches.iter().chain(std::iter::once(&self.current_batch))
    }
    
    /// 清空所有批次
    pub fn clear(&mut self) {
        self.current_batch.clear();
        self.completed_batches.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sprite_batch() {
        let mut batch = SpriteBatch::new(10);
        
        for i in 0..5 {
            let instance = SpriteInstance::new(
                Vec3::new(i as f32, 0.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec4::new(0.0, 0.0, 1.0, 1.0),
                Vec4::ONE,
            );
            assert!(batch.add(instance));
        }
        
        assert_eq!(batch.len(), 5);
        assert!(!batch.is_full());
    }
    
    #[test]
    fn test_sprite_batch_renderer() {
        let mut renderer = SpriteBatchRenderer::new(2);
        
        // 添加3个精灵,应该创建2个批次
        for i in 0..3 {
            let instance = SpriteInstance::new(
                Vec3::new(i as f32, 0.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec4::new(0.0, 0.0, 1.0, 1.0),
                Vec4::ONE,
            );
            renderer.add_sprite(instance);
        }
        
        renderer.finish();
        assert_eq!(renderer.batches().count(), 2);
    }
}
