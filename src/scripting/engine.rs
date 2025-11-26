// ============================================================================
// 增强的脚本系统
// 支持 JavaScript 热重载、生命周期管理、协程
// ============================================================================

use bevy_ecs::prelude::*;
use rquickjs::{Context, Runtime, Function, Object, Value, Ctx};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver, unbounded};

use crate::bindings::protocol::{BindingCommand, BindingEvent, ComponentData};

/// 脚本组件
#[derive(Component)]
pub struct Script {
    pub source: String,
    pub enabled: bool,
}

impl Script {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            enabled: true,
        }
    }
}

/// 脚本资源句柄
#[derive(Component)]
pub struct ScriptAsset {
    pub path: String,
    pub hot_reload: bool,
}

/// 脚本运行时状态
#[derive(Component)]
pub struct ScriptState {
    /// 是否已初始化
    pub initialized: bool,
    /// 上次修改时间 (用于热重载)
    pub last_modified: u64,
    /// 协程状态
    pub coroutines: Vec<Coroutine>,
    /// 本地变量存储
    pub locals: HashMap<String, ScriptValue>,
}

impl Default for ScriptState {
    fn default() -> Self {
        Self {
            initialized: false,
            last_modified: 0,
            coroutines: Vec::new(),
            locals: HashMap::new(),
        }
    }
}

/// 脚本值类型
#[derive(Debug, Clone)]
pub enum ScriptValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
}

/// 协程状态
#[derive(Debug, Clone)]
pub struct Coroutine {
    pub id: u64,
    pub state: CoroutineState,
    pub wait_condition: WaitCondition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoroutineState {
    Running,
    Waiting,
    Completed,
}

#[derive(Debug, Clone)]
pub enum WaitCondition {
    None,
    Seconds(f32),
    Frames(u32),
    Event(String),
}

/// 脚本引擎资源
#[derive(Resource)]
pub struct ScriptEngine {
    runtime: Runtime,
    contexts: HashMap<Entity, Context>,
    command_tx: Sender<(Entity, BindingCommand)>,
    command_rx: Receiver<(Entity, BindingCommand)>,
    event_queue: VecDeque<(Entity, BindingEvent)>,
    global_bindings: Arc<Mutex<GlobalBindings>>,
    next_coroutine_id: u64,
}

/// 全局绑定状态
#[derive(Default)]
pub struct GlobalBindings {
    pub input_state: InputState,
    pub time: TimeState,
    pub random_seed: u64,
}

#[derive(Default)]
pub struct InputState {
    pub keys_pressed: std::collections::HashSet<String>,
    pub mouse_position: [f32; 2],
    pub mouse_buttons: [bool; 3],
}

#[derive(Default)]
pub struct TimeState {
    pub delta_time: f32,
    pub total_time: f64,
    pub frame_count: u64,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create JS runtime");
        let (tx, rx) = unbounded();
        
        Self {
            runtime,
            contexts: HashMap::new(),
            command_tx: tx,
            command_rx: rx,
            event_queue: VecDeque::new(),
            global_bindings: Arc::new(Mutex::new(GlobalBindings::default())),
            next_coroutine_id: 0,
        }
    }
    
    /// 为实体创建脚本上下文
    pub fn create_context(&mut self, entity: Entity) -> &Context {
        let ctx = Context::full(&self.runtime).expect("Failed to create context");
        self.setup_context(&ctx, entity);
        self.contexts.entry(entity).or_insert(ctx)
    }
    
    /// 移除实体的脚本上下文
    pub fn remove_context(&mut self, entity: Entity) {
        self.contexts.remove(&entity);
    }
    
