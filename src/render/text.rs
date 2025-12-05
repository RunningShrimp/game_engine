// ============================================================================
// MSDF (Multi-channel Signed Distance Field) 文本渲染系统
// 支持高质量缩放、描边、阴影、轮廓等效果
// ============================================================================

use crate::impl_default;
use std::collections::HashMap;

/// MSDF 字体资源
#[derive(Debug)]
pub struct MsdfFont {
    /// 字体名称
    pub name: String,
    /// 字体图集纹理句柄
    pub atlas_texture: u32,
    /// 图集尺寸
    pub atlas_size: [u32; 2],
    /// 距离场范围 (像素)
    pub distance_range: f32,
    /// 字形数据
    pub glyphs: HashMap<char, GlyphData>,
    /// 字距调整表
    pub kerning: HashMap<(char, char), f32>,
    /// 行高
    pub line_height: f32,
    /// 基线偏移
    pub ascender: f32,
    pub descender: f32,
}

/// 单个字形数据
#[derive(Debug, Clone, Copy)]
pub struct GlyphData {
    /// Unicode 码点
    pub unicode: u32,
    /// 字形前进宽度
    pub advance: f32,
    /// 平面坐标 (相对于基线)
    pub plane_bounds: Option<GlyphBounds>,
    /// 图集 UV 坐标
    pub atlas_bounds: Option<GlyphBounds>,
}

/// 边界框
#[derive(Debug, Clone, Copy)]
pub struct GlyphBounds {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

impl GlyphBounds {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }
}

/// 文本样式
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// 字体大小 (像素)
    pub font_size: f32,
    /// 文本颜色
    pub color: [f32; 4],
    /// 描边宽度 (0 = 无描边)
    pub stroke_width: f32,
    /// 描边颜色
    pub stroke_color: [f32; 4],
    /// 阴影偏移
    pub shadow_offset: [f32; 2],
    /// 阴影模糊半径
    pub shadow_blur: f32,
    /// 阴影颜色
    pub shadow_color: [f32; 4],
    /// 字间距调整
    pub letter_spacing: f32,
    /// 行间距倍数
    pub line_spacing: f32,
    /// 对齐方式
    pub alignment: TextAlignment,
    /// 是否启用软边缘
    pub soft_edges: bool,
}

impl_default!(TextStyle {
    font_size: 16.0,
    color: [1.0, 1.0, 1.0, 1.0],
    stroke_width: 0.0,
    stroke_color: [0.0, 0.0, 0.0, 1.0],
    shadow_offset: [0.0, 0.0],
    shadow_blur: 0.0,
    shadow_color: [0.0, 0.0, 0.0, 0.5],
    letter_spacing: 0.0,
    line_spacing: 1.0,
    alignment: TextAlignment::Left,
    soft_edges: true,
});

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

/// 文本布局结果
#[derive(Debug, Clone)]
pub struct TextLayout {
    /// 字形实例列表
    pub glyphs: Vec<GlyphInstance>,
    /// 总边界框
    pub bounds: [f32; 4], // x, y, width, height
    /// 行信息
    pub lines: Vec<LineInfo>,
}

/// 单个字形实例
#[derive(Debug, Clone, Copy)]
pub struct GlyphInstance {
    /// 位置 (左下角)
    pub position: [f32; 2],
    /// 尺寸
    pub size: [f32; 2],
    /// UV 坐标 (左下)
    pub uv_min: [f32; 2],
    /// UV 坐标 (右上)
    pub uv_max: [f32; 2],
    /// 颜色
    pub color: [f32; 4],
    /// 是否是阴影/描边层
    pub layer: GlyphLayer,
}

/// 字形层类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphLayer {
    Shadow,
    Stroke,
    Fill,
}

/// 行信息
#[derive(Debug, Clone)]
pub struct LineInfo {
    pub start_glyph: usize,
    pub end_glyph: usize,
    pub width: f32,
    pub baseline_y: f32,
}

