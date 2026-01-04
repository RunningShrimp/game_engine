//! 文档同步
//!
//! 处理文档状态同步和冲突解决。

use super::{CrdtDocument, CrdtOperation, SessionId};

// =============================================================================
// 同步状态
// =============================================================================

/// 同步状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// 已同步
    Synced,
    /// 同步中
    Syncing,
    /// 离线
    Offline,
    /// 冲突
    Conflict,
    /// 错误
    Error,
}

// =============================================================================
// 文档同步
// =============================================================================

/// 文档同步器
pub struct DocumentSync {
    /// 文档
    document: CrdtDocument,
    /// 本地操作队列
    local_ops: Vec<CrdtOperation>,
    /// 远程操作队列
    remote_ops: Vec<CrdtOperation>,
    /// 同步状态
    status: SyncStatus,
    /// 最后同步时间
    last_sync: Option<std::time::Instant>,
}

impl DocumentSync {
    /// 创建新同步器
    pub fn new() -> Self {
        Self {
            document: CrdtDocument::new(),
            local_ops: Vec::new(),
            remote_ops: Vec::new(),
            status: SyncStatus::Synced,
            last_sync: None,
        }
    }

    /// 获取文档
    pub fn document(&self) -> &CrdtDocument {
        &self.document
    }

    /// 获取可变文档
    pub fn document_mut(&mut self) -> &mut CrdtDocument {
        &mut self.document
    }

    /// 应用本地操作
    pub fn apply_local(&mut self, operation: CrdtOperation) {
        self.local_ops.push(operation.clone());
        self.document.apply(operation);
        self.status = SyncStatus::Syncing;
    }

    /// 应用远程操作
    pub fn apply_remote(&mut self, operation: CrdtOperation) {
        self.remote_ops.push(operation.clone());
        self.document.apply(operation);
    }

    /// 同步文档
    pub fn sync(&mut self, remote_state: &CrdtDocument) -> Result<(), SyncError> {
        self.status = SyncStatus::Syncing;

        // 合并远程状态
        // 简化实现：直接应用所有操作
        for op in &self.remote_ops {
            self.document.apply(op.clone());
        }

        self.status = SyncStatus::Synced;
        self.last_sync = Some(std::time::Instant::now());
        self.local_ops.clear();
        self.remote_ops.clear();

        Ok(())
    }

    /// 获取同步状态
    pub fn status(&self) -> SyncStatus {
        self.status
    }

    /// 获取待同步操作数量
    pub fn pending_ops(&self) -> usize {
        self.local_ops.len()
    }

    /// 标记为离线
    pub fn mark_offline(&mut self) {
        self.status = SyncStatus::Offline;
    }

    /// 标记为错误
    pub fn mark_error(&mut self) {
        self.status = SyncStatus::Error;
    }
}

impl Default for DocumentSync {
    fn default() -> Self {
        Self::new()
    }
}

/// 同步错误
#[derive(Debug, Clone)]
pub enum SyncError {
    /// 网络错误
    Network(String),
    /// 序列化错误
    Serialization(String),
    /// 冲突
    Conflict,
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::Network(msg) => write!(f, "Network error: {msg}"),
            SyncError::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            SyncError::Conflict => write!(f, "Sync conflict"),
            SyncError::Other(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl std::error::Error for SyncError {}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_creation() {
        let sync = DocumentSync::new();
        assert_eq!(sync.status(), SyncStatus::Synced);
    }

    #[test]
    fn test_apply_local() {
        let mut sync = DocumentSync::new();

        let op = CrdtOperation::InsertText {
            position: 0,
            text: "Hello".to_string(),
            site_id: "site1".to_string(),
        };

        sync.apply_local(op);
        assert_eq!(sync.document().get_text(), "Hello");
        assert_eq!(sync.status(), SyncStatus::Syncing);
        assert_eq!(sync.pending_ops(), 1);
    }

    #[test]
    fn test_sync() {
        let mut sync = DocumentSync::new();

        sync.apply_local(CrdtOperation::InsertText {
            position: 0,
            text: "Hello".to_string(),
            site_id: "site1".to_string(),
        });

        let mut remote_doc = CrdtDocument::new();
        remote_doc.insert_text(0, "World".to_string(), "site2".to_string());

        let result = sync.sync(&remote_doc);
        assert!(result.is_ok());
        assert_eq!(sync.status(), SyncStatus::Synced);
        assert_eq!(sync.pending_ops(), 0);
    }
}
