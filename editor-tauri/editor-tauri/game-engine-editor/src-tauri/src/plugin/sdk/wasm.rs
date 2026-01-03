//! # WASM Plugin SDK
//!
//! Tools and utilities for developing WebAssembly plugins.

use crate::plugin::Result;

/// WASM plugin interface
pub struct WasmPlugin {
    // This would contain the WASM runtime and instance
    _private: (),
}

impl WasmPlugin {
    /// Create a new WASM plugin
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Load WASM module
    pub fn load_module(&self, wasm_bytes: &[u8]) -> Result<()> {
        // WASM插件支持计划中（当前使用本地插件）
        // This would use wasmtime or wasmer
        let _ = wasm_bytes;
        Ok(())
    }

    /// Call a function in the WASM module
    pub fn call_function(&self, name: &str, args: &[wasmtime::Val]) -> Result<Vec<wasmtime::Val>> {
        // 函数调用通过命令模式实现
        let _ = name;
        let _ = args;
        Err(crate::plugin::PluginError::Other(
            "WASM execution not yet implemented".to_string(),
        ))
    }
}

impl Default for WasmPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// WASM plugin builder
pub struct WasmPluginBuilder {
    name: String,
    version: String,
}

impl WasmPluginBuilder {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    pub fn build(self) -> WasmPluginMetadata {
        WasmPluginMetadata {
            name: self.name,
            version: self.version,
        }
    }
}

/// WASM plugin metadata
pub struct WasmPluginMetadata {
    pub name: String,
    pub version: String,
}

/// Template code for a WASM plugin (Wat format)
pub const WASM_PLUGIN_TEMPLATE: &str = r#"
(module
  ;; Plugin name
  (memory (export "memory") 1)

  ;; Plugin metadata
  (data (i32.const 0) "my_wasm_plugin")
  (data (i32.const 32) "0.1.0")

  ;; Export plugin name
  (func (export "name") (result i32 i32)
    (i32.const 0)
    (i32.const 14)
  )

  ;; Export plugin version
  (func (export "version") (result i32 i32)
    (i32.const 32)
    (i32.const 5)
  )

  ;; Plugin initialization
  (func (export "on_load") (result i32)
    (i32.const 0)  ;; Return 0 for success
  )

  ;; Plugin update
  (func (export "on_update") (param f32)
    ;; Update logic here
  )

  ;; Plugin cleanup
  (func (export "on_unload") (result i32)
    (i32.const 0)  ;; Return 0 for success
  )
)
"#;

/// Template for WASM plugin in Rust (for compilation to WASM)
pub const WASM_RUST_TEMPLATE: &str = r#"
use game_engine_editor::plugin::wasm::WasmPluginApi;

#[no_mangle]
pub extern "C" fn name() -> *const u8 {
    b"my_wasm_plugin\0".as_ptr()
}

#[no_mangle]
pub extern "C" fn version() -> *const u8 {
    b"0.1.0\0".as_ptr()
}

#[no_mangle]
pub extern "C" fn on_load() -> i32 {
    0 // Success
}

#[no_mangle]
pub extern "C" fn on_update(delta_time: f32) {
    // Update logic
}

#[no_mangle]
pub extern "C" fn on_unload() -> i32 {
    0 // Success
}
"#;

/// Generate Cargo.toml for WASM plugin
pub fn generate_wasm_cargo_toml(name: &str, version: &str) -> String {
    format!(
        r#"
[package]
name = "{}"
version = "{}"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
opt-level = "z"
lto = true
"#,
        name, version
    )
}

/// Wasmtime wrapper for easier WASM execution
pub struct WasmRuntime {
    engine: wasmtime::Engine,
    module: Option<wasmtime::Module>,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub fn new() -> Result<Self> {
        let engine = wasmtime::Engine::default();
        Ok(Self {
            engine,
            module: None,
        })
    }

    /// Load a WASM module
    pub fn load_module(&mut self, wasm_bytes: &[u8]) -> Result<()> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)
            .map_err(|e| crate::plugin::PluginError::LoadFailed(e.to_string()))?;
        self.module = Some(module);
        Ok(())
    }

    /// Create an instance of the loaded module
    pub fn instantiate(&self) -> Result<WasmInstance> {
        let module = self
            .module
            .as_ref()
            .ok_or_else(|| crate::plugin::PluginError::LoadFailed("No module loaded".to_string()))?;

        let mut store = wasmtime::Store::new(&self.engine, ());
        let instance = wasmtime::Instance::new(&mut store, module, &[])
            .map_err(|e| crate::plugin::PluginError::LoadFailed(e.to_string()))?;

        Ok(WasmInstance { store, instance })
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// WASM instance wrapper
pub struct WasmInstance {
    store: wasmtime::Store<()>,
    instance: wasmtime::Instance,
}

impl WasmInstance {
    /// Get a function export
    pub fn get_func(&mut self, name: &str) -> Result<wasmtime::Func> {
        let func = self
            .instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| {
                crate::plugin::PluginError::LoadFailed(format!("Function '{}' not found", name))
            })?;
        Ok(func)
    }

    /// Call a function
    pub fn call(&mut self, name: &str, args: &[wasmtime::Val]) -> Result<Vec<wasmtime::Val>> {
        let func = self.get_func(name)?;
        let mut results = vec![wasmtime::Val::null(); func.ty(&self.store).results().len()];
        func.call(&mut self.store, args, &mut results)
            .map_err(|e| crate::plugin::PluginError::Other(e.to_string()))?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_builder() {
        let metadata = WasmPluginBuilder::new("test".to_string(), "0.1.0".to_string()).build();
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "0.1.0");
    }

    #[test]
    fn test_generate_wasm_cargo_toml() {
        let toml = generate_wasm_cargo_toml("my_plugin", "0.1.0");
        assert!(toml.contains("my_plugin"));
        assert!(toml.contains("wasm-bindgen"));
    }
}
