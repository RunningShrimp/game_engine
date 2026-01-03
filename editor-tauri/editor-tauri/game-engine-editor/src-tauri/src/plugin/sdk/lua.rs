//! # Lua Plugin SDK
//!
//! Tools and utilities for developing Lua plugins.

use crate::plugin::Result;

/// Lua plugin interface
pub struct LuaPlugin {
    // This would contain the Lua runtime and state
    _private: (),
}

impl LuaPlugin {
    /// Create a new Lua plugin
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Load Lua script
    pub fn load_script(&self, script: &str) -> Result<()> {
        // Lua插件支持计划中（当前使用Rust插件）
        // This would use mlua or rlua
        let _ = script;
        Ok(())
    }

    /// Call a Lua function
    pub fn call_function(&self, name: &str, args: Vec<mlua::Value>) -> Result<Vec<mlua::Value>> {
        // 函数调用通过命令模式实现
        let _ = name;
        let _ = args;
        Err(crate::plugin::PluginError::Other(
            "Lua execution not yet implemented".to_string(),
        ))
    }
}

impl Default for LuaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Lua plugin template
pub const LUA_MINIMAL_TEMPLATE: &str = r#"
-- Minimal Lua plugin example

local plugin = {
    name = "my_lua_plugin",
    version = "0.1.0",
    apiVersion = "0.1.0"
}

function plugin:on_load(context)
    print("Lua plugin loaded!")
    print("Config:", context.config)
end

function plugin:on_update(context, delta_time)
    -- Update logic here
end

function plugin:on_unload(context)
    print("Lua plugin unloaded!")
end

return plugin
"#;

/// Lua advanced plugin template
pub const LUA_ADVANCED_TEMPLATE: &str = r#"
-- Advanced Lua plugin example

local plugin = {
    name = "my_advanced_lua_plugin",
    version = "0.1.0",
    apiVersion = "0.1.0",
    description = "An advanced Lua plugin",
    author = "Your Name",

    -- Plugin state
    state = {
        counter = 0,
        last_update = 0
    }
}

function plugin:on_load(context)
    print("Advanced Lua plugin loaded!")

    -- Subscribe to events
    context.engine_api:add_event_listener("scene.load", function(event)
        print("Scene loaded:", event.data.path)
    end)

    -- Access resources
    local assets = context.resource_manager:list_assets()
    print("Available assets:", #assets)

    -- Register custom component
    context.engine_api:register_component({
        type = "LuaScript",
        properties = {
            script_file = {type = "string", default = ""},
            auto_start = {type = "boolean", default = true}
        }
    })
end

function plugin:on_update(context, delta_time)
    self.state.counter = self.state.counter + 1

    if self.state.counter % 60 == 0 then
        print(string.format("Plugin updated %d times", self.state.counter))
        self.state.last_update = os.time()
    end
end

function plugin:on_event(event)
    print("Received event:", event.type, event.data)
end

function plugin:on_unload(context)
    print("Advanced Lua plugin unloaded!")
    print(string.format("Total updates: %d", self.state.counter))
end

return plugin
"#;

/// Lua API bindings
pub const LUA_API_BINDINGS: &str = r#"
-- Lua Plugin API

-- Engine API
engine_api = {
    -- Get engine version
    get_version = function()
        return "0.1.0"
    end,

    -- Get active scene
    get_active_scene = function()
        -- Returns scene object
    end,

    -- Register component
    register_component = function(definition)
        -- Register custom component type
    end,

    -- Add event listener
    add_event_listener = function(event_name, handler)
        -- Subscribe to events
    end,

    -- Remove event listener
    remove_event_listener = function(event_name, handler)
        -- Unsubscribe from events
    end
}

-- Resource Manager
resource_manager = {
    -- Load asset
    load_asset = function(path)
        -- Returns asset object
    end,

    -- Save asset
    save_asset = function(path, data)
        -- Save asset data
    end,

    -- List assets
    list_assets = function()
        -- Returns array of asset info
    end,

    -- Get asset info
    get_asset_info = function(path)
        -- Returns asset metadata
    end
}

-- Utility functions
utils = {
    -- Log message
    log = function(message)
        print("[LOG]", message)
    end,

    -- Log warning
    warn = function(message)
        print("[WARN]", message)
    end,

    -- Log error
    error = function(message)
        print("[ERROR]", message)
    end,

    -- Format string
    format = function(fmt, ...)
        return string.format(fmt, ...)
    end,

    -- Get current time
    get_time = function()
        return os.time()
    end
}
"#;

/// Lua plugin builder
pub struct LuaPluginBuilder {
    name: String,
    version: String,
    script: String,
}

impl LuaPluginBuilder {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            script: String::new(),
        }
    }

    pub fn with_script(mut self, script: String) -> Self {
        self.script = script;
        self
    }

    pub fn with_minimal_script(self) -> Self {
        self.with_script(LUA_MINIMAL_TEMPLATE.to_string())
    }

    pub fn with_advanced_script(self) -> Self {
        self.with_script(LUA_ADVANCED_TEMPLATE.to_string())
    }

    pub fn build(self) -> LuaPluginDefinition {
        LuaPluginDefinition {
            name: self.name,
            version: self.version,
            script: self.script,
        }
    }
}

