//! # Event System
//!
//! Event bus for plugin communication and coordination.

use crate::plugin::api::PluginEvent;
use crate::plugin::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Event bus for pub/sub communication
pub struct EventBus {
    sender: broadcast::Sender<PluginEvent>,
    subscribers: Arc<RwLock<Vec<EventSubscriber>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            sender,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Publish an event
    pub async fn publish(&self, event: PluginEvent) {
        let _ = self.sender.send(event.clone());

        // Notify subscribers
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            subscriber.send(event.clone());
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> EventSubscriber {
        let receiver = self.sender.subscribe();
        EventSubscriber::new(receiver)
    }

    /// Subscribe to specific event types
    pub fn subscribe_filtered<F>(&self, filter: F) -> FilteredEventSubscriber
    where
        F: Fn(&PluginEvent) -> bool + Send + Sync + 'static,
    {
        let receiver = self.sender.subscribe();
        FilteredEventSubscriber::new(receiver, filter)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event subscriber
#[derive(Clone)]
pub struct EventSubscriber {
    receiver: broadcast::Receiver<PluginEvent>,
}

impl EventSubscriber {
    fn new(receiver: broadcast::Receiver<PluginEvent>) -> Self {
        Self { receiver }
    }

    /// Receive the next event
    pub async fn recv(&mut self) -> Result<PluginEvent> {
        self.receiver
            .recv()
            .await
            .map_err(|e| crate::plugin::PluginError::EventError(e.to_string()))
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Result<PluginEvent> {
        self.receiver
            .try_recv()
            .map_err(|e| crate::plugin::PluginError::EventError(e.to_string()))
    }

    /// Send an event directly (for internal use)
    fn send(&self, event: PluginEvent) {
        let _ = self.sender.send(event);
    }

    /// Get receiver count
    pub fn receiver_count(&self) -> usize {
        self.receiver.receiver_count()
    }
}

/// Filtered event subscriber
pub struct FilteredEventSubscriber {
    receiver: broadcast::Receiver<PluginEvent>,
    filter: Box<dyn Fn(&PluginEvent) -> bool + Send + Sync>,
}

impl FilteredEventSubscriber {
    fn new(
        receiver: broadcast::Receiver<PluginEvent>,
        filter: impl Fn(&PluginEvent) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            receiver,
            filter: Box::new(filter),
        }
    }

    /// Receive the next event that matches the filter
    pub async fn recv(&mut self) -> Result<PluginEvent> {
        loop {
            let event = self
                .receiver
                .recv()
                .await
                .map_err(|e| crate::plugin::PluginError::EventError(e.to_string()))?;

            if (self.filter)(&event) {
                return Ok(event);
            }
        }
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Result<PluginEvent> {
        loop {
            let event = self
                .receiver
                .try_recv()
                .map_err(|e| crate::plugin::PluginError::EventError(e.to_string()))?;

            if (self.filter)(&event) {
                return Ok(event);
            }
        }
    }
}

/// Event handler trait
pub trait EventHandler: Send + Sync {
    /// Handle an event
    fn handle(&self, event: &PluginEvent);
}

/// Simple event handler function wrapper
pub struct FnEventHandler<F>
where
    F: Fn(&PluginEvent) + Send + Sync,
{
    f: F,
}

impl<F> EventHandler for FnEventHandler<F>
where
    F: Fn(&PluginEvent) + Send + Sync,
{
    fn handle(&self, event: &PluginEvent) {
        (self.f)(event);
    }
}

/// Create an event handler from a function
pub fn event_handler<F>(f: F) -> FnEventHandler<F>
where
    F: Fn(&PluginEvent) + Send + Sync,
{
    FnEventHandler { f }
}

/// Event dispatcher for managing multiple handlers
pub struct EventDispatcher {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add an event handler
    pub fn add_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    /// Dispatch an event to all handlers
    pub fn dispatch(&self, event: &PluginEvent) {
        for handler in &self.handlers {
            handler.handle(event);
        }
    }

    /// Remove all handlers
    pub fn clear(&mut self) {
        self.handlers.clear();
    }

    /// Get handler count
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new();
        let mut subscriber = bus.subscribe();

        // Publish event
        tokio::spawn(async move {
            bus.publish(PluginEvent::Tick { delta_time: 0.016 }).await;
        });

        // Receive event
        let event = subscriber.recv().await.unwrap();
        match event {
            PluginEvent::Tick { delta_time } => {
                assert_eq!(delta_time, 0.016);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut sub1 = bus.subscribe();
        let mut sub2 = bus.subscribe();

        tokio::spawn(async move {
            bus.publish(PluginEvent::Tick { delta_time: 0.016 }).await;
        });

        let event1 = sub1.recv().await.unwrap();
        let event2 = sub2.recv().await.unwrap();

        assert!(matches!(event1, PluginEvent::Tick { .. }));
        assert!(matches!(event2, PluginEvent::Tick { .. }));
    }

    #[tokio::test]
    async fn test_filtered_subscriber() {
        let bus = EventBus::new();

        // Filter for only Tick events
        let is_tick = |event: &PluginEvent| matches!(event, PluginEvent::Tick { .. });
        let mut sub = bus.subscribe_filtered(is_tick);

        tokio::spawn(async move {
            bus.publish(PluginEvent::Tick { delta_time: 0.016 }).await;
            bus.publish(PluginEvent::PluginLoaded {
                name: "test".to_string(),
            })
            .await;
        });

        // Should receive only Tick event
        let event = sub.recv().await.unwrap();
        assert!(matches!(event, PluginEvent::Tick { .. }));
    }

    #[test]
    fn test_event_handler() {
        let dispatcher = EventDispatcher::new();

        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        dispatcher.add_handler(Box::new(event_handler(move |event| {
            if matches!(event, PluginEvent::Tick { .. }) {
                called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        })));

        dispatcher.dispatch(&PluginEvent::Tick { delta_time: 0.016 });

        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }
}
