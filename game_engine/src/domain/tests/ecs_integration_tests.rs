//  ECS集成测试模块
// 
//  测试ECS与域层组件的集成，包括：
//  - 物理服务与ECS Transform集成
//  - 场景服务与ECS实体集成
//  - Actor系统与ECS资源集成
//  - 事件总线与ECS资源集成

use crate::domain::{
    actor::{AudioActor, AudioActorMessage, PhysicsActor, PhysicsActorMessage},
    audio::AudioSourceId,
    event_bus::{EventBusResource, EventPriority, TestEvent},
    physics::{Collider, ColliderId, RigidBody, RigidBodyId, RigidBodyType},
    scene::{Scene, SceneId},
    services::{AudioDomainService, PhysicsDomainService, SceneDomainService},
    value_objects::Volume,
};
use crate::ecs::{Sprite, Time, Transform, Velocity};
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

// ============================================================================
// 物理服务与ECS集成测试
// ============================================================================

#[cfg(test)]
mod physics_ecs_integration_tests {
    use super::*;

    #[test]
    fn test_physics_service_ecs_resource() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());
        world.insert_resource(Time::default());

        // 验证物理服务已注册为资源
        assert!(world.get_resource::<PhysicsDomainService>().is_some());
    }

    #[test]
    fn test_physics_service_create_body() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let result = physics.create_body(body);
            assert!(result.is_ok());
        }

        // 验证刚体已添加
        if let Some(physics) = world.get_resource::<PhysicsDomainService>() {
            assert_eq!(physics.get_world().bodies.len(), 1);
        }
    }

    #[test]
    fn test_physics_service_step_simulation() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());
        world.insert_resource(Time::default());

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.create_body(body).expect("Test: operation should succeed");
        }

        // 步进模拟
        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let result = physics.step_simulation(0.016);
            assert!(result.is_ok());
        }

        // 验证刚体位置已更新（受重力影响）
        if let Some(physics) = world.get_resource::<PhysicsDomainService>() {
            let pos = physics.get_body_position(RigidBodyId::new(1));
            assert!(pos.is_ok());
            let position = pos.expect("Test: operation should succeed");
            // Y坐标应该小于初始值10.0（受重力影响）
            assert!(position.y < 10.0);
        }
    }

    #[test]
    fn test_physics_service_apply_force() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 0.0, 0.0),
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.create_body(body).expect("Test: operation should succeed");
        }

        // 应用力
        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let result = physics.apply_force(RigidBodyId::new(1), Vec3::new(0.0, 100.0, 0.0));
            assert!(result.is_ok());
        }

        // 步进模拟
        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.step_simulation(0.016).expect("Test: operation should succeed");
        }

        // 验证刚体位置已更新
        if let Some(physics) = world.get_resource::<PhysicsDomainService>() {
            let pos = physics.get_body_position(RigidBodyId::new(1));
            assert!(pos.is_ok());
            let position = pos.expect("Test: operation should succeed");
            // Y坐标应该大于初始值0.0（受向上力影响）
            assert!(position.y > 0.0);
        }
    }

    #[test]
    fn test_physics_service_set_position() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 0.0, 0.0),
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.create_body(body).expect("Test: operation should succeed");
        }

        // 设置位置
        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let result = physics.set_body_position(RigidBodyId::new(1), Vec3::new(5.0, 10.0, 15.0));
            assert!(result.is_ok());
        }

        // 验证位置已更新
        if let Some(physics) = world.get_resource::<PhysicsDomainService>() {
            let pos = physics.get_body_position(RigidBodyId::new(1));
            assert!(pos.is_ok());
            let position = pos.expect("Test: operation should succeed");
            assert_eq!(position.x, 5.0);
            assert_eq!(position.y, 10.0);
            assert_eq!(position.z, 15.0);
        }
    }

    #[test]
    fn test_physics_service_with_collider() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.create_body(body).expect("Test: operation should succeed");
        }

        // 创建碰撞体
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::new(1.0, 1.0, 1.0));

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let result = physics.create_collider(collider, RigidBodyId::new(1));
            assert!(result.is_ok());
        }

        // 步进模拟
        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.step_simulation(0.016).expect("Test: operation should succeed");
        }

        // 验证刚体仍然存在
        if let Some(physics) = world.get_resource::<PhysicsDomainService>() {
            assert_eq!(physics.get_world().bodies.len(), 1);
        }
    }

    #[test]
    fn test_physics_service_destroy_body() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 0.0, 0.0),
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.create_body(body).expect("Test: operation should succeed");
        }

        // 销毁刚体
        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            let result = physics.destroy_body(RigidBodyId::new(1));
            assert!(result.is_ok());
        }

        // 验证刚体已销毁
        if let Some(physics) = world.get_resource::<PhysicsDomainService>() {
            assert_eq!(physics.get_world().bodies.len(), 0);
        }
    }
}

