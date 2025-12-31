//! # Render Bottleneck Analyzer
//!
//! 渲染瓶颈详细分析工具。
//!
//! ## 核心组件
//!
//! 1. **RenderAnalyzer** - 渲染分析器
//! 2. **OverdrawDetector** - Overdraw检测器
//! 3. **BandwidthAnalyzer** - 带宽分析器
//! 4. **PipelineProfiler** - Pipeline状态分析器

use std::collections::HashMap;
use std::time::Duration;

/// 渲染瓶颈类型
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderBottleneckType {
    /// Overdraw过高
    Overdraw,
    /// 带宽瓶颈
    Bandwidth,
    /// 纹理采样过多
    TextureSampling,
    /// 几何处理瓶颈
    Geometry,
    /// 光照计算瓶颈
    Lighting,
    /// 后处理瓶颈
    PostProcessing,
    /// 同步等待
    Synchronization,
}

/// 渲染统计信息
#[derive(Clone, Debug)]
pub struct RenderStats {
    /// 帧号
    pub frame_number: u64,
    /// 总渲染时间
    pub total_time: Duration,
    /// 几何处理时间
    pub geometry_time: Duration,
    /// 光照计算时间
    pub lighting_time: Duration,
    /// 后处理时间
    pub post_processing_time: Duration,
    /// Overdraw比率（0.0-1.0）
    pub overdraw_ratio: f32,
    /// 带宽使用量（字节/帧）
    pub bandwidth_usage: u64,
    /// 纹理采样次数
    pub texture_samples: u32,
    /// 三角形数量
    pub triangle_count: u32,
    /// 纹理数量
    pub texture_count: u32,
    /// render target切换次数
    pub render_target_switches: u32,
    /// Pipeline状态改变次数
    pub pipeline_changes: u32,
}

impl Default for RenderStats {
    fn default() -> Self {
        Self {
            frame_number: 0,
            total_time: Duration::ZERO,
            geometry_time: Duration::ZERO,
            lighting_time: Duration::ZERO,
            post_processing_time: Duration::ZERO,
            overdraw_ratio: 0.0,
            bandwidth_usage: 0,
            texture_samples: 0,
            triangle_count: 0,
            texture_count: 0,
            render_target_switches: 0,
            pipeline_changes: 0,
        }
    }
}

/// Overdraw检测器
pub struct OverdrawDetector {
    /// 历史数据
    history: Vec<f32>,
    /// 最大历史记录数
    max_history: usize,
    /// 警告阈值
    warning_threshold: f32,
    /// 严重阈值
    severe_threshold: f32,
}

impl OverdrawDetector {
    /// 创建新的检测器
    pub fn new(max_history: usize, warning_threshold: f32, severe_threshold: f32) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
            warning_threshold,
            severe_threshold,
        }
    }

    /// 记录overdraw比率
    pub fn record_overdraw(&mut self, ratio: f32) {
        let ratio = ratio.max(0.0).min(10.0); // 限制在0-10x
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(ratio);
    }

    /// 分析overdraw
    pub fn analyze(&self) -> OverdrawAnalysis {
        if self.history.is_empty() {
            return OverdrawAnalysis {
                average_overdraw: 0.0,
                max_overdraw: 0.0,
                severity: OverdrawSeverity::None,
                recommendations: Vec::new(),
            };
        }

        let average = self.history.iter().sum::<f32>() / self.history.len() as f32;
        let max = *self.history.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);

        let severity = if average > self.severe_threshold {
            OverdrawSeverity::Severe
        } else if average > self.warning_threshold {
            OverdrawSeverity::Moderate
        } else {
            OverdrawSeverity::None
        };

        let mut recommendations = Vec::new();

        if severity != OverdrawSeverity::None {
            recommendations.push("Use early-z culling to reduce overdraw".to_string());
            recommendations.push("Sort geometry by depth".to_string());
            recommendations.push("Consider using Z-prepass".to_string());
        }

        if severity == OverdrawSeverity::Severe {
            recommendations.push("Reduce particle count and transparency".to_string());
            recommendations.push("Use occlusion culling".to_string());
        }

        OverdrawAnalysis {
            average_overdraw: average,
            max_overdraw: max,
            severity,
            recommendations,
        }
    }

    /// 获取趋势
    pub fn get_trend(&self) -> Trend {
        if self.history.len() < 3 {
            return Trend::Stable;
        }

        let recent: f32 = self.history.iter().rev().take(5).sum::<f32>() / self.history.len().min(5) as f32;
        let older: f32 = self.history.iter().take(self.history.len().saturating_sub(5))
            .sum::<f32>() / self.history.len().saturating_sub(5).max(1) as f32;

        if recent > older * 1.1 {
            Trend::Worsening
        } else if recent < older * 0.9 {
            Trend::Improving
        } else {
            Trend::Stable
        }
    }
}

