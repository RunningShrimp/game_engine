//! 物理脏标记追踪模块
//!
//! 通过脏标记机制优化物理系统与 Transform 的同步，
//! 仅同步发生变化的物体，减少不必要的数据传输。
//!
//! ## 优化效果
//!
//! - 静止物体零开销
//! - 休眠物体自动跳过
//! - 批量同步减少分散访问
//!
//! ## 使用示例
//!
//! ```ignore
//! // 使用优化后的同步系统
//! fn physics_sync_system(
//!     physics_state: Res<PhysicsState>,
//!     mut query: Query<(&RigidBodyComp, &mut Transform, &mut PhysicsDirty)>,
//! ) {
//!     for (rb, mut transform, mut dirty) in query.iter_mut() {
//!         if dirty.needs_sync() {
//!             // 执行同步...
//!             dirty.clear();
//!         }
//!     }
//! }
//! ```

use crate::impl_default;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

// ============================================================================
// 脏标记组件
// ============================================================================

/// 物理脏标记组件
///
/// 跟踪物理属性的变化状态，用于优化同步
#[derive(Component, Default, Clone, Copy)]
pub struct PhysicsDirty {
    /// Transform 已改变（需要同步到物理）
    pub transform_changed: bool,
    /// 物理状态已改变（需要同步到 Transform）
    pub physics_changed: bool,
    /// 速度已改变
    pub velocity_changed: bool,
    /// 上次同步的帧号
    pub last_sync_frame: u64,
}

impl PhysicsDirty {
    /// 创建新的脏标记
    pub fn new() -> Self {
        Self {
            transform_changed: false,
            physics_changed: false,
            velocity_changed: false,
            last_sync_frame: 0,
        }
    }

    /// 标记 Transform 已改变
    #[inline]
    pub fn mark_transform_changed(&mut self) {
        self.transform_changed = true;
    }

    /// 标记物理状态已改变
    #[inline]
    pub fn mark_physics_changed(&mut self) {
        self.physics_changed = true;
    }

    /// 标记速度已改变
    #[inline]
    pub fn mark_velocity_changed(&mut self) {
        self.velocity_changed = true;
    }

    /// 检查是否需要从 Transform 同步到物理
    #[inline]
    pub fn needs_physics_update(&self) -> bool {
        self.transform_changed
    }

    /// 检查是否需要从物理同步到 Transform
    #[inline]
    pub fn needs_transform_update(&self) -> bool {
        self.physics_changed
    }

    /// 清除所有脏标记
    #[inline]
    pub fn clear(&mut self) {
        self.transform_changed = false;
        self.physics_changed = false;
        self.velocity_changed = false;
    }

    /// 清除 Transform 脏标记
    #[inline]
    pub fn clear_transform(&mut self) {
        self.transform_changed = false;
    }

    /// 清除物理脏标记
    #[inline]
    pub fn clear_physics(&mut self) {
        self.physics_changed = false;
    }

    /// 更新同步帧号
    #[inline]
    pub fn update_frame(&mut self, frame: u64) {
        self.last_sync_frame = frame;
    }
}

// ============================================================================
// 物理同步状态
// ============================================================================

/// 缓存的物理状态（用于脏检测）
#[derive(Component, Clone, Copy)]
pub struct CachedPhysicsState {
    /// 缓存的位置
    pub position: Vec3,
    /// 缓存的旋转
    pub rotation: Quat,
    /// 缓存的线速度
    pub linear_velocity: Vec3,
    /// 缓存的角速度
    pub angular_velocity: f32,
    /// 是否休眠
    pub sleeping: bool,
}

impl_default!(CachedPhysicsState {
    position: Vec3::ZERO,
    rotation: Quat::IDENTITY,
    linear_velocity: Vec3::ZERO,
    angular_velocity: 0.0,
    sleeping: false,
});

impl CachedPhysicsState {
    /// 检测位置是否改变（使用阈值）
    pub fn position_changed(&self, new_pos: Vec3, threshold: f32) -> bool {
        (self.position - new_pos).length_squared() > threshold * threshold
    }

    /// 检测旋转是否改变（使用阈值）
    pub fn rotation_changed(&self, new_rot: Quat, threshold: f32) -> bool {
        self.rotation.angle_between(new_rot) > threshold
    }

    /// 更新缓存状态
    pub fn update(
        &mut self,
        position: Vec3,
        rotation: Quat,
        linear_vel: Vec3,
        angular_vel: f32,
        sleeping: bool,
    ) {
        self.position = position;
        self.rotation = rotation;
        self.linear_velocity = linear_vel;
        self.angular_velocity = angular_vel;
        self.sleeping = sleeping;
    }
}

// ============================================================================
// 同步配置
// ============================================================================

/// 物理同步配置
#[derive(Resource, Clone)]
pub struct PhysicsSyncConfig {
    /// 位置变化阈值（低于此值不同步）
    pub position_threshold: f32,
    /// 旋转变化阈值（弧度）
    pub rotation_threshold: f32,
    /// 是否跳过休眠体
    pub skip_sleeping: bool,
    /// 批量同步大小
    pub batch_size: usize,
    /// 是否启用脏检测
    pub dirty_tracking_enabled: bool,
}

impl_default!(PhysicsSyncConfig {
    position_threshold: 0.0001,
    rotation_threshold: 0.0001,
    skip_sleeping: true,
    batch_size: 256,
    dirty_tracking_enabled: true,
});

// ============================================================================
// 优化的同步系统
// ============================================================================

