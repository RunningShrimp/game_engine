//! 渲染批处理优化器
//!
//! 提供智能的批处理优化，包括：
//! - 按渲染状态优先级排序
//! - 批处理统计和性能监控
//! - 自动批处理合并
//! - 状态切换成本分析
//!
//! ## 设计原则
//!
//! 1. **最小化状态切换**：按状态切换成本排序（Pipeline > Blend > Depth > Material > Mesh）
//! 2. **最大化批处理**：合并相同状态的绘制调用
//! 3. **性能监控**：实时统计批处理效果
//! 4. **自适应优化**：根据场景动态调整批处理策略

pub use crate::render::instance_batch::BatchKey;
use std::time::Instant;

/// 渲染状态切换成本权重
#[derive(Debug, Clone, Copy)]
pub struct StateSwitchCost {
    /// Pipeline切换成本（最高）
    pub pipeline: f32,
    /// Blend模式切换成本
    pub blend: f32,
    /// Depth测试切换成本
    pub depth: f32,
    /// Material切换成本
    pub material: f32,
    /// Mesh切换成本（最低）
    pub mesh: f32,
}

impl Default for StateSwitchCost {
    fn default() -> Self {
        Self {
            pipeline: 100.0, // Pipeline切换最昂贵
            blend: 50.0,
            depth: 30.0,
            material: 10.0,
            mesh: 1.0, // Mesh切换相对便宜
        }
    }
}

/// 批处理优化统计信息
///
/// 注意：与`instance_batch::BatchStats`不同，此结构专门用于批处理优化统计
#[derive(Debug, Clone, Default)]
pub struct BatchOptimizerStats {
    /// 总批次数
    pub total_batches: usize,
    /// 总实例数
    pub total_instances: u32,
    /// 唯一材质数
    pub unique_materials: usize,
    /// 唯一网格数
    pub unique_meshes: usize,
    /// 状态切换次数
    pub state_switches: u32,
    /// 批处理优化率（0.0-1.0）
    pub optimization_ratio: f32,
    /// 平均每批次实例数
    pub avg_instances_per_batch: f32,
    /// 最大批次实例数
    pub max_instances_per_batch: u32,
}

/// 批处理优化器
pub struct BatchOptimizer {
    /// 状态切换成本配置
    cost: StateSwitchCost,
    /// 最大每批次实例数
    max_instances_per_batch: u32,
    /// 批处理统计
    stats: BatchOptimizerStats,
    /// 性能监控开始时间
    optimization_start: Option<Instant>,
}

impl BatchOptimizer {
    /// 创建新的批处理优化器
    pub fn new(max_instances_per_batch: u32) -> Self {
        Self {
            cost: StateSwitchCost::default(),
            max_instances_per_batch,
            stats: BatchOptimizerStats::default(),
            optimization_start: None,
        }
    }

    /// 使用自定义状态切换成本创建
    pub fn with_cost(cost: StateSwitchCost, max_instances_per_batch: u32) -> Self {
        Self {
            cost,
            max_instances_per_batch,
            stats: BatchOptimizerStats::default(),
            optimization_start: None,
        }
    }

    /// 优化批次列表
    ///
    /// 按渲染状态优先级排序，最小化状态切换成本
    pub fn optimize_batches(&mut self, batches: &mut Vec<OptimizedBatch>) {
        self.optimization_start = Some(Instant::now());

        // 按BatchKey排序（BatchKey已经实现了按优先级排序）
        batches.sort_by(|a, b| a.key.cmp(&b.key));

        // 合并相同状态的批次
        self.merge_batches(batches);

        // 计算统计信息
        self.calculate_stats(batches);
    }

    /// 合并相同状态的批次
    fn merge_batches(&self, batches: &mut Vec<OptimizedBatch>) {
        if batches.is_empty() {
            return;
        }

        let mut merged = Vec::new();
        let mut current = batches[0].clone();
        let mut _state_switches = 0u32;

        for batch in batches.iter().skip(1) {
            // 检查是否可以合并（相同状态且未超过最大实例数）
            if current.key == batch.key
                && (current.instance_count + batch.instance_count) <= self.max_instances_per_batch
            {
                // 合并批次
                current.instance_count += batch.instance_count;
                current.instances.extend_from_slice(&batch.instances);
            } else {
                // 无法合并，保存当前批次，开始新批次
                merged.push(current);
                current = batch.clone();
                _state_switches += 1;
            }
        }

        // 添加最后一个批次
        merged.push(current);

        // 替换原列表
        batches.clear();
        batches.extend(merged);
    }

