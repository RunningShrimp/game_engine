//! # Performance Report Generator
//!
//! 生成详细的性能分析报告 - 支持HTML和PDF格式输出。
//!
//! ## 核心组件
//!
//! 1. **ReportGenerator** - 报告生成器
//! 2. **HtmlReportBuilder** - HTML报告构建器
//! 3. **PdfReportBuilder** - PDF报告构建器
//! 4. **ReportData** - 报告数据模型

use super::auto_fix::AutoFixResult;
use super::memory_analyzer::MemoryBottleneckReport;
use super::optimization_suggestion::{OptimizationReport, OptimizationSuggestion};
use super::render_analyzer::RenderBottleneckReport;
use crate::performance::profiler::PerformanceMetrics;
use std::time::{SystemTime, UNIX_EPOCH};

/// 报告格式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportFormat {
    Html,
    Pdf,
    Markdown,
}

/// 报告配置
#[derive(Clone, Debug)]
pub struct ReportConfig {
    /// 包含图表
    pub include_charts: bool,
    /// 包含详细日志
    pub include_detailed_logs: bool,
    /// 包含历史对比
    pub include_historical_comparison: bool,
    /// 主题
    pub theme: ReportTheme,
}

/// 报告主题
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportTheme {
    Light,
    Dark,
    Auto,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_charts: true,
            include_detailed_logs: true,
            include_historical_comparison: false,
            theme: ReportTheme::Auto,
        }
    }
}

/// 性能报告数据
#[derive(Clone, Debug)]
pub struct PerformanceReportData {
    /// 报告标题
    pub title: String,
    /// 生成时间
    pub generated_at: SystemTime,
    /// 项目名称
    pub project_name: String,
    /// 项目路径
    pub project_path: String,
    /// 性能指标
    pub metrics: PerformanceMetrics,
    /// 渲染瓶颈报告
    pub render_report: Option<RenderBottleneckReport>,
    /// 内存瓶颈报告
    pub memory_report: Option<MemoryBottleneckReport>,
    /// 优化建议报告
    pub optimization_report: Option<OptimizationReport>,
    /// 自动修复结果
    pub auto_fix_results: Vec<AutoFixResult>,
    /// 会话持续时间（秒）
    pub session_duration_seconds: u64,
}

/// 报告生成器
pub struct ReportGenerator {
    /// HTML构建器
    html_builder: HtmlReportBuilder,
    /// PDF构建器
    pdf_builder: PdfReportBuilder,
}

impl ReportGenerator {
    /// 创建新的生成器
    pub fn new() -> Self {
        Self {
            html_builder: HtmlReportBuilder::new(),
            pdf_builder: PdfReportBuilder::new(),
        }
    }

    /// 生成报告
    pub fn generate(
        &self,
        data: &PerformanceReportData,
        format: ReportFormat,
        config: &ReportConfig,
    ) -> Result<GeneratedReport, ReportError> {
        match format {
            ReportFormat::Html => self.html_builder.build(data, config),
            ReportFormat::Pdf => self.pdf_builder.build(data, config),
            ReportFormat::Markdown => Err(ReportError::UnsupportedFormat),
        }
    }