/// 优化的物理到 Transform 同步系统
///
/// 使用脏标记和休眠检测减少同步开销
///
/// **注意**: 此系统已更新为使用富领域对象架构
#[cfg(feature = "physics_2d")]
pub fn optimized_physics_sync_system(
    physics_service: Res<super::PhysicsDomainService>,
    config: Res<PhysicsSyncConfig>,
    mut query: Query<(
        &super::RigidBodyComp,
        &mut crate::ecs::Transform,
        Option<&mut PhysicsDirty>,
        Option<&mut CachedPhysicsState>,
    )>,
) {
    let world = physics_service.get_world();
    for (rb_comp, mut transform, dirty_opt, cached_opt) in query.iter_mut() {
        // 获取刚体状态（从富领域对象）
        let Some(body_state) = world.get_body_state(rb_comp.body_id) else {
            continue;
        };

        // 跳过休眠体（如果配置启用）
        if config.skip_sleeping && body_state.sleeping {
            continue;
        }

        let new_position = body_state.position;
        let new_rotation = body_state.rotation;

        // 使用脏检测
        if config.dirty_tracking_enabled {
            if let (Some(mut cached), Some(mut dirty)) = (cached_opt, dirty_opt) {
                // 检查是否真的改变了
                let pos_changed = cached.position_changed(new_position, config.position_threshold);
                let rot_changed = cached.rotation_changed(new_rotation, config.rotation_threshold);

                if !pos_changed && !rot_changed {
                    continue;
                }

                // 更新 Transform
                transform.pos = new_position;
                transform.rot = new_rotation;

                // 更新缓存
                cached.update(
                    new_position,
                    new_rotation,
                    body_state.linear_velocity,
                    body_state.angular_velocity,
                    body_state.sleeping,
                );

                // 清除脏标记
                dirty.clear_physics();
            } else {
                // 没有脏检测组件，直接同步
                transform.pos = new_position;
                transform.rot = new_rotation;
            }
        } else {
            // 禁用脏检测，直接同步
            transform.pos = new_position;
            transform.rot = new_rotation;
        }
    }
}

/// Transform 到物理的同步系统
///
/// 当 Transform 被外部修改时，同步到物理世界
///
/// **注意**: 此系统已更新为使用富领域对象架构
#[cfg(feature = "physics_2d")]
pub fn transform_to_physics_sync_system(
    mut physics_service: ResMut<super::PhysicsDomainService>,
    query: Query<
        (&super::RigidBodyComp, &crate::ecs::Transform, &PhysicsDirty),
        Changed<crate::ecs::Transform>,
    >,
) {
    for (rb_comp, transform, dirty) in query.iter() {
        // 只同步被外部修改的 Transform
        if !dirty.transform_changed {
            continue;
        }

        // 使用富领域对象API更新位置
        if let Err(e) = physics_service.set_body_position(rb_comp.body_id, transform.pos) {
            tracing::warn!(target: "physics", "Failed to sync transform to physics for body {}: {:?}", rb_comp.body_id.as_u64(), e);
        }
    }
}

// ============================================================================
// 批量同步辅助
// ============================================================================

/// 批量同步数据
pub struct BatchSyncData {
    /// 实体列表
    pub entities: Vec<Entity>,
    /// 位置列表
    pub positions: Vec<Vec3>,
    /// 旋转列表
    pub rotations: Vec<Quat>,
}

impl BatchSyncData {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            positions: Vec::with_capacity(capacity),
            rotations: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.positions.clear();
        self.rotations.clear();
    }

    pub fn push(&mut self, entity: Entity, position: Vec3, rotation: Quat) {
        self.entities.push(entity);
        self.positions.push(position);
        self.rotations.push(rotation);
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

// ============================================================================
// 统计信息
// ============================================================================

/// 物理同步统计
#[derive(Resource, Default, Clone, Copy)]
pub struct PhysicsSyncStats {
    /// 总同步次数
    pub total_syncs: u64,
    /// 跳过的休眠体数
    pub skipped_sleeping: u64,
    /// 跳过的未变化物体数
    pub skipped_unchanged: u64,
    /// 本帧同步数
    pub frame_syncs: u32,
}

impl PhysicsSyncStats {
    pub fn reset_frame(&mut self) {
        self.frame_syncs = 0;
    }

    pub fn record_sync(&mut self) {
        self.total_syncs += 1;
        self.frame_syncs += 1;
    }

    pub fn record_skip_sleeping(&mut self) {
        self.skipped_sleeping += 1;
    }

    pub fn record_skip_unchanged(&mut self) {
        self.skipped_unchanged += 1;
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_dirty_default() {
        let dirty = PhysicsDirty::default();
        assert!(!dirty.transform_changed);
        assert!(!dirty.physics_changed);
        assert!(!dirty.velocity_changed);
    }

    #[test]
    fn test_physics_dirty_mark_and_clear() {
        let mut dirty = PhysicsDirty::new();

        dirty.mark_transform_changed();
        assert!(dirty.needs_physics_update());

        dirty.clear_transform();
        assert!(!dirty.needs_physics_update());

        dirty.mark_physics_changed();
        assert!(dirty.needs_transform_update());

        dirty.clear();
        assert!(!dirty.needs_transform_update());
    }

    #[test]
    fn test_cached_physics_state_position_changed() {
        let cached = CachedPhysicsState {
            position: Vec3::ZERO,
            ..Default::default()
        };

        // 小变化不应触发
        assert!(!cached.position_changed(Vec3::new(0.00001, 0.0, 0.0), 0.0001));

        // 大变化应触发
        assert!(cached.position_changed(Vec3::new(1.0, 0.0, 0.0), 0.0001));
    }

    #[test]
    fn test_physics_sync_config_default() {
        let config = PhysicsSyncConfig::default();
        assert!(config.skip_sleeping);
        assert!(config.dirty_tracking_enabled);
        assert_eq!(config.batch_size, 256);
    }
}