// ============================================================================
// 场景服务与ECS集成测试
// ============================================================================

#[cfg(test)]
mod scene_ecs_integration_tests {
    use super::*;

    #[test]
    fn test_scene_service_ecs_resource() {
        let mut world = World::new();
        world.insert_resource(SceneDomainService::new());

        // 验证场景服务已注册为资源
        assert!(world.get_resource::<SceneDomainService>().is_some());
    }

    #[test]
    fn test_scene_service_create_scene() {
        let mut world = World::new();
        world.insert_resource(SceneDomainService::new());

        // 创建场景
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            let result = scene_service.create_scene(SceneId::new(1), "test_scene");
            assert!(result.is_ok());
        }

        // 验证场景已创建
        if let Some(scene_service) = world.get_resource::<SceneDomainService>() {
            let scene = scene_service.get_scene(SceneId::new(1));
            assert!(scene.is_some());
            assert_eq!(scene.expect("Test: operation should succeed").id(), SceneId::new(1));
        }
    }

    #[test]
    fn test_scene_service_with_entities() {
        let mut world = World::new();
        world.insert_resource(SceneDomainService::new());

        // 创建场景
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            scene_service.create_scene(SceneId::new(1), "test_scene").expect("Test: operation should succeed");
        }

        // 创建ECS实体
        let entity1 = world.spawn((
            Transform {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Sprite::default(),
        )).id();

        let entity2 = world.spawn((
            Transform {
                pos: Vec3::new(10.0, 0.0, 0.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Sprite::default(),
        )).id();

        // 验证实体已创建
        assert!(world.get_entity(entity1).is_ok());
        assert!(world.get_entity(entity2).is_ok());

        // 验证场景存在
        if let Some(scene_service) = world.get_resource::<SceneDomainService>() {
            let scene = scene_service.get_scene(SceneId::new(1));
            assert!(scene.is_some());
        }
    }

    #[test]
    fn test_scene_service_switch_scene() {
        let mut world = World::new();
        world.insert_resource(SceneDomainService::new());

        // 创建多个场景
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            scene_service.create_scene(SceneId::new(1), "scene1").expect("Test: operation should succeed");
            scene_service.create_scene(SceneId::new(2), "scene2").expect("Test: operation should succeed");
        }

        // 切换到场景1
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            let result = scene_service.switch_to_scene(SceneId::new(1));
            assert!(result.is_ok());
        }

        // 验证活跃场景
        if let Some(scene_service) = world.get_resource::<SceneDomainService>() {
            let active_scene = scene_service.get_active_scene();
            assert!(active_scene.is_some());
            assert_eq!(active_scene.expect("Test: operation should succeed").id(), SceneId::new(1));
        }

        // 切换到场景2
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            scene_service.switch_to_scene(SceneId::new(2)).expect("Test: operation should succeed");
        }

        // 验证活跃场景已更新
        if let Some(scene_service) = world.get_resource::<SceneDomainService>() {
            let active_scene = scene_service.get_active_scene();
            assert!(active_scene.is_some());
            assert_eq!(active_scene.expect("Test: operation should succeed").id(), SceneId::new(2));
        }
    }

    #[test]
    fn test_scene_service_delete_scene() {
        let mut world = World::new();
        world.insert_resource(SceneDomainService::new());

        // 创建场景
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            scene_service.create_scene(SceneId::new(1), "test_scene").expect("Test: operation should succeed");
        }

        // 删除场景
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            let result = scene_service.delete_scene(SceneId::new(1));
            assert!(result.is_ok());
        }

        // 验证场景已删除
        if let Some(scene_service) = world.get_resource::<SceneDomainService>() {
            let scene = scene_service.get_scene(SceneId::new(1));
            assert!(scene.is_none());
        }
    }
}

// ============================================================================
// Actor系统与ECS集成测试
// ============================================================================

#[cfg(test)]
mod actor_ecs_integration_tests {
    use super::*;
    use crate::domain::actor::ActorHandle;

    #[test]
    fn test_audio_actor_ecs_resource() {
        let mut world = World::new();
        
        // 创建Actor系统并注册音频Actor
        let mut actor_system = crate::domain::actor::ActorSystem::new();
        let audio_handle = actor_system.register("audio", AudioActor::new()).expect("Test: operation should succeed");
        
        // 将Actor句柄注册为ECS资源
        world.insert_resource(audio_handle);

        // 验证音频Actor句柄已注册为资源
        assert!(world.get_resource::<ActorHandle<AudioActorMessage>>().is_some());
    }

