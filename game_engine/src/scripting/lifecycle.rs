//  脚本生命周期钩子系统
//
//  提供Unity风格的生命周期钩子，支持on_update、on_fixed_update等
//  多语言脚本集成，避免频繁跨语言调用

use crate::ecs::{Entity, Time};
use crate::platform::{KeyCode, MouseButton};
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 脚本生命周期阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecyclePhase {
    /// 启用时调用（组件添加后第一次）
    OnEnable,
    /// 禁用时调用
    OnDisable,
    /// 销毁时调用
    OnDestroy,
    /// 脚本初始化时调用（首次加载，在on_enable之前）
    OnStart,
    /// 脚本关闭时调用（清理资源，在on_destroy之后）
    OnShutdown,
    /// 每帧更新
    OnUpdate,
    /// 固定时间步更新（物理等）
    OnFixedUpdate,
    /// 延迟更新（所有OnUpdate完成后）
    OnLateUpdate,
    /// 碰撞进入
    OnCollisionEnter,
    /// 碰撞持续
    OnCollisionStay,
    /// 碰撞退出
    OnCollisionExit,
    /// 触发器进入
    OnTriggerEnter,
    /// 触发器持续
    OnTriggerStay,
    /// 触发器退出
    OnTriggerExit,
    /// 应用暂停
    OnPause,
    /// 应用恢复
    OnResume,
}

/// 生命周期钩子trait
pub trait LifecycleHooks: Send + Sync {
    /// 脚本启动钩子（首次初始化时调用，在on_enable之前）
    fn on_start(&mut self, entity: Entity) {}
    /// 启用钩子
    fn on_enable(&mut self, entity: Entity) {}
    /// 禁用钩子
    fn on_disable(&mut self, entity: Entity) {}
    /// 销毁钩子
    fn on_destroy(&mut self, entity: Entity) {}
    /// 脚本关闭钩子（清理资源，在on_destroy之后）
    fn on_shutdown(&mut self, entity: Entity) {}
    /// 更新钩子
    fn on_update(&mut self, entity: Entity, delta_time: f32) {}
    /// 固定更新钩子
    fn on_fixed_update(&mut self, entity: Entity, fixed_delta_time: f32) {}
    /// 延迟更新钩子
    fn on_late_update(&mut self, entity: Entity, delta_time: f32) {}
    /// 碰撞进入钩子
    fn on_collision_enter(&mut self, entity: Entity, other: Entity) {}
    /// 碰撞持续钩子
    fn on_collision_stay(&mut self, entity: Entity, other: Entity) {}
    /// 碰撞退出钩子
    fn on_collision_exit(&mut self, entity: Entity, other: Entity) {}
    /// 触发器进入钩子
    fn on_trigger_enter(&mut self, entity: Entity, other: Entity) {}
    /// 触发器持续钩子
    fn on_trigger_stay(&mut self, entity: Entity, other: Entity) {}
    /// 触发器退出钩子
    fn on_trigger_exit(&mut self, entity: Entity, other: Entity) {}
    /// 键盘按下钩子
    fn on_key_down(&mut self, entity: Entity, key: crate::platform::KeyCode) {}
    /// 键盘释放钩子
    fn on_key_up(&mut self, entity: Entity, key: crate::platform::KeyCode) {}
    /// 鼠标按下钩子
    fn on_mouse_down(&mut self, entity: Entity, button: crate::platform::MouseButton) {}
    /// 鼠标释放钩子
    fn on_mouse_up(&mut self, entity: Entity, button: crate::platform::MouseButton) {}
    /// 应用暂停钩子
    fn on_pause(&mut self, entity: Entity) {}
    /// 应用恢复钩子
    fn on_resume(&mut self, entity: Entity) {}
}

/// 生命周期钩子组件 - 附加到使用钩子的实体
#[derive(Component)]
pub struct LifecycleHooksComponent {
    /// 钩子实现
    pub hooks: Box<dyn LifecycleHooks>,
    /// 是否已启用
    pub enabled: bool,
    /// 是否已调用on_start
    pub(crate) start_called: bool,
    /// 是否已调用on_enable
    pub(crate) enabled_called: bool,
    /// 是否已调用on_shutdown
    pub(crate) shutdown_called: bool,
}

impl std::fmt::Debug for LifecycleHooksComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LifecycleHooksComponent")
            .field("enabled", &self.enabled)
            .field("start_called", &self.start_called)
            .field("enabled_called", &self.enabled_called)
            .field("shutdown_called", &self.shutdown_called)
            .finish()
    }
}

