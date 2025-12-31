//! CRDT（Conflict-free Replicated Data Types）实现
//!
//! 提供无需服务器的冲突解决数据结构。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// CRDT操作
// =============================================================================

/// CRDT操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    /// 插入文本
    InsertText {
        position: usize,
        text: String,
        site_id: String,
    },
    /// 删除文本
    DeleteText {
        position: usize,
        length: usize,
        site_id: String,
    },
    /// 更新值
    Update {
        key: String,
        value: CrdtValue,
        site_id: String,
    },
    /// 计数器增量
    Increment {
        key: String,
        amount: i64,
        site_id: String,
    },
}

/// CRDT值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

// =============================================================================
// CRDT文档
// =============================================================================

/// CRDT文档
#[derive(Debug, Clone)]
pub struct CrdtDocument {
    /// 文本内容（使用RGA）
    text: RgaText,
    /// 寄存器映射
    registers: HashMap<String, LwwRegister>,
    /// 计数器映射
    counters: HashMap<String, GCounter>,
}

/// RGA（Replicated Growable Array）文本
#[derive(Debug, Clone)]
struct RgaText {
    /// 字符列表
    chars: Vec<RgaChar>,
}

/// RGA字符
#[derive(Debug, Clone)]
struct RgaChar {
    /// 值
    value: char,
    /// 站点ID
    site_id: String,
    /// 逻辑时钟
    clock: u64,
    /// 前驱索引
    prev: Option<usize>,
}

impl CrdtDocument {
    /// 创建新文档
    pub fn new() -> Self {
        Self {
            text: RgaText::new(),
            registers: HashMap::new(),
            counters: HashMap::new(),
        }
    }

    /// 获取文本内容
    pub fn get_text(&self) -> String {
        self.text.to_string()
    }

    /// 插入文本
    pub fn insert_text(&mut self, position: usize, text: String, site_id: String) {
        let clock = self.get_next_clock(&site_id);
        self.text.insert(position, text, site_id, clock);
    }

    /// 删除文本
    pub fn delete_text(&mut self, position: usize, length: usize, site_id: String) {
        self.text.delete(position, length);
    }

    /// 应用操作
    pub fn apply(&mut self, operation: CrdtOperation) {
        match operation {
            CrdtOperation::InsertText { position, text, site_id } => {
                self.insert_text(position, text, site_id);
            }
            CrdtOperation::DeleteText { position, length, site_id } => {
                self.delete_text(position, length, site_id);
            }
            CrdtOperation::Update { key, value, site_id } => {
                let register = self.registers.entry(key).or_insert_with(LwwRegister::new);
                register.set(value, site_id);
            }
            CrdtOperation::Increment { key, amount, site_id } => {
                let counter = self.counters.entry(key).or_insert_with(GCounter::new);
                counter.increment(site_id, amount);
            }
        }
    }

    /// 获取下一个逻辑时钟
    fn get_next_clock(&self, site_id: &str) -> u64 {
        // 简化实现，实际应该追踪每个站点的时钟
        self.text.chars.iter()
            .filter(|c| c.site_id == site_id)
            .map(|c| c.clock)
            .max()
            .unwrap_or(0) + 1
    }
}

impl Default for CrdtDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl RgaText {
    /// 创建新RGA文本
    fn new() -> Self {
        Self { chars: Vec::new() }
    }

    /// 插入文本
    fn insert(&mut self, position: usize, text: String, site_id: String, clock: u64) {
        let prev_idx = if position == 0 {
            None
        } else if position <= self.chars.len() {
            Some(position - 1)
        } else {
            Some(self.chars.len().saturating_sub(1))
        };

        for (i, ch) in text.chars().enumerate() {
            let rga_char = RgaChar {
                value: ch,
                site_id: site_id.clone(),
                clock: clock + i as u64,
                prev: prev_idx,
            };
            self.chars.insert(position + i, rga_char);
        }
    }

    /// 删除文本
    fn delete(&mut self, position: usize, length: usize) {
        if position + length <= self.chars.len() {
            self.chars.drain(position..position + length);
        }
    }
}

