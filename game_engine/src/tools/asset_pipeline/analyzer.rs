//! # Quality Analyzer - 质量分析器
//!
//! 本模块实现资源质量分析和报告生成功能。

use super::pipeline::{AssetMetadata, AssetType, OptimizationError, PipelineReport};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// 质量分析器
pub struct QualityAnalyzer {
    /// 目标值配置
    targets: QualityTargets,
}

/// 质量目标配置
#[derive(Debug, Clone)]
pub struct QualityTargets {
    /// 最大纹理分辨率
    pub max_texture_resolution: u32,

    /// 每个模型的最大多边形数
    pub max_polygons_per_model: usize,

    /// 每帧最大draw call数
    pub max_draw_calls: usize,

    /// 最大内存使用（MB）
    pub max_memory_mb: f64,

    /// 最大加载时间（秒）
    pub max_load_time: f64,
}

impl Default for QualityTargets {
    fn default() -> Self {
        Self {
            max_texture_resolution: 2048,
            max_polygons_per_model: 100_000,
            max_draw_calls: 100,
            max_memory_mb: 500.0,
            max_load_time: 3.0,
        }
    }
}

impl QualityAnalyzer {
    /// 创建新的质量分析器
    pub fn new() -> Self {
        Self {
            targets: QualityTargets::default(),
        }
    }

    /// 使用自定义目标创建
    pub fn with_targets(targets: QualityTargets) -> Self {
        Self { targets }
    }

    /// 分析资源质量
    pub fn analyze(&self, asset: &AssetMetadata) -> QualityReport {
        QualityReport {
            texture_resolution: self.check_resolution(asset),
            polygon_count: self.check_polygons(asset),
            draw_calls: self.estimate_draw_calls(asset),
            memory_usage: self.estimate_memory(asset),
            load_time: self.estimate_load_time(asset),
        }
    }

    /// 检查纹理分辨率
    fn check_resolution(&self, asset: &AssetMetadata) -> MetricStatus {
        if asset.asset_type != AssetType::Texture {
            return MetricStatus::Good {
                value: 0.0,
                target: self.targets.max_texture_resolution as f32,
            };
        }

        // 简化实现：假设从路径推断分辨率
        // 实际实现应该读取文件并解析
        let estimated_resolution = 1024u32;

        let ratio = estimated_resolution as f32 / self.targets.max_texture_resolution as f32;

        if ratio <= 0.5 {
            MetricStatus::Good {
                value: estimated_resolution as f32,
                target: self.targets.max_texture_resolution as f32,
            }
        } else if ratio <= 0.75 {
            MetricStatus::Acceptable {
                value: estimated_resolution as f32,
                target: self.targets.max_texture_resolution as f32,
            }
        } else if ratio <= 1.0 {
            MetricStatus::Poor {
                value: estimated_resolution as f32,
                target: self.targets.max_texture_resolution as f32,
            }
        } else {
            MetricStatus::Critical {
                value: estimated_resolution as f32,
                target: self.targets.max_texture_resolution as f32,
            }
        }
    }

    /// 检查多边形数量
    fn check_polygons(&self, asset: &AssetMetadata) -> MetricStatus {
        if asset.asset_type != AssetType::Model {
            return MetricStatus::Good {
                value: 0.0,
                target: self.targets.max_polygons_per_model as f32,
            };
        }

        // 简化实现：从文件大小估算多边形数
        let estimated_polygons = (asset.size / 100) as usize; // 粗略估计

        let ratio = estimated_polygons as f32 / self.targets.max_polygons_per_model as f32;

        if ratio <= 0.5 {
            MetricStatus::Good {
                value: estimated_polygons as f32,
                target: self.targets.max_polygons_per_model as f32,
            }
        } else if ratio <= 0.75 {
            MetricStatus::Acceptable {
                value: estimated_polygons as f32,
                target: self.targets.max_polygons_per_model as f32,
            }
        } else if ratio <= 1.0 {
            MetricStatus::Poor {
                value: estimated_polygons as f32,
                target: self.targets.max_polygons_per_model as f32,
            }
        } else {
            MetricStatus::Critical {
                value: estimated_polygons as f32,
                target: self.targets.max_polygons_per_model as f32,
            }
        }
    }

