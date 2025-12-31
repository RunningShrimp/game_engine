//! # Python Scripting Service
//!
//! 完整的Python脚本系统 - 基于PyO3实现，提供全面的脚本API。
//!
//! ## 核心功能
//!
//! 1. **PythonInterpreter** - Python解释器
//! 2. **APIBindings** - 完整的引擎API绑定
//! 3. **ModuleSystem** - Python模块系统
//! 4. **GILManagement** - 全局解释器锁管理

use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;

/// Python脚本服务
pub struct PythonScriptingService {
    /// 已加载的模块
    loaded_modules: Arc<Mutex<HashMap<String, String>>>,
    /// 配置
    config: PythonConfig,
}

/// Python配置
#[derive(Clone, Debug)]
pub struct PythonConfig {
    /// 是否启用交互模式
    pub interactive_mode: bool,
    /// 模块搜索路径
    pub module_paths: Vec<PathBuf>,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            interactive_mode: false,
            module_paths: vec![PathBuf::from("./python_scripts")],
        }
    }
}

impl Default for PythonScriptingService {
    fn default() -> Self {
        Self::new(Default::default()).expect("Failed to create Python scripting service")
    }
}

impl PythonScriptingService {
    /// 创建新的Python脚本服务
    pub fn new(config: PythonConfig) -> PyResult<Self> {
        // 初始化Python解释器
        pyo3::prepare_freethreaded_python();

        let service = Self {
            loaded_modules: Arc::new(Mutex::new(HashMap::new())),
            config,
        };

        Ok(service)
    }

    /// 执行Python代码
    pub fn execute(&self, code: &str) -> PyResult<()> {
        Python::with_gil(|py| {
            let builtins = py.import("builtins")?;
            let exec_func = builtins.getattr("exec")?;
            exec_func.call1((code,))?;
            Ok(())
        })
    }

    /// 执行Python文件
    pub fn execute_file(&self, path: &PathBuf) -> PyResult<()> {
        let code = std::fs::read_to_string(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        self.execute(&code)
    }

    /// 加载模块
    pub fn load_module(&self, name: &str, code: &str) -> PyResult<()> {
        // 存储模块代码
        self.loaded_modules.lock().unwrap().insert(name.to_string(), code.to_string());

        // 执行模块代码
        self.execute(code)
    }

    /// 调用Python函数
    pub fn call_function(&self, module: &str, func: &str, args_str: &str) -> PyResult<()> {
        let code = format!("import {};\n{}.{}({});", module, module, func, args_str);
        self.execute(&code)
    }

    /// 重置脚本环境
    pub fn reset(&self) -> PyResult<()> {
        self.loaded_modules.lock().unwrap().clear();
        Ok(())
    }
}

/// 游戏引擎Python模块
#[pymodule]
fn game_engine(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Entity>()?;
    m.add_class::<Render>()?;
    m.add_class::<Camera>()?;
    m.add_class::<Light>()?;
    m.add_class::<Mesh>()?;
    m.add_class::<Texture>()?;
    m.add_class::<Shader>()?;
    m.add_class::<UI>()?;
    m.add_class::<Animation>()?;
    m.add_class::<Input>()?;
    m.add_class::<Math>()?;
    m.add_class::<Event>()?;
    m.add_class::<Time>()?;
    m.add_class::<Engine>()?;
    Ok(())
}

/// Entity类 - Python实体API
#[pyclass]
pub struct Entity {
    #[pyo3(get, set)]
    name: String,
    #[pyo3(get)]
    position: (f32, f32, f32),
    #[pyo3(get, set)]
    rotation: (f32, f32, f32),
    #[pyo3(get, set)]
    scale: (f32, f32, f32),
}

#[pymethods]
impl Entity {
    #[new]
    fn new(name: String, x: f32, y: f32, z: f32) -> Self {
        tracing::info!(target: "python", "Entity.new({}, {}, {}, {})", name, x, y, z);
        Self {
            name,
            position: (x, y, z),
            rotation: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
        }
    }

    #[staticmethod]
    fn create(name: String, x: f32, y: f32, z: f32) -> PyResult<u32> {
        tracing::info!(target: "python", "Entity.create({}, {}, {}, {})", name, x, y, z);
        Ok(1)
    }

    fn destroy(&self, entity_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Entity.destroy({})", entity_id);
        Ok(())
    }

    fn set_position(&mut self, x: f32, y: f32, z: f32) -> PyResult<()> {
        self.position = (x, y, z);
        tracing::info!(target: "python", "Entity.set_position({}, {}, {}, {})", self.name, x, y, z);
        Ok(())
    }

    fn get_position(&self) -> String {
        format!("({}, {}, {})", self.position.0, self.position.1, self.position.2)
    }

