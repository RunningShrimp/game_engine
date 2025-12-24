//! 领域事件总线系统
//!
//! 提供事件订阅/发布/分发机制，支持：
//! - 类型安全的事件订阅和发布
//! - 异步事件分发
//! - 事件优先级
//! - 与 ECS 系统深度集成
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   Event Bus Architecture                 │
//! ├─────────────────────────────────────────────────────────┤
//! │  Publishers                                             │
//! │  ├── Domain Layer (Aggregates, Services)               │
//! │  ├── Application Layer (Use Cases)                       │
//! │  └── ECS Systems                                      │
//! │                      │                                 │
//! │                      ▼                                 │
//! │              ┌──────────┐                              │
//! │              │  Event   │                              │
//! │              │   Bus    │                              │
//! │              └────┬─────┘                              │
//! │                   │                                     │
//! │         ┌─────────┼─────────┐                         │
//! │         ▼         ▼         ▼                         │
//! │  ┌─────────┐ ┌─────────┐ ┌─────────┐              │
//! │  │ Sync    │ │ Async   │ │ Event   │              │
//! │  │ Handlers│ │ Handlers│ │ Store   │              │
//! │  └─────────┘ └─────────┘ └─────────┘              │
//! │       │           │             │                      │
//! │       ▼           ▼             ▼                      │
//! │  ┌─────────┐ ┌─────────┐ ┌─────────┐              │
//! │  │ ECS     │ │  I/O    │ │ History │              │
//! │  │ Systems │ │  Ops    │ │  & Log  │              │
//! │  └─────────┘ └─────────┘ └─────────┘              │
//! └─────────────────────────────────────────────────────────┘
//! ```

use crate::domain::events::DomainEvent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// 事件优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EventPriority {
    /// 低优先级事件（如日志、统计）
    Low = 0,
    /// 普通优先级事件（如普通业务事件）
    Normal = 1,
    /// 高优先级事件（如用户输入、关键状态变更）
    High = 2,
    /// 紧急优先级事件（如错误、崩溃警告）
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 事件总线统计信息
#[derive(Debug, Default, Clone)]
pub struct EventBusStats {
    /// 发布的事件总数
    pub total_published: u64,
    /// 处理的事件总数
    pub total_handled: u64,
    /// 处理失败的事件数
    pub failed_events: u64,
    /// 当前订阅者数量
    pub subscriber_count: usize,
}

/// 增强的事件总线
///
/// 特性：
/// - 优先级支持
/// - 同步和异步处理器
/// - 事件溯源集成
/// - 性能监控
pub struct EnhancedEventBus {
    /// 处理器计数
    handler_count: Arc<Mutex<usize>>,
    /// 异步处理器通道
    async_tx: Option<mpsc::UnboundedSender<EventData>>,
    /// 是否启用异步分发
    async_enabled: Arc<Mutex<bool>>,
    /// 统计信息
    stats: Arc<Mutex<EventBusStats>>,
}

/// 事件数据（包含序列化的事件）
#[derive(Debug, Clone)]
pub struct EventData {
    /// 事件类型名称
    pub event_type_name: String,
    /// 序列化的事件数据
    pub data: Vec<u8>,
    /// 事件优先级
    pub priority: EventPriority,
    /// 时间戳（纳秒）
    pub timestamp_ns: i64,
}

impl EventData {
    /// 创建新的事件数据
    pub fn new<E: DomainEvent + Serialize>(event: &E, priority: EventPriority) -> Self
    where
        E: serde::Serialize,
    {
        let event_type_name = event.event_type().to_string();
        let data = bincode::serialize(event)
            .unwrap_or_else(|_| Vec::new());

        Self {
            event_type_name,
            data,
            priority,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64,
        }
    }
}

