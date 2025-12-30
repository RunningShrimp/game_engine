//! 微内核架构
//!
//! 微内核架构将引擎核心功能保持最小化，其他功能作为服务运行在用户空间。
//!
//! ## 架构设计
//!
//! Microkernel Core (内核空间 - 最小化核心)
//!   - Scheduler (任务调度)
//!   - Message Bus (消息总线)
//!   - Resource Manager (资源管理器)
//!   - IPC Mechanism (进程间通信)
//!
//! User Space Services (用户空间 - 独立服务)
//!   - Physics Service
//!   - Render Service
//!   - Audio Service
//!   - Network Service
//!   - Scripting Service
//!   - UI Service
//!
//! ## 核心概念
//!
//! - **微内核 (Microkernel)**: 提供最基础的服务（调度、消息传递、资源管理）
//! - **服务 (Service)**: 独立的功能模块，通过 IPC 与内核和其他服务通信
//! - **消息 (Message)**: 服务间通信的基本单位
//! - **服务注册表 (Service Registry)**: 管理所有可用服务
//!
//! ## 优势
//!
//! 1. **模块化**: 每个服务独立开发和测试
//! 2. **可扩展性**: 可以动态加载/卸载服务
//! 3. **隔离性**: 服务崩溃不会影响整个系统
//! 4. **灵活性**: 可以替换服务实现而不影响其他服务
//! 5. **安全性**: 服务运行在隔离的环境中
//!
//! ## 挑战
//!
//! 1. **性能开销**: IPC 有一定的性能开销
//! 2. **复杂性**: 调试分布式系统更复杂
//! 3. **依赖管理**: 服务间依赖需要仔细管理
//! 4. **启动顺序**: 需要确保依赖服务先启动

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

use bevy_ecs::prelude::*;

pub mod ipc;
pub mod message;
pub mod registry;
pub mod scheduler;
pub mod service;

pub use ipc::{IpcChannel, IpcError, Request, Response};
pub use message::{Message, MessageId, MessagePayload, MessageType};
pub use registry::{ServiceRegistry, ServiceRegistryError};
pub use scheduler::{SchedulerConfig, ServiceScheduler};
pub use service::{Service, ServiceId, ServiceInfo, ServiceState};

#[derive(Debug, Clone, thiserror::Error)]
pub enum MicrokernelError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Service already exists: {0}")]
    ServiceAlreadyExists(String),

    #[error("Service error: {0}")]
    ServiceError(String),

    #[error("IPC error: {0}")]
    IpcError(#[from] IpcError),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Timeout waiting for response")]
    Timeout,
}

impl From<ServiceRegistryError> for MicrokernelError {
    fn from(err: ServiceRegistryError) -> Self {
        MicrokernelError::RegistryError(err.to_string())
    }
}

pub struct Microkernel {
    registry: Arc<ServiceRegistry>,
    scheduler: Arc<ServiceScheduler>,
    ipc_channels: Arc<RwLock<HashMap<ServiceId, IpcChannel>>>,
}

impl Microkernel {
    pub fn new() -> Self {
        let registry = Arc::new(ServiceRegistry::new());
        let scheduler = Arc::new(ServiceScheduler::new(SchedulerConfig::default()));

        Self {
            registry,
            scheduler,
            ipc_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn registry(&self) -> &Arc<ServiceRegistry> {
        &self.registry
    }

    pub fn scheduler(&self) -> &Arc<ServiceScheduler> {
        &self.scheduler
    }

    pub async fn register_service<S: Service + 'static>(
        &self,
        service: S,
    ) -> Result<ServiceId, MicrokernelError> {
        let service_id = service.id();
        let service_arc = Arc::new(Mutex::new(service));

        self.registry.register(service_id.clone(), service_arc).await?;

        let ipc_channel = IpcChannel::new(service_id.clone());
        self.ipc_channels.write().await.insert(service_id.clone(), ipc_channel);

        Ok(service_id)
    }

    pub async fn unregister_service(&self, service_id: &ServiceId) -> Result<(), MicrokernelError> {
        self.registry.unregister(service_id).await?;
        self.ipc_channels.write().await.remove(service_id);
        Ok(())
    }

    pub async fn send_message(
        &self,
        target: &ServiceId,
        message: Message,
    ) -> Result<Option<Message>, MicrokernelError> {
        let channels = self.ipc_channels.read().await;
        let channel = channels
            .get(target)
            .ok_or_else(|| MicrokernelError::ServiceNotFound(target.as_str().to_string()))?;

        channel.send(message).await.map_err(MicrokernelError::from)
    }

    pub async fn request(
        &self,
        target: &ServiceId,
        message: Message,
        timeout: Duration,
    ) -> Result<Response, MicrokernelError> {
        let channels = self.ipc_channels.read().await;
        let channel = channels
            .get(target)
            .ok_or_else(|| MicrokernelError::ServiceNotFound(target.as_str().to_string()))?;

        tokio::time::timeout(timeout, channel.request(message))
            .await
            .map_err(|_| MicrokernelError::Timeout)?
            .map_err(MicrokernelError::from)
    }

    pub async fn update(&self) {
        self.scheduler.update().await;

        let services = self.registry.services();
        for (_, service) in services {
            let mut s = service.lock().await;
            let _ = s.update().await;
        }
    }

    pub async fn shutdown(&self) {
        let services = self.registry.services();
        for (_, service) in services {
            let mut s = service.lock().await;
            let _ = s.shutdown().await;
        }
    }
}

impl Default for Microkernel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_microkernel_creation() {
        let kernel = Microkernel::new();
        assert!(kernel.registry().services().is_empty());
    }

    #[tokio::test]
    async fn test_service_registration() {
        let kernel = Microkernel::new();
        let service = DummyService::new("test_service".to_string());

        let service_id =
            kernel.register_service(service).await.expect("Failed to register test_service");
        assert_eq!(kernel.registry().services().len(), 1);

        // 验证服务已注册，使用实际的元组结构
        let services = kernel.registry().services();
        // services 是 Vec<(ServiceId, Arc<Mutex<dyn Service>>)>
        if let Some((id, _service)) =
            services.iter().find(|(id, _)| id.to_string() == "test_service")
        {
            assert_eq!(id.to_string(), "test_service");
        }
    }

    #[tokio::test]
    async fn test_dummy_service_multiple_instances() {
        // 测试 DummyService::new 的完整使用
        let service1 = DummyService::new("service1".to_string());
        let service2 = DummyService::new("service2".to_string());

        assert_ne!(service1.id(), service2.id());
        assert_eq!(service1.name(), "service1");
        assert_eq!(service2.name(), "service2");
    }
}

struct DummyService {
    id: ServiceId,
    name: String,
}

impl DummyService {
    fn new(name: String) -> Self {
        Self {
            id: ServiceId::new(&name),
            name,
        }
    }
}

#[async_trait::async_trait]
impl Service for DummyService {
    fn id(&self) -> ServiceId {
        self.id.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn start(&mut self) -> Result<(), service::ServiceError> {
        Ok(())
    }

    async fn update(&mut self) -> Result<(), service::ServiceError> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), service::ServiceError> {
        Ok(())
    }

    async fn handle_message(
        &mut self,
        _message: Message,
    ) -> Result<Option<Message>, service::ServiceError> {
        Ok(None)
    }
}
