// Lua生命周期钩子示例
//
// 演示如何在游戏引擎中使用Lua脚本与生命周期钩子

use game_engine::{
    ecs::{Entity, World},
    scripting::{
        lua_lifecycle::LuaLifecycleHooksFactory,
        lifecycle::LifecycleHooksComponent,
        lua_support::{LuaEngine, LuaValue},
        ScriptingConfig,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Lua 生命周期钩子示例 ===\n");

    // 创建ECS世界
    let mut world = World::new();

    // 设置脚本系统
    let config = ScriptingConfig {
        enable_lua: true,
        ..Default::default()
    };
    game_engine::scripting::setup_scripting(&mut world, config);

    // 创建Lua引擎
    let mut lua_engine = LuaEngine::new();
    lua_engine.register_engine_api();

    // 创建玩家实体
    let player_entity = world.spawn_empty().id();

    println!("✓ 创建玩家实体: {:?}", player_entity);

    // 示例1: 基本的生命周期钩子
    println!("\n=== 示例1: 基本生命周期钩子 ===");
    basic_lifecycle_example(player_entity)?;

    // 示例2: 玩家移动控制脚本
    println!("\n=== 示例2: 玩家移动控制 ===");
    player_movement_example(player_entity)?;

    // 示例3: 碰撞处理
    println!("\n=== 示例3: 碰撞检测 ===");
    collision_detection_example(player_entity)?;

    // 示例4: 状态机脚本
    println!("\n=== 示例4: 敌人AI状态机 ===");
    enemy_state_machine_example(player_entity)?;

    println!("\n=== 示例完成 ===");
    Ok(())
}

/// 示例1: 基本的生命周期钩子
///
/// 演示 onEnable, onUpdate, onDisable, onDestroy 钩子
fn basic_lifecycle_example(entity: Entity) -> Result<(), Box<dyn std::error::Error>> {
    use mlua::Lua;
    use std::sync::{Arc, Mutex};

    let lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new(Lua::new()));

    let script_source = r#"
        -- 全局变量
        local enabledTime = 0
        local updateCount = 0

        function onEnable(entity)
            Engine.log("实体已启用: " .. entity)
            enabledTime = Engine.time()
        end

        function onUpdate(entity, deltaTime)
            updateCount = updateCount + 1
            Engine.log("更新次数: " .. updateCount .. ", Delta时间: " .. deltaTime)
        end

        function onDisable(entity)
            Engine.log("实体已禁用: " .. entity)
        end

        function onDestroy(entity)
            local lifetime = Engine.time() - enabledTime
            Engine.log("实体销毁 - 生命周期: " .. lifetime .. "秒, 总更新次数: " .. updateCount)
        end
    "#
    .to_string();

    // 创建生命周期钩子
    let hooks = LuaLifecycleHooksFactory::create_hooks(
        "basic_lifecycle".to_string(),
        script_source,
        entity,
        lua,
    )?;

    // 包装为组件
    let component = LifecycleHooksComponent::new(hooks);

    // 模拟生命周期事件
    println!("模拟 onEnable...");
    component.hooks.on_enable(entity);

    println!("模拟 onUpdate (3帧)...");
    for i in 1..=3 {
        println!("  帧 {}", i);
        component.hooks.on_update(entity, 0.016);
    }

    println!("模拟 onDisable...");
    component.hooks.on_disable(entity);

    println!("模拟 onDestroy...");
    component.hooks.on_destroy(entity);

    Ok(())
}

/// 示例2: 玩家移动控制脚本
///
/// 演示如何在 onUpdate 中处理玩家输入和移动
fn player_movement_example(entity: Entity) -> Result<(), Box<dyn std::error::Error>> {
    use mlua::Lua;
    use std::sync::{Arc, Mutex};

    let lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new(Lua::new()));

    let script_source = r#"
        -- 玩家状态
        local player = {
            x = 0,
            y = 0,
            speed = 5,
            health = 100
        }

        function onEnable(entity)
            Engine.log("玩家初始化 - 位置: (" .. player.x .. ", " .. player.y .. "), 生命: " .. player.health)
        end

        function onUpdate(entity, deltaTime)
            -- 模拟输入处理
            math.randomseed(os.time())
            local input = {
                horizontal = math.random() > 0.5 and 1 or -1,
                vertical = math.random() > 0.5 and 1 or -1
            }

            -- 更新位置
            player.x = player.x + input.horizontal * player.speed * deltaTime
            player.y = player.y + input.vertical * player.speed * deltaTime

            Engine.log("玩家位置更新: (" .. string.format("%.2f", player.x) .. ", " .. string.format("%.2f", player.y) .. ")")

            -- 边界检查
            if player.x < -10 then player.x = -10 end
            if player.x > 10 then player.x = 10 end
            if player.y < -10 then player.y = -10 end
            if player.y > 10 then player.y = 10 end
        end

        -- 暴露玩家状态供外部访问
        function getPlayerState()
            return {
                x = player.x,
                y = player.y,
                health = player.health
            }
        end
    "#
    .to_string();

    let hooks = LuaLifecycleHooksFactory::create_hooks(
        "player_movement".to_string(),
        script_source,
        entity,
        lua,
    )?;

    let component = LifecycleHooksComponent::new(hooks);

    // 模拟游戏循环
    component.hooks.on_enable(entity);
    println!("模拟5帧移动...");
    for _ in 0..5 {
        component.hooks.on_update(entity, 0.016);
    }

    Ok(())
}

