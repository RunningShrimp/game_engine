//! WebAssembly 脚本支持模块
//!
//! 提供完整的 WebAssembly 运行时支持，基于 wasmtime 库实现。
//!
//! ## 功能特性
//!
//! - WASM 模块加载和执行
//! - 宿主函数注册
//! - 类型安全的函数调用
//! - 内存管理
//!
//! ## 示例
//!
//! ```rust,ignore
//! use game_engine::scripting::wasm_support::{WasmRuntime, WasmValue};
//!
//! let mut runtime = WasmRuntime::new().expect("Failed to create WASM runtime");
//! runtime.load_module("game_logic", &wasm_bytes).expect("Failed to load module");
//! let result = runtime.call_function("game_logic", "update", vec![WasmValue::F32(0.016)]);
//! ```

use std::collections::HashMap;

#[cfg(feature = "wasm")]
use wasmtime::{Engine, Instance, Linker, Memory, Module as WasmtimeModule, Store, Caller, TypedFunc};

/// WASM 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmType {
    /// 32位整数
    I32,
    /// 64位整数
    I64,
    /// 32位浮点数
    F32,
    /// 64位浮点数
    F64,
}

/// WASM 值
#[derive(Debug, Clone, PartialEq)]
pub enum WasmValue {
    /// 32位整数值
    I32(i32),
    /// 64位整数值
    I64(i64),
    /// 32位浮点值
    F32(f32),
    /// 64位浮点值
    F64(f64),
}

impl WasmValue {
    /// 获取值的类型
    pub fn get_type(&self) -> WasmType {
        match self {
            WasmValue::I32(_) => WasmType::I32,
            WasmValue::I64(_) => WasmType::I64,
            WasmValue::F32(_) => WasmType::F32,
            WasmValue::F64(_) => WasmType::F64,
        }
    }

    /// 尝试获取 i32 值
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            WasmValue::I32(v) => Some(*v),
            _ => None,
        }
    }

    /// 尝试获取 i64 值
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            WasmValue::I64(v) => Some(*v),
            _ => None,
        }
    }

    /// 尝试获取 f32 值
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            WasmValue::F32(v) => Some(*v),
            _ => None,
        }
    }

    /// 尝试获取 f64 值
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            WasmValue::F64(v) => Some(*v),
            _ => None,
        }
    }
}

/// WASM 函数签名
#[derive(Debug, Clone)]
pub struct WasmFunction {
    /// 函数名称
    pub name: String,
    /// 参数类型
    pub param_types: Vec<WasmType>,
    /// 返回类型
    pub return_type: Option<WasmType>,
}

/// WASM 模块 - 表示一个已加载的 WebAssembly 模块
pub struct WasmModule {
    /// 模块名称
    name: String,
    /// 模块字节码
    bytecode: Vec<u8>,
    /// 导出的函数
    exports: HashMap<String, WasmFunction>,
    /// 模块是否已加载
    loaded: bool,
    /// wasmtime 模块（当启用 wasm feature 时）
    #[cfg(feature = "wasm")]
    module: Option<wasmtime::Module>,
    /// wasmtime 实例
    #[cfg(feature = "wasm")]
    instance: Option<wasmtime::Instance>,
}

impl WasmModule {
    /// 创建新的 WASM 模块
    pub fn new(name: impl Into<String>, bytecode: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            bytecode,
            exports: HashMap::new(),
            loaded: false,
            #[cfg(feature = "wasm")]
            module: None,
            #[cfg(feature = "wasm")]
            instance: None,
        }
    }

    /// 获取模块名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 检查模块是否已加载
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// 获取导出的函数列表
    pub fn get_exports(&self) -> Vec<&str> {
        self.exports.keys().map(|s| s.as_str()).collect()
    }

    /// 获取字节码
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// 加载模块（向后兼容）
    pub fn load(&mut self) -> Result<(), String> {
        self.loaded = true;
        Ok(())
    }

    /// 调用函数（向后兼容）
    pub fn call_function(
        &self,
        function_name: &str,
        args: Vec<WasmValue>,
    ) -> Result<Option<WasmValue>, String> {
        let _ = (function_name, args);
        if !self.loaded {
            return Err("Module not loaded".to_string());
        }
        // 占位符实现，实际调用需要通过 WasmRuntime
        Ok(None)
    }
}

