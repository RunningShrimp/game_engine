//! # Optimization Suggestion Generator
//!
//! 智能优化建议生成器 - 基于检测到的瓶颈自动生成可执行的优化建议。
//!
//! ## 核心组件
//!
//! 1. **OptimizationSuggestion** - 优化建议结构
//! 2. **SuggestionGenerator** - 建议生成器
//! 3. **PriorityCalculator** - 优先级计算器
//! 4. **ImpactEstimator** - 影响评估器

use super::memory_analyzer::MemoryBottleneckReport;
use super::render_analyzer::RenderBottleneckReport;
use crate::performance::profiler::{Bottleneck, PerformanceCategory, Severity};
use std::collections::HashMap;

/// 优化建议类别
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SuggestionCategory {
    /// 渲染优化
    Rendering,
    /// 内存优化
    Memory,
    /// CPU优化
    CPU,
    /// 资源管理
    Resource,
    /// 代码质量
    CodeQuality,
    /// 架构改进
    Architecture,
}

/// 优化建议
#[derive(Clone, Debug)]
pub struct OptimizationSuggestion {
    /// 建议ID
    pub id: String,
    /// 类别
    pub category: SuggestionCategory,
    /// 严重程度
    pub severity: Severity,
    /// 标题
    pub title: String,
    /// 详细描述
    pub description: String,
    /// 预期改进
    pub expected_improvement: String,
    /// 实施步骤
    pub implementation_steps: Vec<String>,
    /// 是否可自动修复
    pub can_auto_fix: bool,
    /// 预计工作量（小时）
    pub estimated_effort_hours: u32,
    /// 相关文件/组件
    pub affected_components: Vec<String>,
    /// 依赖的其他建议
    pub dependencies: Vec<String>,
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 额外资源链接
    pub references: Vec<String>,
}

/// 风险等级
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// 优化建议报告
#[derive(Clone, Debug)]
pub struct OptimizationReport {
    /// 生成的建议
    pub suggestions: Vec<OptimizationSuggestion>,
    /// 按优先级排序的建议ID
    pub priority_order: Vec<String>,
    /// 总体评估
    pub overall_assessment: OverallAssessment,
    /// 快速胜利（可快速实施的改进）
    pub quick_wins: Vec<String>,
    /// 长期战略建议
    pub strategic_improvements: Vec<String>,
}

/// 总体评估
#[derive(Clone, Debug)]
pub struct OverallAssessment {
    /// 健康得分 (0-100)
    pub health_score: u32,
    /// 最关键问题
    pub critical_issues: Vec<String>,
    /// 改进潜力
    pub improvement_potential: String,
    /// 总体建议
    pub general_recommendations: Vec<String>,
}

/// 建议生成器
pub struct SuggestionGenerator {
    /// 渲染优化知识库
    rendering_knowledge: RenderingKnowledgeBase,
    /// 内存优化知识库
    memory_knowledge: MemoryKnowledgeBase,
    /// 历史数据
    historical_data: HashMap<String, f64>,
}

impl SuggestionGenerator {
    /// 创建新的生成器
    pub fn new() -> Self {
        Self {
            rendering_knowledge: RenderingKnowledgeBase::new(),
            memory_knowledge: MemoryKnowledgeBase::new(),
            historical_data: HashMap::new(),
        }
    }

    /// 生成优化建议
    pub fn generate_suggestions(
        &self,
        bottlenecks: &[Bottleneck],
        render_report: Option<&RenderBottleneckReport>,
        memory_report: Option<&MemoryBottleneckReport>,
    ) -> OptimizationReport {
        let mut suggestions = Vec::new();

        // 基于通用瓶颈生成建议
        for bottleneck in bottlenecks {
            let category_suggestions = self.generate_for_bottleneck(bottleneck);
            suggestions.extend(category_suggestions);
        }

        // 基于渲染分析生成建议
        if let Some(report) = render_report {
            let render_suggestions = self.rendering_knowledge.generate_suggestions(report);
            suggestions.extend(render_suggestions);
        }

        // 基于内存分析生成建议
        if let Some(report) = memory_report {
            let memory_suggestions = self.memory_knowledge.generate_suggestions(report);
            suggestions.extend(memory_suggestions);
        }

        // 去重和优先级排序
        let suggestions = self.deduplicate_and_prioritize(suggestions);

        // 计算总体评估
        let overall_assessment = self.calculate_overall_assessment(&suggestions, bottlenecks);

        // 分类建议
        let (quick_wins, strategic_improvements) = self.categorize_suggestions(&suggestions);

        // 生成优先级顺序
        let priority_order = self.calculate_priority_order(&suggestions);

        OptimizationReport {
            suggestions,
            priority_order,
            overall_assessment,
            quick_wins,
            strategic_improvements,
        }
    }

