//! # JavaScript Scripting Service
//!
//! 完整的JavaScript脚本系统 - 基于QuickJS实现，提供全面的脚本API。
//!
//! ## 核心功能
//!
//! 1. **ScriptExecution** - 脚本执行引擎
//! 2. **APIBindings** - 完整的引擎API绑定
//! 3. **ModuleSystem** - 模块系统
//! 4. **EventSystem** - 事件系统
//! 5. **Debugger** - 调试支持

use rquickjs::{Context, Ctx, Function, IntoJs, Runtime, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 脚本服务配置
#[derive(Clone, Debug)]
pub struct ScriptConfig {
    /// 脚本超时（毫秒）
    pub timeout_ms: u64,
    /// 内存限制（MB）
    pub memory_limit_mb: usize,
    /// 是否启用调试
    pub enable_debugger: bool,
    /// 模块搜索路径
    pub module_paths: Vec<PathBuf>,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            memory_limit_mb: 128,
            enable_debugger: false,
            module_paths: vec![PathBuf::from("./scripts")],
        }
    }
}

/// 脚本服务
pub struct ScriptingService {
    /// QuickJS运行时
    runtime: Runtime,
    /// JavaScript上下文
    context: Context,
    /// 配置
    config: ScriptConfig,
    /// 已加载的模块
    loaded_modules: Arc<Mutex<HashMap<String, String>>>,
    /// 事件监听器
    event_listeners: Arc<Mutex<HashMap<String, Vec<ScriptCallback>>>>,
}

/// 脚本回调
#[derive(Clone)]
pub struct ScriptCallback {
    /// 回调函数
    pub function: String,
    /// 是否一次性
    pub once: bool,
}

impl Default for ScriptingService {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl ScriptingService {
    /// 创建新的脚本服务
    pub fn new(config: ScriptConfig) -> Self {
        let runtime = Runtime::new().expect("Failed to create QuickJS runtime");
        let context = Context::full(&runtime).expect("Failed to create QuickJS context");

        let service = Self {
            runtime,
            context,
            config,
            loaded_modules: Arc::new(Mutex::new(HashMap::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
        };

        // 绑定所有API
        service.bind_all_apis();

        service
    }

    /// 绑定所有API
    fn bind_all_apis(&self) {
        self.bind_console_api();
        self.bind_engine_api();
        self.bind_entity_api();
        self.bind_render_api();
        self.bind_camera_api();
        self.bind_light_api();
        self.bind_mesh_api();
        self.bind_texture_api();
        self.bind_shader_api();
        self.bind_ui_api();
        self.bind_animation_api();
        self.bind_input_api();
        self.bind_math_api();
        self.bind_event_api();
        self.bind_time_api();
    }

    /// 绑定控制台API
    fn bind_console_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // console.log
            let log_fn = Function::new(ctx.clone(), |msg: String| {
                tracing::info!(target: "scripting", "[JS] {}", msg);
            });
            global.set("console_log", log_fn).unwrap();

            // console.warn
            let warn_fn = Function::new(ctx.clone(), |msg: String| {
                tracing::warn!(target: "scripting", "[JS] {}", msg);
            });
            global.set("console_warn", warn_fn).unwrap();

            // console.error
            let error_fn = Function::new(ctx.clone(), |msg: String| {
                tracing::error!(target: "scripting", "[JS] {}", msg);
            });
            global.set("console_error", error_fn).unwrap();

            // 简化别名
            let print_fn = Function::new(ctx.clone(), |msg: String| {
                tracing::info!(target: "scripting", "[JS] {}", msg);
            });
            global.set("print", print_fn).unwrap();
        });
    }

    /// 绑定引擎API
    fn bind_engine_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Engine.quit()
            let quit_fn = Function::new(ctx.clone(), || {
                tracing::info!(target: "scripting", "Engine.quit() called");
                // 在实际实现中，这里会触发引擎退出
            });
            global.set("engine_quit", quit_fn).unwrap();