impl LifecycleHooksComponent {
    /// 创建新的生命周期钩子组件
    pub fn new(hooks: Box<dyn LifecycleHooks>) -> Self {
        Self {
            hooks,
            enabled: true,
            start_called: false,
            enabled_called: false,
            shutdown_called: false,
        }
    }

    /// 启用钩子
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用钩子
    pub fn disable(&mut self) {
        self.enabled = false;
        self.enabled_called = false;
    }
}

/// 生命周期系统调度器
///
/// 批处理钩子调用以减少跨语言调用开销
#[derive(bevy_ecs::prelude::Resource, Clone, Debug)]
pub struct LifecycleScheduler {
    /// 待处理的启动钩子
    pending_starts: Arc<Mutex<Vec<Entity>>>,
    /// 待处理的启用钩子
    pending_enables: Arc<Mutex<Vec<Entity>>>,
    /// 待处理的禁用钩子
    pending_disables: Arc<Mutex<Vec<Entity>>>,
    /// 待处理的销毁钩子
    pending_destroys: Arc<Mutex<Vec<Entity>>>,
    /// 待处理的关闭钩子
    pending_shutdowns: Arc<Mutex<Vec<Entity>>>,
    /// 碰撞事件队列
    collision_events: Arc<Mutex<Vec<CollisionEvent>>>,
    /// 触发器事件队列
    trigger_events: Arc<Mutex<Vec<TriggerEvent>>>,
    /// 输入事件队列
    input_events: Arc<Mutex<Vec<InputEvent>>>,
    /// 应用生命周期事件队列
    app_lifecycle_events: Arc<Mutex<Vec<AppLifecycleEvent>>>,
}

/// 碰撞事件
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity: Entity,
    pub other: Entity,
    pub phase: CollisionPhase,
}

/// 碰撞阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionPhase {
    Enter,
    Stay,
    Exit,
}

/// 触发器事件
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub entity: Entity,
    pub other: Entity,
    pub phase: CollisionPhase,
}

/// 输入事件
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub entity: Entity,
    pub event_type: InputEventType,
}

/// 输入事件类型
#[derive(Debug, Clone)]
pub enum InputEventType {
    KeyDown(crate::platform::KeyCode),
    KeyUp(crate::platform::KeyCode),
    MouseDown(crate::platform::MouseButton),
    MouseUp(crate::platform::MouseButton),
}

/// 应用生命周期事件
#[derive(Debug, Clone)]
pub struct AppLifecycleEvent {
    pub entity: Entity,
    pub event_type: AppLifecycleEventType,
}

/// 应用生命周期事件类型
#[derive(Debug, Clone, Copy)]
pub enum AppLifecycleEventType {
    Pause,
    Resume,
}

impl Default for LifecycleScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleScheduler {
    /// 创建新的生命周期调度器
    pub fn new() -> Self {
        Self {
            pending_starts: Arc::new(Mutex::new(Vec::new())),
            pending_enables: Arc::new(Mutex::new(Vec::new())),
            pending_disables: Arc::new(Mutex::new(Vec::new())),
            pending_destroys: Arc::new(Mutex::new(Vec::new())),
            pending_shutdowns: Arc::new(Mutex::new(Vec::new())),
            collision_events: Arc::new(Mutex::new(Vec::new())),
            trigger_events: Arc::new(Mutex::new(Vec::new())),
            input_events: Arc::new(Mutex::new(Vec::new())),
            app_lifecycle_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 队列启动事件
    pub fn queue_start(&self, entity: Entity) {
        if let Ok(mut starts) = self.pending_starts.lock() {
            starts.push(entity);
        }
    }

    /// 队列启用事件
    pub fn queue_enable(&self, entity: Entity) {
        if let Ok(mut enables) = self.pending_enables.lock() {
            enables.push(entity);
        }
    }

    /// 队列禁用事件
    pub fn queue_disable(&self, entity: Entity) {
        if let Ok(mut disables) = self.pending_disables.lock() {
            disables.push(entity);
        }
    }

    /// 队列销毁事件
    pub fn queue_destroy(&self, entity: Entity) {
        if let Ok(mut destroys) = self.pending_destroys.lock() {
            destroys.push(entity);
        }
    }

    /// 队列碰撞事件
    pub fn queue_collision(&self, event: CollisionEvent) {
        if let Ok(mut events) = self.collision_events.lock() {
            events.push(event);
        }
    }

    /// 队列触发器事件
    pub fn queue_trigger(&self, event: TriggerEvent) {
        if let Ok(mut events) = self.trigger_events.lock() {
            events.push(event);
        }
    }

    /// 队列输入事件
    pub fn queue_input(&self, event: InputEvent) {
        if let Ok(mut events) = self.input_events.lock() {
            events.push(event);
        }
    }

    /// 队列应用生命周期事件
    pub fn queue_app_lifecycle(&self, event: AppLifecycleEvent) {
        if let Ok(mut events) = self.app_lifecycle_events.lock() {
            events.push(event);
        }
    }

    /// 队列关闭事件
    pub fn queue_shutdown(&self, entity: Entity) {
        if let Ok(mut shutdowns) = self.pending_shutdowns.lock() {
            shutdowns.push(entity);
        }
    }

    /// 处理待处理的启动钩子
    pub fn process_starts(&self, world: &mut World) {
        if let Ok(mut starts) = self.pending_starts.lock() {
            let entities = std::mem::take(&mut *starts);
            drop(starts); // 释放锁

            for entity in entities {
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        if !hooks.start_called {
                            hooks.hooks.on_start(entity);
                            hooks.start_called = true;
                        }
                    }
                }
            }
        }
    }

