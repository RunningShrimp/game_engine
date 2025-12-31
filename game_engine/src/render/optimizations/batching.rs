// Draw Call批处理
//
// 通过合并渲染调用减少GPU状态切换

use std::collections::HashMap;

// ============================================================================
// 批处理配置
// ============================================================================

/// 批处理策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchingStrategy {
    /// 静态批处理（在加载时合并静态几何体）
    Static,
    /// 动态批处理（每帧合并动态几何体）
    Dynamic,
    /// 实例化渲染（使用GPU实例化）
    Instancing,
}

/// 批处理配置
#[derive(Debug, Clone)]
pub struct BatchingConfig {
    /// 批处理策略
    pub strategy: BatchingStrategy,
    /// 最大批处理大小（顶点数）
    pub max_batch_size: usize,
    /// 最小批处理大小（低于此值不批处理）
    pub min_batch_size: usize,
    /// 是否启用材质排序
    pub sort_by_material: bool,
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            strategy: BatchingStrategy::Dynamic,
            max_batch_size: 65536, // 64k vertices
            min_batch_size: 100,
            sort_by_material: true,
        }
    }
}

// ============================================================================
// 渲染批次
// ============================================================================

/// 渲染批次
#[derive(Debug, Clone)]
pub struct RenderBatch {
    /// 批次ID
    pub id: u32,
    /// 材质ID
    pub material_id: u32,
    /// 网格ID列表
    pub mesh_ids: Vec<u32>,
    /// 顶点总数
    pub vertex_count: usize,
    /// 索引总数
    pub index_count: usize,
    /// 批次边界（AABB）
    pub bounds: BatchBounds,
}

/// 批次边界
#[derive(Debug, Clone, Copy)]
pub struct BatchBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Default for BatchBounds {
    fn default() -> Self {
        Self {
            min: [0.0; 3],
            max: [0.0; 3],
        }
    }
}

// ============================================================================
// 静态批处理器
// ============================================================================

/// 静态批处理器
pub struct StaticBatcher {
    /// 批次映射（材质ID -> 批次）
    batches: HashMap<u32, RenderBatch>,
    /// 配置
    config: BatchingConfig,
    /// 下一个批次ID
    next_batch_id: u32,
}

impl StaticBatcher {
    /// 创建新的静态批处理器
    pub fn new(config: BatchingConfig) -> Self {
        Self {
            batches: HashMap::new(),
            config,
            next_batch_id: 0,
        }
    }

    /// 添加网格到批次
    pub fn add_mesh(&mut self, mesh_id: u32, material_id: u32, vertex_count: usize, index_count: usize) {
        let batch = self.batches.entry(material_id).or_insert_with(|| {
            let id = self.next_batch_id;
            self.next_batch_id += 1;
            RenderBatch {
                id,
                material_id,
                mesh_ids: Vec::new(),
                vertex_count: 0,
                index_count: 0,
                bounds: BatchBounds::default(),
            }
        });

        // 检查批次大小限制
        if batch.vertex_count + vertex_count > self.config.max_batch_size {
            // 创建新批次
            let id = self.next_batch_id;
            self.next_batch_id += 1;
            let new_batch = RenderBatch {
                id,
                material_id,
                mesh_ids: vec![mesh_id],
                vertex_count,
                index_count,
                bounds: BatchBounds::default(),
            };
            self.batches.insert(material_id, new_batch);
        } else {
            batch.mesh_ids.push(mesh_id);
            batch.vertex_count += vertex_count;
            batch.index_count += index_count;
        }
    }

    /// 完成批处理并返回所有批次
    pub fn finalize(&self) -> Vec<RenderBatch> {
        let mut batches: Vec<_> = self.batches.values().cloned().collect();

        // 可选：按材质排序
        if self.config.sort_by_material {
            batches.sort_by(|a, b| a.material_id.cmp(&b.material_id));
        }

        batches
    }

    /// 清空所有批次
    pub fn clear(&mut self) {
        self.batches.clear();
        self.next_batch_id = 0;
    }
}

// ============================================================================
// 动态批处理器
// ============================================================================

/// 动态批处理器
pub struct DynamicBatcher {
    /// 当前帧的批次
    current_batches: HashMap<u32, RenderBatch>,
    /// 配置
    config: BatchingConfig,
    /// 下一个批次ID
    next_batch_id: u32,
}

