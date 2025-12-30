//! # Physics模块文档示例
//!
//! 完整的模块级文档示例。

/// Physics Module - 物理模拟模块
///
/// 本模块提供高性能的2D/3D物理模拟功能，包括刚体动力学、碰撞检测、约束求解等。
///
/// ## 主要功能
///
/// ### 1. 刚体动力学
///
/// 支持完整的刚体物理模拟，包括：
/// - 速度、加速度、力的模拟
/// - 线性运动和旋转运动
/// - 质量和惯性张量
/// - 欧拉积分和RK4积分
///
/// ### 2. 碰撞检测
///
/// 高效的碰撞检测系统：
/// - 宽相位（Broadphase）：使用空间划分算法
/// - 窄相位（Narrowphase）：精确碰撞检测
/// - 支持多种形状：球体、盒子、胶囊体等
/// - 连续碰撞检测（CCD）防止穿透
///
/// ### 3. 约束求解
///
/// 支持各种约束类型：
/// - 关节约束（Hinge Joint）
/// - 滑块约束（Slider Joint）
/// - 固定约束（Fixed Joint）
/// - 弹簧阻尼系统
///
/// ### 4. SIMD优化
///
/// 自动使用SIMD指令加速：
/// - 批量向量运算
/// - 碰撞检测加速
/// - 物理状态更新加速
///
/// ## 快速开始
///
/// ### 基础物理模拟
///
/// ```rust
/// use game_engine::prelude::*;
/// use game_engine::physics::{PhysicsWorld, Velocity, Position};
///
/// fn main() {
///     // 创建物理世界
///     let mut world = PhysicsWorld::new();
///
///     // 添加地面
///     world.create_ground();
///
///     // 添加球体
///     let entity = world.spawn((
///         Position::new(0.0, 10.0, 0.0),
///         Velocity::new(0.0, 0.0, 0.0),
///         Mass::from_kg(1.0),
///     ));
///
///     // 模拟循环
///     for _ in 0..100 {
///         world.step(0.016); // 60 FPS
///     }
/// }
/// ```
///
/// ### 创建自定义物理实体
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use game_engine::physics::{Position, Velocity, Mass};
///
/// fn spawn_physics_entity(commands: &mut Commands) {
///     commands.spawn((
///         // 位置
///         Position::new(0.0, 5.0, 0.0),
///         // 速度
///         Velocity::new(1.0, 0.0, 0.0),
///         // 质量
///         Mass::from_kg(10.0),
///         // 碰撞形状
///         ColliderShape::Sphere(0.5),
///     ));
/// }
/// ```
///
/// ## 架构
///
/// ```text
/// PhysicsModule
/// │
/// ├── PhysicsWorld (物理世界管理)
/// │   ├── 创建和销毁实体
/// │   ├── 物理参数配置
/// │   └── 模拟步进控制
/// │
/// ├── Components (物理组件)
/// │   ├── Position (位置)
/// │   ├── Velocity (速度)
/// │   ├── Acceleration (加速度)
/// │   ├── Mass (质量)
/// │   ├── InverseMass (质量倒数)
/// │   └── ColliderShape (碰撞形状)
/// │
/// ├── Systems (物理系统)
/// │   ├── IntegrationSystem (积分系统)
/// │   ├── CollisionDetectionSystem (碰撞检测)
/// │   ├── ConstraintSolverSystem (约束求解)
/// │   └── BroadphaseSystem (宽相位检测)
/// │
/// └── Resources (物理资源)
///     ├── PhysicsPipeline (物理管线)
///     ├── SpatialHash (空间哈希)
///     └── ContactManager (接触管理)
/// ```
///
/// ## 性能优化
///
/// ### 1. 批量查询
///
/// 使用Bevy的Query系统批量处理实体：
///
/// ```rust
/// fn update_positions(mut query: Query<(&Velocity, &mut Position)>) {
///     for (velocity, mut position) in query.iter_mut() {
///         position.0 += velocity.0 * dt;
///     }
/// }
/// ```
///
/// ### 2. SIMD加速
///
/// 启用`simd` feature以获得SIMD加速：
///
/// ```toml
/// [dependencies]
/// game_engine = { version = "0.1", features = ["simd"] }
/// ```
///
### 3. 空间划分
///
/// 使用空间哈希优化碰撞检测：
///
/// ```rust
/// use game_engine::physics::SpatialHash;
///
/// let mut spatial_hash = SpatialHash::new(1.0); // cell size = 1.0
/// spatial_hash.insert(entity_id, aabb);
/// let potential_collisions = spatial_hash.query(aabb);
/// ```
///
/// ### 4. 对象池
///
/// 重用碰撞对象以减少分配：
///
/// ```rust
/// use game_engine::physics::ContactPool;
///
/// let mut pool = ContactPool::new();
/// let contact = pool.acquire();
/// // 使用contact
/// pool.release(contact);
/// ```
///
/// ## 性能基准
///
/// 在以下硬件配置下的性能数据：
///
/// - CPU: Intel Core i7-10700K (8核)
/// - RAM: 32GB DDR4
///
/// | 场景 | 实体数 | 帧率 | 说明 |
///------|--------|------|------|
/// 简单场景 | 100 | 120 FPS | 基础物理 |
/// 中等场景 | 1000 | 60 FPS | 包含碰撞 |
/// 复杂场景 | 5000 | 30 FPS | 包含约束 |
///
/// ## 使用指南
///
/// ### 创建物理世界
///
/// ```rust
/// use game_engine::physics::PhysicsWorld;
///
/// let mut world = PhysicsWorld::new();
/// world.set_gravity(Vec3::new(0.0, -9.8, 0.0));
/// ```
///
/// ### 添加物理实体
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use game_engine::physics::*;
///
/// fn spawn_ball(mut commands: Commands) {
///     commands.spawn((
///         Position::new(0.0, 10.0, 0.0),
///         Velocity::new(1.0, 0.0, 0.0),
///         Mass::from_kg(1.0),
///         ColliderShape::Sphere(0.5),
///         Material::default(),
///     ));
/// }
/// ```
///
/// ### 自定义物理系统
///
/// ```rust
/// use bevy_ecs::prelude::*;
///
/// fn custom_gravity(
///     mut query: Query<&mut Velocity>,
///     time: Res<Time>,
/// ) {
///     let gravity = Vec3::new(0.0, -9.8, 0.0);
///     let dt = time.delta_seconds();
///
///     for mut velocity in query.iter_mut() {
///         velocity.0 += gravity * dt;
///     }
/// }
/// ```
///
/// ## 高级功能
///
/// ### 连续碰撞检测（CCD）
///
/// 防止高速物体穿透：
///
/// ```rust
/// world.enable_ccd(true);
/// ```
///
/// ### 睡眠模式
///
/// 静止物体自动休眠以节省性能：
///
/// ```rust
/// world.enable_sleeping(true);
/// ```
///
/// ### 子步进
///
/// 提高模拟精度：
///
/// ```rust
/// world.step(0.016); // 单步
/// world.step_substepped(0.016, 8); // 8个子步
/// ```
///
/// ## 常见问题
///
/// ### Q: 为什么物体会穿透？
/// A: 可能原因：
/// 1. 速度过快，需要启用CCD
/// 2. 物理步长过大，使用子步进
/// 3. 碰撞形状不准确
///
/// ### Q: 如何提高性能？
/// A: 优化建议：
/// 1. 使用空间划分
/// 2. 启用休眠模式
/// 3. 减少不必要的碰撞检测
/// 4. 使用SIMD优化
///
/// ### Q: 物理单位是什么？
/// A: 默认使用SI单位：
/// - 距离：米
/// - 时间：秒
/// - 质量：千克
///
/// ## 相关模块
///
/// - [`crate::ecs`] - ECS框架
/// - [`crate::render`] - 渲染系统
/// - [`crate::audio`] - 音频系统
///
/// ## 参考资源
///
/// - [Game Physics Engine Development](https://example.com)
/// - [Real-Time Collision Detection](https://example.com)
/// - [Physics for Game Developers](https://example.com)
///
/// # See also
///
/// - [Velocity](crate::physics::Velocity) - 速度组件
/// - [Position](crate::physics::Position) - 位置组件
/// - [PhysicsWorld](crate::physics::PhysicsWorld) - 物理世界