    /// 为特定瓶颈生成建议
    fn generate_for_bottleneck(&self, bottleneck: &Bottleneck) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        match bottleneck.category {
            PerformanceCategory::Rendering => {
                // 根据描述判断具体渲染瓶颈
                let desc = bottleneck.description.to_lowercase();
                if desc.contains("frame") || desc.contains("fps") {
                    suggestions.push(self.create_frame_time_suggestion(bottleneck));
                } else if desc.contains("draw") {
                    suggestions.push(self.create_draw_call_suggestion(bottleneck));
                } else if desc.contains("triangle") {
                    suggestions.push(self.create_triangle_suggestion(bottleneck));
                } else if desc.contains("gpu") {
                    suggestions.push(self.create_gpu_suggestion(bottleneck));
                }
            }
            PerformanceCategory::Cpu => {
                suggestions.push(self.create_cpu_suggestion(bottleneck));
            }
            PerformanceCategory::Memory => {
                suggestions.push(self.create_memory_suggestion(bottleneck));
            }
            _ => {}
        }

        suggestions
    }

    /// 创建帧时间优化建议
    fn create_frame_time_suggestion(&self, bottleneck: &Bottleneck) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: "opt-frame-time-001".to_string(),
            category: SuggestionCategory::Rendering,
            severity: bottleneck.severity,
            title: "优化帧时间".to_string(),
            description: format!(
                "当前帧时间为 {:.2}ms，超过目标值 {:.2}ms。高帧时间会导致游戏卡顿和响应延迟。",
                bottleneck.current_value, bottleneck.target_value
            ),
            expected_improvement: "帧时间减少 30-50%，提升至 60FPS".to_string(),
            implementation_steps: vec![
                "识别和优化耗时长的系统".to_string(),
                "减少每帧的计算量".to_string(),
                "使用job system并行化任务".to_string(),
                "实施帧率限制和自适应质量".to_string(),
            ],
            can_auto_fix: false,
            estimated_effort_hours: match bottleneck.severity {
                Severity::Critical => 40,
                Severity::High => 20,
                Severity::Medium => 10,
                Severity::Low => 4,
            },
            affected_components: vec![
                "游戏循环".to_string(),
                "物理系统".to_string(),
                "AI系统".to_string(),
            ],
            dependencies: vec![],
            risk_level: RiskLevel::Medium,
            references: vec!["https://developer.nvidia.com/vulkan-shader-optimization".to_string()],
        }
    }

    /// 创建Draw Call优化建议
    fn create_draw_call_suggestion(&self, bottleneck: &Bottleneck) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: "opt-draw-calls-001".to_string(),
            category: SuggestionCategory::Rendering,
            severity: bottleneck.severity,
            title: "减少Draw Call数量".to_string(),
            description: format!(
                "当前Draw Call数量为 {}，超过推荐值。过多的Draw Calls会增加CPU开销。",
                bottleneck.current_value as u64
            ),
            expected_improvement: "Draw Calls减少 60-80%，CPU负载降低".to_string(),
            implementation_steps: vec![
                "实施动态批处理（Dynamic Batching）".to_string(),
                "使用GPU Instancing".to_string(),
                "合并使用相同材质的对象".to_string(),
                "使用Texture Atlases合并贴图".to_string(),
                "实施SRP Batcher（如适用）".to_string(),
            ],
            can_auto_fix: false,
            estimated_effort_hours: 16,
            affected_components: vec!["渲染管线".to_string(), "材质系统".to_string()],
            dependencies: vec![],
            risk_level: RiskLevel::Low,
            references: vec!["https://docs.unity3d.com/Manual/DrawCallBatching.html".to_string()],
        }
    }

    /// 创建三角形优化建议
    fn create_triangle_suggestion(&self, bottleneck: &Bottleneck) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: "opt-triangles-001".to_string(),
            category: SuggestionCategory::Rendering,
            severity: bottleneck.severity,
            title: "优化几何体复杂度".to_string(),
            description: format!(
                "当前三角形数量为 {}，超过目标值。高几何复杂度会影响GPU性能。",
                bottleneck.current_value as u64
            ),
            expected_improvement: "三角形数量减少 40-60%，GPU负载降低".to_string(),
            implementation_steps: vec![
                "使用LOD系统自动简化网格".to_string(),
                "移除隐藏的几何体（背面剔除）".to_string(),
                "使用遮挡剔除（Occlusion Culling）".to_string(),
                "优化模型拓扑结构".to_string(),
            ],
            can_auto_fix: false,
            estimated_effort_hours: 24,
            affected_components: vec![
                "LOD系统".to_string(),
                "网格系统".to_string(),
                "遮挡剔除".to_string(),
            ],
            dependencies: vec!["opt-lod-001".to_string()],
            risk_level: RiskLevel::Low,
            references: vec!["https://docs.unity3d.com/Manual/LevelOfDetail.html".to_string()],
        }
    }

    /// 创建CPU优化建议
    fn create_cpu_suggestion(&self, bottleneck: &Bottleneck) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: "opt-cpu-001".to_string(),
            category: SuggestionCategory::CPU,
            severity: bottleneck.severity,
            title: "优化CPU性能".to_string(),
            description: format!(
                "CPU使用率为 {:.1}%，超过安全阈值。高CPU使用会导致游戏卡顿。",
                bottleneck.current_value
            ),
            expected_improvement: "CPU使用率降低 20-30%".to_string(),
            implementation_steps: vec![
                "使用profiler识别热点函数".to_string(),
                "优化算法复杂度".to_string(),
                "使用对象池减少分配".to_string(),
                "启用SIMD优化".to_string(),
                "多线程并行处理".to_string(),
            ],
            can_auto_fix: false,
            estimated_effort_hours: 32,
            affected_components: vec![
                "物理引擎".to_string(),
                "AI系统".to_string(),
                "动画系统".to_string(),
            ],
            dependencies: vec![],
            risk_level: RiskLevel::Medium,
            references: vec!["https://doc.rust-lang.org/nomicon/vec.html".to_string()],
        }
    }

    /// 创建GPU优化建议
    fn create_gpu_suggestion(&self, bottleneck: &Bottleneck) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: "opt-gpu-001".to_string(),
            category: SuggestionCategory::Rendering,
            severity: bottleneck.severity,
            title: "优化GPU性能".to_string(),
            description: format!(
                "GPU使用率为 {:.1}%，接近饱和。需要优化渲染负载。",
                bottleneck.current_value
            ),
            expected_improvement: "GPU使用率降低 15-25%".to_string(),
            implementation_steps: vec![
                "优化shader复杂度".to_string(),
                "减少overdraw".to_string(),
                "降低分辨率或使用动态分辨率".to_string(),
                "优化后处理效果".to_string(),
                "使用compute shader加速计算".to_string(),
            ],
            can_auto_fix: false,
            estimated_effort_hours: 28,
            affected_components: vec![
                "Shader系统".to_string(),
                "后处理".to_string(),
                "渲染管线".to_string(),
            ],
            dependencies: vec![],
            risk_level: RiskLevel::Medium,
            references: vec![
                "https://developer.nvidia.com/gpugems/GPUGems/gpugems_ch02.html".to_string(),
            ],
        }
    }

    /// 创建内存优化建议
    fn create_memory_suggestion(&self, bottleneck: &Bottleneck) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: "opt-memory-001".to_string(),
            category: SuggestionCategory::Memory,
            severity: bottleneck.severity,
            title: "优化内存使用".to_string(),
            description: format!(
                "内存使用为 {:.2} MB，超过目标值。高内存使用可能导致崩溃。",
                bottleneck.current_value
            ),
            expected_improvement: "内存使用减少 20-40%".to_string(),
            implementation_steps: vec![
                "使用内存分析工具识别泄漏".to_string(),
                "实施资源卸载策略".to_string(),
                "优化纹理和网格压缩".to_string(),
                "使用对象池".to_string(),
                "启用内存profiling".to_string(),
            ],
            can_auto_fix: false,
            estimated_effort_hours: 20,
            affected_components: vec![
                "资源管理器".to_string(),
                "纹理系统".to_string(),
                "网格系统".to_string(),
            ],
            dependencies: vec![],
            risk_level: RiskLevel::Low,
            references: vec!["https://doc.rust-lang.org/std/alloc/index.html".to_string()],
        }
    }

    /// 去重和优先级排序
    fn deduplicate_and_prioritize(
        &self,
        mut suggestions: Vec<OptimizationSuggestion>,
    ) -> Vec<OptimizationSuggestion> {
        // 去重（基于ID）
        let mut seen = std::collections::HashSet::new();
        suggestions.retain(|s| seen.insert(s.id.clone()));

        // 按严重程度和预计工作量排序
        suggestions.sort_by(|a, b| {
            // 首先按严重程度降序
            match (a.severity, b.severity) {
                (Severity::Critical, Severity::Critical)
                | (Severity::High, Severity::High)
                | (Severity::Medium, Severity::Medium)
                | (Severity::Low, Severity::Low) => {}
                (Severity::Critical, _) => return std::cmp::Ordering::Less,
                (_, Severity::Critical) => return std::cmp::Ordering::Greater,
                (Severity::High, Severity::Medium) | (Severity::High, Severity::Low) => {
                    return std::cmp::Ordering::Less;
                }
                (Severity::Medium | Severity::Low, Severity::High) => {
                    return std::cmp::Ordering::Greater;
                }
                (Severity::Medium, Severity::Low) => return std::cmp::Ordering::Less,
                (Severity::Low, Severity::Medium) => return std::cmp::Ordering::Greater,
            }

            // 相同严重程度按工作量升序（quick wins优先）
            a.estimated_effort_hours.cmp(&b.estimated_effort_hours)
        });

        suggestions
    }

    /// 计算总体评估
    fn calculate_overall_assessment(
        &self,
        suggestions: &[OptimizationSuggestion],
        bottlenecks: &[Bottleneck],
    ) -> OverallAssessment {
        // 计算健康得分
        let critical_count =
            bottlenecks.iter().filter(|b| b.severity == Severity::Critical).count();
        let high_count = bottlenecks.iter().filter(|b| b.severity == Severity::High).count();
        let medium_count = bottlenecks.iter().filter(|b| b.severity == Severity::Medium).count();

        let health_score = 100u32.saturating_sub(
            (critical_count * 25) as u32 + (high_count * 10) as u32 + (medium_count * 3) as u32,
        );

        // 识别关键问题
        let critical_issues: Vec<String> = bottlenecks
            .iter()
            .filter(|b| b.severity == Severity::Critical)
            .map(|b| format!("{:?}: {}", b.category, b.description))
            .collect();

        // 改进潜力
        let improvement_potential = if health_score >= 80 {
            "良好，有少量改进空间".to_string()
        } else if health_score >= 60 {
            "中等，有显著改进潜力".to_string()
        } else if health_score >= 40 {
            "较大，需要重点优化".to_string()
        } else {
            "巨大，需要全面重构".to_string()
        };

        // 总体建议
        let mut general_recommendations = Vec::new();
        if health_score < 70 {
            general_recommendations.push("建议优先处理Critical和High级别的瓶颈".to_string());
        }
        if critical_count > 3 {
            general_recommendations.push("关键问题过多，建议制定分阶段优化计划".to_string());
        }
        if suggestions.iter().any(|s| s.can_auto_fix) {
            general_recommendations.push("部分优化可自动应用，建议优先执行".to_string());
        }

        OverallAssessment {
            health_score,
            critical_issues,
            improvement_potential,
            general_recommendations,
        }
    }

    /// 分类建议
    fn categorize_suggestions(
        &self,
        suggestions: &[OptimizationSuggestion],
    ) -> (Vec<String>, Vec<String>) {
        let quick_wins: Vec<String> = suggestions
            .iter()
            .filter(|s| s.estimated_effort_hours <= 8 && s.risk_level == RiskLevel::Low)
            .map(|s| s.id.clone())
            .collect();

        let strategic_improvements: Vec<String> = suggestions
            .iter()
            .filter(|s| {
                s.estimated_effort_hours > 24 || s.category == SuggestionCategory::Architecture
            })
            .map(|s| s.id.clone())
            .collect();

        (quick_wins, strategic_improvements)
    }

    /// 计算优先级顺序
    fn calculate_priority_order(&self, suggestions: &[OptimizationSuggestion]) -> Vec<String> {
        suggestions.iter().map(|s| s.id.clone()).collect()
    }
}

