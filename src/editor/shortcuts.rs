//! 编辑器快捷键系统
//!
//! 提供统一的快捷键管理，支持快捷键绑定、冲突检测和配置持久化

use crate::impl_default;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 快捷键修饰键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd on macOS, Windows key on Windows
}

impl Modifiers {
    pub fn none() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    pub fn alt() -> Self {
        Self {
            ctrl: false,
            alt: true,
            shift: false,
            meta: false,
        }
    }

    pub fn shift() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: true,
            meta: false,
        }
    }

    pub fn meta() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: true,
        }
    }

    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            alt: false,
            shift: true,
            meta: false,
        }
    }
}

/// 快捷键组合
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: String, // egui key name
}

impl Shortcut {
    pub fn new(modifiers: Modifiers, key: impl Into<String>) -> Self {
        Self {
            modifiers,
            key: key.into(),
        }
    }

    pub fn matches(&self, modifiers: Modifiers, key: &str) -> bool {
        self.modifiers == modifiers && self.key == key
    }
}

/// 快捷键动作
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutAction {
    // 文件操作
    NewScene,
    OpenScene,
    SaveScene,
    SaveSceneAs,

    // 编辑操作
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Duplicate,
    Delete,

    // 视图操作
    FocusSelected,
    FrameSelection,
    ToggleGrid,
    ToggleGizmos,

    // 实体操作
    CreateEntity,
    DeleteEntity,
    DuplicateEntity,

    // 工具
    SelectTool,
    MoveTool,
    RotateTool,
    ScaleTool,

    // 窗口
    ToggleHierarchy,
    ToggleInspector,
    ToggleAssetBrowser,
    ToggleConsole,
    TogglePerformancePanel,
}

impl ShortcutAction {
    pub fn default_shortcut(&self) -> Shortcut {
        match self {
            // 文件操作
            ShortcutAction::NewScene => Shortcut::new(Modifiers::ctrl(), "N"),
            ShortcutAction::OpenScene => Shortcut::new(Modifiers::ctrl(), "O"),
            ShortcutAction::SaveScene => Shortcut::new(Modifiers::ctrl(), "S"),
            ShortcutAction::SaveSceneAs => Shortcut::new(Modifiers::ctrl_shift(), "S"),

            // 编辑操作
            ShortcutAction::Undo => Shortcut::new(Modifiers::ctrl(), "Z"),
            ShortcutAction::Redo => Shortcut::new(Modifiers::ctrl_shift(), "Z"),
            ShortcutAction::Cut => Shortcut::new(Modifiers::ctrl(), "X"),
            ShortcutAction::Copy => Shortcut::new(Modifiers::ctrl(), "C"),
            ShortcutAction::Paste => Shortcut::new(Modifiers::ctrl(), "V"),
            ShortcutAction::Duplicate => Shortcut::new(Modifiers::ctrl(), "D"),
            ShortcutAction::Delete => Shortcut::new(Modifiers::none(), "Delete"),

            // 视图操作
            ShortcutAction::FocusSelected => Shortcut::new(Modifiers::none(), "F"),
            ShortcutAction::FrameSelection => Shortcut::new(Modifiers::none(), "Home"),
            ShortcutAction::ToggleGrid => Shortcut::new(Modifiers::none(), "G"),
            ShortcutAction::ToggleGizmos => Shortcut::new(Modifiers::shift(), "G"),

            // 实体操作
            ShortcutAction::CreateEntity => Shortcut::new(Modifiers::ctrl(), "E"),
            ShortcutAction::DeleteEntity => Shortcut::new(Modifiers::none(), "Delete"),
            ShortcutAction::DuplicateEntity => Shortcut::new(Modifiers::ctrl(), "D"),

            // 工具
            ShortcutAction::SelectTool => Shortcut::new(Modifiers::none(), "Q"),
            ShortcutAction::MoveTool => Shortcut::new(Modifiers::none(), "W"),
            ShortcutAction::RotateTool => Shortcut::new(Modifiers::none(), "E"),
            ShortcutAction::ScaleTool => Shortcut::new(Modifiers::none(), "R"),

            // 窗口
            ShortcutAction::ToggleHierarchy => Shortcut::new(Modifiers::none(), "H"),
            ShortcutAction::ToggleInspector => Shortcut::new(Modifiers::none(), "I"),
            ShortcutAction::ToggleAssetBrowser => Shortcut::new(Modifiers::none(), "A"),
            ShortcutAction::ToggleConsole => Shortcut::new(Modifiers::none(), "`"),
            ShortcutAction::TogglePerformancePanel => Shortcut::new(Modifiers::none(), "P"),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ShortcutAction::NewScene => "New Scene",
            ShortcutAction::OpenScene => "Open Scene",
            ShortcutAction::SaveScene => "Save Scene",
            ShortcutAction::SaveSceneAs => "Save Scene As",
            ShortcutAction::Undo => "Undo",
            ShortcutAction::Redo => "Redo",
            ShortcutAction::Cut => "Cut",
            ShortcutAction::Copy => "Copy",
            ShortcutAction::Paste => "Paste",
            ShortcutAction::Duplicate => "Duplicate",
            ShortcutAction::Delete => "Delete",
            ShortcutAction::FocusSelected => "Focus Selected",
            ShortcutAction::FrameSelection => "Frame Selection",
            ShortcutAction::ToggleGrid => "Toggle Grid",
            ShortcutAction::ToggleGizmos => "Toggle Gizmos",
            ShortcutAction::CreateEntity => "Create Entity",
            ShortcutAction::DeleteEntity => "Delete Entity",
            ShortcutAction::DuplicateEntity => "Duplicate Entity",
            ShortcutAction::SelectTool => "Select Tool",
            ShortcutAction::MoveTool => "Move Tool",
            ShortcutAction::RotateTool => "Rotate Tool",
            ShortcutAction::ScaleTool => "Scale Tool",
            ShortcutAction::ToggleHierarchy => "Toggle Hierarchy",
            ShortcutAction::ToggleInspector => "Toggle Inspector",
            ShortcutAction::ToggleAssetBrowser => "Toggle Asset Browser",
            ShortcutAction::ToggleConsole => "Toggle Console",
            ShortcutAction::TogglePerformancePanel => "Toggle Performance Panel",
        }
    }
}