impl DynamicBatcher {
    /// 创建新的动态批处理器
    pub fn new(config: BatchingConfig) -> Self {
        Self {
            current_batches: HashMap::new(),
            config,
            next_batch_id: 0,
        }
    }

    /// 开始新帧
    pub fn begin_frame(&mut self) {
        self.current_batches.clear();
    }

    /// 添加网格到当前批次
    pub fn add_mesh(&mut self, mesh_id: u32, material_id: u32, vertex_count: usize, index_count: usize) {
        let batch = self.current_batches.entry(material_id).or_insert_with(|| {
            let id = self.next_batch_id;
            self.next_batch_id += 1;
            RenderBatch {
                id,
                material_id,
                mesh_ids: Vec::new(),
                vertex_count: 0,
                index_count: 0,
                bounds: BatchBounds::default(),
            }
        });

        // 检查批次大小限制
        if batch.vertex_count + vertex_count > self.config.max_batch_size {
            // 不添加到批次，单独渲染
            return;
        }

        batch.mesh_ids.push(mesh_id);
        batch.vertex_count += vertex_count;
        batch.index_count += index_count;
    }

    /// 获取当前帧的批次
    pub fn get_batches(&self) -> Vec<RenderBatch> {
        let mut batches: Vec<_> = self.current_batches.values().cloned().collect();

        // 过滤掉太小的批次
        batches.retain(|b| b.vertex_count >= self.config.min_batch_size);

        // 按材质排序
        if self.config.sort_by_material {
            batches.sort_by(|a, b| a.material_id.cmp(&b.material_id));
        }

        batches
    }
}

// ============================================================================
// 实例化渲染
// ============================================================================

/// 实例数据
#[derive(Debug, Clone)]
pub struct InstanceData {
    /// 变换矩阵
    pub transform: [[f32; 4]; 4],
    /// 实例颜色
    pub color: [f32; 4],
    /// 自定义数据
    pub custom_data: Vec<f32>,
}

/// 实例化批次
pub struct InstancedBatch {
    /// 基础网格ID
    pub base_mesh_id: u32,
    /// 材质ID
    pub material_id: u32,
    /// 实例数据
    pub instances: Vec<InstanceData>,
    /// 最大实例数
    pub max_instances: usize,
}

impl InstancedBatch {
    /// 创建新的实例化批次
    pub fn new(base_mesh_id: u32, material_id: u32, max_instances: usize) -> Self {
        Self {
            base_mesh_id,
            material_id,
            instances: Vec::with_capacity(max_instances),
            max_instances,
        }
    }

    /// 添加实例
    pub fn add_instance(&mut self, instance: InstanceData) -> bool {
        if self.instances.len() < self.max_instances {
            self.instances.push(instance);
            true
        } else {
            false
        }
    }

    /// 获取实例数量
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// 清空实例
    pub fn clear(&mut self) {
        self.instances.clear();
    }
}

/// 实例化批处理器
pub struct InstancingBatcher {
    /// 实例化批次映射（网格ID + 材质ID -> 批次）
    batches: HashMap<(u32, u32), InstancedBatch>,
    /// 最大实例数
    max_instances: usize,
}

impl InstancingBatcher {
    /// 创建新的实例化批处理器
    pub fn new(max_instances: usize) -> Self {
        Self {
            batches: HashMap::new(),
            max_instances,
        }
    }

    /// 添加实例
    pub fn add_instance(&mut self, mesh_id: u32, material_id: u32, instance: InstanceData) {
        let batch = self
            .batches
            .entry((mesh_id, material_id))
            .or_insert_with(|| InstancedBatch::new(mesh_id, material_id, self.max_instances));

        batch.add_instance(instance);
    }

    /// 获取所有批次
    pub fn get_batches(&self) -> Vec<&InstancedBatch> {
        self.batches.values().collect()
    }

    /// 清空所有批次
    pub fn clear(&mut self) {
        self.batches.clear();
    }
}

// ============================================================================
// 批处理管理器
// ============================================================================

/// 批处理管理器
pub struct BatchingManager {
    /// 静态批处理器
    static_batcher: StaticBatcher,
    /// 动态批处理器
    dynamic_batcher: DynamicBatcher,
    /// 实例化批处理器
    instancing_batcher: InstancingBatcher,
    /// 配置
    config: BatchingConfig,
}