    /// 保存报告到文件
    pub fn save_to_file(
        &self,
        data: &PerformanceReportData,
        format: ReportFormat,
        config: &ReportConfig,
        output_path: &str,
    ) -> Result<(), ReportError> {
        let report = self.generate(data, format, config)?;

        std::fs::write(output_path, report.content)
            .map_err(|e| ReportError::IoError(e.to_string()))?;

        Ok(())
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 生成的报告
#[derive(Clone, Debug)]
pub struct GeneratedReport {
    /// 报告内容
    pub content: String,
    /// 格式
    pub format: ReportFormat,
    /// 字节数
    pub size_bytes: usize,
}

/// 报告错误
#[derive(Clone, Debug)]
pub enum ReportError {
    IoError(String),
    TemplateError(String),
    UnsupportedFormat,
    GenerationError(String),
}

// ==================== HTML报告构建器 ====================

/// HTML报告构建器
pub struct HtmlReportBuilder {
    /// CSS样式
    styles: String,
}

impl HtmlReportBuilder {
    fn new() -> Self {
        Self {
            styles: Self::get_default_styles(),
        }
    }

    /// 构建HTML报告
    fn build(
        &self,
        data: &PerformanceReportData,
        config: &ReportConfig,
    ) -> Result<GeneratedReport, ReportError> {
        let mut html = String::new();

        // HTML头部
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"zh-CN\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str(&format!("    <title>{}</title>\n", data.title));
        html.push_str("    <style>\n");
        html.push_str(&self.styles);
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // 报告头部
        html.push_str(&self.build_header(data));

        // 性能概览
        html.push_str(&self.build_overview(data));

        // 渲染分析
        if let Some(ref render_report) = data.render_report {
            html.push_str(&self.build_render_section(render_report));
        }

        // 内存分析
        if let Some(ref memory_report) = data.memory_report {
            html.push_str(&self.build_memory_section(memory_report));
        }

        // 优化建议
        if let Some(ref opt_report) = data.optimization_report {
            html.push_str(&self.build_optimization_section(opt_report));
        }

        // 自动修复结果
        if !data.auto_fix_results.is_empty() {
            html.push_str(&self.build_auto_fix_section(&data.auto_fix_results));
        }

        // 详细日志
        if config.include_detailed_logs {
            html.push_str(&self.build_detailed_logs(data));
        }

        // 报告尾部
        html.push_str(&self.build_footer(data));

        html.push_str("</body>\n");
        html.push_str("</html>");

        let size = html.len();
        Ok(GeneratedReport {
            content: html,
            format: ReportFormat::Html,
            size_bytes: size,
        })
    }

    /// 构建头部
    fn build_header(&self, data: &PerformanceReportData) -> String {
        format!(
            "<div class=\"header\">\n\
             <h1>{}</h1>\n\
             <div class=\"meta\">\n\
             <p><strong>项目:</strong> {}</p>\n\
             <p><strong>生成时间:</strong> {}</p>\n\
             <p><strong>会话时长:</strong> {} 秒</p>\n\
             </div>\n\
             </div>\n",
            data.title,
            data.project_name,
            format_system_time(data.generated_at),
            data.session_duration_seconds
        )
    }

    /// 构建概览
    fn build_overview(&self, data: &PerformanceReportData) -> String {
        let metrics = &data.metrics;
        let fps = metrics.fps;
        let frame_time_ms = metrics.frame_time.as_secs_f64() * 1000.0;
        let cpu_percent = metrics.cpu_usage * 100.0;
        let memory_mb = metrics.memory_usage as f64 / 1_000_000.0;

        format!(
            "<div class=\"section\">\n\
             <h2>性能概览</h2>\n\
             <div class=\"metrics-grid\">\n\
             <div class=\"metric-card fps\">\n\
             <div class=\"metric-value\">{:.1}</div>\n\
             <div class=\"metric-label\">FPS</div>\n\
             </div>\n\
             <div class=\"metric-card frame-time\">\n\
             <div class=\"metric-value\">{:.2}</div>\n\
             <div class=\"metric-label\">帧时间 (ms)</div>\n\
             </div>\n\
             <div class=\"metric-card draw-calls\">\n\
             <div class=\"metric-value\">{}</div>\n\
             <div class=\"metric-label\">Draw Calls</div>\n\
             </div>\n\
             <div class=\"metric-card triangles\">\n\
             <div class=\"metric-value\">{}</div>\n\
             <div class=\"metric-label\">三角形</div>\n\
             </div>\n\
             <div class=\"metric-card cpu\">\n\
             <div class=\"metric-value\">{:.1}%</div>\n\
             <div class=\"metric-label\">CPU使用率</div>\n\
             </div>\n\
             <div class=\"metric-card memory\">\n\
             <div class=\"metric-value\">{:.1}</div>\n\
             <div class=\"metric-label\">内存 (MB)</div>\n\
             </div>\n\
             </div>\n\
             </div>\n",
            fps, frame_time_ms, metrics.draw_calls, metrics.triangle_count, cpu_percent, memory_mb
        )
    }

    /// 构建渲染部分
    fn build_render_section(&self, report: &RenderBottleneckReport) -> String {
        let mut html = String::new();
        html.push_str("<div class=\"section\">\n");
        html.push_str("<h2>渲染分析</h2>\n");

        // Overdraw分析
        html.push_str("<h3>Overdraw分析</h3>\n");
        html.push_str(&self.build_overdraw_table(&report.overdraw_analysis));

        // 带宽分析
        html.push_str("<h3>带宽分析</h3>\n");
        html.push_str(&self.build_bandwidth_table(&report.bandwidth_analysis));

        // Pipeline分析
        html.push_str("<h3>Pipeline状态分析</h3>\n");
        html.push_str(&self.build_pipeline_table(&report.pipeline_analysis));

        // 瓶颈列表
        if !report.bottlenecks.is_empty() {
            html.push_str("<h3>检测到的瓶颈</h3>\n");
            html.push_str("<ul class=\"bottleneck-list\">\n");
            for bottleneck in &report.bottlenecks {
                let severity = match bottleneck.severity {
                    super::render_analyzer::Severity::None => {
                        crate::performance::profiler::Severity::Low
                    }
                    super::render_analyzer::Severity::Low => {
                        crate::performance::profiler::Severity::Low
                    }
                    super::render_analyzer::Severity::Medium => {
                        crate::performance::profiler::Severity::Medium
                    }
                    super::render_analyzer::Severity::High => {
                        crate::performance::profiler::Severity::High
                    }
                    super::render_analyzer::Severity::Critical => {
                        crate::performance::profiler::Severity::Critical
                    }
                };
                html.push_str(&format!(
                    "<li class=\"bottleneck {}\">{:?}: {}</li>\n",
                    format_severity(severity),
                    bottleneck.bottleneck_type,
                    bottleneck.description
                ));
            }
            html.push_str("</ul>\n");
        }

        html.push_str("</div>\n");
        html
    }

    /// 构建Overdraw表格
    fn build_overdraw_table(&self, analysis: &super::render_analyzer::OverdrawAnalysis) -> String {
        let severity = match analysis.severity {
            super::render_analyzer::OverdrawSeverity::None => {
                crate::performance::profiler::Severity::Low
            }
            super::render_analyzer::OverdrawSeverity::Moderate => {
                crate::performance::profiler::Severity::Medium
            }
            super::render_analyzer::OverdrawSeverity::Severe => {
                crate::performance::profiler::Severity::Critical
            }
        };

        format!(
            "<table class=\"data-table\">\n\
             <tr><th>指标</th><th>值</th></tr>\n\
             <tr><td>平均Overdraw</td><td>{:.2}x</td></tr>\n\
             <tr><td>最大Overdraw</td><td>{:.2}x</td></tr>\n\
             <tr><td>严重程度</td><td class=\"{}\">{:?}</td></tr>\n\
             </table>\n",
            analysis.average_overdraw,
            analysis.max_overdraw,
            format_severity(severity),
            analysis.severity
        )
    }

    /// 构建带宽表格
    fn build_bandwidth_table(
        &self,
        analysis: &super::render_analyzer::BandwidthAnalysis,
    ) -> String {
        format!(
            "<table class=\"data-table\">\n\
             <tr><th>指标</th><th>值</th></tr>\n\
             <tr><td>平均总带宽</td><td>{:.2} MB/frame</td></tr>\n\
             <tr><td>峰值带宽</td><td>{:.2} MB/frame</td></tr>\n\
             <tr><td>纹理带宽</td><td>{:.2} MB/frame</td></tr>\n\
             <tr><td>顶点带宽</td><td>{:.2} MB/frame</td></tr>\n\
             </table>\n",
            analysis.average_total as f64 / 1_000_000.0,
            analysis.peak_total as f64 / 1_000_000.0,
            analysis.breakdown.textures as f64 / 1_000_000.0,
            analysis.breakdown.vertices as f64 / 1_000_000.0
        )
    }

    /// 构建Pipeline表格
    fn build_pipeline_table(&self, analysis: &super::render_analyzer::PipelineAnalysis) -> String {
        let most_frequent = analysis.most_frequent_change.as_deref().unwrap_or("None");

        format!(
            "<table class=\"data-table\">\n\
             <tr><th>指标</th><th>值</th></tr>\n\
             <tr><td>总状态变化</td><td>{}</td></tr>\n\
             <tr><td>每帧变化</td><td>{:.2}</td></tr>\n\
             <tr><td>最频繁变化</td><td>{}</td></tr>\n\
             </table>\n",
            analysis.total_changes, analysis.changes_per_frame, most_frequent
        )
    }

    /// 构建内存部分
    fn build_memory_section(&self, report: &MemoryBottleneckReport) -> String {
        let mut html = String::new();
        html.push_str("<div class=\"section\">\n");
        html.push_str("<h2>内存分析</h2>\n");

        // 泄漏列表
        if !report.leaks.is_empty() {
            html.push_str("<h3>内存泄漏</h3>\n");
            html.push_str("<ul class=\"leak-list\">\n");
            for leak in &report.leaks {
                html.push_str(&format!(
                    "<li class=\"leak {}\">{}: {}个对象, {:.2} MB</li>\n",
                    format_leak_severity(leak.severity),
                    leak.leak_type,
                    leak.leak_count,
                    leak.total_size as f64 / 1_000_000.0
                ));
            }
            html.push_str("</ul>\n");
        }

        // 碎片化
        html.push_str("<h3>内存碎片化</h3>\n");
        html.push_str(&format!(
            "<p>碎片化率: {:.1}% ({:?})</p>\n",
            report.fragmentation_report.current_fragmentation * 100.0,
            report.fragmentation_report.severity
        ));

        // 分配热点
        if !report.hotspot_report.top_allocation_types.is_empty() {
            html.push_str("<h3>分配热点</h3>\n");
            html.push_str("<table class=\"data-table\">\n");
            html.push_str("<tr><th>类型</th><th>分配次数</th><th>总大小</th></tr>\n");
            for (name, stats) in &report.hotspot_report.top_allocation_types {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{:.2} MB</td></tr>\n",
                    name,
                    stats.allocation_count,
                    stats.total_size as f64 / 1_000_000.0
                ));
            }
            html.push_str("</table>\n");
        }

        html.push_str("</div>\n");
        html
    }

