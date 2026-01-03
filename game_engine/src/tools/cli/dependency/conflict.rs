//! # 版本冲突检测
//!
//! 检测依赖版本冲突，包括semver不兼容、重复依赖等。
//!
//! ## 功能
//!
//! - Semver版本冲突检测
//! - 重复依赖检测
//! - 传递依赖冲突分析
//! - 冲突解决建议
//!
//! ## 使用示例
//!
//! ```no_run
//! use game_engine::tools::cli::dependency::conflict::ConflictDetector;
//! use game_engine::tools::cli::dependency::graph::DependencyGraph;
//!
//! let graph = DependencyGraph::from_project(".").unwrap();
//! let detector = ConflictDetector::new(&graph);
//! let conflicts = detector.detect_all_conflicts();
//!
//! for conflict in conflicts {
//!     println!("Conflict: {}", conflict);
//! }
//! ```

use super::graph::{Dependency, DependencyGraph, DependencyKind, DependencySource};
use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet};

/// 版本冲突检测器
#[derive(Debug, Clone)]
pub struct ConflictDetector<'a> {
    /// 依赖图引用
    graph: &'a DependencyGraph,

    /// 已解析的版本信息
    resolved_versions: HashMap<String, Version>,
}

impl<'a> ConflictDetector<'a> {
    /// 创建新的冲突检测器
    pub fn new(graph: &'a DependencyGraph) -> Self {
        let mut detector = Self {
            graph,
            resolved_versions: HashMap::new(),
        };

        // 解析版本信息
        detector.resolve_versions();

        detector
    }

    /// 解析所有依赖的版本信息
    fn resolve_versions(&mut self) {
        for package in &self.graph.packages {
            if let Ok(version) = Version::parse(&package.version) {
                self.resolved_versions.insert(package.name.clone(), version);
            }
        }
    }

    /// 检测所有冲突
    pub fn detect_all_conflicts(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();

        // 检测版本要求冲突
        conflicts.extend(self.detect_version_requirement_conflicts());

        // 检测重复依赖
        conflicts.extend(self.detect_duplicate_dependencies());

        // 检测传递依赖冲突
        conflicts.extend(self.detect_transitive_conflicts());

        conflicts
    }

    /// 检测版本要求冲突
    ///
    /// 检测同一个依赖的不同版本要求是否兼容
    fn detect_version_requirement_conflicts(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();

        // 收集每个包的所有版本要求
        let mut version_requirements: HashMap<String, Vec<VersionRequirement>> = HashMap::new();

        for (_package_name, dependencies) in &self.graph.dependencies {
            for dep in dependencies {
                version_requirements.entry(dep.name.clone()).or_insert_with(Vec::new).push(
                    VersionRequirement {
                        requirement: dep.version_req.clone(),
                        source: _package_name.clone(),
                        kind: dep.kind,
                    },
                );
            }
        }

        // 检查每个依赖的版本要求是否兼容
        for (dep_name, requirements) in &version_requirements {
            if requirements.len() > 1 {
                // 尝试找到满足所有要求的版本
                if let Some(conflict) = self.check_version_compatibility(dep_name, requirements) {
                    conflicts.push(conflict);
                }
            }
        }

        conflicts
    }

    /// 检查版本兼容性
    fn check_version_compatibility(
        &self,
        dep_name: &str,
        requirements: &[VersionRequirement],
    ) -> Option<VersionConflict> {
        // 如果没有解析的版本，无法检查
        let resolved_version = self.resolved_versions.get(dep_name)?;

        // 检查是否所有要求都满足
        let mut unsatisfied = Vec::new();

        for req in requirements {
            if let Ok(version_req) = VersionReq::parse(&req.requirement) {
                if !version_req.matches(resolved_version) {
                    unsatisfied.push(req.clone());
                }
            }
        }

        if !unsatisfied.is_empty() {
            Some(VersionConflict::VersionRequirementMismatch {
                dependency: dep_name.to_string(),
                resolved_version: resolved_version.to_string(),
                unsatisfied_requirements: unsatisfied,
            })
        } else {
            None
        }
    }

