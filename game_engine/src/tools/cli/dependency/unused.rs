//! # 未使用依赖检测
//!
//! 分析代码引用，检测未被使用的依赖项。
//!
//! ## 功能
//!
//! - 扫描Rust源代码中的extern crate和use语句
//! - 检测未在代码中使用的依赖
//! - 区分正常依赖、开发依赖和构建依赖
//! - 提供安全的移除建议
//!
//! ## 使用示例
//!
//! ```no_run
//! use game_engine::tools::cli::dependency::unused::UnusedDetector;
//! use game_engine::tools::cli::dependency::graph::DependencyGraph;
//!
//! let graph = DependencyGraph::from_project(".").unwrap();
//! let detector = UnusedDetector::new(&graph);
//! let unused = detector.detect_unused_dependencies();
//!
//! for dep in unused {
//!     println!("Unused: {} ({})", dep.name, dep.kind);
//! }
//! ```

use super::graph::{Dependency, DependencyGraph, DependencyKind};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// 未使用依赖检测器
#[derive(Debug, Clone)]
pub struct UnusedDetector<'a> {
    /// 依赖图引用
    graph: &'a DependencyGraph,

    /// 扫描的源代码文件
    source_files: Vec<PathBuf>,

    /// 扫描到的外部crate引用
    external_crates: HashSet<String>,
}

impl<'a> UnusedDetector<'a> {
    /// 创建新的未使用依赖检测器
    pub fn new(graph: &'a DependencyGraph) -> Self {
        let mut detector = Self {
            graph,
            source_files: Vec::new(),
            external_crates: HashSet::new(),
        };

        // 扫描源代码文件
        detector.scan_source_files();

        detector
    }

    /// 扫描项目中的所有Rust源代码文件
    fn scan_source_files(&mut self) {
        let src_dir = self.graph.project_root.join("src");

        if src_dir.exists() {
            self.scan_directory(&src_dir);
        }

        // 扫描examples
        let examples_dir = self.graph.project_root.join("examples");
        if examples_dir.exists() {
            self.scan_directory(&examples_dir);
        }

        // 扫描tests
        let tests_dir = self.graph.project_root.join("tests");
        if tests_dir.exists() {
            self.scan_directory(&tests_dir);
        }

        // 扫描benches
        let benches_dir = self.graph.project_root.join("benches");
        if benches_dir.exists() {
            self.scan_directory(&benches_dir);
        }
    }

