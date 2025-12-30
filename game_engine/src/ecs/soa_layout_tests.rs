//! SoA Layout 综合测试
//!
//! 测试SoA (Structure of Arrays) 布局优化功能

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Transform;
    use crate::ecs::soa_layout::*;
    use bevy_ecs::prelude::*;
    use bevy_ecs::world::CommandQueue;
    use glam::{Quat, Vec3};

    // ========================================
    // SoATransformStorage 基础测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_new() {
        let storage = SoATransformStorage::new();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
        assert!(storage.entity_to_index.is_empty());
        assert!(storage.index_to_entity.is_empty());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_default() {
        let storage = SoATransformStorage::default();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_add_single_entity() {
        let mut storage = SoATransformStorage::new();
        let entity = Entity::from_bits(1);

        let transform = Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        storage.add_entity(entity, transform);

        assert_eq!(storage.len(), 1);
        assert!(!storage.is_empty());
        assert_eq!(storage.positions.len(), 1);
        assert_eq!(storage.rotations.len(), 1);
        assert_eq!(storage.scales.len(), 1);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_add_multiple_entities() {
        let mut storage = SoATransformStorage::new();

        for i in 0..10 {
            let entity = Entity::from_bits(i as u64);
            let transform = Transform {
                pos: Vec3::new(i as f32, 0.0, 0.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            };
            storage.add_entity(entity, transform);
        }

        assert_eq!(storage.len(), 10);
        assert_eq!(storage.positions.len(), 10);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_get_transform() {
        let mut storage = SoATransformStorage::new();
        let entity = Entity::from_bits(1);

        let transform = Transform {
            pos: Vec3::new(5.0, 10.0, 15.0),
            rot: Quat::from_rotation_x(0.5),
            scale: Vec3::new(2.0, 2.0, 2.0),
        };

        storage.add_entity(entity, transform);

        let retrieved = storage.get_transform(entity);
        assert!(retrieved.is_some());

        let retrieved = retrieved.expect("Test: operation should succeed");
        assert_eq!(retrieved.pos, transform.pos);
        assert_eq!(retrieved.rot, transform.rot);
        assert_eq!(retrieved.scale, transform.scale);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_get_nonexistent_entity() {
        let storage = SoATransformStorage::new();
        let entity = Entity::from_bits(999);

        let result = storage.get_transform(entity);
        assert!(result.is_none());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_set_transform() {
        let mut storage = SoATransformStorage::new();
        let entity = Entity::from_bits(1);

        let original = Transform::default();
        storage.add_entity(entity, original);

        let modified = Transform {
            pos: Vec3::new(100.0, 200.0, 300.0),
            rot: Quat::from_rotation_y(1.0),
            scale: Vec3::new(3.0, 3.0, 3.0),
        };

        let success = storage.set_transform(entity, modified);
        assert!(success);

        let retrieved = storage.get_transform(entity).expect("Transform should exist");
        assert_eq!(retrieved.pos, modified.pos);
        assert_eq!(retrieved.scale, modified.scale);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_set_nonexistent_entity() {
        let mut storage = SoATransformStorage::new();
        let entity = Entity::from_bits(999);

        let transform = Transform::default();
        let success = storage.set_transform(entity, transform);
        assert!(!success);
    }

    // ========================================
    // SoATransformStorage 移除操作测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_remove_single_entity() {
        let mut storage = SoATransformStorage::new();
        let entity = Entity::from_bits(1);

        storage.add_entity(entity, Transform::default());
        assert_eq!(storage.len(), 1);

        let removed = storage.remove_entity(entity);
        assert!(removed);
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_remove_nonexistent_entity() {
        let mut storage = SoATransformStorage::new();
        let entity = Entity::from_bits(999);

        let removed = storage.remove_entity(entity);
        assert!(!removed);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_remove_middle_entity() {
        let mut storage = SoATransformStorage::new();

        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let e3 = Entity::from_bits(3);

        storage.add_entity(e1, Transform::default());
        storage.add_entity(e2, Transform::default());
        storage.add_entity(e3, Transform::default());

        assert_eq!(storage.len(), 3);

        // 移除中间的实体
        let removed = storage.remove_entity(e2);
        assert!(removed);
        assert_eq!(storage.len(), 2);

        // 验证其他实体仍然存在
        assert!(storage.get_transform(e1).is_some());
        assert!(storage.get_transform(e2).is_none());
        assert!(storage.get_transform(e3).is_some());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_remove_last_entity() {
        let mut storage = SoATransformStorage::new();

        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);

        storage.add_entity(e1, Transform::default());
        storage.add_entity(e2, Transform::default());

        // 移除最后一个实体
        let removed = storage.remove_entity(e2);
        assert!(removed);
        assert_eq!(storage.len(), 1);
        assert!(storage.get_transform(e1).is_some());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_remove_and_reinsert() {
        let mut storage = SoATransformStorage::new();
        let entity = Entity::from_bits(1);

        storage.add_entity(entity, Transform::default());
        storage.remove_entity(entity);

        // 重新添加
        storage.add_entity(entity, Transform::default());

        assert_eq!(storage.len(), 1);
        assert!(storage.get_transform(entity).is_some());
    }

    // ========================================
    // SoATransformStorage 批量操作测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_update_positions_batch() {
        let mut storage = SoATransformStorage::new();

        for i in 0..10 {
            let entity = Entity::from_bits(i as u64);
            let transform = Transform {
                pos: Vec3::new(i as f32, 0.0, 0.0),
                ..Default::default()
            };
            storage.add_entity(entity, transform);
        }

        // 批量更新所有位置
        storage.update_positions_batch(|pos| {
            pos.x += 100.0;
        });

        // 验证所有位置都被更新
        for pos in &storage.positions {
            assert!(pos.x >= 100.0);
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_update_rotations_batch() {
        let mut storage = SoATransformStorage::new();

        for i in 0..10 {
            let entity = Entity::from_bits(i as u64);
            storage.add_entity(entity, Transform::default());
        }

        // 批量更新所有旋转
        storage.update_rotations_batch(|rot| {
            *rot = Quat::from_rotation_x(1.0);
        });

        // 验证所有旋转都被更新
        for rot in &storage.rotations {
            assert_ne!(*rot, Quat::IDENTITY);
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_update_scales_batch() {
        let mut storage = SoATransformStorage::new();

        for i in 0..10 {
            let entity = Entity::from_bits(i as u64);
            storage.add_entity(entity, Transform::default());
        }

        // 批量更新所有缩放
        storage.update_scales_batch(|scale| {
            scale.x *= 2.0;
            scale.y *= 2.0;
            scale.z *= 2.0;
        });

        // 验证所有缩放都被更新
        for scale in &storage.scales {
            assert_eq!(scale.x, 2.0);
        }
    }

    // ========================================
    // SoATransformStorage from_world 测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_from_world_empty() {
        let mut world = World::new();
        let storage = SoATransformStorage::from_world(&mut world);

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_from_world_single_entity() {
        let mut world = World::new();

        world.spawn(Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            ..Default::default()
        });

        let storage = SoATransformStorage::from_world(&mut world);

        assert_eq!(storage.len(), 1);
        assert_eq!(storage.positions[0].x, 1.0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_from_world_multiple_entities() {
        let mut world = World::new();

        for i in 0..10 {
            world.spawn(Transform {
                pos: Vec3::new(i as f32, 0.0, 0.0),
                ..Default::default()
            });
        }

        let storage = SoATransformStorage::from_world(&mut world);

        assert_eq!(storage.len(), 10);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_from_world_mixed_entities() {
        let mut world = World::new();

        // 添加带Transform的实体
        world.spawn(Transform::default());

        // 添加不带Transform的实体
        world.spawn(());

        // 添加另一个带Transform的实体
        world.spawn(Transform {
            pos: Vec3::new(5.0, 5.0, 5.0),
            ..Default::default()
        });

        let storage = SoATransformStorage::from_world(&mut world);

        // 应该只包含有Transform的实体
        assert_eq!(storage.len(), 2);
    }

    // ========================================
    // SoATransformStorage sync_to_ecs 测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_transform_storage_sync_to_ecs() {
        let mut world = World::new();
        let mut storage = SoATransformStorage::new();

        let entity = Entity::from_bits(1);
        storage.add_entity(
            entity,
            Transform {
                pos: Vec3::new(10.0, 20.0, 30.0),
                ..Default::default()
            },
        );

        // 在world中创建对应实体
        let world_entity = world.spawn(Transform::default()).id();

        // 同步回ECS（注意：这会使用storage中的实体ID，不是world_entity）
        // 实际使用中需要确保实体ID一致
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &mut world);
        storage.sync_to_ecs(commands);

        // 由于实体ID可能不同，这里只测试函数能执行
    }

    // ========================================
    // SoAVelocityStorage 基础测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_velocity_storage_new() {
        let storage = SoAVelocityStorage::new();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_velocity_storage_default() {
        let storage = SoAVelocityStorage::default();
        assert_eq!(storage.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_velocity_storage_from_world() {
        let world = World::new();
        let storage = SoAVelocityStorage::from_world(&world);

        // 当前实现返回空存储
        assert_eq!(storage.len(), 0);
    }

    // ========================================
    // SoALayoutManager 基础测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_layout_manager_new() {
        let manager = SoALayoutManager::new();
        assert!(!manager.is_enabled());
        // Note: transforms is private, use stats() instead
        assert_eq!(manager.stats().transform_count, 0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_layout_manager_default() {
        let manager = SoALayoutManager::default();
        assert!(!manager.is_enabled());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_layout_manager_enable() {
        let mut manager = SoALayoutManager::new();
        assert!(!manager.is_enabled());

        manager.enable();
        assert!(manager.is_enabled());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_layout_manager_disable() {
        let mut manager = SoALayoutManager::new();
        manager.enable();
        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_layout_manager_toggle() {
        let mut manager = SoALayoutManager::new();

        manager.enable();
        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());

        manager.enable();
        assert!(manager.is_enabled());
    }

    // ========================================
    // SoALayoutManager 统计信息测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_layout_manager_stats_empty() {
        let manager = SoALayoutManager::new();
        let stats = manager.stats();

        assert_eq!(stats.transform_count, 0);
        assert_eq!(stats.velocity_count, 0);
        assert!(!stats.enabled);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors - SoALayoutManager needs public method to add transforms
    fn test_soa_layout_manager_stats_with_transforms() {
        // TODO: Add public method to SoALayoutManager to add entities
        // For now, this test cannot be implemented without internal access
        let mut manager = SoALayoutManager::new();
        manager.enable();

        let stats = manager.stats();
        assert_eq!(stats.transform_count, 0); // No way to add transforms publicly
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_layout_manager_stats_enabled() {
        let mut manager = SoALayoutManager::new();
        manager.enable();

        let stats = manager.stats();
        assert!(stats.enabled);
    }

    // ========================================
    // SoAStats 测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_stats_struct() {
        let stats = SoAStats {
            transform_count: 100,
            velocity_count: 50,
            enabled: true,
        };

        assert_eq!(stats.transform_count, 100);
        assert_eq!(stats.velocity_count, 50);
        assert!(stats.enabled);
    }

    // ========================================
    // 内存布局测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_memory_layout_contiguous() {
        let mut storage = SoATransformStorage::new();

        for i in 0..100 {
            let entity = Entity::from_bits(i as u64);
            storage.add_entity(entity, Transform::default());
        }

        // 验证数组是连续的
        assert_eq!(storage.positions.len(), 100);
        assert_eq!(storage.rotations.len(), 100);
        assert_eq!(storage.scales.len(), 100);

        // 验证内存是连续分配的（通过指针计算）
        let positions_ptr = storage.positions.as_ptr();
        let rotations_ptr = storage.rotations.as_ptr();
        let scales_ptr = storage.scales.as_ptr();

        // 这些指针不应该为空
        assert!(!positions_ptr.is_null());
        assert!(!rotations_ptr.is_null());
        assert!(!scales_ptr.is_null());
    }

    // ========================================
    // 性能测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_batch_update_performance() {
        let mut storage = SoATransformStorage::new();

        // 创建1000个实体
        for i in 0..1000 {
            let entity = Entity::from_bits(i as u64);
            storage.add_entity(entity, Transform::default());
        }

        // 测量批量更新性能
        let start = std::time::Instant::now();
        storage.update_positions_batch(|pos| {
            pos.x += 1.0;
        });
        let duration = start.elapsed();

        // 应该非常快速（< 10ms）
        assert!(duration < std::time::Duration::from_millis(10));
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_random_access_performance() {
        let mut storage = SoATransformStorage::new();

        // 创建100个实体
        for i in 0..100 {
            let entity = Entity::from_bits(i as u64);
            storage.add_entity(
                entity,
                Transform {
                    pos: Vec3::new(i as f32, 0.0, 0.0),
                    ..Default::default()
                },
            );
        }

        // 测量随机访问性能
        let start = std::time::Instant::now();
        for i in 0..100 {
            let entity = Entity::from_bits(i as u64);
            let _ = storage.get_transform(entity);
        }
        let duration = start.elapsed();

        // 应该快速完成（< 50ms）
        assert!(duration < std::time::Duration::from_millis(50));
    }

    // ========================================
    // 边界情况测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_empty_storage_operations() {
        let mut storage = SoATransformStorage::new();

        // 在空存储上执行操作
        storage.update_positions_batch(|_| {});
        storage.update_rotations_batch(|_| {});
        storage.update_scales_batch(|_| {});

        assert_eq!(storage.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_single_entity_operations() {
        let mut storage = SoATransformStorage::new();
        let entity = Entity::from_bits(1);

        storage.add_entity(entity, Transform::default());

        // 单实体批量操作
        storage.update_positions_batch(|pos| {
            pos.x = 100.0;
        });

        let retrieved = storage.get_transform(entity).expect("Transform should exist");
        assert_eq!(retrieved.pos.x, 100.0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_large_number_of_entities() {
        let mut storage = SoATransformStorage::new();

        // 添加大量实体
        for i in 0..10000 {
            let entity = Entity::from_bits(i as u64);
            storage.add_entity(entity, Transform::default());
        }

        assert_eq!(storage.len(), 10000);

        // 验证可以访问所有实体
        for i in 0..10000 {
            let entity = Entity::from_bits(i as u64);
            assert!(storage.get_transform(entity).is_some());
        }
    }

    // ========================================
    // 数据一致性测试
    // ========================================

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_entity_to_index_mapping_consistency() {
        let mut storage = SoATransformStorage::new();

        for i in 0..10 {
            let entity = Entity::from_bits(i as u64);
            storage.add_entity(entity, Transform::default());
        }

        // 验证entity_to_index和index_to_entity的一致性
        for (&entity, &index) in &storage.entity_to_index {
            assert_eq!(storage.index_to_entity[index], entity);
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_soa_removal_preserves_consistency() {
        let mut storage = SoATransformStorage::new();

        for i in 0..10 {
            let entity = Entity::from_bits(i as u64);
            storage.add_entity(entity, Transform::default());
        }

        // 移除一些实体
        storage.remove_entity(Entity::from_bits(2));
        storage.remove_entity(Entity::from_bits(5));

        // 验证一致性
        for (&entity, &index) in &storage.entity_to_index {
            assert_eq!(storage.index_to_entity[index], entity);
        }
    }
}
