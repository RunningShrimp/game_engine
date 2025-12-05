use super::ConfigResult;
use crate::impl_default;
use serde::{Deserialize, Serialize};

/// 输入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// 鼠标灵敏度
    pub mouse_sensitivity: f32,

    /// 鼠标反转Y轴
    pub mouse_invert_y: bool,

    /// 手柄死区
    pub gamepad_deadzone: f32,

    /// 手柄振动
    pub gamepad_vibration: bool,

    /// 键盘映射
    #[serde(default)]
    pub key_bindings: KeyBindings,
}

impl_default!(InputConfig {
    mouse_sensitivity: 1.0,
    mouse_invert_y: false,
    gamepad_deadzone: 0.1,
    gamepad_vibration: true,
    key_bindings: KeyBindings::default(),
});

impl InputConfig {
    /// 验证配置
    pub fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }
}

/// 键盘映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub forward: String,
    pub backward: String,
    pub left: String,
    pub right: String,
    pub jump: String,
    pub crouch: String,
    pub sprint: String,
    pub interact: String,
}

impl_default!(KeyBindings {
    forward: "W".to_string(),
    backward: "S".to_string(),
    left: "A".to_string(),
    right: "D".to_string(),
    jump: "Space".to_string(),
    crouch: "C".to_string(),
    sprint: "Shift".to_string(),
    interact: "E".to_string(),
});