    /// 递归扫描目录中的所有.rs文件
    fn scan_directory(&mut self, dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();

                if path.is_dir() {
                    self.scan_directory(&path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    self.source_files.push(path);
                }
            }
        }
    }

    /// 检测未使用的依赖
    pub fn detect_unused_dependencies(&self) -> Vec<UnusedDependency> {
        let mut unused = Vec::new();
        let mut external_crates = HashSet::new();

        // 分析每个源文件，提取使用的外部crate
        for source_file in &self.source_files {
            if let Ok(content) = fs::read_to_string(source_file) {
                self.extract_external_crates_from_content(&content, &mut external_crates);
            }
        }

        // 检查每个依赖是否被使用
        for (dep_name, dependencies) in &self.graph.dependencies {
            for dep in dependencies {
                // 跳过标准库和内部crate
                if self.is_std_or_internal(&dep.name) {
                    continue;
                }

                // 检查是否在代码中使用
                if !self.is_crate_used(&dep.name, &external_crates) {
                    unused.push(UnusedDependency {
                        name: dep.name.clone(),
                        version: dep.version_req.clone(),
                        kind: dep.kind,
                        optional: dep.optional,
                        reason: UnusedReason::NotReferenced,
                        safe_to_remove: true,
                    });
                }
            }
        }

        unused
    }

    /// 提取源代码中使用的外部crate
    fn extract_external_crates_from_content(
        &self,
        content: &str,
        external_crates: &mut HashSet<String>,
    ) {
        // 提取extern crate语句
        for line in content.lines() {
            let line = line.trim();

            // 匹配 extern crate foo;
            if let Some(crate_name) = line.strip_prefix("extern crate ") {
                let crate_name = crate_name.trim_end_matches(';').trim();

                // 处理重命名：extern crate foo as bar
                let actual_name = if let Some(as_pos) = crate_name.find(" as ") {
                    &crate_name[..as_pos]
                } else {
                    crate_name
                };

                external_crates.insert(actual_name.to_string());
            }
        }

        // 提取use语句中的crate名称
        for line in content.lines() {
            let line = line.trim();

            if let Some(path) = line.strip_prefix("use ") {
                // 提取use语句中的路径
                let path = path.trim_end_matches(';').trim();

                // 提取第一段（通常是crate名）
                if let Some(first_segment) = path.split("::").next() {
                    // 跳过self, super, crate等
                    if !matches!(first_segment, "self" | "super" | "crate") {
                        external_crates.insert(first_segment.to_string());
                    }
                }
            }
        }

        // 检查属性中的crate引用（如#[cfg(feature = "foo")]）
        for line in content.lines() {
            if line.contains("::") {
                // 提取路径中的crate名
                for segment in line.split("::") {
                    let segment =
                        segment.trim().trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if !segment.is_empty()
                        && !matches!(
                            segment,
                            "self" | "super" | "crate" | "std" | "core" | "alloc"
                        )
                    {
                        external_crates.insert(segment.to_string());
                    }
                }
            }
        }
    }

    /// 检查是否为标准库或内部crate
    fn is_std_or_internal(&self, name: &str) -> bool {
        // 标准库crate
        let std_crates = [
            "std",
            "core",
            "alloc",
            "proc_macro",
            "rustc_llvm",
            "rustc_middle",
            "rustc_span",
            "compiler_builtins",
            "panic_abort",
            "panic_unwind",
        ];

        std_crates.contains(&name) || name.starts_with("rustc_")
    }

    /// 检查crate是否在代码中被使用
    fn is_crate_used(&self, crate_name: &str, external_crates: &HashSet<String>) -> bool {
        // 直接引用
        if external_crates.contains(crate_name) {
            return true;
        }

        // 检查可能的宏引用（宏通常不以::形式使用）
        // 例如：serde!, clap!等
        for used_crate in external_crates {
            if used_crate.ends_with('!') {
                let name_without_bang = &used_crate[..used_crate.len() - 1];
                if name_without_bang == crate_name {
                    return true;
                }
            }
        }

        // 检查feature激活
        // 如果依赖是可选的，可能在feature中启用
        // 这里简化处理，假设可选依赖可能被使用
        false
    }

    /// 生成未使用依赖的移除建议
    pub fn generate_suggestions(&self, unused: &[UnusedDependency]) -> Vec<RemovalSuggestion> {
        unused
            .iter()
            .map(|dep| RemovalSuggestion {
                dependency: dep.name.clone(),
                reason: dep.reason.clone(),
                safe_to_remove: dep.safe_to_remove,
                removal_command: self.generate_removal_command(dep),
                savings: self.estimate_size_saving(dep),
            })
            .collect()
    }

    /// 生成移除命令
    fn generate_removal_command(&self, dep: &UnusedDependency) -> String {
        match dep.kind {
            DependencyKind::Normal => {
                format!("cargo remove {}", dep.name)
            }
            DependencyKind::Dev => {
                format!("cargo remove {} --dev", dep.name)
            }
            DependencyKind::Build => {
                format!("cargo remove {} --build", dep.name)
            }
        }
    }

    /// 估计移除依赖后节省的空间
    fn estimate_size_saving(&self, dep: &UnusedDependency) -> SizeEstimate {
        // 简化实现：基于常见crate的大小估算
        // 完整实现可以查询crates.io API获取实际大小

        let known_sizes: HashMap<&str, SizeEstimate> = HashMap::from([
            (
                "serde",
                SizeEstimate {
                    download: "2MB".to_string(),
                    disk: "5MB".to_string(),
                },
            ),
            (
                "tokio",
                SizeEstimate {
                    download: "3MB".to_string(),
                    disk: "10MB".to_string(),
                },
            ),
            (
                "clap",
                SizeEstimate {
                    download: "500KB".to_string(),
                    disk: "2MB".to_string(),
                },
            ),
            (
                "regex",
                SizeEstimate {
                    download: "400KB".to_string(),
                    disk: "1MB".to_string(),
                },
            ),
        ]);

        known_sizes.get(dep.name.as_str()).cloned().unwrap_or(SizeEstimate {
            download: "Unknown".to_string(),
            disk: "Unknown".to_string(),
        })
    }

    /// 分析可优化的依赖
    pub fn analyze_optimization_opportunities(&self) -> OptimizationReport {
        let unused = self.detect_unused_dependencies();
        let suggestions = self.generate_suggestions(&unused);

        let total_unused = unused.len();
        let safe_to_remove_count = suggestions.iter().filter(|s| s.safe_to_remove).count();

        let potential_savings = self.calculate_potential_savings(&suggestions);

        OptimizationReport {
            unused_dependencies: unused,
            removal_suggestions: suggestions,
            total_unused,
            safe_to_remove_count,
            potential_savings,
        }
    }

    /// 计算潜在的节省
    fn calculate_potential_savings(&self, suggestions: &[RemovalSuggestion]) -> String {
        let total_download_kb: usize = suggestions
            .iter()
            .filter_map(|s| s.savings.download.parse::<usize>().ok())
            .sum();

        if total_download_kb > 0 {
            format!("~{total_download_kb} total")
        } else {
            "Unknown".to_string()
        }
    }
}

