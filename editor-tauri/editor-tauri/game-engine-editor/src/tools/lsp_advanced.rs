//! P2-1: LSP高级功能扩展
//!
//! 提供代码重构、语义分析、代码质量检查等高级LSP功能

use tower_lsp::lsp_types::*;
use std::collections::HashMap;

/// 代码重构引擎
pub struct RefactoringEngine {
    /// 可用的重构操作
    operations: Vec<RefactoringOperation>,
}

#[derive(Debug, Clone)]
pub struct RefactoringOperation {
    pub name: String,
    pub description: String,
    pub kind: RefactoringKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefactoringKind {
    /// 提取方法
    ExtractMethod,
    /// 重命名符号
    RenameSymbol,
    /// 内联变量
    InlineVariable,
    /// 提取变量
    ExtractVariable,
    /// 移动代码
    MoveCode,
    /// 代码清理
    CodeCleanup,
}

impl RefactoringEngine {
    pub fn new() -> Self {
        Self {
            operations: vec![
                RefactoringOperation {
                    name: "提取方法".to_string(),
                    description: "将选中的代码块提取为新方法".to_string(),
                    kind: RefactoringKind::ExtractMethod,
                },
                RefactoringOperation {
                    name: "重命名符号".to_string(),
                    description: "重命名变量、函数或类型".to_string(),
                    kind: RefactoringKind::RenameSymbol,
                },
                RefactoringOperation {
                    name: "内联变量".to_string(),
                    description: "内联临时变量到其使用位置".to_string(),
                    kind: RefactoringKind::InlineVariable,
                },
            ],
        }
    }

    /// 分析代码重构机会
    pub fn analyze_refactoring_opportunities(
        &self,
        code: &str,
        uri: &str,
    ) -> Vec<RefactoringSuggestion> {
        let mut suggestions = Vec::new();

        // 实现代码分析（简化版本）
        // 分析代码并提供重构建议

        // 检测重复代码
        if self.has_duplicate_code(code) {
            suggestions.push(RefactoringSuggestion {
                kind: RefactoringKind::ExtractMethod,
                message: "检测到重复代码块，建议提取为方法".to_string(),
                range: Range::default(),
                priority: SuggestionPriority::Medium,
            });
        }

        // 检测魔法数字
        let magic_numbers = self.find_magic_numbers(code);
        if !magic_numbers.is_empty() {
            suggestions.push(RefactoringSuggestion {
                kind: RefactoringKind::ExtractVariable,
                message: format!("检测到{}个魔法数字，建议提取为常量", magic_numbers.len()),
                range: Range::default(),
                priority: SuggestionPriority::Low,
            });
        }

        // 检测长函数
        if self.has_long_function(code) {
            suggestions.push(RefactoringSuggestion {
                kind: RefactoringKind::ExtractMethod,
                message: "函数过长，建议拆分为多个小函数".to_string(),
                range: Range::default(),
                priority: SuggestionPriority::Medium,
            });
        }

        // 检测复杂度
        if self.has_high_complexity(code) {
            suggestions.push(RefactoringSuggestion {
                kind: RefactoringKind::Simplify,
                message: "代码复杂度较高，建议简化逻辑".to_string(),
                range: Range::default(),
                priority: SuggestionPriority::High,
            });
        }

        suggestions
    }

    /// 查找代码中的魔法数字
    fn find_magic_numbers(&self, code: &str) -> Vec<String> {
        let mut magic_numbers = Vec::new();

        // 简化的魔法数字检测：查找独立的数字字面量
        // 排除常见的合理数字（0, 1, 2, 100等）
        let number_regex = regex::Regex::new(r"(?<![a-zA-Z0-9_])([3-9]|[1-9]\d+)(?![a-zA-Z0-9_])").unwrap();

        for caps in number_regex.captures_iter(code) {
            if let Some(num) = caps.get(1) {
                let num_str = num.as_str();
                // 排除一些常见情况
                if !self.is_common_number(num_str) {
                    magic_numbers.push(num_str.to_string());
                }
            }
        }

        magic_numbers
    }

