//  事件总线测试模块
// 
//  提供对事件总线系统的全面测试覆盖，包括优先级、异步分发、统计等。

use crate::domain::event_bus::*;
use crate::domain::events::DomainEvent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestEvent {
    value: u32,
    name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HighPriorityEvent {
    critical: bool,
}

impl DomainEvent for HighPriorityEvent {
    fn event_type(&self) -> &'static str {
        "HighPriorityEvent"
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

#[cfg(test)]
mod event_priority_tests {
    use super::*;

    #[test]
    fn test_event_priority_ordering() {
        assert!(EventPriority::Critical > EventPriority::High);
        assert!(EventPriority::High > EventPriority::Normal);
        assert!(EventPriority::Normal > EventPriority::Low);
    }

    #[test]
    fn test_event_priority_equality() {
        assert_eq!(EventPriority::Normal, EventPriority::Normal);
        assert_ne!(EventPriority::High, EventPriority::Low);
    }

    #[test]
    fn test_event_priority_default() {
        assert_eq!(EventPriority::default(), EventPriority::Normal);
    }

    #[test]
    fn test_event_priority_ord() {
        let mut priorities = vec![
            EventPriority::Low,
            EventPriority::Critical,
            EventPriority::Normal,
            EventPriority::High,
        ];
        
        priorities.sort();
        
        assert_eq!(priorities, vec![
            EventPriority::Low,
            EventPriority::Normal,
            EventPriority::High,
            EventPriority::Critical,
        ]);
    }
}

#[cfg(test)]
mod event_data_tests {
    use super::*;

    #[test]
    fn test_event_data_creation() {
        let event = TestEvent {
            value: 42,
            name: "Test".to_string(),
        };
        
        let event_data = EventData::new(&event, EventPriority::Normal);
        
        assert_eq!(event_data.event_type_name, "TestEvent");
        assert_eq!(event_data.priority, EventPriority::Normal);
        assert!(event_data.timestamp_ns > 0);
    }

    #[test]
    fn test_event_data_serialization() {
        let event = TestEvent {
            value: 123,
            name: "Serialization Test".to_string(),
        };
        
        let event_data = EventData::new(&event, EventPriority::High);
        
        assert!(!event_data.data.is_empty());
    }

    #[test]
    fn test_event_data_with_high_priority() {
        let event = HighPriorityEvent { critical: true };
        
        let event_data = EventData::new(&event, EventPriority::Critical);
        
        assert_eq!(event_data.priority, EventPriority::Critical);
    }

    #[test]
    fn test_event_data_timestamp_monotonic() {
        let event1 = TestEvent {
            value: 1,
            name: "First".to_string(),
        };
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let event2 = TestEvent {
            value: 2,
            name: "Second".to_string(),
        };
        
        let data1 = EventData::new(&event1, EventPriority::Normal);
        let data2 = EventData::new(&event2, EventPriority::Normal);
        
        assert!(data2.timestamp_ns > data1.timestamp_ns);
    }
}

#[cfg(test)]
mod event_bus_stats_tests {
    use super::*;

    #[test]
    fn test_event_bus_stats_default() {
        let stats = EventBusStats::default();
        
        assert_eq!(stats.total_published, 0);
        assert_eq!(stats.total_handled, 0);
        assert_eq!(stats.failed_events, 0);
        assert_eq!(stats.subscriber_count, 0);
    }

    #[test]
    fn test_event_bus_stats_clone() {
        let stats = EventBusStats {
            total_published: 100,
            total_handled: 95,
            failed_events: 5,
            subscriber_count: 10,
        };
        
        let cloned = stats.clone();
        
        assert_eq!(cloned.total_published, 100);
        assert_eq!(cloned.total_handled, 95);
        assert_eq!(cloned.failed_events, 5);
        assert_eq!(cloned.subscriber_count, 10);
    }
}

#[cfg(test)]
mod enhanced_event_bus_tests {
    use super::*;

    #[test]
    fn test_enhanced_event_bus_creation() {
        let bus = EnhancedEventBus::new();
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 0);
        assert_eq!(stats.subscriber_count, 0);
    }

    #[test]
    fn test_enhanced_event_bus_default() {
        let bus = EnhancedEventBus::default();
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 0);
    }

    #[test]
    fn test_enhanced_event_bus_publish() {
        let bus = EnhancedEventBus::new();
        
        let event = TestEvent {
            value: 42,
            name: "Test".to_string(),
        };
        
        bus.publish(event, EventPriority::Normal);
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 1);
    }

    #[test]
    fn test_enhanced_event_bus_publish_multiple() {
        let bus = EnhancedEventBus::new();
        
        for i in 0..10 {
            let event = TestEvent {
                value: i,
                name: format!("Event {}", i),
            };
            bus.publish(event, EventPriority::Normal);
        }
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 10);
    }

    #[test]
    fn test_enhanced_event_bus_publish_with_priorities() {
        let bus = EnhancedEventBus::new();
        
        bus.publish(TestEvent {
            value: 1,
            name: "Low".to_string(),
        }, EventPriority::Low);
        
        bus.publish(TestEvent {
            value: 2,
            name: "Normal".to_string(),
        }, EventPriority::Normal);
        
        bus.publish(TestEvent {
            value: 3,
            name: "High".to_string(),
        }, EventPriority::High);
        
        bus.publish(HighPriorityEvent { critical: true }, EventPriority::Critical);
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 4);
    }

    #[test]
    fn test_enhanced_event_bus_add_handler() {
        let bus = EnhancedEventBus::new();
        
        bus.add_handler();
        bus.add_handler();
        bus.add_handler();
        
        let stats = bus.get_stats();
        assert_eq!(stats.subscriber_count, 3);
    }

    #[test]
    fn test_enhanced_event_bus_with_async() {
        let (bus, mut rx) = EnhancedEventBus::with_async();
        
        let event = TestEvent {
            value: 42,
            name: "Async Test".to_string(),
        };
        
        bus.publish(event, EventPriority::Normal);
        
        let received = rx.try_recv();
        assert!(received.is_ok());
        
        let event_data = received.unwrap();
        assert_eq!(event_data.event_type_name, "TestEvent");
    }

    #[test]
    fn test_enhanced_event_bus_stats_tracking() {
        let bus = EnhancedEventBus::new();
        
        bus.publish(TestEvent {
            value: 1,
            name: "Test1".to_string(),
        }, EventPriority::Normal);
        
        bus.publish(TestEvent {
            value: 2,
            name: "Test2".to_string(),
        }, EventPriority::High);
        
        bus.add_handler();
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 2);
        assert_eq!(stats.subscriber_count, 1);
    }
}