/// WASM 运行时 - 管理 WebAssembly 模块的加载和执行
pub struct WasmRuntime {
    /// 已加载的模块
    modules: HashMap<String, WasmModule>,
    /// 宿主函数注册表
    host_functions: HashMap<String, Box<dyn Fn(Vec<WasmValue>) -> Result<Option<WasmValue>, String> + Send + Sync>>,
    /// wasmtime 引擎（当启用 wasm feature 时）
    #[cfg(feature = "wasm")]
    engine: wasmtime::Engine,
    /// wasmtime 存储（当启用 wasm feature 时）
    #[cfg(feature = "wasm")]
    store: wasmtime::Store<()>,
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create default WasmRuntime")
    }
}

impl WasmRuntime {
    /// 创建新的 WASM 运行时
    #[cfg(feature = "wasm")]
    pub fn new() -> Result<Self, String> {
        let engine = wasmtime::Engine::default();
        let store = wasmtime::Store::new(&engine, ());
        
        Ok(Self {
            modules: HashMap::new(),
            host_functions: HashMap::new(),
            engine,
            store,
        })
    }

    /// 创建新的 WASM 运行时（无 wasm feature 时的回退实现）
    #[cfg(not(feature = "wasm"))]
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            modules: HashMap::new(),
            host_functions: HashMap::new(),
        })
    }

    /// 加载 WASM 模块
    #[cfg(feature = "wasm")]
    pub fn load_module(&mut self, name: impl Into<String>, bytecode: &[u8]) -> Result<(), String> {
        let name = name.into();
        
        // 编译 WASM 模块
        let module = wasmtime::Module::new(&self.engine, bytecode)
            .map_err(|e| format!("Failed to compile WASM module: {}", e))?;
        
        // 创建导入（暂时使用空导入，后续可以添加宿主函数）
        let linker = wasmtime::Linker::new(&self.engine);
        
        // 实例化模块
        let instance = linker.instantiate(&mut self.store, &module)
            .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?;
        
        // 获取导出的函数
        let mut exports = HashMap::new();
        for export in module.exports() {
            if let wasmtime::ExternType::Func(func_type) = export.ty() {
                let param_types: Vec<WasmType> = func_type.params()
                    .map(|p| val_type_to_wasm_type(&p))
                    .collect();
                let return_type = func_type.results().next().map(|r| val_type_to_wasm_type(&r));
                
                exports.insert(export.name().to_string(), WasmFunction {
                    name: export.name().to_string(),
                    param_types,
                    return_type,
                });
            }
        }
        
        let mut wasm_module = WasmModule::new(name.clone(), bytecode.to_vec());
        wasm_module.loaded = true;
        wasm_module.module = Some(module);
        wasm_module.instance = Some(instance);
        wasm_module.exports = exports;
        
        self.modules.insert(name, wasm_module);
        Ok(())
    }

    /// 加载 WASM 模块（无 wasm feature 时的回退实现）
    #[cfg(not(feature = "wasm"))]
    pub fn load_module(&mut self, name: impl Into<String>, bytecode: &[u8]) -> Result<(), String> {
        let name = name.into();
        let mut module = WasmModule::new(name.clone(), bytecode.to_vec());
        module.loaded = true;
        self.modules.insert(name, module);
        Ok(())
    }

    /// 卸载 WASM 模块
    pub fn unload_module(&mut self, name: &str) -> Result<(), String> {
        self.modules.remove(name)
            .ok_or_else(|| format!("Module '{}' not found", name))?;
        Ok(())
    }

    /// 调用 WASM 函数
    #[cfg(feature = "wasm")]
    pub fn call_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        args: Vec<WasmValue>,
    ) -> Result<Option<WasmValue>, String> {
        let module = self.modules.get(module_name)
            .ok_or_else(|| format!("Module '{}' not found", module_name))?;
        
        let instance = module.instance.as_ref()
            .ok_or_else(|| format!("Module '{}' not instantiated", module_name))?;
        
        // 获取函数
        let func = instance.get_func(&mut self.store, function_name)
            .ok_or_else(|| format!("Function '{}' not found in module '{}'", function_name, module_name))?;
        
        // 转换参数
        let wasm_args: Vec<wasmtime::Val> = args.iter().map(wasm_value_to_val).collect();
        
        // 准备结果
        let func_type = func.ty(&self.store);
        let result_count = func_type.results().len();
        let mut results = vec![wasmtime::Val::I32(0); result_count];
        
        // 调用函数
        func.call(&mut self.store, &wasm_args, &mut results)
            .map_err(|e| format!("Failed to call function '{}': {}", function_name, e))?;
        
        // 返回结果
        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(val_to_wasm_value(&results[0])))
        }
    }

    /// 调用 WASM 函数（无 wasm feature 时的回退实现）
    #[cfg(not(feature = "wasm"))]
    pub fn call_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        args: Vec<WasmValue>,
    ) -> Result<Option<WasmValue>, String> {
        let _ = (module_name, function_name, args);
        Err("WebAssembly support not enabled. Enable the 'wasm' feature.".to_string())
    }

    /// 注册宿主函数
    pub fn register_host_function<F>(&mut self, module_name: &str, function_name: &str, func: F)
    where
        F: Fn(Vec<WasmValue>) -> Result<Option<WasmValue>, String> + Send + Sync + 'static,
    {
        let key = format!("{}::{}", module_name, function_name);
        self.host_functions.insert(key, Box::new(func));
    }

    /// 注册引擎 API
    pub fn register_engine_api(&mut self) {
        // 实体操作
        self.register_host_function("env", "spawn_entity", |_args| {
            // 返回新实体 ID（占位实现）
            Ok(Some(WasmValue::I32(1)))
        });

        self.register_host_function("env", "despawn_entity", |args| {
            if let Some(entity_id) = args.first().and_then(|v| v.as_i32()) {
                // 删除实体（占位实现）
                let _ = entity_id;
                Ok(None)
            } else {
                Err("despawn_entity requires an entity ID".to_string())
            }
        });

        // 组件操作
        self.register_host_function("env", "add_component", |args| {
            if args.len() >= 2 {
                // 添加组件（占位实现）
                Ok(None)
            } else {
                Err("add_component requires entity ID and component type".to_string())
            }
        });

        self.register_host_function("env", "get_component", |args| {
            if let Some(_entity_id) = args.first().and_then(|v| v.as_i32()) {
                // 获取组件（占位实现）
                Ok(None)
            } else {
                Err("get_component requires an entity ID".to_string())
            }
        });

        // 变换操作
        self.register_host_function("env", "set_position", |args| {
            if args.len() >= 4 {
                // 设置位置（占位实现）
                Ok(None)
            } else {
                Err("set_position requires entity ID and x, y, z coordinates".to_string())
            }
        });

        self.register_host_function("env", "get_position", |args| {
            if let Some(_entity_id) = args.first().and_then(|v| v.as_i32()) {
                // 获取位置（占位实现）
                Ok(Some(WasmValue::F32(0.0)))
            } else {
                Err("get_position requires an entity ID".to_string())
            }
        });

        // 输入操作
        self.register_host_function("env", "is_key_pressed", |args| {
            if let Some(_key_code) = args.first().and_then(|v| v.as_i32()) {
                // 检查按键状态（占位实现）
                Ok(Some(WasmValue::I32(0)))
            } else {
                Err("is_key_pressed requires a key code".to_string())
            }
        });

        self.register_host_function("env", "get_mouse_position", |_args| {
            // 获取鼠标位置（占位实现）
            Ok(Some(WasmValue::F32(0.0)))
        });

        // 音频操作
        self.register_host_function("env", "play_sound", |args| {
            if let Some(_sound_id) = args.first().and_then(|v| v.as_i32()) {
                // 播放声音（占位实现）
                Ok(None)
            } else {
                Err("play_sound requires a sound ID".to_string())
            }
        });

        self.register_host_function("env", "stop_sound", |args| {
            if let Some(_sound_id) = args.first().and_then(|v| v.as_i32()) {
                // 停止声音（占位实现）
                Ok(None)
            } else {
                Err("stop_sound requires a sound ID".to_string())
            }
        });

        // 时间操作
        self.register_host_function("env", "get_delta_time", |_args| {
            // 获取帧时间差（占位实现）
            Ok(Some(WasmValue::F32(0.016)))
        });

        self.register_host_function("env", "get_time", |_args| {
            // 获取游戏时间（占位实现）
            Ok(Some(WasmValue::F64(0.0)))
        });

        // 日志操作
        self.register_host_function("env", "log_info", |_args| {
            // 日志输出（占位实现）
            Ok(None)
        });

        self.register_host_function("env", "log_error", |_args| {
            // 错误日志输出（占位实现）
            Ok(None)
        });
    }

    /// 获取已加载的模块列表
    pub fn get_loaded_modules(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }

    /// 检查模块是否已加载
    pub fn is_module_loaded(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    /// 获取模块的导出函数
    pub fn get_module_exports(&self, name: &str) -> Option<Vec<&str>> {
        self.modules.get(name).map(|m| m.get_exports())
    }
}