    /// 判断是否是常见数字（不需要提取为常量）
    fn is_common_number(&self, num: &str) -> bool {
        // 常见阈值：0, 1, 2, 10, 100, 1000等
        matches!(num, "0" | "1" | "2" | "10" | "100" | "1000")
    }

    /// 检测是否有长函数
    fn has_long_function(&self, code: &str) -> bool {
        // 简化检测：如果函数超过50行，认为过长
        let lines: Vec<&str> = code.lines().collect();
        lines.len() > 50
    }

    /// 检测是否有高复杂度
    fn has_high_complexity(&self, code: &str) -> bool {
        // 简化的复杂度检测：计算控制流语句数量
        let if_count = code.matches("if ").count();
        let match_count = code.matches("match ").count();
        let loop_count = code.matches("for ").count() + code.matches("while ").count();
        let continue_count = code.matches("continue ").count();

        let complexity_score = if_count + match_count * 2 + loop_count * 2 + continue_count;
        complexity_score > 10  // 复杂度阈值
    }

    fn has_duplicate_code(&self, code: &str) -> bool {
        // 简化实现：检测重复的代码行
        let lines: Vec<&str> = code.lines().collect();
        let mut seen = std::collections::HashSet::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.len() > 20 && !seen.insert(trimmed) {
                return true;
            }
        }

        false
    }

