//! # Plugin Sandbox
//!
//! Sandboxed execution environment for plugins.

use crate::plugin::{api::PluginPermission, PluginError, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Sandbox for isolating plugin execution
pub struct Sandbox {
    permissions: HashSet<PluginPermission>,
    allowed_paths: HashSet<PathBuf>,
    allowed_network_hosts: HashSet<String>,
    resource_limits: ResourceLimits,
    state: Arc<RwLock<SandboxState>>,
}

/// Sandbox state tracking
#[derive(Debug, Default)]
struct SandboxState {
    files_accessed: HashSet<PathBuf>,
    network_requests: Vec<NetworkRequest>,
    memory_usage: usize,
    cpu_time: u64,
}

/// Network request tracking
#[derive(Debug, Clone)]
struct NetworkRequest {
    host: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Resource limits for sandboxed plugins
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: usize,
    pub max_cpu_time_ms: u64,
    pub max_file_handles: usize,
    pub max_network_connections: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_time_ms: 1000,
            max_file_handles: 100,
            max_network_connections: 10,
        }
    }
}

impl Sandbox {
    /// Create a new sandbox with given permissions
    pub fn new(permissions: HashSet<PluginPermission>) -> Self {
        Self {
            permissions,
            allowed_paths: HashSet::new(),
            allowed_network_hosts: HashSet::new(),
            resource_limits: ResourceLimits::default(),
            state: Arc::new(RwLock::new(SandboxState::default())),
        }
    }

    /// Create a restrictive sandbox (no permissions)
    pub fn restrictive() -> Self {
        Self::new(HashSet::new())
    }

    /// Create a permissive sandbox (all permissions)
    pub fn permissive() -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(PluginPermission::Read);
        permissions.insert(PluginPermission::Write);
        permissions.insert(PluginPermission::Network);
        permissions.insert(PluginPermission::Filesystem);

        Self::new(permissions)
    }

    /// Add allowed path for file access
    pub fn allow_path(&mut self, path: PathBuf) {
        self.allowed_paths.insert(path);
    }

    /// Add allowed network host
    pub fn allow_host(&mut self, host: String) {
        self.allowed_network_hosts.insert(host);
    }

    /// Set resource limits
    pub fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.resource_limits = limits;
    }

    /// Check if plugin has a specific permission
    pub fn has_permission(&self, permission: &PluginPermission) -> bool {
        self.permissions.contains(permission)
    }

    /// Check if plugin can access a path
    pub fn can_access_path(&self, path: &PathBuf) -> bool {
        if !self.permissions.contains(&PluginPermission::Filesystem) {
            return false;
        }

        if self.allowed_paths.is_empty() {
            return true; // No restrictions
        }

        self.allowed_paths
            .iter()
            .any(|allowed| path.starts_with(allowed))
    }

    /// Check if plugin can connect to a host
    pub fn can_connect_to(&self, host: &str) -> bool {
        if !self.permissions.contains(&PluginPermission::Network) {
            return false;
        }

        if self.allowed_network_hosts.is_empty() {
            return true; // No restrictions
        }

        self.allowed_network_hosts.contains(host)
    }

    /// Validate a file access operation
    pub fn validate_file_access(&self, path: &PathBuf, write: bool) -> Result<()> {
        if write && !self.has_permission(&PluginPermission::Write) {
            return Err(PluginError::PermissionDenied(
                "Write permission required".to_string(),
            ));
        }

        if !self.has_permission(&PluginPermission::Filesystem) {
            return Err(PluginError::PermissionDenied(
                "Filesystem permission required".to_string(),
            ));
        }

        if !self.can_access_path(path) {
            return Err(PluginError::PermissionDenied(format!(
                "Access to path '{}' denied",
                path.display()
            )));
        }

        // Track access
        if let Ok(mut state) = self.state.write() {
            state.files_accessed.insert(path.clone());
        }

        Ok(())
    }

    /// Validate a network operation
    pub fn validate_network_access(&self, host: &str) -> Result<()> {
        if !self.has_permission(&PluginPermission::Network) {
            return Err(PluginError::PermissionDenied(
                "Network permission required".to_string(),
            ));
        }

        if !self.can_connect_to(host) {
            return Err(PluginError::PermissionDenied(format!(
                "Connection to host '{}' denied",
                host
            )));
        }

        // Track request
        if let Ok(mut state) = self.state.write() {
            state.network_requests.push(NetworkRequest {
                host: host.to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(())
    }

    /// Get sandbox state
    pub fn state(&self) -> SandboxStateSnapshot {
        let state = self.state.read().unwrap();
        SandboxStateSnapshot {
            files_accessed: state.files_accessed.clone(),
            network_requests_count: state.network_requests.len(),
            memory_usage: state.memory_usage,
            cpu_time: state.cpu_time,
        }
    }

    /// Reset sandbox state
    pub fn reset_state(&self) {
        if let Ok(mut state) = self.state.write() {
            *state = SandboxState::default();
        }
    }
}

/// Snapshot of sandbox state
#[derive(Debug, Clone)]
pub struct SandboxStateSnapshot {
    pub files_accessed: HashSet<PathBuf>,
    pub network_requests_count: usize,
    pub memory_usage: usize,
    pub cpu_time: u64,
}

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub permissions: Vec<PluginPermission>,
    pub allowed_paths: Vec<PathBuf>,
    pub allowed_hosts: Vec<String>,
    pub resource_limits: ResourceLimits,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            permissions: vec![PluginPermission::Read],
            allowed_paths: Vec::new(),
            allowed_hosts: Vec::new(),
            resource_limits: ResourceLimits::default(),
        }
    }
}

