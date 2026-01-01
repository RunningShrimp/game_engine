//! # LSP-DAP Integration
//!
//! 集成DAP（Debug Adapter Protocol）到LSP服务器，提供统一的调试和语言服务体验。
//!
//! ## 功能
//!
//! - 在LSP服务器中启动DAP服务器
//! - 调试会话管理
//! - 断点同步（LSP文档 <-> DAP断点）
//! - 变量监视同步
//! - 调用栈和作用域信息提供

use crate::debug::dap::server::{Breakpoint, DapConfig, DapServer, DapSessionState};
use std::sync::Arc;
use tokio::sync::Mutex;

/// LSP-DAP集成器
///
/// 负责在LSP服务器中管理DAP调试会话
pub struct LspDapIntegrator {
    /// DAP服务器实例
    dap_server: Option<DapServer>,
    /// DAP服务器是否正在运行
    is_running: Arc<Mutex<bool>>,
    /// 当前调试会话状态
    session_state: Arc<Mutex<DapSessionState>>,
    /// 调试的文档URI
    debug_document_uri: Arc<Mutex<Option<String>>>,
    /// 调试的语言
    debug_language: Arc<Mutex<Option<String>>>,
}

impl LspDapIntegrator {
    /// 创建新的LSP-DAP集成器
    pub fn new() -> Self {
        Self {
            dap_server: None,
            is_running: Arc::new(Mutex::new(false)),
            session_state: Arc::new(Mutex::new(DapSessionState::NotStarted)),
            debug_document_uri: Arc::new(Mutex::new(None)),
            debug_language: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动DAP服务器
    ///
    /// 在指定的端口上启动DAP服务器，用于调试会话
    pub async fn start_dap_server(&mut self, port: u16) -> Result<(), String> {
        let config = DapConfig {
            host: "127.0.0.1".to_string(),
            port,
            supported_languages: vec![
                "lua".to_string(),
                "typescript".to_string(),
                "javascript".to_string(),
                "python".to_string(),
            ],
            enable_conditional_breakpoints: true,
            enable_logpoints: true,
        };

        let dap_server = DapServer::new(config);
        dap_server.start().await?;

        self.dap_server = Some(dap_server);
        *self.is_running.lock().await = true;

        tracing::info!("DAP server started on port {}", port);
        Ok(())
    }

    /// 停止DAP服务器
    pub async fn stop_dap_server(&mut self) -> Result<(), String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.stop().await?;
        }

        self.dap_server = None;
        *self.is_running.lock().await = false;
        *self.session_state.lock().await = DapSessionState::Terminated;

        tracing::info!("DAP server stopped");
        Ok(())
    }

    /// 开始调试会话
    ///
    /// 启动指定文档的调试会话
    pub async fn start_debugging(
        &mut self,
        document_uri: String,
        language: String,
    ) -> Result<(), String> {
        // 确保DAP服务器正在运行
        if !*self.is_running.lock().await {
            return Err("DAP server is not running".to_string());
        }

        // 设置调试上下文
        *self.debug_document_uri.lock().await = Some(document_uri.clone());
        *self.debug_language.lock().await = Some(language.clone());
        *self.session_state.lock().await = DapSessionState::Initializing;

        tracing::info!(
            "Starting debugging session for {} (language: {})",
            document_uri,
            language
        );

        // 这里可以添加特定语言的调试初始化逻辑
        // 例如：Lua调试器初始化、TypeScript调试器等

        *self.session_state.lock().await = DapSessionState::Stopped;

        Ok(())
    }

    /// 停止调试会话
    pub async fn stop_debugging(&mut self) -> Result<(), String> {
        *self.debug_document_uri.lock().await = None;
        *self.debug_language.lock().await = None;
        *self.session_state.lock().await = DapSessionState::Terminated;

        tracing::info!("Debugging session stopped");
        Ok(())
    }

    /// 继续执行
    pub async fn continue_execution(&self) -> Result<(), String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.continue_execution().await?;
            *self.session_state.lock().await = DapSessionState::Running;
        }
        Ok(())
    }

    /// 暂停执行
    pub async fn pause_execution(&self) -> Result<(), String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.pause().await?;
            *self.session_state.lock().await = DapSessionState::Stopped;
        }
        Ok(())
    }

    /// 单步执行
    pub async fn step_over(&self) -> Result<(), String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.step_over().await?;
        }
        Ok(())
    }

    /// 单步进入
    pub async fn step_into(&self) -> Result<(), String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.step_into().await?;
        }
        Ok(())
    }

    /// 单步跳出
    pub async fn step_out(&self) -> Result<(), String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.step_out().await?;
        }
        Ok(())
    }

    /// 设置断点
    ///
    /// 在指定文档的指定行设置断点
    pub async fn set_breakpoints(
        &self,
        document_uri: &str,
        lines: Vec<i64>,
    ) -> Result<Vec<Breakpoint>, String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.set_breakpoints(document_uri, lines).await
        } else {
            Err("DAP server is not running".to_string())
        }
    }

    /// 获取调用栈
    pub async fn get_stack_trace(
        &self,
    ) -> Result<Vec<crate::debug::dap::server::StackFrame>, String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.stack_trace().await
        } else {
            Err("DAP server is not running".to_string())
        }
    }

    /// 获取作用域
    pub async fn get_scopes(
        &self,
        frame_id: i64,
    ) -> Result<Vec<crate::debug::dap::server::Scope>, String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.scopes(frame_id).await
        } else {
            Err("DAP server is not running".to_string())
        }
    }

    /// 获取变量
    pub async fn get_variables(
        &self,
        variables_reference: i64,
    ) -> Result<Vec<crate::debug::dap::server::Variable>, String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.variables(variables_reference).await
        } else {
            Err("DAP server is not running".to_string())
        }
    }

    /// 求值表达式
    pub async fn evaluate(&self, expression: &str, frame_id: i64) -> Result<String, String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.evaluate(expression, frame_id).await
        } else {
            Err("DAP server is not running".to_string())
        }
    }

    /// 获取当前调试会话状态
    pub async fn get_session_state(&self) -> DapSessionState {
        *self.session_state.lock().await
    }

    /// 检查DAP服务器是否正在运行
    pub async fn is_dap_running(&self) -> bool {
        *self.is_running.lock().await
    }

    /// 获取当前调试的文档URI
    pub async fn get_debug_document_uri(&self) -> Option<String> {
        self.debug_document_uri.lock().await.clone()
    }

    /// 获取当前调试的语言
    pub async fn get_debug_language(&self) -> Option<String> {
        self.debug_language.lock().await.clone()
    }

    /// 发送自定义DAP请求
    pub async fn send_dap_request(
        &self,
        command: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        if let Some(dap_server) = &self.dap_server {
            dap_server.send_request(command, arguments).await
        } else {
            Err("DAP server is not running".to_string())
        }
    }
}

impl Default for LspDapIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lsp_dap_integrator_creation() {
        let integrator = LspDapIntegrator::new();
        assert!(!integrator.is_dap_running().await);
        assert_eq!(
            integrator.get_session_state().await,
            DapSessionState::NotStarted
        );
    }

    #[tokio::test]
    async fn test_default_integrator() {
        let integrator = LspDapIntegrator::default();
        assert!(!integrator.is_dap_running().await);
    }
}
