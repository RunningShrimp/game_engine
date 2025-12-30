//! 物理组件统一trait
//!
//! 此模块定义了物理组件的统一接口，用于：
//! - 统一不同物理组件的访问方式
//! - 减少代码重复
//! - 提供类型安全的物理组件操作
//!
//! # 设计目标
//!
//! 1. **统一接口**: 为所有物理组件提供一致的访问方式
//! 2. **类型安全**: 通过trait确保类型安全的物理操作
//! 3. **性能优化**: 支持批量操作和SIMD优化
//!
//! # 支持的组件
//!
//! - `RigidBodyComp`: 刚体组件
//! - `ColliderComp`: 碰撞体组件
//! - `SoftBodyComponent`: 软体组件
//! - `Position`, `Velocity`: 速度组件
//!
//! # 使用示例
//!
//! ```rust
//! use game_engine::physics::traits::PhysicsComponent;
//! use bevy_ecs::prelude::*;
//!
//! fn sync_physics_system(query: Query<&impl PhysicsComponent>) {
//!     for component in query.iter() {
//!         if component.is_enabled() {
//!             let pos = component.get_position();
//!             // 同步位置...
//!         }
//!     }
//! }
//! ```

use bevy_ecs::prelude::*;
use bevy_ecs::query::{QueryData, WorldQuery};
use glam::{Quat, Vec3};
use std::fmt::Debug;

// =============================================================================
// PhysicsComponent Trait
// =============================================================================

/// 物理组件统一trait
///
/// 为所有物理组件提供统一的访问接口，支持：
/// - 位置和旋转访问
/// - 速度访问
/// - 启用/禁用状态
/// - 组件元数据
///
/// # 泛型
///
/// 此trait不需要泛型参数，具体类型通过方法返回
///
/// # 示例
///
/// ```rust
/// use game_engine::physics::traits::PhysicsComponent;
/// ```
pub trait PhysicsComponent: Send + Sync + Debug {
    /// 获取位置
    ///
    /// # 返回
    ///
    /// 返回组件的当前位置（如果是None表示无效）
    fn get_position(&self) -> Option<Vec3>;

    /// 获取旋转
    ///
    /// # 返回
    ///
    /// 返回组件的当前旋转（如果是None表示无效）
    fn get_rotation(&self) -> Option<Quat>;

    /// 获取线速度
    ///
    /// # 返回
    ///
    /// 返回组件的当前线速度（如果是None表示无效）
    fn get_linear_velocity(&self) -> Option<Vec3>;

    /// 获取角速度
    ///
    /// # 返回
    ///
    /// 返回组件的当前角速度（如果是None表示无效）
    fn get_angular_velocity(&self) -> Option<Vec3>;

    /// 检查组件是否启用
    ///
    /// # 返回
    ///
    /// 如果组件启用参与物理模拟，返回 `true`
    fn is_enabled(&self) -> bool;

    /// 检查组件是否休眠
    ///
    /// # 返回
    ///
    /// 如果组件处于休眠状态（不参与模拟），返回 `true`
    fn is_sleeping(&self) -> bool;

    /// 获取组件类型名称
    ///
    /// # 返回
    ///
    /// 返回组件的类型名称（用于调试和日志）
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

// =============================================================================
// PhysicsComponentMut Trait
// =============================================================================

/// 可变物理组件trait
///
/// 提供修改物理组件状态的能力
pub trait PhysicsComponentMut: PhysicsComponent {
    /// 设置位置
    fn set_position(&mut self, position: Vec3);

    /// 设置旋转
    fn set_rotation(&mut self, rotation: Quat);

    /// 设置线速度
    fn set_linear_velocity(&mut self, velocity: Vec3);

    /// 设置角速度
    fn set_angular_velocity(&mut self, velocity: Vec3);

    /// 启用/禁用组件
    fn set_enabled(&mut self, enabled: bool);

    /// 唤醒组件（如果休眠）
    fn wake_up(&mut self);

    /// 休眠组件
    fn sleep(&mut self);
}

// =============================================================================
// ECS组件集成
// =============================================================================

// 为RigidBodyComp实现PhysicsComponent（需要在mod.rs中定义）
// 这里提供实现示例，实际实现需要在组件定义处

// 为Position组件实现PhysicsComponent
impl PhysicsComponent for crate::physics::velocity_components::Position {
    fn get_position(&self) -> Option<Vec3> {
        Some(self.0)
    }

    fn get_rotation(&self) -> Option<Quat> {
        None // Position组件没有旋转
    }

    fn get_linear_velocity(&self) -> Option<Vec3> {
        None // Position组件没有速度
    }

    fn get_angular_velocity(&self) -> Option<Vec3> {
        None
    }