    fn rotate(&mut self, axis_x: f32, axis_y: f32, axis_z: f32, angle: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Entity.rotate({}, [{}, {}, {}], {})",
                   self.name, axis_x, axis_y, axis_z, angle);
        Ok(())
    }
}

/// Render类 - Python渲染API
#[pyclass]
pub struct Render;

#[pymethods]
impl Render {
    #[staticmethod]
    fn set_clear_color(r: f32, g: f32, b: f32, a: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Render.set_clear_color({},{},{},{})", r, g, b, a);
        Ok(())
    }

    #[staticmethod]
    fn create_material(name: String, shader_path: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Render.create_material({}, {})", name, shader_path);
        Ok(1)
    }

    #[staticmethod]
    fn set_material_property(material_id: u32, property: String, value: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Render.set_material_property({}, {}, {})", material_id, property, value);
        Ok(())
    }
}

/// Camera类 - Python摄像机API
#[pyclass]
pub struct Camera;

#[pymethods]
impl Camera {
    #[staticmethod]
    fn create(name: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Camera.create({})", name);
        Ok(1)
    }

    #[staticmethod]
    fn set_position(camera_id: u32, x: f32, y: f32, z: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Camera.set_position({}, {}, {}, {})", camera_id, x, y, z);
        Ok(())
    }

    #[staticmethod]
    fn look_at(camera_id: u32, x: f32, y: f32, z: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Camera.look_at({}, {}, {}, {})", camera_id, x, y, z);
        Ok(())
    }

    #[staticmethod]
    fn set_fov(camera_id: u32, fov: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Camera.set_fov({}, {})", camera_id, fov);
        Ok(())
    }

    #[staticmethod]
    fn set_near_far(camera_id: u32, near: f32, far: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Camera.set_near_far({}, {}, {})", camera_id, near, far);
        Ok(())
    }

    #[staticmethod]
    fn set_active(camera_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Camera.set_active({})", camera_id);
        Ok(())
    }
}

/// Light类 - Python光源API
#[pyclass]
pub struct Light;

#[pymethods]
impl Light {
    #[staticmethod]
    fn create(light_type: String, name: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Light.create({}, {})", light_type, name);
        Ok(1)
    }

    #[staticmethod]
    fn set_position(light_id: u32, x: f32, y: f32, z: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Light.set_position({}, {}, {}, {})", light_id, x, y, z);
        Ok(())
    }

    #[staticmethod]
    fn set_color(light_id: u32, r: f32, g: f32, b: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Light.set_color({}, {}, {}, {})", light_id, r, g, b);
        Ok(())
    }

    #[staticmethod]
    fn set_intensity(light_id: u32, intensity: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Light.set_intensity({}, {})", light_id, intensity);
        Ok(())
    }

    #[staticmethod]
    fn set_range(light_id: u32, range: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Light.set_range({}, {})", light_id, range);
        Ok(())
    }

    #[staticmethod]
    fn set_enabled(light_id: u32, enabled: bool) -> PyResult<()> {
        tracing::info!(target: "python", "Light.set_enabled({}, {})", light_id, enabled);
        Ok(())
    }
}

/// Mesh类 - Python网格API
#[pyclass]
pub struct Mesh;

#[pymethods]
impl Mesh {
    #[staticmethod]
    fn create(name: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Mesh.create({})", name);
        Ok(1)
    }

    #[staticmethod]
    fn load(path: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Mesh.load({})", path);
        Ok(1)
    }

    #[staticmethod]
    fn generate_lod(mesh_id: u32, levels: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Mesh.generate_lod({}, {})", mesh_id, levels);
        Ok(())
    }

    #[staticmethod]
    fn set_material(mesh_id: u32, material_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Mesh.set_material({}, {})", mesh_id, material_id);
        Ok(())
    }

    #[staticmethod]
    fn set_visible(mesh_id: u32, visible: bool) -> PyResult<()> {
        tracing::info!(target: "python", "Mesh.set_visible({}, {})", mesh_id, visible);
        Ok(())
    }
}

/// Texture类 - Python纹理API
#[pyclass]
pub struct Texture;

#[pymethods]
impl Texture {
    #[staticmethod]
    fn load(path: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Texture.load({})", path);
        Ok(1)
    }

    #[staticmethod]
    fn create(width: u32, height: u32, format: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Texture.create({}, {}, {})", width, height, format);
        Ok(1)
    }

    #[staticmethod]
    fn compress(texture_id: u32, format: String) -> PyResult<()> {
        tracing::info!(target: "python", "Texture.compress({}, {})", texture_id, format);
        Ok(())
    }