    /// 构建优化建议部分
    fn build_optimization_section(&self, report: &OptimizationReport) -> String {
        let mut html = String::new();
        html.push_str("<div class=\"section\">\n");
        html.push_str("<h2>优化建议</h2>\n");

        // 总体评估
        html.push_str("<h3>总体评估</h3>\n");
        html.push_str(&format!(
            "<div class=\"health-score score-{}\">\n\
             <div class=\"score-value\">{}</div>\n\
             <div class=\"score-label\">健康得分</div>\n\
             </div>\n",
            score_class(report.overall_assessment.health_score),
            report.overall_assessment.health_score
        ));

        // 建议列表
        html.push_str("<h3>推荐优化</h3>\n");
        html.push_str("<div class=\"suggestions-list\">\n");
        for suggestion in &report.suggestions {
            html.push_str(&self.build_suggestion_card(suggestion));
        }
        html.push_str("</div>\n");

        html.push_str("</div>\n");
        html
    }

    /// 构建建议卡片
    fn build_suggestion_card(&self, suggestion: &OptimizationSuggestion) -> String {
        format!(
            "<div class=\"suggestion-card {} {}\">\n\
             <h4>{}</h4>\n\
             <p class=\"description\">{}</p>\n\
             <p><strong>预期改进:</strong> {}</p>\n\
             <p><strong>预计工作量:</strong> {} 小时</p>\n\
             <p><strong>风险等级:</strong> {:?}</p>\n\
             <p><strong>可自动修复:</strong> {}</p>\n\
             </div>\n",
            format_severity(suggestion.severity),
            format_category(suggestion.category.clone()),
            suggestion.title,
            suggestion.description,
            suggestion.expected_improvement,
            suggestion.estimated_effort_hours,
            suggestion.risk_level,
            if suggestion.can_auto_fix {
                "是"
            } else {
                "否"
            }
        )
    }

