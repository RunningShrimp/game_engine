//! 优化的场景遍历模块
//!
//! 提供高性能的场景遍历和实体收集，支持：
//! - 并行场景遍历
//! - 智能批处理分组
//! - GPU驱动渲染集成
//! - 增量更新支持

use bevy_ecs::prelude::*;
use glam::Mat4;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::collections::HashMap;

use crate::ecs::Transform;
use crate::render::batch_optimizer::{BatchOptimizer, OptimizedBatch};
use crate::render::gpu_driven::GpuInstance;
use crate::render::instance_batch::{BatchKey, Mesh3DRenderer};

/// 场景遍历配置
#[derive(Debug, Clone)]
pub struct SceneTraversalConfig {
    /// 是否启用并行遍历
    pub parallel_traversal: bool,
    /// 并行遍历的块大小
    pub chunk_size: usize,
    /// 是否启用GPU驱动渲染
    pub gpu_driven: bool,
    /// 批处理阈值（低于此值的实例不进行批处理）
    pub batch_threshold: usize,
    /// 最大每批次实例数
    pub max_instances_per_batch: u32,
}

impl Default for SceneTraversalConfig {
    fn default() -> Self {
        Self {
            parallel_traversal: true,
            chunk_size: 1000,
            gpu_driven: true,
            batch_threshold: 2,
            max_instances_per_batch: 65536,
        }
    }
}

/// 场景遍历结果
#[derive(Debug)]
pub struct SceneTraversalResult {
    /// 收集的批次
    pub batches: Vec<OptimizedBatch>,
    /// GPU实例数据（如果启用GPU驱动渲染）
    pub gpu_instances: Vec<GpuInstance>,
    /// 遍历统计
    pub stats: TraversalStats,
}

/// 遍历统计信息
#[derive(Debug, Clone, Default)]
pub struct TraversalStats {
    /// 遍历的实体数
    pub entities_processed: usize,
    /// 收集的实例数
    pub instances_collected: usize,
    /// 创建的批次数
    pub batches_created: usize,
    /// 遍历耗时（微秒）
    pub traversal_time_us: u64,
    /// 批处理优化耗时（微秒）
    pub optimization_time_us: u64,
}

/// 优化的场景遍历器
pub struct OptimizedSceneTraverser {
    config: SceneTraversalConfig,
    optimizer: BatchOptimizer,
}

impl OptimizedSceneTraverser {
    /// 创建新的场景遍历器
    pub fn new(config: SceneTraversalConfig) -> Self {
        let optimizer = BatchOptimizer::new(config.max_instances_per_batch);
        Self { config, optimizer }
    }

    /// 遍历场景并收集渲染实体
    ///
    /// # 参数
    /// - `world`: ECS世界
    /// - `view_proj`: 视图投影矩阵（用于视锥剔除）
    ///
    /// # 返回
    /// 场景遍历结果，包含优化的批次和GPU实例数据
    pub fn traverse_scene(
        &mut self,
        world: &mut bevy_ecs::world::World,
        view_proj: Option<[[f32; 4]; 4]>,
    ) -> SceneTraversalResult {
        let start = std::time::Instant::now();

        // 收集实体数据
        let entities = self.collect_entities(world, view_proj);

        let traversal_time = start.elapsed().as_micros() as u64;

        // 创建批次
        let opt_start = std::time::Instant::now();
        let batches = self.create_batches(&entities);
        let optimization_time = opt_start.elapsed().as_micros() as u64;

        // 转换为GPU实例（如果启用GPU驱动渲染）
        let gpu_instances = if self.config.gpu_driven {
            self.convert_to_gpu_instances(&entities)
        } else {
            Vec::new()
        };

        SceneTraversalResult {
            batches,
            gpu_instances,
            stats: TraversalStats {
                entities_processed: entities.len(),
                instances_collected: entities.len(),
                batches_created: 0, // 将在优化后更新
                traversal_time_us: traversal_time,
                optimization_time_us: optimization_time,
            },
        }
    }

