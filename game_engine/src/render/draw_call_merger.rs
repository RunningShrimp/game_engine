//! Draw Call合并优化模块
//!
//! 提供智能的draw call合并策略，最大化减少draw call数量：
//! - 状态切换成本分析
//! - 智能批次合并
//! - 动态批次大小调整
//! - 性能监控和自适应优化

use crate::render::batch_optimizer::{BatchOptimizer, OptimizedBatch};
use crate::render::instance_batch::BatchKey;
use std::time::Instant;

/// Draw Call合并配置
#[derive(Debug, Clone)]
pub struct DrawCallMergeConfig {
    /// 是否启用智能合并
    pub enable_smart_merge: bool,
    /// 最大合并距离（状态切换成本阈值）
    pub max_merge_cost: f32,
    /// 最小批次大小（低于此值不合并）
    pub min_batch_size: usize,
    /// 最大批次大小（超过此值拆分）
    pub max_batch_size: usize,
    /// 是否启用动态调整
    pub enable_dynamic_adjustment: bool,
}

impl Default for DrawCallMergeConfig {
    fn default() -> Self {
        Self {
            enable_smart_merge: true,
            max_merge_cost: 50.0, // 允许中等成本的状态切换
            min_batch_size: 2,
            max_batch_size: 65536,
            enable_dynamic_adjustment: true,
        }
    }
}

/// Draw Call合并器
pub struct DrawCallMerger {
    config: DrawCallMergeConfig,
    optimizer: BatchOptimizer,
    merge_stats: MergeStats,
}

/// 合并统计信息
#[derive(Debug, Clone, Default)]
pub struct MergeStats {
    /// 原始draw call数
    pub original_draw_calls: usize,
    /// 合并后draw call数
    pub merged_draw_calls: usize,
    /// 合并的批次数
    pub merged_batches: usize,
    /// 节省的draw call数
    pub saved_draw_calls: usize,
    /// 合并率（0.0-1.0）
    pub merge_ratio: f32,
    /// 合并耗时（微秒）
    pub merge_time_us: u64,
}

impl DrawCallMerger {
    /// 创建新的draw call合并器
    pub fn new(config: DrawCallMergeConfig) -> Self {
        let optimizer = BatchOptimizer::new(config.max_batch_size as u32);
        Self {
            config,
            optimizer,
            merge_stats: MergeStats::default(),
        }
    }

    /// 创建批次键（用于测试和调试）
    pub fn create_batch_key(
        mesh_id: u64,
        material_id: u64,
        pipeline_id: u32,
        blend_mode: u8,
        depth_test: bool,
        render_flags: u32,
    ) -> BatchKey {
        BatchKey {
            mesh_id,
            material_id,
            pipeline_id,
            blend_mode,
            depth_test,
            render_flags: render_flags as u16,
        }
    }

    /// 合并draw calls
    ///
    /// 智能合并相同或相似状态的批次，减少draw call数量。
    pub fn merge_draw_calls(&mut self, batches: &mut Vec<OptimizedBatch>) -> MergeStats {
        let start = Instant::now();
        let original_count = batches.len();

        if !self.config.enable_smart_merge || batches.is_empty() {
            return MergeStats {
                original_draw_calls: original_count,
                merged_draw_calls: original_count,
                merged_batches: 0,
                saved_draw_calls: 0,
                merge_ratio: 0.0,
                merge_time_us: start.elapsed().as_micros() as u64,
            };
        }

        // 1. 按状态优先级排序
        batches.sort_by(|a, b| a.key.cmp(&b.key));

        // 2. 智能合并
        let merged = self.smart_merge(batches);

        // 3. 更新批次列表
        batches.clear();
        batches.extend(merged);

        let merged_count = batches.len();
        let saved = original_count.saturating_sub(merged_count);
        let merge_ratio = if original_count > 0 {
            saved as f32 / original_count as f32
        } else {
            0.0
        };

        self.merge_stats = MergeStats {
            original_draw_calls: original_count,
            merged_draw_calls: merged_count,
            merged_batches: saved,
            saved_draw_calls: saved,
            merge_ratio,
            merge_time_us: start.elapsed().as_micros() as u64,
        };

        self.merge_stats.clone()
    }

    /// 智能合并批次
    fn smart_merge(&self, batches: &[OptimizedBatch]) -> Vec<OptimizedBatch> {
        if batches.is_empty() {
            return Vec::new();
        }

        let mut merged = Vec::new();
        let mut current = batches[0].clone();

        for batch in batches.iter().skip(1) {
            // 计算状态切换成本
            let switch_cost = self.optimizer.calculate_switch_cost(&current.key, &batch.key);

            // 检查是否可以合并
            if self.can_merge(&current, batch, switch_cost) {
                // 合并批次
                current.instance_count += batch.instance_count;
                current.instances.extend_from_slice(&batch.instances);
            } else {
                // 无法合并，保存当前批次
                merged.push(current);
                current = batch.clone();
            }
        }

        // 添加最后一个批次
        merged.push(current);
        merged
    }

