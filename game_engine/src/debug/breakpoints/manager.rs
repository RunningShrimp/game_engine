// 断点管理模块
//
// 管理调试断点的添加、删除、启用/禁用和命中检测

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 断点ID类型
pub type BreakpointId = i64;

/// 断点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakpointStatus {
    /// 未验证
    Unverified,
    /// 已验证
    Verified,
    /// 错误
    Error,
}

/// 断点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakpointType {
    /// 源码行断点
    Line,
    /// 函数断点
    Function,
    /// 异常断点
    Exception,
    /// 日志点
    Log,
}

/// 断点条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointCondition {
    /// 条件表达式
    pub expression: String,
    /// 命中次数
    pub hit_count: Option<i32>,
}

/// 断点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointInfo {
    /// 断点ID
    pub id: BreakpointId,
    /// 断点类型
    pub bp_type: BreakpointType,
    /// 源文件路径
    pub source_path: String,
    /// 行号
    pub line: i64,
    /// 列号（可选）
    pub column: Option<i64>,
    /// 函数名（用于函数断点）
    pub function_name: Option<String>,
    /// 状态
    pub status: BreakpointStatus,
    /// 是否启用
    pub enabled: bool,
    /// 条件
    pub condition: Option<BreakpointCondition>,
    /// 日志消息（用于日志点）
    pub log_message: Option<String>,
    /// 命中次数
    pub hit_count: u32,
    /// 创建时间
    pub created_at: u64,
    /// 最后命中时间
    pub last_hit_at: Option<u64>,
}

/// 断点管理器
pub struct BreakpointManager {
    /// 断点存储（key为 "file_path:line"）
    breakpoints: Arc<RwLock<HashMap<String, BreakpointInfo>>>,
    /// 下一个断点ID
    next_id: Arc<RwLock<BreakpointId>>,
    /// 断点命中回调
    hit_callback: Arc<RwLock<Option<Box<dyn Fn(BreakpointInfo) + Send + Sync>>>>,
}