    /// 处理待处理的启用钩子
    pub fn process_enables(&self, world: &mut World) {
        if let Ok(mut enables) = self.pending_enables.lock() {
            let entities = std::mem::take(&mut *enables);
            drop(enables); // 释放锁

            for entity in entities {
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        if hooks.enabled && !hooks.enabled_called {
                            hooks.hooks.on_enable(entity);
                            hooks.enabled_called = true;
                        }
                    }
                }
            }
        }
    }

    /// 处理待处理的禁用钩子
    pub fn process_disables(&self, world: &mut World) {
        if let Ok(mut disables) = self.pending_disables.lock() {
            let entities = std::mem::take(&mut *disables);
            drop(disables); // 释放锁

            for entity in entities {
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        hooks.hooks.on_disable(entity);
                        hooks.disable();
                    }
                }
            }
        }
    }

    /// 处理待处理的销毁钩子
    pub fn process_destroys(&self, world: &mut World) {
        if let Ok(mut destroys) = self.pending_destroys.lock() {
            let entities = std::mem::take(&mut *destroys);
            drop(destroys); // 释放锁

            for entity in entities {
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        hooks.hooks.on_destroy(entity);
                        // 注意：实体销毁由ECS系统处理
                    }
                }
            }
        }
    }

    /// 处理待处理的关闭钩子
    pub fn process_shutdowns(&self, world: &mut World) {
        if let Ok(mut shutdowns) = self.pending_shutdowns.lock() {
            let entities = std::mem::take(&mut *shutdowns);
            drop(shutdowns); // 释放锁

            for entity in entities {
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        if !hooks.shutdown_called {
                            hooks.hooks.on_shutdown(entity);
                            hooks.shutdown_called = true;
                        }
                    }
                }
            }
        }
    }

    /// 处理碰撞事件
    pub fn process_collisions(&self, world: &mut World) {
        if let Ok(mut events_guard) = self.collision_events.lock() {
            let events = std::mem::take(&mut *events_guard);
            drop(events_guard); // 释放锁

            for event in events {
                if let Ok(mut entity_mut) = world.get_entity_mut(event.entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        if !hooks.enabled {
                            continue;
                        }

                        match event.phase {
                            CollisionPhase::Enter => {
                                hooks.hooks.on_collision_enter(event.entity, event.other)
                            }
                            CollisionPhase::Stay => {
                                hooks.hooks.on_collision_stay(event.entity, event.other)
                            }
                            CollisionPhase::Exit => {
                                hooks.hooks.on_collision_exit(event.entity, event.other)
                            }
                        }
                    }
                }
            }
        }
    }

    /// 处理触发器事件
    pub fn process_triggers(&self, world: &mut World) {
        if let Ok(mut events_guard) = self.trigger_events.lock() {
            let events = std::mem::take(&mut *events_guard);
            drop(events_guard); // 释放锁

            for event in events {
                if let Ok(mut entity_mut) = world.get_entity_mut(event.entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        if !hooks.enabled {
                            continue;
                        }

                        match event.phase {
                            CollisionPhase::Enter => {
                                hooks.hooks.on_trigger_enter(event.entity, event.other)
                            }
                            CollisionPhase::Stay => {
                                hooks.hooks.on_trigger_stay(event.entity, event.other)
                            }
                            CollisionPhase::Exit => {
                                hooks.hooks.on_trigger_exit(event.entity, event.other)
                            }
                        }
                    }
                }
            }
        }
    }

    /// 运行所有OnUpdate钩子
    pub fn run_updates(&self, world: &mut World, delta_time: f32) {
        let mut query = world.query::<(Entity, &mut LifecycleHooksComponent)>();
        for (entity, mut hooks) in query.iter_mut(world) {
            if hooks.enabled {
                hooks.hooks.on_update(entity, delta_time);
            }
        }
    }

    /// 运行所有OnFixedUpdate钩子
    pub fn run_fixed_updates(&self, world: &mut World, fixed_delta_time: f32) {
        let mut query = world.query::<(Entity, &mut LifecycleHooksComponent)>();
        for (entity, mut hooks) in query.iter_mut(world) {
            if hooks.enabled {
                hooks.hooks.on_fixed_update(entity, fixed_delta_time);
            }
        }
    }

    /// 运行所有OnLateUpdate钩子
    pub fn run_late_updates(&self, world: &mut World, delta_time: f32) {
        let mut query = world.query::<(Entity, &mut LifecycleHooksComponent)>();
        for (entity, mut hooks) in query.iter_mut(world) {
            if hooks.enabled {
                hooks.hooks.on_late_update(entity, delta_time);
            }
        }
    }

    /// 处理输入事件
    pub fn process_input_events(&self, world: &mut World) {
        if let Ok(mut events_guard) = self.input_events.lock() {
            let events = std::mem::take(&mut *events_guard);
            drop(events_guard); // 释放锁

            for event in events {
                if let Ok(mut entity_mut) = world.get_entity_mut(event.entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        if !hooks.enabled {
                            continue;
                        }

                        match event.event_type {
                            InputEventType::KeyDown(key) => {
                                hooks.hooks.on_key_down(event.entity, key)
                            }
                            InputEventType::KeyUp(key) => hooks.hooks.on_key_up(event.entity, key),
                            InputEventType::MouseDown(button) => {
                                hooks.hooks.on_mouse_down(event.entity, button)
                            }
                            InputEventType::MouseUp(button) => {
                                hooks.hooks.on_mouse_up(event.entity, button)
                            }
                        }
                    }
                }
            }
        }
    }

    /// 处理应用生命周期事件
    pub fn process_app_lifecycle_events(&self, world: &mut World) {
        if let Ok(mut events_guard) = self.app_lifecycle_events.lock() {
            let events = std::mem::take(&mut *events_guard);
            drop(events_guard); // 释放锁

            for event in events {
                if let Ok(mut entity_mut) = world.get_entity_mut(event.entity) {
                    if let Some(mut hooks) = entity_mut.get_mut::<LifecycleHooksComponent>() {
                        if !hooks.enabled {
                            continue;
                        }

                        match event.event_type {
                            AppLifecycleEventType::Pause => hooks.hooks.on_pause(event.entity),
                            AppLifecycleEventType::Resume => hooks.hooks.on_resume(event.entity),
                        }
                    }
                }
            }
        }
    }
}