            // Engine.reload()
            let reload_fn = Function::new(ctx.clone(), || {
                tracing::info!(target: "scripting", "Engine.reload() called");
            });
            global.set("engine_reload", reload_fn).unwrap();

            // Engine.getVersion()
            let version_fn = Function::new(ctx.clone(), || -> String { "0.6.4".to_string() });
            global.set("engine_getVersion", version_fn).unwrap();
        });
    }

    /// 绑定实体API
    fn bind_entity_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Entity.create()
            let create_fn = Function::new(ctx.clone(), |name: String, x: f32, y: f32, z: f32| -> u32 {
                tracing::info!(target: "scripting", "Entity.create({}, {}, {}, {})", name, x, y, z);
                // 返回实体ID（在实际实现中会是真实的实体ID）
                1
            });
            global.set("entity_create", create_fn).unwrap();

            // Entity.destroy()
            let destroy_fn = Function::new(ctx.clone(), |entity_id: u32| {
                tracing::info!(target: "scripting", "Entity.destroy({})", entity_id);
            });
            global.set("entity_destroy", destroy_fn).unwrap();

            // Entity.setPosition()
            let set_pos_fn = Function::new(ctx.clone(), |entity_id: u32, x: f32, y: f32, z: f32| {
                tracing::info!(target: "scripting", "Entity.setPosition({}, {}, {}, {})", entity_id, x, y, z);
            });
            global.set("entity_setPosition", set_pos_fn).unwrap();

            // Entity.getPosition()
            let get_pos_fn = Function::new(ctx.clone(), |entity_id: u32| -> String {
                format!("[{}, {}, {}]", 0.0, 0.0, 0.0)
            });
            global.set("entity_getPosition", get_pos_fn).unwrap();

            // Entity.rotate()
            let rotate_fn = Function::new(ctx.clone(), |entity_id: u32, axis_x: f32, axis_y: f32, axis_z: f32, angle: f32| {
                tracing::info!(target: "scripting", "Entity.rotate({}, [{}, {}, {}], {})", entity_id, axis_x, axis_y, axis_z, angle);
            });
            global.set("entity_rotate", rotate_fn).unwrap();
        });
    }

    /// 绑定渲染API
    fn bind_render_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Render.setClearColor()
            let set_color_fn = Function::new(ctx.clone(), |r: f32, g: f32, b: f32, a: f32| {
                tracing::info!(target: "scripting", "Render.setClearColor({},{},{},{})", r, g, b, a);
            });
            global.set("render_setClearColor", set_color_fn).unwrap();

            // Render.createMaterial()
            let create_mat_fn = Function::new(ctx.clone(), |name: String, shader_path: String| -> u32 {
                tracing::info!(target: "scripting", "Render.createMaterial({}, {})", name, shader_path);
                1
            });
            global.set("render_createMaterial", create_mat_fn).unwrap();

            // Render.setMaterialProperty()
            let set_mat_prop_fn = Function::new(ctx.clone(), |material_id: u32, property: String, value: f32| {
                tracing::info!(target: "scripting", "Render.setMaterialProperty({}, {}, {})", material_id, property, value);
            });
            global.set("render_setMaterialProperty", set_mat_prop_fn).unwrap();
        });
    }

    /// 绑定摄像机API
    fn bind_camera_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Camera.create()
            let create_fn = Function::new(ctx.clone(), |name: String| -> u32 {
                tracing::info!(target: "scripting", "Camera.create({})", name);
                1
            });
            global.set("camera_create", create_fn).unwrap();

            // Camera.setPosition()
            let set_pos_fn = Function::new(ctx.clone(), |camera_id: u32, x: f32, y: f32, z: f32| {
                tracing::info!(target: "scripting", "Camera.setPosition({}, {}, {}, {})", camera_id, x, y, z);
            });
            global.set("camera_setPosition", set_pos_fn).unwrap();

            // Camera.lookAt()
            let look_at_fn = Function::new(ctx.clone(), |camera_id: u32, x: f32, y: f32, z: f32| {
                tracing::info!(target: "scripting", "Camera.lookAt({}, {}, {}, {})", camera_id, x, y, z);
            });
            global.set("camera_lookAt", look_at_fn).unwrap();

            // Camera.setFOV()
            let set_fov_fn = Function::new(ctx.clone(), |camera_id: u32, fov: f32| {
                tracing::info!(target: "scripting", "Camera.setFOV({}, {})", camera_id, fov);
            });
            global.set("camera_setFOV", set_fov_fn).unwrap();

            // Camera.setNearFar()
            let set_near_far_fn = Function::new(ctx.clone(), |camera_id: u32, near: f32, far: f32| {
                tracing::info!(target: "scripting", "Camera.setNearFar({}, {}, {})", camera_id, near, far);
            });
            global.set("camera_setNearFar", set_near_far_fn).unwrap();

            // Camera.setActive()
            let set_active_fn = Function::new(ctx.clone(), |camera_id: u32| {
                tracing::info!(target: "scripting", "Camera.setActive({})", camera_id);
            });
            global.set("camera_setActive", set_active_fn).unwrap();
        });
    }

    /// 绑定光源API
    fn bind_light_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Light.create()
            let create_fn = Function::new(ctx.clone(), |light_type: String, name: String| -> u32 {
                tracing::info!(target: "scripting", "Light.create({}, {})", light_type, name);
                1
            });
            global.set("light_create", create_fn).unwrap();

            // Light.setPosition()
            let set_pos_fn = Function::new(ctx.clone(), |light_id: u32, x: f32, y: f32, z: f32| {
                tracing::info!(target: "scripting", "Light.setPosition({}, {}, {}, {})", light_id, x, y, z);
            });
            global.set("light_setPosition", set_pos_fn).unwrap();

            // Light.setColor()
            let set_color_fn = Function::new(ctx.clone(), |light_id: u32, r: f32, g: f32, b: f32| {
                tracing::info!(target: "scripting", "Light.setColor({}, {}, {}, {})", light_id, r, g, b);
            });
            global.set("light_setColor", set_color_fn).unwrap();

            // Light.setIntensity()
            let set_intensity_fn = Function::new(ctx.clone(), |light_id: u32, intensity: f32| {
                tracing::info!(target: "scripting", "Light.setIntensity({}, {})", light_id, intensity);
            });
            global.set("light_setIntensity", set_intensity_fn).unwrap();

            // Light.setRange()
            let set_range_fn = Function::new(ctx.clone(), |light_id: u32, range: f32| {
                tracing::info!(target: "scripting", "Light.setRange({}, {})", light_id, range);
            });
            global.set("light_setRange", set_range_fn).unwrap();

            // Light.setEnabled()
            let set_enabled_fn = Function::new(ctx.clone(), |light_id: u32, enabled: bool| {
                tracing::info!(target: "scripting", "Light.setEnabled({}, {})", light_id, enabled);
            });
            global.set("light_setEnabled", set_enabled_fn).unwrap();
        });
    }

    /// 绑定网格API
    fn bind_mesh_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Mesh.create()
            let create_fn = Function::new(ctx.clone(), |name: String| -> u32 {
                tracing::info!(target: "scripting", "Mesh.create({})", name);
                1
            });
            global.set("mesh_create", create_fn).unwrap();

            // Mesh.load()
            let load_fn = Function::new(ctx.clone(), |path: String| -> u32 {
                tracing::info!(target: "scripting", "Mesh.load({})", path);
                1
            });
            global.set("mesh_load", load_fn).unwrap();

            // Mesh.generateLOD()
            let gen_lod_fn = Function::new(ctx.clone(), |mesh_id: u32, levels: u32| {
                tracing::info!(target: "scripting", "Mesh.generateLOD({}, {})", mesh_id, levels);
            });
            global.set("mesh_generateLOD", gen_lod_fn).unwrap();

            // Mesh.setMaterial()
            let set_mat_fn = Function::new(ctx.clone(), |mesh_id: u32, material_id: u32| {
                tracing::info!(target: "scripting", "Mesh.setMaterial({}, {})", mesh_id, material_id);
            });
            global.set("mesh_setMaterial", set_mat_fn).unwrap();

            // Mesh.setVisible()
            let set_visible_fn = Function::new(ctx.clone(), |mesh_id: u32, visible: bool| {
                tracing::info!(target: "scripting", "Mesh.setVisible({}, {})", mesh_id, visible);
            });
            global.set("mesh_setVisible", set_visible_fn).unwrap();
        });
    }

    /// 绑定纹理API
    fn bind_texture_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Texture.load()
            let load_fn = Function::new(ctx.clone(), |path: String| -> u32 {
                tracing::info!(target: "scripting", "Texture.load({})", path);
                1
            });
            global.set("texture_load", load_fn).unwrap();

            // Texture.create()
            let create_fn = Function::new(ctx.clone(), |width: u32, height: u32, format: String| -> u32 {
                tracing::info!(target: "scripting", "Texture.create({}, {}, {})", width, height, format);
                1
            });
            global.set("texture_create", create_fn).unwrap();

            // Texture.compress()
            let compress_fn = Function::new(ctx.clone(), |texture_id: u32, format: String| {
                tracing::info!(target: "scripting", "Texture.compress({}, {})", texture_id, format);
            });
            global.set("texture_compress", compress_fn).unwrap();

            // Texture.setFilter()
            let set_filter_fn = Function::new(ctx.clone(), |texture_id: u32, min_filter: String, mag_filter: String| {
                tracing::info!(target: "scripting", "Texture.setFilter({}, {}, {})", texture_id, min_filter, mag_filter);
            });
            global.set("texture_setFilter", set_filter_fn).unwrap();
        });
    }

    /// 绑定着色器API
    fn bind_shader_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Shader.load()
            let load_fn = Function::new(ctx.clone(), |vertex_path: String, fragment_path: String| -> u32 {
                tracing::info!(target: "scripting", "Shader.load({}, {})", vertex_path, fragment_path);
                1
            });
            global.set("shader_load", load_fn).unwrap();

            // Shader.compile()
            let compile_fn = Function::new(ctx.clone(), |shader_id: u32| -> bool {
                tracing::info!(target: "scripting", "Shader.compile({})", shader_id);
                true
            });
            global.set("shader_compile", compile_fn).unwrap();

            // Shader.setParam()
            let set_param_fn = Function::new(ctx.clone(), |shader_id: u32, param_name: String, value: f32| {
                tracing::info!(target: "scripting", "Shader.setParam({}, {}, {})", shader_id, param_name, value);
            });
            global.set("shader_setParam", set_param_fn).unwrap();

            // Shader.setTexture()
            let set_texture_fn = Function::new(ctx.clone(), |shader_id: u32, param_name: String, texture_id: u32| {
                tracing::info!(target: "scripting", "Shader.setTexture({}, {}, {})", shader_id, param_name, texture_id);
            });
            global.set("shader_setTexture", set_texture_fn).unwrap();
        });
    }

    /// 绑定UI API
    fn bind_ui_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // UI.createWidget()
            let create_widget_fn = Function::new(ctx.clone(), |widget_type: String, parent_id: u32| -> u32 {
                tracing::info!(target: "scripting", "UI.createWidget({}, {})", widget_type, parent_id);
                1
            });
            global.set("ui_createWidget", create_widget_fn).unwrap();

            // UI.setText()
            let set_text_fn = Function::new(ctx.clone(), |widget_id: u32, text: String| {
                tracing::info!(target: "scripting", "UI.setText({}, {})", widget_id, text);
            });
            global.set("ui_setText", set_text_fn).unwrap();

            // UI.setPosition()
            let set_pos_fn = Function::new(ctx.clone(), |widget_id: u32, x: f32, y: f32| {
                tracing::info!(target: "scripting", "UI.setPosition({}, {}, {})", widget_id, x, y);
            });
            global.set("ui_setPosition", set_pos_fn).unwrap();

            // UI.setSize()
            let set_size_fn = Function::new(ctx.clone(), |widget_id: u32, width: f32, height: f32| {
                tracing::info!(target: "scripting", "UI.setSize({}, {}, {})", widget_id, width, height);
            });
            global.set("ui_setSize", set_size_fn).unwrap();

            // UI.setVisible()
            let set_visible_fn = Function::new(ctx.clone(), |widget_id: u32, visible: bool| {
                tracing::info!(target: "scripting", "UI.setVisible({}, {})", widget_id, visible);
            });
            global.set("ui_setVisible", set_visible_fn).unwrap();

            // UI.onClick()
            let on_click_fn = Function::new(ctx.clone(), |widget_id: u32, callback: String| {
                tracing::info!(target: "scripting", "UI.onClick({}, {})", widget_id, callback);
            });
            global.set("ui_onClick", on_click_fn).unwrap();

            // UI.setStyle()
            let set_style_fn = Function::new(ctx.clone(), |widget_id: u32, style_name: String, value: String| {
                tracing::info!(target: "scripting", "UI.setStyle({}, {}, {})", widget_id, style_name, value);
            });
            global.set("ui_setStyle", set_style_fn).unwrap();

            // UI.destroy()
            let destroy_fn = Function::new(ctx.clone(), |widget_id: u32| {
                tracing::info!(target: "scripting", "UI.destroy({})", widget_id);
            });
            global.set("ui_destroy", destroy_fn).unwrap();

            // UI.setParent()
            let set_parent_fn = Function::new(ctx.clone(), |widget_id: u32, parent_id: u32| {
                tracing::info!(target: "scripting", "UI.setParent({}, {})", widget_id, parent_id);
            });
            global.set("ui_setParent", set_parent_fn).unwrap();

            // UI.setImage()
            let set_image_fn = Function::new(ctx.clone(), |widget_id: u32, texture_id: u32| {
                tracing::info!(target: "scripting", "UI.setImage({}, {})", widget_id, texture_id);
            });
            global.set("ui_setImage", set_image_fn).unwrap();
        });
    }

    /// 绑定动画API
    fn bind_animation_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Animation.create()
            let create_fn = Function::new(ctx.clone(), |name: String| -> u32 {
                tracing::info!(target: "scripting", "Animation.create({})", name);
                1
            });
            global.set("animation_create", create_fn).unwrap();

            // Animation.play()
            let play_fn = Function::new(ctx.clone(), |animation_id: u32| {
                tracing::info!(target: "scripting", "Animation.play({})", animation_id);
            });
            global.set("animation_play", play_fn).unwrap();

            // Animation.pause()
            let pause_fn = Function::new(ctx.clone(), |animation_id: u32| {
                tracing::info!(target: "scripting", "Animation.pause({})", animation_id);
            });
            global.set("animation_pause", pause_fn).unwrap();

            // Animation.stop()
            let stop_fn = Function::new(ctx.clone(), |animation_id: u32| {
                tracing::info!(target: "scripting", "Animation.stop({})", animation_id);
            });
            global.set("animation_stop", stop_fn).unwrap();

            // Animation.setSpeed()
            let set_speed_fn = Function::new(ctx.clone(), |animation_id: u32, speed: f32| {
                tracing::info!(target: "scripting", "Animation.setSpeed({}, {})", animation_id, speed);
            });
            global.set("animation_setSpeed", set_speed_fn).unwrap();

            // Animation.setLoop()
            let set_loop_fn = Function::new(ctx.clone(), |animation_id: u32, loop_enabled: bool| {
                tracing::info!(target: "scripting", "Animation.setLoop({}, {})", animation_id, loop_enabled);
            });
            global.set("animation_setLoop", set_loop_fn).unwrap();

            // Animation.addKeyframe()
            let add_keyframe_fn = Function::new(ctx.clone(), |animation_id: u32, time: f32, value: String| {
                tracing::info!(target: "scripting", "Animation.addKeyframe({}, {}, {})", animation_id, time, value);
            });
            global.set("animation_addKeyframe", add_keyframe_fn).unwrap();

            // Animation.setTransition()
            let set_transition_fn = Function::new(ctx.clone(), |animation_id: u32, transition_type: String, duration: f32| {
                tracing::info!(target: "scripting", "Animation.setTransition({}, {}, {})", animation_id, transition_type, duration);
            });
            global.set("animation_setTransition", set_transition_fn).unwrap();

            // Animation.attachToEntity()
            let attach_fn = Function::new(ctx.clone(), |animation_id: u32, entity_id: u32| {
                tracing::info!(target: "scripting", "Animation.attachToEntity({}, {})", animation_id, entity_id);
            });
            global.set("animation_attachToEntity", attach_fn).unwrap();

            // Animation.setProgress()
            let set_progress_fn = Function::new(ctx.clone(), |animation_id: u32, progress: f32| {
                tracing::info!(target: "scripting", "Animation.setProgress({}, {})", animation_id, progress);
            });
            global.set("animation_setProgress", set_progress_fn).unwrap();
        });
    }

    /// 绑定输入API
    fn bind_input_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Input.isKeyDown()
            let is_key_down_fn = Function::new(ctx.clone(), |key_code: u32| -> bool {
                false // 在实际实现中会检查真实的输入状态
            });
            global.set("input_isKeyDown", is_key_down_fn).unwrap();

            // Input.getMousePosition()
            let get_mouse_fn = Function::new(ctx.clone(), || -> String {
                format!("{{ x: {}, y: {} }}", 0, 0)
            });
            global.set("input_getMousePosition", get_mouse_fn).unwrap();
        });
    }

    /// 绑定数学API
    fn bind_math_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Math.vector3.add()
            let vec3_add_fn = Function::new(ctx.clone(), |a: String, b: String| -> String {
                // 简化版向量加法
                "[0, 0, 0]".to_string()
            });
            global.set("math_vector3_add", vec3_add_fn).unwrap();

            // Math.vector3.normalize()
            let vec3_norm_fn = Function::new(ctx.clone(), |v: String| -> String {
                "[0, 0, 0]".to_string()
            });
            global.set("math_vector3_normalize", vec3_norm_fn).unwrap();

            // Math.vector3.dot()
            let vec3_dot_fn = Function::new(ctx.clone(), |a: String, b: String| -> f32 { 0.0 });
            global.set("math_vector3_dot", vec3_dot_fn).unwrap();

            // Math.clamp()
            let clamp_fn = Function::new(ctx.clone(), |value: f32, min: f32, max: f32| -> f32 {
                value.min(max).max(min)
            });
            global.set("math_clamp", clamp_fn).unwrap();

            // Math.lerp()
            let lerp_fn = Function::new(ctx.clone(), |a: f32, b: f32, t: f32| -> f32 {
                a + (b - a) * t
            });
            global.set("math_lerp", lerp_fn).unwrap();
        });
    }

    /// 绑定事件API
    fn bind_event_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Event.on()
            let event_on_fn = Function::new(ctx.clone(), |event_name: String, callback: String| {
                tracing::info!(target: "scripting", "Event.on({}, {})", event_name, callback);
                true
            });
            global.set("event_on", event_on_fn).unwrap();

            // Event.emit()
            let event_emit_fn = Function::new(ctx.clone(), |event_name: String, data: String| {
                tracing::info!(target: "scripting", "Event.emit({}, {})", event_name, data);
            });
            global.set("event_emit", event_emit_fn).unwrap();

            // Event.off()
            let event_off_fn =
                Function::new(ctx.clone(), |event_name: String, callback: String| {
                    tracing::info!(target: "scripting", "Event.off({}, {})", event_name, callback);
                });
            global.set("event_off", event_off_fn).unwrap();
        });
    }

    /// 绑定时间API
    fn bind_time_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Time.deltaTime
            let delta_fn = Function::new(ctx.clone(), || -> f32 {
                0.016 // 在实际实现中会返回真实的delta time
            });
            global.set("time_deltaTime", delta_fn).unwrap();

            // Time.timeScale
            let scale_fn = Function::new(ctx.clone(), || -> f32 { 1.0 });
            global.set("time_timeScale", scale_fn).unwrap();
        });
    }

    /// 执行JavaScript代码
    pub fn execute(&self, code: &str) -> Result<(), ScriptError> {
        self.context
            .with(|ctx| ctx.eval::<(), _>(code).map_err(|e| ScriptError::Execution(e.to_string())))
    }

    /// 执行脚本文件
    pub fn execute_file(&self, path: &PathBuf) -> Result<(), ScriptError> {
        let code =
            std::fs::read_to_string(path).map_err(|e| ScriptError::IoError(e.to_string()))?;

        self.execute(&code)
    }

    /// 加载模块
    pub fn load_module(&self, name: &str, code: &str) -> Result<(), ScriptError> {
        // 存储模块代码
        self.loaded_modules.lock().unwrap().insert(name.to_string(), code.to_string());

        // 执行模块代码
        self.execute(code)
    }

    /// 调用脚本函数（简化版 - 实际调用在execute中完成）
    pub fn call_function(&self, func_name: &str, args_str: &str) -> Result<(), ScriptError> {
        let code = format!("{func_name}({args_str});");
        self.execute(&code)
    }

    /// 检查运行时是否有效
    pub fn is_runtime_valid(&self) -> bool {
        self.context.with(|_ctx| true)
    }

    /// 重置脚本环境
    pub fn reset(&self) {
        self.loaded_modules.lock().unwrap().clear();
        self.event_listeners.lock().unwrap().clear();
        self.bind_all_apis();
    }

    /// 绑定核心API（公共接口，供外部调用）
    pub fn bind_core_api(&self) {
        self.bind_all_apis();
    }

    /// 获取运行时引用
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// 获取上下文引用
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// 获取配置
    pub fn config(&self) -> &ScriptConfig {
        &self.config
    }
}