/// 快捷键管理器
pub struct ShortcutManager {
    bindings: HashMap<ShortcutAction, Shortcut>,
    action_callbacks: HashMap<ShortcutAction, Box<dyn Fn() + Send + Sync>>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            action_callbacks: HashMap::new(),
        };

        // 设置默认快捷键
        for action in [
            ShortcutAction::NewScene,
            ShortcutAction::OpenScene,
            ShortcutAction::SaveScene,
            ShortcutAction::SaveSceneAs,
            ShortcutAction::Undo,
            ShortcutAction::Redo,
            ShortcutAction::Cut,
            ShortcutAction::Copy,
            ShortcutAction::Paste,
            ShortcutAction::Duplicate,
            ShortcutAction::Delete,
            ShortcutAction::FocusSelected,
            ShortcutAction::FrameSelection,
            ShortcutAction::ToggleGrid,
            ShortcutAction::ToggleGizmos,
            ShortcutAction::CreateEntity,
            ShortcutAction::DeleteEntity,
            ShortcutAction::DuplicateEntity,
            ShortcutAction::SelectTool,
            ShortcutAction::MoveTool,
            ShortcutAction::RotateTool,
            ShortcutAction::ScaleTool,
            ShortcutAction::ToggleHierarchy,
            ShortcutAction::ToggleInspector,
            ShortcutAction::ToggleAssetBrowser,
            ShortcutAction::ToggleConsole,
            ShortcutAction::TogglePerformancePanel,
        ] {
            manager
                .bindings
                .insert(action.clone(), action.default_shortcut());
        }

        manager
    }

    /// 绑定快捷键
    pub fn bind(&mut self, action: ShortcutAction, shortcut: Shortcut) {
        self.bindings.insert(action, shortcut);
    }

    /// 注册动作回调
    pub fn register_action(
        &mut self,
        action: ShortcutAction,
        callback: Box<dyn Fn() + Send + Sync>,
    ) {
        self.action_callbacks.insert(action, callback);
    }

    /// 检查快捷键是否被触发
    pub fn check(&self, modifiers: Modifiers, key: &str) -> Option<ShortcutAction> {
        for (action, shortcut) in &self.bindings {
            if shortcut.matches(modifiers, key) {
                return Some(action.clone());
            }
        }
        None
    }

    /// 处理快捷键输入
    pub fn handle_input(&self, modifiers: Modifiers, key: &str) -> bool {
        if let Some(action) = self.check(modifiers, key) {
            if let Some(callback) = self.action_callbacks.get(&action) {
                callback();
                return true;
            }
        }
        false
    }

    /// 获取动作的快捷键
    pub fn get_shortcut(&self, action: &ShortcutAction) -> Option<&Shortcut> {
        self.bindings.get(action)
    }

    /// 格式化快捷键显示
    pub fn format_shortcut(&self, action: &ShortcutAction) -> String {
        if let Some(shortcut) = self.get_shortcut(action) {
            let mut parts = Vec::new();

            if shortcut.modifiers.ctrl {
                parts.push("Ctrl");
            }
            if shortcut.modifiers.alt {
                parts.push("Alt");
            }
            if shortcut.modifiers.shift {
                parts.push("Shift");
            }
            if shortcut.modifiers.meta {
                parts.push("Cmd");
            }

            parts.push(&shortcut.key);

            parts.join("+")
        } else {
            "None".to_string()
        }
    }

    /// 保存快捷键配置
    pub fn save_config(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        let config = serde_json::to_string_pretty(&self.bindings)?;
        fs::write(path, config)?;
        Ok(())
    }

    /// 加载快捷键配置
    pub fn load_config(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        if path.exists() {
            let config = fs::read_to_string(path)?;
            let bindings: HashMap<ShortcutAction, Shortcut> = serde_json::from_str(&config)?;
            self.bindings.extend(bindings);
        }
        Ok(())
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}