/// 将 wasmtime ValType 转换为 WasmType
#[cfg(feature = "wasm")]
fn val_type_to_wasm_type(val_type: &wasmtime::ValType) -> WasmType {
    match val_type {
        wasmtime::ValType::I32 => WasmType::I32,
        wasmtime::ValType::I64 => WasmType::I64,
        wasmtime::ValType::F32 => WasmType::F32,
        wasmtime::ValType::F64 => WasmType::F64,
        _ => WasmType::I32, // 默认回退到 I32
    }
}

/// 将 WasmValue 转换为 wasmtime Val
#[cfg(feature = "wasm")]
fn wasm_value_to_val(value: &WasmValue) -> wasmtime::Val {
    match value {
        WasmValue::I32(v) => wasmtime::Val::I32(*v),
        WasmValue::I64(v) => wasmtime::Val::I64(*v),
        WasmValue::F32(v) => wasmtime::Val::F32(v.to_bits()),
        WasmValue::F64(v) => wasmtime::Val::F64(v.to_bits()),
    }
}

/// 将 wasmtime Val 转换为 WasmValue
#[cfg(feature = "wasm")]
fn val_to_wasm_value(val: &wasmtime::Val) -> WasmValue {
    match val {
        wasmtime::Val::I32(v) => WasmValue::I32(*v),
        wasmtime::Val::I64(v) => WasmValue::I64(*v),
        wasmtime::Val::F32(v) => WasmValue::F32(f32::from_bits(*v)),
        wasmtime::Val::F64(v) => WasmValue::F64(f64::from_bits(*v)),
        _ => WasmValue::I32(0), // 默认回退
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_value_type() {
        assert_eq!(WasmValue::I32(42).get_type(), WasmType::I32);
        assert_eq!(WasmValue::I64(42).get_type(), WasmType::I64);
        assert_eq!(WasmValue::F32(3.14).get_type(), WasmType::F32);
        assert_eq!(WasmValue::F64(3.14159).get_type(), WasmType::F64);
    }

    #[test]
    fn test_wasm_value_conversion() {
        assert_eq!(WasmValue::I32(42).as_i32(), Some(42));
        assert_eq!(WasmValue::I64(42).as_i64(), Some(42));
        assert_eq!(WasmValue::F32(3.14).as_f32(), Some(3.14));
        assert_eq!(WasmValue::F64(3.14159).as_f64(), Some(3.14159));
        
        assert_eq!(WasmValue::I32(42).as_i64(), None);
        assert_eq!(WasmValue::I64(42).as_i32(), None);
    }

    #[test]
    fn test_wasm_module_creation() {
        let module = WasmModule::new("test_module", vec![0, 1, 2, 3]);
        assert_eq!(module.name(), "test_module");
        assert!(!module.is_loaded());
        assert!(module.get_exports().is_empty());
    }

    #[test]
    fn test_wasm_runtime_creation() {
        let runtime = WasmRuntime::new().expect("Failed to create WASM runtime");
        assert!(runtime.get_loaded_modules().is_empty());
    }

    #[test]
    fn test_wasm_runtime_register_api() {
        let mut runtime = WasmRuntime::new().expect("Failed to create WASM runtime");
        runtime.register_engine_api();
        // 验证宿主函数已注册
        assert!(!runtime.host_functions.is_empty());
    }

    #[test]
    fn test_wasm_runtime_load_unload() {
        let mut runtime = WasmRuntime::new().expect("Failed to create WASM runtime");
        
        // 使用空字节码测试基本加载
        let result = runtime.load_module("empty", &[]);
        // 空字节码应该失败（因为不是有效的 WASM）
        #[cfg(feature = "wasm")]
        assert!(result.is_err());
        
        #[cfg(not(feature = "wasm"))]
        {
            assert!(result.is_ok());
            assert!(runtime.is_module_loaded("empty"));
            assert!(runtime.unload_module("empty").is_ok());
            assert!(!runtime.is_module_loaded("empty"));
        }
    }
}