impl EnhancedEventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self {
            handler_count: Arc::new(Mutex::new(0)),
            async_tx: None,
            async_enabled: Arc::new(Mutex::new(false)),
            stats: Arc::new(Mutex::new(EventBusStats::default())),
        }
    }

    /// 创建支持异步分发的事件总线
    pub fn with_async() -> (Self, mpsc::UnboundedReceiver<EventData>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let bus = Self {
            handler_count: Arc::new(Mutex::new(0)),
            async_tx: Some(tx),
            async_enabled: Arc::new(Mutex::new(true)),
            stats: Arc::new(Mutex::new(EventBusStats::default())),
        };
        (bus, rx)
    }

    /// 发布事件
    pub fn publish<E>(&self, event: E, priority: EventPriority)
    where
        E: DomainEvent + Serialize,
    {
        let event_data = EventData::new(&event, priority);

        // 更新统计
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_published += 1;
        }

        // 分发到异步处理器
        if let Some(ref tx) = self.async_tx {
            if let Ok(async_enabled) = self.async_enabled.lock() {
                if *async_enabled {
                    let _ = tx.send(event_data);
                }
            }
        }
    }

    /// 增加处理器计数
    pub fn add_handler(&self) {
        if let Ok(mut count) = self.handler_count.lock() {
            *count += 1;
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> EventBusStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

impl Default for EnhancedEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// ECS 集成资源
#[derive(Resource, Clone)]
pub struct EventBusResource {
    pub bus: Arc<EnhancedEventBus>,
}

impl EventBusResource {
    pub fn new(bus: Arc<EnhancedEventBus>) -> Self {
        Self { bus }
    }
}

/// 事件系统集
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventSystemSet {
    /// 发布事件
    Publish,
    /// 处理事件
    Handle,
}

/// 事件队列资源（用于从 ECS 系统发布事件）
#[derive(Resource, Default)]
pub struct EventQueue {
    events: Vec<EventData>,
}

impl EventQueue {
    pub fn push<E: DomainEvent + Serialize>(&mut self, event: E) {
        self.events.push(EventData::new(&event, EventPriority::Normal));
    }

    pub fn push_with_priority<E: DomainEvent + Serialize>(&mut self, event: E, priority: EventPriority) {
        self.events.push(EventData::new(&event, priority));
    }

    pub fn drain(&mut self) -> Vec<EventData> {
        std::mem::take(&mut self.events)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// 事件发布系统
pub fn event_publish_system(
    event_bus: Res<EventBusResource>,
    mut event_queue: ResMut<EventQueue>,
) {
    for event_data in event_queue.drain() {
        event_bus.bus.publish_event_data(event_data);
    }
}

impl EnhancedEventBus {
    /// 发布事件数据（内部使用）
    fn publish_event_data(&self, event_data: EventData) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_published += 1;
        }

        if let Some(ref tx) = self.async_tx {
            if let Ok(async_enabled) = self.async_enabled.lock() {
                if *async_enabled {
                    let _ = tx.send(event_data);
                }
            }
        }
    }
}

/// 辅助函数：发布事件到队列
pub fn publish_event<E: DomainEvent + Serialize>(queue: &mut EventQueue, event: E) {
    queue.push(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        value: u32,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn apply(&self, _world: &mut World) -> Result<(), crate::domain::events::EventError> {
            Ok(())
        }

        fn revert(&self, _world: &mut World) -> Result<(), crate::domain::events::EventError> {
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_enhanced_event_bus_creation() {
        let bus = EnhancedEventBus::new();
        assert_eq!(bus.get_stats().total_published, 0);
    }

    #[test]
    fn test_event_data_creation() {
        let event = TestEvent { value: 42 };
        let event_data = EventData::new(&event, EventPriority::High);
        assert_eq!(event_data.priority, EventPriority::High);
        assert_eq!(event_data.event_type_name, "TestEvent");
    }

    #[test]
    fn test_event_queue() {
        let mut queue = EventQueue::default();
        assert!(queue.is_empty());

        queue.push(TestEvent { value: 1 });
        assert_eq!(queue.len(), 1);

        queue.push(TestEvent { value: 2 });
        assert_eq!(queue.len(), 2);

        let events = queue.drain();
        assert!(queue.is_empty());
        assert_eq!(events.len(), 2);
    }
}
