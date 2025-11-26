use std::collections::HashMap;

/// 批次键 - 用于合并相同材质和纹理的绘制调用
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BatchKey {
    pub material_id: u32,
    pub texture_id: u32,
    pub shader_id: u32,
}

/// 批次数据
#[derive(Debug, Clone)]
pub struct Batch {
    pub key: BatchKey,
    pub instance_count: u32,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

/// 批量渲染器 - 合并绘制调用以减少CPU开销
pub struct BatchRenderer {
    batches: HashMap<BatchKey, Vec<Batch>>,
    max_instances_per_batch: u32,
}

impl BatchRenderer {
    pub fn new(max_instances_per_batch: u32) -> Self {
        Self {
            batches: HashMap::new(),
            max_instances_per_batch,
        }
    }
    
    /// 添加绘制调用到批次
    pub fn add_draw_call(
        &mut self,
        key: BatchKey,
        vertex_offset: u32,
        index_offset: u32,
        index_count: u32,
    ) {
        let batches = self.batches.entry(key).or_insert_with(Vec::new);
        
        // 尝试合并到现有批次
        if let Some(last_batch) = batches.last_mut() {
            if last_batch.instance_count < self.max_instances_per_batch {
                last_batch.instance_count += 1;
                return;
            }
        }
        
        // 创建新批次
        batches.push(Batch {
            key,
            instance_count: 1,
            vertex_offset,
            index_offset,
            index_count,
        });
    }
    
    /// 获取所有批次
    pub fn get_batches(&self) -> Vec<&Batch> {
        self.batches.values().flatten().collect()
    }
    
    /// 清空批次
    pub fn clear(&mut self) {
        self.batches.clear();
    }
    
    /// 获取批次统计信息
    pub fn stats(&self) -> BatchStats {
        let total_batches: usize = self.batches.values().map(|v| v.len()).sum();
        let total_instances: u32 = self.batches.values()
            .flatten()
            .map(|b| b.instance_count)
            .sum();
        
        BatchStats {
            total_batches,
            total_instances,
            unique_materials: self.batches.len(),
        }
    }
}

/// 批次统计信息
#[derive(Debug, Clone)]
pub struct BatchStats {
    pub total_batches: usize,
    pub total_instances: u32,
    pub unique_materials: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_renderer() {
        let mut renderer = BatchRenderer::new(100);
        
        let key = BatchKey {
            material_id: 1,
            texture_id: 1,
            shader_id: 1,
        };
        
        // 添加多个绘制调用
        for i in 0..150 {
            renderer.add_draw_call(key, i * 4, i * 6, 6);
        }
        
        let stats = renderer.stats();
        assert_eq!(stats.total_batches, 2); // 应该分成2个批次
        assert_eq!(stats.total_instances, 150);
        assert_eq!(stats.unique_materials, 1);
    }
}