impl MsdfFont {
    /// 从 JSON 元数据加载字体
    pub fn from_json(json_data: &str, atlas_texture: u32) -> Result<Self, String> {
        let data: serde_json::Value = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to parse font JSON: {}", e))?;

        let atlas = &data["atlas"];
        let metrics = &data["metrics"];

        let atlas_size = [
            atlas["width"].as_u64().unwrap_or(1024) as u32,
            atlas["height"].as_u64().unwrap_or(1024) as u32,
        ];

        let distance_range = atlas["distanceRange"].as_f64().unwrap_or(4.0) as f32;

        let line_height = metrics["lineHeight"].as_f64().unwrap_or(1.0) as f32;
        let ascender = metrics["ascender"].as_f64().unwrap_or(0.8) as f32;
        let descender = metrics["descender"].as_f64().unwrap_or(-0.2) as f32;

        let mut glyphs = HashMap::new();

        if let Some(glyph_array) = data["glyphs"].as_array() {
            for glyph in glyph_array {
                let unicode = glyph["unicode"].as_u64().unwrap_or(0) as u32;
                if let Some(ch) = char::from_u32(unicode) {
                    let advance = glyph["advance"].as_f64().unwrap_or(0.0) as f32;

                    let plane_bounds = glyph.get("planeBounds").map(|pb| GlyphBounds {
                        left: pb["left"].as_f64().unwrap_or(0.0) as f32,
                        bottom: pb["bottom"].as_f64().unwrap_or(0.0) as f32,
                        right: pb["right"].as_f64().unwrap_or(0.0) as f32,
                        top: pb["top"].as_f64().unwrap_or(0.0) as f32,
                    });

                    let atlas_bounds = glyph.get("atlasBounds").map(|ab| GlyphBounds {
                        left: ab["left"].as_f64().unwrap_or(0.0) as f32,
                        bottom: ab["bottom"].as_f64().unwrap_or(0.0) as f32,
                        right: ab["right"].as_f64().unwrap_or(0.0) as f32,
                        top: ab["top"].as_f64().unwrap_or(0.0) as f32,
                    });

                    glyphs.insert(
                        ch,
                        GlyphData {
                            unicode,
                            advance,
                            plane_bounds,
                            atlas_bounds,
                        },
                    );
                }
            }
        }

        let mut kerning = HashMap::new();

        if let Some(kerning_array) = data["kerning"].as_array() {
            for kern in kerning_array {
                let first = kern["unicode1"].as_u64().unwrap_or(0) as u32;
                let second = kern["unicode2"].as_u64().unwrap_or(0) as u32;
                let advance = kern["advance"].as_f64().unwrap_or(0.0) as f32;

                if let (Some(c1), Some(c2)) = (char::from_u32(first), char::from_u32(second)) {
                    kerning.insert((c1, c2), advance);
                }
            }
        }

        Ok(Self {
            name: data["name"].as_str().unwrap_or("Unknown").to_string(),
            atlas_texture,
            atlas_size,
            distance_range,
            glyphs,
            kerning,
            line_height,
            ascender,
            descender,
        })
    }

    /// 获取字形数据
    pub fn get_glyph(&self, ch: char) -> Option<&GlyphData> {
        self.glyphs.get(&ch)
    }

    /// 获取字距调整
    pub fn get_kerning(&self, first: char, second: char) -> f32 {
        self.kerning.get(&(first, second)).copied().unwrap_or(0.0)
    }
}

/// 文本排版器
#[derive(Default)]
pub struct TextLayouter {
    /// 缓存的字体
    fonts: HashMap<String, MsdfFont>,
}

impl TextLayouter {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册字体
    pub fn register_font(&mut self, name: String, font: MsdfFont) {
        self.fonts.insert(name, font);
    }

    /// 获取字体
    pub fn get_font(&self, name: &str) -> Option<&MsdfFont> {
        self.fonts.get(name)
    }