impl BreakpointManager {
    /// 创建新的断点管理器
    pub fn new() -> Self {
        Self {
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            hit_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// 添加断点
    pub async fn add_breakpoint(
        &self,
        source_path: String,
        line: i64,
        bp_type: BreakpointType,
    ) -> BreakpointInfo {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;

        let key = format!("{}:{}", source_path, line);

        let bp = BreakpointInfo {
            id,
            bp_type,
            source_path: source_path.clone(),
            line,
            column: None,
            function_name: None,
            status: BreakpointStatus::Verified,
            enabled: true,
            condition: None,
            log_message: None,
            hit_count: 0,
            created_at: self.current_timestamp(),
            last_hit_at: None,
        };

        let mut breakpoints = self.breakpoints.write().await;
        breakpoints.insert(key.clone(), bp.clone());

        tracing::info!("Breakpoint added: {} at {}:{}", id, source_path, line);

        bp
    }

    /// 删除断点
    pub async fn remove_breakpoint(&self, source_path: &str, line: i64) -> bool {
        let key = format!("{}:{}", source_path, line);
        let mut breakpoints = self.breakpoints.write().await;

        if let Some(bp) = breakpoints.remove(&key) {
            tracing::info!("Breakpoint removed: {}", bp.id);
            true
        } else {
            false
        }
    }

    /// 根据ID删除断点
    pub async fn remove_breakpoint_by_id(&self, id: BreakpointId) -> bool {
        let mut breakpoints = self.breakpoints.write().await;

        let mut found_key = None;
        for (key, bp) in breakpoints.iter() {
            if bp.id == id {
                found_key = Some(key.clone());
                break;
            }
        }

        if let Some(key) = found_key {
            breakpoints.remove(&key);
            tracing::info!("Breakpoint removed by ID: {}", id);
            true
        } else {
            false
        }
    }

    /// 启用/禁用断点
    pub async fn set_breakpoint_enabled(
        &self,
        source_path: &str,
        line: i64,
        enabled: bool,
    ) -> bool {
        let key = format!("{}:{}", source_path, line);
        let mut breakpoints = self.breakpoints.write().await;

        if let Some(bp) = breakpoints.get_mut(&key) {
            bp.enabled = enabled;
            tracing::info!(
                "Breakpoint {}: {} at {}:{}",
                if enabled { "enabled" } else { "disabled" },
                bp.id,
                source_path,
                line
            );
            true
        } else {
            false
        }
    }

    /// 设置断点条件
    pub async fn set_breakpoint_condition(
        &self,
        source_path: &str,
        line: i64,
        condition: BreakpointCondition,
    ) -> bool {
        let key = format!("{}:{}", source_path, line);
        let mut breakpoints = self.breakpoints.write().await;

        if let Some(bp) = breakpoints.get_mut(&key) {
            bp.condition = Some(condition);
            tracing::info!(
                "Breakpoint condition set: {} at {}:{}",
                bp.id,
                source_path,
                line
            );
            true
        } else {
            false
        }
    }

    /// 检查断点是否应该触发
    pub async fn should_break(&self, source_path: &str, line: i64) -> Option<BreakpointInfo> {
        let key = format!("{}:{}", source_path, line);
        let breakpoints = self.breakpoints.read().await;

        if let Some(bp) = breakpoints.get(&key) {
            if !bp.enabled {
                return None;
            }

            // 检查条件
            if let Some(condition) = &bp.condition {
                // 评估条件表达式
                if let Some(eval_result) = Self::evaluate_condition(&condition.expression) {
                    if !eval_result {
                        tracing::debug!(
                            "Breakpoint condition false: {} at {}:{} (expression: {})",
                            bp.id,
                            source_path,
                            line,
                            condition.expression
                        );
                        return None;
                    }
                }

                // 检查命中次数
                if let Some(hit_count) = condition.hit_count {
                    if bp.hit_count < hit_count as u32 {
                        return None;
                    }
                }
            }

            Some(bp.clone())
        } else {
            None
        }
    }

    /// 评估条件表达式
    /// 返回Some(true)表示条件满足，Some(false)表示不满足，None表示无法评估
    fn evaluate_condition(expression: &str) -> Option<bool> {
        // 简化的条件表达式评估
        // 实际实现应该集成完整的表达式解析器

        let expression = expression.trim();

        // 处理布尔值
        match expression {
            "true" => return Some(true),
            "false" => return Some(false),
            _ => {}
        }

        // 处理比较表达式
        if expression.contains("==") {
            let parts: Vec<&str> = expression.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim();
                // 简化：直接比较字符串
                return Some(left == right);
            }
        }

        if expression.contains("!=") {
            let parts: Vec<&str> = expression.split("!=").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim();
                return Some(left != right);
            }
        }

        if expression.contains(">") {
            let parts: Vec<&str> = expression.split(">").collect();
            if parts.len() == 2 {
                if let (Ok(left), Ok(right)) = (
                    parts[0].trim().parse::<i64>(),
                    parts[1].trim().parse::<i64>(),
                ) {
                    return Some(left > right);
                }
            }
        }

        if expression.contains("<") {
            let parts: Vec<&str> = expression.split("<").collect();
            if parts.len() == 2 {
                if let (Ok(left), Ok(right)) = (
                    parts[0].trim().parse::<i64>(),
                    parts[1].trim().parse::<i64>(),
                ) {
                    return Some(left < right);
                }
            }
        }

        // 默认：如果无法评估，返回true（触发断点）
        tracing::warn!("Could not evaluate breakpoint condition: '{}'", expression);
        Some(true)
    }

    /// 断点命中
    pub async fn hit_breakpoint(&self, source_path: &str, line: i64) {
        let key = format!("{}:{}", source_path, line);
        let mut breakpoints = self.breakpoints.write().await;

        if let Some(bp) = breakpoints.get_mut(&key) {
            bp.hit_count += 1;
            bp.last_hit_at = Some(self.current_timestamp());

            tracing::info!(
                "Breakpoint hit: {} at {}:{} (count: {})",
                bp.id,
                source_path,
                line,
                bp.hit_count
            );

            // 调用回调
            let callback = self.hit_callback.read().await;
            if let Some(cb) = callback.as_ref() {
                cb(bp.clone());
            }
        }
    }

