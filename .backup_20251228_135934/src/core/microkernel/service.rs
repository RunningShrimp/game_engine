//! 服务定义
//!
//! 定义微内核架构中的服务接口和基础实现。

use std::fmt;

use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceId(String);

impl ServiceId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum ServiceState {
    #[default]
    Uninitialized,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
}


#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub id: ServiceId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub state: ServiceState,
}

impl ServiceInfo {
    pub fn new(id: ServiceId, name: String, version: String, description: String) -> Self {
        Self {
            id,
            name,
            version,
            description,
            state: ServiceState::Uninitialized,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ServiceError {
    #[error("Service is not in the correct state")]
    InvalidState,

    #[error("Service not found: {0}")]
    NotFound(String),

    #[error("Service initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Service update failed: {0}")]
    UpdateFailed(String),

    #[error("Service shutdown failed: {0}")]
    ShutdownFailed(String),

    #[error("Message handling failed: {0}")]
    MessageHandlingFailed(String),

    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),
}

#[async_trait]
pub trait Service: Send + Sync {
    fn id(&self) -> ServiceId;

    fn name(&self) -> &str;

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        ""
    }

    fn dependencies(&self) -> Vec<ServiceId> {
        Vec::new()
    }

    async fn start(&mut self) -> Result<(), ServiceError>;

    async fn update(&mut self) -> Result<(), ServiceError>;

    async fn shutdown(&mut self) -> Result<(), ServiceError>;

    async fn handle_message(
        &mut self,
        message: super::Message,
    ) -> Result<Option<super::Message>, ServiceError>;

    fn info(&self) -> ServiceInfo {
        ServiceInfo::new(
            self.id(),
            self.name().to_string(),
            self.version().to_string(),
            self.description().to_string(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct ServiceContext {
    pub id: ServiceId,
    pub state: ServiceState,
}

impl ServiceContext {
    pub fn new(id: ServiceId) -> Self {
        Self {
            id,
            state: ServiceState::Uninitialized,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub enabled: bool,
    pub priority: u8,
    pub auto_start: bool,
    pub max_retries: u32,
    pub restart_on_failure: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 128,
            auto_start: true,
            max_retries: 3,
            restart_on_failure: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_id() {
        let id = ServiceId::new("test_service");
        assert_eq!(id.as_str(), "test_service");
    }

    #[test]
    fn test_service_info() {
        let id = ServiceId::new("test_service");
        let info = ServiceInfo::new(
            id.clone(),
            "Test Service".to_string(),
            "1.0.0".to_string(),
            "A test service".to_string(),
        );
        assert_eq!(info.id, id);
        assert_eq!(info.name, "Test Service");
    }
}