impl BatchingManager {
    /// 创建新的批处理管理器
    pub fn new(config: BatchingConfig, max_instances: usize) -> Self {
        Self {
            static_batcher: StaticBatcher::new(config.clone()),
            dynamic_batcher: DynamicBatcher::new(config.clone()),
            instancing_batcher: InstancingBatcher::new(max_instances),
            config,
        }
    }

    /// 添加静态网格
    pub fn add_static_mesh(&mut self, mesh_id: u32, material_id: u32, vertex_count: usize, index_count: usize) {
        self.static_batcher.add_mesh(mesh_id, material_id, vertex_count, index_count);
    }

    /// 添加动态网格
    pub fn add_dynamic_mesh(&mut self, mesh_id: u32, material_id: u32, vertex_count: usize, index_count: usize) {
        self.dynamic_batcher.add_mesh(mesh_id, material_id, vertex_count, index_count);
    }

    /// 添加实例
    pub fn add_instance(&mut self, mesh_id: u32, material_id: u32, instance: InstanceData) {
        self.instancing_batcher.add_instance(mesh_id, material_id, instance);
    }

    /// 获取静态批次
    pub fn get_static_batches(&self) -> Vec<RenderBatch> {
        self.static_batcher.finalize()
    }

    /// 开始新帧
    pub fn begin_frame(&mut self) {
        self.dynamic_batcher.begin_frame();
    }

    /// 获取动态批次
    pub fn get_dynamic_batches(&self) -> Vec<RenderBatch> {
        self.dynamic_batcher.get_batches()
    }

    /// 获取实例化批次
    pub fn get_instanced_batches(&self) -> Vec<&InstancedBatch> {
        self.instancing_batcher.get_batches()
    }

    /// 清空所有批处理器
    pub fn clear(&mut self) {
        self.static_batcher.clear();
        self.dynamic_batcher.begin_frame();
        self.instancing_batcher.clear();
    }
}

// ============================================================================
// 统计信息
// ============================================================================

/// 批处理统计信息
#[derive(Debug, Clone, Copy)]
pub struct BatchingStatistics {
    /// 原始draw call数量
    pub original_draw_calls: u32,
    /// 批处理后draw call数量
    pub batched_draw_calls: u32,
    /// 减少的draw call数量
    pub reduced_draw_calls: u32,
    /// 批处理效率（百分比）
    pub efficiency: f32,
}

impl BatchingStatistics {
    /// 计算批处理统计
    pub fn calculate(original: u32, batched: u32) -> Self {
        let reduced = original.saturating_sub(batched);
        let efficiency = if original > 0 {
            (reduced as f32 / original as f32) * 100.0
        } else {
            0.0
        };

        Self {
            original_draw_calls: original,
            batched_draw_calls: batched,
            reduced_draw_calls: reduced,
            efficiency,
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_batcher() {
        let mut batcher = StaticBatcher::new(BatchingConfig::default());

        batcher.add_mesh(1, 100, 1000, 1500);
        batcher.add_mesh(2, 100, 2000, 3000);

        let batches = batcher.finalize();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].vertex_count, 3000);
    }

    #[test]
    fn test_dynamic_batcher() {
        let mut batcher = DynamicBatcher::new(BatchingConfig::default());

        batcher.begin_frame();
        batcher.add_mesh(1, 100, 1000, 1500);
        batcher.add_mesh(2, 100, 2000, 3000);

        let batches = batcher.get_batches();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].vertex_count, 3000);
    }

    #[test]
    fn test_instancing_batcher() {
        let mut batcher = InstancingBatcher::new(100);

        let instance = InstanceData {
            transform: [[1.0, 0.0, 0.0, 0.0]; 4],
            color: [1.0, 0.0, 0.0, 1.0],
            custom_data: vec![],
        };

        batcher.add_instance(1, 100, instance.clone());
        batcher.add_instance(1, 100, instance.clone());

        let batches = batcher.get_batches();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].instance_count(), 2);
    }

    #[test]
    fn test_batching_statistics() {
        let stats = BatchingStatistics::calculate(100, 20);
        assert_eq!(stats.original_draw_calls, 100);
        assert_eq!(stats.batched_draw_calls, 20);
        assert_eq!(stats.reduced_draw_calls, 80);
        assert!((stats.efficiency - 80.0).abs() < 0.01);
    }
}
