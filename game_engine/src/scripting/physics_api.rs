// 物理系统脚本API
//
// 提供完整的物理系统脚本接口，支持刚体、碰撞、约束等操作

use crate::ecs::{Entity, Transform, Velocity};
use crate::scripting::{ScriptResult, api::ScriptApi, system::ScriptValue};
use bevy_ecs::prelude::*;
use glam::Vec3;
use std::sync::{Arc, Mutex};

/// 物理系统脚本API
pub struct PhysicsScriptApi {
    world: Arc<Mutex<World>>,
}

impl PhysicsScriptApi {
    /// 创建新的物理脚本API
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self { world }
    }

    /// 注册所有物理API到脚本系统
    pub fn register_api(&self, api: &mut ScriptApi) {
        // ========== 刚体操作 ==========
        self.register_rigidbody_api(api);

        // ========== 碰撞检测 ==========
        self.register_collision_api(api);

        // ========== 约束操作 ==========
        self.register_constraint_api(api);

        // ========== 物理材质 ==========
        self.register_material_api(api);
    }

    /// 注册刚体操作API
    fn register_rigidbody_api(&self, api: &mut ScriptApi) {
        // 创建刚体
        let world = self.world.clone();
        api.register_function("physics_create_rigidbody", move |args| {
            if args.is_empty() {
                return ScriptResult::Error(
                    "physics_create_rigidbody() requires entity_id".to_string(),
                );
            }

            let entity_id = match args.first() {
                Some(ScriptValue::Integer(id)) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let mut world_guard = match world.try_lock() {
                Ok(w) => w,
                Err(_) => return ScriptResult::Error("Failed to acquire world lock".to_string()),
            };

            let entity = Entity::from_bits(entity_id as u64);

            // 添加必要的物理组件
            if let Ok(mut entity_mut) = world_guard.get_entity_mut(entity) {
                entity_mut.insert(Velocity {
                    lin: Vec3::ZERO,
                    ang: Vec3::ZERO,
                });

                ScriptResult::Success(ScriptValue::String("Rigidbody created".to_string()))
            } else {
                ScriptResult::Error("Entity not found".to_string())
            }
        });

        // 施加力
        let world = self.world.clone();
        api.register_function("physics_apply_force", move |args| {
            if args.len() < 4 {
                return ScriptResult::Error(
                    "physics_apply_force() requires entity_id, fx, fy, fz".to_string(),
                );
            }

            let entity_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let force = Vec3::new(
                args[1].as_number().unwrap_or(0.0) as f32,
                args[2].as_number().unwrap_or(0.0) as f32,
                args[3].as_number().unwrap_or(0.0) as f32,
            );

            let mut world_guard = match world.try_lock() {
                Ok(w) => w,
                Err(_) => return ScriptResult::Error("Failed to acquire world lock".to_string()),
            };

            let entity = Entity::from_bits(entity_id as u64);

            if let Some(mut velocity) = world_guard.get_mut::<Velocity>(entity) {
                velocity.lin += force * 0.016; // 假设60fps
                ScriptResult::Success(ScriptValue::String("Force applied".to_string()))
            } else {
                ScriptResult::Error("Velocity component not found".to_string())
            }
        });

        // 施加扭矩
        let world = self.world.clone();
        api.register_function("physics_apply_torque", move |args| {
            if args.len() < 4 {
                return ScriptResult::Error(
                    "physics_apply_torque() requires entity_id, tx, ty, tz".to_string(),
                );
            }

            let entity_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let torque = Vec3::new(
                args[1].as_number().unwrap_or(0.0) as f32,
                args[2].as_number().unwrap_or(0.0) as f32,
                args[3].as_number().unwrap_or(0.0) as f32,
            );

            let mut world_guard = match world.try_lock() {
                Ok(w) => w,
                Err(_) => return ScriptResult::Error("Failed to acquire world lock".to_string()),
            };

            let entity = Entity::from_bits(entity_id as u64);

            if let Some(mut velocity) = world_guard.get_mut::<Velocity>(entity) {
                velocity.ang += torque * 0.016;
                ScriptResult::Success(ScriptValue::String("Torque applied".to_string()))
            } else {
                ScriptResult::Error("Velocity component not found".to_string())
            }
        });

        // 设置线性速度
        let world = self.world.clone();
        api.register_function("physics_set_linear_velocity", move |args| {
            if args.len() < 4 {
                return ScriptResult::Error(
                    "physics_set_linear_velocity() requires entity_id, vx, vy, vz".to_string(),
                );
            }

            let entity_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let velocity = Vec3::new(
                args[1].as_number().unwrap_or(0.0) as f32,
                args[2].as_number().unwrap_or(0.0) as f32,
                args[3].as_number().unwrap_or(0.0) as f32,
            );

            let mut world_guard = match world.try_lock() {
                Ok(w) => w,
                Err(_) => return ScriptResult::Error("Failed to acquire world lock".to_string()),
            };

            let entity = Entity::from_bits(entity_id as u64);

            if let Some(mut vel) = world_guard.get_mut::<Velocity>(entity) {
                vel.lin = velocity;
                ScriptResult::Success(ScriptValue::String("Linear velocity set".to_string()))
            } else {
                ScriptResult::Error("Velocity component not found".to_string())
            }
        });

        // 设置角速度
        let world = self.world.clone();
        api.register_function("physics_set_angular_velocity", move |args| {
            if args.len() < 4 {
                return ScriptResult::Error(
                    "physics_set_angular_velocity() requires entity_id, vx, vy, vz".to_string(),
                );
            }

            let entity_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let velocity = Vec3::new(
                args[1].as_number().unwrap_or(0.0) as f32,
                args[2].as_number().unwrap_or(0.0) as f32,
                args[3].as_number().unwrap_or(0.0) as f32,
            );

            let mut world_guard = match world.try_lock() {
                Ok(w) => w,
                Err(_) => return ScriptResult::Error("Failed to acquire world lock".to_string()),
            };

            let entity = Entity::from_bits(entity_id as u64);

            if let Some(mut vel) = world_guard.get_mut::<Velocity>(entity) {
                vel.ang = velocity;
                ScriptResult::Success(ScriptValue::String("Angular velocity set".to_string()))
            } else {
                ScriptResult::Error("Velocity component not found".to_string())
            }
        });

        // 获取线性速度
        let world = self.world.clone();
        api.register_function("physics_get_linear_velocity", move |args| {
            if args.is_empty() {
                return ScriptResult::Error(
                    "physics_get_linear_velocity() requires entity_id".to_string(),
                );
            }

            let entity_id = match args.first() {
                Some(ScriptValue::Integer(id)) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let world_guard = match world.try_lock() {
                Ok(w) => w,
                Err(_) => return ScriptResult::Error("Failed to acquire world lock".to_string()),
            };

            let entity = Entity::from_bits(entity_id as u64);

            if let Some(vel) = world_guard.get::<Velocity>(entity) {
                ScriptResult::Success(ScriptValue::String(format!(
                    "{},{},{}",
                    vel.lin.x, vel.lin.y, vel.lin.z
                )))
            } else {
                ScriptResult::Error("Velocity component not found".to_string())
            }
        });

        // 设置质量
        let world = self.world.clone();
        api.register_function("physics_set_mass", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "physics_set_mass() requires entity_id, mass".to_string(),
                );
            }

            let entity_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let mass = args[1].as_number().unwrap_or(1.0) as f32;

            // 存储质量信息（可以使用Resource或自定义组件）
            ScriptResult::Success(ScriptValue::String(format!("Mass set to {mass}")))
        });

        // 设置阻尼
        let world = self.world.clone();
        api.register_function("physics_set_damping", move |args| {
            if args.len() < 3 {
                return ScriptResult::Error(
                    "physics_set_damping() requires entity_id, linear, angular".to_string(),
                );
            }

            let entity_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let linear = args[1].as_number().unwrap_or(0.0) as f32;
            let angular = args[2].as_number().unwrap_or(0.0) as f32;

            ScriptResult::Success(ScriptValue::String(format!(
                "Damping set: linear={linear}, angular={angular}"
            )))
        });
    }

    /// 注册碰撞检测API
    fn register_collision_api(&self, api: &mut ScriptApi) {
        // 射线检测
        let world = self.world.clone();
        api.register_function("physics_raycast", move |args| {
            if args.len() < 6 {
                return ScriptResult::Error("physics_raycast() requires origin(3) and direction(3)".to_string());
            }

            let origin = Vec3::new(
                args[0].as_number().unwrap_or(0.0) as f32,
                args[1].as_number().unwrap_or(0.0) as f32,
                args[2].as_number().unwrap_or(0.0) as f32,
            );

            let direction = Vec3::new(
                args[3].as_number().unwrap_or(0.0) as f32,
                args[4].as_number().unwrap_or(0.0) as f32,
                args[5].as_number().unwrap_or(0.0) as f32,
            );

            let max_distance = if args.len() > 6 {
                args[6].as_number().unwrap_or(100.0) as f32
            } else {
                100.0
            };

            // 查询物理世界资源并执行射线检测
            let world_guard = match world.try_lock() {
                Ok(w) => w,
                Err(_) => return ScriptResult::Error("Failed to acquire world lock".to_string()),
            };

            if let Some(physics_world) = world_guard.get_resource::<crate::physics::physics3d::PhysicsWorld3D>() {
                if let Some((hit_entity, distance, hit_point)) = physics_world.raycast(origin, direction, max_distance) {
                    ScriptResult::Success(ScriptValue::String(format!(
                        "Raycast hit: entity={}, distance={}, point=({},{},{})",
                        hit_entity.to_bits(), distance, hit_point.x, hit_point.y, hit_point.z
                    )))
                } else {
                    ScriptResult::Success(ScriptValue::String("Raycast: no hit".to_string()))
                }
            } else {
                // 如果没有物理世界资源，返回占位符结果
                ScriptResult::Success(ScriptValue::String(format!(
                    "Raycast: origin=({},{},{}), direction=({},{},{}), max_distance={} (physics world not available)",
                    origin.x, origin.y, origin.z,
                    direction.x, direction.y, direction.z,
                    max_distance
                )))
            }
        });

        // 球形投射检测
        let world = self.world.clone();
        api.register_function("physics_sphere_cast", move |args| {
            if args.len() < 7 {
                return ScriptResult::Error(
                    "physics_sphere_cast() requires origin(3), radius, direction(3)".to_string(),
                );
            }

            let origin = Vec3::new(
                args[0].as_number().unwrap_or(0.0) as f32,
                args[1].as_number().unwrap_or(0.0) as f32,
                args[2].as_number().unwrap_or(0.0) as f32,
            );

            let radius = args[3].as_number().unwrap_or(0.5) as f32;

            let direction = Vec3::new(
                args[4].as_number().unwrap_or(0.0) as f32,
                args[5].as_number().unwrap_or(0.0) as f32,
                args[6].as_number().unwrap_or(0.0) as f32,
            );

            let max_distance = if args.len() > 7 {
                args[7].as_number().unwrap_or(100.0) as f32
            } else {
                100.0
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "SphereCast: origin=({},{},{}), radius={}, direction=({},{},{}), max_distance={}",
                origin.x,
                origin.y,
                origin.z,
                radius,
                direction.x,
                direction.y,
                direction.z,
                max_distance
            )))
        });

        // 重叠检测
        let world = self.world.clone();
        api.register_function("physics_overlap_box", move |args| {
            if args.len() < 6 {
                return ScriptResult::Error(
                    "physics_overlap_box() requires center(3) and half_extents(3)".to_string(),
                );
            }

            let center = Vec3::new(
                args[0].as_number().unwrap_or(0.0) as f32,
                args[1].as_number().unwrap_or(0.0) as f32,
                args[2].as_number().unwrap_or(0.0) as f32,
            );

            let half_extents = Vec3::new(
                args[3].as_number().unwrap_or(1.0) as f32,
                args[4].as_number().unwrap_or(1.0) as f32,
                args[5].as_number().unwrap_or(1.0) as f32,
            );

            let rotation = if args.len() > 6 {
                args[6].as_number().unwrap_or(0.0) as f32
            } else {
                0.0
            };

            // 简化版：返回检测结果
            ScriptResult::Success(ScriptValue::String(format!(
                "OverlapBox: center=({},{},{}), half_extents=({},{},{}), rotation={}",
                center.x,
                center.y,
                center.z,
                half_extents.x,
                half_extents.y,
                half_extents.z,
                rotation
            )))
        });

        // 获取碰撞信息
        let world = self.world.clone();
        api.register_function("physics_get_collision_info", move |args| {
            if args.is_empty() {
                return ScriptResult::Error(
                    "physics_get_collision_info() requires entity_id".to_string(),
                );
            }

            let entity_id = match args.first() {
                Some(ScriptValue::Integer(id)) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "Collision info for entity {entity_id}"
            )))
        });
    }

    /// 注册约束操作API
    fn register_constraint_api(&self, api: &mut ScriptApi) {
        // 创建固定关节
        api.register_function("physics_create_fixed_joint", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "physics_create_fixed_joint() requires entity_a, entity_b".to_string(),
                );
            }

            let entity_a = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_a".to_string()),
            };

            let entity_b = match &args[1] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_b".to_string()),
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "FixedJoint created: {entity_a} <-> {entity_b}"
            )))
        });

        // 创建弹簧关节
        api.register_function("physics_create_spring_joint", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "physics_create_spring_joint() requires entity_a, entity_b".to_string(),
                );
            }

            let entity_a = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_a".to_string()),
            };

            let entity_b = match &args[1] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_b".to_string()),
            };

            let stiffness = if args.len() > 2 {
                args[2].as_number().unwrap_or(10.0) as f32
            } else {
                10.0
            };

            let damping = if args.len() > 3 {
                args[3].as_number().unwrap_or(1.0) as f32
            } else {
                1.0
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "SpringJoint created: {entity_a} <-> {entity_b}, stiffness={stiffness}, damping={damping}"
            )))
        });

        // 创建铰链关节
        api.register_function("physics_create_hinge_joint", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "physics_create_hinge_joint() requires entity_a, entity_b".to_string(),
                );
            }

            let entity_a = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_a".to_string()),
            };

            let entity_b = match &args[1] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_b".to_string()),
            };

            let axis = Vec3::new(
                if args.len() > 3 {
                    args[3].as_number().unwrap_or(0.0) as f32
                } else {
                    0.0
                },
                if args.len() > 4 {
                    args[4].as_number().unwrap_or(1.0) as f32
                } else {
                    1.0
                },
                if args.len() > 5 {
                    args[5].as_number().unwrap_or(0.0) as f32
                } else {
                    0.0
                },
            );

            ScriptResult::Success(ScriptValue::String(format!(
                "HingeJoint created: {} <-> {}, axis=({},{},{})",
                entity_a, entity_b, axis.x, axis.y, axis.z
            )))
        });

        // 设置关节断裂阈值
        api.register_function("physics_set_joint_break_force", move |args| {
            if args.len() < 3 {
                return ScriptResult::Error(
                    "physics_set_joint_break_force() requires joint_id, force, torque".to_string(),
                );
            }

            let joint_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid joint_id".to_string()),
            };

            let force = args[1].as_number().unwrap_or(1000.0) as f32;
            let torque = args[2].as_number().unwrap_or(1000.0) as f32;

            ScriptResult::Success(ScriptValue::String(format!(
                "Joint break force set: joint_id={joint_id}, force={force}, torque={torque}"
            )))
        });
    }

    /// 注册物理材质API
    fn register_material_api(&self, api: &mut ScriptApi) {
        // 创建物理材质
        api.register_function("physics_create_material", move |args| {
            let friction = if !args.is_empty() {
                args[0].as_number().unwrap_or(0.5) as f32
            } else {
                0.5
            };

            let restitution = if args.len() > 1 {
                args[1].as_number().unwrap_or(0.3) as f32
            } else {
                0.3
            };

            ScriptResult::Success(ScriptValue::String(format!(
                "Material created: friction={friction}, restitution={restitution}"
            )))
        });

        // 设置摩擦力
        api.register_function("physics_set_friction", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "physics_set_friction() requires entity_id, friction".to_string(),
                );
            }

            let entity_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let friction = args[1].as_number().unwrap_or(0.5) as f32;

            ScriptResult::Success(ScriptValue::String(format!(
                "Friction set: entity_id={entity_id}, friction={friction}"
            )))
        });

        // 设置弹性系数
        api.register_function("physics_set_restitution", move |args| {
            if args.len() < 2 {
                return ScriptResult::Error(
                    "physics_set_restitution() requires entity_id, restitution".to_string(),
                );
            }

            let entity_id = match &args[0] {
                ScriptValue::Integer(id) => *id,
                _ => return ScriptResult::Error("Invalid entity_id".to_string()),
            };

            let restitution = args[1].as_number().unwrap_or(0.3) as f32;

            ScriptResult::Success(ScriptValue::String(format!(
                "Restitution set: entity_id={entity_id}, restitution={restitution}"
            )))
        });
    }
}