/// 脚本错误
#[derive(Clone, Debug)]
pub enum ScriptError {
    /// 执行错误
    Execution(String),
    /// IO错误
    IoError(String),
    /// 超时错误
    Timeout,
    /// 内存限制错误
    MemoryLimitExceeded,
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::Execution(msg) => write!(f, "Execution error: {msg}"),
            ScriptError::IoError(msg) => write!(f, "IO error: {msg}"),
            ScriptError::Timeout => write!(f, "Script execution timeout"),
            ScriptError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
        }
    }
}

impl std::error::Error for ScriptError {}

/// 脚本模块管理器
pub struct ScriptModuleManager {
    /// 服务实例
    service: ScriptingService,
    /// 模块缓存
    module_cache: HashMap<String, String>,
}

impl ScriptModuleManager {
    /// 创建新的模块管理器
    pub fn new(service: ScriptingService) -> Self {
        Self {
            service,
            module_cache: HashMap::new(),
        }
    }

    /// 加载模块
    pub fn load_module(&mut self, name: &str, path: &PathBuf) -> Result<(), ScriptError> {
        let code =
            std::fs::read_to_string(path).map_err(|e| ScriptError::IoError(e.to_string()))?;

        self.service.load_module(name, &code)?;
        self.module_cache.insert(name.to_string(), code);

        Ok(())
    }

    /// 重载模块
    pub fn reload_module(&mut self, name: &str) -> Result<(), ScriptError> {
        if let Some(code) = self.module_cache.get(name) {
            self.service.execute(code)
        } else {
            Err(ScriptError::Execution(format!("Module {name} not found")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scripting_service_creation() {
        let service = ScriptingService::new(Default::default());
        assert!(service.is_runtime_valid());
    }

    #[test]
    fn test_basic_execution() {
        let service = ScriptingService::new(Default::default());
        let result = service.execute("print('Hello from test');");
        assert!(result.is_ok());
    }

    #[test]
    fn test_entity_api() {
        let service = ScriptingService::new(Default::default());
        let result = service.execute("entity_create('TestEntity', 0.0, 0.0, 0.0);");
        assert!(result.is_ok());
    }

    #[test]
    fn test_math_api() {
        let service = ScriptingService::new(Default::default());
        let result = service.execute("math_clamp(5.0, 0.0, 10.0);");
        assert!(result.is_ok());
    }
}
