// Python生命周期钩子示例
//
// 演示如何在游戏引擎中使用Python脚本与生命周期钩子

use game_engine::{
    ecs::{Entity, World},
    scripting::{
        python_lifecycle::PythonLifecycleHooksFactory,
        lifecycle::LifecycleHooksComponent,
        system::PythonContext,
        ScriptingConfig,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Python 生命周期钩子示例 ===\n");

    // 创建ECS世界
    let mut world = World::new();

    // 设置脚本系统
    let config = ScriptingConfig {
        enable_python: true,
        ..Default::default()
    };
    setup_scripting(&mut world, config);

    // 创建Python上下文
    let py_context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>> =
        std::sync::Arc::new(std::sync::Mutex::new(PythonContext::new()));

    // 创建玩家实体
    let player_entity = world.spawn_empty().id();

    println!("✓ 创建玩家实体: {:?}", player_entity);

    // 示例1: 基本的生命周期钩子
    println!("\n=== 示例1: 基本生命周期钩子 ===");
    basic_lifecycle_example(player_entity, py_context.clone())?;

    // 示例2: 面向对象的玩家控制器
    println!("\n=== 示例2: OOP玩家控制器 ===");
    oop_player_controller_example(player_entity, py_context.clone())?;

    // 示例3: 装饰器和上下文管理器
    println!("\n=== 示例3: Python装饰器 ===");
    decorator_example(player_entity, py_context.clone())?;

    // 示例4: 异步和生成器
    println!("\n=== 示例4: 生成器和协程 ===");
    generator_example(player_entity, py_context.clone())?;

    println!("\n=== 示例完成 ===");
    Ok(())
}

/// 示例1: 基本的生命周期钩子
///
/// 演示 on_enable, on_update, on_disable, on_destroy 钩子
fn basic_lifecycle_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
# 全局变量
enabled_time = 0
update_count = 0

def on_enable(entity):
    global enabled_time
    print(f"实体已启用: {entity}")
    enabled_time = __import__('game_engine').get_time()

def on_update(entity, delta_time):
    global update_count
    update_count += 1
    print(f"更新次数: {update_count}, Delta时间: {delta_time}")

def on_disable(entity):
    print(f"实体已禁用: {entity}")

def on_destroy(entity):
    global enabled_time, update_count
    lifetime = __import__('game_engine').get_time() - enabled_time
    print(f"实体销毁 - 生命周期: {lifetime:.2f}秒, 总更新次数: {update_count}")
    "#
    .to_string();

    let hooks = PythonLifecycleHooksFactory::create_hooks(
        "basic_lifecycle".to_string(),
        script_source,
        entity,
        context,
    )?;

    let component = LifecycleHooksComponent::new(hooks);

    // 模拟生命周期事件
    println!("模拟 on_enable...");
    component.hooks.on_enable(entity);

    println!("模拟 on_update (3帧)...");
    for i in 1..=3 {
        println!("  帧 {}", i);
        component.hooks.on_update(entity, 0.016);
    }

    println!("模拟 on_disable...");
    component.hooks.on_disable(entity);

    println!("模拟 on_destroy...");
    component.hooks.on_destroy(entity);

    Ok(())
}

/// 示例2: 面向对象的玩家控制器
///
/// 演示Python的类、继承、属性
fn oop_player_controller_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
import random

class Component:
    def __init__(self):
        self.enabled = False
        self.entity = 0

    def on_enable(self, entity):
        self.entity = entity
        self.enabled = True

    def on_disable(self, entity):
        self.enabled = False