    /// 设置上下文的 API 绑定
    fn setup_context(&self, ctx: &Context, entity: Entity) {
        let tx = self.command_tx.clone();
        let bindings = Arc::clone(&self.global_bindings);
        let entity_bits = entity.to_bits();
        
        ctx.with(|ctx| {
            let global = ctx.globals();
            
            // ==================== Engine API ====================
            let engine_obj = Object::new(ctx.clone()).unwrap();
            
            // Engine.log(level, msg)
            engine_obj.set("log", Function::new(ctx.clone(), |level: String, msg: String| {
                match level.as_str() {
                    "error" => tracing::error!(target: "script", "{}", msg),
                    "warn" => tracing::warn!(target: "script", "{}", msg),
                    "debug" => tracing::debug!(target: "script", "{}", msg),
                    _ => tracing::info!(target: "script", "{}", msg),
                }
            })).unwrap();
            
            // Engine.spawn(components_json) -> entity_id
            let tx_clone = tx.clone();
            engine_obj.set("spawn", Function::new(ctx.clone(), move |json: String| -> u64 {
                if let Ok(components) = serde_json::from_str::<Vec<ComponentData>>(&json) {
                    let _ = tx_clone.send((Entity::from_bits(entity_bits), BindingCommand::SpawnEntity { components }));
                }
                0
            })).unwrap();
            
            // Engine.despawn(entity_id)
            let tx_clone = tx.clone();
            engine_obj.set("despawn", Function::new(ctx.clone(), move |entity_id: u64| {
                let _ = tx_clone.send((Entity::from_bits(entity_bits), BindingCommand::DespawnEntity { entity_id }));
            })).unwrap();
            
            // Engine.getEntity() -> 当前实体 ID
            engine_obj.set("getEntity", Function::new(ctx.clone(), move || -> u64 {
                entity_bits
            })).unwrap();
            
            global.set("Engine", engine_obj).unwrap();
            
            // ==================== Transform API ====================
            let transform_obj = Object::new(ctx.clone()).unwrap();
            
            let tx_clone = tx.clone();
            transform_obj.set("setPosition", Function::new(ctx.clone(), move |x: f32, y: f32, z: f32| {
                let _ = tx_clone.send((Entity::from_bits(entity_bits), BindingCommand::SetPosition { 
                    entity_id: entity_bits, x, y, z 
                }));
            })).unwrap();
            
            let tx_clone = tx.clone();
            transform_obj.set("setRotation", Function::new(ctx.clone(), move |x: f32, y: f32, z: f32, w: f32| {
                let _ = tx_clone.send((Entity::from_bits(entity_bits), BindingCommand::SetRotation { 
                    entity_id: entity_bits, x, y, z, w 
                }));
            })).unwrap();
            
            let tx_clone = tx.clone();
            transform_obj.set("setScale", Function::new(ctx.clone(), move |x: f32, y: f32, z: f32| {
                let _ = tx_clone.send((Entity::from_bits(entity_bits), BindingCommand::SetScale { 
                    entity_id: entity_bits, x, y, z 
                }));
            })).unwrap();
            
            global.set("Transform", transform_obj).unwrap();
            
            // ==================== Input API ====================
            let input_obj = Object::new(ctx.clone()).unwrap();
            let bindings_clone = Arc::clone(&bindings);
            
            input_obj.set("isKeyPressed", Function::new(ctx.clone(), move |key: String| -> bool {
                bindings_clone.lock().unwrap().input_state.keys_pressed.contains(&key)
            })).unwrap();
            
            let bindings_clone = Arc::clone(&bindings);
            input_obj.set("isKeyDown", Function::new(ctx.clone(), move |key: String| -> bool {
                bindings_clone.lock().unwrap().input_state.keys_pressed.contains(&key)
            })).unwrap();
            
            let bindings_clone = Arc::clone(&bindings);
            input_obj.set("getMousePosition", Function::new(ctx.clone(), move || -> Vec<f32> {
                let pos = bindings_clone.lock().unwrap().input_state.mouse_position;
                vec![pos[0], pos[1]]
            })).unwrap();
            
            let bindings_clone = Arc::clone(&bindings);
            input_obj.set("isMouseButtonPressed", Function::new(ctx.clone(), move |button: u32| -> bool {
                let state = &bindings_clone.lock().unwrap().input_state;
                state.mouse_buttons.get(button as usize).copied().unwrap_or(false)
            })).unwrap();
            
            global.set("Input", input_obj).unwrap();
            
            // ==================== Time API ====================
            let time_obj = Object::new(ctx.clone()).unwrap();
            let bindings_clone = Arc::clone(&bindings);
            
            time_obj.set("deltaTime", Function::new(ctx.clone(), move || -> f32 {
                bindings_clone.lock().unwrap().time.delta_time
            })).unwrap();
            
            let bindings_clone = Arc::clone(&bindings);
            time_obj.set("totalTime", Function::new(ctx.clone(), move || -> f64 {
                bindings_clone.lock().unwrap().time.total_time
            })).unwrap();
            
            let bindings_clone = Arc::clone(&bindings);
            time_obj.set("frameCount", Function::new(ctx.clone(), move || -> u64 {
                bindings_clone.lock().unwrap().time.frame_count
            })).unwrap();
            
            global.set("Time", time_obj).unwrap();
            
            // ==================== Math API ====================
            let math_obj = Object::new(ctx.clone()).unwrap();
            
            math_obj.set("lerp", Function::new(ctx.clone(), |a: f32, b: f32, t: f32| -> f32 {
                a + (b - a) * t.clamp(0.0, 1.0)
            })).unwrap();
            
            math_obj.set("clamp", Function::new(ctx.clone(), |value: f32, min: f32, max: f32| -> f32 {
                value.clamp(min, max)
            })).unwrap();
            
            math_obj.set("distance", Function::new(ctx.clone(), |x1: f32, y1: f32, x2: f32, y2: f32| -> f32 {
                ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
            })).unwrap();
            
            math_obj.set("normalize", Function::new(ctx.clone(), |x: f32, y: f32| -> Vec<f32> {
                let len = (x * x + y * y).sqrt();
                if len > 0.0001 {
                    vec![x / len, y / len]
                } else {
                    vec![0.0, 0.0]
                }
            })).unwrap();
            
            math_obj.set("random", Function::new(ctx.clone(), || -> f64 {
                rand::random::<f64>()
            })).unwrap();
            
            math_obj.set("randomRange", Function::new(ctx.clone(), |min: f32, max: f32| -> f32 {
                min + rand::random::<f32>() * (max - min)
            })).unwrap();
            
            global.set("GameMath", math_obj).unwrap();
            
            // ==================== Audio API ====================
            let audio_obj = Object::new(ctx.clone()).unwrap();
            let tx_clone = tx.clone();
            
            audio_obj.set("play", Function::new(ctx.clone(), move |name: String, path: String, volume: f32, looped: bool| {
                let _ = tx_clone.send((Entity::from_bits(entity_bits), BindingCommand::PlaySound { 
                    name, path, volume, looped 
                }));
            })).unwrap();
            
            let tx_clone = tx.clone();
            audio_obj.set("stop", Function::new(ctx.clone(), move |name: String| {
                let _ = tx_clone.send((Entity::from_bits(entity_bits), BindingCommand::StopSound { name }));
            })).unwrap();
            
            let tx_clone = tx.clone();
            audio_obj.set("setVolume", Function::new(ctx.clone(), move |name: String, volume: f32| {
                let _ = tx_clone.send((Entity::from_bits(entity_bits), BindingCommand::SetVolume { name, volume }));
            })).unwrap();
            
            global.set("Audio", audio_obj).unwrap();
            
            // ==================== Console API ====================
            let console_obj = Object::new(ctx.clone()).unwrap();
            
            console_obj.set("log", Function::new(ctx.clone(), |args: rquickjs::Rest<String>| {
                let msg = args.0.join(" ");
                tracing::info!(target: "script.console", "{}", msg);
            })).unwrap();
            
            console_obj.set("warn", Function::new(ctx.clone(), |args: rquickjs::Rest<String>| {
                let msg = args.0.join(" ");
                tracing::warn!(target: "script.console", "{}", msg);
            })).unwrap();
            
            console_obj.set("error", Function::new(ctx.clone(), |args: rquickjs::Rest<String>| {
                let msg = args.0.join(" ");
                tracing::error!(target: "script.console", "{}", msg);
            })).unwrap();
            
            global.set("console", console_obj).unwrap();
            
            // ==================== 全局便捷函数 ====================
            global.set("print", Function::new(ctx.clone(), |msg: String| {
                println!("[JS] {}", msg);
            })).unwrap();
        });
    }
    