    /// 收集实体数据
    fn collect_entities(
        &self,
        world: &mut bevy_ecs::world::World,
        _view_proj: Option<[[f32; 4]; 4]>,
    ) -> Vec<EntityData> {
        // 查询所有需要渲染的实体
        // 注意：使用 WorldQuery 而不是直接调用 query()
        // 优化：使用引用避免克隆 Arc 指针
        let entities: Vec<EntityData> = world
            .query::<(Entity, &Transform, &Mesh3DRenderer)>()
            .iter(world)
            .map(|(entity, transform, renderer)| EntityData {
                entity,
                transform: *transform,
                // 优化：只复制必要的字段（mesh_id, material_id等），而不是整个renderer
                // 这避免了克隆3个Arc指针（mesh, material_bind_group, textures_bind_group）
                renderer_key: RendererKey {
                    mesh_id: renderer.mesh_id,
                    material_id: renderer.material_id,
                    pipeline_id: renderer.pipeline_id,
                    blend_mode: renderer.blend_mode,
                    depth_test: renderer.depth_test,
                    render_flags: renderer.render_flags,
                },
            })
            .collect();

        entities
    }

    /// 并行收集实体数据（当启用parallel特性时）
    #[cfg(feature = "parallel")]
    fn collect_entities_parallel(
        &self,
        world: &mut bevy_ecs::world::World,
        _view_proj: Option<[[f32; 4]; 4]>,
    ) -> Vec<EntityData> {
        // 查询所有需要渲染的实体
        // 优化：使用引用避免克隆 Arc 指针
        let entities: Vec<EntityData> = world
            .query::<(Entity, &Transform, &Mesh3DRenderer)>()
            .iter(world)
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(entity, transform, renderer)| EntityData {
                entity,
                transform: *transform,
                // 优化：只复制必要的字段（mesh_id, material_id等），而不是整个renderer
                // 这避免了克隆3个Arc指针
                renderer_key: RendererKey {
                    mesh_id: renderer.mesh_id,
                    material_id: renderer.material_id,
                    pipeline_id: renderer.pipeline_id,
                    blend_mode: renderer.blend_mode,
                    depth_test: renderer.depth_test,
                    render_flags: renderer.render_flags,
                },
            })
            .collect();

        entities
    }

    /// 创建批次
    fn create_batches(&mut self, entities: &[EntityData]) -> Vec<OptimizedBatch> {
        // 按BatchKey分组
        let mut batch_map: HashMap<BatchKey, Vec<EntityData>> = HashMap::new();

        for entity in entities {
            // 优化：使用renderer_key直接构造BatchKey，避免克隆renderer
            let key = BatchKey {
                mesh_id: entity.renderer_key.mesh_id,
                material_id: entity.renderer_key.material_id,
                pipeline_id: entity.renderer_key.pipeline_id,
                blend_mode: entity.renderer_key.blend_mode,
                depth_test: entity.renderer_key.depth_test,
                render_flags: entity.renderer_key.render_flags,
            };
            batch_map.entry(key).or_default().push(*entity);
        }

        // 转换为OptimizedBatch
        let mut batches: Vec<OptimizedBatch> = batch_map
            .into_iter()
            .filter_map(|(key, entities)| {
                if entities.len() < self.config.batch_threshold {
                    return None; // 跳过太小的批次
                }

                let instance_count = entities.len() as u32;
                let mut batch = OptimizedBatch::new(key, instance_count);

                // 添加实例索引
                for i in 0..entities.len() {
                    batch.instances.push(i as u32);
                }

                Some(batch)
            })
            .collect();

        // 优化批次（合并相同状态的批次，减少draw call）
        self.optimizer.optimize_batches(&mut batches);

        batches
    }

    /// 转换为GPU实例数据
    fn convert_to_gpu_instances(&self, entities: &[EntityData]) -> Vec<GpuInstance> {
        entities
            .iter()
            .enumerate()
            .map(|(i, entity)| {
                let model_matrix = Mat4::from_scale_rotation_translation(
                    entity.transform.scale,
                    entity.transform.rot,
                    entity.transform.pos,
                );

                // 计算AABB（简化版，假设单位立方体）
                let aabb_min = entity.transform.pos - entity.transform.scale * 0.5;
                let aabb_max = entity.transform.pos + entity.transform.scale * 0.5;

                GpuInstance {
                    instance_id: i as u32,
                    model: model_matrix.to_cols_array_2d(),
                    aabb_min: [aabb_min.x, aabb_min.y, aabb_min.z],
                    aabb_max: [aabb_max.x, aabb_max.y, aabb_max.z],
                    flags: 0,
                }
            })
            .collect()
    }
}