class PlayerController(Component):
    def __init__(self):
        super().__init__()
        self.position = {'x': 0.0, 'y': 0.0}
        self.speed = 5.0
        self.health = 100

    def on_enable(self, entity):
        super().on_enable(entity)
        print(f"玩家控制器初始化 - 位置: ({self.position['x']}, {self.position['y']}), 生命: {self.health}")

    def on_update(self, entity, delta_time):
        # 模拟输入处理
        input = self._simulate_input()

        # 更新位置
        self._move(
            input['horizontal'] * self.speed * delta_time,
            input['vertical'] * self.speed * delta_time
        )

        print(f"玩家位置: ({self.position['x']:.2f}, {self.position['y']:.2f})")

        # 边界检查
        self._clamp_position(-10, 10)

    def _move(self, x, y):
        self.position['x'] += x
        self.position['y'] += y

    def _simulate_input(self):
        return {
            'horizontal': 1 if random.random() > 0.5 else -1,
            'vertical': 1 if random.random() > 0.5 else -1
        }

    def _clamp_position(self, min_val, max_val):
        self.position['x'] = max(min_val, min(max_val, self.position['x']))
        self.position['y'] = max(min_val, min(max_val, self.position['y']))

    def take_damage(self, amount):
        self.health = max(0, self.health - amount)
        print(f"玩家受到伤害: {amount}, 剩余生命: {self.health}")

# 创建实例
player_controller = PlayerController()

def on_enable(entity):
    player_controller.on_enable(entity)