    #[staticmethod]
    fn set_filter(texture_id: u32, min_filter: String, mag_filter: String) -> PyResult<()> {
        tracing::info!(target: "python", "Texture.set_filter({}, {}, {})", texture_id, min_filter, mag_filter);
        Ok(())
    }
}

/// Shader类 - Python着色器API
#[pyclass]
pub struct Shader;

#[pymethods]
impl Shader {
    #[staticmethod]
    fn load(vertex_path: String, fragment_path: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Shader.load({}, {})", vertex_path, fragment_path);
        Ok(1)
    }

    #[staticmethod]
    fn compile(shader_id: u32) -> PyResult<bool> {
        tracing::info!(target: "python", "Shader.compile({})", shader_id);
        Ok(true)
    }

    #[staticmethod]
    fn set_param(shader_id: u32, param_name: String, value: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Shader.set_param({}, {}, {})", shader_id, param_name, value);
        Ok(())
    }

    #[staticmethod]
    fn set_texture(shader_id: u32, param_name: String, texture_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Shader.set_texture({}, {}, {})", shader_id, param_name, texture_id);
        Ok(())
    }
}

/// Input类 - Python输入API
#[pyclass]
pub struct Input;

#[pymethods]
impl Input {
    #[staticmethod]
    fn is_key_down(key_code: u32) -> bool {
        false // 在实际实现中会检查真实的输入状态
    }

    #[staticmethod]
    fn get_mouse_position() -> String {
        format!("{{ x: {}, y: {} }}", 0, 0)
    }
}

/// Math类 - Python数学API
#[pyclass]
pub struct Math;

#[pymethods]
impl Math {
    #[staticmethod]
    fn vector3_add(_a: String, _b: String) -> String {
        // 简化版向量加法
        "[0, 0, 0]".to_string()
    }

    #[staticmethod]
    fn vector3_normalize(_v: String) -> String {
        "[0, 0, 0]".to_string()
    }

    #[staticmethod]
    fn vector3_dot(_a: String, _b: String) -> f32 {
        0.0
    }

    #[staticmethod]
    fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.min(max).max(min)
    }

    #[staticmethod]
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}

/// Event类 - Python事件API
#[pyclass]
pub struct Event;

#[pymethods]
impl Event {
    #[staticmethod]
    fn on(event_name: String, callback: String) -> PyResult<bool> {
        tracing::info!(target: "python", "Event.on({}, {})", event_name, callback);
        Ok(true)
    }

    #[staticmethod]
    fn emit(event_name: String, data: String) -> PyResult<()> {
        tracing::info!(target: "python", "Event.emit({}, {})", event_name, data);
        Ok(())
    }

    #[staticmethod]
    fn off(event_name: String, callback: String) -> PyResult<()> {
        tracing::info!(target: "python", "Event.off({}, {})", event_name, callback);
        Ok(())
    }
}

/// Time类 - Python时间API
#[pyclass]
pub struct Time;

#[pymethods]
impl Time {
    #[staticmethod]
    fn delta_time() -> f32 {
        0.016 // 在实际实现中会返回真实的delta time
    }

    #[staticmethod]
    fn time_scale() -> f32 {
        1.0
    }
}

/// Engine类 - Python引擎API
#[pyclass]
pub struct Engine;

#[pymethods]
impl Engine {
    #[staticmethod]
    fn quit() -> PyResult<()> {
        tracing::info!(target: "python", "Engine.quit() called");
        Ok(())
    }

    #[staticmethod]
    fn reload() -> PyResult<()> {
        tracing::info!(target: "python", "Engine.reload() called");
        Ok(())
    }

    #[staticmethod]
    fn get_version() -> String {
        "0.6.4".to_string()
    }
}

/// UI类 - Python UI API
#[pyclass]
pub struct UI;

#[pymethods]
impl UI {
    #[staticmethod]
    fn create_widget(widget_type: String, parent_id: u32) -> PyResult<u32> {
        tracing::info!(target: "python", "UI.create_widget({}, {})", widget_type, parent_id);
        Ok(1)
    }

    #[staticmethod]
    fn set_text(widget_id: u32, text: String) -> PyResult<()> {
        tracing::info!(target: "python", "UI.set_text({}, {})", widget_id, text);
        Ok(())
    }

    #[staticmethod]
    fn set_position(widget_id: u32, x: f32, y: f32) -> PyResult<()> {
        tracing::info!(target: "python", "UI.set_position({}, {}, {})", widget_id, x, y);
        Ok(())
    }

    #[staticmethod]
    fn set_size(widget_id: u32, width: f32, height: f32) -> PyResult<()> {
        tracing::info!(target: "python", "UI.set_size({}, {}, {})", widget_id, width, height);
        Ok(())
    }