/// Lua plugin definition
pub struct LuaPluginDefinition {
    pub name: String,
    pub version: String,
    pub script: String,
}

/// Generate plugin manifest for Lua plugin
pub fn generate_lua_manifest(name: &str, version: &str) -> String {
    format!(
        r#"
name = "{}"
version = "{}"
type = "lua"
script = "plugin.lua"
"#,
        name, version
    )
}

/// Mlua wrapper for easier Lua execution
pub struct LuaRuntime {
    lua: mlua::Lua,
}

impl LuaRuntime {
    /// Create a new Lua runtime
    pub fn new() -> Result<Self> {
        let lua = mlua::Lua::new();
        Ok(Self { lua })
    }

    /// Load and execute a script
    pub fn load_script(&self, script: &str) -> Result<mlua::Table> {
        self.lua
            .load(script)
            .eval()
            .map_err(|e| crate::plugin::PluginError::Other(e.to_string()))
    }

    /// Call a Lua function
    pub fn call_function<'a, R, A>(&self, name: &str, args: A) -> Result<R>
    where
        R: mlua::FromLua<'a>,
        A: mlua::IntoLuaMulti<'a>,
    {
        let func: mlua::Function = self
            .lua
            .globals()
            .get(name)
            .map_err(|e| crate::plugin::PluginError::Other(e.to_string()))?;

        func.call(args)
            .map_err(|e| crate::plugin::PluginError::Other(e.to_string()))
    }

    /// Register a global value
    pub fn register_global(&self, name: &str, value: mlua::Value) -> Result<()> {
        self.lua
            .globals()
            .set(name, value)
            .map_err(|e| crate::plugin::PluginError::Other(e.to_string()))
    }

    /// Get Lua state
    pub fn state(&self) -> &mlua::Lua {
        &self.lua
    }
}

impl Default for LuaRuntime {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_builder() {
        let definition = LuaPluginBuilder::new("test".to_string(), "0.1.0".to_string())
            .with_minimal_script()
            .build();

        assert_eq!(definition.name, "test");
        assert_eq!(definition.version, "0.1.0");
        assert!(!definition.script.is_empty());
    }

    #[test]
    fn test_generate_lua_manifest() {
        let manifest = generate_lua_manifest("my_plugin", "0.1.0");
        assert!(manifest.contains("my_plugin"));
        assert!(manifest.contains("0.1.0"));
        assert!(manifest.contains("lua"));
    }

    #[test]
    fn test_lua_runtime() {
        let runtime = LuaRuntime::new().unwrap();
        let result: String = runtime
            .load_script("return 'Hello, Lua!'")
            .unwrap()
            .get("1")
            .unwrap();
        assert_eq!(result, "Hello, Lua!");
    }
}
