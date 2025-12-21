pub mod hardware_info;
pub mod power_aware;
pub mod winit;

use thiserror::Error;

#[cfg(target_arch = "wasm32")]
pub mod web_fs;

#[cfg(target_arch = "wasm32")]
pub mod web_input;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod mobile;

pub mod console;

use std::path::Path;
use std::sync::mpsc::Sender;

// ============================================================================
// Platform Window Abstraction
// ============================================================================

/// 平台窗口抽象 - 支持桌面、移动端、Web
pub trait Window: Send + Sync {
    fn size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
    fn set_title(&self, title: &str);
    fn set_fullscreen(&self, fullscreen: bool);
    fn set_cursor_visible(&self, visible: bool);

    #[cfg(not(target_arch = "wasm32"))]
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle;

    #[cfg(not(target_arch = "wasm32"))]
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle;
}

// ============================================================================
// Input Abstraction
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    // Keyboard
    KeyPressed {
        key: KeyCode,
        modifiers: Modifiers,
    },
    KeyReleased {
        key: KeyCode,
        modifiers: Modifiers,
    },
    CharInput(char),

    // Mouse
    MouseMoved {
        x: f32,
        y: f32,
    },
    MouseButtonPressed {
        button: MouseButton,
        x: f32,
        y: f32,
    },
    MouseButtonReleased {
        button: MouseButton,
        x: f32,
        y: f32,
    },
    MouseWheel {
        delta_x: f32,
        delta_y: f32,
    },
    MouseEntered,
    MouseLeft,

    // Touch (mobile/tablet)
    TouchStart {
        id: u64,
        x: f32,
        y: f32,
    },
    TouchMove {
        id: u64,
        x: f32,
        y: f32,
    },
    TouchEnd {
        id: u64,
        x: f32,
        y: f32,
    },

    // Gamepad
    GamepadConnected(u32),
    GamepadDisconnected(u32),
    GamepadAxis {
        id: u32,
        axis: GamepadAxis,
        value: f32,
    },
    GamepadButton {
        id: u32,
        button: GamepadButton,
        pressed: bool,
    },

    // Window
    WindowResized {
        width: u32,
        height: u32,
    },
    WindowFocused(bool),
    WindowCloseRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Escape,
    Tab,
    CapsLock,
    Shift,
    Control,
    Alt,
    Meta,
    Space,
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    NumLock,
    ScrollLock,
    Pause,
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
}

#[derive(bevy_ecs::prelude::Resource, Default, Clone)]
pub struct InputBuffer {
    pub events: Vec<InputEvent>,
    pub mouse_states: std::collections::HashMap<u32, MouseState>,
}

/// 输入动作状态
#[derive(bevy_ecs::prelude::Resource, Default, Clone)]
pub struct InputActions {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub jump: bool,
    pub crouch: bool,
    pub sprint: bool,
    pub interact: bool,
}

/// 输入系统抽象
pub trait Input: Send + Sync {
    fn poll_events(&mut self) -> Vec<InputEvent>;
    fn is_key_pressed(&self, key: KeyCode) -> bool;
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;
    fn mouse_position(&self) -> (f32, f32);
    fn set_cursor_grab(&mut self, grab: bool);
    fn set_cursor_visible(&mut self, visible: bool);

    /// XR 输入 (可选)
    #[cfg(feature = "xr")]
    fn xr_actions(&self) -> Option<&XrActionSet>;
}

// ============================================================================
// Filesystem Abstraction
// ============================================================================

#[derive(Error, Debug)]
pub enum FsError {
    #[error("File not found")]
    NotFound,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

#[derive(Debug, Clone)]
pub enum FsEvent {
    Modified(std::path::PathBuf),
    Created(std::path::PathBuf),
    Deleted(std::path::PathBuf),
}

pub struct WatchHandle {
    #[allow(dead_code)]
    inner: Box<dyn std::any::Any + Send>,
}

/// 文件系统抽象 - 支持 Native IO 和 Web fetch
#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait Filesystem: Send + Sync {
    /// 异步读取文件
    async fn read(&self, path: &Path) -> Result<Vec<u8>, FsError>;
    /// 异步写入文件
    async fn write(&self, path: &Path, data: &[u8]) -> Result<(), FsError>;
    /// 同步检查文件是否存在（保持同步以兼容现有代码）
    fn exists(&self, path: &Path) -> bool;
    /// 异步检查文件是否存在
    async fn exists_async(&self, path: &Path) -> bool;
    /// 异步创建目录
    async fn create_dir_all(&self, path: &Path) -> Result<(), FsError>;
    /// 异步删除文件
    async fn remove_file(&self, path: &Path) -> Result<(), FsError>;
    /// 异步读取目录
    async fn read_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>, FsError>;
    /// 文件监视（保持同步）
    fn watch(&self, path: &Path, tx: Sender<FsEvent>) -> Result<WatchHandle, FsError>;