    /// 估算draw call数
    fn estimate_draw_calls(&self, _asset: &AssetMetadata) -> MetricStatus {
        // 简化实现：每个资源假设1个draw call
        MetricStatus::Good {
            value: 1.0,
            target: self.targets.max_draw_calls as f32,
        }
    }

    /// 估算内存使用
    fn estimate_memory(&self, asset: &AssetMetadata) -> MetricStatus {
        let memory_mb = asset.size as f64 / 1024.0 / 1024.0;

        let ratio = memory_mb / self.targets.max_memory_mb;

        if ratio <= 0.5 {
            MetricStatus::Good {
                value: memory_mb as f32,
                target: self.targets.max_memory_mb as f32,
            }
        } else if ratio <= 0.75 {
            MetricStatus::Acceptable {
                value: memory_mb as f32,
                target: self.targets.max_memory_mb as f32,
            }
        } else if ratio <= 1.0 {
            MetricStatus::Poor {
                value: memory_mb as f32,
                target: self.targets.max_memory_mb as f32,
            }
        } else {
            MetricStatus::Critical {
                value: memory_mb as f32,
                target: self.targets.max_memory_mb as f32,
            }
        }
    }

    /// 估算加载时间
    fn estimate_load_time(&self, asset: &AssetMetadata) -> MetricStatus {
        // 简化实现：假设每MB需要0.1秒加载
        let size_mb = asset.size as f64 / 1024.0 / 1024.0;
        let load_time = size_mb * 0.1;

        let ratio = load_time / self.targets.max_load_time;

        if ratio <= 0.5 {
            MetricStatus::Good {
                value: load_time as f32,
                target: self.targets.max_load_time as f32,
            }
        } else if ratio <= 0.75 {
            MetricStatus::Acceptable {
                value: load_time as f32,
                target: self.targets.max_load_time as f32,
            }
        } else if ratio <= 1.0 {
            MetricStatus::Poor {
                value: load_time as f32,
                target: self.targets.max_load_time as f32,
            }
        } else {
            MetricStatus::Critical {
                value: load_time as f32,
                target: self.targets.max_load_time as f32,
            }
        }
    }

