// TypeScript生命周期钩子示例
//
// 演示如何在游戏引擎中使用TypeScript脚本与生命周期钩子
// TypeScript使用QuickJS引擎（通过rquickjs），提供类型安全的脚本编写

use game_engine::{
    ecs::{Entity, World},
    scripting::{
        typescript_lifecycle::TypeScriptLifecycleHooksFactory,
        lifecycle::LifecycleHooksComponent,
        system::{JavaScriptContext, ScriptContext},
        ScriptingConfig,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== TypeScript 生命周期钩子示例 ===\n");

    // 创建ECS世界
    let mut world = World::new();

    // 设置脚本系统
    let config = ScriptingConfig {
        enable_typescript: true,
        ..Default::default()
    };
    setup_scripting(&mut world, config);

    // 创建TypeScript上下文
    let ts_context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>> =
        std::sync::Arc::new(std::sync::Mutex::new(JavaScriptContext::new()));

    // 创建玩家实体
    let player_entity = world.spawn_empty().id();

    println!("✓ 创建玩家实体: {:?}", player_entity);

    // 示例1: 带类型注解的基本生命周期钩子
    println!("\n=== 示例1: 类型安全的生命周期钩子 ===");
    typed_lifecycle_example(player_entity, ts_context.clone())?;

    // 示例2: 面向对象的玩家控制器
    println!("\n=== 示例2: OOP玩家控制器 ===");
    oop_player_controller_example(player_entity, ts_context.clone())?;

    // 示例3: 装饰器模式的组件系统
    println!("\n=== 示例3: 装饰器模式 ===");
    decorator_pattern_example(player_entity, ts_context.clone())?;

    // 示例4: 泛型约束系统
    println!("\n=== 示例4: 泛型约束 ===");
    generic_constraints_example(player_entity, ts_context.clone())?;

    println!("\n=== 示例完成 ===");
    Ok(())
}

/// 示例1: 类型安全的生命周期钩子
///
/// 演示TypeScript的类型系统和接口
fn typed_lifecycle_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
        // 定义实体接口
        interface Entity {
            id: number;
            readonly: boolean;
        }

        // 定义时间接口
        interface DeltaTime {
            value: number;
            readonly: boolean;
        }

        // 定义组件状态
        interface ComponentState {
            enabled: boolean;
            updateCount: number;
            enabledTime: number;
        }

        // 组件状态
        const state: ComponentState = {
            enabled: false,
            updateCount: 0,
            enabledTime: 0
        };

        function onEnable(entity: number): void {
            Engine.log("实体已启用: " + entity);
            state.enabled = true;
            state.enabledTime = Engine.time();
        }

        function onUpdate(entity: number, deltaTime: number): void {
            if (!state.enabled) return;

            state.updateCount++;
            Engine.log("更新次数: " + state.updateCount + ", Delta时间: " + deltaTime);
        }

        function onDisable(entity: number): void {
            Engine.log("实体已禁用: " + entity);
            state.enabled = false;
        }

        function onDestroy(entity: number): void {
            const lifetime = Engine.time() - state.enabledTime;
            Engine.log("实体销毁 - 生命周期: " + lifetime + "秒, 总更新次数: " + state.updateCount);
        }
    "#
    .to_string();

    let hooks = TypeScriptLifecycleHooksFactory::create_hooks(
        "typed_lifecycle".to_string(),
        script_source,
        entity,
        context,
    )?;

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

/// 示例2: 面向对象的玩家控制器
///
/// 演示TypeScript的类、继承、访问修饰符
fn oop_player_controller_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
        // 基础组件类
        abstract class Component {
            protected enabled: boolean = false;
            protected entity: number = 0;

            onEnable(entity: number): void {
                this.entity = entity;
                this.enabled = true;
            }

            onDisable(entity: number): void {
                this.enabled = false;
            }
        }

        // 可移动接口
        interface Movable {
            move(x: number, y: number): void;
            getPosition(): { x: number; y: number };
        }

        // 玩家控制器类
        class PlayerController extends Component implements Movable {
            private position: { x: number; y: number };
            private speed: number;
            private health: number;

            constructor() {
                super();
                this.position = { x: 0, y: 0 };
                this.speed = 5.0;
                this.health = 100;
            }

            onEnable(entity: number): void {
                super.onEnable(entity);
                Engine.log("玩家控制器初始化 - 位置: (" +
                          this.position.x + ", " + this.position.y +
                          "), 生命: " + this.health);
            }

            onUpdate(entity: number, deltaTime: number): void {
                // 模拟输入处理
                const input = this.simulateInput();

                // 更新位置
                this.move(
                    input.horizontal * this.speed * deltaTime,
                    input.vertical * this.speed * deltaTime
                );

                Engine.log("玩家位置: (" +
                          this.position.x.toFixed(2) + ", " +
                          this.position.y.toFixed(2) + ")");

                // 边界检查
                this.clampPosition(-10, 10);
            }

            move(x: number, y: number): void {
                this.position.x += x;
                this.position.y += y;
            }

            getPosition(): { x: number; y: number } {
                return { ...this.position };
            }

            takeDamage(amount: number): void {
                this.health = Math.max(0, this.health - amount);
                Engine.log("玩家受到伤害: " + amount + ", 剩余生命: " + this.health);
            }

            private simulateInput(): { horizontal: number; vertical: number } {
                return {
                    horizontal: Math.random() > 0.5 ? 1 : -1,
                    vertical: Math.random() > 0.5 ? 1 : -1
                };
            }

            private clampPosition(min: number, max: number): void {
                this.position.x = Math.max(min, Math.min(max, this.position.x));
                this.position.y = Math.max(min, Math.min(max, this.position.y));
            }
        }

        // 创建实例
        const playerController = new PlayerController();

        // 导出生命周期函数
        function onEnable(entity: number): void {
            playerController.onEnable(entity);
        }

        function onUpdate(entity: number, deltaTime: number): void {
            playerController.onUpdate(entity, deltaTime);
        }
    "#
    .to_string();

    let hooks = TypeScriptLifecycleHooksFactory::create_hooks(
        "oop_player_controller".to_string(),
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

/// 示例3: 装饰器模式
///
/// 演示TypeScript装饰器（如果QuickJS支持）和元编程
fn decorator_pattern_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
        // 简单的装饰器模式实现
        interface Component {
            onEnable?(entity: number): void;
            onUpdate?(entity: number, deltaTime: number): void;
        }

        // 日志装饰器
        function withLogging<T extends Component>(component: T): T {
            const originalOnEnable = component.onEnable;
            const originalOnUpdate = component.onUpdate;

            component.onEnable = function(entity: number) {
                Engine.log("[LOG] onEnable called");
                if (originalOnEnable) originalOnEnable.call(component, entity);
            };

            component.onUpdate = function(entity: number, deltaTime: number) {
                Engine.log("[LOG] onUpdate called");
                if (originalOnUpdate) originalOnUpdate.call(component, entity, deltaTime);
            };

            return component;
        }

        // 性能监控装饰器
        function withPerformanceMonitoring<T extends Component>(component: T): T {
            const originalOnUpdate = component.onUpdate;
            let updateCount = 0;
            let totalTime = 0;

            component.onUpdate = function(entity: number, deltaTime: number) {
                const start = Engine.time();

                if (originalOnUpdate) {
                    originalOnUpdate.call(component, entity, deltaTime);
                }

                const elapsed = Engine.time() - start;
                updateCount++;
                totalTime += elapsed;

                if (updateCount % 10 === 0) {
                    const avgTime = totalTime / updateCount;
                    Engine.log("[PERF] Average update time: " +
                              (avgTime * 1000).toFixed(3) + "ms");
                }
            };

            return component;
        }

        // 基础移动组件
        const movementComponent = {
            onEnable(entity: number) {
                Engine.log("Movement component enabled");
            },
            onUpdate(entity: number, deltaTime: number) {
                // 模拟移动逻辑
                Math.random();
            }
        };

        // 应用装饰器
        const decoratedComponent = withPerformanceMonitoring(
            withLogging(movementComponent)
        );

        // 导出生命周期函数
        function onEnable(entity: number) {
            if (decoratedComponent.onEnable) {
                decoratedComponent.onEnable(entity);
            }
        }

        function onUpdate(entity: number, deltaTime: number) {
            if (decoratedComponent.onUpdate) {
                decoratedComponent.onUpdate(entity, deltaTime);
            }
        }
    "#
    .to_string();

    let hooks = TypeScriptLifecycleHooksFactory::create_hooks(
        "decorator_pattern".to_string(),
        script_source,
        entity,
        context,
    )?;

    let component = LifecycleHooksComponent::new(hooks);

    component.hooks.on_enable(entity);
    println!("模拟15帧以查看性能统计...");
    for i in 0..15 {
        println!("  帧 {}", i + 1);
        component.hooks.on_update(entity, 0.016);
    }

    Ok(())
}

/// 示例4: 泛型约束系统
///
/// 演示TypeScript的泛型和类型约束
fn generic_constraints_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
        // 定义实体接口
        interface Entity {
            id: number;
        }

        // 定义组件接口
        interface Component<T> {
            value: T;
            getValue(): T;
            setValue(value: T): void;
        }

        // 数值组件实现
        class NumericComponent implements Component<number> {
            constructor(public value: number) {}

            getValue(): number {
                return this.value;
            }

            setValue(value: number): void {
                this.value = value;
            }
        }

        // 字符串组件实现
        class StringComponent implements Component<string> {
            constructor(public value: string) {}

            getValue(): string {
                return this.value;
            }

            setValue(value: string): void {
                this.value = value;
            }
        }

        // 组件管理器（泛型）
        class ComponentManager<T extends Entity> {
            private components: Map<string, Component<any>> = new Map();

            registerComponent(name: string, component: Component<any>): void {
                this.components.set(name, component);
                Engine.log("Registered component: " + name);
            }

            getComponent<U>(name: string): Component<U> | undefined {
                return this.components.get(name);
            }
        }

        // 使用示例
        const manager = new ComponentManager<Entity>();

        function onEnable(entity: number): void {
            // 注册组件
            manager.registerComponent("health", new NumericComponent(100));
            manager.registerComponent("name", new StringComponent("Player"));

            Engine.log("Entity " + entity + " initialized with components");
        }

        function onUpdate(entity: number, deltaTime: number): void {
            // 获取和使用组件
            const health = manager.getComponent<number>("health");
            if (health) {
                Engine.log("Health: " + health.getValue());
            }

            const name = manager.getComponent<string>("name");
            if (name) {
                Engine.log("Name: " + name.getValue());
            }
        }
    "#
    .to_string();

    let hooks = TypeScriptLifecycleHooksFactory::create_hooks(
        "generic_constraints".to_string(),
        script_source,
        entity,
        context,
    )?;

    let component = LifecycleHooksComponent::new(hooks);

    component.hooks.on_enable(entity);
    println!("模拟3帧更新...");
    for _ in 0..3 {
        component.hooks.on_update(entity, 0.016);
    }

    Ok(())
}

