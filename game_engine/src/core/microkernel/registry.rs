//! 服务注册表
//!
//! 管理所有注册的服务，提供服务查找和依赖解析功能。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, Mutex};

use super::{Service, ServiceId, ServiceInfo};

#[derive(Debug, Clone, thiserror::Error)]
pub enum ServiceRegistryError {
    #[error("Service not found: {0}")]
    NotFound(String),

    #[error("Service already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<ServiceId>),

    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),
}

pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<ServiceId, Arc<Mutex<dyn Service>>>>>,
    service_info: Arc<RwLock<HashMap<ServiceId, ServiceInfo>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_info: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register<S: Service + 'static>(
        &self,
        service_id: ServiceId,
        service: Arc<Mutex<S>>,
    ) -> Result<(), ServiceRegistryError> {
        let mut services = self.services.write().await;
        if services.contains_key(&service_id) {
            return Err(ServiceRegistryError::AlreadyRegistered(service_id.as_str().to_string()));
        }
        services.insert(service_id.clone(), service);
        Ok(())
    }

    pub async fn unregister(&self, service_id: &ServiceId) -> Result<(), ServiceRegistryError> {
        let mut services = self.services.write().await;
        let mut info = self.service_info.write().await;

        if services.remove(service_id).is_none() {
            return Err(ServiceRegistryError::NotFound(service_id.as_str().to_string()));
        }

        info.remove(service_id);
        Ok(())
    }

    pub async fn get(&self, service_id: &ServiceId) -> Option<Arc<Mutex<dyn Service>>> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    pub async fn services(&self) -> HashMap<ServiceId, Arc<Mutex<dyn Service>>> {
        let services = self.services.read().await;
        services.clone()
    }

    pub async fn service_info(&self, service_id: &ServiceId) -> Option<ServiceInfo> {
        let info = self.service_info.read().await;
        info.get(service_id).cloned()
    }

    pub async fn all_service_info(&self) -> Vec<ServiceInfo> {
        let info = self.service_info.read().await;
        info.values().cloned().collect()
    }

    pub async fn resolve_dependencies(&self, service_id: &ServiceId) -> Result<Vec<ServiceId>, ServiceRegistryError> {
        let mut visited = Vec::new();
        let mut visiting = Vec::new();
        self.resolve_dependencies_recursive(service_id, &mut visited, &mut visiting).await
    }

    fn resolve_dependencies_recursive<'a>(
        &'a self,
        service_id: &'a ServiceId,
        visited: &'a mut Vec<ServiceId>,
        visiting: &'a mut Vec<ServiceId>,
    ) -> impl std::future::Future<Output = Result<Vec<ServiceId>, ServiceRegistryError>> + 'a {
        async move {
            if visited.contains(service_id) {
                return Ok(vec![]);
            }

            if visiting.contains(service_id) {
                let mut cycle = visiting.clone();
                cycle.push(service_id.clone());
                return Err(ServiceRegistryError::CircularDependency(cycle));
            }

            visiting.push(service_id.clone());

            let service = self.get(service_id).await
                .ok_or_else(|| ServiceRegistryError::NotFound(service_id.as_str().to_string()))?;

            let service_guard = service.lock().await;
            let dependencies = service_guard.dependencies();
            drop(service_guard);

            let mut resolved = Vec::new();
            for dep_id in dependencies {
                let dep_id_clone = dep_id.clone();
                resolved.extend(
                    Box::pin(self.resolve_dependencies_recursive(&dep_id_clone, visited, visiting)).await?
                );
            }

            visiting.pop();
            visited.push(service_id.clone());
            resolved.push(service_id.clone());

            Ok(resolved)
        }
    }

    pub async fn get_startup_order(&self) -> Result<Vec<ServiceId>, ServiceRegistryError> {
        let services = self.services.read().await;
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for service_id in services.keys() {
            self.get_startup_order_recursive(service_id, &mut order, &mut visited).await?;
        }

        Ok(order)
    }

    fn get_startup_order_recursive<'a>(
        &'a self,
        service_id: &'a ServiceId,
        order: &'a mut Vec<ServiceId>,
        visited: &'a mut std::collections::HashSet<ServiceId>,
    ) -> impl std::future::Future<Output = Result<(), ServiceRegistryError>> + 'a {
        async move {
            if visited.contains(service_id) {
                return Ok(());
            }

            visited.insert(service_id.clone());

            let service = self.get(service_id).await
                .ok_or_else(|| ServiceRegistryError::NotFound(service_id.as_str().to_string()))?;

            let service_guard = service.lock().await;
            let dependencies = service_guard.dependencies();
            drop(service_guard);

            for dep_id in dependencies {
                let dep_id_clone = dep_id.clone();
                Box::pin(self.get_startup_order_recursive(&dep_id_clone, order, visited)).await?;
            }

            if !order.contains(service_id) {
                order.push(service_id.clone());
            }

            Ok(())
        }
    }

    pub async fn start_all(&self) -> Result<(), ServiceRegistryError> {
        let startup_order = self.get_startup_order().await?;

        for service_id in &startup_order {
            if let Some(service) = self.get(service_id).await {
                let mut service_guard = service.lock().await;
                let _ = service_guard.start().await;
            }
        }

        Ok(())
    }

    pub async fn update_all(&self) {
        let services = self.services.read().await;
        for (_, service) in services.iter() {
            let mut service_guard = service.lock().await;
            let _ = service_guard.update().await;
        }
    }

    pub async fn shutdown_all(&self) {
        let services = self.services.read().await;
        for (_, service) in services.iter() {
            let mut service_guard = service.lock().await;
            let _ = service_guard.shutdown().await;
        }
    }

    pub async fn count(&self) -> usize {
        let services = self.services.read().await;
        services.len()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ServiceRegistry::new();
        assert_eq!(registry.count().await, 0);
    }
}