#[cfg(test)]
mod event_queue_tests {
    use super::*;

    #[test]
    fn test_event_queue_default() {
        let queue = EventQueue::default();
        
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_event_queue_push() {
        let mut queue = EventQueue::default();
        
        let event = TestEvent {
            value: 42,
            name: "Test".to_string(),
        };
        
        queue.push(event);
        
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }

    #[test]
    fn test_event_queue_push_multiple() {
        let mut queue = EventQueue::default();
        
        for i in 0..10 {
            queue.push(TestEvent {
                value: i,
                name: format!("Event {}", i),
            });
        }
        
        assert_eq!(queue.len(), 10);
    }

    #[test]
    fn test_event_queue_push_with_priority() {
        let mut queue = EventQueue::default();
        
        queue.push_with_priority(TestEvent {
            value: 1,
            name: "Low".to_string(),
        }, EventPriority::Low);
        
        queue.push_with_priority(TestEvent {
            value: 2,
            name: "Critical".to_string(),
        }, EventPriority::Critical);
        
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_event_queue_drain() {
        let mut queue = EventQueue::default();
        
        queue.push(TestEvent {
            value: 1,
            name: "Test1".to_string(),
        });
        
        queue.push(TestEvent {
            value: 2,
            name: "Test2".to_string(),
        });
        
        let events = queue.drain();
        
        assert_eq!(events.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_event_queue_drain_empty() {
        let mut queue = EventQueue::default();
        
        let events = queue.drain();
        
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_event_queue_drain_multiple_times() {
        let mut queue = EventQueue::default();
        
        queue.push(TestEvent {
            value: 1,
            name: "Test1".to_string(),
        });
        
        let events1 = queue.drain();
        assert_eq!(events1.len(), 1);
        
        queue.push(TestEvent {
            value: 2,
            name: "Test2".to_string(),
        });
        
        let events2 = queue.drain();
        assert_eq!(events2.len(), 1);
    }
}

#[cfg(test)]
mod event_bus_resource_tests {
    use super::*;

    #[test]
    fn test_event_bus_resource_creation() {
        let bus = Arc::new(EnhancedEventBus::new());
        let resource = EventBusResource::new(bus.clone());
        
        assert_eq!(Arc::strong_count(&bus), 2);
    }

    #[test]
    fn test_event_bus_resource_clone() {
        let bus = Arc::new(EnhancedEventBus::new());
        let resource = EventBusResource::new(bus);
        
        let cloned = resource.clone();
        
        assert_eq!(Arc::strong_count(&resource.bus), 3);
    }
}

#[cfg(test)]
mod event_system_set_tests {
    use super::*;

    #[test]
    fn test_event_system_set_values() {
        assert_eq!(EventSystemSet::Publish, EventSystemSet::Publish);
        assert_eq!(EventSystemSet::Handle, EventSystemSet::Handle);
        assert_ne!(EventSystemSet::Publish, EventSystemSet::Handle);
    }

    #[test]
    fn test_event_system_set_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(EventSystemSet::Publish);
        set.insert(EventSystemSet::Handle);
        
        assert_eq!(set.len(), 2);
        
        set.insert(EventSystemSet::Publish);
        assert_eq!(set.len(), 2);
    }
}

#[cfg(test)]
mod publish_event_helper_tests {
    use super::*;

    #[test]
    fn test_publish_event_helper() {
        let mut queue = EventQueue::default();
        
        let event = TestEvent {
            value: 42,
            name: "Helper Test".to_string(),
        };
        
        publish_event(&mut queue, event);
        
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_publish_event_helper_multiple() {
        let mut queue = EventQueue::default();
        
        for i in 0..5 {
            publish_event(&mut queue, TestEvent {
                value: i,
                name: format!("Event {}", i),
            });
        }
        
        assert_eq!(queue.len(), 5);
    }
}

#[cfg(test)]
mod event_bus_integration_tests {
    use super::*;

    #[test]
    fn test_event_bus_queue_integration() {
        let bus = Arc::new(EnhancedEventBus::new());
        let mut queue = EventQueue::default();
        
        publish_event(&mut queue, TestEvent {
            value: 1,
            name: "Test1".to_string(),
        });
        
        publish_event(&mut queue, TestEvent {
            value: 2,
            name: "Test2".to_string(),
        });
        
        assert_eq!(queue.len(), 2);
        
        let events = queue.drain();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_event_bus_async_integration() {
        let (bus, mut rx) = EnhancedEventBus::with_async();
        let mut queue = EventQueue::default();
        
        queue.push_with_priority(TestEvent {
            value: 1,
            name: "Async Integration".to_string(),
        }, EventPriority::High);
        
        let events = queue.drain();
        for event in events {
            bus.publish_event_data(event);
        }
        
        let received = rx.try_recv();
        assert!(received.is_ok());
    }

    #[test]
    fn test_event_bus_stats_integration() {
        let bus = EnhancedEventBus::new();
        let mut queue = EventQueue::default();
        
        for i in 0..5 {
            queue.push(TestEvent {
                value: i,
                name: format!("Stat Event {}", i),
            });
        }
        
        let events = queue.drain();
        for event in events {
            bus.publish_event_data(event);
        }
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 5);
    }
}

#[cfg(test)]
mod event_bus_edge_cases_tests {
    use super::*;

    #[test]
    fn test_event_bus_large_number_of_events() {
        let bus = EnhancedEventBus::new();
        
        for i in 0..1000 {
            bus.publish(TestEvent {
                value: i,
                name: format!("Large Event {}", i),
            }, EventPriority::Normal);
        }
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 1000);
    }

    #[test]
    fn test_event_queue_large_number_of_events() {
        let mut queue = EventQueue::default();
        
        for i in 0..1000 {
            queue.push(TestEvent {
                value: i,
                name: format!("Queue Event {}", i),
            });
        }
        
        assert_eq!(queue.len(), 1000);
        
        let events = queue.drain();
        assert_eq!(events.len(), 1000);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_event_bus_all_priorities() {
        let bus = EnhancedEventBus::new();
        
        bus.publish(TestEvent {
            value: 1,
            name: "Low".to_string(),
        }, EventPriority::Low);
        
        bus.publish(TestEvent {
            value: 2,
            name: "Normal".to_string(),
        }, EventPriority::Normal);
        
        bus.publish(TestEvent {
            value: 3,
            name: "High".to_string(),
        }, EventPriority::High);
        
        bus.publish(HighPriorityEvent { critical: true }, EventPriority::Critical);
        
        let stats = bus.get_stats();
        assert_eq!(stats.total_published, 4);
    }

    #[test]
    fn test_event_data_empty_serialization() {
        let event = TestEvent {
            value: 0,
            name: "".to_string(),
        };
        
        let event_data = EventData::new(&event, EventPriority::Normal);
        
        assert_eq!(event_data.event_type_name, "TestEvent");
    }
}