    /// 检查是否可以合并
    fn can_merge(&self, a: &OptimizedBatch, b: &OptimizedBatch, switch_cost: f32) -> bool {
        // 相同状态可以合并
        if a.key == b.key {
            let total_instances = a.instance_count + b.instance_count;
            return total_instances <= self.config.max_batch_size as u32;
        }

        // 状态切换成本低于阈值且批次大小合适
        if switch_cost <= self.config.max_merge_cost {
            let total_instances = a.instance_count + b.instance_count;
            if total_instances <= self.config.max_batch_size as u32 {
                // 检查批次大小是否满足最小要求
                return a.instance_count >= self.config.min_batch_size as u32
                    || b.instance_count >= self.config.min_batch_size as u32;
            }
        }

        false
    }

    /// 获取合并统计信息
    pub fn stats(&self) -> &MergeStats {
        &self.merge_stats
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.merge_stats = MergeStats::default();
    }
}

/// 场景遍历优化器
///
/// 结合场景遍历和draw call合并，提供端到端的渲染优化。
pub struct SceneTraversalOptimizer {
    traverser: OptimizedSceneTraverser,
    merger: DrawCallMerger,
}

impl SceneTraversalOptimizer {
    /// 创建新的场景遍历优化器
    pub fn new(
        traversal_config: SceneTraversalConfig,
        merge_config: DrawCallMergeConfig,
    ) -> Self {
        Self {
            traverser: OptimizedSceneTraverser::new(traversal_config),
            merger: DrawCallMerger::new(merge_config),
        }
    }

    /// 优化场景遍历和draw call合并
    pub fn optimize(
        &mut self,
        world: &mut bevy_ecs::world::World,
        view_proj: Option<[[f32; 4]; 4]>,
    ) -> OptimizedSceneResult {
        // 1. 遍历场景
        let traversal_result = self.traverser.traverse_scene(world, view_proj);

        // 2. 合并draw calls
        let mut batches = traversal_result.batches;
        let merge_stats = self.merger.merge_draw_calls(&mut batches);

        OptimizedSceneResult {
            batches,
            gpu_instances: traversal_result.gpu_instances,
            traversal_stats: traversal_result.stats,
            merge_stats,
        }
    }
}

/// 优化后的场景结果
#[derive(Debug)]
pub struct OptimizedSceneResult {
    /// 优化后的批次
    pub batches: Vec<OptimizedBatch>,
    /// GPU实例数据
    pub gpu_instances: Vec<GpuInstance>,
    /// 遍历统计
    pub traversal_stats: TraversalStats,
    /// 合并统计
    pub merge_stats: MergeStats,
}

// 重新导出类型
pub use crate::render::scene_traversal::{
    IncrementalSceneUpdater, OptimizedSceneTraverser, SceneTraversalConfig, SceneTraversalResult,
    TraversalStats,
};
pub use crate::render::gpu_driven::GpuInstance;

// bevy_ecs::prelude 未在此文件中使用，但可能在未来需要
// use bevy_ecs::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_call_merge_config_default() {
        let config = DrawCallMergeConfig::default();
        assert!(config.enable_smart_merge);
        assert_eq!(config.max_merge_cost, 50.0);
        assert_eq!(config.min_batch_size, 2);
    }

    #[test]
    fn test_draw_call_merger_new() {
        let config = DrawCallMergeConfig::default();
        let merger = DrawCallMerger::new(config);
        assert_eq!(merger.stats().original_draw_calls, 0);
    }

    #[test]
    fn test_draw_call_merger_merge_same_state() {
        let config = DrawCallMergeConfig {
            max_batch_size: 100,
            ..Default::default()
        };
        let mut merger = DrawCallMerger::new(config);

        let key = BatchKey {
            mesh_id: 1,
            material_id: 1,
            pipeline_id: 1,
            blend_mode: 0,
            depth_test: true,
            render_flags: 0,
        };

        let mut batches = vec![
            OptimizedBatch::new(key, 10),
            OptimizedBatch::new(key, 20),
            OptimizedBatch::new(key, 30),
        ];

        let stats = merger.merge_draw_calls(&mut batches);
        assert_eq!(stats.merged_draw_calls, 1); // 应该合并为一个批次
        assert!(stats.saved_draw_calls > 0);
    }
}

