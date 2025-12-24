//  性能分析和报告生成工具
//
//  分析性能数据并生成详细报告
//  - 性能分析
//  - 瓶颈检测
//  - HTML 报告生成
//  - 对标对比

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

//  性能分析结果
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// 分析名称
    pub name: String,
    /// 指标数据
    pub metrics: HashMap<String, f64>,
    /// 瓶颈列表
    pub bottlenecks: Vec<Bottleneck>,
    /// 建议列表
    pub recommendations: Vec<String>,
}

//  瓶颈信息
#[derive(Debug, Clone)]
pub struct Bottleneck {
    /// 瓶颈名称
    pub name: String,
    /// 严重程度 (0-100)
    pub severity: u32,
    /// 描述
    pub description: String,
    /// 建议的优化
    pub suggestion: String,
}

//  性能分析器
pub struct PerformanceAnalyzer {
    /// 分析结果
    analyses: HashMap<String, PerformanceAnalysis>,
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceAnalyzer {
    /// 创建新的性能分析器
    pub fn new() -> Self {
        Self {
            analyses: HashMap::new(),
        }
    }

    /// 分析音频性能
    pub fn analyze_audio(&mut self, fps: f32, memory_mb: f32) -> PerformanceAnalysis {
        let mut analysis = PerformanceAnalysis {
            name: "Audio Performance".to_string(),
            metrics: HashMap::new(),
            bottlenecks: Vec::new(),
            recommendations: Vec::new(),
        };

        // 记录指标
        analysis.metrics.insert("fps".to_string(), fps as f64);
        analysis.metrics.insert("memory_mb".to_string(), memory_mb as f64);

        // 检测瓶颈
        if fps < 30.0 {
            analysis.bottlenecks.push(Bottleneck {
                name: "Low Frame Rate".to_string(),
                severity: 90,
                description: "Frame rate is below 30 FPS".to_string(),
                suggestion: "Consider reducing audio processing quality or batch size".to_string(),
            });
        }

        if memory_mb > 500.0 {
            analysis.bottlenecks.push(Bottleneck {
                name: "High Memory Usage".to_string(),
                severity: 70,
                description: format!("Memory usage is {:.2} MB", memory_mb),
                suggestion: "Reduce number of active audio sources or use compression".to_string(),
            });
        }

        // 生成建议
        if fps > 60.0 && memory_mb < 100.0 {
            analysis.recommendations.push(
                "Audio performance is excellent. Consider enabling advanced effects.".to_string(),
            );
        }

        self.analyses.insert(analysis.name.clone(), analysis.clone());
        analysis
    }

    /// 分析物理性能
    pub fn analyze_physics(
        &mut self,
        fps: f32,
        memory_mb: f32,
        body_count: u32,
    ) -> PerformanceAnalysis {
        let mut analysis = PerformanceAnalysis {
            name: "Physics Performance".to_string(),
            metrics: HashMap::new(),
            bottlenecks: Vec::new(),
            recommendations: Vec::new(),
        };

        analysis.metrics.insert("fps".to_string(), fps as f64);
        analysis.metrics.insert("memory_mb".to_string(), memory_mb as f64);
        analysis.metrics.insert("body_count".to_string(), body_count as f64);

        // 物理性能分析
        let fps_per_body = fps / body_count.max(1) as f32;

        if fps_per_body < 0.1 {
            analysis.bottlenecks.push(Bottleneck {
                name: "Physics Simulation Too Slow".to_string(),
                severity: 85,
                description: format!(
                    "Physics performance degrading: {:.4} FPS per body",
                    fps_per_body
                ),
                suggestion: "Reduce body count or use GPU physics acceleration".to_string(),
            });
        }

        if body_count > 10000 {
            analysis.bottlenecks.push(Bottleneck {
                name: "Too Many Physics Bodies".to_string(),
                severity: 75,
                description: format!("Simulating {} physics bodies", body_count),
                suggestion: "Use batch processing or GPU compute for large body counts".to_string(),
            });
        }

        self.analyses.insert(analysis.name.clone(), analysis.clone());
        analysis
    }

    /// 分析 AI 性能
    pub fn analyze_ai(
        &mut self,
        fps: f32,
        agent_count: u32,
        path_length_avg: f32,
    ) -> PerformanceAnalysis {
        let mut analysis = PerformanceAnalysis {
            name: "AI Performance".to_string(),
            metrics: HashMap::new(),
            bottlenecks: Vec::new(),
            recommendations: Vec::new(),
        };

        analysis.metrics.insert("fps".to_string(), fps as f64);
        analysis.metrics.insert("agent_count".to_string(), agent_count as f64);
        analysis.metrics.insert("avg_path_length".to_string(), path_length_avg as f64);

        // AI 性能分析
        if fps < 60.0 && agent_count > 100 {
            analysis.bottlenecks.push(Bottleneck {
                name: "High Agent Load".to_string(),
                severity: 80,
                description: format!("Too many agents ({}) for {:.1} FPS", agent_count, fps),
                suggestion: "Use batch pathfinding or reduce update frequency".to_string(),
            });
        }

        if path_length_avg > 1000.0 {
            analysis.bottlenecks.push(Bottleneck {
                name: "Long Paths".to_string(),
                severity: 60,
                description: format!("Average path length is {:.0}", path_length_avg),
                suggestion: "Use hierarchical pathfinding or waypoint optimization".to_string(),
            });
        }

        self.analyses.insert(analysis.name.clone(), analysis.clone());
        analysis
    }

    /// 生成 HTML 报告
    pub fn generate_html_report<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Performance Analysis Report</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            border-bottom: 3px solid #007bff;
            padding-bottom: 10px;
        }
        h2 {
            color: #555;
            margin-top: 30px;
        }
        .analysis {
            margin-bottom: 30px;
            border-left: 4px solid #007bff;
            padding-left: 15px;
        }
        .metrics {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px;
            margin: 15px 0;
        }
        .metric {
            background-color: #f9f9f9;
            padding: 15px;
            border-radius: 5px;
            border-left: 4px solid #28a745;
        }
        .metric-label {
            font-weight: bold;
            color: #555;
        }
        .metric-value {
            font-size: 1.5em;
            color: #007bff;
            margin-top: 5px;
        }
        .bottlenecks {
            margin: 20px 0;
        }
        .bottleneck {
            background-color: #fff3cd;
            border: 1px solid #ffc107;
            border-radius: 5px;
            padding: 15px;
            margin-bottom: 10px;
        }
        .bottleneck.critical {
            background-color: #f8d7da;
            border-color: #f5c6cb;
        }
        .severity {
            display: inline-block;
            background-color: #dc3545;
            color: white;
            padding: 3px 8px;
            border-radius: 3px;
            font-weight: bold;
            margin-right: 10px;
        }
        .recommendation {
            background-color: #d4edda;
            border: 1px solid #c3e6cb;
            border-radius: 5px;
            padding: 10px;
            margin-bottom: 5px;
        }
        .timestamp {
            color: #999;
            font-size: 0.9em;
            margin-bottom: 20px;
        }
    </style>
</head>
<body>
<div class="container">
    <h1>🎮 Performance Analysis Report</h1>
"#,
        );

        html.push_str(&format!(
            "    <p class=\"timestamp\">Generated: {}</p>\n",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| format!("Timestamp: {}", d.as_secs()))
                .unwrap_or_default()
        ));

        // 添加每个分析
        for analysis in self.analyses.values() {
            html.push_str(&format!(
                "    <div class=\"analysis\">\n        <h2>{}</h2>\n",
                analysis.name
            ));

            // 添加指标
            html.push_str("        <div class=\"metrics\">\n");
            for (key, value) in &analysis.metrics {
                html.push_str(&format!(
                    "            <div class=\"metric\">\n                <div class=\"metric-label\">{}</div>\n                <div class=\"metric-value\">{:.2}</div>\n            </div>\n",
                    key, value
                ));
            }
            html.push_str("        </div>\n");

            // 添加瓶颈
            if !analysis.bottlenecks.is_empty() {
                html.push_str(
                    "        <div class=\"bottlenecks\">\n            <h3>⚠️ Bottlenecks</h3>\n",
                );
                for bottleneck in &analysis.bottlenecks {
                    let critical_class = if bottleneck.severity > 80 {
                        " critical"
                    } else {
                        ""
                    };
                    html.push_str(&format!(
                        "            <div class=\"bottleneck{}\">\n                <div><span class=\"severity\">{}</span> {}</div>\n                <p>{}</p>\n                <strong>Suggestion:</strong> {}\n            </div>\n",
                        critical_class,
                        bottleneck.severity,
                        bottleneck.name,
                        bottleneck.description,
                        bottleneck.suggestion
                    ));
                }
                html.push_str("        </div>\n");
            }

            // 添加建议
            if !analysis.recommendations.is_empty() {
                html.push_str("        <div>\n            <h3>✅ Recommendations</h3>\n");
                for rec in &analysis.recommendations {
                    html.push_str(&format!(
                        "            <div class=\"recommendation\">{}</div>\n",
                        rec
                    ));
                }
                html.push_str("        </div>\n");
            }

            html.push_str("    </div>\n");
        }

        html.push_str("</div>\n</body>\n</html>");

        fs::write(path, html)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_analysis() {
        let mut analyzer = PerformanceAnalyzer::new();
        let analysis = analyzer.analyze_audio(60.0, 100.0);

        assert_eq!(analysis.name, "Audio Performance");
        assert!(analysis.metrics.contains_key("fps"));
    }

    #[test]
    fn test_physics_analysis() {
        let mut analyzer = PerformanceAnalyzer::new();
        let analysis = analyzer.analyze_physics(30.0, 200.0, 5000);

        assert_eq!(analysis.name, "Physics Performance");
        assert!(analysis.bottlenecks.is_empty());
    }

    #[test]
    fn test_ai_analysis() {
        let mut analyzer = PerformanceAnalyzer::new();
        let analysis = analyzer.analyze_ai(45.0, 500, 800.0);

        assert_eq!(analysis.name, "AI Performance");
        assert!(analysis.metrics.contains_key("agent_count"));
    }

    #[test]
    fn test_bottleneck_severity() {
        let bottleneck = Bottleneck {
            name: "test".to_string(),
            severity: 85,
            description: "test description".to_string(),
            suggestion: "test suggestion".to_string(),
        };

        assert!(bottleneck.severity > 80);
    }

    #[test]
    fn test_html_report_generation() {
        let mut analyzer = PerformanceAnalyzer::new();
        analyzer.analyze_audio(60.0, 100.0);

        let report_path = "/tmp/test_performance_report.html";
        let result = analyzer.generate_html_report(report_path);

        assert!(result.is_ok());
        assert!(Path::new(report_path).exists());

        let content = fs::read_to_string(report_path).unwrap();
        assert!(content.contains("Performance Analysis Report"));
    }
}