    /// 构建自动修复部分
    fn build_auto_fix_section(&self, results: &[AutoFixResult]) -> String {
        let mut html = String::new();
        html.push_str("<div class=\"section\">\n");
        html.push_str("<h2>自动修复结果</h2>\n");
        html.push_str("<ul class=\"autofix-list\">\n");

        for result in results {
            match result {
                AutoFixResult::Success {
                    optimization_id,
                    improvement_description,
                    ..
                } => {
                    html.push_str(&format!(
                        "<li class=\"autofix success\">✓ {}: {}</li>\n",
                        optimization_id, improvement_description
                    ));
                }
                AutoFixResult::Skipped { reason } => {
                    html.push_str(&format!(
                        "<li class=\"autofix skipped\">⊘ 跳过: {}</li>\n",
                        reason
                    ));
                }
                AutoFixResult::Failed { error } => {
                    html.push_str(&format!(
                        "<li class=\"autofix failed\">✗ 失败: {}</li>\n",
                        error
                    ));
                }
            }
        }

        html.push_str("</ul>\n");
        html.push_str("</div>\n");
        html
    }

    /// 构建详细日志
    fn build_detailed_logs(&self, data: &PerformanceReportData) -> String {
        "<div class=\"section\">\n\
         <h2>详细日志</h2>\n\
         <pre class=\"logs\">\n\
         性能分析已完成\n\
         会话时长: "
            .to_string()
            + &data.session_duration_seconds.to_string()
            + " 秒\n\
         </pre>\n\
         </div>\n"
    }