impl Default for SuggestionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 渲染优化知识库 ====================

/// 渲染优化知识库
pub struct RenderingKnowledgeBase {
    /// 优化模式
    patterns: Vec<RenderingPattern>,
}

impl RenderingKnowledgeBase {
    fn new() -> Self {
        Self {
            patterns: vec![
                RenderingPattern::Overdraw,
                RenderingPattern::Batching,
                RenderingPattern::Shader,
            ],
        }
    }

    fn generate_suggestions(&self, report: &RenderBottleneckReport) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Overdraw优化
        if report.overdraw_analysis.average_overdraw > 1.5 {
            suggestions.push(OptimizationSuggestion {
                id: "opt-render-overdraw-001".to_string(),
                category: SuggestionCategory::Rendering,
                severity: Severity::High,
                title: "减少Overdraw".to_string(),
                description: format!(
                    "检测到高overdraw比率: {:.2}。Overdraw会浪费GPU填充率。",
                    report.overdraw_analysis.average_overdraw
                ),
                expected_improvement: "GPU负载降低 20-40%".to_string(),
                implementation_steps: vec![
                    "优化渲染顺序（从前向后）".to_string(),
                    "使用Early-Z优化".to_string(),
                    "优化shader避免discard".to_string(),
                    "使用遮挡裁剪".to_string(),
                ],
                can_auto_fix: false,
                estimated_effort_hours: 12,
                affected_components: vec!["渲染管线".to_string()],
                dependencies: vec![],
                risk_level: RiskLevel::Low,
                references: vec![],
            });
        }