    /// 向后兼容的同步方法
    fn read_sync(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        // Runtime-aware sync helper: use block_in_place inside a runtime; otherwise use a small executor
        let path_clone = path.to_path_buf();
        // 对于trait方法，我们需要直接调用tokio::fs而不是通过self
        // 因为self在async move中无法使用
        run_sync(async move {
            tokio::fs::read(&path_clone)
                .await
                .map_err(|e| FsError::IoError(e.to_string()))
        })
    }

    fn write_sync(&self, path: &Path, data: &[u8]) -> Result<(), FsError> {
        // Runtime-aware sync helper: use block_in_place inside a runtime; otherwise use a small executor
        let path_clone = path.to_path_buf();
        let data_clone = data.to_vec();
        run_sync(async move {
            tokio::fs::write(&path_clone, &data_clone)
                .await
                .map_err(|e| FsError::IoError(e.to_string()))
        })
    }
}

// Runtime-aware helper for syncing an async future to a blocking context.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_sync<Fut: std::future::Future + Send + 'static>(fut: Fut) -> Fut::Output 
where
    Fut::Output: Send,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        // Inside runtime: use block_in_place to avoid blocking the runtime
        // Note: This requires the future to be Send, which is already required by the function signature
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fut)
        })
    } else {
        // Outside runtime, use a small executor to drive the future to completion.
        pollster::block_on(fut)
    }
}

#[cfg(target_arch = "wasm32")]
pub trait Filesystem: Send + Sync {
    fn read_async(
        &self,
        url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, FsError>> + Send>>;
    fn cache_get(&self, key: &str) -> Option<Vec<u8>>;
    fn cache_set(&self, key: &str, data: &[u8]);
}

// ============================================================================
// XR Input (Placeholder for OpenXR integration)
// ============================================================================

#[cfg(feature = "xr")]
pub struct XrActionSet {
    pub hand_poses: [XrHandPose; 2],
    pub trigger_values: [f32; 2],
    pub grip_values: [f32; 2],
    pub thumbstick: [[f32; 2]; 2],
    pub button_a: bool,
    pub button_b: bool,
}

#[cfg(feature = "xr")]
#[derive(Default, Clone, Copy)]
pub struct XrHandPose {
    pub position: [f32; 3],
    pub orientation: [f32; 4], // quaternion
    pub is_active: bool,
}

// ============================================================================
// Native Filesystem Implementation
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct NativeFilesystem;

#[cfg(not(target_arch = "wasm32"))]
impl NativeFilesystem {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl Filesystem for NativeFilesystem {
    async fn read(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        tokio::fs::read(path)
            .await
            .map_err(|e| FsError::IoError(e.to_string()))
    }

    async fn write(&self, path: &Path, data: &[u8]) -> Result<(), FsError> {
        tokio::fs::write(path, data)
            .await
            .map_err(|e| FsError::IoError(e.to_string()))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    async fn exists_async(&self, path: &Path) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    async fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|e| FsError::IoError(e.to_string()))
    }

    async fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| FsError::IoError(e.to_string()))
    }

    async fn read_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>, FsError> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path)
            .await
            .map_err(|e| FsError::IoError(e.to_string()))?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| FsError::IoError(e.to_string()))?
        {
            entries.push(entry.path());
        }

        Ok(entries)
    }

    fn watch(&self, _path: &Path, _tx: Sender<FsEvent>) -> Result<WatchHandle, FsError> {
        // NOTE: 文件监视功能待实现，当前返回空句柄
        Ok(WatchHandle {
            inner: Box::new(()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_write_sync_outside_runtime() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_sync.txt");
        let data = b"hello sync";
        std::fs::write(&path, data).unwrap();

        let fs = NativeFilesystem::new();
        let read = fs.read_sync(&path).expect("read_sync failed");
        assert_eq!(read, data);

        let write_path = dir.path().join("test_sync_write.txt");
        fs.write_sync(&write_path, b"written").expect("write_sync failed");
        let got = std::fs::read(&write_path).unwrap();
        assert_eq!(got, b"written");
    }

    #[tokio::test]
    async fn test_read_write_sync_inside_runtime() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_async_sync.txt");
        let data = b"hello async".to_vec();
        tokio::fs::write(&path, &data).await.unwrap();

        let fs = NativeFilesystem::new();
        let read = fs.read_sync(&path).expect("read_sync inside runtime failed");
        assert_eq!(read, data);
    }
}

// Web平台实现在 web_fs.rs 和 web_input.rs 模块中
#[cfg(target_arch = "wasm32")]
pub use web_fs::WebFilesystem;

#[cfg(target_arch = "wasm32")]
pub use web_input::WebInput;

// 移动平台优化
#[cfg(any(target_os = "android", target_os = "ios"))]
pub use mobile::{
    GyroscopeData, MobileAdaptivePerformance, MobileConfig, MobileInputHandler,
    MobilePerformanceMonitor, PerformanceIssue, TouchPoint, get_mobile_config, is_mobile_platform,
};

// 控制台平台支持
pub use console::{
    ButtonState, ConsoleConfig, ConsoleInputHandler, ConsolePerformanceMonitor, ConsolePlatform,
    ControllerState, get_console_config, is_console_platform,
};
