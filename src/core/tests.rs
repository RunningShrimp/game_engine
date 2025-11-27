//! 核心模块单元测试
//!
//! 覆盖核心系统的关键功能测试

#[cfg(test)]
mod resources_tests {
    use crate::core::resources::{RenderStats, Benchmark, LogEvents};
    
    #[test]
    fn test_render_stats_default() {
        let stats = RenderStats::default();
        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.instances, 0);
        assert_eq!(stats.culled_objects, 0);
        assert_eq!(stats.total_objects, 0);
    }
    
    #[test]
    fn test_culling_ratio() {
        let stats = RenderStats {
            culled_objects: 30,
            total_objects: 100,
            ..Default::default()
        };
        
        // 30% 被剔除
        let culling_ratio = stats.culled_objects as f32 / stats.total_objects as f32;
        assert!((culling_ratio - 0.3).abs() < 0.01);
    }
    
    #[test]
    fn test_benchmark_default() {
        let bench = Benchmark::default();
        assert!(!bench.enabled);
        assert_eq!(bench.sprite_count, 0);
    }
    
    #[test]
    fn test_log_events() {
        let mut log = LogEvents::with_capacity(3);
        
        log.push("msg1".to_string());
        log.push("msg2".to_string());
        log.push("msg3".to_string());
        
        assert_eq!(log.entries.len(), 3);
        
        // 超出容量应该移除最老的
        log.push("msg4".to_string());
        assert_eq!(log.entries.len(), 3);
        assert_eq!(log.entries.front().unwrap(), "msg2");
        assert_eq!(log.entries.back().unwrap(), "msg4");
    }
}

#[cfg(test)]
mod ecs_integration_tests {
    use bevy_ecs::prelude::*;
    use crate::ecs::{Transform, Sprite, Time};
    use glam::{Vec3, Quat};
    
    #[test]
    fn test_time_default() {
        let time = Time::default();
        assert_eq!(time.delta_seconds, 0.0);
        assert_eq!(time.elapsed_seconds, 0.0);
    }
    
    #[test]
    fn test_time_update() {
        let mut time = Time::default();
        time.delta_seconds = 0.016; // ~60fps
        time.elapsed_seconds = 1.0;
        
        assert!((time.delta_seconds - 0.016).abs() < 0.001);
        assert!((time.elapsed_seconds - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_world_creation_with_resources() {
        let mut world = World::default();
        
        // 插入时间资源
        world.insert_resource(Time::default());
        
        // 验证资源存在
        assert!(world.get_resource::<Time>().is_some());
    }
    
    #[test]
    fn test_entity_spawn_with_components() {
        let mut world = World::default();
        
        // 生成实体
        let entity = world.spawn((
            Transform {
                pos: Vec3::new(1.0, 2.0, 3.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Sprite {
                color: [1.0, 0.0, 0.0, 1.0],
                tex_index: 0,
                normal_tex_index: 0,
                uv_off: [0.0, 0.0],
                uv_scale: [1.0, 1.0],
                layer: 0.0,
            },
        )).id();
        
        // 验证组件
        let transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(transform.pos.x, 1.0);
        assert_eq!(transform.pos.y, 2.0);
        assert_eq!(transform.pos.z, 3.0);
        
        let sprite = world.get::<Sprite>(entity).unwrap();
        assert_eq!(sprite.color[0], 1.0); // red
    }
    
    #[test]
    fn test_transform_modification() {
        let mut world = World::default();
        
        let entity = world.spawn(Transform {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        }).id();
        
        // 修改transform
        {
            let mut transform = world.get_mut::<Transform>(entity).unwrap();
            transform.pos = Vec3::new(10.0, 20.0, 30.0);
        }
        
        // 验证修改
        let transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(transform.pos.x, 10.0);
        assert_eq!(transform.pos.y, 20.0);
        assert_eq!(transform.pos.z, 30.0);
    }
}
