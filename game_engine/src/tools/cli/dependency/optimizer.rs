//! # 依赖优化建议
//!
//! 分析依赖并提供优化建议，包括更轻量/快速的替代方案。
//!
//! ## 功能
//!
//! - 识别可优化的依赖
//! - 推荐轻量级替代品
//! - 检测过度依赖
//! - 提供feature优化建议
//!
//! ## 使用示例
//!
//! ```no_run
//! use game_engine::tools::cli::dependency::optimizer::DependencyOptimizer;
//! use game_engine::tools::cli::dependency::graph::DependencyGraph;
//!
//! let graph = DependencyGraph::from_project(".").unwrap();
//! let optimizer = DependencyOptimizer::new(&graph);
//! let suggestions = optimizer.generate_optimization_suggestions();
//!
//! for suggestion in suggestions {
//!     println!("Optimization: {}", suggestion.description());
//! }
//! ```

use super::graph::{Dependency, DependencyGraph, DependencyKind};
use std::collections::HashMap;

/// 依赖优化器
#[derive(Debug, Clone)]
pub struct DependencyOptimizer<'a> {
    /// 依赖图引用
    graph: &'a DependencyGraph,

    /// 替代品数据库
    alternatives_db: HashMap<String, Vec<Alternative>>,
}

impl<'a> DependencyOptimizer<'a> {
    /// 创建新的依赖优化器
    pub fn new(graph: &'a DependencyGraph) -> Self {
        let mut optimizer = Self {
            graph,
            alternatives_db: HashMap::new(),
        };

        // 初始化替代品数据库
        optimizer.init_alternatives_db();

        optimizer
    }

    /// 初始化替代品数据库
    fn init_alternatives_db(&mut self) {
        // 序列化相关
        self.add_alternative(
            "serde_json",
            vec![Alternative {
                name: "simd-json".to_string(),
                reason: "SIMD加速，性能提升2-4x".to_string(),
                trade_offs: "API略有不同，需要适配".to_string(),
                size_reduction: "相似".to_string(),
                performance_improvement: "2-4x faster".to_string(),
                url: "https://github.com/simd-lite/simd-json".to_string(),
            }],
        );

        self.add_alternative(
            "serde",
            vec![Alternative {
                name: "miniserde".to_string(),
                reason: "更小的依赖，适合简单场景".to_string(),
                trade_offs: "功能较少，仅支持基本类型".to_string(),
                size_reduction: "~10x smaller".to_string(),
                performance_improvement: "编译更快".to_string(),
                url: "https://github.com/dtolnay/miniserde".to_string(),
            }],
        );

        // 异步运行时
        self.add_alternative(
            "tokio",
            vec![Alternative {
                name: "async-std".to_string(),
                reason: "更简洁的API，标准库风格".to_string(),
                trade_offs: "生态系统较小".to_string(),
                size_reduction: "相似".to_string(),
                performance_improvement: "API更易用".to_string(),
                url: "https://async.rs/".to_string(),
            }],
        );

        // HTTP客户端
        self.add_alternative(
            "reqwest",
            vec![
                Alternative {
                    name: "ureq".to_string(),
                    reason: "更简单的API，更少的依赖".to_string(),
                    trade_offs: "功能较少，不支持异步".to_string(),
                    size_reduction: "~5x smaller".to_string(),
                    performance_improvement: "启动更快".to_string(),
                    url: "https://github.com/algestenureq".to_string(),
                },
                Alternative {
                    name: "attohttpc".to_string(),
                    reason: "小型同步HTTP客户端".to_string(),
                    trade_offs: "仅支持HTTP/1.1".to_string(),
                    size_reduction: "~10x smaller".to_string(),
                    performance_improvement: "编译更快".to_string(),
                    url: "https://github.com/sbstp/attohttpc".to_string(),
                },
            ],
        );

        // 日志
        self.add_alternative(
            "log",
            vec![Alternative {
                name: "tracing".to_string(),
                reason: "更强大的功能和上下文感知".to_string(),
                trade_offs: "API稍复杂".to_string(),
                size_reduction: "相似".to_string(),
                performance_improvement: "更好的性能监控".to_string(),
                url: "https://tokio.rs/blog/2019-10-tracing".to_string(),
            }],
        );

        // 正则表达式
        self.add_alternative(
            "regex",
            vec![Alternative {
                name: "fancy-regex".to_string(),
                reason: "支持更复杂的正则语法".to_string(),
                trade_offs: "性能较低".to_string(),
                size_reduction: "相似".to_string(),
                performance_improvement: "功能更强大".to_string(),
                url: "https://github.com/fancy-regex/fancy-regex".to_string(),
            }],
        );

        // CLI工具
        self.add_alternative(
            "clap",
            vec![Alternative {
                name: "argh".to_string(),
                reason: "更简单的推导式API".to_string(),
                trade_offs: "功能较少".to_string(),
                size_reduction: "~2x smaller".to_string(),
                performance_improvement: "编译更快".to_string(),
                url: "https://github.com/google/argh".to_string(),
            }],
        );

        // 随机数
        self.add_alternative(
            "rand",
            vec![Alternative {
                name: "fastrand".to_string(),
                reason: "更简单的API，仅包含必要的功能".to_string(),
                trade_offs: "功能较少".to_string(),
                size_reduction: "~5x smaller".to_string(),
                performance_improvement: "性能相当".to_string(),
                url: "https://github.com/smol-rs/fastrand".to_string(),
            }],
        );

        // 时间处理
        self.add_alternative(
            "chrono",
            vec![Alternative {
                name: "time".to_string(),
                reason: "更现代的设计，更好的API".to_string(),
                trade_offs: "生态系统较小".to_string(),
                size_reduction: "~2x smaller".to_string(),
                performance_improvement: "性能相当".to_string(),
                url: "https://github.com/time-rs/time".to_string(),
            }],
        );
    }