impl SandboxConfig {
    pub fn into_sandbox(self) -> Sandbox {
        let mut sandbox = Sandbox::new(self.permissions.into_iter().collect());
        for path in self.allowed_paths {
            sandbox.allow_path(path);
        }
        for host in self.allowed_hosts {
            sandbox.allow_host(host);
        }
        sandbox.set_resource_limits(self.resource_limits);
        sandbox
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restrictive_sandbox() {
        let sandbox = Sandbox::restrictive();
        assert!(!sandbox.has_permission(&PluginPermission::Read));
        assert!(!sandbox.has_permission(&PluginPermission::Write));
        assert!(!sandbox.has_permission(&PluginPermission::Network));
    }

    #[test]
    fn test_permissive_sandbox() {
        let sandbox = Sandbox::permissive();
        assert!(sandbox.has_permission(&PluginPermission::Read));
        assert!(sandbox.has_permission(&PluginPermission::Write));
        assert!(sandbox.has_permission(&PluginPermission::Network));
    }

    #[test]
    fn test_path_access_control() {
        let mut sandbox = Sandbox::permissive();
        sandbox.allow_path(PathBuf::from("/tmp/allowed"));

        assert!(sandbox.can_access_path(&PathBuf::from("/tmp/allowed/file.txt")));
        assert!(!sandbox.can_access_path(&PathBuf::from("/tmp/other/file.txt")));
    }

    #[test]
    fn test_network_access_control() {
        let mut sandbox = Sandbox::permissive();
        sandbox.allow_host("example.com".to_string());

        assert!(sandbox.can_connect_to("example.com"));
        assert!(!sandbox.can_connect_to("other.com"));
    }

    #[test]
    fn test_file_access_validation() {
        let mut sandbox = Sandbox::permissive();
        sandbox.allow_path(PathBuf::from("/tmp/allowed"));

        let result = sandbox.validate_file_access(&PathBuf::from("/tmp/allowed/file.txt"), false);
        assert!(result.is_ok());

        let result = sandbox.validate_file_access(&PathBuf::from("/tmp/disallowed/file.txt"), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_limits() {
        let limits = ResourceLimits {
            max_memory_mb: 1024,
            max_cpu_time_ms: 2000,
            max_file_handles: 200,
            max_network_connections: 20,
        };

        assert_eq!(limits.max_memory_mb, 1024);
        assert_eq!(limits.max_cpu_time_ms, 2000);
    }
}
