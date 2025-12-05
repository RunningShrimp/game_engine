//! 编辑器配置管理
//!
//! 提供编辑器设置的持久化存储和加载

use crate::impl_default;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 编辑器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// UI 缩放
    pub ui_scale: f32,
    /// 主题
    pub theme: EditorTheme,
    /// 窗口布局
    pub window_layout: WindowLayout,
    /// 资源浏览器设置
    pub asset_browser: AssetBrowserConfig,
    /// 场景编辑器设置
    pub scene_editor: SceneEditorConfig,
    /// 性能面板设置
    pub performance_panel: PerformancePanelConfig,
}

/// 编辑器主题
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorTheme {
    Dark,
    Light,
    Auto, // 跟随系统
}

/// 窗口布局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowLayout {
    /// 是否显示层级视图
    pub show_hierarchy: bool,
    /// 是否显示检查器
    pub show_inspector: bool,
    /// 是否显示资源浏览器
    pub show_asset_browser: bool,
    /// 是否显示控制台
    pub show_console: bool,
    /// 是否显示性能面板
    pub show_performance_panel: bool,
    /// 层级视图宽度
    pub hierarchy_width: f32,
    /// 检查器宽度
    pub inspector_width: f32,
    /// 资源浏览器宽度
    pub asset_browser_width: f32,
    /// 控制台高度
    pub console_height: f32,
    /// 性能面板高度
    pub performance_panel_height: f32,
}

/// 资源浏览器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBrowserConfig {
    /// 视图模式
    pub view_mode: AssetViewMode,
    /// 缩略图大小
    pub thumbnail_size: f32,
    /// 显示文件扩展名
    pub show_extensions: bool,
    /// 默认过滤类型
    pub default_filter: Option<String>,
}

/// 资源视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetViewMode {
    List,
    Grid,
    Details,
}

/// 场景编辑器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEditorConfig {
    /// 默认视图模式
    pub default_view_mode: String,
    /// 网格大小
    pub grid_size: f32,
    /// 默认显示网格
    pub show_grid: bool,
    /// 默认显示辅助线
    pub show_gizmos: bool,
    /// 网格颜色
    pub grid_color: [f32; 4],
    /// 背景颜色
    pub background_color: [f32; 4],
}

/// 性能面板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePanelConfig {
    /// 更新频率 (Hz)
    pub update_frequency: f32,
    /// 历史数据点数
    pub history_size: usize,
    /// 显示哪些指标
    pub visible_metrics: Vec<String>,
    /// 图表颜色
    pub chart_colors: Vec<[f32; 4]>,
}

impl_default!(EditorConfig {
    ui_scale: 1.0,
    theme: EditorTheme::Dark,
    window_layout: WindowLayout::default(),
    asset_browser: AssetBrowserConfig::default(),
    scene_editor: SceneEditorConfig::default(),
    performance_panel: PerformancePanelConfig::default(),
});

impl_default!(WindowLayout {
    show_hierarchy: true,
    show_inspector: true,
    show_asset_browser: true,
    show_console: false,
    show_performance_panel: false,
    hierarchy_width: 250.0,
    inspector_width: 300.0,
    asset_browser_width: 300.0,
    console_height: 200.0,
    performance_panel_height: 200.0,
});

impl_default!(AssetBrowserConfig {
    view_mode: AssetViewMode::List,
    thumbnail_size: 64.0,
    show_extensions: true,
    default_filter: None,
});

impl_default!(SceneEditorConfig {
    default_view_mode: "Perspective".to_string(),
    grid_size: 1.0,
    show_grid: true,
    show_gizmos: true,
    grid_color: [0.3, 0.3, 0.3, 1.0],
    background_color: [0.1, 0.1, 0.1, 1.0],
});

impl_default!(PerformancePanelConfig {
    update_frequency: 10.0,
    history_size: 300,
    visible_metrics: vec![
        "FPS".to_string(),
        "Frame Time".to_string(),
        "Draw Calls".to_string(),
    ],
    chart_colors: vec![
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
    ],
});

/// 编辑器配置管理器
pub struct EditorConfigManager {
    config: EditorConfig,
    config_path: PathBuf,
}

impl EditorConfigManager {
    /// 创建配置管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取默认配置路径
    fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("game_engine")
            .join("editor_config.json")
    }

    /// 内部加载配置
    fn load_config_internal(path: &PathBuf) -> Result<EditorConfig, Box<dyn std::error::Error>> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: EditorConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(EditorConfig::default())
        }
    }

    /// 获取配置
    pub fn config(&self) -> &EditorConfig {
        &self.config
    }

    /// 获取配置的可变引用
    pub fn config_mut(&mut self) -> &mut EditorConfig {
        &mut self.config
    }

    /// 保存配置
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;

        Ok(())
    }

    /// 加载配置
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = Self::load_config_internal(&self.config_path)?;
        Ok(())
    }

    /// 重置为默认配置
    pub fn reset_to_defaults(&mut self) {
        self.config = EditorConfig::default();
    }
}

impl Default for EditorConfigManager {
    fn default() -> Self {
        let config_path = Self::default_config_path();
        let config = Self::load_config_internal(&config_path).unwrap_or_default();

        Self {
            config,
            config_path,
        }
    }
}