/// 示例3: 碰撞检测
///
/// 演示 onCollisionEnter, onCollisionStay, onCollisionExit 钩子
fn collision_detection_example(entity: Entity) -> Result<(), Box<dyn std::error::Error>> {
    use mlua::Lua;
    use std::sync::{Arc, Mutex};

    let lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new(Lua::new()));

    let script_source = r#"
        -- 使用表来追踪碰撞
        local collisions = {}

        function onCollisionEnter(entity, other)
            Engine.log("碰撞进入 - 实体: " .. entity .. ", 与: " .. other)
            collisions[other] = true
        end

        function onCollisionStay(entity, other)
            Engine.log("碰撞持续 - 实体: " .. entity .. ", 与: " .. other)
        end

        function onCollisionExit(entity, other)
            Engine.log("碰撞退出 - 实体: " .. entity .. ", 与: " .. other)
            collisions[other] = nil
        end

        function getCollisionCount()
            local count = 0
            for _ in pairs(collisions) do
                count = count + 1
            end
            return count
        end
    "#
    .to_string();

    let hooks = LuaLifecycleHooksFactory::create_hooks(
        "collision_detection".to_string(),
        script_source,
        entity,
        lua,
    )?;

    let component = LifecycleHooksComponent::new(hooks);

    // 模拟碰撞事件
    let other_entity = Entity::from_raw(999);

    println!("模拟碰撞进入...");
    component.hooks.on_collision_enter(entity, other_entity);

    println!("模拟碰撞持续 (2帧)...");
    for _ in 0..2 {
        component.hooks.on_collision_stay(entity, other_entity);
    }

    println!("模拟碰撞退出...");
    component.hooks.on_collision_exit(entity, other_entity);

    Ok(())
}