    /// 获取所有断点
    pub async fn get_all_breakpoints(&self) -> Vec<BreakpointInfo> {
        let breakpoints = self.breakpoints.read().await;
        breakpoints.values().cloned().collect()
    }

    /// 获取特定文件的断点
    pub async fn get_breakpoints_for_file(&self, source_path: &str) -> Vec<BreakpointInfo> {
        let breakpoints = self.breakpoints.read().await;
        breakpoints
            .values()
            .filter(|bp| bp.source_path == source_path)
            .cloned()
            .collect()
    }

    /// 清除所有断点
    pub async fn clear_all(&self) {
        let mut breakpoints = self.breakpoints.write().await;
        let count = breakpoints.len();
        breakpoints.clear();

        tracing::info!("Cleared {} breakpoints", count);
    }

    /// 设置命中回调
    pub async fn set_hit_callback<F>(&self, callback: F)
    where
        F: Fn(BreakpointInfo) + Send + Sync + 'static,
    {
        let mut cb = self.hit_callback.write().await;
        *cb = Some(Box::new(callback));
    }

    /// 获取断点统计
    pub async fn get_stats(&self) -> BreakpointStats {
        let breakpoints = self.breakpoints.read().await;

        let total = breakpoints.len();
        let enabled = breakpoints.values().filter(|bp| bp.enabled).count();
        let disabled = total - enabled;
        let verified = breakpoints
            .values()
            .filter(|bp| bp.status == BreakpointStatus::Verified)
            .count();
        let total_hits: u32 = breakpoints.values().map(|bp| bp.hit_count).sum();

        BreakpointStats {
            total,
            enabled,
            disabled,
            verified,
            total_hits,
        }
    }

    /// 获取当前时间戳（微秒）
    fn current_timestamp(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64
    }
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 断点统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointStats {
    /// 总断点数
    pub total: usize,
    /// 启用的断点数
    pub enabled: usize,
    /// 禁用的断点数
    pub disabled: usize,
    /// 已验证的断点数
    pub verified: usize,
    /// 总命中次数
    pub total_hits: u32,
}

/// 断点验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointValidationResult {
    /// 断点ID
    pub id: BreakpointId,
    /// 是否验证通过
    pub verified: bool,
    /// 错误消息（如果验证失败）
    pub error_message: Option<String>,
    /// 调整后的行号（如果需要）
    pub adjusted_line: Option<i64>,
}

/// 断点验证器
pub struct BreakpointValidator {
    /// 支持的源文件扩展名
    supported_extensions: Vec<String>,
    /// 源文件根路径
    source_roots: Vec<String>,
}

impl BreakpointValidator {
    /// 创建新的断点验证器
    pub fn new() -> Self {
        Self {
            supported_extensions: vec![
                ".lua".to_string(),
                ".ts".to_string(),
                ".js".to_string(),
                ".py".to_string(),
                ".rs".to_string(),
            ],
            source_roots: vec![
                "scripts/".to_string(),
                "src/".to_string(),
                "assets/".to_string(),
            ],
        }
    }

    /// 添加源文件根路径
    pub fn add_source_root(&mut self, root: String) {
        self.source_roots.push(root);
    }

    /// 验证断点
    pub fn validate(&self, bp: &BreakpointInfo) -> BreakpointValidationResult {
        // 检查源文件扩展名
        let has_valid_extension =
            self.supported_extensions.iter().any(|ext| bp.source_path.ends_with(ext));

        if !has_valid_extension {
            return BreakpointValidationResult {
                id: bp.id,
                verified: false,
                error_message: Some(format!(
                    "Unsupported file type: {}. Supported: {:?}",
                    bp.source_path, self.supported_extensions
                )),
                adjusted_line: None,
            };
        }

        // 检查行号
        if bp.line <= 0 {
            return BreakpointValidationResult {
                id: bp.id,
                verified: false,
                error_message: Some("Line number must be positive".to_string()),
                adjusted_line: None,
            };
        }

        // 验证源文件存在性
        match self.validate_source_file(&bp.source_path, bp.line) {
            Ok(Some(adjusted_line)) => BreakpointValidationResult {
                id: bp.id,
                verified: true,
                error_message: None,
                adjusted_line: Some(adjusted_line),
            },
            Ok(None) => BreakpointValidationResult {
                id: bp.id,
                verified: true,
                error_message: None,
                adjusted_line: None,
            },
            Err(e) => BreakpointValidationResult {
                id: bp.id,
                verified: false,
                error_message: Some(e),
                adjusted_line: None,
            },
        }
    }