        // Bandwidth优化
        use super::render_analyzer::RenderBottleneckType;
        if let Some(bandwidth_bottleneck) = report
            .bottlenecks
            .iter()
            .find(|b| b.bottleneck_type == RenderBottleneckType::Bandwidth)
        {
            suggestions.push(OptimizationSuggestion {
                id: "opt-render-bandwidth-001".to_string(),
                category: SuggestionCategory::Rendering,
                severity: match bandwidth_bottleneck.severity {
                    super::render_analyzer::Severity::None => Severity::Low,
                    super::render_analyzer::Severity::Low => Severity::Low,
                    super::render_analyzer::Severity::Medium => Severity::Medium,
                    super::render_analyzer::Severity::High => Severity::High,
                    super::render_analyzer::Severity::Critical => Severity::Critical,
                },
                title: "优化显存带宽使用".to_string(),
                description: bandwidth_bottleneck.description.clone(),
                expected_improvement: "显存带宽使用降低 30-50%".to_string(),
                implementation_steps: vec![
                    "使用纹理压缩格式".to_string(),
                    "优化纹理大小".to_string(),
                    "使用Mipmaps".to_string(),
                    "优化渲染目标分辨率".to_string(),
                ],
                can_auto_fix: false,
                estimated_effort_hours: 16,
                affected_components: vec!["纹理系统".to_string(), "渲染管线".to_string()],
                dependencies: vec![],
                risk_level: RiskLevel::Low,
                references: vec![],
            });
        }

