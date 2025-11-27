/// 高性能环形缓冲区
/// 
/// 用于替代Vec存储历史数据，提供O(1)的push操作

use std::collections::VecDeque;

/// 固定容量的环形缓冲区
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    /// 创建指定容量的环形缓冲区
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    /// 添加元素，如果已满则移除最旧的元素
    pub fn push(&mut self, value: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();  // O(1) 操作
        }
        self.buffer.push_back(value);
    }
    
    /// 获取迭代器
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }
    
    /// 获取元素数量
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    
    /// 获取最新的元素
    pub fn last(&self) -> Option<&T> {
        self.buffer.back()
    }
    
    /// 获取最旧的元素
    pub fn first(&self) -> Option<&T> {
        self.buffer.front()
    }
}

impl<T> RingBuffer<T> 
where 
    T: Copy + Into<f32>
{
    /// 计算平均值（仅适用于可转换为f32的类型）
    pub fn average(&self) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.buffer.iter().map(|&x| x.into()).sum();
        sum / self.buffer.len() as f32
    }
    
    /// 计算最小值
    pub fn min(&self) -> Option<f32> {
        self.buffer.iter()
            .map(|&x| x.into())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }
    
    /// 计算最大值
    pub fn max(&self) -> Option<f32> {
        self.buffer.iter()
            .map(|&x| x.into())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

impl<T> Default for RingBuffer<T> {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut buffer = RingBuffer::new(3);
        
        buffer.push(1.0f32);
        buffer.push(2.0);
        buffer.push(3.0);
        
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.average(), 2.0);
        
        // 添加第4个元素，应该移除第1个
        buffer.push(4.0);
        
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.first(), Some(&2.0));
        assert_eq!(buffer.last(), Some(&4.0));
        assert_eq!(buffer.average(), 3.0);
    }
    
    #[test]
    fn test_ring_buffer_stats() {
        let mut buffer = RingBuffer::new(5);
        
        buffer.push(10.0f32);
        buffer.push(20.0);
        buffer.push(30.0);
        buffer.push(40.0);
        buffer.push(50.0);
        
        assert_eq!(buffer.average(), 30.0);
        assert_eq!(buffer.min(), Some(10.0));
        assert_eq!(buffer.max(), Some(50.0));
    }
    
    #[test]
    fn test_ring_buffer_empty() {
        let buffer: RingBuffer<f32> = RingBuffer::new(10);
        
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.average(), 0.0);
        assert_eq!(buffer.min(), None);
        assert_eq!(buffer.max(), None);
    }
}