def on_update(entity, delta_time):
    player_controller.on_update(entity, delta_time)
    "#
    .to_string();

    let hooks = PythonLifecycleHooksFactory::create_hooks(
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

/// 示例3: 装饰器和上下文管理器
///
/// 演示Python的装饰器、with语句、上下文管理器
fn decorator_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
import time
from contextlib import contextmanager

# 日志装饰器
def log_calls(func):
    def wrapper(*args, **kwargs):
        print(f"[LOG] {func.__name__} called")
        result = func(*args, **kwargs)
        return result
    return wrapper

# 性能监控装饰器
class PerformanceMonitor:
    def __init__(self, func):
        self.func = func
        self.call_count = 0
        self.total_time = 0.0

    def __call__(self, *args, **kwargs):
        start = time.time()
        result = self.func(*args, **kwargs)
        elapsed = time.time() - start

        self.call_count += 1
        self.total_time += elapsed

        if self.call_count % 10 == 0:
            avg_time = self.total_time / self.call_count
            print(f"[PERF] Average update time: {avg_time * 1000:.3f}ms")

        return result

# 移动组件
class MovementComponent:
    @log_calls
    def on_enable(self, entity):
        print("Movement component enabled")

    @PerformanceMonitor
    def on_update(self, entity, delta_time):
        # 模拟移动逻辑
        random.random()

# 创建实例
movement = MovementComponent()

@contextmanager
def component_context(component):
    """上下文管理器示例"""
    print("Entering component context")
    try:
        yield component
    finally:
        print("Exiting component context")

def on_enable(entity):
    with component_context(movement) as comp:
        comp.on_enable(entity)

def on_update(entity, delta_time):
    with component_context(movement) as comp:
        comp.on_update(entity, delta_time)
    "#
    .to_string();

    let hooks = PythonLifecycleHooksFactory::create_hooks(
        "decorator_example".to_string(),
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

/// 示例4: 生成器和协程
///
/// 演示Python的生成器、yield、协程模式
fn generator_example(
    entity: Entity,
    context: std::sync::Arc<std::sync::Mutex<dyn ScriptContext>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_source = r#"
import random

# 生成器：巡逻路径生成器
def patrol_path_generator(waypoints):
    """生成器：循环返回巡逻点"""
    index = 0
    while True:
        yield waypoints[index % len(waypoints)]
        index += 1

# 生成器：敌人AI行为状态机
class EnemyAI:
    def __init__(self):
        self.state = "idle"
        self.patrol_points = [
            {'x': -5, 'y': 0},
            {'x': 5, 'y': 0},
            {'x': 0, 'y': 5}
        ]
        self.patrol_gen = patrol_path_generator(self.patrol_points)
        self.current_target = None
        self.health = 100
        self.position = {'x': 0, 'y': 0}

    def update_state(self, delta_time):
        """状态机更新"""
        if self.state == "idle":
            if random.random() > 0.95:
                self.state = "patrol"
                print("状态切换: idle -> patrol")

        elif self.state == "patrol":
            if self.current_target is None:
                self.current_target = next(self.patrol_gen)
                print(f"巡逻目标: ({self.current_target['x']}, {self.current_target['y']})")

            # 移动向目标
            self._move_towards(self.current_target, delta_time)

            # 检查是否到达
            if self._distance_to(self.current_target) < 0.1:
                self.current_target = None
                self.state = "idle"
                print("到达巡逻点，切换到idle")

        elif self.state == "chase":
            print("正在追逐目标...")
            self.state = "idle"

        elif self.state == "attack":
            print("正在攻击目标!")
            self.state = "chase"

    def on_collision_enter(self, entity, other):
        """碰撞到玩家"""
        self.state = "attack"
        print(f"检测到玩家 {other}，切换到attack状态")

    def on_collision_exit(self, entity, other):
        """失去碰撞"""
        if self.state == "attack":
            self.state = "chase"
            print(f"失去玩家碰撞 {other}，attack -> chase")

    def _move_towards(self, target, delta_time):
        dx = target['x'] - self.position['x']
        dy = target['y'] - self.position['y']
        distance = (dx**2 + dy**2) ** 0.5

        if distance > 0:
            speed = 2.0
            self.position['x'] += (dx / distance) * speed * delta_time
            self.position['y'] += (dy / distance) * speed * delta_time

    def _distance_to(self, target):
        dx = target['x'] - self.position['x']
        dy = target['y'] - self.position['y']
        return (dx**2 + dy**2) ** 0.5

# 创建AI实例
enemy_ai = EnemyAI()

def on_enable(entity):
    print("敌人AI启用 - 初始状态: " + enemy_ai.state)

def on_update(entity, delta_time):
    enemy_ai.update_state(delta_time)

def on_fixed_update(entity, fixed_delta_time):
    if enemy_ai.state in ["chase", "attack"]:
        print("固定更新 - 执行物理碰撞检测")

def on_collision_enter(entity, other):
    enemy_ai.on_collision_enter(entity, other)

def on_collision_exit(entity, other):
    enemy_ai.on_collision_exit(entity, other)
    "#
    .to_string();

    let hooks = PythonLifecycleHooksFactory::create_hooks(
        "generator_example".to_string(),
        script_source,
        entity,
        context,
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
/// 1. 确保启用 `pyo3` feature:
///    `cargo run --example python_lifecycle_example --features pyo3`
///
/// 2. 可用的生命周期钩子:
///    - on_enable(entity: int) -> None
///    - on_disable(entity: int) -> None
///    - on_destroy(entity: int) -> None
///    - on_update(entity: int, delta_time: float) -> None
///    - on_fixed_update(entity: int, fixed_delta_time: float) -> None
///    - on_late_update(entity: int, delta_time: float) -> None
///    - on_collision_enter(entity: int, other: int) -> None
///    - on_collision_stay(entity: int, other: int) -> None
///    - on_collision_exit(entity: int, other: int) -> None
///    - on_trigger_enter(entity: int, other: int) -> None
///    - on_trigger_stay(entity: int, other: int) -> None
///    - on_trigger_exit(entity: int, other: int) -> None
///
/// 3. Python环境内置API:
///    - game_engine.log(message: str) -> None - 记录日志
///    - game_engine.get_time() -> float - 获取当前时间戳
///    - game_engine.spawn_entity() -> int - 创建实体（占位实现）
///
/// 4. Python特性:
///    - 面向对象（类、继承、多态）
///    - 装饰器和上下文管理器
///    - 生成器和协程模式
///    - 动态类型和鸭子类型
///    - 丰富的标准库
///
/// 5. 与其他语言对比:
///    - JavaScript/TypeScript: 更适合Web/前端背景开发者
///    - Lua: 性能最好，适合高频调用
///    - Python: 生态最丰富，适合AI/数据处理
///    - C#: 静态类型，适合大型项目
///
/// 6. 最佳实践:
///    - 使用类型注解提高代码可读性
///    - 利用生成器实现协程模式
///    - 使用with语句管理资源
///    - 利用装饰器实现AOP（面向切面编程）
///    - 避免全局状态，使用类封装