        suggestions
    }
}

/// 渲染优化模式
#[derive(Clone, Copy, Debug)]
enum RenderingPattern {
    Overdraw,
    Batching,
    Shader,
}

// ==================== 内存优化知识库 ====================

/// 内存优化知识库
pub struct MemoryKnowledgeBase {
    /// 优化模式
    patterns: Vec<MemoryPattern>,
}

impl MemoryKnowledgeBase {
    fn new() -> Self {
        Self {
            patterns: vec![
                MemoryPattern::LeakFix,
                MemoryPattern::FragmentationReduction,
                MemoryPattern::PoolOptimization,
            ],
        }
    }

    fn generate_suggestions(&self, report: &MemoryBottleneckReport) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // 泄漏修复
        if !report.leaks.is_empty() {
            for leak in &report.leaks {
                suggestions.push(OptimizationSuggestion {
                    id: format!("opt-mem-leak-{:?}", leak.leak_type),
                    category: SuggestionCategory::Memory,
                    severity: match leak.severity {
                        super::memory_analyzer::LeakSeverity::Moderate => Severity::Medium,
                        super::memory_analyzer::LeakSeverity::High => Severity::High,
                        super::memory_analyzer::LeakSeverity::Critical => Severity::Critical,
                    },
                    title: format!("修复内存泄漏: {}", leak.leak_type),
                    description: format!(
                        "检测到{}个对象未释放，总计{:.2}MB。",
                        leak.leak_count,
                        leak.total_size as f64 / 1_000_000.0
                    ),
                    expected_improvement: "消除内存泄漏，避免崩溃".to_string(),
                    implementation_steps: vec![
                        "检查对象生命周期".to_string(),
                        "添加RAII包装器".to_string(),
                        "使用智能指针".to_string(),
                        "实施内存监控".to_string(),
                    ],
                    can_auto_fix: false,
                    estimated_effort_hours: match leak.severity {
                        super::memory_analyzer::LeakSeverity::Moderate => 8,
                        super::memory_analyzer::LeakSeverity::High => 16,
                        super::memory_analyzer::LeakSeverity::Critical => 32,
                    },
                    affected_components: vec![leak.leak_type.clone()],
                    dependencies: vec![],
                    risk_level: RiskLevel::Medium,
                    references: vec![],
                });
            }
        }