    /// 执行脚本代码
    pub fn execute(&self, entity: Entity, code: &str) -> Result<(), String> {
        if let Some(ctx) = self.contexts.get(&entity) {
            ctx.with(|ctx| {
                ctx.eval::<(), _>(code).map_err(|e| format!("{:?}", e))
            })
        } else {
            Err("No context for entity".to_string())
        }
    }
    
    /// 调用脚本函数
    pub fn call_function(&self, entity: Entity, name: &str, args: &[ScriptValue]) -> Result<ScriptValue, String> {
        if let Some(ctx) = self.contexts.get(&entity) {
            ctx.with(|ctx| {
                let global = ctx.globals();
                if let Ok(func) = global.get::<_, Function>(name) {
                    // 简化处理 - 实际实现需要转换 ScriptValue
                    match func.call::<_, Value>(()) {
                        Ok(_) => Ok(ScriptValue::Null),
                        Err(e) => Err(format!("{:?}", e)),
                    }
                } else {
                    Err(format!("Function '{}' not found", name))
                }
            })
        } else {
            Err("No context for entity".to_string())
        }
    }
    
    /// 触发事件
    pub fn dispatch_event(&mut self, entity: Entity, event: BindingEvent) {
        self.event_queue.push_back((entity, event));
    }
    