    fn is_enabled(&self) -> bool {
        true // Position组件始终启用
    }

    fn is_sleeping(&self) -> bool {
        false
    }

    fn type_name(&self) -> &'static str {
        "Position"
    }
}

impl PhysicsComponentMut for crate::physics::velocity_components::Position {
    fn set_position(&mut self, position: Vec3) {
        self.0 = position;
    }

    fn set_rotation(&mut self, _rotation: Quat) {
        // Position组件不支持旋转
    }

    fn set_linear_velocity(&mut self, _velocity: Vec3) {
        // Position组件不支持速度
    }

    fn set_angular_velocity(&mut self, _velocity: Vec3) {
        // Position组件不支持角速度
    }

    fn set_enabled(&mut self, _enabled: bool) {
        // Position组件始终启用
    }

    fn wake_up(&mut self) {
        // Position组件不支持休眠
    }

    fn sleep(&mut self) {
        // Position组件不支持休眠
    }
}

// 为InverseMass组件实现PhysicsComponent
impl PhysicsComponent for crate::physics::velocity_components::InverseMass {
    fn get_position(&self) -> Option<Vec3> {
        None
    }

    fn get_rotation(&self) -> Option<Quat> {
        None
    }

    fn get_linear_velocity(&self) -> Option<Vec3> {
        None
    }

    fn get_angular_velocity(&self) -> Option<Vec3> {
        None
    }

    fn is_enabled(&self) -> bool {
        self.0 > 0.0 // 有效质量表示启用
    }

    fn is_sleeping(&self) -> bool {
        self.0 == 0.0 // 零逆质量（无限质量）表示静态
    }

    fn type_name(&self) -> &'static str {
        "InverseMass"
    }
}

impl PhysicsComponentMut for crate::physics::velocity_components::InverseMass {
    fn set_position(&mut self, _position: Vec3) {
        // InverseMass不支持位置
    }

    fn set_rotation(&mut self, _rotation: Quat) {
        // InverseMass不支持旋转
    }

    fn set_linear_velocity(&mut self, _velocity: Vec3) {
        // InverseMass不支持速度
    }

    fn set_angular_velocity(&mut self, _velocity: Vec3) {
        // InverseMass不支持角速度
    }

    fn set_enabled(&mut self, _enabled: bool) {
        // InverseMass通过质量值控制
    }

    fn wake_up(&mut self) {
        // InverseMass不支持休眠
    }

    fn sleep(&mut self) {
        // InverseMass不支持休眠
    }
}

// 为Velocity组件实现PhysicsComponent
impl PhysicsComponent for crate::physics::velocity_components::Velocity {
    fn get_position(&self) -> Option<Vec3> {
        None // Velocity组件没有位置
    }

    fn get_rotation(&self) -> Option<Quat> {
        None
    }

    fn get_linear_velocity(&self) -> Option<Vec3> {
        Some(self.0)
    }

    fn get_angular_velocity(&self) -> Option<Vec3> {
        None // Velocity组件只有线速度
    }

    fn is_enabled(&self) -> bool {
        true // Velocity组件始终启用
    }

    fn is_sleeping(&self) -> bool {
        false
    }

    fn type_name(&self) -> &'static str {
        "Velocity"
    }
}

impl PhysicsComponentMut for crate::physics::velocity_components::Velocity {
    fn set_position(&mut self, _position: Vec3) {
        // Velocity组件不支持设置位置
    }

    fn set_rotation(&mut self, _rotation: Quat) {
        // Velocity组件不支持旋转
    }

    fn set_linear_velocity(&mut self, velocity: Vec3) {
        self.0 = velocity;
    }

    fn set_angular_velocity(&mut self, _velocity: Vec3) {
        // Velocity组件只有线速度，不支持角速度
    }

    fn set_enabled(&mut self, _enabled: bool) {
        // Velocity组件始终启用
    }

    fn wake_up(&mut self) {
        // Velocity组件不支持休眠
    }

    fn sleep(&mut self) {
        // Velocity组件不支持休眠
    }
}

// 为 GlobalTransform 组件实现 PhysicsComponent（简化版）
impl PhysicsComponent for crate::physics::velocity_components::GlobalTransform {
    fn get_position(&self) -> Option<Vec3> {
        // 从变换矩阵提取位置
        Some(self.0.transform_point3(glam::Vec3::ZERO))
    }

    fn get_rotation(&self) -> Option<Quat> {
        // 从变换矩阵提取旋转（简化实现）
        None // 完整实现需要从矩阵提取四元数
    }

    fn get_linear_velocity(&self) -> Option<Vec3> {
        None
    }

    fn get_angular_velocity(&self) -> Option<Vec3> {
        None
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn is_sleeping(&self) -> bool {
        false
    }

    fn type_name(&self) -> &'static str {
        "GlobalTransform"
    }
}

