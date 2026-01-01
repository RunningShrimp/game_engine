// 物理系统脚本API示例
//
// 演示如何在脚本中使用完整的物理系统API，包括刚体、碰撞、约束等

use game_engine::{
    ecs::{Entity, World},
    scripting::{
        api::ScriptApi,
        physics_api::PhysicsScriptApi,
        system::{JavaScriptContext, ScriptContext, ScriptSystem, ScriptValue},
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== 物理系统脚本API示例 ===\n");

    // 创建ECS世界
    let mut world = World::new();

    // 创建脚本系统
    let mut script_system = ScriptSystem::new();
    let mut js_context = JavaScriptContext::new();
    script_system.register_context(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        Box::new(js_context.clone()),
    );

    // 创建并注册物理API
    let world_arc = std::sync::Arc::new(std::sync::Mutex::new(world));
    let physics_api = PhysicsScriptApi::new(world_arc.clone());

    // 注册物理API到脚本系统
    let mut api = ScriptApi::new();
    physics_api.register_api(&mut api);

    println!("✓ 物理API已注册到脚本系统");

    // 示例1: 基础刚体操作
    println!("\n=== 示例1: 基础刚体操作 ===");
    basic_rigidbody_example(&script_system)?;

    // 示例2: 力和扭矩
    println!("\n=== 示例2: 应用力与扭矩 ===");
    force_and_torque_example(&script_system)?;

    // 示例3: 碰撞检测
    println!("\n=== 示例3: 碰撞检测系统 ===");
    collision_detection_example(&script_system)?;

    // 示例4: 关节约束
    println!("\n=== 示例4: 关节与约束 ===");
    constraint_example(&script_system)?;

    // 示例5: 物理材质
    println!("\n=== 示例5: 物理材质 ===");
    material_example(&script_system)?;

    println!("\n=== 所有示例完成 ===");
    Ok(())
}

/// 示例1: 基础刚体操作
fn basic_rigidbody_example(
    script_system: &ScriptSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 创建刚体
        function createRigidbody() {
            const result = physics_create_rigidbody(1);
            Engine.log("Created rigidbody: " + result);
        }

        // 设置线性速度
        function setVelocity() {
            const result = physics_set_linear_velocity(1, 5.0, 0.0, 0.0);
            Engine.log("Set linear velocity: " + result);
        }

        // 设置角速度
        function setAngularVelocity() {
            const result = physics_set_angular_velocity(1, 0.0, 1.0, 0.0);
            Engine.log("Set angular velocity: " + result);
        }

        // 获取速度
        function getVelocity() {
            const result = physics_get_linear_velocity(1);
            Engine.log("Get linear velocity: " + result);
        }

        // 设置质量
        function setMass() {
            const result = physics_set_mass(1, 10.0);
            Engine.log("Set mass: " + result);
        }

        // 设置阻尼
        function setDamping() {
            const result = physics_set_damping(1, 0.1, 0.1);
            Engine.log("Set damping: " + result);
        }

        // 执行所有操作
        createRigidbody();
        setVelocity();
        setAngularVelocity();
        getVelocity();
        setMass();
        setDamping();
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

/// 示例2: 应用力与扭矩
fn force_and_torque_example(
    script_system: &ScriptSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 应用力
        function applyForce() {
            const result = physics_apply_force(1, 0.0, 10.0, 0.0);
            Engine.log("Applied upward force: " + result);
        }

        // 应用扭矩
        function applyTorque() {
            const result = physics_apply_torque(1, 0.0, 0.0, 5.0);
            Engine.log("Applied torque: " + result);
        }

        // 模拟物理步进
        function simulateStep() {
            Engine.log("Simulating physics step...");

            // 应用向上的力（跳跃）
            applyForce();

            // 应用旋转扭矩
            applyTorque();

            // 获取新速度
            const velocity = physics_get_linear_velocity(1);
            Engine.log("Velocity after force: " + velocity);
        }

        // 运行多个物理步
        for (let i = 0; i < 3; i++) {
            simulateStep();
        }
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

/// 示例3: 碰撞检测系统
fn collision_detection_example(
    script_system: &ScriptSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 射线检测
        function performRaycast() {
            const origin = [0, 1, 0];
            const direction = [0, -1, 0];
            const maxDistance = 10.0;
            const result = physics_raycast(
                origin[0], origin[1], origin[2],
                direction[0], direction[1], direction[2],
                maxDistance
            );
            Engine.log("Raycast result: " + result);
        }

        // 球形投射检测
        function performSphereCast() {
            const origin = [0, 2, 0];
            const radius = 0.5;
            const direction = [0, -1, 0];
            const result = physics_sphere_cast(
                origin[0], origin[1], origin[2],
                radius,
                direction[0], direction[1], direction[2],
                10.0
            );
            Engine.log("SphereCast result: " + result);
        }

        // 盒子重叠检测
        function performOverlapCheck() {
            const center = [0, 1, 0];
            const halfExtents = [1, 1, 1];
            const rotation = 0.0;
            const result = physics_overlap_box(
                center[0], center[1], center[2],
                halfExtents[0], halfExtents[1], halfExtents[2],
                rotation
            );
            Engine.log("OverlapBox result: " + result);
        }

        // 获取碰撞信息
        function getCollisionInfo() {
            const result = physics_get_collision_info(1);
            Engine.log("Collision info: " + result);
        }

        // 执行所有碰撞检测
        performRaycast();
        performSphereCast();
        performOverlapCheck();
        getCollisionInfo();
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

/// 示例4: 关节与约束
fn constraint_example(
    script_system: &ScriptSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 创建固定关节
        function createFixedJoint() {
            const entityA = 1;
            const entityB = 2;
            const result = physics_create_fixed_joint(entityA, entityB);
            Engine.log("FixedJoint: " + result);
        }

        // 创建弹簧关节
        function createSpringJoint() {
            const entityA = 3;
            const entityB = 4;
            const stiffness = 20.0;
            const damping = 0.5;
            const result = physics_create_spring_joint(entityA, entityB, stiffness, damping);
            Engine.log("SpringJoint: " + result);
        }

        // 创建铰链关节
        function createHingeJoint() {
            const entityA = 5;
            const entityB = 6;
            const axisX = 0.0;
            const axisY = 1.0;
            const axisZ = 0.0;
            const result = physics_create_hinge_joint(entityA, entityB, axisX, axisY, axisZ);
            Engine.log("HingeJoint: " + result);
        }

        // 设置关节断裂力
        function setJointBreakForce() {
            const jointId = 1;
            const force = 5000.0;
            const torque = 5000.0;
            const result = physics_set_joint_break_force(jointId, force, torque);
            Engine.log("Joint break force: " + result);
        }

        // 创建所有类型的关节
        createFixedJoint();
        createSpringJoint();
        createHingeJoint();
        setJointBreakForce();
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

/// 示例5: 物理材质
fn material_example(
    script_system: &ScriptSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 创建物理材质
        function createMaterial() {
            const friction = 0.7;
            const restitution = 0.3;
            const result = physics_create_material(friction, restitution);
            Engine.log("Material created: " + result);
        }

        // 设置摩擦力
        function setFriction() {
            const entityId = 1;
            const friction = 0.8;
            const result = physics_set_friction(entityId, friction);
            Engine.log("Friction set: " + result);
        }

        // 设置弹性系数
        function setRestitution() {
            const entityId = 1;
            const restitution = 0.5;
            const result = physics_set_restitution(entityId, restitution);
            Engine.log("Restitution set: " + result);
        }

        // 创建并使用材质
        createMaterial();
        setFriction();
        setRestitution();
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

/// 使用说明
///
/// ## 物理系统脚本API完整参考
///
/// ### 刚体操作 (Rigidbody Operations)
///
/// **创建刚体**
/// ```javascript
/// physics_create_rigidbody(entity_id)
/// ```
///
/// **施加力**
/// ```javascript
/// physics_apply_force(entity_id, fx, fy, fz)  // 力的三分量
/// ```
///
/// **施加扭矩**
/// ```javascript
/// physics_apply_torque(entity_id, tx, ty, tz)  // 扭矩的三分量
/// ```
///
/// **设置线性速度**
/// ```javascript
/// physics_set_linear_velocity(entity_id, vx, vy, vz)
/// ```
///
/// **设置角速度**
/// ```javascript
/// physics_set_angular_velocity(entity_id, vx, vy, vz)
/// ```
///
/// **获取线性速度**
/// ```javascript
/// const velocity = physics_get_linear_velocity(entity_id)  // 返回 "vx,vy,vz" 字符串
/// ```
///
/// **设置质量**
/// ```javascript
/// physics_set_mass(entity_id, mass)
/// ```
///
/// **设置阻尼**
/// ```javascript
/// physics_set_damping(entity_id, linear_damping, angular_damping)
/// ```
///
/// ### 碰撞检测 (Collision Detection)
///
/// **射线检测**
/// ```javascript
/// physics_raycast(ox, oy, oz, dx, dy, dz, max_distance)
/// ```
/// - 参数: 原点(ox,oy,oz), 方向(dx,dy,dz), 最大距离
/// - 返回: 检测信息字符串
///
/// **球形投射检测**
/// ```javascript
/// physics_sphere_cast(ox, oy, oz, radius, dx, dy, dz, max_distance)
/// ```
/// - 参数: 原点, 半径, 方向, 最大距离
///
/// **盒子重叠检测**
/// ```javascript
/// physics_overlap_box(cx, cy, cz, hx, hy, hz, rotation)
/// ```
/// - 参数: 中心点, 半尺寸, 旋转角度
///
/// **获取碰撞信息**
/// ```javascript
/// physics_get_collision_info(entity_id)
/// ```
///
/// ### 约束操作 (Constraints)
///
/// **创建固定关节**
/// ```javascript
/// physics_create_fixed_joint(entity_a, entity_b)
/// ```
///
/// **创建弹簧关节**
/// ```javascript
/// physics_create_spring_joint(entity_a, entity_b, stiffness, damping)
/// ```
///
/// **创建铰链关节**
/// ```javascript
/// physics_create_hinge_joint(entity_a, entity_b, axis_x, axis_y, axis_z)
/// ```
///
/// **设置关节断裂力**
/// ```javascript
/// physics_set_joint_break_force(joint_id, break_force, break_torque)
/// ```
///
/// ### 物理材质 (Physics Materials)
///
/// **创建材质**
/// ```javascript
/// physics_create_material(friction, restitution)
/// ```
/// - friction: 摩擦系数 (0-1)
/// - restitution: 弹性系数 (0-1)
///
/// **设置摩擦力**
/// ```javascript
/// physics_set_friction(entity_id, friction)
/// ```
///
/// **设置弹性系数**
/// ```javascript
/// physics_set_restitution(entity_id, restitution)
/// ```
///
/// ## Lua示例
/// ```lua
/// -- 创建刚体
/// physics_create_rigidbody(1)
///
/// -- 设置速度
/// physics_set_linear_velocity(1, 5.0, 0.0, 0.0)
///
/// -- 应用力
/// physics_apply_force(1, 0.0, 10.0, 0.0)
/// ```
///
/// ## Python示例
/// ```python
/// # 创建刚体
/// physics_create_rigidbody(1)
///
/// # 设置速度
/// physics_set_linear_velocity(1, 5.0, 0.0, 0.0)
///
/// # 应用力
/// physics_apply_force(1, 0.0, 10.0, 0.0)
/// ```
///
/// ## 性能注意事项
///
/// 1. **批量操作**: 尽量在固定更新(on_fixed_update)中执行物理操作
/// 2. **射线检测**: 射线检测相对昂贵，避免每帧调用
/// 3. **约束数量**: 限制约束数量以保持性能
/// 4. **物理步频**: 使用固定时间步长(50Hz)获得稳定物理模拟
///
/// ## 生命周期集成
///
/// 所有物理API都可以在生命周期钩子中使用:
///
/// ```javascript
/// function on_enable(entity) {
///     physics_create_rigidbody(entity);
///     physics_set_mass(entity, 10.0);
/// }
///
/// function on_update(entity, deltaTime) {
///     // 根据输入施加力
///     if (is_jumping) {
///         physics_apply_force(entity, 0, 10, 0);
///     }
/// }
///
/// function on_collision_enter(entity, other) {
///     Engine.log("Collision with: " + other);
/// }
/// ```