    /// 计算统计信息
    fn calculate_stats(&mut self, batches: &[OptimizedBatch]) {
        if batches.is_empty() {
            self.stats = BatchOptimizerStats::default();
            return;
        }

        let total_instances: u32 = batches.iter().map(|b| b.instance_count).sum();
        let unique_materials: std::collections::HashSet<u64> =
            batches.iter().map(|b| b.key.material_id).collect();
        let unique_meshes: std::collections::HashSet<u64> =
            batches.iter().map(|b| b.key.mesh_id).collect();

        let max_instances = batches.iter().map(|b| b.instance_count).max().unwrap_or(0);

        // 计算状态切换次数（相邻批次状态不同）
        let mut state_switches = 0u32;
        for i in 1..batches.len() {
            if batches[i].key != batches[i - 1].key {
                state_switches += 1;
            }
        }

        // 计算优化率（假设原始每个实例一个批次）
        let original_batches = total_instances as usize;
        let optimized_batches = batches.len();
        let optimization_ratio = if original_batches > 0 {
            1.0 - (optimized_batches as f32 / original_batches as f32)
        } else {
            0.0
        };

        self.stats = BatchOptimizerStats {
            total_batches: optimized_batches,
            total_instances,
            unique_materials: unique_materials.len(),
            unique_meshes: unique_meshes.len(),
            state_switches,
            optimization_ratio,
            avg_instances_per_batch: if optimized_batches > 0 {
                total_instances as f32 / optimized_batches as f32
            } else {
                0.0
            },
            max_instances_per_batch: max_instances,
        };
    }

    /// 获取统计信息
    pub fn stats(&self) -> &BatchOptimizerStats {
        &self.stats
    }

    /// 获取优化耗时（微秒）
    pub fn optimization_time_us(&self) -> Option<u64> {
        self.optimization_start.map(|start| start.elapsed().as_micros() as u64)
    }

    /// 计算状态切换成本
    pub fn calculate_switch_cost(&self, from: &BatchKey, to: &BatchKey) -> f32 {
        let mut cost = 0.0;

        if from.pipeline_id != to.pipeline_id {
            cost += self.cost.pipeline;
        }
        if from.blend_mode != to.blend_mode {
            cost += self.cost.blend;
        }
        if from.depth_test != to.depth_test {
            cost += self.cost.depth;
        }
        if from.material_id != to.material_id {
            cost += self.cost.material;
        }
        if from.mesh_id != to.mesh_id {
            cost += self.cost.mesh;
        }

        cost
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = BatchOptimizerStats::default();
        self.optimization_start = None;
    }
}

/// 优化后的批次
#[derive(Debug, Clone)]
pub struct OptimizedBatch {
    /// 批次键
    pub key: BatchKey,
    /// 实例数量
    pub instance_count: u32,
    /// 实例数据
    pub instances: Vec<u32>, // 实例索引或ID
    /// 顶点偏移
    pub vertex_offset: u32,
    /// 索引偏移
    pub index_offset: u32,
    /// 索引数量
    pub index_count: u32,
}

impl OptimizedBatch {
    /// 创建新批次
    pub fn new(key: BatchKey, instance_count: u32) -> Self {
        Self {
            key,
            instance_count,
            instances: Vec::new(),
            vertex_offset: 0,
            index_offset: 0,
            index_count: 0,
        }
    }
}

/// 批处理性能监控器
pub struct BatchPerformanceMonitor {
    /// 历史统计信息
    history: Vec<BatchOptimizerStats>,
    /// 最大历史记录数
    max_history: usize,
    /// 性能警告阈值
    warning_thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// 最小优化率（低于此值发出警告）
    pub min_optimization_ratio: f32,
    /// 最大状态切换次数（超过此值发出警告）
    pub max_state_switches: u32,
    /// 最小平均每批次实例数（低于此值发出警告）
    pub min_avg_instances: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            min_optimization_ratio: 0.5, // 至少50%的优化率
            max_state_switches: 100,     // 最多100次状态切换
            min_avg_instances: 10.0,     // 平均每批次至少10个实例
        }
    }
}

