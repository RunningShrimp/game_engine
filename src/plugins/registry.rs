//! Plugin registry
//!
//! Manages plugin metadata, inter-plugin communication, and dependencies.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use crate::error::{Error, Result};
use super::api::{Plugin, PluginContext, PluginMetadata, PluginEvent, PluginState};

/// Inter-plugin message
#[derive(Debug, Clone)]
pub struct PluginMessage {
    /// Sender plugin name
    pub sender: String,
    /// Receiver plugin name (or empty for broadcast)
    pub receiver: String,
    /// Message type
    pub message_type: String,
    /// Message data
    pub data: String,
}

impl PluginMessage {
    /// Create a new message
    pub fn new(sender: String, receiver: String, message_type: String, data: String) -> Self {
        Self {
            sender,
            receiver,
            message_type,
            data,
        }
    }

    /// Create a broadcast message
    pub fn broadcast(sender: String, message_type: String, data: String) -> Self {
        Self {
            sender,
            receiver: String::new(),
            message_type,
            data,
        }
    }
}

/// Message handler callback type
pub type MessageHandler = Box<dyn Fn(&PluginMessage) + Send + Sync>;

/// Plugin registry entry
#[derive(Clone)]
pub struct PluginEntry {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Plugin state
    pub state: PluginState,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Plugins that depend on this one
    pub dependents: Vec<String>,
    /// Load order (lower loads first)
    pub load_order: usize,
}

impl PluginEntry {
    /// Create a new plugin entry
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            dependencies: metadata.dependencies.clone(),
            metadata,
            state: PluginState::Loaded,
            dependents: Vec::new(),
            load_order: 0,
        }
    }
}