/// 使用说明
///
/// 1. 确保启用 `typescript` feature:
///    `cargo run --example typescript_lifecycle_example --features typescript`
///
/// 2. TypeScript vs JavaScript:
///    - TypeScript提供编译时类型检查
///    - TypeScript支持接口、泛型、装饰器等高级特性
///    - TypeScript代码会被编译为JavaScript，然后在QuickJS中执行
///    - 性能与JavaScript相同（运行时都是纯JavaScript）
///
/// 3. 可用的生命周期钩子:
///    - onEnable(entity: number): void
///    - onDisable(entity: number): void
///    - onDestroy(entity: number): void
///    - onUpdate(entity: number, deltaTime: number): void
///    - onFixedUpdate(entity: number, fixedDeltaTime: number): void
///    - onLateUpdate(entity: number, deltaTime: number): void
///    - onCollisionEnter(entity: number, other: number): void
///    - onCollisionStay(entity: number, other: number): void
///    - onCollisionExit(entity: number, other: number): void
///    - OnTriggerEnter(entity: number, other: number): void
///    - OnTriggerStay(entity: number, other: number): void
///    - OnTriggerExit(entity: number, other: number): void
///
/// 4. TypeScript环境内置API:
///    - Engine.log(message: string): void - 记录日志
///    - Engine.time(): number - 获取当前时间戳
///    - Engine.spawnEntity(): number - 创建实体（占位实现）
///
/// 5. 类型安全示例:
///    ```typescript
///    interface Vector3 {
///        x: number;
///        y: number;
///        z: number;
///    }
///
///    class Transform implements Component {
///        position: Vector3;
///        rotation: Vector3;
///        scale: Vector3;
///    }
///    ```
///
/// 6. 最佳实践:
///    - 使用接口定义组件契约
///    - 使用类封装相关逻辑
///    - 使用泛型实现可复用组件
///    - 使用访问修饰符控制可见性
///    - 利用类型推断减少冗余代码
