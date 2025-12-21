use bevy_ecs::world::World;

// 简化Schedule结构体，避免依赖问题
#[derive(Default)]
pub struct Schedule {
    systems: Vec<Box<dyn FnMut(&mut World) + Send + Sync + 'static>>,
}

impl Schedule {
    pub fn default() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

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

    #[cfg(feature = "physics_2d")]
    {
        // 可以在这里添加 2D 物理系统
    }

    schedule
}