    fn has_magic_numbers(&self, code: &str) -> bool {
        // 简化实现：检测字面数字
        code.lines()
            .any(|line| line.contains(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'][..]))
    }
}

#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub kind: RefactoringKind,
    pub message: String,
    pub range: Range,
    pub priority: SuggestionPriority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionPriority {
    High,
    Medium,
    Low,
}

/// 代码质量分析器
pub struct CodeQualityAnalyzer;

impl CodeQualityAnalyzer {
    /// 分析代码质量
    pub fn analyze(&self, code: &str, uri: &str) -> CodeQualityReport {
        let lines = code.lines().count();
        let functions = self.count_functions(code);
        let complexity = self.calculate_complexity(code);
        let coverage = self.estimate_coverage(code);

        CodeQualityReport {
            uri: uri.to_string(),
            total_lines: lines,
            function_count: functions,
            cyclomatic_complexity: complexity,
            code_coverage: coverage,
            issues: self.find_issues(code),
            metrics: self.calculate_metrics(code),
        }
    }

    fn count_functions(&self, code: &str) -> usize {
        code.matches("fn ").count() + code.matches("async fn ").count()
    }

    fn calculate_complexity(&self, code: &str) -> f64 {
        // 简化的圈复杂度计算
        let if_count = code.matches("if ").count() as f64;
        let for_count = code.matches("for ").count() as f64;
        let while_count = code.matches("while ").count() as f64;
        let match_count = code.matches("match ").count() as f64;

        1.0 + if_count + for_count + while_count + match_count * 2.0
    }

    fn estimate_coverage(&self, code: &str) -> f64 {
        // 基于测试文件估计覆盖率
        let has_tests = code.contains("#[test]") || code.contains("#[cfg(test)]");
        if has_tests {
            60.0 // 有测试的文件估计60%
        } else {
            30.0 // 没有测试的文件估计30%
        }
    }

    fn find_issues(&self, code: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        // 检测过长函数
        for (i, line) in code.lines().enumerate() {
            if line.len() > 100 {
                issues.push(Issue {
                    severity: IssueSeverity::Warning,
                    line: i as u32,
                    message: format!("行过长 ({} 字符)", line.len()),
                    suggestion: "考虑拆分为多行".to_string(),
                });
            }
        }

        issues
    }

    fn calculate_metrics(&self, code: &str) -> CodeMetrics {
        let lines = code.lines().count();
        let blank_lines = code.lines().filter(|l| l.is_empty()).count();
        let comment_lines = code.lines().filter(|l| l.trim().starts_with("//")).count();

        CodeMetrics {
            lines_of_code: lines - blank_lines - comment_lines,
            blank_lines,
            comment_lines,
            comment_ratio: if lines > 0 {
                comment_lines as f64 / lines as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeQualityReport {
    pub uri: String,
    pub total_lines: usize,
    pub function_count: usize,
    pub cyclomatic_complexity: f64,
    pub code_coverage: f64,
    pub issues: Vec<Issue>,
    pub metrics: CodeMetrics,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub line: u32,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub comment_ratio: f64,
}

/// 代码依赖分析器
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// 创建新的依赖分析器
    pub fn new() -> Self {
        Self
    }

    /// 分析模块依赖
    pub fn analyze_dependencies(&self, code: &str) -> DependencyGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut module_name = String::from("current");

        // 提取模块声明
        if let Some(mod_line) = code.lines().find(|l| l.starts_with("mod ")) {
            if let Some(name) = mod_line.strip_prefix("mod ") {
                module_name = name.trim_end_matches('{').trim().to_string();
            }
        }

        // 添加当前模块节点
        nodes.push(DependencyNode {
            name: module_name.clone(),
            kind: DependencyKind::Module,
        });

        // 解析use语句
        let use_regex = regex::Regex::new(r"use\s+([^;]+);").unwrap();
        let mut deps = std::collections::HashSet::new();

        for caps in use_regex.captures_iter(code) {
            if let Some(dep_path) = caps.get(1) {
                let dep = dep_path.as_str().trim();
                deps.insert(dep.to_string());

                // 添加依赖节点
                let dep_kind = if dep.contains("::") {
                    DependencyKind::Item
                } else {
                    DependencyKind::Module
                };

                nodes.push(DependencyNode {
                    name: dep.to_string(),
                    kind: dep_kind,
                });

                // 添加依赖边
                edges.push(DependencyEdge {
                    from: module_name.clone(),
                    to: dep.to_string(),
                    kind: DependencyEdgeKind::Uses,
                });
            }
        }

        // 解析extern crate语句
        let extern_regex = regex::Regex::new(r"extern\s+crate\s+(\w+);").unwrap();
        for caps in extern_regex.captures_iter(code) {
            if let Some(crate_name) = caps.get(1) {
                let crate_name_str = crate_name.as_str().to_string();

                nodes.push(DependencyNode {
                    name: crate_name_str.clone(),
                    kind: DependencyKind::ExternalCrate,
                });

                edges.push(DependencyEdge {
                    from: module_name.clone(),
                    to: crate_name_str,
                    kind: DependencyEdgeKind::ExternCrate,
                });
            }
        }

        DependencyGraph { nodes, edges }
    }

    /// 分析未使用的依赖
    pub fn find_unused_dependencies(&self, code: &str) -> Vec<String> {
        let mut unused = Vec::new();

        // 收集所有use声明
        let use_regex = regex::Regex::new(r"use\s+([^;]+);").unwrap();
        let mut declared_deps = std::collections::HashSet::new();

        for caps in use_regex.captures_iter(code) {
            if let Some(dep) = caps.get(1) {
                declared_deps.insert(dep.as_str().to_string());
            }
        }

        // 检查每个依赖是否在代码中被使用
        for dep in &declared_deps {
            // 提取依赖的最后一部分（实际使用的名称）
            let name = if dep.contains("::") {
                dep.split("::").last().unwrap_or(dep)
            } else {
                dep
            };

            // 检查是否在代码中使用
            let usage_pattern = format!(r"\b{}\b", regex::escape(name));
            let usage_regex = regex::Regex::new(&usage_pattern).unwrap();

            // 排除use声明本身
            let code_without_uses = use_regex.replace_all(code, "");

            if !usage_regex.is_match(&code_without_uses) {
                unused.push(dep.clone());
            }
        }

        unused
    }
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub name: String,
    pub kind: DependencyKind,
}

#[derive(Debug, Clone)]
pub enum DependencyKind {
    Module,
    Struct,
    Enum,
    Function,
    Trait,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub kind: DependencyKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactoring_engine() {
        let engine = RefactoringEngine::new();
        assert_eq!(engine.operations.len(), 3);
    }

    #[test]
    fn test_code_quality_analyzer() {
        let analyzer = CodeQualityAnalyzer;
        let code = r#"
fn main() {
    println!("Hello");
    println!("World");
}
"#;

        let report = analyzer.analyze(code, "test.rs");
        assert_eq!(report.function_count, 1);
    }
}