/// Overdraw严重程度
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverdrawSeverity {
    /// 正常
    None,
    /// 中度
    Moderate,
    /// 严重
    Severe,
}

/// 趋势
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trend {
    /// 改善中
    Improving,
    /// 稳定
    Stable,
    /// 恶化中
    Worsening,
}

/// Overdraw分析结果
#[derive(Clone, Debug)]
pub struct OverdrawAnalysis {
    pub average_overdraw: f32,
    pub max_overdraw: f32,
    pub severity: OverdrawSeverity,
    pub recommendations: Vec<String>,
}

// ==================== 带宽分析器 ====================

/// 带宽分析器
pub struct BandwidthAnalyzer {
    /// 带宽使用历史
    history: Vec<BandwidthSample>,
    /// 最大历史记录数
    max_history: usize,
    /// 警告阈值（字节/帧）
    warning_threshold: u64,
}

/// 带宽采样
#[derive(Clone, Debug)]
pub struct BandwidthSample {
    pub frame_number: u64,
    pub total_bytes: u64,
    pub texture_bytes: u64,
    pub vertex_bytes: u64,
    pub index_bytes: u64,
    pub render_target_bytes: u64,
}

impl BandwidthAnalyzer {
    /// 创建新的分析器
    pub fn new(max_history: usize, warning_threshold: u64) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
            warning_threshold,
        }
    }

    /// 记录带宽使用
    pub fn record_bandwidth(&mut self, frame_number: u64, sample: BandwidthSample) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(sample);
    }

    /// 分析带宽
    pub fn analyze(&self) -> BandwidthAnalysis {
        if self.history.is_empty() {
            return BandwidthAnalysis {
                average_total: 0,
                peak_total: 0,
                breakdown: BandwidthBreakdown {
                    textures: 0,
                    vertices: 0,
                    indices: 0,
                    render_targets: 0,
                },
                severity: BandwidthSeverity::None,
                bottleneck_type: None,
                recommendations: Vec::new(),
            };
        }

        let avg_total = self.history.iter().map(|s| s.total_bytes).sum::<u64>() / self.history.len() as u64;
        let peak_total = self.history.iter().map(|s| s.total_bytes).max().unwrap_or(0);

        let mut breakdown = BandwidthBreakdown {
            textures: 0,
            vertices: 0,
            indices: 0,
            render_targets: 0,
        };

        for sample in &self.history {
            breakdown.textures += sample.texture_bytes;
            breakdown.vertices += sample.vertex_bytes;
            breakdown.indices += sample.index_bytes;
            breakdown.render_targets += sample.render_target_bytes;
        }

        let total_samples = self.history.len() as u64;
        breakdown.textures /= total_samples;
        breakdown.vertices /= total_samples;
        breakdown.indices /= total_samples;
        breakdown.render_targets /= total_samples;

        // 判断瓶颈类型
        let mut bottleneck_type = None;
        let mut severity = BandwidthSeverity::None;

        if avg_total > self.warning_threshold {
            // 找出最大的带宽使用源
            let max_category = [
                ("textures", breakdown.textures),
                ("vertices", breakdown.vertices),
                ("indices", breakdown.indices),
                ("render_targets", breakdown.render_targets),
            ].iter().max_by_key(|(_, bytes)| *bytes).map(|(name, _)| *name);

            bottleneck_type = Some(max_category.unwrap().to_string());

            let ratio = avg_total as f64 / self.warning_threshold as f64;
            severity = if ratio > 2.0 {
                BandwidthSeverity::Critical
            } else if ratio > 1.5 {
                BandwidthSeverity::High
            } else {
                BandwidthSeverity::Moderate
            };
        }

        let mut recommendations = Vec::new();

        if let Some(category) = &bottleneck_type {
            match category.as_str() {
                "textures" => {
                    recommendations.push("Use texture compression (BC7, ASTC)".to_string());
                    recommendations.push("Reduce texture resolution".to_string());
                    recommendations.push("Use texture atlases".to_string());
                }
                "vertices" => {
                    recommendations.push("Use LOD for distant objects".to_string());
                    recommendations.push("Implement frustum culling".to_string());
                }
                "render_targets" => {
                    recommendations.push("Reduce render target size".to_string());
                    recommendations.push("Use fewer render passes".to_string());
                }
                _ => {}
            }
        }

        BandwidthAnalysis {
            average_total: avg_total,
            peak_total,
            breakdown,
            severity,
            bottleneck_type,
            recommendations,
        }
    }
}