/// 生命周期系统
///
/// ECS系统，自动调度生命周期钩子
pub fn lifecycle_system(world: &mut World) {
    // 获取时间资源
    let delta_time = world
        .get_resource::<crate::ecs::Time>()
        .map(|t| t.delta_seconds)
        .unwrap_or(0.016);

    // 获取生命周期调度器并克隆以避免借用冲突
    let scheduler = world.get_resource::<LifecycleScheduler>().unwrap();
    let scheduler = scheduler.clone();

    // 处理待处理事件（按顺序）
    scheduler.process_starts(world);
    scheduler.process_enables(world);
    scheduler.process_disables(world);
    scheduler.process_destroys(world);
    scheduler.process_shutdowns(world);
    scheduler.process_collisions(world);
    scheduler.process_triggers(world);
    scheduler.process_input_events(world);
    scheduler.process_app_lifecycle_events(world);

    // 运行更新钩子
    scheduler.run_updates(world, delta_time);
    scheduler.run_late_updates(world, delta_time);
}

/// 固定更新生命周期系统
pub fn fixed_update_lifecycle_system(world: &mut World) {
    let fixed_delta_time = 0.02; // 固定时间步长50Hz

    let scheduler = world.get_resource::<LifecycleScheduler>().unwrap();
    let scheduler = scheduler.clone();

    scheduler.run_fixed_updates(world, fixed_delta_time);
}

/// 自动启动钩子系统
///
/// 当LifecycleHooksComponent被添加到实体时，自动队列on_start事件
pub fn auto_start_lifecycle_system(
    mut commands: Commands,
    scheduler: Res<LifecycleScheduler>,
    query: Query<(Entity, &LifecycleHooksComponent), (Added<LifecycleHooksComponent>,)>,
) {
    for (entity, _hooks) in query.iter() {
        scheduler.queue_start(entity);
        scheduler.queue_enable(entity);
    }
}

/// 安装生命周期系统到World
pub fn setup_lifecycle_systems(world: &mut World) {
    world.insert_resource(LifecycleScheduler::new());
}