/// Plugin registry
pub struct PluginRegistry {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<String, PluginEntry>>>,
    /// Message handlers
    message_handlers: Arc<RwLock<HashMap<String, Vec<MessageHandler>>>>,
    /// Message queue
    message_queue: Arc<RwLock<Vec<PluginMessage>>>,
    /// Event subscriptions
    event_subscriptions: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// API providers (plugins that provide APIs)
    api_providers: Arc<RwLock<HashMap<String, Arc<dyn Plugin>>>>,
    /// API consumers (plugins that use APIs)
    api_consumers: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Arc::new(RwLock::new(HashMap::new())),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            event_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            api_providers: Arc::new(RwLock::new(HashMap::new())),
            api_consumers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&self, metadata: PluginMetadata) -> Result<()> {
        let name = metadata.name.clone();

        // Check if already registered
        {
            let plugins = self.plugins.read().unwrap();
            if plugins.contains_key(&name) {
                return Err(Error::PluginError(format!("Plugin {} is already registered", name)));
            }
        }

        // Check dependencies exist
        {
            let plugins = self.plugins.read().unwrap();
            for dep in &metadata.dependencies {
                if !plugins.contains_key(dep) {
                    return Err(Error::PluginDependencyError {
                        plugin: name.clone(),
                        missing: dep.clone(),
                    });
                }
            }
        }

        // Calculate load order based on dependencies
        let load_order = self.calculate_load_order(&metadata.dependencies);

        // Create plugin entry
        let entry = PluginEntry {
            metadata,
            load_order,
            ..Default::default()
        };

        // Register the plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(name.clone(), entry);
        }

        // Update dependents
        {
            let mut plugins = self.plugins.write().unwrap();
            for dep in entry.dependencies {
                if let Some(dep_entry) = plugins.get_mut(&dep) {
                    dep_entry.dependents.push(name.clone());
                }
            }
        }

        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&self, name: &str) -> Result<()> {
        // Check if any plugins depend on this one
        {
            let plugins = self.plugins.read().unwrap();
            if let Some(entry) = plugins.get(name) {
                if !entry.dependents.is_empty() {
                    return Err(Error::PluginError(format!(
                        "Cannot unregister plugin {}: {} plugins depend on it",
                        name,
                        entry.dependents.len()
                    )));
                }
            }
        }

        // Remove the plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(name);
        }

        // Remove message handlers
        {
            let mut handlers = self.message_handlers.write().unwrap();
            handlers.remove(name);
        }

        // Remove event subscriptions
        {
            let mut subscriptions = self.event_subscriptions.write().unwrap();
            subscriptions.remove(name);
        }

        // Remove API providers
        {
            let mut providers = self.api_providers.write().unwrap();
            providers.remove(name);
        }

        // Remove API consumers
        {
            let mut consumers = self.api_consumers.write().unwrap();
            consumers.remove(name);
        }

        Ok(())
    }

    /// Calculate load order for a plugin based on dependencies
    fn calculate_load_order(&self, dependencies: &[String]) -> usize {
        let plugins = self.plugins.read().unwrap();
        let mut max_order = 0;

        for dep in dependencies {
            if let Some(entry) = plugins.get(dep) {
                max_order = max_order.max(entry.load_order);
            }
        }

        max_order + 1
    }

    /// Get a plugin entry
    pub fn get_plugin(&self, name: &str) -> Option<PluginEntry> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).cloned()
    }

    /// Get all plugins
    pub fn get_all_plugins(&self) -> Vec<PluginEntry> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }

    /// Get plugins sorted by load order
    pub fn get_plugins_by_load_order(&self) -> Vec<PluginEntry> {
        let mut plugins = self.get_all_plugins();
        plugins.sort_by_key(|p| p.load_order);
        plugins
    }

    /// Update plugin state
    pub fn update_state(&self, name: &str, state: PluginState) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(entry) = plugins.get_mut(name) {
            entry.state = state;
            Ok(())
        } else {
            Err(Error::PluginNotFound(name.to_string()))
        }
    }

    /// Get plugin state
    pub fn get_state(&self, name: &str) -> Option<PluginState> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).map(|p| p.state)
    }

    /// Register a message handler for a plugin
    pub fn register_message_handler(&self, plugin_name: String, handler: MessageHandler) {
        let mut handlers = self.message_handlers.write().unwrap();
        handlers.entry(plugin_name).or_insert_with(Vec::new).push(handler);
    }

    /// Send a message to a plugin
    pub fn send_message(&self, message: PluginMessage) {
        let mut queue = self.message_queue.write().unwrap();
        queue.push(message);
    }

    /// Process all pending messages
    pub fn process_messages(&self) {
        // Get all messages
        let messages = {
            let mut queue = self.message_queue.write().unwrap();
            std::mem::take(&mut *queue)
        };

        // Process each message
        for message in messages {
            let handlers = self.message_handlers.read().unwrap();

            if message.receiver.is_empty() {
                // Broadcast to all plugins
                for (_, plugin_handlers) in handlers.iter() {
                    for handler in plugin_handlers {
                        handler(&message);
                    }
                }
            } else {
                // Send to specific plugin
                if let Some(plugin_handlers) = handlers.get(&message.receiver) {
                    for handler in plugin_handlers {
                        handler(&message);
                    }
                }
            }
        }
    }

    /// Subscribe to events
    pub fn subscribe_to_event(&self, plugin_name: String, event_type: String) {
        let mut subscriptions = self.event_subscriptions.write().unwrap();
        subscriptions
            .entry(plugin_name)
            .or_insert_with(HashSet::new)
            .insert(event_type);
    }

    /// Publish an event to subscribed plugins
    pub fn publish_event(&self, plugin: &mut dyn Plugin, event: &PluginEvent) {
        let event_type = format!("{:?}", event);
        let subscriptions = self.event_subscriptions.read().unwrap();

        for (plugin_name, events) in subscriptions.iter() {
            if events.contains(&event_type) {
                let context = PluginContext::new(
                    std::path::PathBuf::from("data"),
                    std::path::PathBuf::from("config"),
                    false,
                );
                plugin.on_event(&context, event);
            }
        }
    }

    /// Register an API provider
    pub fn register_api_provider(&self, name: String, plugin: Arc<dyn Plugin>) {
        let mut providers = self.api_providers.write().unwrap();
        providers.insert(name, plugin);
    }

    /// Get an API provider
    pub fn get_api_provider(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        let providers = self.api_providers.read().unwrap();
        providers.get(name).cloned()
    }

    /// Register an API consumer
    pub fn register_api_consumer(&self, consumer: String, provider: String) {
        let mut consumers = self.api_consumers.write().unwrap();
        consumers
            .entry(provider)
            .or_insert_with(Vec::new)
            .push(consumer);
    }

    /// Get plugins that depend on a given plugin
    pub fn get_dependents(&self, name: &str) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .get(name)
            .map(|p| p.dependents.clone())
            .unwrap_or_default()
    }

    /// Check if a plugin can be unloaded
    pub fn can_unload(&self, name: &str) -> bool {
        let plugins = self.plugins.read().unwrap();
        if let Some(entry) = plugins.get(name) {
            entry.dependents.is_empty()
        } else {
            false
        }
    }

    /// Get plugin dependencies
    pub fn get_dependencies(&self, name: &str) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .get(name)
            .map(|p| p.dependencies.clone())
            .unwrap_or_default()
    }

    /// Get the number of registered plugins
    pub fn plugin_count(&self) -> usize {
        let plugins = self.plugins.read().unwrap();
        plugins.len()
    }

    /// Validate all plugin dependencies
    pub fn validate_dependencies(&self) -> Result<()> {
        let plugins = self.plugins.read().unwrap();

        for (name, entry) in plugins.iter() {
            for dep in &entry.dependencies {
                if !plugins.contains_key(dep) {
                    return Err(Error::PluginDependencyError {
                        plugin: name.clone(),
                        missing: dep.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check for circular dependencies
    pub fn check_circular_dependencies(&self) -> Result<()> {
        let plugins = self.plugins.read().unwrap();

        for (name, entry) in plugins.iter() {
            if self.has_circular_dependency(name, &mut HashSet::new()) {
                return Err(Error::PluginError(format!(
                    "Circular dependency detected for plugin {}",
                    name
                )));
            }
        }

        Ok(())
    }

    /// Helper function to detect circular dependencies
    fn has_circular_dependency(&self, name: &str, visited: &mut HashSet<String>) -> bool {
        if visited.contains(name) {
            return true;
        }

        visited.insert(name.clone());

        let plugins = self.plugins.read().unwrap();
        if let Some(entry) = plugins.get(name) {
            for dep in &entry.dependencies {
                if self.has_circular_dependency(dep, visited) {
                    return true;
                }
            }
        }

        visited.remove(name);
        false
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.plugin_count(), 0);
    }

    #[test]
    fn test_plugin_registration() {
        let registry = PluginRegistry::new();
        let metadata = PluginMetadata::new("test-plugin".to_string(), "1.0.0".to_string());

        assert!(registry.register_plugin(metadata).is_ok());
        assert_eq!(registry.plugin_count(), 1);

        let plugin = registry.get_plugin("test-plugin");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = PluginRegistry::new();
        let metadata = PluginMetadata::new("test-plugin".to_string(), "1.0.0".to_string());

        assert!(registry.register_plugin(metadata.clone()).is_ok());
        assert!(registry.register_plugin(metadata).is_err());
    }

    #[test]
    fn test_message_sending() {
        let registry = PluginRegistry::new();
        let message = PluginMessage::new(
            "sender".to_string(),
            "receiver".to_string(),
            "test".to_string(),
            "data".to_string(),
        );

        registry.send_message(message);
        registry.process_messages(); // Should not panic
    }

    #[test]
    fn test_load_order() {
        let registry = PluginRegistry::new();

        // Register plugin with no dependencies
        let metadata1 = PluginMetadata::new("plugin1".to_string(), "1.0.0".to_string());
        registry.register_plugin(metadata1).unwrap();

        // Register plugin that depends on plugin1
        let metadata2 = PluginMetadata::new("plugin2".to_string(), "1.0.0".to_string())
            .with_dependency("plugin1".to_string());
        registry.register_plugin(metadata2).unwrap();

        let plugins = registry.get_plugins_by_load_order();
        assert_eq!(plugins[0].metadata.name, "plugin1");
        assert_eq!(plugins[1].metadata.name, "plugin2");
    }
}