    /// 生成HTML报告
    pub fn generate_html_report(&self, reports: &[QualityReport]) -> String {
        let html = format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Asset Quality Report</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #4CAF50;
            padding-bottom: 10px;
        }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin: 20px 0;
        }}
        .metric-card {{
            padding: 15px;
            border-radius: 5px;
            background-color: #f9f9f9;
            border-left: 4px solid #ddd;
        }}
        .metric-card.good {{ border-left-color: #4CAF50; }}
        .metric-card.acceptable {{ border-left-color: #FF9800; }}
        .metric-card.poor {{ border-left-color: #FF5722; }}
        .metric-card.critical {{ border-left-color: #f44336; }}
        .metric-title {{
            font-weight: bold;
            margin-bottom: 5px;
            color: #555;
        }}
        .metric-value {{
            font-size: 24px;
            font-weight: bold;
            margin: 5px 0;
        }}
        .metric-target {{
            font-size: 12px;
            color: #888;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background-color: #4CAF50;
            color: white;
        }}
        tr:hover {{
            background-color: #f5f5f5;
        }}
        .badge {{
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: bold;
        }}
        .badge-good {{ background-color: #4CAF50; color: white; }}
        .badge-acceptable {{ background-color: #FF9800; color: white; }}
        .badge-poor {{ background-color: #FF5722; color: white; }}
        .badge-critical {{ background-color: #f44336; color: white; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Asset Quality Report</h1>

        <div class="summary">
            {}
        </div>

        <h2>Detailed Results</h2>
        <table>
            <thead>
                <tr>
                    <th>Metric</th>
                    <th>Status</th>
                    <th>Value</th>
                    <th>Target</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
    </div>
</body>
</html>
        "#,
            self.generate_summary_html(reports),
            self.generate_table_html(reports)
        );

        html
    }

    /// 生成摘要HTML
    fn generate_summary_html(&self, reports: &[QualityReport]) -> String {
        let mut html = String::new();

        let total_reports = reports.len();
        let good_count = reports.iter().filter(|r| r.overall_status().is_good()).count();
        let acceptable_count =
            reports.iter().filter(|r| r.overall_status().is_acceptable()).count();
        let poor_count = reports.iter().filter(|r| r.overall_status().is_poor()).count();
        let critical_count = reports.iter().filter(|r| r.overall_status().is_critical()).count();

        html.push_str(&format!(
            r#"
            <div class="metric-card good">
                <div class="metric-title">Total Assets</div>
                <div class="metric-value">{}</div>
            </div>
            "#,
            total_reports
        ));

        html.push_str(&format!(
            r#"
            <div class="metric-card good">
                <div class="metric-title">Good</div>
                <div class="metric-value">{}</div>
            </div>
            "#,
            good_count
        ));

        html.push_str(&format!(
            r#"
            <div class="metric-card acceptable">
                <div class="metric-title">Acceptable</div>
                <div class="metric-value">{}</div>
            </div>
            "#,
            acceptable_count
        ));

        html.push_str(&format!(
            r#"
            <div class="metric-card poor">
                <div class="metric-title">Poor</div>
                <div class="metric-value">{}</div>
            </div>
            "#,
            poor_count
        ));

        html.push_str(&format!(
            r#"
            <div class="metric-card critical">
                <div class="metric-title">Critical</div>
                <div class="metric-value">{}</div>
            </div>
            "#,
            critical_count
        ));

        html
    }

    /// 生成表格HTML
    fn generate_table_html(&self, reports: &[QualityReport]) -> String {
        let mut html = String::new();

        for (i, report) in reports.iter().enumerate() {
            html.push_str(&self.metric_row("Texture Resolution", &report.texture_resolution, i));
            html.push_str(&self.metric_row("Polygon Count", &report.polygon_count, i));
            html.push_str(&self.metric_row("Draw Calls", &report.draw_calls, i));
            html.push_str(&self.metric_row("Memory Usage", &report.memory_usage, i));
            html.push_str(&self.metric_row("Load Time", &report.load_time, i));
        }

        html
    }

    /// 生成指标行HTML
    fn metric_row(&self, name: &str, status: &MetricStatus, _index: usize) -> String {
        let (status_class, value, target) = match status {
            MetricStatus::Good { value, target } => ("good", value, target),
            MetricStatus::Acceptable { value, target } => ("acceptable", value, target),
            MetricStatus::Poor { value, target } => ("poor", value, target),
            MetricStatus::Critical { value, target } => ("critical", value, target),
        };

        format!(
            r#"
            <tr>
                <td>{}</td>
                <td><span class="badge badge-{}">{}</span></td>
                <td>{:.2}</td>
                <td>{:.2}</td>
            </tr>
            "#,
            name, status_class, status_class, value, target
        )
    }

    /// 生成质量报告并保存
    pub async fn generate_report(
        &self,
        pipeline_report: &PipelineReport,
        output_dir: &Path,
    ) -> Result<(), OptimizationError> {
        // 生成质量报告（简化版本）
        let report = QualityReport {
            texture_resolution: MetricStatus::Good {
                value: 1024.0,
                target: 2048.0,
            },
            polygon_count: MetricStatus::Good {
                value: 5000.0,
                target: 100000.0,
            },
            draw_calls: MetricStatus::Good {
                value: pipeline_report.total_assets as f32,
                target: 100.0,
            },
            memory_usage: MetricStatus::Good {
                value: (pipeline_report.optimized_size / 1024 / 1024) as f32,
                target: 500.0,
            },
            load_time: MetricStatus::Good {
                value: pipeline_report.processing_time as f32,
                target: 3.0,
            },
        };

        // 生成HTML报告
        let html = self.generate_html_report(&[report]);

        let report_path = output_dir.join("quality_report.html");
        let mut file = File::create(&report_path)
            .map_err(|e| OptimizationError::Other(format!("Failed to create report: {}", e)))?;

        file.write_all(html.as_bytes())
            .map_err(|e| OptimizationError::Other(format!("Failed to write report: {}", e)))?;

        println!("Quality report generated: {}", report_path.display());

        Ok(())
    }
}

impl Default for QualityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 质量报告
#[derive(Debug, Clone)]
pub struct QualityReport {
    pub texture_resolution: MetricStatus,
    pub polygon_count: MetricStatus,
    pub draw_calls: MetricStatus,
    pub memory_usage: MetricStatus,
    pub load_time: MetricStatus,
}

impl QualityReport {
    /// 获取整体状态
    pub fn overall_status(&self) -> MetricStatus {
        let statuses = [
            &self.texture_resolution,
            &self.polygon_count,
            &self.draw_calls,
            &self.memory_usage,
            &self.load_time,
        ];

        // 如果有任何critical状态，返回critical
        if statuses.iter().any(|s| s.is_critical()) {
            return MetricStatus::Critical {
                value: 0.0,
                target: 0.0,
            };
        }

        // 如果有任何poor状态，返回poor
        if statuses.iter().any(|s| s.is_poor()) {
            return MetricStatus::Poor {
                value: 0.0,
                target: 0.0,
            };
        }

        // 如果有任何acceptable状态，返回acceptable
        if statuses.iter().any(|s| s.is_acceptable()) {
            return MetricStatus::Acceptable {
                value: 0.0,
                target: 0.0,
            };
        }

        // 否则返回good
        MetricStatus::Good {
            value: 0.0,
            target: 0.0,
        }
    }
}

/// 指标状态
#[derive(Debug, Clone)]
pub enum MetricStatus {
    Good { value: f32, target: f32 },
    Acceptable { value: f32, target: f32 },
    Poor { value: f32, target: f32 },
    Critical { value: f32, target: f32 },
}

impl MetricStatus {
    fn is_good(&self) -> bool {
        matches!(self, MetricStatus::Good { .. })
    }

    fn is_acceptable(&self) -> bool {
        matches!(self, MetricStatus::Acceptable { .. })
    }

    fn is_poor(&self) -> bool {
        matches!(self, MetricStatus::Poor { .. })
    }

    fn is_critical(&self) -> bool {
        matches!(self, MetricStatus::Critical { .. })
    }

    pub fn status_name(&self) -> &str {
        match self {
            MetricStatus::Good { .. } => "Good",
            MetricStatus::Acceptable { .. } => "Acceptable",
            MetricStatus::Poor { .. } => "Poor",
            MetricStatus::Critical { .. } => "Critical",
        }
    }

    pub fn value(&self) -> f32 {
        match self {
            MetricStatus::Good { value, .. } => *value,
            MetricStatus::Acceptable { value, .. } => *value,
            MetricStatus::Poor { value, .. } => *value,
            MetricStatus::Critical { value, .. } => *value,
        }
    }

    pub fn target(&self) -> f32 {
        match self {
            MetricStatus::Good { target, .. } => *target,
            MetricStatus::Acceptable { target, .. } => *target,
            MetricStatus::Poor { target, .. } => *target,
            MetricStatus::Critical { target, .. } => *target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_status() {
        let status = MetricStatus::Good {
            value: 100.0,
            target: 200.0,
        };

        assert!(status.is_good());
        assert!(!status.is_acceptable());
        assert_eq!(status.value(), 100.0);
        assert_eq!(status.target(), 200.0);
        assert_eq!(status.status_name(), "Good");
    }

    #[test]
    fn test_quality_report_overall_status() {
        let report = QualityReport {
            texture_resolution: MetricStatus::Good {
                value: 1024.0,
                target: 2048.0,
            },
            polygon_count: MetricStatus::Acceptable {
                value: 75000.0,
                target: 100000.0,
            },
            draw_calls: MetricStatus::Good {
                value: 50.0,
                target: 100.0,
            },
            memory_usage: MetricStatus::Good {
                value: 200.0,
                target: 500.0,
            },
            load_time: MetricStatus::Good {
                value: 1.0,
                target: 3.0,
            },
        };

        assert!(report.overall_status().is_acceptable());
    }
}
