//! 技术债务管理工具
//!
//! 用于识别、跟踪和解决代码库中的技术债务。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use regex::Regex;

/// 技术债务管理器
pub struct TechnicalDebtManager {
    /// 项目根目录
    project_root: PathBuf,
    /// 债务项列表
    debt_items: Vec<DebtItem>,
    /// 债务统计
    statistics: DebtStatistics,
}

/// 债务项
#[derive(Debug, Clone)]
pub struct DebtItem {
    /// ID
    pub id: String,
    /// 文件路径
    pub file_path: PathBuf,
    /// 行号
    pub line_number: usize,
    /// 债务类型
    pub debt_type: DebtType,
    /// 优先级
    pub priority: Priority,
    /// 描述
    pub description: String,
    /// 建议修复方案
    pub suggested_fix: String,
    /// 预计工作量（人时）
    pub estimated_effort_hours: usize,
    /// 创建时间
    pub created_at: String,
    /// 状态
    pub status: DebtStatus,
}

/// 债务类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebtType {
    /// TODO 注释
    Todo,
    /// FIXME 注释
    Fixme,
    /// HACK 注释
    Hack,
    /// XXX 注释
    Xxx,
    /// 未使用的代码
    UnusedCode,
    /// 重复代码
    DuplicatedCode,
    /// 复杂度过高
    HighComplexity,
    /// 缺少测试
    MissingTests,
    /// 性能问题
    PerformanceIssue,
    /// 安全问题
    SecurityIssue,
    /// 代码风格问题
    StyleIssue,
    /// 其他
    Other,
}

/// 优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// P0 - 关键
    P0,
    /// P1 - 高
    P1,
    /// P2 - 中
    P2,
    /// P3 - 低
    P3,
}

/// 债务状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebtStatus {
    /// 未处理
    Open,
    /// 已计划
    Planned,
    /// 进行中
    InProgress,
    /// 已解决
    Resolved,
    /// 已忽略
    Ignored,
}

/// 债务统计
#[derive(Debug, Clone)]
pub struct DebtStatistics {
    /// 总债务数
    pub total_debts: usize,
    /// 按类型分类
    pub by_type: HashMap<DebtType, usize>,
    /// 按优先级分类
    pub by_priority: HashMap<Priority, usize>,
    /// 按状态分类
    pub by_status: HashMap<DebtStatus, usize>,
    /// 总预计工作量
    pub total_effort_hours: usize,
}