impl PhysicsComponentMut for crate::physics::velocity_components::GlobalTransform {
    fn set_position(&mut self, position: Vec3) {
        self.0.w_axis.x = position.x;
        self.0.w_axis.y = position.y;
        self.0.w_axis.z = position.z;
    }

    fn set_rotation(&mut self, _rotation: Quat) {
        // 完整实现需要更新矩阵的上部 3x3 部分
    }

    fn set_linear_velocity(&mut self, _velocity: Vec3) {
        // GlobalTransform 不支持速度
    }

    fn set_angular_velocity(&mut self, _velocity: Vec3) {
        // GlobalTransform 不支持角速度
    }

    fn set_enabled(&mut self, _enabled: bool) {
        // GlobalTransform 始终启用
    }

    fn wake_up(&mut self) {
        // GlobalTransform 不支持休眠
    }

    fn sleep(&mut self) {
        // GlobalTransform 不支持休眠
    }
}

// =============================================================================
// PhysicsQuery Helper
// =============================================================================

/// 物理查询辅助结构
///
/// 提供类型安全的物理组件批量查询
pub struct PhysicsQuery<'w, 's, Q: QueryData> {
    query: Query<'w, 's, Q>,
}

impl<'w, 's, Q: QueryData> PhysicsQuery<'w, 's, Q> {
    /// 从Query创建物理查询（简化版）
    pub fn from_query(query: Query<'w, 's, Q>) -> Self {
        Self { query }
    }
}

// =============================================================================
// 批量操作辅助函数
// =============================================================================

/// 批量获取所有组件的位置
///
/// # 参数
///
/// - `components`: 物理组件切片
///
/// # 返回
///
/// 返回位置向量（None表示组件没有位置）
pub fn batch_get_positions<T: PhysicsComponent>(components: &[T]) -> Vec<Option<Vec3>> {
    components.iter().map(|c| c.get_position()).collect()
}

/// 批量获取所有组件的速度
///
/// # 参数
///
/// - `components`: 物理组件切片
///
/// # 返回
///
/// 返回线速度向量（None表示组件没有速度）
pub fn batch_get_velocities<T: PhysicsComponent>(components: &[T]) -> Vec<Option<Vec3>> {
    components.iter().map(|c| c.get_linear_velocity()).collect()
}

/// 批量检查组件是否启用
///
/// # 参数
///
/// - `components`: 物理组件切片
///
/// # 返回
///
/// 返回启用状态向量
pub fn batch_check_enabled<T: PhysicsComponent>(components: &[T]) -> Vec<bool> {
    components.iter().map(|c| c.is_enabled()).collect()
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_component() {
        let pos = crate::physics::Position(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(pos.get_position(), Some(Vec3::new(1.0, 2.0, 3.0)));
        assert!(pos.is_enabled());
        assert!(!pos.is_sleeping());
        assert_eq!(pos.type_name(), "Position");
    }

    #[test]
    fn test_position_component_mut() {
        let mut pos = crate::physics::Position(Vec3::ZERO);

        pos.set_position(Vec3::new(5.0, 6.0, 7.0));
        assert_eq!(pos.get_position(), Some(Vec3::new(5.0, 6.0, 7.0)));
    }

    #[test]
    fn test_velocity_component() {
        let vel = crate::physics::Velocity(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(vel.get_linear_velocity(), Some(Vec3::new(1.0, 2.0, 3.0)));
        assert!(vel.is_enabled());
    }

    #[test]
    fn test_velocity_component_mut() {
        let mut vel = crate::physics::Velocity(Vec3::ZERO);

        vel.set_linear_velocity(Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(vel.get_linear_velocity(), Some(Vec3::new(10.0, 20.0, 30.0)));
    }

    #[test]
    fn test_batch_operations() {
        let positions = vec![
            crate::physics::Position(Vec3::new(1.0, 0.0, 0.0)),
            crate::physics::Position(Vec3::new(0.0, 1.0, 0.0)),
            crate::physics::Position(Vec3::new(0.0, 0.0, 1.0)),
        ];

        let result = batch_get_positions(&positions);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Some(Vec3::new(1.0, 0.0, 0.0)));
        assert_eq!(result[1], Some(Vec3::new(0.0, 1.0, 0.0)));
        assert_eq!(result[2], Some(Vec3::new(0.0, 0.0, 1.0)));
    }

    #[test]
    fn test_batch_check_enabled() {
        let positions = vec![
            crate::physics::Position(Vec3::ZERO),
            crate::physics::Position(Vec3::X),
            crate::physics::Position(Vec3::Y),
        ];

        let enabled = batch_check_enabled(&positions);
        assert_eq!(enabled, vec![true, true, true]);
    }
}
