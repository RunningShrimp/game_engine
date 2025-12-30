//  事件类型注册系统

use super::{DomainEvent, EventError};
use crate::serialization::compat::bincode_compat;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
// bincode 2.0 uses encode/decode API

/// 事件工厂 trait
pub trait EventFactory: Send + Sync + 'static {
    /// 创建事件实例
    fn create_event(&self, data: &[u8]) -> Result<Box<dyn DomainEvent>, EventError>;

    /// 获取事件类型名称
    fn event_type_name(&self) -> &'static str;
}

/// 泛型事件工厂实现
struct GenericEventFactory<E: DomainEvent + Serialize + for<'de> Deserialize<'de>> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E: DomainEvent + Serialize + for<'de> Deserialize<'de>> GenericEventFactory<E> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E: DomainEvent + Serialize + for<'de> Deserialize<'de> + std::default::Default> EventFactory
    for GenericEventFactory<E>
{
    fn create_event(&self, data: &[u8]) -> Result<Box<dyn DomainEvent>, EventError> {
        bincode_compat::deserialize::<E>(data)
            .map(|event| Box::new(event) as Box<dyn DomainEvent>)
            .map_err(|e| EventError::SerializationError(e.to_string()))
    }

    fn event_type_name(&self) -> &'static str {
        E::event_type(&E::default())
    }
}

impl std::fmt::Debug for EventTypeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventTypeRegistry")
            .field(
                "registered_types",
                &self.type_name_to_factory.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

/// 事件类型注册表
#[derive(Default)]
pub struct EventTypeRegistry {
    /// 事件类型名称 -> 事件工厂
    type_name_to_factory: HashMap<String, Arc<dyn EventFactory>>,
    /// 类型ID -> 事件类型名称
    type_id_to_name: HashMap<TypeId, String>,
}

impl EventTypeRegistry {
    /// 创建新的事件类型注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册事件类型
    pub fn register_event_type<
        E: DomainEvent + Serialize + for<'de> Deserialize<'de> + Default + Serialize,
    >(
        &mut self,
    ) -> Result<(), EventError> {
        let event_factory = Arc::new(GenericEventFactory::<E>::new());
        let event_type_name = E::event_type(&E::default());
        let type_id = TypeId::of::<E>();

        // 检查是否已经注册过该类型
        if self.type_name_to_factory.contains_key(event_type_name) {
            return Ok(()); // 已注册，忽略
        }

        self.type_name_to_factory.insert(event_type_name.to_string(), event_factory);
        self.type_id_to_name.insert(type_id, event_type_name.to_string());

        Ok(())
    }

    /// 根据事件类型名称创建事件实例
    pub fn create_event(
        &self,
        event_type: &str,
        data: &[u8],
    ) -> Result<Box<dyn DomainEvent>, EventError> {
        if let Some(factory) = self.type_name_to_factory.get(event_type) {
            factory.create_event(data)
        } else {
            Err(EventError::SerializationError(format!(
                "Unknown event type: {event_type}"
            )))
        }
    }

    /// 根据事件类型ID获取事件类型名称
    pub fn get_event_type_name(&self, type_id: TypeId) -> Option<&str> {
        self.type_id_to_name.get(&type_id).map(|s| s.as_str())
    }
}

/// 自动注册事件类型的宏
#[macro_export]
macro_rules! register_event {
    ($event_type:ty) => {
        // 为事件类型实现Default trait
        impl Default for $event_type {
            fn default() -> Self {
                // 为事件类型提供默认实现
                // 注意：如果事件类型没有合理的默认值，应该手动实现此trait
                // 这里返回一个假设有Default字段的结构体
                // 实际使用时，应该为具体事件类型提供正确的Default实现
                compile_error!(
                    "Event type must implement Default trait manually. \
                     Use #[derive(Default)] if appropriate, or provide custom implementation.\n\
                     Example: impl Default for MyEvent {{ fn default() -> Self {{ ... }} }}"
                );
            }
        }

        // 注册事件类型到全局注册表
        struct EventRegister;

        impl EventRegister {
            #[ctor::ctor]
            fn register() {
                let registry = $crate::core::event_sourcing::EventTypeRegistry::new();
                registry.register_event_type::<$event_type>().unwrap_or(());
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试事件类型
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        data: String,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn apply(&self, _world: &mut World) -> Result<(), EventError> {
            Ok(())
        }

        fn revert(&self, _world: &mut World) -> Result<(), EventError> {
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // 手动实现Default
    impl Default for TestEvent {
        fn default() -> Self {
            Self {
                data: "default".to_string(),
            }
        }
    }

    #[test]
    fn test_event_registry() {
        let mut registry = EventTypeRegistry::new();
        registry
            .register_event_type::<TestEvent>()
            .expect("Failed to register TestEvent type");

        // 测试创建事件
        let test_data = TestEvent {
            data: "test".to_string(),
        };
        let serialized = bincode_compat::serialize(&test_data)
            .map_err(|e| Box::new(e))
            .expect("Failed to serialize TestEvent data");

        let created_event = registry
            .create_event("TestEvent", &serialized)
            .expect("Failed to create TestEvent from registry");
        assert_eq!(created_event.event_type(), "TestEvent");
    }

    #[test]
    #[should_panic(expected = "Unknown event type")]
    fn test_unknown_event_type() {
        let registry = EventTypeRegistry::new();
        registry
            .create_event("UnknownEvent", &[])
            .expect("Should panic with unknown event type error");
    }
}