    /// 处理事件队列
    pub fn process_events(&mut self) {
        while let Some((entity, event)) = self.event_queue.pop_front() {
            if let Some(ctx) = self.contexts.get(&entity) {
                let event_json = serde_json::to_string(&event).unwrap_or_default();
                ctx.with(|ctx| {
                    let code = format!(
                        "if (typeof __onEngineEvent === 'function') __onEngineEvent({});",
                        event_json
                    );
                    let _ = ctx.eval::<(), _>(code);
                });
            }
        }
    }
    
    /// 获取待处理的命令
    pub fn poll_commands(&self) -> Vec<(Entity, BindingCommand)> {
        self.command_rx.try_iter().collect()
    }
    
    /// 更新全局状态
    pub fn update_globals(&self, delta_time: f32, total_time: f64, frame_count: u64) {
        let mut bindings = self.global_bindings.lock().unwrap();
        bindings.time.delta_time = delta_time;
        bindings.time.total_time = total_time;
        bindings.time.frame_count = frame_count;
    }
    
    /// 更新输入状态
    pub fn update_input(&self, keys: &std::collections::HashSet<String>, mouse_pos: [f32; 2], mouse_buttons: [bool; 3]) {
        let mut bindings = self.global_bindings.lock().unwrap();
        bindings.input_state.keys_pressed = keys.clone();
        bindings.input_state.mouse_position = mouse_pos;
        bindings.input_state.mouse_buttons = mouse_buttons;
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ECS Systems
// ============================================================================

/// 初始化脚本系统
pub fn setup_scripting(world: &mut World) {
    world.insert_resource(ScriptEngine::new());
}

/// 脚本初始化系统 - 为新脚本创建上下文
pub fn script_init_system(
    mut engine: ResMut<ScriptEngine>,
    query: Query<(Entity, &Script), (Added<Script>, Without<ScriptState>)>,
    mut commands: Commands,
) {
    for (entity, script) in query.iter() {
        engine.create_context(entity);
        
        // 执行脚本源码
        if !script.source.is_empty() {
            if let Err(e) = engine.execute(entity, &script.source) {
                tracing::error!(target: "scripting", "Failed to initialize script: {}", e);
            }
        }
        
        // 添加状态组件
        commands.entity(entity).insert(ScriptState::default());
        
        // 调用 onStart
        let _ = engine.execute(entity, "if (typeof onStart === 'function') onStart();");
    }
}

/// 脚本更新系统
pub fn script_update_system(
    engine: Res<ScriptEngine>,
    time: Res<crate::ecs::Time>,
    query: Query<(Entity, &Script, &ScriptState)>,
) {
    // 更新全局时间
    engine.update_globals(
        time.delta,
        time.elapsed as f64,
        time.frame_count as u64,
    );
    
    for (entity, script, state) in query.iter() {
        if !script.enabled || !state.initialized {
            continue;
        }
        
        // 调用 onUpdate
        let code = format!("if (typeof onUpdate === 'function') onUpdate({});", time.delta);
        if let Err(e) = engine.execute(entity, &code) {
            tracing::error!(target: "scripting", "Script update error: {}", e);
        }
    }
}

/// 脚本命令处理系统
pub fn script_command_system(
    engine: Res<ScriptEngine>,
    mut commands: Commands,
) {
    for (source_entity, cmd) in engine.poll_commands() {
        match cmd {
            BindingCommand::SpawnEntity { components } => {
                let mut entity_cmd = commands.spawn_empty();
                for comp in components {
                    match comp {
                        ComponentData::Transform { position, rotation, scale } => {
                            entity_cmd.insert(crate::ecs::Transform {
                                pos: glam::Vec3::new(position[0], position[1], position[2]),
                                rot: glam::Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]),
                                scale: glam::Vec3::new(scale[0], scale[1], scale[2]),
                            });
                        }
                        ComponentData::Sprite { color, .. } => {
                            entity_cmd.insert(crate::ecs::Sprite {
                                color,
                                ..Default::default()
                            });
                        }
                        _ => {}
                    }
                }
            }
            BindingCommand::DespawnEntity { entity_id } => {
                if let Some(entity) = Entity::try_from_bits(entity_id) {
                    commands.entity(entity).despawn();
                }
            }
            BindingCommand::PlaySound { name, path, volume, looped } => {
                tracing::info!(target: "scripting", "Play sound: {} from {} (vol: {}, loop: {})", name, path, volume, looped);
                // TODO: 实际音频播放
            }
            BindingCommand::StopSound { name } => {
                tracing::info!(target: "scripting", "Stop sound: {}", name);
            }
            _ => {}
        }
    }
}