    /// 布局文本
    pub fn layout_text(
        &self,
        text: &str,
        font_name: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> Option<TextLayout> {
        let font = self.fonts.get(font_name)?;

        let scale = style.font_size / font.line_height;
        let line_height = font.line_height * scale * style.line_spacing;

        let mut glyphs = Vec::new();
        let mut lines = Vec::new();

        let mut cursor_x = 0.0f32;
        let mut cursor_y = -font.ascender * scale;
        let mut line_start = 0;
        let mut line_width = 0.0f32;
        let mut prev_char: Option<char> = None;

        for ch in text.chars() {
            if ch == '\n' {
                // 换行
                lines.push(LineInfo {
                    start_glyph: line_start,
                    end_glyph: glyphs.len(),
                    width: line_width,
                    baseline_y: cursor_y,
                });

                cursor_x = 0.0;
                cursor_y -= line_height;
                line_start = glyphs.len();
                line_width = 0.0;
                prev_char = None;
                continue;
            }

            let glyph_data = match font.get_glyph(ch) {
                Some(g) => g,
                None => font.get_glyph('?').unwrap_or_else(|| {
                    // 回退到空格
                    font.get_glyph(' ').unwrap()
                }),
            };

            // 应用字距调整
            if let Some(prev) = prev_char {
                cursor_x += font.get_kerning(prev, ch) * scale;
            }

            // 自动换行检查
            if let Some(max_w) = max_width {
                let advance = glyph_data.advance * scale + style.letter_spacing;
                if cursor_x + advance > max_w && cursor_x > 0.0 {
                    lines.push(LineInfo {
                        start_glyph: line_start,
                        end_glyph: glyphs.len(),
                        width: line_width,
                        baseline_y: cursor_y,
                    });

                    cursor_x = 0.0;
                    cursor_y -= line_height;
                    line_start = glyphs.len();
                    line_width = 0.0;
                }
            }

            // 生成字形实例
            if let (Some(plane), Some(atlas)) = (glyph_data.plane_bounds, glyph_data.atlas_bounds) {
                let x = cursor_x + plane.left * scale;
                let y = cursor_y + plane.bottom * scale;
                let w = plane.width() * scale;
                let h = plane.height() * scale;

                let atlas_size = font.atlas_size;
                let uv_min = [
                    atlas.left / atlas_size[0] as f32,
                    1.0 - atlas.top / atlas_size[1] as f32, // 翻转 Y
                ];
                let uv_max = [
                    atlas.right / atlas_size[0] as f32,
                    1.0 - atlas.bottom / atlas_size[1] as f32,
                ];

                // 阴影层
                if style.shadow_blur > 0.0 || style.shadow_offset != [0.0, 0.0] {
                    glyphs.push(GlyphInstance {
                        position: [x + style.shadow_offset[0], y + style.shadow_offset[1]],
                        size: [w, h],
                        uv_min,
                        uv_max,
                        color: style.shadow_color,
                        layer: GlyphLayer::Shadow,
                    });
                }

                // 描边层
                if style.stroke_width > 0.0 {
                    glyphs.push(GlyphInstance {
                        position: [x, y],
                        size: [w, h],
                        uv_min,
                        uv_max,
                        color: style.stroke_color,
                        layer: GlyphLayer::Stroke,
                    });
                }

                // 填充层
                glyphs.push(GlyphInstance {
                    position: [x, y],
                    size: [w, h],
                    uv_min,
                    uv_max,
                    color: style.color,
                    layer: GlyphLayer::Fill,
                });
            }

            cursor_x += glyph_data.advance * scale + style.letter_spacing;
            line_width = cursor_x;
            prev_char = Some(ch);
        }

        // 最后一行
        if line_start < glyphs.len() || lines.is_empty() {
            lines.push(LineInfo {
                start_glyph: line_start,
                end_glyph: glyphs.len(),
                width: line_width,
                baseline_y: cursor_y,
            });
        }

        // 计算边界框
        let total_height = lines.len() as f32 * line_height;
        let max_line_width = lines.iter().map(|l| l.width).fold(0.0f32, f32::max);

        // 应用对齐
        if style.alignment != TextAlignment::Left {
            for line in &lines {
                let offset = match style.alignment {
                    TextAlignment::Center => (max_line_width - line.width) / 2.0,
                    TextAlignment::Right => max_line_width - line.width,
                    _ => 0.0,
                };

                if offset != 0.0 {
                    for glyph in &mut glyphs[line.start_glyph..line.end_glyph] {
                        glyph.position[0] += offset;
                    }
                }
            }
        }

        Some(TextLayout {
            glyphs,
            bounds: [0.0, cursor_y, max_line_width, total_height],
            lines,
        })
    }
}


/// MSDF 着色器代码
pub const MSDF_SHADER: &str = r#"
struct MsdfUniforms {
    screen_size: vec2<f32>,
    px_range: f32,      // Distance field range in pixels
    threshold: f32,     // Edge threshold (typically 0.5)
};

@group(0) @binding(0) var<uniform> uniforms: MsdfUniforms;
@group(1) @binding(0) var msdf_texture: texture_2d<f32>;
@group(1) @binding(1) var msdf_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) params: vec4<f32>, // x: layer (0=shadow, 1=stroke, 2=fill), y: stroke_width, z: softness, w: reserved
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) params: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // 屏幕坐标转 NDC
    let ndc = vec2<f32>(
        (in.position.x / uniforms.screen_size.x) * 2.0 - 1.0,
        1.0 - (in.position.y / uniforms.screen_size.y) * 2.0
    );
    
    out.clip_position = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    out.params = in.params;
    
    return out;
}

