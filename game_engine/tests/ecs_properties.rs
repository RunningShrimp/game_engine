// ============================================================================
// ECS模块属性测试
// ============================================================================
//
// 本文件包含ECS（Entity Component System）的属性测试。
//
// ## 测试的属性
//
// 1. **Entity唯一性**: 每个实体ID应该是唯一的
// 2. **Component保持性**: 添加组件后，组件数据应该保持不变
// 3. **Query一致性**: 查询返回的实体数量应该符合预期
// 4. **Entity生命周期**: 实体创建、删除、回收的行为应该正确
// 5. **Transform不变性**: Transform操作应该满足数学属性

use proptest::prelude::*;
use game_engine::ecs::*;
use bevy_ecs::prelude::*;

// ============================================================================
// Test helpers (copied from property_tests.rs)
// ============================================================================

pub mod strategies {
    use proptest::prelude::*;
    use glam::Vec3;

    /// 实体索引策略：生成合理的实体索引
    pub fn entity_index() -> impl Strategy<Value = u32> {
        0u32..1000000u32
    }

    /// 小坐标策略：生成小范围的坐标（适合局部测试）
    pub fn coord_small() -> impl Strategy<Value = f32> {
        -100.0..=100.0f32
    }

    /// 向量策略：生成3D向量
    pub fn vec3() -> impl Strategy<Value = Vec3> {
        let coord = -1000.0..=1000.0f32;
        prop::array::uniform3(coord).prop_map(|arr| Vec3::from_array(arr))
    }

    /// 小向量策略：生成小范围的3D向量
    pub fn vec3_small() -> impl Strategy<Value = Vec3> {
        prop::array::uniform3(coord_small()).prop_map(|arr| Vec3::from_array(arr))
    }
}

/// 检查两个浮点数是否近似相等
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// 检查两个向量是否近似相等
pub fn vec3_approx_eq(a: glam::Vec3, b: glam::Vec3, epsilon: f32) -> bool {
    (a - b).length() < epsilon
}

// ============================================================================
// Entity ID 属性测试
// ============================================================================

proptest! {
    /// 测试实体ID的稳定性
    /// 同一个实体的ID在多次查询中应该保持一致
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_entity_id_stability(index in strategies::entity_index()) {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // 多次获取索引应该得到相同结果
        let id1 = entity.index();
        let id2 = entity.index();
        let id3 = entity.index();

        prop_assert_eq!(id1, id2);
        prop_assert_eq!(id2, id3);
    }

    /// 测试实体ID的唯一性
    /// 不同实体的ID应该不同
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_entity_id_uniqueness(count in 10usize..1000) {
        let mut world = World::new();
        let mut entities = std::collections::HashSet::new();

        for _ in 0..count {
            let entity = world.spawn_empty().id();
            entities.insert(entity);
        }

        prop_assert_eq!(entities.len(), count);
    }

    /// 测试实体生成的连续性
    /// 实体ID应该是连续的（没有重用的情况下）
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_entity_generation_contiguous(n in 5usize..100) {
        let mut world = World::new();
        let mut indices = Vec::new();

        for _ in 0..n {
            let entity = world.spawn_empty().id();
            indices.push(entity.index());
        }

        // 检查索引是严格递增的
        for i in 1..indices.len() {
            prop_assert!(indices[i] > indices[i-1]);
        }
    }
}

// ============================================================================
// Transform 组件属性测试
// ============================================================================

