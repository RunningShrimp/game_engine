//! UI主题系统
//!
//! 提供可定制的UI主题和样式。

use glam::Vec4;
use serde::{Deserialize, Serialize};

/// UI主题
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Theme {
    /// 颜色方案
    pub colors: ColorScheme,
    /// 字体配置
    pub fonts: FontScheme,
    /// 样式配置
    pub styles: StyleScheme,
}

/// 颜色方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// 主色调
    pub primary: UIColor,
    /// 次要色调
    pub secondary: UIColor,
    /// 成功色
    pub success: UIColor,
    /// 警告色
    pub warning: UIColor,
    /// 错误色
    pub error: UIColor,
    /// 信息色
    pub info: UIColor,
    /// 背景色
    pub background: UIColor,
    /// 表面色
    pub surface: UIColor,
}

/// UI颜色
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UIColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl UIColor {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    pub fn to_vec4(self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, self.a)
    }

    pub fn multiply(self, factor: f32) -> Self {
        Self {
            r: (self.r * factor).min(1.0),
            g: (self.g * factor).min(1.0),
            b: (self.b * factor).min(1.0),
            a: self.a,
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: UIColor::rgb(0.2, 0.6, 1.0),
            secondary: UIColor::rgb(0.8, 0.8, 0.8),
            success: UIColor::rgb(0.2, 0.8, 0.4),
            warning: UIColor::rgb(1.0, 0.6, 0.2),
            error: UIColor::rgb(1.0, 0.2, 0.2),
            info: UIColor::rgb(0.2, 0.6, 1.0),
            background: UIColor::rgb(0.1, 0.1, 0.1),
            surface: UIColor::rgb(0.15, 0.15, 0.15),
        }
    }
}

/// 字体方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontScheme {
    pub family: String,
    pub sizes: FontSizes,
    pub weights: FontWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizes {
    pub tiny: f32,
    pub small: f32,
    pub normal: f32,
    pub medium: f32,
    pub large: f32,
    pub huge: f32,
}

impl Default for FontSizes {
    fn default() -> Self {
        Self {
            tiny: 10.0,
            small: 12.0,
            normal: 14.0,
            medium: 16.0,
            large: 20.0,
            huge: 24.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontWeights {
    pub light: FontWeight,
    pub normal: FontWeight,
    pub medium: FontWeight,
    pub bold: FontWeight,
}

impl Default for FontWeights {
    fn default() -> Self {
        Self {
            light: FontWeight::Light,
            normal: FontWeight::Normal,
            medium: FontWeight::Medium,
            bold: FontWeight::Bold,
        }
    }
}

impl Default for FontScheme {
    fn default() -> Self {
        Self {
            family: "Arial".to_string(),
            sizes: FontSizes::default(),
            weights: FontWeights::default(),
        }
    }
}

/// 样式方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleScheme {
    pub border_radius: f32,
    pub border_width: f32,
    pub shadow_offset: f32,
    pub shadow_blur: f32,
    pub shadow_color: UIColor,
    pub transition_duration: f32,
}

impl Default for StyleScheme {
    fn default() -> Self {
        Self {
            border_radius: 4.0,
            border_width: 1.0,
            shadow_offset: 2.0,
            shadow_blur: 4.0,
            shadow_color: UIColor::rgba(0.0, 0.0, 0.0, 0.3),
            transition_duration: 0.2,
        }
    }
}

impl Theme {
    pub fn light() -> Self {
        Self {
            colors: ColorScheme {
                background: UIColor::rgb(0.95, 0.95, 0.95),
                surface: UIColor::rgb(1.0, 1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn dark() -> Self {
        Self {
            colors: ColorScheme {
                background: UIColor::rgb(0.1, 0.1, 0.1),
                surface: UIColor::rgb(0.15, 0.15, 0.15),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.colors.primary.a = alpha;
        self.colors.secondary.a = alpha;
        self
    }
}

pub struct UIStyle;

impl UIStyle {
    pub fn button(theme: &Theme, hovered: bool, pressed: bool) -> ButtonStyle {
        let base_color = theme.colors.primary;
        let color = if pressed {
            base_color.multiply(0.8)
        } else if hovered {
            base_color.multiply(1.2)
        } else {
            base_color
        };

        ButtonStyle {
            background_color: color,
            border_color: theme.colors.secondary,
            border_radius: theme.styles.border_radius,
            text_color: UIColor::rgb(1.0, 1.0, 1.0),
        }
    }

    pub fn input(theme: &Theme, focused: bool) -> InputStyle {
        InputStyle {
            background_color: theme.colors.surface,
            border_color: if focused {
                theme.colors.primary
            } else {
                theme.colors.secondary
            },
            border_radius: theme.styles.border_radius,
            text_color: UIColor::rgb(1.0, 1.0, 1.0),
            placeholder_color: UIColor::rgba(1.0, 1.0, 1.0, 0.5),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub background_color: UIColor,
    pub border_color: UIColor,
    pub border_radius: f32,
    pub text_color: UIColor,
}

#[derive(Debug, Clone)]
pub struct InputStyle {
    pub background_color: UIColor,
    pub border_color: UIColor,
    pub border_radius: f32,
    pub text_color: UIColor,
    pub placeholder_color: UIColor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.colors.primary.r, 0.2);
        assert_eq!(theme.fonts.sizes.normal, 14.0);
    }

    #[test]
    fn test_color_multiply() {
        let color = UIColor::rgb(0.5, 0.5, 0.5);
        let darker = color.multiply(0.5);
        assert_eq!(darker.r, 0.25);
    }

    #[test]
    fn test_theme_variants() {
        let light = Theme::light();
        let dark = Theme::dark();
        assert!(light.colors.background.r > 0.5);
        assert!(dark.colors.background.r < 0.5);
    }
}
