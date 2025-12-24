//! 事件类型注册表
//!
//! 提供事件类型的注册、序列化/反序列化和验证功能。
//!
//! ## 设计原则
//!
//! 1. **类型安全**：确保事件类型正确注册和验证
//! 2. **序列化支持**：支持完整的事件序列化/反序列化
//! 3. **版本兼容**：支持事件类型的版本管理
//! 4. **性能优化**：使用HashMap快速查找
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::domain::event_registry::EventRegistry;
//! use game_engine::domain::events::DomainEvent;
//!
//! // 注册事件类型
//! let mut registry = EventRegistry::new();
//! registry.register::<SceneLoadedEvent>();
//!
//! // 序列化事件
//! let event = SceneLoadedEvent { scene_id: "scene1".to_string() };
//! let serialized = registry.serialize(&event)?;
//!
//! // 反序列化事件
//! let deserialized: Box<dyn DomainEvent> = registry.deserialize("SceneLoadedEvent", &serialized)?;
//! ```

use crate::domain::events::{DomainEvent, EventError};
use crate::error::{safe_read, safe_write};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing;

/// 事件类型信息
#[derive(Debug, Clone)]
pub struct EventTypeInfo {
    /// 事件类型名称
    pub name: &'static str,
    /// 类型ID
    pub type_id: TypeId,
    /// 版本号
    pub version: u32,
    /// 是否已弃用
    pub deprecated: bool,
}

/// 事件反序列化器trait
trait EventDeserializer: Send + Sync {
    /// 从字节数据反序列化事件
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn DomainEvent>, EventError>;
    
    /// 获取事件类型信息
    fn type_info(&self) -> EventTypeInfo;
}

/// 类型化的事件反序列化器
struct TypedEventDeserializer<E: DomainEvent + Serialize + for<'de> Deserialize<'de> + 'static> {
    type_info: EventTypeInfo,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: DomainEvent + Serialize + for<'de> Deserialize<'de> + 'static> TypedEventDeserializer<E> {
    fn new(name: &'static str, version: u32) -> Self {
        Self {
            type_info: EventTypeInfo {
                name,
                type_id: TypeId::of::<E>(),
                version,
                deprecated: false,
            },
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E: DomainEvent + Serialize + for<'de> Deserialize<'de> + 'static> EventDeserializer for TypedEventDeserializer<E> {
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn DomainEvent>, EventError> {
        let event: E = bincode::deserialize(data)
            .map_err(|e| EventError::SerializationError(format!("Failed to deserialize {}: {}", self.type_info.name, e)))?;
        Ok(Box::new(event))
    }
    
    fn type_info(&self) -> EventTypeInfo {
        self.type_info.clone()
    }
}

/// 事件类型注册表
///
/// 管理所有已注册的事件类型，支持序列化/反序列化和类型验证
pub struct EventRegistry {
    /// 事件类型名称 -> 反序列化器映射
    deserializers: Arc<RwLock<HashMap<String, Box<dyn EventDeserializer>>>>,
    /// 类型ID -> 事件类型名称映射（用于快速查找）
    type_id_to_name: Arc<RwLock<HashMap<TypeId, String>>>,
    /// 事件类型名称 -> 版本映射（用于版本管理）
    versions: Arc<RwLock<HashMap<String, u32>>>,
}

impl std::fmt::Debug for EventRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let registered_types: Vec<String> = if let Ok(deserializers) = self.deserializers.read() {
            deserializers.keys().cloned().collect()
        } else {
            Vec::new()
        };
        
        let type_count = registered_types.len();
        
        let versions: HashMap<String, u32> = if let Ok(versions_guard) = self.versions.read() {
            versions_guard.clone()
        } else {
            HashMap::new()
        };
        
        f.debug_struct("EventRegistry")
            .field("registered_types", &registered_types)
            .field("type_count", &type_count)
            .field("versions", &versions)
            .finish()
    }
}

