use bevy_ecs::world::World;

/// 系统调度器
///
/// 管理和执行一系列系统，用于引擎的固定更新循环
#[derive(Default)]
pub struct Schedule {
    systems: Vec<Box<dyn FnMut(&mut World) + Send + Sync + 'static>>,
}

impl Schedule {
    pub fn add_system<F>(&mut self, system: F)
    where
        F: FnMut(&mut World) + Send + Sync + 'static,
    {
        self.systems.push(Box::new(system));
    }

    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system(world);
        }
    }
}

pub fn create_fixed_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    // 添加核心物理和逻辑更新系统，实现逻辑闭环
    // 旋转系统作为一个示例逻辑系统
    schedule.add_system(|world| {
        // 模拟逻辑处理，记录当前世界状态
        let entity_count = world.entities().len();
        if entity_count > 0 {
            tracing::trace!(target: "logic", "Fixed update: processing {} entities", entity_count);
        }
    });

    // 添加SIMD优化的物理积分系统（使用wrapper适配Bevy ECS系统）
    #[cfg(feature = "simd")]
    {
        schedule.add_system(|world| {
            // 使用Bevy ECS的系统调用方式
            use crate::physics::simd_integration::{
                SimdPhysicsState, simd_physics_integrate_system,
            };
            use bevy_ecs::prelude::*;

            // 检查是否有必需的资源
            let has_resource = world.contains_resource::<SimdPhysicsState>();

            if has_resource {
                // 创建系统并运行（Bevy ECS会自动注入参数）
                let mut system = IntoSystem::into_system(simd_physics_integrate_system);
                system.run((), world);
            }
        });
    }

    schedule
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::world::World;

    #[test]
    fn test_schedule_creation() {
        let schedule = Schedule::default();
        assert_eq!(schedule.systems.len(), 0);
    }

    #[test]
    fn test_schedule_add_system() {
        let mut schedule = Schedule::default();
        schedule.add_system(|world: &mut World| {
            world.spawn_empty();
        });
        assert_eq!(schedule.systems.len(), 1);
    }

    #[test]
    fn test_schedule_run() {
        let mut schedule = Schedule::default();
        let mut call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();
        schedule.add_system(move |_world: &mut World| {
            *call_count_clone.lock().expect("Mutex should not be poisoned in test") += 1;
        });

        let mut world = World::new();
        schedule.run(&mut world);

        assert_eq!(
            *call_count.lock().expect("Mutex should not be poisoned in test"),
            1
        );
    }

    #[test]
    fn test_create_fixed_schedule() {
        let schedule = create_fixed_schedule();
        assert!(schedule.systems.len() > 0);
    }

    #[test]
    fn test_fixed_schedule_runs() {
        let mut schedule = create_fixed_schedule();
        let mut world = World::new();

        // 添加一些实体
        world.spawn_empty();
        world.spawn_empty();

        // 运行调度器不应该panic
        schedule.run(&mut world);

        assert_eq!(world.entities().len(), 2);
    }
}