    /// 验证源文件存在性和行数
    /// 返回Ok(Some(adjusted_line))如果需要调整行号
    /// 返回Ok(None)如果文件有效且无需调整
    /// 返回Err(message)如果验证失败
    fn validate_source_file(&self, source_path: &str, line: i64) -> Result<Option<i64>, String> {
        use std::path::{Path, PathBuf};

        // 尝试在源根目录中查找文件
        let mut found_path: Option<PathBuf> = None;

        // 如果路径是绝对路径，直接检查
        if Path::new(source_path).is_absolute() {
            if Path::new(source_path).exists() {
                found_path = Some(PathBuf::from(source_path));
            }
        } else {
            // 在源根目录中搜索
            for root in &self.source_roots {
                let full_path = PathBuf::from(root).join(source_path);
                if full_path.exists() {
                    found_path = Some(full_path);
                    break;
                }
            }
        }

        // 如果找到了文件，检查行数
        if let Some(path) = found_path {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let line_count = content.lines().count() as i64;

                    if line > line_count {
                        Err(format!(
                            "Line {} exceeds file length ({})",
                            line, line_count
                        ))
                    } else {
                        // 文件有效，无需调整行号
                        Ok(None)
                    }
                }
                Err(e) => Err(format!("Failed to read file: {}", e)),
            }
        } else {
            // 文件不存在，但可能是动态生成的脚本
            // 在开发模式下，我们允许设置断点，运行时会验证
            tracing::warn!(
                "Source file not found: {} (breakpoint will be verified at runtime)",
                source_path
            );
            Ok(None)
        }
    }

    /// 批量验证断点
    pub fn validate_batch(
        &self,
        breakpoints: &[BreakpointInfo],
    ) -> Vec<BreakpointValidationResult> {
        breakpoints.iter().map(|bp| self.validate(bp)).collect()
    }

    /// 获取支持的文件扩展名
    pub fn supported_extensions(&self) -> &[String] {
        &self.supported_extensions
    }

    /// 添加支持的文件扩展名
    pub fn add_supported_extension(&mut self, ext: String) {
        if !ext.starts_with('.') {
            self.supported_extensions.push(format!(".{}", ext));
        } else {
            self.supported_extensions.push(ext);
        }
    }
}

impl Default for BreakpointValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_breakpoint() {
        let manager = BreakpointManager::new();