    /// 检测重复依赖
    ///
    /// 检测同一个依赖是否被多次引入（可能是不同版本）
    fn detect_duplicate_dependencies(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();
        let mut seen_packages: HashMap<String, Vec<String>> = HashMap::new();

        for package in &self.graph.packages {
            if package.depth > 0 {
                // 跳过根包
                seen_packages
                    .entry(package.name.clone())
                    .or_insert_with(Vec::new)
                    .push(package.version.clone());
            }
        }

        // 检查是否有重复的依赖
        for (name, versions) in seen_packages {
            let unique_versions: HashSet<_> = versions.iter().collect();
            if unique_versions.len() > 1 {
                conflicts.push(VersionConflict::DuplicateDependency {
                    dependency: name,
                    versions: versions.into_iter().collect(),
                });
            }
        }

        conflicts
    }

    /// 检测传递依赖冲突
    ///
    /// 检测传递依赖是否与直接依赖冲突
    fn detect_transitive_conflicts(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();

        // 收集直接依赖
        let direct_deps: HashSet<String> = self
            .graph
            .dependencies
            .values()
            .flat_map(|deps| deps.iter().filter(|d| d.source == DependencySource::Direct))
            .map(|d| d.name.clone())
            .collect();

        // 检查传递依赖
        for (_package_name, dependencies) in &self.graph.dependencies {
            for dep in dependencies {
                if dep.source != DependencySource::Direct {
                    // 这是传递依赖
                    if direct_deps.contains(&dep.name) {
                        // 检查版本是否兼容
                        if let Some(conflict) = self.check_transitive_compatibility(dep) {
                            conflicts.push(conflict);
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// 检查传递依赖的兼容性
    fn check_transitive_compatibility(&self, dep: &Dependency) -> Option<VersionConflict> {
        let resolved_version = self.resolved_versions.get(&dep.name)?;

        // 检查版本要求是否满足
        if let Ok(version_req) = VersionReq::parse(&dep.version_req) {
            if !version_req.matches(resolved_version) {
                return Some(VersionConflict::TransitiveDependencyConflict {
                    dependency: dep.name.clone(),
                    direct_requirement: "*".to_string(), // 需要从直接依赖中获取
                    transitive_requirement: dep.version_req.clone(),
                    resolved_version: resolved_version.to_string(),
                });
            }
        }

        None
    }

    /// 生成冲突解决建议
    pub fn suggest_resolution(&self, conflict: &VersionConflict) -> Vec<String> {
        match conflict {
            VersionConflict::VersionRequirementMismatch {
                dependency,
                resolved_version,
                unsatisfied_requirements,
            } => {
                let mut suggestions = Vec::new();

                suggestions.push(format!(
                    "更新 {} 到版本 {} 以满足所有要求",
                    dependency, resolved_version
                ));

                // 尝试找到兼容版本
                if let Some(compatible_version) =
                    self.find_compatible_version(dependency, unsatisfied_requirements)
                {
                    suggestions.push(format!("使用版本 {} 作为折中方案", compatible_version));
                }

                suggestions.push("考虑统一依赖的版本要求".to_string());

                suggestions
            }

            VersionConflict::DuplicateDependency {
                dependency,
                versions,
            } => {
                let mut suggestions = Vec::new();

                suggestions.push(format!(
                    "依赖 {} 存在多个版本: {}",
                    dependency,
                    versions.join(", ")
                ));

                suggestions.push(format!("在Cargo.toml中明确指定 {} 的版本", dependency));

                suggestions.push("使用cargo update来统一依赖版本".to_string());

                suggestions
            }

            VersionConflict::TransitiveDependencyConflict {
                dependency,
                transitive_requirement,
                ..
            } => {
                let mut suggestions = Vec::new();

                suggestions.push(format!(
                    "传递依赖 {} 的版本要求 {} 与直接依赖冲突",
                    dependency, transitive_requirement
                ));

                suggestions.push("在Cargo.toml中明确指定版本".to_string());

                suggestions.push("使用[dependencies.features]来控制特性".to_string());

                suggestions
            }
        }
    }

    /// 查找兼容版本
    fn find_compatible_version(
        &self,
        _dependency: &str,
        _requirements: &[VersionRequirement],
    ) -> Option<String> {
        // 简化实现：返回最新版本
        // 完整实现应该查询crates.io API
        Some("latest".to_string())
    }

    /// 生成冲突报告
    pub fn generate_report(&self) -> ConflictReport {
        let conflicts = self.detect_all_conflicts();

        let total_conflicts = conflicts.len();
        let critical_count = conflicts.iter().filter(|c| c.is_critical()).count();

        ConflictReport {
            conflicts,
            total_conflicts,
            critical_count,
            has_critical: critical_count > 0,
        }
    }
}

/// 版本要求信息
#[derive(Debug, Clone)]
struct VersionRequirement {
    /// 版本要求字符串
    requirement: String,

    /// 来源包
    source: String,

    /// 依赖类型
    kind: DependencyKind,
}

/// 版本冲突
#[derive(Debug, Clone)]
pub enum VersionConflict {
    /// 版本要求不匹配
    VersionRequirementMismatch {
        /// 依赖名称
        dependency: String,
        /// 已解析的版本
        resolved_version: String,
        /// 不满足的要求列表
        unsatisfied_requirements: Vec<VersionRequirement>,
    },

    /// 重复依赖（不同版本）
    DuplicateDependency {
        /// 依赖名称
        dependency: String,
        /// 所有版本
        versions: Vec<String>,
    },

    /// 传递依赖冲突
    TransitiveDependencyConflict {
        /// 依赖名称
        dependency: String,
        /// 直接依赖的版本要求
        direct_requirement: String,
        /// 传递依赖的版本要求
        transitive_requirement: String,
        /// 已解析的版本
        resolved_version: String,
    },
}

impl VersionConflict {
    /// 是否为严重冲突
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            VersionConflict::VersionRequirementMismatch { .. }
                | VersionConflict::TransitiveDependencyConflict { .. }
        )
    }

    /// 获取冲突描述
    pub fn description(&self) -> String {
        match self {
            VersionConflict::VersionRequirementMismatch {
                dependency,
                resolved_version,
                unsatisfied_requirements,
            } => {
                format!(
                    "依赖 {} 的版本 {} 不满足以下要求:\n{}",
                    dependency,
                    resolved_version,
                    unsatisfied_requirements
                        .iter()
                        .map(|r| format!("  - {} (来自 {})", r.requirement, r.source))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }

            VersionConflict::DuplicateDependency {
                dependency,
                versions,
            } => {
                format!("依赖 {} 存在多个版本: {}", dependency, versions.join(", "))
            }

            VersionConflict::TransitiveDependencyConflict {
                dependency,
                direct_requirement,
                transitive_requirement,
                resolved_version,
            } => {
                format!(
                    "传递依赖 {} 冲突:\n  直接要求: {}\n  传递要求: {}\n  当前版本: {}",
                    dependency, direct_requirement, transitive_requirement, resolved_version
                )
            }
        }
    }
}

/// 冲突报告
#[derive(Debug, Clone)]
pub struct ConflictReport {
    /// 所有冲突
    pub conflicts: Vec<VersionConflict>,

    /// 总冲突数
    pub total_conflicts: usize,

    /// 严重冲突数
    pub critical_count: usize,

    /// 是否存在严重冲突
    pub has_critical: bool,
}

impl ConflictReport {
    /// 显示报告
    pub fn display(&self) -> String {
        let mut output = String::new();

        output.push_str("📋 依赖版本冲突报告\n");
        output.push_str(&format!("总冲突数: {}\n", self.total_conflicts));
        output.push_str(&format!("严重冲突: {}\n", self.critical_count));
        output.push_str("\n");

        if self.conflicts.is_empty() {
            output.push_str("✅ 未发现版本冲突\n");
        } else {
            output.push_str("⚠️  发现以下冲突:\n\n");

            for (i, conflict) in self.conflicts.iter().enumerate() {
                let marker = if conflict.is_critical() {
                    "🔴"
                } else {
                    "⚠️ "
                };
                output.push_str(&format!("{} 冲突 #{}:\n", marker, i + 1));
                output.push_str(&conflict.description());
                output.push_str("\n\n");
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_detection() {
        // 测试需要实际的依赖图
        // let graph = DependencyGraph::from_project(".").unwrap();
        // let detector = ConflictDetector::new(&graph);
        // let conflicts = detector.detect_all_conflicts();
    }
}
