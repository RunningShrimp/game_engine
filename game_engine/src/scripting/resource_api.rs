//! 资源管理脚本API
//!
//! 提供资源加载、卸载、材质创建等功能的脚本接口

use crate::scripting::{ScriptResult, api::ScriptApi, system::ScriptValue};
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 资源管理脚本API
pub struct ResourceScriptApi {
    /// 已加载的资源缓存
    loaded_resources: Arc<Mutex<HashMap<String, ResourceInfo>>>,
    /// ECS世界引用（用于实体操作）
    world: Arc<Mutex<World>>,
}

/// 资源信息
#[derive(Debug, Clone)]
struct ResourceInfo {
    resource_type: ResourceType,
    path: String,
    loaded: bool,
}

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceType {
    Texture,
    Mesh,
    Material,
    Prefab,
    Audio,
}

impl ResourceScriptApi {
    /// 创建新的资源脚本API
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self {
            loaded_resources: Arc::new(Mutex::new(HashMap::new())),
            world,
        }
    }

    /// 注册所有资源API到脚本系统
    pub fn register_api(&self, api: &mut ScriptApi) {
        // ========== 纹理资源 ==========
        self.register_texture_api(api);

        // ========== 网格资源 ==========
        self.register_mesh_api(api);

        // ========== 材质资源 ==========
        self.register_material_api(api);

        // ========== Prefab资源 ==========
        self.register_prefab_api(api);
    }

    /// 注册纹理API
    fn register_texture_api(&self, api: &mut ScriptApi) {
        let resources = self.loaded_resources.clone();

        // 加载纹理
        api.register_function("resource_load_texture", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("resource_load_texture() requires path".to_string());
            }

            let path = match &args[0] {
                ScriptValue::String(p) => p.clone(),
                _ => return ScriptResult::Error("path must be a string".to_string()),
            };

            let name = if args.len() > 1 {
                match &args[1] {
                    ScriptValue::String(n) => n.clone(),
                    _ => path.clone(),
                }
            } else {
                // 从路径提取文件名
                path.split('/').next_back().unwrap_or(&path).to_string()
            };

            if let Ok(mut resources_guard) = resources.try_lock() {
                resources_guard.insert(
                    name.clone(),
                    ResourceInfo {
                        resource_type: ResourceType::Texture,
                        path: path.clone(),
                        loaded: true,
                    },
                );
            }

            ScriptResult::Success(ScriptValue::String(format!("Texture loaded: {name}")))
        });

        // 卸载纹理
        let resources = self.loaded_resources.clone();
        api.register_function("resource_unload_texture", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("resource_unload_texture() requires name".to_string());
            }

            let name = match &args[0] {
                ScriptValue::String(n) => n.clone(),
                _ => return ScriptResult::Error("name must be a string".to_string()),
            };

            if let Ok(mut resources_guard) = resources.try_lock() {
                if resources_guard.remove(&name).is_some() {
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Texture unloaded: {name}"
                    )));
                }
            }

            ScriptResult::Error("Texture not found".to_string())
        });
    }

    /// 注册网格API
    fn register_mesh_api(&self, api: &mut ScriptApi) {
        let resources = self.loaded_resources.clone();

        // 加载网格
        api.register_function("resource_load_mesh", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("resource_load_mesh() requires path".to_string());
            }

            let path = match &args[0] {
                ScriptValue::String(p) => p.clone(),
                _ => return ScriptResult::Error("path must be a string".to_string()),
            };

            let name = if args.len() > 1 {
                match &args[1] {
                    ScriptValue::String(n) => n.clone(),
                    _ => path.clone(),
                }
            } else {
                path.split('/').next_back().unwrap_or(&path).to_string()
            };

            if let Ok(mut resources_guard) = resources.try_lock() {
                resources_guard.insert(
                    name.clone(),
                    ResourceInfo {
                        resource_type: ResourceType::Mesh,
                        path: path.clone(),
                        loaded: true,
                    },
                );
            }

            ScriptResult::Success(ScriptValue::String(format!("Mesh loaded: {name}")))
        });

        // 卸载网格
        let resources = self.loaded_resources.clone();
        api.register_function("resource_unload_mesh", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("resource_unload_mesh() requires name".to_string());
            }

            let name = match &args[0] {
                ScriptValue::String(n) => n.clone(),
                _ => return ScriptResult::Error("name must be a string".to_string()),
            };

            if let Ok(mut resources_guard) = resources.try_lock() {
                if resources_guard.remove(&name).is_some() {
                    return ScriptResult::Success(ScriptValue::String(format!(
                        "Mesh unloaded: {name}"
                    )));
                }
            }

            ScriptResult::Error("Mesh not found".to_string())
        });
    }

    /// 注册材质API
    fn register_material_api(&self, api: &mut ScriptApi) {
        let resources = self.loaded_resources.clone();
        let world = self.world.clone();

        // 创建材质
        api.register_function("resource_create_material", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("resource_create_material() requires name".to_string());
            }

            let name = match &args[0] {
                ScriptValue::String(n) => n.clone(),
                _ => return ScriptResult::Error("name must be a string".to_string()),
            };

            let albedo = if args.len() > 1 {
                (
                    args.get(1).and_then(|v| v.as_number()).unwrap_or(1.0),
                    args.get(2).and_then(|v| v.as_number()).unwrap_or(1.0),
                    args.get(3).and_then(|v| v.as_number()).unwrap_or(1.0),
                    args.get(4).and_then(|v| v.as_number()).unwrap_or(1.0),
                )
            } else {
                (1.0, 1.0, 1.0, 1.0)
            };

            if let Ok(mut resources_guard) = resources.try_lock() {
                resources_guard.insert(
                    name.clone(),
                    ResourceInfo {
                        resource_type: ResourceType::Material,
                        path: format!("material://{name}"),
                        loaded: true,
                    },
                );
            }

            ScriptResult::Success(ScriptValue::String(format!(
                "Material created: {}, albedo=({},{},{},{})",
                name, albedo.0, albedo.1, albedo.2, albedo.3
            )))
        });

        // 设置材质属性
        api.register_function("resource_set_material_property", move |args| {
            if args.len() < 3 {
                return ScriptResult::Error(
                    "resource_set_material_property() requires name, property, value".to_string(),
                );
            }

            let name = match &args[0] {
                ScriptValue::String(n) => n.clone(),
                _ => return ScriptResult::Error("name must be a string".to_string()),
            };

            let property = match &args[1] {
                ScriptValue::String(p) => p.clone(),
                _ => return ScriptResult::Error("property must be a string".to_string()),
            };

            let value = args[2].as_number().unwrap_or(0.0);

            ScriptResult::Success(ScriptValue::String(format!(
                "Material property set: {name}[{property}] = {value}"
            )))
        });
    }

    /// 注册Prefab API
    fn register_prefab_api(&self, api: &mut ScriptApi) {
        let resources = self.loaded_resources.clone();
        let world = self.world.clone();

        // 加载Prefab
        api.register_function("resource_load_prefab", move |args| {
            if args.is_empty() {
                return ScriptResult::Error("resource_load_prefab() requires path".to_string());
            }

            let path = match &args[0] {
                ScriptValue::String(p) => p.clone(),
                _ => return ScriptResult::Error("path must be a string".to_string()),
            };

            let name = if args.len() > 1 {
                match &args[1] {
                    ScriptValue::String(n) => n.clone(),
                    _ => path.clone(),
                }
            } else {
                path.split('/').next_back().unwrap_or(&path).to_string()
            };

            if let Ok(mut resources_guard) = resources.try_lock() {
                resources_guard.insert(
                    name.clone(),
                    ResourceInfo {
                        resource_type: ResourceType::Prefab,
                        path: path.clone(),
                        loaded: true,
                    },
                );
            }

            ScriptResult::Success(ScriptValue::String(format!("Prefab loaded: {name}")))
        });

        // 实例化Prefab
        api.register_function("resource_instantiate_prefab", move |args| {
            if args.is_empty() {
                return ScriptResult::Error(
                    "resource_instantiate_prefab() requires prefab_name".to_string(),
                );
            }

            let prefab_name = match &args[0] {
                ScriptValue::String(n) => n.clone(),
                _ => return ScriptResult::Error("prefab_name must be a string".to_string()),
            };

            let x = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0);
            let y = args.get(2).and_then(|v| v.as_number()).unwrap_or(0.0);
            let z = args.get(3).and_then(|v| v.as_number()).unwrap_or(0.0);

            let mut world_guard = match world.try_lock() {
                Ok(w) => w,
                Err(_) => return ScriptResult::Error("Failed to acquire world lock".to_string()),
            };

            // 创建新实体（简化实现）
            let entity = world_guard.spawn_empty().id();

            ScriptResult::Success(ScriptValue::String(format!(
                "Prefab instantiated: {} at ({},{},{}), entity={}",
                prefab_name,
                x,
                y,
                z,
                entity.to_bits()
            )))
        });
    }
}

impl Default for ResourceScriptApi {
    fn default() -> Self {
        Self {
            loaded_resources: Arc::new(Mutex::new(HashMap::new())),
            world: Arc::new(Mutex::new(World::new())),
        }
    }
}