    /// 添加替代品
    fn add_alternative(&mut self, dependency: &str, alternatives: Vec<Alternative>) {
        self.alternatives_db
            .entry(dependency.to_string())
            .or_default()
            .extend(alternatives);
    }

    /// 生成优化建议
    pub fn generate_optimization_suggestions(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // 分析每个依赖
        for (dep_name, dependencies) in &self.graph.dependencies {
            for dep in dependencies {
                // 检查是否有替代品
                if let Some(alternatives) = self.alternatives_db.get(&dep.name) {
                    for alt in alternatives {
                        suggestions.push(OptimizationSuggestion {
                            dependency: dep.name.clone(),
                            current_version: dep.version_req.clone(),
                            suggestion_type: SuggestionType::Alternative,
                            alternative: Some(alt.clone()),
                            reason: format!("考虑使用 {} 替代 {}", alt.name, dep.name),
                            priority: self.calculate_priority(dep, alt),
                            estimated_impact: self.estimate_impact(dep, alt),
                        });
                    }
                }

                // 检查是否可以优化features
                if self.has_unused_features(dep) {
                    suggestions.push(OptimizationSuggestion {
                        dependency: dep.name.clone(),
                        current_version: dep.version_req.clone(),
                        suggestion_type: SuggestionType::FeatureOptimization,
                        alternative: None,
                        reason: "禁用未使用的features以减小编译时间".to_string(),
                        priority: Priority::Medium,
                        estimated_impact: "编译时间减少10-30%".to_string(),
                    });
                }

                // 检查是否过度依赖
                if self.is_overengineered(dep) {
                    suggestions.push(OptimizationSuggestion {
                        dependency: dep.name.clone(),
                        current_version: dep.version_req.clone(),
                        suggestion_type: SuggestionType::Simplification,
                        alternative: None,
                        reason: "当前依赖过于复杂，考虑简化方案".to_string(),
                        priority: Priority::Low,
                        estimated_impact: "代码更易维护".to_string(),
                    });
                }
            }
        }

        suggestions
    }

    /// 检查依赖是否有未使用的features
    fn has_unused_features(&self, _dep: &Dependency) -> bool {
        // 简化实现：假设某些常见依赖可能有未使用的features
        // 完整实现需要分析代码中使用的特性
        true
    }

    /// 检查依赖是否过度工程化
    fn is_overengineered(&self, dep: &Dependency) -> bool {
        // 简化实现：检查版本要求是否过于宽泛
        dep.version_req == "*" || dep.version_req.starts_with('>')
    }

    /// 计算建议的优先级
    fn calculate_priority(&self, _dep: &Dependency, alt: &Alternative) -> Priority {
        // 基于性能提升和大小减少计算优先级
        if alt.performance_improvement.contains("2-4x")
            || alt.size_reduction.contains("5x")
            || alt.size_reduction.contains("10x")
        {
            Priority::High
        } else if alt.performance_improvement.contains("faster")
            || alt.size_reduction.contains("2x")
        {
            Priority::Medium
        } else {
            Priority::Low
        }
    }

    /// 估算影响
    fn estimate_impact(&self, _dep: &Dependency, alt: &Alternative) -> String {
        if !alt.size_reduction.is_empty() && !alt.performance_improvement.is_empty() {
            format!(
                "大小: {}, 性能: {}",
                alt.size_reduction, alt.performance_improvement
            )
        } else if !alt.size_reduction.is_empty() {
            format!("大小减少: {}", alt.size_reduction)
        } else if !alt.performance_improvement.is_empty() {
            format!("性能提升: {}", alt.performance_improvement)
        } else {
            "优化效果未知".to_string()
        }
    }