impl TechnicalDebtManager {
    /// 创建新的技术债务管理器
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            debt_items: Vec::new(),
            statistics: DebtStatistics {
                total_debts: 0,
                by_type: HashMap::new(),
                by_priority: HashMap::new(),
                by_status: HashMap::new(),
                total_effort_hours: 0,
            },
        }
    }

    /// 扫描所有技术债务
    pub fn scan_all_debt(&mut self) -> Result<usize, DebtError> {
        println!("开始扫描技术债务...");

        // 扫描源代码目录
        let src_dir = self.project_root.join("src");
        if src_dir.exists() {
            self.scan_directory(&src_dir)?;
        }

        // 扫描测试目录
        let tests_dir = self.project_root.join("tests");
        if tests_dir.exists() {
            self.scan_directory(&tests_dir)?;
        }

        // 扫描示例目录
        let examples_dir = self.project_root.join("examples");
        if examples_dir.exists() {
            self.scan_directory(&examples_dir)?;
        }

        // 计算统计信息
        self.calculate_statistics();

        println!("扫描完成！发现 {} 个技术债务", self.debt_items.len());

        Ok(self.debt_items.len())
    }

    /// 扫描目录中的所有文件
    fn scan_directory(&mut self, dir: &Path) -> Result<(), DebtError> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| DebtError::IoError(format!("无法读取目录 {}: {}", dir.display(), e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| DebtError::IoError(format!("无法读取目录项: {}", e)))?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.scan_file(&path)?;
            }
        }

        Ok(())
    }

    /// 扫描单个文件
    fn scan_file(&mut self, file_path: &Path) -> Result<(), DebtError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| DebtError::IoError(format!("无法读取文件 {}: {}", file_path.display(), e)))?;

        let lines: Vec<&str> = content.lines().collect();

        // 编译正则表达式
        let todo_regex = Regex::new(r"//\s*TODO(?:\(|:)?\s*(.+)").unwrap();
        let fixme_regex = Regex::new(r"//\s*FIXME(?:\(|:)?\s*(.+)").unwrap();
        let hack_regex = Regex::new(r"//\s*HACK(?:\(|:)?\s*(.+)").unwrap();
        let xxx_regex = Regex::new(r"//\s*XXX(?:\(|:)?\s*(.+)").unwrap();
        let unused_regex = Regex::new(r"#\[allow\(dead_code\)]").unwrap();
        let unimplemented_regex = Regex::new(r"unimplemented!\(\)").unwrap();
        let todo_macro_regex = Regex::new(r"todo!\(\)").unwrap();

        for (line_num, line) in lines.iter().enumerate() {
            let line_number = line_num + 1;

            // 检查 TODO
            if let Some(caps) = todo_regex.captures(line) {
                self.debt_items.push(DebtItem {
                    id: format!("todo-{:04}", self.debt_items.len()),
                    file_path: file_path.to_path_buf(),
                    line_number,
                    debt_type: DebtType::Todo,
                    priority: Self::infer_priority_from_description(caps.get(1).map(|m| m.as_str()).unwrap_or("")),
                    description: caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
                    suggested_fix: "实现待办事项".to_string(),
                    estimated_effort_hours: Self::estimate_effort(&caps.get(1).map(|m| m.as_str()).unwrap_or("")),
                    created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    status: DebtStatus::Open,
                });
            }

            // 检查 FIXME
            if let Some(caps) = fixme_regex.captures(line) {
                self.debt_items.push(DebtItem {
                    id: format!("fixme-{:04}", self.debt_items.len()),
                    file_path: file_path.to_path_buf(),
                    line_number,
                    debt_type: DebtType::Fixme,
                    priority: Priority::P1,
                    description: caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
                    suggested_fix: "修复标记的问题".to_string(),
                    estimated_effort_hours: 4,
                    created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    status: DebtStatus::Open,
                });
            }

            // 检查 HACK
            if let Some(caps) = hack_regex.captures(line) {
                self.debt_items.push(DebtItem {
                    id: format!("hack-{:04}", self.debt_items.len()),
                    file_path: file_path.to_path_buf(),
                    line_number,
                    debt_type: DebtType::Hack,
                    priority: Priority::P2,
                    description: caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
                    suggested_fix: "重构为更清晰的实现".to_string(),
                    estimated_effort_hours: 8,
                    created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    status: DebtStatus::Open,
                });
            }

            // 检查 XXX
            if let Some(caps) = xxx_regex.captures(line) {
                self.debt_items.push(DebtItem {
                    id: format!("xxx-{:04}", self.debt_items.len()),
                    file_path: file_path.to_path_buf(),
                    line_number,
                    debt_type: DebtType::Xxx,
                    priority: Priority::P2,
                    description: caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
                    suggested_fix: "改进标记的代码".to_string(),
                    estimated_effort_hours: 4,
                    created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    status: DebtStatus::Open,
                });
            }

            // 检查未使用的代码
            if unused_regex.is_match(line) {
                self.debt_items.push(DebtItem {
                    id: format!("unused-{:04}", self.debt_items.len()),
                    file_path: file_path.to_path_buf(),
                    line_number,
                    debt_type: DebtType::UnusedCode,
                    priority: Priority::P2,
                    description: "未使用的代码".to_string(),
                    suggested_fix: "移除未使用的代码或添加 pub 导出".to_string(),
                    estimated_effort_hours: 1,
                    created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    status: DebtStatus::Open,
                });
            }

            // 检查未实现
            if unimplemented_regex.is_match(line) {
                self.debt_items.push(DebtItem {
                    id: format!("unimplemented-{:04}", self.debt_items.len()),
                    file_path: file_path.to_path_buf(),
                    line_number,
                    debt_type: DebtType::Todo,
                    priority: Priority::P0,
                    description: "未实现的功能".to_string(),
                    suggested_fix: "实现该功能".to_string(),
                    estimated_effort_hours: 8,
                    created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    status: DebtStatus::Open,
                });
            }

            // 检查 todo 宏
            if todo_macro_regex.is_match(line) {
                self.debt_items.push(DebtItem {
                    id: format!("todo-macro-{:04}", self.debt_items.len()),
                    file_path: file_path.to_path_buf(),
                    line_number,
                    debt_type: DebtType::Todo,
                    priority: Priority::P0,
                    description: "使用 todo! 宏的未实现代码".to_string(),
                    suggested_fix: "实现该功能".to_string(),
                    estimated_effort_hours: 8,
                    created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    status: DebtStatus::Open,
                });
            }
        }

        Ok(())
    }

    /// 从描述推断优先级
    fn infer_priority_from_description(description: &str) -> Priority {
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("关键") || desc_lower.contains("crash") || desc_lower.contains("bug") {
            Priority::P0
        } else if desc_lower.contains("重要") || desc_lower.contains("urgent") {
            Priority::P1
        } else if desc_lower.contains("优化") || desc_lower.contains("改进") {
            Priority::P2
        } else {
            Priority::P3
        }
    }

    /// 估算工作量
    fn estimate_effort(description: &str) -> usize {
        if description.contains("实现") || description.contains("add") {
            8
        } else if description.contains("修复") || description.contains("fix") {
            4
        } else if description.contains("优化") || description.contains("optimize") {
            16
        } else if description.contains("重构") || description.contains("refactor") {
            24
        } else {
            2
        }
    }

    /// 计算统计信息
    fn calculate_statistics(&mut self) {
        let mut by_type = HashMap::new();
        let mut by_priority = HashMap::new();
        let mut by_status = HashMap::new();
        let mut total_effort = 0;

        for debt in &self.debt_items {
            *by_type.entry(debt.debt_type.clone()).or_insert(0) += 1;
            *by_priority.entry(debt.priority).or_insert(0) += 1;
            *by_status.entry(debt.status.clone()).or_insert(0) += 1;
            total_effort += debt.estimated_effort_hours;
        }

        self.statistics = DebtStatistics {
            total_debts: self.debt_items.len(),
            by_type,
            by_priority,
            by_status,
            total_effort_hours: total_effort,
        };
    }

    /// 获取债务项列表
    pub fn get_debts(&self) -> &[DebtItem] {
        &self.debt_items
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> &DebtStatistics {
        &self.statistics
    }

    /// 按优先级筛选债务
    pub fn filter_by_priority(&self, priority: Priority) -> Vec<&DebtItem> {
        self.debt_items
            .iter()
            .filter(|debt| debt.priority == priority)
            .collect()
    }

    /// 按类型筛选债务
    pub fn filter_by_type(&self, debt_type: DebtType) -> Vec<&DebtItem> {
        self.debt_items
            .iter()
            .filter(|debt| debt.debt_type == debt_type)
            .collect()
    }

    /// 按状态筛选债务
    pub fn filter_by_status(&self, status: DebtStatus) -> Vec<&DebtItem> {
        self.debt_items
            .iter()
            .filter(|debt| debt.status == status)
            .collect()
    }

    /// 获取高优先级债务（P0 和 P1）
    pub fn get_high_priority_debts(&self) -> Vec<&DebtItem> {
        self.debt_items
            .iter()
            .filter(|debt| debt.priority == Priority::P0 || debt.priority == Priority::P1)
            .collect()
    }

    /// 生成债务报告
    pub fn generate_report(&self) -> String {
        let mut report = String::from("# 技术债务报告\n\n");

        // 概览
        report.push_str(&format!("## 概览\n\n"));
        report.push_str(&format!("- 总债务数: {}\n", self.statistics.total_debts));
        report.push_str(&format!("- 总预计工作量: {} 人时\n\n", self.statistics.total_effort_hours));

        // 按优先级分类
        report.push_str("## 按优先级分类\n\n");
        for priority in &[Priority::P0, Priority::P1, Priority::P2, Priority::P3] {
            let count = self.statistics.by_priority.get(priority).unwrap_or(&0);
            report.push_str(&format!("- {:?}: {}\n", priority, count));
        }
        report.push_str("\n");

        // 按类型分类
        report.push_str("## 按类型分类\n\n");
        for (debt_type, count) in &self.statistics.by_type {
            report.push_str(&format!("- {:?}: {}\n", debt_type, count));
        }
        report.push_str("\n");

        // 高优先级债务详情
        report.push_str("## 高优先级债务 (P0 & P1)\n\n");
        for debt in self.get_high_priority_debts() {
            report.push_str(&format!(
                "### {}\n\n- **文件**: {}:{}\n- **类型**: {:?}\n- **描述**: {}\n- **建议**: {}\n- **工作量**: {} 人时\n\n",
                debt.id,
                debt.file_path.display(),
                debt.line_number,
                debt.debt_type,
                debt.description,
                debt.suggested_fix,
                debt.estimated_effort_hours
            ));
        }

        report
    }

    /// 导出债务列表到 CSV
    pub fn export_to_csv(&self, output_path: &Path) -> Result<(), DebtError> {
        let mut csv = String::from("ID,File,Line,Type,Priority,Description,Suggested Fix,Effort (hours)\n");

        for debt in &self.debt_items {
            csv.push_str(&format!(
                "{},{},{},{:?},{:?},\"{}\",\"{}\",{}\n",
                debt.id,
                debt.file_path.display(),
                debt.line_number,
                debt.debt_type,
                debt.priority,
                debt.description.replace("\"", "\"\""),
                debt.suggested_fix.replace("\"", "\"\""),
                debt.estimated_effort_hours
            ));
        }

        std::fs::write(output_path, csv)
            .map_err(|e| DebtError::IoError(format!("无法写入 CSV 文件: {}", e)))?;

        Ok(())
    }
}

/// 债务错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebtError {
    /// IO 错误
    IoError(String),
    /// 解析错误
    ParseError(String),
}

impl std::fmt::Display for DebtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebtError::IoError(msg) => write!(f, "IO error: {}", msg),
            DebtError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for DebtError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_inference() {
        assert_eq!(
            TechnicalDebtManager::infer_priority_from_description("修复关键bug"),
            Priority::P0
        );
        assert_eq!(
            TechnicalDebtManager::infer_priority_from_description("优化性能"),
            Priority::P2
        );
    }

    #[test]
    fn test_effort_estimation() {
        assert!(TechnicalDebtManager::estimate_effort("实现新功能") > 0);
        assert!(TechnicalDebtManager::estimate_effort("修复bug") > 0);
    }
}