/// 带宽严重程度
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BandwidthSeverity {
    None,
    Moderate,
    High,
    Critical,
}

/// 带宽分类
#[derive(Clone, Debug)]
pub struct BandwidthBreakdown {
    pub textures: u64,
    pub vertices: u64,
    pub indices: u64,
    pub render_targets: u64,
}

/// 带宽分析结果
#[derive(Clone, Debug)]
pub struct BandwidthAnalysis {
    pub average_total: u64,
    pub peak_total: u64,
    pub breakdown: BandwidthBreakdown,
    pub severity: BandwidthSeverity,
    pub bottleneck_type: Option<String>,
    pub recommendations: Vec<String>,
}

// ==================== Pipeline状态分析器 ====================

/// Pipeline状态分析器
pub struct PipelineProfiler {
    /// 状态改变历史
    state_changes: Vec<StateChange>,
    /// 当前状态
    current_state: PipelineState,
}

/// Pipeline状态
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PipelineState {
    pub shader: String,
    pub blend_mode: BlendMode,
    pub depth_test: bool,
    pub depth_write: bool,
    pub cull_mode: CullMode,
    pub topology: PrimitiveTopology,
}

/// 混合模式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    Opaque,
    Alpha,
    Additive,
    Multiply,
}

/// 剔除模式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CullMode {
    None,
    Back,
    Front,
}

/// 图元拓扑
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveTopology {
    TriangleList,
    TriangleStrip,
    LineList,
    PointList,
}

/// 状态改变记录
#[derive(Clone, Debug)]
pub struct StateChange {
    pub frame_number: u64,
    pub old_state: PipelineState,
    pub new_state: PipelineState,
    pub change_type: StateChangeType,
}

/// 状态改变类型
#[derive(Clone, Debug)]
pub enum StateChangeType {
    Shader,
    BlendMode,
    DepthTest,
    CullMode,
    Topology,
    Multiple,
}

impl PipelineProfiler {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            state_changes: Vec::new(),
            current_state: PipelineState {
                shader: String::new(),
                blend_mode: BlendMode::Opaque,
                depth_test: true,
                depth_write: true,
                cull_mode: CullMode::Back,
                topology: PrimitiveTopology::TriangleList,
            },
        }
    }

    /// 记录状态改变
    pub fn record_state_change(&mut self, frame_number: u64, new_state: PipelineState) {
        let change_type = self.detect_change_type(&self.current_state, &new_state);

        self.state_changes.push(StateChange {
            frame_number,
            old_state: self.current_state.clone(),
            new_state: new_state.clone(),
            change_type,
        });

        self.current_state = new_state;
    }

    /// 检测改变类型
    fn detect_change_type(&self, old: &PipelineState, new: &PipelineState) -> StateChangeType {
        let mut changes = Vec::new();

        if old.shader != new.shader {
            changes.push("shader");
        }
        if old.blend_mode != new.blend_mode {
            changes.push("blend");
        }
        if old.depth_test != new.depth_test {
            changes.push("depth_test");
        }
        if old.cull_mode != new.cull_mode {
            changes.push("cull");
        }
        if old.topology != new.topology {
            changes.push("topology");
        }

        if changes.len() == 0 {
            StateChangeType::Multiple // 无改变，返回Multiple避免误判
        } else if changes.len() == 1 {
            match changes[0] {
                "shader" => StateChangeType::Shader,
                "blend" => StateChangeType::BlendMode,
                "depth_test" => StateChangeType::DepthTest,
                "cull" => StateChangeType::CullMode,
                "topology" => StateChangeType::Topology,
                _ => StateChangeType::Multiple,
            }
        } else {
            StateChangeType::Multiple
        }
    }

    /// 分析状态改变
    pub fn analyze(&self, frame_count: u32) -> PipelineAnalysis {
        if self.state_changes.is_empty() {
            return PipelineAnalysis {
                total_changes: 0,
                changes_per_frame: 0.0,
                most_frequent_change: None,
                recommendations: Vec::new(),
            };
        }

        // 统计各类改变次数
        let mut shader_changes = 0;
        let mut blend_changes = 0;
        let mut depth_changes = 0;
        let mut cull_changes = 0;
        let mut topo_changes = 0;

        for change in &self.state_changes {
            match change.change_type {
                StateChangeType::Shader => shader_changes += 1,
                StateChangeType::BlendMode => blend_changes += 1,
                StateChangeType::DepthTest => depth_changes += 1,
                StateChangeType::CullMode => cull_changes += 1,
                StateChangeType::Topology => topo_changes += 1,
                StateChangeType::Multiple => {}
            }
        }

        let total_changes = self.state_changes.len();
        let changes_per_frame = total_changes as f32 / frame_count as f32;

        // 找出最频繁的改变
        let most_frequent = [
            ("shader", shader_changes),
            ("blend", blend_changes),
            ("depth", depth_changes),
            ("cull", cull_changes),
            ("topology", topo_changes),
        ].iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| *name);

        let mut recommendations = Vec::new();

        if changes_per_frame > 10.0 {
            recommendations.push("Sort draw calls by pipeline state to reduce changes".to_string());
        }

        let max_other = blend_changes.max(depth_changes).max(cull_changes).max(topo_changes);
        if shader_changes > max_other {
            recommendations.push("Group objects by shader to minimize shader switches".to_string());
        }

        if blend_changes > total_changes / 3 {
            recommendations.push("Batch opaque and transparent objects separately".to_string());
        }

        PipelineAnalysis {
            total_changes,
            changes_per_frame,
            most_frequent_change: most_frequent.map(String::from),
            recommendations,
        }
    }
}

