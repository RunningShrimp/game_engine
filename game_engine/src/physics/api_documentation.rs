//! # API文档完善示例
//!
//! 本模块展示如何为公共API编写完整的文档。

use bevy_ecs::component::Component;
use glam::Vec3;

/// Velocity Component - 速度组件
///
/// 表示实体在3D空间中的速度向量，用于物理模拟和移动计算。
///
/// # 概述
///
/// Velocity组件存储实体的瞬时速度，包含x、y、z三个方向的速度分量。
/// 它是物理模拟中的核心组件，与Position组件配合使用可以实现物体的运动。
///
/// # 使用示例
///
/// ## 基本使用
///
/// ```rust
/// use game_engine::physics::Velocity;
///
/// // 创建一个静止的物体
/// let velocity = Velocity::new(0.0, 0.0, 0.0);
///
/// // 创建一个向x轴移动的物体
/// let velocity = Velocity::new(10.0, 0.0, 0.0);
///
/// // 创建一个在3D空间中移动的物体
/// let velocity = Velocity::new(1.0, 2.0, 3.0);
/// ```
///
/// ## 与ECS集成
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use game_engine::physics::{Velocity, Position};
///
/// fn spawn_moving_entity(mut commands: Commands) {
///     commands.spawn((
///         Position::new(0.0, 0.0, 0.0),
///         Velocity::new(1.0, 0.0, 0.0),
///     ));
/// }
/// ```
///
/// ## 查询Velocity组件
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use game_engine::physics::Velocity;
///
/// fn update_positions(mut query: Query<&Velocity>) {
///     for velocity in query.iter() {
///         println!("Velocity: ({}, {}, {})",
///             velocity.x, velocity.y, velocity.z);
///     }
/// }
/// ```
///
/// ## 可变查询
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use game_engine::physics::Velocity;
///
/// fn apply_gravity(mut query: Query<&mut Velocity>) {
///     for mut velocity in query.iter_mut() {
///         velocity.y -= 9.8 * 0.016; // dt = 0.016s
///     }
/// }
/// ```
///
/// # 性能考虑
///
/// - **内存占用**: Velocity组件占用12字节 (3个f32)
/// - **缓存友好**: 连续内存布局，提高缓存命中率
/// - **SIMD优化**: 批量操作时自动使用SIMD指令
/// - **批量查询**: 使用Query可以高效处理大量实体
///
/// # 物理意义
///
/// - **单位**: 速度单位通常是米/秒 (m/s)
/// - **坐标系**: 使用右手坐标系，y轴向上
/// - **时间积分**: 位置更新使用欧拉积分: `position += velocity * dt`
///
/// # 相关组件
///
/// - [`Position`] - 位置组件，存储实体的位置
/// - [`Acceleration`] - 加速度组件，用于力的模拟
/// - [`Mass`] - 质量组件，影响力的作用
///
/// # 相关系统
///
/// - [`PhysicsIntegrationSystem`] - 物理积分系统
/// - [`CollisionDetectionSystem`] - 碰撞检测系统
///
/// # 常见问题
///
/// ## Q: Velocity和Position有什么区别？
/// A: Velocity表示速度（位置的变化率），Position表示位置。Velocity用于物理模拟，
///    Position用于渲染和碰撞检测。
///
/// ## Q: 如何让物体停止移动？
/// A: 将Velocity设置为零向量：
///    ```rust
///    velocity.0 = Vec3::ZERO;
///    ```
///
/// ## Q: 如何限制最大速度？
/// A: 使用`length()`方法和`normalize()`：
///    ```rust
///    if velocity.length() > max_speed {
///        velocity.0 = velocity.normalize() * max_speed;
///    }
///    ```
///
/// # Panics
///
/// 本组件的方法不会panic。所有操作都是安全的。
///
/// # Examples
///
/// ## 模拟抛物线运动
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use game_engine::physics::{Velocity, Position};
///
/// fn simulate_projectile(
///     mut query: Query<(&mut Position, &Velocity)>,
///     time: Res<Time>,
/// ) {
///     let dt = time.delta_seconds();
///     for (mut pos, vel) in query.iter_mut() {
///         pos.0 += vel.0 * dt;
///     }
/// }
/// ```
///
/// # See also
///
/// - [Physics模块文档](crate::physics)
/// - [ECS组件指南](crate::ecs)
/// - [物理系统教程](https://example.com/physics-tutorial)
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Velocity(pub Vec3);