/// 未使用的依赖信息
#[derive(Debug, Clone)]
pub struct UnusedDependency {
    /// 依赖名称
    pub name: String,

    /// 版本要求
    pub version: String,

    /// 依赖类型
    pub kind: DependencyKind,

    /// 是否为可选依赖
    pub optional: bool,

    /// 未使用的原因
    pub reason: UnusedReason,

    /// 是否安全移除
    pub safe_to_remove: bool,
}

/// 未使用的原因
#[derive(Debug, Clone)]
pub enum UnusedReason {
    /// 代码中未引用
    NotReferenced,

    /// 仅在测试中使用，但声明为正常依赖
    OnlyUsedInTests,

    /// 仅在特定feature中使用，但未标记为可选
    OnlyUsedInFeature(String),

    /// 被其他依赖替代
    ReplacedBy(String),
}

/// 移除建议
#[derive(Debug, Clone)]
pub struct RemovalSuggestion {
    /// 依赖名称
    pub dependency: String,

    /// 未使用原因
    pub reason: UnusedReason,

    /// 是否安全移除
    pub safe_to_remove: bool,

    /// 移除命令
    pub removal_command: String,

    /// 预估节省的空间
    pub savings: SizeEstimate,
}

/// 大小估算
#[derive(Debug, Clone)]
pub struct SizeEstimate {
    /// 下载大小
    pub download: String,

    /// 磁盘占用
    pub disk: String,
}

/// 优化机会报告
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    /// 未使用的依赖列表
    pub unused_dependencies: Vec<UnusedDependency>,

    /// 移除建议列表
    pub removal_suggestions: Vec<RemovalSuggestion>,

    /// 总未使用依赖数
    pub total_unused: usize,

    /// 可安全移除的依赖数
    pub safe_to_remove_count: usize,

    /// 潜在节省
    pub potential_savings: String,
}

impl OptimizationReport {
    /// 显示优化报告
    pub fn display(&self) -> String {
        let mut output = String::new();

        output.push_str("📋 依赖优化报告\n");
        output.push_str(&format!("未使用依赖数: {}\n", self.total_unused));
        output.push_str(&format!("可安全移除: {}\n", self.safe_to_remove_count));
        output.push_str(&format!("潜在节省: {}\n", self.potential_savings));
        output.push('\n');

        if self.unused_dependencies.is_empty() {
            output.push_str("✅ 未发现未使用的依赖\n");
        } else {
            output.push_str("⚠️  发现以下未使用的依赖:\n\n");

            for (i, dep) in self.unused_dependencies.iter().enumerate() {
                let marker = if dep.safe_to_remove {
                    "🟢"
                } else {
                    "⚠️ "
                };
                output.push_str(&format!("{} #{}: {}\n", marker, i + 1, dep.name));
                output.push_str(&format!("   版本: {}\n", dep.version));
                output.push_str(&format!("   类型: {:?}\n", dep.kind));
                output.push_str(&format!("   原因: {:?}\n", dep.reason));

                if let Some(suggestion) = self.removal_suggestions.get(i) {
                    output.push_str(&format!("   移除命令: {}\n", suggestion.removal_command));
                    output.push_str(&format!("   节省空间: {}\n", suggestion.savings.download));
                }

                output.push('\n');
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_or_internal_detection() {
        let detector = UnusedDetector {
            graph: &DependencyGraph {
                project_root: PathBuf::from("."),
                packages: vec![],
                dependencies: HashMap::new(),
                adjacency_list: HashMap::new(),
            },
            source_files: vec![],
            external_crates: HashSet::new(),
        };

        assert!(detector.is_std_or_internal("std"));
        assert!(detector.is_std_or_internal("core"));
        assert!(detector.is_std_or_internal("alloc"));
        assert!(!detector.is_std_or_internal("serde"));
    }
}