/// Pipeline分析结果
#[derive(Clone, Debug)]
pub struct PipelineAnalysis {
    pub total_changes: usize,
    pub changes_per_frame: f32,
    pub most_frequent_change: Option<String>,
    pub recommendations: Vec<String>,
}

// ==================== 渲染分析器 ====================

/// 渲染分析器（主入口）
pub struct RenderAnalyzer {
    overdraw_detector: OverdrawDetector,
    bandwidth_analyzer: BandwidthAnalyzer,
    pipeline_profiler: PipelineProfiler,
}

impl RenderAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            overdraw_detector: OverdrawDetector::new(1000, 1.5, 3.0),
            bandwidth_analyzer: BandwidthAnalyzer::new(1000, 500_000_000), // 500MB警告阈值
            pipeline_profiler: PipelineProfiler::new(),
        }
    }

    /// 分析渲染统计
    pub fn analyze(&self, stats: &RenderStats, frame_count: u32) -> RenderBottleneckReport {
        // 分析overdraw
        let overdraw_analysis = self.overdraw_detector.analyze();

        // 分析带宽
        let bandwidth_analysis = self.bandwidth_analyzer.analyze();

        // 分析pipeline状态
        let pipeline_analysis = self.pipeline_profiler.analyze(frame_count);

        // 综合分析
        let bottlenecks = self.detect_render_bottlenecks(stats, &overdraw_analysis, &bandwidth_analysis);

        // 生成建议
        let mut recommendations = Vec::new();
        recommendations.extend(overdraw_analysis.recommendations.clone());
        recommendations.extend(bandwidth_analysis.recommendations.clone());
        recommendations.extend(pipeline_analysis.recommendations.clone());

        RenderBottleneckReport {
            overdraw_analysis,
            bandwidth_analysis,
            pipeline_analysis,
            bottlenecks,
            recommendations: self.deduplicate_recommendations(recommendations),
        }
    }

    /// 检测渲染瓶颈
    fn detect_render_bottlenecks(
        &self,
        stats: &RenderStats,
        overdraw: &OverdrawAnalysis,
        bandwidth: &BandwidthAnalysis,
    ) -> Vec<RenderBottleneck> {
        let mut bottlenecks = Vec::new();

        // Overdraw瓶颈
        if overdraw.severity != OverdrawSeverity::None {
            bottlenecks.push(RenderBottleneck {
                bottleneck_type: RenderBottleneckType::Overdraw,
                severity: self.overdraw_to_severity(overdraw.severity),
                description: format!("Overdraw ratio: {:.2}x", overdraw.average_overdraw),
                impact: "Pixel shader is processing same pixels multiple times".to_string(),
            });
        }

        // 带宽瓶颈
        if let Some(category) = &bandwidth.bottleneck_type {
            bottlenecks.push(RenderBottleneck {
                bottleneck_type: RenderBottleneckType::Bandwidth,
                severity: self.bandwidth_to_severity(bandwidth.severity),
                description: format!("{} bandwidth bottleneck: {:.2} MB/frame",
                                    category, bandwidth.average_total as f64 / 1_000_000.0),
                impact: "Memory bandwidth is limiting performance".to_string(),
            });
        }

        // 几何处理瓶颈
        if stats.geometry_time > stats.total_time / 3 {
            bottlenecks.push(RenderBottleneck {
                bottleneck_type: RenderBottleneckType::Geometry,
                severity: Severity::Medium,
                description: format!("Geometry processing: {:?}", stats.geometry_time),
                impact: "Vertex processing is taking too long".to_string(),
            });
        }

        // 光照计算瓶颈
        if stats.lighting_time > stats.total_time / 3 {
            bottlenecks.push(RenderBottleneck {
                bottleneck_type: RenderBottleneckType::Lighting,
                severity: Severity::Medium,
                description: format!("Lighting calculations: {:?}", stats.lighting_time),
                impact: "Fragment shader lighting is expensive".to_string(),
            });
        }

        // 纹理采样瓶颈
        if stats.texture_samples > 1000000 {
            bottlenecks.push(RenderBottleneck {
                bottleneck_type: RenderBottleneckType::TextureSampling,
                severity: Severity::Low,
                description: format!("Texture samples: {}", stats.texture_samples),
                impact: "Consider reducing texture lookups or using mipmaps".to_string(),
            });
        }

        bottlenecks
    }

    fn overdraw_to_severity(&self, severity: OverdrawSeverity) -> Severity {
        match severity {
            OverdrawSeverity::None => Severity::None,
            OverdrawSeverity::Moderate => Severity::Medium,
            OverdrawSeverity::Severe => Severity::High,
        }
    }

    fn bandwidth_to_severity(&self, severity: BandwidthSeverity) -> Severity {
        match severity {
            BandwidthSeverity::None => Severity::None,
            BandwidthSeverity::Moderate => Severity::Medium,
            BandwidthSeverity::High => Severity::High,
            BandwidthSeverity::Critical => Severity::Critical,
        }
    }

    fn deduplicate_recommendations(&self, recommendations: Vec<String>) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        recommendations.into_iter()
            .filter(|r| seen.insert(r.clone()))
            .collect()
    }

    /// 记录overdraw
    pub fn record_overdraw(&mut self, ratio: f32) {
        self.overdraw_detector.record_overdraw(ratio);
    }

    /// 记录带宽
    pub fn record_bandwidth(&mut self, frame_number: u64, sample: BandwidthSample) {
        self.bandwidth_analyzer.record_bandwidth(frame_number, sample);
    }

    /// 记录pipeline状态改变
    pub fn record_pipeline_change(&mut self, frame_number: u64, new_state: PipelineState) {
        self.pipeline_profiler.record_state_change(frame_number, new_state);
    }
}