        let bp = manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        assert_eq!(bp.line, 42);
        assert_eq!(bp.bp_type, BreakpointType::Line);
        assert!(bp.enabled);
        assert_eq!(bp.status, BreakpointStatus::Verified);
    }

    #[tokio::test]
    async fn test_remove_breakpoint() {
        let manager = BreakpointManager::new();

        manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        let removed = manager.remove_breakpoint("/path/to/file.lua", 42).await;
        assert!(removed);

        let removed_again = manager.remove_breakpoint("/path/to/file.lua", 42).await;
        assert!(!removed_again);
    }

    #[tokio::test]
    async fn test_enable_disable_breakpoint() {
        let manager = BreakpointManager::new();

        manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        // 禁用
        let success = manager.set_breakpoint_enabled("/path/to/file.lua", 42, false).await;
        assert!(success);

        let breakpoints = manager.get_all_breakpoints().await;
        assert!(!breakpoints[0].enabled);

        // 启用
        let success = manager.set_breakpoint_enabled("/path/to/file.lua", 42, true).await;
        assert!(success);

        let breakpoints = manager.get_all_breakpoints().await;
        assert!(breakpoints[0].enabled);
    }

    #[tokio::test]
    async fn test_should_break() {
        let manager = BreakpointManager::new();

        manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        let should_break = manager.should_break("/path/to/file.lua", 42).await;
        assert!(should_break.is_some());

        let should_not_break = manager.should_break("/path/to/file.lua", 43).await;
        assert!(should_not_break.is_none());
    }

    #[tokio::test]
    async fn test_disabled_breakpoint_doesnt_trigger() {
        let manager = BreakpointManager::new();

        manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        manager.set_breakpoint_enabled("/path/to/file.lua", 42, false).await;

        let should_break = manager.should_break("/path/to/file.lua", 42).await;
        assert!(should_break.is_none());
    }

    #[tokio::test]
    async fn test_hit_breakpoint() {
        let manager = BreakpointManager::new();

        manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        manager.hit_breakpoint("/path/to/file.lua", 42).await;
        manager.hit_breakpoint("/path/to/file.lua", 42).await;

        let breakpoints = manager.get_all_breakpoints().await;
        assert_eq!(breakpoints[0].hit_count, 2);
    }

    #[tokio::test]
    async fn test_get_breakpoints_for_file() {
        let manager = BreakpointManager::new();

        manager
            .add_breakpoint("/path/to/file1.lua".to_string(), 10, BreakpointType::Line)
            .await;
        manager
            .add_breakpoint("/path/to/file1.lua".to_string(), 20, BreakpointType::Line)
            .await;
        manager
            .add_breakpoint("/path/to/file2.lua".to_string(), 30, BreakpointType::Line)
            .await;

        let file1_bps = manager.get_breakpoints_for_file("/path/to/file1.lua").await;
        assert_eq!(file1_bps.len(), 2);

        let file2_bps = manager.get_breakpoints_for_file("/path/to/file2.lua").await;
        assert_eq!(file2_bps.len(), 1);
    }

    #[tokio::test]
    async fn test_breakpoint_stats() {
        let manager = BreakpointManager::new();

        manager
            .add_breakpoint("/path/to/file1.lua".to_string(), 10, BreakpointType::Line)
            .await;
        manager
            .add_breakpoint("/path/to/file2.lua".to_string(), 20, BreakpointType::Line)
            .await;

        manager.set_breakpoint_enabled("/path/to/file1.lua", 10, false).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.enabled, 1);
        assert_eq!(stats.disabled, 1);
    }

    #[test]
    fn test_validator_valid_breakpoint() {
        let validator = BreakpointValidator::new();

        let bp = BreakpointInfo {
            id: 1,
            bp_type: BreakpointType::Line,
            source_path: "/path/to/file.lua".to_string(),
            line: 42,
            column: None,
            function_name: None,
            status: BreakpointStatus::Unverified,
            enabled: true,
            condition: None,
            log_message: None,
            hit_count: 0,
            created_at: 0,
            last_hit_at: None,
        };

        let result = validator.validate(&bp);
        assert!(result.verified);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_validator_invalid_extension() {
        let validator = BreakpointValidator::new();

        let bp = BreakpointInfo {
            id: 1,
            bp_type: BreakpointType::Line,
            source_path: "/path/to/file.txt".to_string(),
            line: 42,
            column: None,
            function_name: None,
            status: BreakpointStatus::Unverified,
            enabled: true,
            condition: None,
            log_message: None,
            hit_count: 0,
            created_at: 0,
            last_hit_at: None,
        };

        let result = validator.validate(&bp);
        assert!(!result.verified);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_validator_with_source_roots() {
        let mut validator = BreakpointValidator::new();
        validator.add_source_root("test/".to_string());

        let bp = BreakpointInfo {
            id: 1,
            bp_type: BreakpointType::Line,
            source_path: "script.lua".to_string(),
            line: 10,
            column: None,
            function_name: None,
            status: BreakpointStatus::Unverified,
            enabled: true,
            condition: None,
            log_message: None,
            hit_count: 0,
            created_at: 0,
            last_hit_at: None,
        };

        let result = validator.validate(&bp);
        // 文件不存在但应该有警告而不是错误
        assert!(result.verified);
    }

    #[test]
    fn test_evaluate_condition_true() {
        // 测试 true 条件
        assert_eq!(BreakpointManager::evaluate_condition("true"), Some(true));
    }

    #[test]
    fn test_evaluate_condition_false() {
        // 测试 false 条件
        assert_eq!(BreakpointManager::evaluate_condition("false"), Some(false));
    }

    #[test]
    fn test_evaluate_condition_equals() {
        // 测试相等条件
        assert_eq!(BreakpointManager::evaluate_condition("x == x"), Some(true));
        assert_eq!(BreakpointManager::evaluate_condition("x == y"), Some(false));
    }

    #[test]
    fn test_evaluate_condition_not_equals() {
        // 测试不等条件
        assert_eq!(BreakpointManager::evaluate_condition("x != y"), Some(true));
        assert_eq!(BreakpointManager::evaluate_condition("x != x"), Some(false));
    }

    #[test]
    fn test_evaluate_condition_greater_than() {
        // 测试大于条件
        assert_eq!(BreakpointManager::evaluate_condition("10 > 5"), Some(true));
        assert_eq!(BreakpointManager::evaluate_condition("5 > 10"), Some(false));
    }

    #[test]
    fn test_evaluate_condition_less_than() {
        // 测试小于条件
        assert_eq!(BreakpointManager::evaluate_condition("5 < 10"), Some(true));
        assert_eq!(BreakpointManager::evaluate_condition("10 < 5"), Some(false));
    }

    #[tokio::test]
    async fn test_conditional_breakpoint() {
        let manager = BreakpointManager::new();

        // 添加带条件的断点
        let bp = manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        // 设置条件
        let condition = BreakpointCondition {
            expression: "true".to_string(),
            hit_count: None,
        };
        manager.set_breakpoint_condition("/path/to/file.lua", 42, condition).await;

        // 条件满足，应该触发
        let should_break = manager.should_break("/path/to/file.lua", 42).await;
        assert!(should_break.is_some());
    }

    #[tokio::test]
    async fn test_conditional_breakpoint_false() {
        let manager = BreakpointManager::new();

        // 添加带条件的断点
        let bp = manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        // 设置false条件
        let condition = BreakpointCondition {
            expression: "false".to_string(),
            hit_count: None,
        };
        manager.set_breakpoint_condition("/path/to/file.lua", 42, condition).await;

        // 条件不满足，不应该触发
        let should_break = manager.should_break("/path/to/file.lua", 42).await;
        assert!(should_break.is_none());
    }

    #[tokio::test]
    async fn test_hit_count_condition() {
        let manager = BreakpointManager::new();

        // 添加带命中次数条件的断点
        let bp = manager
            .add_breakpoint("/path/to/file.lua".to_string(), 42, BreakpointType::Line)
            .await;

        // 设置命中次数为3
        let condition = BreakpointCondition {
            expression: "true".to_string(),
            hit_count: Some(3),
        };
        manager.set_breakpoint_condition("/path/to/file.lua", 42, condition).await;

        // 前两次不应该触发
        assert!(manager.should_break("/path/to/file.lua", 42).await.is_none());
        assert!(manager.should_break("/path/to/file.lua", 42).await.is_none());

        // 模拟命中
        manager.hit_breakpoint("/path/to/file.lua", 42).await;
        manager.hit_breakpoint("/path/to/file.lua", 42).await;
        manager.hit_breakpoint("/path/to/file.lua", 42).await;

        // 第三次应该触发
        assert!(manager.should_break("/path/to/file.lua", 42).await.is_some());
    }

    #[tokio::test]
    async fn test_validator_supported_extensions() {
        let validator = BreakpointValidator::new();
        let extensions = validator.supported_extensions();

        assert!(extensions.contains(&".lua".to_string()));
        assert!(extensions.contains(&".ts".to_string()));
        assert!(extensions.contains(&".js".to_string()));
    }

    #[tokio::test]
    async fn test_validator_add_extension() {
        let mut validator = BreakpointValidator::new();
        validator.add_supported_extension("cpp".to_string());

        let extensions = validator.supported_extensions();
        assert!(extensions.contains(&".cpp".to_string()));
    }
}