impl BatchPerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
            warning_thresholds: PerformanceThresholds::default(),
        }
    }

    /// 记录统计信息
    pub fn record_stats(&mut self, stats: BatchOptimizerStats) {
        // 先检查性能警告（在move之前）
        self.check_warnings(&stats);

        // 然后添加到历史记录
        self.history.push(stats);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// 检查性能警告
    fn check_warnings(&self, stats: &BatchOptimizerStats) {
        if stats.optimization_ratio < self.warning_thresholds.min_optimization_ratio {
            tracing::warn!(
                target: "render",
                "Batch optimization ratio is low: {:.2}% (threshold: {:.2}%)",
                stats.optimization_ratio * 100.0,
                self.warning_thresholds.min_optimization_ratio * 100.0
            );
        }

        if stats.state_switches > self.warning_thresholds.max_state_switches {
            tracing::warn!(
                target: "render",
                "Too many state switches: {} (threshold: {})",
                stats.state_switches,
                self.warning_thresholds.max_state_switches
            );
        }

        if stats.avg_instances_per_batch < self.warning_thresholds.min_avg_instances {
            tracing::warn!(
                target: "render",
                "Average instances per batch is low: {:.2} (threshold: {:.2})",
                stats.avg_instances_per_batch,
                self.warning_thresholds.min_avg_instances
            );
        }
    }

    /// 获取平均统计信息
    pub fn average_stats(&self) -> Option<BatchOptimizerStats> {
        if self.history.is_empty() {
            return None;
        }

        let count = self.history.len() as f32;
        let total_batches: usize = self.history.iter().map(|s| s.total_batches).sum();
        let total_instances: u32 = self.history.iter().map(|s| s.total_instances).sum();
        let total_switches: u32 = self.history.iter().map(|s| s.state_switches).sum();
        let total_optimization: f32 = self.history.iter().map(|s| s.optimization_ratio).sum();
        let total_avg_instances: f32 = self.history.iter().map(|s| s.avg_instances_per_batch).sum();

        Some(BatchOptimizerStats {
            total_batches: (total_batches as f32 / count) as usize,
            total_instances: (total_instances as f32 / count) as u32,
            unique_materials: 0, // 需要特殊处理
            unique_meshes: 0,    // 需要特殊处理
            state_switches: (total_switches as f32 / count) as u32,
            optimization_ratio: total_optimization / count,
            avg_instances_per_batch: total_avg_instances / count,
            max_instances_per_batch: self
                .history
                .iter()
                .map(|s| s.max_instances_per_batch)
                .max()
                .unwrap_or(0),
        })
    }

    /// 清空历史记录
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_optimizer() {
        let mut optimizer = BatchOptimizer::new(100);

        let mut batches = vec![
            OptimizedBatch::new(
                BatchKey {
                    mesh_id: 1,
                    material_id: 1,
                    pipeline_id: 1,
                    blend_mode: 0,
                    depth_test: true,
                    render_flags: 0,
                },
                50,
            ),
            OptimizedBatch::new(
                BatchKey {
                    mesh_id: 1,
                    material_id: 1,
                    pipeline_id: 1,
                    blend_mode: 0,
                    depth_test: true,
                    render_flags: 0,
                },
                30,
            ),
            OptimizedBatch::new(
                BatchKey {
                    mesh_id: 2,
                    material_id: 1,
                    pipeline_id: 1,
                    blend_mode: 0,
                    depth_test: true,
                    render_flags: 0,
                },
                20,
            ),
        ];

        optimizer.optimize_batches(&mut batches);

        let stats = optimizer.stats();
        assert_eq!(stats.total_batches, 2); // 前两个应该合并
        assert_eq!(stats.total_instances, 100);
    }

    #[test]
    fn test_state_switch_cost() {
        let optimizer = BatchOptimizer::new(100);

        let from = BatchKey {
            mesh_id: 1,
            material_id: 1,
            pipeline_id: 1,
            blend_mode: 0,
            depth_test: true,
            render_flags: 0,
        };

        let to = BatchKey {
            mesh_id: 2,
            material_id: 2,
            pipeline_id: 2,
            blend_mode: 1,
            depth_test: false,
            render_flags: 0,
        };

        let cost = optimizer.calculate_switch_cost(&from, &to);
        // 应该包含所有状态的切换成本
        assert!(cost > 0.0);
    }
}
