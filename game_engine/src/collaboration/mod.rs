//! # 实时协作系统（Real-time Collaboration）
//!
//! 基于CRDT的冲突自由实时协作功能。
//!
//! ## 功能特性
//!
//! - **CRDT数据结构**: 无需服务器的冲突解决
//! - **WebSocket实时通信**: 低延迟数据同步
//! - **操作转换**: OT/CRDT混合支持
//! - **版本控制集成**: Git友好
//! - **离线支持**: 本地操作队列

use crate::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub mod crdt;
pub mod network;
pub mod sync;

pub use crdt::{CrdtDocument, CrdtOperation, GCounter, LwwRegister};
pub use network::{CollaborationNetwork, NetworkMessage, WebSocketClient};
pub use sync::{DocumentSync, SyncStatus};

// =============================================================================
// 协作会话
// =============================================================================

/// 协作会话ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u64);

impl SessionId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// 用户ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId {
    pub id: String,
    pub name: String,
}

impl UserId {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}

/// 协作会话
#[derive(Debug, Clone)]
pub struct CollaborationSession {
    /// 会话ID
    pub id: SessionId,
    /// 会话名称
    pub name: String,
    /// 创建者
    pub creator: UserId,
    /// 参与者
    pub participants: HashMap<UserId, ParticipantInfo>,
    /// 创建时间
    pub created_at: std::time::Instant,
    /// 文档
    pub document: CrdtDocument,
}

/// 参与者信息
#[derive(Debug, Clone)]
pub struct ParticipantInfo {
    /// 用户
    pub user: UserId,
    /// 加入时间
    pub joined_at: std::time::Instant,
    /// 光标位置
    pub cursor: Option<CursorInfo>,
    /// 在线状态
    pub online: bool,
    /// 角色权限
    pub role: ParticipantRole,
}

/// 光标信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorInfo {
    /// 文档路径
    pub document_path: String,
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
    /// 选择范围
    pub selection: Option<(usize, usize, usize, usize)>,
}

/// 参与者角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantRole {
    /// 所有者
    Owner,
    /// 编辑者
    Editor,
    /// 查看者
    Viewer,
}

impl CollaborationSession {
    /// 创建新会话
    pub fn new(id: SessionId, name: String, creator: UserId) -> Self {
        let mut participants = HashMap::new();
        participants.insert(
            creator.clone(),
            ParticipantInfo {
                user: creator.clone(),
                joined_at: std::time::Instant::now(),
                cursor: None,
                online: true,
                role: ParticipantRole::Owner,
            },
        );

        Self {
            id,
            name,
            creator,
            participants,
            created_at: std::time::Instant::now(),
            document: CrdtDocument::new(),
        }
    }

    /// 添加参与者
    pub fn add_participant(&mut self, user: UserId, role: ParticipantRole) {
        self.participants.insert(
            user.clone(),
            ParticipantInfo {
                user,
                joined_at: std::time::Instant::now(),
                cursor: None,
                online: true,
                role,
            },
        );
    }

    /// 移除参与者
    pub fn remove_participant(&mut self, user: &UserId) {
        self.participants.remove(user);
    }

    /// 更新光标
    pub fn update_cursor(&mut self, user: &UserId, cursor: CursorInfo) {
        if let Some(info) = self.participants.get_mut(user) {
            info.cursor = Some(cursor);
        }
    }

    /// 获取在线参与者数量
    pub fn online_count(&self) -> usize {
        self.participants.values().filter(|p| p.online).count()
    }

    /// 获取参与者列表
    pub fn get_participants(&self) -> Vec<&ParticipantInfo> {
        self.participants.values().collect()
    }
}

// =============================================================================
// 协作管理器
// =============================================================================

/// 协作管理器
pub struct CollaborationManager {
    /// 活跃会话
    sessions: HashMap<SessionId, CollaborationSession>,
    /// 当前用户
    current_user: UserId,
    /// 网络客户端
    network: CollaborationNetwork,
}

impl CollaborationManager {
    /// 创建新的协作管理器
    pub fn new(current_user: UserId) -> Self {
        Self {
            sessions: HashMap::new(),
            current_user,
            network: CollaborationNetwork::new(),
        }
    }

    /// 创建新会话
    pub fn create_session(&mut self, name: String) -> SessionId {
        let id = SessionId::new(rand::random());
        let session = CollaborationSession::new(id, name.clone(), self.current_user.clone());

        self.sessions.insert(id, session);
        id
    }

    /// 加入会话
    pub fn join_session(
        &mut self,
        session_id: SessionId,
        role: ParticipantRole,
    ) -> Result<(), CollaborationError> {
        let session =
            self.sessions.get_mut(&session_id).ok_or(CollaborationError::SessionNotFound)?;

        session.add_participant(self.current_user.clone(), role);
        Ok(())
    }