    /// 构建尾部
    fn build_footer(&self, data: &PerformanceReportData) -> String {
        format!(
            "<div class=\"footer\">\n\
             <p>由游戏引擎性能分析器生成</p>\n\
             <p>生成时间: {}</p>\n\
             </div>\n",
            format_system_time(data.generated_at)
        )
    }

    /// 获取默认CSS样式
    fn get_default_styles() -> String {
        include_str!("report_styles.css").to_string()
    }
}

// ==================== PDF报告构建器 ====================

/// PDF报告构建器（简化版，实际使用需要PDF库）
pub struct PdfReportBuilder;

impl PdfReportBuilder {
    fn new() -> Self {
        Self
    }

    /// 构建PDF报告
    fn build(
        &self,
        _data: &PerformanceReportData,
        _config: &ReportConfig,
    ) -> Result<GeneratedReport, ReportError> {
        // 简化版：返回占位符
        // 实际实现需要使用 printpdf 或 lopdf 等库
        Err(ReportError::UnsupportedFormat)
    }
}

// ==================== 辅助函数 ====================

/// 格式化系统时间
fn format_system_time(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    use std::time::{Duration, UNIX_EPOCH};

    if let Some(datetime) = UNIX_EPOCH.checked_add(Duration::from_secs(secs)) {
        // 简化版时间格式
        format!("{:?}", datetime)
    } else {
        "Unknown".to_string()
    }
}

/// 格式化严重程度为CSS类
fn format_severity(severity: crate::performance::profiler::Severity) -> &'static str {
    match severity {
        crate::performance::profiler::Severity::Low => "severity-low",
        crate::performance::profiler::Severity::Medium => "severity-medium",
        crate::performance::profiler::Severity::High => "severity-high",
        crate::performance::profiler::Severity::Critical => "severity-critical",
    }
}

/// 格式化泄漏严重程度
fn format_leak_severity(severity: super::memory_analyzer::LeakSeverity) -> &'static str {
    match severity {
        super::memory_analyzer::LeakSeverity::Moderate => "leak-moderate",
        super::memory_analyzer::LeakSeverity::High => "leak-high",
        super::memory_analyzer::LeakSeverity::Critical => "leak-critical",
    }
}

/// 格式化类别
fn format_category(category: super::optimization_suggestion::SuggestionCategory) -> &'static str {
    match category {
        super::optimization_suggestion::SuggestionCategory::Rendering => "category-rendering",
        super::optimization_suggestion::SuggestionCategory::Memory => "category-memory",
        super::optimization_suggestion::SuggestionCategory::CPU => "category-cpu",
        super::optimization_suggestion::SuggestionCategory::Resource => "category-resource",
        super::optimization_suggestion::SuggestionCategory::CodeQuality => "category-codequality",
        super::optimization_suggestion::SuggestionCategory::Architecture => "category-architecture",
    }
}

/// 健康得分CSS类
fn score_class(score: u32) -> &'static str {
    if score >= 80 {
        "score-good"
    } else if score >= 60 {
        "score-medium"
    } else {
        "score-poor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_generator_creation() {
        let generator = ReportGenerator::new();
        // 测试生成器创建
        assert!(generator.html_builder.styles.len() > 0);
    }
}