    #[test]
    fn test_audio_actor_send_message() {
        let mut world = World::new();
        
        // 创建Actor系统并注册音频Actor
        let mut actor_system = crate::domain::actor::ActorSystem::new();
        let audio_handle = actor_system.register("audio", AudioActor::new()).expect("Test: operation should succeed");
        
        world.insert_resource(audio_handle);

        // 发送消息到Actor
        if let Some(handle) = world.get_resource::<ActorHandle<AudioActorMessage>>() {
            let result = handle.send(AudioActorMessage::Play {
                source_id: 1,
                path: "test.wav".to_string(),
                volume: 1.0,
                looped: false,
            });
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_physics_actor_ecs_resource() {
        let mut world = World::new();
        
        // 创建Actor系统并注册物理Actor
        let mut actor_system = crate::domain::actor::ActorSystem::new();
        let physics_handle = actor_system.register("physics", PhysicsActor::new()).expect("Test: operation should succeed");
        
        world.insert_resource(physics_handle);

        // 验证物理Actor句柄已注册为资源
        assert!(world.get_resource::<ActorHandle<PhysicsActorMessage>>().is_some());
    }

    #[test]
    fn test_physics_actor_send_message() {
        let mut world = World::new();
        
        // 创建Actor系统并注册物理Actor
        let mut actor_system = crate::domain::actor::ActorSystem::new();
        let physics_handle = actor_system.register("physics", PhysicsActor::new()).expect("Test: operation should succeed");
        
        world.insert_resource(physics_handle);

        // 发送消息到Actor
        if let Some(handle) = world.get_resource::<ActorHandle<PhysicsActorMessage>>() {
            let result = handle.send(PhysicsActorMessage::Step { delta_time: 0.016 });
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_multiple_actors_ecs_resources() {
        let mut world = World::new();
        
        // 创建Actor系统并注册多个Actor
        let mut actor_system = crate::domain::actor::ActorSystem::new();
        let audio_handle = actor_system.register("audio", AudioActor::new()).expect("Test: operation should succeed");
        let physics_handle = actor_system.register("physics", PhysicsActor::new()).expect("Test: operation should succeed");
        
        world.insert_resource(audio_handle);
        world.insert_resource(physics_handle);

        // 验证所有Actor句柄已注册为资源
        assert!(world.get_resource::<ActorHandle<AudioActorMessage>>().is_some());
        assert!(world.get_resource::<ActorHandle<PhysicsActorMessage>>().is_some());
    }
}

// ============================================================================
// 事件总线与ECS集成测试
// ============================================================================

#[cfg(test)]
mod event_bus_ecs_integration_tests {
    use super::*;

    #[test]
    fn test_event_bus_ecs_resource() {
        let mut world = World::new();
        
        // 创建事件总线资源
        let event_bus = std::sync::Arc::new(crate::domain::event_bus::EnhancedEventBus::new());
        world.insert_resource(EventBusResource::new(event_bus));

        // 验证事件总线已注册为资源
        assert!(world.get_resource::<EventBusResource>().is_some());
    }

    #[test]
    fn test_event_bus_publish() {
        let mut world = World::new();
        
        // 创建事件总线资源
        let event_bus = std::sync::Arc::new(crate::domain::event_bus::EnhancedEventBus::new());
        world.insert_resource(EventBusResource::new(event_bus));

        // 发布事件
        if let Some(event_bus_res) = world.get_resource::<EventBusResource>() {
            let event = TestEvent {
                value: 42,
                name: "Test".to_string(),
            };
            event_bus_res.bus.publish(event, EventPriority::Normal);
        }

        // 验证事件已发布
        if let Some(event_bus_res) = world.get_resource::<EventBusResource>() {
            let stats = event_bus_res.bus.get_stats();
            assert_eq!(stats.total_published, 1);
        }
    }

    #[test]
    fn test_event_bus_with_physics_service() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());
        
        // 创建事件总线资源
        let event_bus = std::sync::Arc::new(crate::domain::event_bus::EnhancedEventBus::new());
        world.insert_resource(EventBusResource::new(event_bus));

        // 验证两个资源都存在
        assert!(world.get_resource::<PhysicsDomainService>().is_some());
        assert!(world.get_resource::<EventBusResource>().is_some());
    }
}

// ============================================================================
// 综合集成测试
// ============================================================================

#[cfg(test)]
mod comprehensive_integration_tests {
    use super::*;

    #[test]
    fn test_full_ecs_domain_integration() {
        let mut world = World::new();
        
        // 注册所有领域服务
        world.insert_resource(PhysicsDomainService::new());
        world.insert_resource(SceneDomainService::new());
        world.insert_resource(AudioDomainService::new());
        world.insert_resource(Time::default());
        
        // 创建事件总线资源
        let event_bus = std::sync::Arc::new(crate::domain::event_bus::EnhancedEventBus::new());
        world.insert_resource(EventBusResource::new(event_bus));

        // 创建Actor系统并注册Actor
        let mut actor_system = crate::domain::actor::ActorSystem::new();
        let audio_handle = actor_system.register("audio", AudioActor::new()).expect("Test: operation should succeed");
        let physics_handle = actor_system.register("physics", PhysicsActor::new()).expect("Test: operation should succeed");
        
        world.insert_resource(audio_handle);
        world.insert_resource(physics_handle);

        // 创建场景
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            scene_service.create_scene(SceneId::new(1), "main_scene").expect("Test: operation should succeed");
        }

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.create_body(body).expect("Test: operation should succeed");
        }

        // 创建ECS实体
        let entity = world.spawn((
            Transform {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Sprite::default(),
            Velocity::default(),
        )).id();

        // 验证所有组件都已正确集成
        assert!(world.get_resource::<PhysicsDomainService>().is_some());
        assert!(world.get_resource::<SceneDomainService>().is_some());
        assert!(world.get_resource::<AudioDomainService>().is_some());
        assert!(world.get_resource::<EventBusResource>().is_some());
        assert!(world.get_resource::<ActorHandle<AudioActorMessage>>().is_some());
        assert!(world.get_resource::<ActorHandle<PhysicsActorMessage>>().is_some());
        assert!(world.get_entity(entity).is_ok());

        // 验证场景存在
        if let Some(scene_service) = world.get_resource::<SceneDomainService>() {
            let scene = scene_service.get_scene(SceneId::new(1));
            assert!(scene.is_some());
        }

        // 验证刚体存在
        if let Some(physics) = world.get_resource::<PhysicsDomainService>() {
            assert_eq!(physics.get_world().bodies.len(), 1);
        }
    }

    #[test]
    fn test_ecs_query_with_domain_services() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());
        world.insert_resource(Time::default());