impl EventRegistry {
    /// 创建新的事件类型注册表
    pub fn new() -> Self {
        Self {
            deserializers: Arc::new(RwLock::new(HashMap::new())),
            type_id_to_name: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册事件类型
    ///
    /// # 参数
    /// - `name`: 事件类型名称（必须与`event_type()`返回的值一致）
    /// - `version`: 事件类型版本号
    ///
    /// # 示例
    /// ```rust
    /// registry.register::<SceneLoadedEvent>("SceneLoadedEvent", 1);
    /// ```
    pub fn register<E: DomainEvent + Serialize + for<'de> Deserialize<'de> + 'static>(
        &self,
        name: &'static str,
        version: u32,
    ) -> Result<(), EventError> {
        // 注意：名称验证在运行时进行（通过serialize/deserialize）
        // 如果名称不匹配，会在使用时发现并报告错误
        
        let type_id = TypeId::of::<E>();
        let deserializer: Box<dyn EventDeserializer> = Box::new(TypedEventDeserializer::<E>::new(name, version));

        // 写入映射
        let mut deserializers = safe_write(&self.deserializers, "deserializers")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;
        let mut type_id_map = safe_write(&self.type_id_to_name, "type_id_to_name")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;
        let mut versions = safe_write(&self.versions, "versions")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;

        deserializers.insert(name.to_string(), deserializer);
        type_id_map.insert(type_id, name.to_string());
        versions.insert(name.to_string(), version);

        tracing::debug!(
            target: "domain",
            "Registered event type: {} (version: {}, type_id: {:?})",
            name,
            version,
            type_id
        );

        Ok(())
    }

    /// 序列化事件
    ///
    /// # 参数
    /// - `event`: 要序列化的事件
    ///
    /// # 返回
    /// 序列化后的字节数据
    ///
    /// # 注意
    /// 此方法会验证事件类型是否已注册，并验证事件类型名称是否匹配
    pub fn serialize<E: DomainEvent + Serialize>(&self, event: &E) -> Result<Vec<u8>, EventError> {
        // 验证事件类型是否已注册
        let event_type = event.event_type();
        let type_id = TypeId::of::<E>();
        
        let type_id_map = safe_read(&self.type_id_to_name, "type_id_to_name")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;
        
        // 检查类型ID是否已注册
        if let Some(registered_name) = type_id_map.get(&type_id) {
            // 验证事件类型名称是否匹配
            if registered_name != event_type {
                return Err(EventError::SerializationError(format!(
                    "Event type name mismatch: event_type() returns '{}', but registered as '{}'",
                    event_type,
                    registered_name
                )));
            }
        } else {
            return Err(EventError::UnknownEventType(format!(
                "Event type '{}' (type_id: {:?}) is not registered",
                event_type,
                type_id
            )));
        }

        // 序列化事件
        bincode::serialize(event)
            .map_err(|e| EventError::SerializationError(format!("Failed to serialize {}: {}", event_type, e)))
    }

    /// 反序列化事件
    ///
    /// # 参数
    /// - `event_type`: 事件类型名称
    /// - `data`: 序列化的事件数据
    ///
    /// # 返回
    /// 反序列化后的事件（trait object）
    pub fn deserialize(&self, event_type: &str, data: &[u8]) -> Result<Box<dyn DomainEvent>, EventError> {
        let deserializers = safe_read(&self.deserializers, "deserializers")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;

        let deserializer = deserializers
            .get(event_type)
            .ok_or_else(|| EventError::UnknownEventType(format!("Event type '{}' is not registered", event_type)))?;

        deserializer.deserialize(data)
    }

    /// 检查事件类型是否已注册
    pub fn is_registered(&self, event_type: &str) -> bool {
        let deserializers = match safe_read(&self.deserializers, "deserializers") {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        deserializers.contains_key(event_type)
    }

    /// 获取事件类型信息
    pub fn get_type_info(&self, event_type: &str) -> Option<EventTypeInfo> {
        let deserializers = safe_read(&self.deserializers, "deserializers").ok()?;
        let deserializer = deserializers.get(event_type)?;
        Some(deserializer.type_info())
    }

    /// 获取事件类型版本
    pub fn get_version(&self, event_type: &str) -> Option<u32> {
        let versions = safe_read(&self.versions, "versions").ok()?;
        versions.get(event_type).copied()
    }

    /// 获取所有已注册的事件类型
    pub fn registered_types(&self) -> Vec<String> {
        let deserializers = match safe_read(&self.deserializers, "deserializers") {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        deserializers.keys().cloned().collect()
    }

    /// 取消注册事件类型
    pub fn unregister(&self, event_type: &str) -> Result<(), EventError> {
        let mut deserializers = safe_write(&self.deserializers, "deserializers")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;
        let mut type_id_map = safe_write(&self.type_id_to_name, "type_id_to_name")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;
        let mut versions = safe_write(&self.versions, "versions")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;

        // 获取类型ID以便从type_id_map中删除
        if let Some(deserializer) = deserializers.get(event_type) {
            let type_info = deserializer.type_info();
            type_id_map.remove(&type_info.type_id);
        }

        deserializers.remove(event_type);
        versions.remove(event_type);

        tracing::debug!(target: "domain", "Unregistered event type: {}", event_type);

        Ok(())
    }

    /// 验证事件类型名称
    ///
    /// 检查事件类型名称是否与已注册的类型匹配
    pub fn validate_event_type<E: DomainEvent>(&self, event_type: &str) -> Result<(), EventError> {
        let type_id = TypeId::of::<E>();
        let type_id_map = safe_read(&self.type_id_to_name, "type_id_to_name")
            .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;

        if let Some(registered_name) = type_id_map.get(&type_id) {
            if registered_name != event_type {
                return Err(EventError::SerializationError(format!(
                    "Event type name mismatch: expected '{}', but registered as '{}'",
                    event_type,
                    registered_name
                )));
            }
        } else {
            return Err(EventError::UnknownEventType(format!(
                "Event type '{}' (type_id: {:?}) is not registered",
                event_type,
                type_id
            )));
        }

        Ok(())
    }
}

impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局事件类型注册表（单例）
///
/// 使用`std::sync::Once`确保线程安全的初始化
static GLOBAL_REGISTRY: std::sync::OnceLock<Arc<RwLock<EventRegistry>>> = std::sync::OnceLock::new();

/// 获取全局事件类型注册表
pub fn global_registry() -> Arc<RwLock<EventRegistry>> {
    GLOBAL_REGISTRY.get_or_init(|| Arc::new(RwLock::new(EventRegistry::new()))).clone()
}

/// 便捷函数：注册事件类型到全局注册表
pub fn register_event_type<E: DomainEvent + Serialize + for<'de> Deserialize<'de> + 'static>(
    name: &'static str,
    version: u32,
) -> Result<(), EventError> {
    let registry = global_registry();
    let registry_guard = registry.write()
        .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;
    registry_guard.register::<E>(name, version)
}

/// 便捷函数：从全局注册表反序列化事件
pub fn deserialize_event(event_type: &str, data: &[u8]) -> Result<Box<dyn DomainEvent>, EventError> {
    let registry = global_registry();
    let registry_guard = registry.read()
        .map_err(|e| EventError::SerializationError(format!("Failed to acquire lock: {}", e)))?;
    registry_guard.deserialize(event_type, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::DomainEvent;
    use bevy_ecs::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        value: u32,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn apply(&self, _world: &mut World) -> Result<(), EventError> {
            Ok(())
        }

        fn revert(&self, _world: &mut World) -> Result<(), EventError> {
            Ok(())
        }
    }

    impl Default for TestEvent {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    #[test]
    fn test_event_registry() {
        let registry = EventRegistry::new();

        // 注册事件类型
        registry.register::<TestEvent>("TestEvent", 1).unwrap();

        // 检查是否已注册
        assert!(registry.is_registered("TestEvent"));

        // 序列化和反序列化
        let event = TestEvent { value: 42 };
        let serialized = registry.serialize(&event).unwrap();
        let deserialized = registry.deserialize("TestEvent", &serialized).unwrap();

        // 验证反序列化结果
        assert_eq!(deserialized.event_type(), "TestEvent");
    }

    #[test]
    fn test_event_registry_validation() {
        let registry = EventRegistry::new();

        // 注册事件类型
        registry.register::<TestEvent>("TestEvent", 1).unwrap();

        // 验证正确的事件类型
        let event = TestEvent { value: 42 };
        registry.validate_event_type::<TestEvent>("TestEvent").unwrap();

        // 验证错误的事件类型名称应该失败
        assert!(registry.validate_event_type::<TestEvent>("WrongEvent").is_err());
    }

    #[test]
    fn test_global_registry() {
        // 注册到全局注册表
        register_event_type::<TestEvent>("TestEvent", 1).unwrap();

        // 从全局注册表反序列化
        let event = TestEvent { value: 42 };
        let serialized = bincode::serialize(&event).unwrap();
        let deserialized = deserialize_event("TestEvent", &serialized).unwrap();

        assert_eq!(deserialized.event_type(), "TestEvent");
    }
}