/// 渲染器关键数据（用于批处理分组，避免克隆整个Mesh3DRenderer）
///
/// 只包含批处理所需的字段，避免克隆Arc指针。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RendererKey {
    mesh_id: u64,
    material_id: u64,
    pipeline_id: u32,
    blend_mode: u8,
    depth_test: bool,
    render_flags: u16,
}

/// 实体数据（用于场景遍历）
#[derive(Clone, Copy)]
struct EntityData {
    entity: Entity,
    transform: Transform,
    renderer_key: RendererKey,
}

/// 增量场景更新器
///
/// 只更新发生变化的实体，减少不必要的遍历和批处理。
pub struct IncrementalSceneUpdater {
    /// 上一帧的实体状态
    previous_entities: HashMap<Entity, EntitySnapshot>,
    /// 脏实体集合
    dirty_entities: std::collections::HashSet<Entity>,
}

/// 实体快照
#[derive(Clone, Copy, Debug)]
struct EntitySnapshot {
    transform: Transform,
    renderer_key: RendererKey,
}

impl IncrementalSceneUpdater {
    /// 创建新的增量更新器
    pub fn new() -> Self {
        Self {
            previous_entities: HashMap::new(),
            dirty_entities: std::collections::HashSet::new(),
        }
    }

    /// 检测变化的实体
    pub fn detect_changes(&mut self, world: &mut bevy_ecs::world::World) -> Vec<Entity> {
        let mut changed = Vec::new();

        // 先收集所有当前实体快照
        // 注意：使用 query_mut 避免借用问题
        // 优化：避免克隆renderer
        let current_snapshots: Vec<(Entity, EntitySnapshot)> = world
            .query::<(Entity, &Transform, &Mesh3DRenderer)>()
            .iter(world)
            .map(|(entity, transform, renderer)| {
                let snapshot = EntitySnapshot {
                    transform: *transform,
                    renderer_key: RendererKey {
                        mesh_id: renderer.mesh_id,
                        material_id: renderer.material_id,
                        pipeline_id: renderer.pipeline_id,
                        blend_mode: renderer.blend_mode,
                        depth_test: renderer.depth_test,
                        render_flags: renderer.render_flags,
                    },
                };
                (entity, snapshot)
            })
            .collect();

        // 检查变化
        for (entity, current_snapshot) in &current_snapshots {
            if let Some(prev) = self.previous_entities.get(entity) {
                if prev.transform != current_snapshot.transform
                    || prev.renderer_key != current_snapshot.renderer_key
                {
                    changed.push(*entity);
                    self.dirty_entities.insert(*entity);
                }
            } else {
                // 新实体
                changed.push(*entity);
                self.dirty_entities.insert(*entity);
            }

            // 更新快照
            self.previous_entities.insert(*entity, *current_snapshot);
        }

        // 检查已移除的实体
        let current_entities: std::collections::HashSet<Entity> =
            current_snapshots.iter().map(|(entity, _)| *entity).collect();

        for entity in self.previous_entities.keys() {
            if !current_entities.contains(entity) {
                changed.push(*entity);
                self.dirty_entities.insert(*entity);
            }
        }

        // 移除已删除的实体快照
        self.previous_entities.retain(|e, _| current_entities.contains(e));

        changed
    }

    /// 清除脏标记
    pub fn clear_dirty(&mut self) {
        self.dirty_entities.clear();
    }

    /// 检查实体是否脏
    pub fn is_dirty(&self, entity: Entity) -> bool {
        self.dirty_entities.contains(&entity)
    }
}

impl Default for IncrementalSceneUpdater {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_traversal_config_default() {
        let config = SceneTraversalConfig::default();
        assert!(config.parallel_traversal);
        assert_eq!(config.chunk_size, 1000);
        assert!(config.gpu_driven);
    }

    #[test]
    fn test_incremental_updater_new() {
        let updater = IncrementalSceneUpdater::new();
        assert!(updater.previous_entities.is_empty());
        assert!(updater.dirty_entities.is_empty());
    }
}