    /// 离开会话
    pub fn leave_session(&mut self, session_id: SessionId) -> Result<(), CollaborationError> {
        let session =
            self.sessions.get_mut(&session_id).ok_or(CollaborationError::SessionNotFound)?;

        session.remove_participant(&self.current_user);
        Ok(())
    }

    /// 应用操作
    pub fn apply_operation(
        &mut self,
        session_id: SessionId,
        operation: CrdtOperation,
    ) -> Result<(), CollaborationError> {
        let session =
            self.sessions.get_mut(&session_id).ok_or(CollaborationError::SessionNotFound)?;

        session.document.apply(operation);
        Ok(())
    }

    /// 获取会话
    pub fn get_session(&self, session_id: SessionId) -> Option<&CollaborationSession> {
        self.sessions.get(&session_id)
    }

    /// 获取所有会话
    pub fn get_all_sessions(&self) -> Vec<&CollaborationSession> {
        self.sessions.values().collect()
    }

    /// 广播操作
    pub async fn broadcast_operation(
        &self,
        session_id: SessionId,
        operation: CrdtOperation,
    ) -> Result<(), CollaborationError> {
        let message = NetworkMessage::Operation {
            session_id,
            user_id: self.current_user.clone(),
            operation,
        };

        self.network.broadcast(message).await
    }
}

/// 协作错误
#[derive(Debug, Clone)]
pub enum CollaborationError {
    /// 会话不存在
    SessionNotFound,
    /// 权限不足
    PermissionDenied,
    /// 网络错误
    NetworkError(String),
    /// 操作冲突
    Conflict,
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for CollaborationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollaborationError::SessionNotFound => write!(f, "Session not found"),
            CollaborationError::PermissionDenied => write!(f, "Permission denied"),
            CollaborationError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            CollaborationError::Conflict => write!(f, "Operation conflict"),
            CollaborationError::Other(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl std::error::Error for CollaborationError {}

// =============================================================================
// 协作事件
// =============================================================================

/// 协作事件
#[derive(Debug, Clone)]
pub enum CollaborationEvent {
    /// 用户加入
    UserJoined {
        session_id: SessionId,
        user_id: UserId,
    },
    /// 用户离开
    UserLeft {
        session_id: SessionId,
        user_id: UserId,
    },
    /// 操作应用
    OperationApplied {
        session_id: SessionId,
        user_id: UserId,
        operation: CrdtOperation,
    },
    /// 光标移动
    CursorMoved {
        session_id: SessionId,
        user_id: UserId,
        cursor: CursorInfo,
    },
    /// 会话创建
    SessionCreated { session_id: SessionId, name: String },
    /// 同步状态变化
    SyncStatusChanged {
        session_id: SessionId,
        status: SyncStatus,
    },
}

impl DomainEvent for CollaborationEvent {
    fn event_type(&self) -> &'static str {
        match self {
            CollaborationEvent::UserJoined { .. } => "UserJoined",
            CollaborationEvent::UserLeft { .. } => "UserLeft",
            CollaborationEvent::OperationApplied { .. } => "OperationApplied",
            CollaborationEvent::CursorMoved { .. } => "CursorMoved",
            CollaborationEvent::SessionCreated { .. } => "SessionCreated",
            CollaborationEvent::SyncStatusChanged { .. } => "SyncStatusChanged",
        }
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

// =============================================================================
// ECS集成
// =============================================================================

/// 协作管理器资源
#[derive(Resource)]
pub struct CollaborationManagerResource {
    pub manager: CollaborationManager,
}

/// 会话组件
#[derive(Component, Debug, Clone)]
pub struct SessionComponent {
    pub session_id: SessionId,
    pub name: String,
    pub role: ParticipantRole,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let user = UserId::new("user1".to_string(), "Alice".to_string());
        let session =
            CollaborationSession::new(SessionId::new(1), "Test Session".to_string(), user.clone());

        assert_eq!(session.participants.len(), 1);
        assert_eq!(session.online_count(), 1);
    }

    #[test]
    fn test_add_participant() {
        let user1 = UserId::new("user1".to_string(), "Alice".to_string());
        let user2 = UserId::new("user2".to_string(), "Bob".to_string());

        let mut session =
            CollaborationSession::new(SessionId::new(1), "Test Session".to_string(), user1);

        session.add_participant(user2.clone(), ParticipantRole::Editor);
        assert_eq!(session.participants.len(), 2);
        assert_eq!(session.online_count(), 2);
    }

    #[test]
    fn test_manager_creation() {
        let user = UserId::new("user1".to_string(), "Alice".to_string());
        let manager = CollaborationManager::new(user);

        assert_eq!(manager.sessions.len(), 0);
    }

    #[test]
    fn test_create_session() {
        let user = UserId::new("user1".to_string(), "Alice".to_string());
        let mut manager = CollaborationManager::new(user);

        let session_id = manager.create_session("Test".to_string());
        assert!(manager.get_session(session_id).is_some());
    }
}