/// 示例4: 敌人AI状态机
///
/// 演示使用 onFixedUpdate 实现敌人AI
fn enemy_state_machine_example(entity: Entity) -> Result<(), Box<dyn std::error::Error>> {
    use mlua::Lua;
    use std::sync::{Arc, Mutex};

    let lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new(Lua::new()));

    let script_source = r#"
        -- 敌人AI状态机
        local enemy = {
            state = "idle",  -- idle, patrol, chase, attack
            health = 100,
            position = {x = 0, y = 0},
            target = nil,
            patrolPoints = {{x = -5, y = 0}, {x = 5, y = 0}, {x = 0, y = 5}},
            currentPatrolIndex = 1
        }

        function onEnable(entity)
            Engine.log("敌人AI启用 - 初始状态: " .. enemy.state)
        end

        function onUpdate(entity, deltaTime)
            -- 状态机更新
            if enemy.state == "idle" then
                -- 闲置状态：有50%概率切换到巡逻
                if math.random() > 0.95 then
                    enemy.state = "patrol"
                    Engine.log("状态切换: idle -> patrol")
                end

            elseif enemy.state == "patrol" then
                -- 巡逻状态：向巡逻点移动
                local targetPoint = enemy.patrolPoints[enemy.currentPatrolIndex]
                local dx = targetPoint.x - enemy.position.x
                local dy = targetPoint.y - enemy.position.y
                local distance = math.sqrt(dx*dx + dy*dy)

                if distance < 0.1 then
                    -- 到达巡逻点，切换到下一个
                    enemy.currentPatrolIndex = enemy.currentPatrolIndex % #enemy.patrolPoints + 1
                    enemy.state = "idle"
                    Engine.log("到达巡逻点，切换到idle")
                else
                    -- 移动向巡逻点
                    local speed = 2.0
                    enemy.position.x = enemy.position.x + (dx / distance) * speed * deltaTime
                    enemy.position.y = enemy.position.y + (dy / distance) * speed * deltaTime
                end

            elseif enemy.state == "chase" then
                -- 追逐状态：向目标移动
                if enemy.target then
                    Engine.log("正在追逐目标...")
                else
                    enemy.state = "idle"
                    Engine.log("失去目标，切换到idle")
                end

            elseif enemy.state == "attack" then
                -- 攻击状态
                Engine.log("正在攻击目标!")
                -- 攻击完成后返回追逐
                enemy.state = "chase"
            end
        end

        function onFixedUpdate(entity, fixedDeltaTime)
            -- 固定更新：用于物理相关的计算
            if enemy.state == "chase" or enemy.state == "attack" then
                Engine.log("固定更新 - 执行物理碰撞检测")
            end
        end

        function onCollisionEnter(entity, other)
            -- 碰撞到玩家，切换到攻击状态
            enemy.state = "attack"
            enemy.target = other
            Engine.log("检测到玩家，切换到attack状态")
        end

        function onCollisionExit(entity, other)
            -- 失去碰撞，从攻击状态切换到追逐
            if enemy.target == other then
                enemy.target = nil
                if enemy.state == "attack" then
                    enemy.state = "chase"
                    Engine.log("失去玩家碰撞，attack -> chase")
                end
            end
        end

        function getEnemyState()
            return {
                state = enemy.state,
                health = enemy.health,
                x = enemy.position.x,
                y = enemy.position.y
            }
        end
    "#
    .to_string();

    let hooks = LuaLifecycleHooksFactory::create_hooks(
        "enemy_state_machine".to_string(),
        script_source,
        entity,
        lua,
    )?;

    let component = LifecycleHooksComponent::new(hooks);

    // 模拟游戏循环
    component.hooks.on_enable(entity);

    println!("模拟10帧AI更新...");
    for i in 0..10 {
        println!("  帧 {}", i + 1);
        component.hooks.on_update(entity, 0.016);
        component.hooks.on_fixed_update(entity, 0.02);
    }

    // 模拟玩家碰撞
    let player_entity = Entity::from_raw(100);
    println!("\n模拟玩家碰撞...");
    component.hooks.on_collision_enter(entity, player_entity);

    println!("模拟碰撞中的AI (3帧)...");
    for i in 0..3 {
        println!("  帧 {}", i + 1);
        component.hooks.on_update(entity, 0.016);
    }

    println!("模拟玩家离开...");
    component.hooks.on_collision_exit(entity, player_entity);

    Ok(())
}

/// 使用说明
///
/// 1. 确保启用 `lua` feature: `cargo run --example lua_lifecycle_example --features lua`
///
/// 2. 可用的生命周期钩子:
///    - onEnable(entity): 组件启用时调用
///    - onDisable(entity): 组件禁用时调用
///    - onDestroy(entity): 组件销毁时调用
///    - onUpdate(entity, deltaTime): 每帧调用
///    - onFixedUpdate(entity, fixedDeltaTime): 固定时间步调用（50Hz）
///    - onLateUpdate(entity, deltaTime): 所有更新完成后调用
///    - onCollisionEnter(entity, other): 碰撞开始时调用
///    - onCollisionStay(entity, other): 碰撞持续时调用
///    - onCollisionExit(entity, other): 碰撞结束时调用
///    - OnTriggerEnter(entity, other): 触发器进入时调用
///    - OnTriggerStay(entity, other): 触发器持续时调用
///    - OnTriggerExit(entity, other): 触发器退出时调用
///
/// 3. Lua环境内置API:
///    - Engine.log(message): 记录日志
///    - Engine.time(): 获取当前时间戳
///
/// 4. Lua特性:
///    - 完整的Lua 5.4支持（通过mlua）
///    - 表(table)作为数据结构
///    - 闭包和函数作为一等公民
///    - 协程支持(可配合异步系统)
///
/// 5. 参数说明:
///    - entity: 当前实体ID (数字)
///    - other: 碰撞/触发器的其他实体ID (数字)
///    - deltaTime: 两帧之间的时间间隔 (秒)
///    - fixedDeltaTime: 固定时间步长 (秒, 默认0.02)
///
/// 6. 与JavaScript对比:
///    - Lua性能更好（适合高频调用）
///    - Lua语法更简洁
///    - Lua表比JavaScript对象更轻量
///    - JavaScript拥有更丰富的生态系统