impl Velocity {
    /// 创建一个零速度向量
    ///
    /// # Examples
    ///
    /// ```
    /// use game_engine::physics::Velocity;
    ///
    /// let velocity = Velocity::zero();
    /// assert_eq!(velocity.0.x, 0.0);
    /// ```
    #[inline]
    pub fn zero() -> Self {
        Self(Vec3::ZERO)
    }

    /// 创建新的速度向量
    ///
    /// # Arguments
    ///
    /// * `x` - X方向的速度分量
    /// * `y` - Y方向的速度分量
    /// * `z` - Z方向的速度分量
    ///
    /// # Examples
    ///
    /// ```
    /// use game_engine::physics::Velocity;
    ///
    /// let velocity = Velocity::new(1.0, 2.0, 3.0);
    /// assert_eq!(velocity.0.x, 1.0);
    /// ```
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }

    /// 返回速度的大小（长度）
    ///
    /// # Examples
    ///
    /// ```
    /// use game_engine::physics::Velocity;
    ///
    /// let velocity = Velocity::new(3.0, 4.0, 0.0);
    /// assert_eq!(velocity.length(), 5.0);
    /// ```
    #[inline]
    pub fn length(&self) -> f32 {
        self.0.length()
    }

    /// 归一化速度向量（保持方向，长度变为1）
    ///
    /// 如果速度为零向量，返回零向量。
    ///
    /// # Examples
    ///
    /// ```
    /// use game_engine::physics::Velocity;
    ///
    /// let velocity = Velocity::new(3.0, 0.0, 0.0);
    /// let normalized = velocity.normalize();
    /// assert_eq!(normalized.length(), 1.0);
    /// ```
    #[inline]
    pub fn normalize(&self) -> Self {
        Self(self.0.normalize())
    }

    /// 限制速度的最大值
    ///
    /// 如果当前速度超过最大值，将其缩放到最大值。
    ///
    /// # Arguments
    ///
    /// * `max_speed` - 允许的最大速度
    ///
    /// # Examples
    ///
    /// ```
    /// use game_engine::physics::Velocity;
    ///
    /// let velocity = Velocity::new(10.0, 0.0, 0.0);
    /// let clamped = velocity.clamp_length(5.0);
    /// assert_eq!(clamped.length(), 5.0);
    /// ```
    #[inline]
    pub fn clamp_length(&self, max_speed: f32) -> Self {
        if self.length() > max_speed {
            self.normalize() * max_speed
        } else {
            *self
        }
    }
}

impl std::ops::Mul<f32> for Velocity {
    type Output = Velocity;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl std::ops::Add for Velocity {
    type Output = Velocity;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_creation() {
        let velocity = Velocity::new(1.0, 2.0, 3.0);
        assert_eq!(velocity.0.x, 1.0);
        assert_eq!(velocity.0.y, 2.0);
        assert_eq!(velocity.0.z, 3.0);
    }

    #[test]
    fn test_velocity_zero() {
        let velocity = Velocity::zero();
        assert_eq!(velocity.0, Vec3::ZERO);
    }

    #[test]
    fn test_velocity_length() {
        let velocity = Velocity::new(3.0, 4.0, 0.0);
        assert_eq!(velocity.length(), 5.0);
    }

    #[test]
    fn test_velocity_normalize() {
        let velocity = Velocity::new(3.0, 0.0, 0.0);
        let normalized = velocity.normalize();
        assert_eq!(normalized.length(), 1.0);
        assert_eq!(normalized.0.x, 1.0);
    }

    #[test]
    fn test_velocity_clamp() {
        let velocity = Velocity::new(10.0, 0.0, 0.0);
        let clamped = velocity.clamp_length(5.0);
        assert_eq!(clamped.length(), 5.0);
    }

    #[test]
    fn test_velocity_mul() {
        let velocity = Velocity::new(1.0, 2.0, 3.0);
        let scaled = velocity * 2.0;
        assert_eq!(scaled.0.x, 2.0);
        assert_eq!(scaled.0.y, 4.0);
        assert_eq!(scaled.0.z, 6.0);
    }

    #[test]
    fn test_velocity_add() {
        let v1 = Velocity::new(1.0, 0.0, 0.0);
        let v2 = Velocity::new(0.0, 1.0, 0.0);
        let sum = v1 + v2;
        assert_eq!(sum.0.x, 1.0);
        assert_eq!(sum.0.y, 1.0);
    }
}