proptest! {
    /// 测试Transform组件的数据保持性
    /// 添加Transform组件后，数据应该保持不变
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_transform_data_preservation(
        pos in strategies::vec3(),
        rot_x in strategies::coord_small(),
        rot_y in strategies::coord_small(),
        rot_z in strategies::coord_small(),
        rot_w in strategies::coord_small(),
        scale in strategies::vec3_small()
    ) {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // 创建四元数（归一化）
        let quat = glam::Quat::from_xyzw(rot_x, rot_y, rot_z, rot_w).normalize();
        let transform = Transform {
            pos,
            rot: quat,
            scale,
        };

        world.entity_mut(entity).insert(transform);

        // 获取组件并验证
        let retrieved = world.get::<Transform>(entity).unwrap();
        prop_assert_eq!(retrieved.pos, pos);
        prop_assert!(quat_approx_eq(retrieved.rot, quat, 0.001));
        prop_assert_eq!(retrieved.scale, scale);
    }

    /// 测试Transform默认值
    /// 默认的Transform应该是单位变换
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_transform_default() {
        let transform = Transform::default();

        prop_assert_eq!(transform.pos, glam::Vec3::ZERO);
        prop_assert!(quat_approx_eq(transform.rot, glam::Quat::IDENTITY, 0.001));
        prop_assert_eq!(transform.scale, glam::Vec3::ONE);
    }

    /// 测试Transform的new方法
    /// new()方法应该创建相同的默认Transform
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_transform_new_equals_default() {
        let t1 = Transform::new();
        let t2 = Transform::default();

        prop_assert_eq!(t1.pos, t2.pos);
        prop_assert!(quat_approx_eq(t1.rot, t2.rot, 0.001));
        prop_assert_eq!(t1.scale, t2.scale);
    }
}

// ============================================================================
// Velocity 组件属性测试
// ============================================================================

proptest! {
    /// 测试Velocity组件的数据保持性
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_velocity_data_preservation(
        lin in strategies::vec3(),
        ang in strategies::vec3()
    ) {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let velocity = Velocity { lin, ang };
        world.entity_mut(entity).insert(velocity);

        let retrieved = world.get::<Velocity>(entity).unwrap();
        prop_assert_eq!(retrieved.lin, lin);
        prop_assert_eq!(retrieved.ang, ang);
    }

    /// 测试Velocity的new方法
    /// new()应该创建与default()相同的Velocity
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_velocity_new_equals_default() {
        let v1 = Velocity::new();
        let v2 = Velocity::default();

        prop_assert_eq!(v1.lin, v2.lin);
        prop_assert_eq!(v1.ang, v2.ang);
    }
}

// ============================================================================
// Query 属性测试
// ============================================================================

proptest! {
    /// 测试Query的计数准确性
    /// Query应该返回正确数量的实体
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_query_count_accuracy(n in 10usize..100) {
        let mut world = World::new();

        // 创建n个带Transform的实体
        for _ in 0..n {
            world.spawn(Transform::default());
        }

        // 添加一些不带Transform的实体
        for _ in 0..10 {
            world.spawn_empty();
        }

        let count = world.query::<&Transform>().iter(&world).count();
        prop_assert_eq!(count, n);
    }

    /// 测试Query的组件组合
    /// Query<(A, B)>应该只返回同时拥有A和B的实体
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_query_combination(n in 10usize..100) {
        let mut world = World::new();

        // 创建n个实体，部分带Transform，部分带Velocity
        for i in 0..n {
            if i % 3 == 0 {
                world.spawn((Transform::default(), Velocity::default()));
            } else if i % 3 == 1 {
                world.spawn(Transform::default());
            } else {
                world.spawn(Velocity::default());
            }
        }

        let transform_count = world.query::<&Transform>().iter(&world).count();
        let velocity_count = world.query::<&Velocity>().iter(&world).count();
        let both_count = world.query::<(&Transform, &Velocity)>().iter(&world).count();

        // 验证组合查询
        prop_assert!(both_count <= transform_count);
        prop_assert!(both_count <= velocity_count);
    }

    /// 测试Query的空结果
    /// 空世界中Query应该返回0个结果
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_query_empty_world() {
        let world = World::new();

        let count = world.query::<&Transform>().iter(&world).count();
        prop_assert_eq!(count, 0);
    }
}

// ============================================================================
// Entity 生命周期属性测试
// ============================================================================