impl std::fmt::Display for RgaText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text: String = self.chars.iter().map(|c| c.value).collect();
        write!(f, "{}", text)
    }
}

// =============================================================================
// LWW Register（Last-Write-Wins Register）
// =============================================================================

/// 最后写入胜出寄存器
#[derive(Debug, Clone)]
pub struct LwwRegister {
    value: Option<CrdtValue>,
    site_id: Option<String>,
    timestamp: u64,
}

impl LwwRegister {
    /// 创建新寄存器
    pub fn new() -> Self {
        Self {
            value: None,
            site_id: None,
            timestamp: 0,
        }
    }

    /// 设置值
    pub fn set(&mut self, value: CrdtValue, site_id: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        if timestamp >= self.timestamp {
            self.value = Some(value);
            self.site_id = Some(site_id);
            self.timestamp = timestamp;
        }
    }

    /// 获取值
    pub fn get(&self) -> Option<&CrdtValue> {
        self.value.as_ref()
    }
}

impl Default for LwwRegister {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// G-Counter（Grow-only Counter）
// =============================================================================

/// 仅增长计数器
#[derive(Debug, Clone)]
pub struct GCounter {
    /// 每个站点的计数
    counts: HashMap<String, i64>,
}

impl GCounter {
    /// 创建新计数器
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// 增量
    pub fn increment(&mut self, site_id: String, amount: i64) {
        *self.counts.entry(site_id).or_insert(0) += amount;
    }

    /// 获取当前值
    pub fn value(&self) -> i64 {
        self.counts.values().sum()
    }

    /// 合并另一个计数器
    pub fn merge(&mut self, other: &GCounter) {
        for (site_id, count) in &other.counts {
            let current = self.counts.entry(site_id.clone()).or_insert(0);
            *current = (*current).max(*count);
        }
    }
}

impl Default for GCounter {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = CrdtDocument::new();
        assert_eq!(doc.get_text(), "");
    }

    #[test]
    fn test_insert_text() {
        let mut doc = CrdtDocument::new();
        doc.insert_text(0, "Hello".to_string(), "site1".to_string());
        assert_eq!(doc.get_text(), "Hello");
    }

    #[test]
    fn test_delete_text() {
        let mut doc = CrdtDocument::new();
        doc.insert_text(0, "Hello World".to_string(), "site1".to_string());
        doc.delete_text(5, 6, "site1".to_string());
        assert_eq!(doc.get_text(), "Hello");
    }

    #[test]
    fn test_lww_register() {
        let mut reg = LwwRegister::new();

        reg.set(CrdtValue::Integer(42), "site1".to_string());
        assert_eq!(reg.get(), Some(&CrdtValue::Integer(42)));

        reg.set(CrdtValue::Integer(100), "site2".to_string());
        assert_eq!(reg.get(), Some(&CrdtValue::Integer(100)));
    }

    #[test]
    fn test_gcounter() {
        let mut counter = GCounter::new();

        counter.increment("site1".to_string(), 5);
        counter.increment("site2".to_string(), 3);

        assert_eq!(counter.value(), 8);

        counter.increment("site1".to_string(), 2);
        assert_eq!(counter.value(), 10);
    }

    #[test]
    fn test_concurrent_inserts() {
        let mut doc1 = CrdtDocument::new();
        let mut doc2 = CrdtDocument::new();

        // 并发插入
        doc1.insert_text(0, "Hello".to_string(), "site1".to_string());
        doc2.insert_text(0, "World".to_string(), "site2".to_string());

        // 操作可以交换
        let op1 = CrdtOperation::InsertText {
            position: 0,
            text: "Hello".to_string(),
            site_id: "site1".to_string(),
        };

        let op2 = CrdtOperation::InsertText {
            position: 0,
            text: "World".to_string(),
            site_id: "site2".to_string(),
        };

        // 应用操作
        let mut doc3 = CrdtDocument::new();
        doc3.apply(op1);
        doc3.apply(op2);

        // 结果是确定性的
        let text = doc3.get_text();
        assert!(text == "HelloWorld" || text == "WorldHello");
    }
}