        // 创建多个ECS实体
        for i in 0..10 {
            world.spawn((
                Transform {
                    pos: Vec3::new(i as f32, 0.0, 0.0),
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                Sprite::default(),
                Velocity::default(),
            ));
        }

        // 查询所有实体
        let query = world.query::<(&Transform, &Sprite, &Velocity)>();
        assert_eq!(query.iter(&world).count(), 10);

        // 查询特定位置的实体
        let mut query_filtered = world.query::<&Transform>();
        let count = query_filtered.iter(&world).filter(|t| t.pos.x >= 5.0).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_physics_transform_sync() {
        let mut world = World::new();
        world.insert_resource(PhysicsDomainService::new());
        world.insert_resource(Time::default());

        // 创建刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            RigidBodyType::Dynamic,
            Vec3::new(0.0, 10.0, 0.0),
        );

        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.create_body(body).expect("Test: operation should succeed");
        }

        // 创建ECS实体
        let entity = world.spawn(Transform::default()).id();

        // 步进物理模拟
        if let Some(mut physics) = world.get_resource_mut::<PhysicsDomainService>() {
            physics.step_simulation(0.016).expect("Test: operation should succeed");
        }

        // 获取物理刚体位置
        let physics_pos = if let Some(physics) = world.get_resource::<PhysicsDomainService>() {
            physics.get_body_position(RigidBodyId::new(1)).ok()
        } else {
            None
        };

        assert!(physics_pos.is_some());
        let pos = physics_pos.expect("Test: operation should succeed");
        // Y坐标应该小于初始值10.0（受重力影响）
        assert!(pos.y < 10.0);
    }

    #[test]
    fn test_multiple_scenes_with_entities() {
        let mut world = World::new();
        world.insert_resource(SceneDomainService::new());

        // 创建多个场景
        if let Some(mut scene_service) = world.get_resource_mut::<SceneDomainService>() {
            scene_service.create_scene(SceneId::new(1), "scene1").expect("Test: operation should succeed");
            scene_service.create_scene(SceneId::new(2), "scene2").expect("Test: operation should succeed");
            scene_service.create_scene(SceneId::new(3), "scene3").expect("Test: operation should succeed");
        }

        // 为每个场景创建实体
        for scene_id in 1..=3 {
            for i in 0..5 {
                world.spawn((
                    Transform {
                        pos: Vec3::new(i as f32, 0.0, scene_id as f32),
                        rot: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    Sprite::default(),
                ));
            }
        }

        // 验证所有实体已创建
        let query = world.query::<&Transform>();
        assert_eq!(query.iter(&world).count(), 15);

        // 验证所有场景存在
        if let Some(scene_service) = world.get_resource::<SceneDomainService>() {
            assert!(scene_service.get_scene(SceneId::new(1)).is_some());
            assert!(scene_service.get_scene(SceneId::new(2)).is_some());
            assert!(scene_service.get_scene(SceneId::new(3)).is_some());
        }
    }
}