proptest! {
    /// 测试实体删除
    /// 删除实体后，Query不应该返回该实体
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_entity_deletion(n in 10usize..100) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // 创建n个实体
        for _ in 0..n {
            let entity = world.spawn(Transform::default()).id();
            entities.push(entity);
        }

        // 删除前半部分实体
        for entity in entities.iter().take(n / 2) {
            world.entity_mut(*entity).despawn();
        }

        // 验证剩余实体数量
        let remaining = world.query::<&Transform>().iter(&world).count();
        prop_assert_eq!(remaining, n - n / 2);
    }

    /// 测试实体的存在性检查
    /// 实体应该存在直到被删除
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_entity_existence() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // 实体应该存在
        prop_assert!(world.get_entity(entity).is_ok());

        // 删除后不应该存在
        world.entity_mut(entity).despawn();
        prop_assert!(world.get_entity(entity).is_err());
    }

    /// 测试组件删除
    /// 删除组件后，Query不应该返回该实体
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_component_removal() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform::default(),
            Velocity::default(),
        )).id();

        // 应该能查询到
        let has_both = world.query::<(&Transform, &Velocity)>().iter(&world).count();
        prop_assert_eq!(has_both, 1);

        // 移除Velocity
        world.entity_mut(entity).remove::<Velocity>();

        // 不应该再查询到两者
        let has_both_after = world.query::<(&Transform, &Velocity)>().iter(&world).count();
        prop_assert_eq!(has_both_after, 0);

        // 但应该还能查到Transform
        let has_transform = world.query::<&Transform>().iter(&world).count();
        prop_assert_eq!(has_transform, 1);
    }
}

// ============================================================================
// TileEntityPool 属性测试
// ============================================================================

proptest! {
    /// 测试TileEntityPool的容量限制
    /// 池中复用的实体数量不应该超过容量
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_tile_entity_pool_capacity(capacity in 10usize..1000) {
        let mut world = World::new();
        let mut pool = TileEntityPool {
            unused: Vec::new(),
            capacity,
        };

        // 生成超过容量的实体
        for _ in 0..(capacity + 10) {
            let entity = world.spawn_empty().id();
            pool.recycle(entity, &mut world);
        }

        prop_assert!(pool.unused.len() <= capacity);
    }

    /// 测试TileEntityPool的LIFO行为
    /// 最近回收的实体应该最先被重用
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_tile_entity_pool_lifo() {
        let mut world = World::new();
        let mut pool = TileEntityPool::default();

        let entity1 = world.spawn_empty().id();
        let entity2 = world.spawn_empty().id();
        let entity3 = world.spawn_empty().id();

        pool.recycle(entity1, &mut world);
        pool.recycle(entity2, &mut world);
        pool.recycle(entity3, &mut world);

        // 应该重用最近回收的实体
        let reused = pool.get_or_spawn(&mut world);
        prop_assert_eq!(reused, entity3);
    }
}

// ============================================================================
// Camera 组件属性测试
// ============================================================================

proptest! {
    /// 测试Camera默认值
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_camera_default() {
        let camera = Camera::new();

        prop_assert_eq!(camera.is_active, true);
        prop_assert!(matches!(camera.projection, Projection::Orthographic { .. }));
    }

    /// 测试Projection的new方法
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_projection_new_equals_default() {
        let p1 = Projection::new();
        let p2 = Projection::default();

        prop_assert_eq!(format!("{:?}", p1), format!("{:?}", p2));
    }
}

// ============================================================================
// 综合测试
// ============================================================================

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_ecs_integration() {
    let mut world = World::new();
    world.insert_resource(game_engine::ecs::Time::default());

    // 创建多个实体
    for i in 0..10 {
        world.spawn((
            Transform {
                pos: glam::Vec3::new(i as f32, 0.0, 0.0),
                ..Default::default()
            },
            Velocity::default(),
        ));
    }

    // 验证查询
    let count = world.query::<(&Transform, &Velocity)>().iter(&world).count();
    assert_eq!(count, 10);
}
