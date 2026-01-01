// JavaScript生命周期钩子示例
//
// 演示如何在游戏引擎中使用JavaScript脚本与生命周期钩子

use game_engine::{
    ecs::{Entity, World},
    scripting::{
        javascript_lifecycle::JavaScriptLifecycleHooksFactory,
        lifecycle::LifecycleHooksComponent,
        system::{JavaScriptContext, ScriptContext},
        setup_scripting,
        ScriptingConfig,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== JavaScript 生命周期钩子示例 ===\n");

    // 创建ECS世界
    let mut world = World::new();

    // 设置脚本系统
    let config = ScriptingConfig {
        enable_javascript: true,
        ..Default::default()
    };
    setup_scripting(&mut world, config);

    // 创建JavaScript上下文
    let js_context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>> =
        std::sync::Arc::new(std::sync::Mutex::new(JavaScriptContext::new()));

    // 创建玩家实体
    let player_entity = world.spawn_empty().id();

    println!("✓ 创建玩家实体: {:?}", player_entity);

    // 示例1: 基本的生命周期钩子
    println!("\n=== 示例1: 基本生命周期钩子 ===");
    basic_lifecycle_example(player_entity, js_context.clone())?;

    // 示例2: 移动控制脚本
    println!("\n=== 示例2: 玩家移动控制 ===");
    player_movement_example(player_entity, js_context.clone())?;

    // 示例3: 碰撞处理
    println!("\n=== 示例3: 碰撞检测 ===");
    collision_detection_example(player_entity, js_context.clone())?;

    // 示例4: 计时器和更新逻辑
    println!("\n=== 示例4: 计时器系统 ===");
    timer_system_example(player_entity, js_context.clone())?;

    println!("\n=== 示例完成 ===");
    Ok(())
}

/// 示例1: 基本的生命周期钩子
///
/// 演示 onEnable, onUpdate, onDisable, onDestroy 钩子
fn basic_lifecycle_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
        // 全局变量
        let enabledTime = 0;
        let updateCount = 0;

        function onEnable(entity) {
            Engine.log("实体已启用: " + entity);
            enabledTime = Engine.time();
        }

        function onUpdate(entity, deltaTime) {
            updateCount++;
            Engine.log("更新次数: " + updateCount + ", Delta时间: " + deltaTime);
        }

        function onDisable(entity) {
            Engine.log("实体已禁用: " + entity);
        }

        function onDestroy(entity) {
            let lifetime = Engine.time() - enabledTime;
            Engine.log("实体销毁 - 生命周期: " + lifetime + "秒, 总更新次数: " + updateCount);
        }
    "#
    .to_string();

    // 创建生命周期钩子
    let hooks = JavaScriptLifecycleHooksFactory::create_hooks(
        "basic_lifecycle".to_string(),
        script_source,
        entity,
        context,
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
fn player_movement_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
        // 玩家状态
        const player = {
            x: 0,
            y: 0,
            speed: 5,
            health: 100
        };

        function onEnable(entity) {
            Engine.log("玩家初始化 - 位置: (" + player.x + ", " + player.y + "), 生命: " + player.health);
        }

        function onUpdate(entity, deltaTime) {
            // 模拟输入处理
            const input = {
                horizontal: Math.random() > 0.5 ? 1 : -1,
                vertical: Math.random() > 0.5 ? 1 : -1
            };

            // 更新位置
            player.x += input.horizontal * player.speed * deltaTime;
            player.y += input.vertical * player.speed * deltaTime;

            Engine.log("玩家位置更新: (" + player.x.toFixed(2) + ", " + player.y.toFixed(2) + ")");

            // 边界检查
            if (player.x < -10) player.x = -10;
            if (player.x > 10) player.x = 10;
            if (player.y < -10) player.y = -10;
            if (player.y > 10) player.y = 10;
        }

        // 暴露玩家状态供外部访问
        function getPlayerState() {
            return {
                x: player.x,
                y: player.y,
                health: player.health
            };
        }
    "#
    .to_string();

    let hooks = JavaScriptLifecycleHooksFactory::create_hooks(
        "player_movement".to_string(),
        script_source,
        entity,
        context,
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
fn collision_detection_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
        const collisions = new Set();

        function onCollisionEnter(entity, other) {
            Engine.log("碰撞进入 - 实体: " + entity + ", 与: " + other);
            collisions.add(other);
        }

        function onCollisionStay(entity, other) {
            Engine.log("碰撞持续 - 实体: " + entity + ", 与: " + other);
        }

        function onCollisionExit(entity, other) {
            Engine.log("碰撞退出 - 实体: " + entity + ", 与: " + other);
            collisions.delete(other);
        }

        function getCollisionCount() {
            return collisions.size;
        }
    "#
    .to_string();

    let hooks = JavaScriptLifecycleHooksFactory::create_hooks(
        "collision_detection".to_string(),
        script_source,
        entity,
        context,
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

/// 示例4: 计时器系统
///
/// 演示如何在 onFixedUpdate 和 onLateUpdate 中实现计时器
fn timer_system_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
        let gameTime = 0;
        let fixedUpdateCount = 0;
        let timer = 0;

        function onEnable(entity) {
            Engine.log("计时器系统已启动");
        }

        function onUpdate(entity, deltaTime) {
            gameTime += deltaTime;
            timer += deltaTime;

            // 每1秒触发一次事件
            if (timer >= 1.0) {
                Engine.log("游戏时间: " + gameTime.toFixed(2) + "秒");
                timer = 0;
            }
        }

        function onFixedUpdate(entity, fixedDeltaTime) {
            fixedUpdateCount++;
            if (fixedUpdateCount % 50 === 0) {
                Engine.log("固定更新次数: " + fixedUpdateCount);
            }
        }

        function onLateUpdate(entity, deltaTime) {
            // 在所有更新完成后执行
            Engine.log("延迟更新 - 游戏时间: " + gameTime.toFixed(2));
        }
    "#
    .to_string();

    let hooks = JavaScriptLifecycleHooksFactory::create_hooks(
        "timer_system".to_string(),
        script_source,
        entity,
        context,
    )?;

    let component = LifecycleHooksComponent::new(hooks);

    // 模拟游戏循环
    component.hooks.on_enable(entity);

    println!("模拟10帧更新...");
    for i in 0..10 {
        println!("  帧 {}", i + 1);
        component.hooks.on_update(entity, 0.016);
        component.hooks.on_fixed_update(entity, 0.02);
        component.hooks.on_late_update(entity, 0.016);
    }

    Ok(())
}

/// 使用说明
///
/// 1. 确保启用 `javascript` feature: `cargo run --example javascript_lifecycle_example --features javascript`
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
/// 3. JavaScript环境内置API:
///    - Engine.log(message): 记录日志
///    - Engine.time(): 获取当前时间戳
///    - console.log/warn/error: 控制台输出
///
/// 4. 参数说明:
///    - entity: 当前实体ID (整数)
///    - other: 碰撞/触发器的其他实体ID (整数)
///    - deltaTime: 两帧之间的时间间隔 (秒)
///    - fixedDeltaTime: 固定时间步长 (秒, 默认0.02)