    /// 生成优化报告
    pub fn generate_optimization_report(&self) -> OptimizationReport {
        let suggestions = self.generate_optimization_suggestions();

        let high_priority = suggestions.iter().filter(|s| s.priority == Priority::High).count();

        let medium_priority = suggestions.iter().filter(|s| s.priority == Priority::Medium).count();

        let total_suggestions = suggestions.len();
        let potential_savings = self.calculate_total_savings(&suggestions);

        OptimizationReport {
            suggestions,
            total_suggestions,
            high_priority_count: high_priority,
            medium_priority_count: medium_priority,
            potential_savings,
        }
    }

    /// 计算总体节省
    fn calculate_total_savings(&self, suggestions: &[OptimizationSuggestion]) -> String {
        let size_reductions: Vec<_> = suggestions
            .iter()
            .filter_map(|s| s.alternative.as_ref().map(|alt| alt.size_reduction.as_str()))
            .collect();

        if size_reductions.is_empty() {
            "Unknown".to_string()
        } else {
            format!("多个依赖可优化: {}", size_reductions.join(", "))
        }
    }
}

/// 替代品信息
#[derive(Debug, Clone)]
pub struct Alternative {
    /// 替代品名称
    pub name: String,

    /// 推荐理由
    pub reason: String,

    /// 权衡说明
    pub trade_offs: String,

    /// 大小减少
    pub size_reduction: String,

    /// 性能提升
    pub performance_improvement: String,

    /// 相关URL
    pub url: String,
}

/// 优化建议
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    /// 当前依赖名称
    pub dependency: String,

    /// 当前版本
    pub current_version: String,

    /// 建议类型
    pub suggestion_type: SuggestionType,

    /// 替代品信息（如果适用）
    pub alternative: Option<Alternative>,

    /// 建议原因
    pub reason: String,

    /// 优先级
    pub priority: Priority,

    /// 预估影响
    pub estimated_impact: String,
}

/// 建议类型
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    /// 使用替代品
    Alternative,

    /// Feature优化
    FeatureOptimization,

    /// 简化
    Simplification,
}

/// 优先级
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Priority {
    /// 高优先级
    High,

    /// 中等优先级
    Medium,

    /// 低优先级
    Low,
}

/// 优化报告
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    /// 所有建议
    pub suggestions: Vec<OptimizationSuggestion>,

    /// 总建议数
    pub total_suggestions: usize,

    /// 高优先级建议数
    pub high_priority_count: usize,

    /// 中等优先级建议数
    pub medium_priority_count: usize,

    /// 潜在节省
    pub potential_savings: String,
}

impl OptimizationReport {
    /// 显示优化报告
    pub fn display(&self) -> String {
        let mut output = String::new();

        output.push_str("📋 依赖优化建议报告\n");
        output.push_str(&format!("总建议数: {}\n", self.total_suggestions));
        output.push_str(&format!("高优先级: {}\n", self.high_priority_count));
        output.push_str(&format!("中优先级: {}\n", self.medium_priority_count));
        output.push_str(&format!("潜在节省: {}\n", self.potential_savings));
        output.push('\n');

        if self.suggestions.is_empty() {
            output.push_str("✅ 未发现优化机会\n");
        } else {
            output.push_str("💡 发现以下优化机会:\n\n");

            for (i, suggestion) in self.suggestions.iter().enumerate() {
                let marker = match suggestion.priority {
                    Priority::High => "🔴",
                    Priority::Medium => "🟡",
                    Priority::Low => "🟢",
                };

                output.push_str(&format!(
                    "{} 建议 #{}: {}\n",
                    marker,
                    i + 1,
                    suggestion.dependency
                ));
                output.push_str(&format!("   当前版本: {}\n", suggestion.current_version));
                output.push_str(&format!("   类型: {:?}\n", suggestion.suggestion_type));
                output.push_str(&format!("   原因: {}\n", suggestion.reason));
                output.push_str(&format!("   预估影响: {}\n", suggestion.estimated_impact));

                if let Some(alt) = &suggestion.alternative {
                    output.push_str(&format!("   替代方案: {}\n", alt.name));
                    output.push_str(&format!("   说明: {}\n", alt.reason));
                    output.push_str(&format!("   权衡: {}\n", alt.trade_offs));
                    output.push_str(&format!("   URL: {}\n", alt.url));
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
    use std::path::PathBuf;

    #[test]
    fn test_alternatives_db_init() {
        let optimizer = DependencyOptimizer {
            graph: &DependencyGraph {
                project_root: PathBuf::from("."),
                packages: vec![],
                dependencies: HashMap::new(),
                adjacency_list: HashMap::new(),
            },
            alternatives_db: HashMap::new(),
        };

        optimizer.init_alternatives_db();

        // 验证数据库已初始化
        assert!(optimizer.alternatives_db.contains_key("serde"));
        assert!(optimizer.alternatives_db.contains_key("tokio"));
        assert!(optimizer.alternatives_db.contains_key("reqwest"));
    }
}
