// 覆盖图可视化调试工具
//
// 生成覆盖图的可视化表示

use super::influence_map::{InfluenceGrid, TacticalInfluenceMap};
use std::fmt::Write;

/// 覆盖图可视化器
pub struct InfluenceVisualizer {
    width: usize,
    height: usize,
    cell_size: usize,
}

impl InfluenceVisualizer {
    /// 创建新的可视化器
    pub fn new(width: usize, height: usize, cell_size: usize) -> Self {
        Self {
            width,
            height,
            cell_size,
        }
    }

    /// 生成ASCII艺术表示
    pub fn to_ascii(&self, grid: &InfluenceGrid) -> String {
        let mut output = String::new();

        writeln!(output, "Influence Map ({}x{})", grid.width(), grid.height()).unwrap();
        writeln!(output, "{}", "=".repeat(grid.width() + 2)).unwrap();

        for y in 0..grid.height() {
            write!(output, "|").unwrap();
            for x in 0..grid.width() {
                let value = grid.get(x, y);
                let ch = Self::value_to_char(value);
                write!(output, "{}", ch).unwrap();
            }
            writeln!(output, "|").unwrap();
        }

        writeln!(output, "{}", "=".repeat(grid.width() + 2)).unwrap();

        // 添加图例
        writeln!(output, "Legend:").unwrap();
        writeln!(output, "  '-' = Negative (< -0.5)").unwrap();
        writeln!(output, "  '.' = Weak (-0.5 to -0.2)").unwrap();
        writeln!(output, "  'o' = Neutral (-0.2 to 0.2)").unwrap();
        writeln!(output, "  'O' = Positive (0.2 to 0.5)").unwrap();
        writeln!(output, "  '@' = Strong (> 0.5)").unwrap();

        output
    }

    /// 将数值转换为ASCII字符
    fn value_to_char(value: f32) -> char {
        if value < -0.5 {
            '-'
        } else if value < -0.2 {
            '.'
        } else if value < 0.2 {
            'o'
        } else if value < 0.5 {
            'O'
        } else {
            '@'
        }
    }