        // 碎片化优化
        if report.fragmentation_report.severity
            != super::memory_analyzer::FragmentationSeverity::None
        {
            suggestions.push(OptimizationSuggestion {
                id: "opt-mem-frag-001".to_string(),
                category: SuggestionCategory::Memory,
                severity: Severity::Medium,
                title: "减少内存碎片化".to_string(),
                description: format!(
                    "内存碎片化率为 {:.1}%，影响性能。",
                    report.fragmentation_report.current_fragmentation * 100.0
                ),
                expected_improvement: "内存使用效率提升 10-20%".to_string(),
                implementation_steps: vec![
                    "使用内存池".to_string(),
                    "分配同样大小的对象".to_string(),
                    "使用Arena分配器".to_string(),
                    "实施定期内存压缩".to_string(),
                ],
                can_auto_fix: false,
                estimated_effort_hours: 20,
                affected_components: vec!["内存分配器".to_string()],
                dependencies: vec![],
                risk_level: RiskLevel::Medium,
                references: vec![],
            });
        }

        suggestions
    }
}

/// 内存优化模式
#[derive(Clone, Copy, Debug)]
enum MemoryPattern {
    LeakFix,
    FragmentationReduction,
    PoolOptimization,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_suggestion_generation() {
        let generator = SuggestionGenerator::new();
        let bottlenecks = vec![Bottleneck {
            category: PerformanceCategory::FrameTime,
            severity: Severity::High,
            description: "High frame time".to_string(),
            current_value: 33.3,
            target_value: 16.6,
            impact: "Game stuttering".to_string(),
            timestamp: Instant::now(),
        }];

        let report = generator.generate_suggestions(&bottlenecks, None, None);

        assert!(!report.suggestions.is_empty());
        assert!(report.overall_assessment.health_score < 100);
    }

    #[test]
    fn test_quick_win_identification() {
        let generator = SuggestionGenerator::new();
        let bottlenecks = vec![];

        let report = generator.generate_suggestions(&bottlenecks, None, None);

        // 应该有quick wins分类
        assert!(!report.quick_wins.is_empty() || !report.strategic_improvements.is_empty());
    }
}