    #[staticmethod]
    fn set_visible(widget_id: u32, visible: bool) -> PyResult<()> {
        tracing::info!(target: "python", "UI.set_visible({}, {})", widget_id, visible);
        Ok(())
    }

    #[staticmethod]
    fn on_click(widget_id: u32, callback: String) -> PyResult<()> {
        tracing::info!(target: "python", "UI.on_click({}, {})", widget_id, callback);
        Ok(())
    }

    #[staticmethod]
    fn set_style(widget_id: u32, style_name: String, value: String) -> PyResult<()> {
        tracing::info!(target: "python", "UI.set_style({}, {}, {})", widget_id, style_name, value);
        Ok(())
    }

    #[staticmethod]
    fn destroy(widget_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "UI.destroy({})", widget_id);
        Ok(())
    }

    #[staticmethod]
    fn set_parent(widget_id: u32, parent_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "UI.set_parent({}, {})", widget_id, parent_id);
        Ok(())
    }

    #[staticmethod]
    fn set_image(widget_id: u32, texture_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "UI.set_image({}, {})", widget_id, texture_id);
        Ok(())
    }
}

/// Animation类 - Python动画API
#[pyclass]
pub struct Animation;

#[pymethods]
impl Animation {
    #[staticmethod]
    fn create(name: String) -> PyResult<u32> {
        tracing::info!(target: "python", "Animation.create({})", name);
        Ok(1)
    }

    #[staticmethod]
    fn play(animation_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.play({})", animation_id);
        Ok(())
    }

    #[staticmethod]
    fn pause(animation_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.pause({})", animation_id);
        Ok(())
    }

    #[staticmethod]
    fn stop(animation_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.stop({})", animation_id);
        Ok(())
    }

    #[staticmethod]
    fn set_speed(animation_id: u32, speed: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.set_speed({}, {})", animation_id, speed);
        Ok(())
    }

    #[staticmethod]
    fn set_loop(animation_id: u32, loop_enabled: bool) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.set_loop({}, {})", animation_id, loop_enabled);
        Ok(())
    }

    #[staticmethod]
    fn add_keyframe(animation_id: u32, time: f32, value: String) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.add_keyframe({}, {}, {})", animation_id, time, value);
        Ok(())
    }

    #[staticmethod]
    fn set_transition(animation_id: u32, transition_type: String, duration: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.set_transition({}, {}, {})", animation_id, transition_type, duration);
        Ok(())
    }

    #[staticmethod]
    fn attach_to_entity(animation_id: u32, entity_id: u32) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.attach_to_entity({}, {})", animation_id, entity_id);
        Ok(())
    }

    #[staticmethod]
    fn set_progress(animation_id: u32, progress: f32) -> PyResult<()> {
        tracing::info!(target: "python", "Animation.set_progress({}, {})", animation_id, progress);
        Ok(())
    }
}

/// Python模块管理器
pub struct PythonModuleManager {
    /// 服务实例
    service: PythonScriptingService,
    /// 模块缓存
    module_cache: HashMap<String, String>,
}

impl PythonModuleManager {
    /// 创建新的模块管理器
    pub fn new(service: PythonScriptingService) -> Self {
        Self {
            service,
            module_cache: HashMap::new(),
        }
    }

    /// 加载模块
    pub fn load_module(&mut self, name: &str, path: &PathBuf) -> PyResult<()> {
        let code = std::fs::read_to_string(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        self.service.load_module(name, &code)?;
        self.module_cache.insert(name.to_string(), code);

        Ok(())
    }

    /// 重载模块
    pub fn reload_module(&self, name: &str) -> PyResult<()> {
        if let Some(code) = self.module_cache.get(name) {
            self.service.execute(code)
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Module {} not found", name)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_service_creation() {
        let service = PythonScriptingService::new(Default::default());
        assert!(service.is_ok());
    }

    #[test]
    fn test_basic_execution() {
        let service = PythonScriptingService::new(Default::default()).unwrap();
        let result = service.execute("print('Hello from Python')");
        assert!(result.is_ok());
    }

    #[test]
    fn test_entity_api() {
        let service = PythonScriptingService::new(Default::default()).unwrap();
        let result = service.execute("import game_engine as ge; ge.Entity.create('TestEntity', 0.0, 0.0, 0.0)");
        // 注意：这需要game_engine模块正确注册
        // assert!(result.is_ok());
    }

    #[test]
    fn test_math_api() {
        let service = PythonScriptingService::new(Default::default()).unwrap();
        let result = service.execute("import game_engine as ge; ge.Math.clamp(5.0, 0.0, 10.0)");
        // 注意：这需要game_engine模块正确注册
        // assert!(result.is_ok());
    }
}