    /// 生成热力图（ANSI颜色）
    pub fn to_ansi_heatmap(&self, grid: &InfluenceGrid) -> String {
        let mut output = String::new();

        writeln!(output, "\x1b[1mInfluence Map Heatmap\x1b[0m").unwrap();
        writeln!(output, "Size: {}x{}", grid.width(), grid.height()).unwrap();
        writeln!(output).unwrap();

        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let value = grid.get(x, y);
                let color = Self::value_to_color(value);
                write!(output, "{}  ", color).unwrap();
            }
            writeln!(output).unwrap();
        }

        // 添加颜色图例
        writeln!(output).unwrap();
        writeln!(output, "\x1b[1mLegend:\x1b[0m").unwrap();
        writeln!(output, "  \x1b[31m█\x1b[0m Red = Negative (< 0)").unwrap();
        writeln!(output, "  \x1b[37m█\x1b[0m White = Neutral (≈ 0)").unwrap();
        writeln!(output, "  \x1b[32m█\x1b[0m Green = Positive (> 0)").unwrap();

        output
    }

    /// 将数值转换为ANSI颜色代码
    fn value_to_color(value: f32) -> String {
        if value < -0.5 {
            "\x1b[31m█\x1b[0m".to_string() // 红色（强负）
        } else if value < -0.2 {
            "\x1b[33m█\x1b[0m".to_string() // 黄色（弱负）
        } else if value < 0.2 {
            "\x1b[37m█\x1b[0m".to_string() // 白色（中性）
        } else if value < 0.5 {
            "\x1b[32m▓\x1b[0m".to_string() // 浅绿（弱正）
        } else {
            "\x1b[32m█\x1b[0m".to_string() // 绿色（强正）
        }
    }

    /// 生成统计信息
    pub fn statistics(&self, grid: &InfluenceGrid) -> InfluenceStatistics {
        let mut sum = 0.0;
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut zero_count = 0;

        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let value = grid.get(x, y);
                sum += value;
                min = min.min(value);
                max = max.max(value);

                if value > 0.01 {
                    positive_count += 1;
                } else if value < -0.01 {
                    negative_count += 1;
                } else {
                    zero_count += 1;
                }
            }
        }

        let count = (grid.width() * grid.height()) as f32;
        let mean = sum / count;

        let mut variance_sum = 0.0;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let diff = grid.get(x, y) - mean;
                variance_sum += diff * diff;
            }
        }
        let std_dev = (variance_sum / count).sqrt();

        InfluenceStatistics {
            width: grid.width(),
            height: grid.height(),
            min,
            max,
            mean,
            std_dev,
            positive_count,
            negative_count,
            zero_count,
        }
    }

    /// 可视化战术覆盖图
    pub fn visualize_tactical(&self, tactical: &TacticalInfluenceMap) -> String {
        let mut output = String::new();

        writeln!(output, "\x1b[1m=== Tactical Influence Map ===\x1b[0m").unwrap();
        writeln!(output).unwrap();

        // 领土控制
        writeln!(output, "\x1b[1mTerritory Control:\x1b[0m").unwrap();
        writeln!(output, "{}", self.to_ascii(&tactical.territory)).unwrap();

        // 危险区域
        writeln!(output, "\x1b[1mDanger Areas:\x1b[0m").unwrap();
        writeln!(output, "{}", self.to_ascii(&tactical.danger)).unwrap();

        // 机会区域
        writeln!(output, "\x1b[1mOpportunity Areas:\x1b[0m").unwrap();
        writeln!(output, "{}", self.to_ascii(&tactical.opportunity)).unwrap();

        // 统计信息
        writeln!(output, "\x1b[1mStatistics:\x1b[0m").unwrap();
        writeln!(output, "{:?}", self.statistics(&tactical.territory)).unwrap();

        output
    }

    /// 生成SVG格式可视化
    pub fn to_svg(&self, grid: &InfluenceGrid, title: &str) -> String {
        let mut svg = String::new();

        let svg_width = grid.width() * self.cell_size;
        let svg_height = grid.height() * self.cell_size;

        writeln!(svg, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();
        writeln!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
            svg_width, svg_height
        )
        .unwrap();
        writeln!(svg, "<style>").unwrap();
        writeln!(
            svg,
            "  text {{ font-family: Arial, sans-serif; font-size: 12px; }}"
        )
        .unwrap();
        writeln!(svg, "</style>").unwrap();

        // 标题
        writeln!(svg, r#"<text x="50%" y="20" text-anchor="middle" font-size="16" font-weight="bold">{}</text>"#, title).unwrap();

        // 绘制网格
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let value = grid.get(x, y);
                let color = Self::value_to_rgb(value);

                let px = x * self.cell_size;
                let py = y * self.cell_size + 30; // 偏移以显示标题

                writeln!(
                    svg,
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="rgb({}, {}, {})" stroke="none"/>"#,
                    px, py, self.cell_size, self.cell_size, color.0, color.1, color.2
                ).unwrap();
            }
        }

        // 颜色图例
        writeln!(svg, r#"<defs>"#).unwrap();
        writeln!(
            svg,
            r#"<linearGradient id="legend" x1="0%" y1="0%" x2="100%" y2="0%">"#
        )
        .unwrap();
        writeln!(
            svg,
            r#"<stop offset="0%" style="stop-color:rgb(255,0,0);stop-opacity:1" />"#
        )
        .unwrap();
        writeln!(
            svg,
            r#"<stop offset="50%" style="stop-color:rgb(255,255,255);stop-opacity:1" />"#
        )
        .unwrap();
        writeln!(
            svg,
            r#"<stop offset="100%" style="stop-color:rgb(0,255,0);stop-opacity:1" />"#
        )
        .unwrap();
        writeln!(svg, r#"</linearGradient>"#).unwrap();
        writeln!(svg, r#"</defs>"#).unwrap();
        // 颜色图例 - 确保不会溢出
        let legend_x = if svg_width > 200 {
            svg_width / 2 - 100
        } else {
            0
        };
        writeln!(
            svg,
            r#"<rect x="{}" y="{}" width="200" height="20" fill="url(#legend)" />"#,
            legend_x,
            svg_height + 40
        )
        .unwrap();
        writeln!(
            svg,
            r#"<text x="{}" y="{}" text-anchor="middle">Negative</text>"#,
            legend_x,
            svg_height + 70
        )
        .unwrap();
        writeln!(
            svg,
            r#"<text x="{}" y="{}" text-anchor="middle">Positive</text>"#,
            legend_x + 200,
            svg_height + 70
        )
        .unwrap();

        writeln!(svg, "</svg>").unwrap();

        svg
    }

    /// 将数值转换为RGB颜色
    fn value_to_rgb(value: f32) -> (u8, u8, u8) {
        // 归一化到 [-1, 1]
        let normalized = value.max(-1.0).min(1.0);

        if normalized < 0.0 {
            // 负值：红色渐变
            let intensity = (-normalized * 255.0) as u8;
            (intensity, 0, 0)
        } else {
            // 正值：绿色渐变
            let intensity = (normalized * 255.0) as u8;
            (0, intensity, 0)
        }
    }
}

/// 覆盖图统计信息
#[derive(Debug)]
pub struct InfluenceStatistics {
    pub width: usize,
    pub height: usize,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub std_dev: f32,
    pub positive_count: usize,
    pub negative_count: usize,
    pub zero_count: usize,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_visualization() {
        let mut grid = InfluenceGrid::new(10, 10, 1.0);
        grid.add_source(5, 5, 100.0);

        let viz = InfluenceVisualizer::new(10, 10, 10);
        let ascii = viz.to_ascii(&grid);

        assert!(ascii.contains("Influence Map"));
        assert!(ascii.contains("@")); // 应该有强正值
    }

    #[test]
    fn test_statistics() {
        let mut grid = InfluenceGrid::new(10, 10, 1.0);
        grid.add_source(5, 5, 100.0);

        let viz = InfluenceVisualizer::new(10, 10, 10);
        let stats = viz.statistics(&grid);

        assert_eq!(stats.width, 10);
        assert_eq!(stats.height, 10); // Grid的实际高度是10
        assert!(stats.max > 0.0);
    }

    #[test]
    fn test_svg_generation() {
        let mut grid = InfluenceGrid::new(5, 5, 1.0);
        grid.add_source(2, 2, 50.0);

        let viz = InfluenceVisualizer::new(5, 5, 20);
        let svg = viz.to_svg(&grid, "Test Grid");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Grid"));
    }
}