/// 计算 MSDF 中值
fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}

/// 计算屏幕空间像素范围
fn screen_px_range(uv: vec2<f32>) -> f32 {
    let unit_range = vec2<f32>(uniforms.px_range) / vec2<f32>(textureDimensions(msdf_texture));
    let screen_tex_size = vec2<f32>(1.0) / fwidth(uv);
    return max(0.5 * dot(unit_range, screen_tex_size), 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let msdf = textureSample(msdf_texture, msdf_sampler, in.uv);
    let sd = median(msdf.r, msdf.g, msdf.b);
    
    let screen_px_dist = screen_px_range(in.uv) * (sd - 0.5);
    
    let layer = in.params.x;
    let stroke_width = in.params.y;
    let softness = in.params.z;
    
    var alpha: f32;
    
    if layer < 0.5 {
        // Shadow layer - 扩展距离场
        let shadow_dist = screen_px_dist + stroke_width * 2.0;
        alpha = smoothstep(-softness, softness, shadow_dist);
    } else if layer < 1.5 {
        // Stroke layer - 外扩
        let outer_dist = screen_px_dist + stroke_width;
        let inner_dist = screen_px_dist;
        let outer_alpha = smoothstep(-softness, softness, outer_dist);
        let inner_alpha = smoothstep(-softness, softness, inner_dist);
        alpha = outer_alpha - inner_alpha;
    } else {
        // Fill layer - 标准 MSDF
        alpha = smoothstep(-softness, softness, screen_px_dist);
    }
    
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

// ============================================================================
// 高级效果着色器
// ============================================================================

/// 带渐变的文本
@fragment
fn fs_gradient(in: VertexOutput) -> @location(0) vec4<f32> {
    let msdf = textureSample(msdf_texture, msdf_sampler, in.uv);
    let sd = median(msdf.r, msdf.g, msdf.b);
    let screen_px_dist = screen_px_range(in.uv) * (sd - 0.5);
    let alpha = smoothstep(-1.0, 1.0, screen_px_dist);
    
    // 垂直渐变示例
    let gradient_color = mix(
        vec3<f32>(1.0, 0.0, 0.0), // 顶部颜色
        vec3<f32>(0.0, 0.0, 1.0), // 底部颜色
        in.uv.y
    );
    
    return vec4<f32>(gradient_color, alpha * in.color.a);
}

/// 发光效果
@fragment
fn fs_glow(in: VertexOutput) -> @location(0) vec4<f32> {
    let msdf = textureSample(msdf_texture, msdf_sampler, in.uv);
    let sd = median(msdf.r, msdf.g, msdf.b);
    let screen_px_dist = screen_px_range(in.uv) * (sd - 0.5);
    
    // 内部文本
    let text_alpha = smoothstep(-1.0, 1.0, screen_px_dist);
    
    // 外部发光
    let glow_size = in.params.y; // 使用 stroke_width 作为发光大小
    let glow_dist = screen_px_dist + glow_size;
    let glow_alpha = smoothstep(-glow_size, glow_size * 0.5, glow_dist) * (1.0 - text_alpha);
    
    let glow_color = vec3<f32>(0.0, 1.0, 1.0); // 青色发光
    
    let final_color = mix(glow_color * glow_alpha, in.color.rgb, text_alpha);
    let final_alpha = max(text_alpha, glow_alpha * 0.5);
    
    return vec4<f32>(final_color, final_alpha * in.color.a);
}
"#;

/// MSDF 渲染实例数据 (用于 GPU 提交)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MsdfVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub params: [f32; 4], // layer, stroke_width, softness, reserved
}

/// 将布局转换为顶点数据
pub fn layout_to_vertices(layout: &TextLayout, style: &TextStyle) -> Vec<MsdfVertex> {
    let mut vertices = Vec::with_capacity(layout.glyphs.len() * 6);

    for glyph in &layout.glyphs {
        let layer = match glyph.layer {
            GlyphLayer::Shadow => 0.0,
            GlyphLayer::Stroke => 1.0,
            GlyphLayer::Fill => 2.0,
        };

        let softness = if style.soft_edges { 1.0 } else { 0.1 };
        let params = [layer, style.stroke_width, softness, 0.0];

        let x0 = glyph.position[0];
        let y0 = glyph.position[1];
        let x1 = x0 + glyph.size[0];
        let y1 = y0 + glyph.size[1];

        let u0 = glyph.uv_min[0];
        let v0 = glyph.uv_min[1];
        let u1 = glyph.uv_max[0];
        let v1 = glyph.uv_max[1];

        // 两个三角形组成四边形
        // Triangle 1
        vertices.push(MsdfVertex {
            position: [x0, y0],
            uv: [u0, v1],
            color: glyph.color,
            params,
        });
        vertices.push(MsdfVertex {
            position: [x1, y0],
            uv: [u1, v1],
            color: glyph.color,
            params,
        });
        vertices.push(MsdfVertex {
            position: [x1, y1],
            uv: [u1, v0],
            color: glyph.color,
            params,
        });
        // Triangle 2
        vertices.push(MsdfVertex {
            position: [x0, y0],
            uv: [u0, v1],
            color: glyph.color,
            params,
        });
        vertices.push(MsdfVertex {
            position: [x1, y1],
            uv: [u1, v0],
            color: glyph.color,
            params,
        });
        vertices.push(MsdfVertex {
            position: [x0, y1],
            uv: [u0, v0],
            color: glyph.color,
            params,
        });
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_layout() {
        let mut layouter = TextLayouter::new();

        // 创建测试字体 (简化)
        let mut glyphs = HashMap::new();
        glyphs.insert(
            'A',
            GlyphData {
                unicode: 65,
                advance: 0.6,
                plane_bounds: Some(GlyphBounds {
                    left: 0.0,
                    bottom: 0.0,
                    right: 0.5,
                    top: 0.7,
                }),
                atlas_bounds: Some(GlyphBounds {
                    left: 0.0,
                    bottom: 0.0,
                    right: 64.0,
                    top: 64.0,
                }),
            },
        );
        glyphs.insert(
            ' ',
            GlyphData {
                unicode: 32,
                advance: 0.3,
                plane_bounds: None,
                atlas_bounds: None,
            },
        );

        let font = MsdfFont {
            name: "Test".to_string(),
            atlas_texture: 0,
            atlas_size: [1024, 1024],
            distance_range: 4.0,
            glyphs,
            kerning: HashMap::new(),
            line_height: 1.0,
            ascender: 0.8,
            descender: -0.2,
        };

        layouter.register_font("test".to_string(), font);

        let style = TextStyle::default();
        let layout = layouter.layout_text("A A", "test", &style, None);

        assert!(layout.is_some());
        let layout = layout.unwrap();
        assert!(!layout.glyphs.is_empty());
    }
}
