pub mod winit;

#[cfg(target_arch = "wasm32")]
pub mod web_fs;

#[cfg(target_arch = "wasm32")]
pub mod web_input;

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
    KeyPressed { key: KeyCode, modifiers: Modifiers },
    KeyReleased { key: KeyCode, modifiers: Modifiers },
    CharInput(char),
    
    // Mouse
    MouseMoved { x: f32, y: f32 },
    MouseButtonPressed { button: MouseButton, x: f32, y: f32 },
    MouseButtonReleased { button: MouseButton, x: f32, y: f32 },
    MouseWheel { delta_x: f32, delta_y: f32 },
    
    // Touch (mobile/tablet)
    TouchStart { id: u64, x: f32, y: f32 },
    TouchMove { id: u64, x: f32, y: f32 },
    TouchEnd { id: u64, x: f32, y: f32 },
    
    // Gamepad
    GamepadConnected(u32),
    GamepadDisconnected(u32),
    GamepadAxis { id: u32, axis: GamepadAxis, value: f32 },
    GamepadButton { id: u32, button: GamepadButton, pressed: bool },
    
    // Window
    WindowResized { width: u32, height: u32 },
    WindowFocused(bool),
    WindowCloseRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Escape, Tab, CapsLock, Shift, Control, Alt, Space, Enter, Backspace, Delete,
    Left, Right, Up, Down, Home, End, PageUp, PageDown, Insert,
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
    Left, Right, Middle, Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX, LeftStickY, RightStickX, RightStickY, LeftTrigger, RightTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South, East, North, West,
    LeftBumper, RightBumper, LeftTrigger, RightTrigger,
    Select, Start, Mode,
    LeftThumb, RightThumb,
    DPadUp, DPadDown, DPadLeft, DPadRight,
}

#[derive(bevy_ecs::system::Resource, Default, Clone)]
pub struct InputBuffer {
    pub events: Vec<InputEvent>,
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

#[derive(Debug)]
pub enum FsError {
    NotFound,
    PermissionDenied,
    IoError(String),
    NetworkError(String),
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::NotFound => write!(f, "File not found"),
            FsError::PermissionDenied => write!(f, "Permission denied"),
            FsError::IoError(e) => write!(f, "IO error: {}", e),
            FsError::NetworkError(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl std::error::Error for FsError {}

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
pub trait Filesystem: Send + Sync {
    fn read_sync(&self, path: &Path) -> Result<Vec<u8>, FsError>;
    fn write_sync(&self, path: &Path, data: &[u8]) -> Result<(), FsError>;
    fn exists(&self, path: &Path) -> bool;
    fn watch(&self, path: &Path, tx: Sender<FsEvent>) -> Result<WatchHandle, FsError>;
}

#[cfg(target_arch = "wasm32")]
pub trait Filesystem: Send + Sync {
    fn read_async(&self, url: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, FsError>> + Send>>;
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
pub struct NativeFilesystem;

#[cfg(not(target_arch = "wasm32"))]
impl Default for NativeFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeFilesystem {
    pub fn new() -> Self { Self }
}

#[cfg(not(target_arch = "wasm32"))]
impl Filesystem for NativeFilesystem {
    fn read_sync(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        std::fs::read(path).map_err(|e| FsError::IoError(e.to_string()))
    }
    
    fn write_sync(&self, path: &Path, data: &[u8]) -> Result<(), FsError> {
        std::fs::write(path, data).map_err(|e| FsError::IoError(e.to_string()))
    }
    
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    
    fn watch(&self, _path: &Path, _tx: Sender<FsEvent>) -> Result<WatchHandle, FsError> {
        // TODO: 使用 notify crate 实现文件监视
        Ok(WatchHandle { inner: Box::new(()) })
    }
}

// Web平台实现在 web_fs.rs 和 web_input.rs 模块中
#[cfg(target_arch = "wasm32")]
pub use web_fs::WebFilesystem;

#[cfg(target_arch = "wasm32")]
pub use web_input::WebInput;