impl Default for RenderAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 渲染瓶颈
#[derive(Clone, Debug)]
pub struct RenderBottleneck {
    pub bottleneck_type: RenderBottleneckType,
    pub severity: Severity,
    pub description: String,
    pub impact: String,
}

/// 严重程度
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// 渲染瓶颈报告
#[derive(Clone, Debug)]
pub struct RenderBottleneckReport {
    pub overdraw_analysis: OverdrawAnalysis,
    pub bandwidth_analysis: BandwidthAnalysis,
    pub pipeline_analysis: PipelineAnalysis,
    pub bottlenecks: Vec<RenderBottleneck>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overdraw_detector() {
        let mut detector = OverdrawDetector::new(10, 1.5, 3.0);

        detector.record_overdraw(1.0);
        detector.record_overdraw(2.0);
        detector.record_overdraw(4.0);

        let analysis = detector.analyze();
        assert!(analysis.average_overdraw > 2.0);
        assert!(!analysis.recommendations.is_empty());
    }

    #[test]
    fn test_bandwidth_analyzer() {
        let mut analyzer = BandwidthAnalyzer::new(10, 100_000_000);

        let sample = BandwidthSample {
            frame_number: 1,
            total_bytes: 150_000_000,
            texture_bytes: 100_000_000,
            vertex_bytes: 30_000_000,
            index_bytes: 10_000_000,
            render_target_bytes: 10_000_000,
        };

        analyzer.record_bandwidth(1, sample);
        let analysis = analyzer.analyze();

        assert!(analysis.average_total > 0);
        assert!(analysis.breakdown.textures > 0);
    }

    #[test]
    fn test_pipeline_profiler() {
        let mut profiler = PipelineProfiler::new();
        let state = PipelineState {
            shader: "basic.wgsl".to_string(),
            blend_mode: BlendMode::Opaque,
            depth_test: true,
            depth_write: true,
            cull_mode: CullMode::Back,
            topology: PrimitiveTopology::TriangleList,
        };

        profiler.record_state_change(1, state);
        let analysis = profiler.analyze(60);

        assert_eq!(analysis.total_changes, 1);
    }
}