/// 脚本销毁系统
pub fn script_cleanup_system(
    mut engine: ResMut<ScriptEngine>,
    mut removed: RemovedComponents<Script>,
) {
    for entity in removed.read() {
        // 调用 onDestroy
        let _ = engine.execute(entity, "if (typeof onDestroy === 'function') onDestroy();");
        engine.remove_context(entity);
    }
}

// ============================================================================
// 脚本示例
// ============================================================================

/// 示例脚本模板
pub const EXAMPLE_SCRIPT: &str = r#"
// 游戏对象脚本示例

let speed = 100;
let health = 100;

function onStart() {
    console.log("Entity started: " + Engine.getEntity());
}

function onUpdate(deltaTime) {
    // 输入处理
    let dx = 0, dy = 0;
    
    if (Input.isKeyPressed("W") || Input.isKeyPressed("ArrowUp")) dy -= 1;
    if (Input.isKeyPressed("S") || Input.isKeyPressed("ArrowDown")) dy += 1;
    if (Input.isKeyPressed("A") || Input.isKeyPressed("ArrowLeft")) dx -= 1;
    if (Input.isKeyPressed("D") || Input.isKeyPressed("ArrowRight")) dx += 1;
    
    // 归一化方向
    if (dx !== 0 || dy !== 0) {
        let norm = GameMath.normalize(dx, dy);
        dx = norm[0];
        dy = norm[1];
    }
    
    // 移动
    // Transform.translate(dx * speed * deltaTime, dy * speed * deltaTime, 0);
}

function onDestroy() {
    console.log("Entity destroyed");
}

// 事件处理
function __onEngineEvent(event) {
    if (event.OnCollisionEnter) {
        console.log("Collision with: " + event.OnCollisionEnter.entity_b);
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_script_engine_creation() {
        let engine = ScriptEngine::new();
        assert!(engine.contexts.is_empty());
    }
    
    #[test]
    fn test_global_bindings() {
        let engine = ScriptEngine::new();
        engine.update_globals(0.016, 1.0, 60);
        
        let bindings = engine.global_bindings.lock().unwrap();
        assert!((bindings.time.delta_time - 0.016).abs() < 0.0001);
    }
}
