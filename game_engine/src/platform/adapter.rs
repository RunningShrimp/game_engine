pub use crate::platform::Filesystem as PlatformFilesystem;
pub use crate::platform::FsError;
pub use crate::platform::FsEvent;
pub use crate::platform::Input as PlatformInput;
pub use crate::platform::InputEvent;
pub use crate::platform::KeyCode;
pub use crate::platform::Modifiers;
pub use crate::platform::MouseButton;
pub use crate::platform::WatchHandle;
pub use crate::platform::Window as PlatformWindow;
pub use crate::platform::console::{ConsoleConfig, ConsolePlatform};
pub use crate::platform::hardware_info::HardwareInfo;
pub use crate::platform::power_aware::{PowerAwareManager, PowerState};
pub use crate::platform::winit::WinitWindow;

#[cfg(target_arch = "wasm32")]
pub use crate::platform::web_fs::WebFilesystem;

#[cfg(target_arch = "wasm32")]
pub use crate::platform::web_input::WebInput;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::platform::NativeFilesystem;

pub struct PlatformAdapter {
    pub filesystem: Box<dyn PlatformFilesystem>,
    pub window: Box<dyn PlatformWindow>,
    pub input: Box<dyn PlatformInput>,
    pub hardware: HardwareInfo,
    pub power_aware: PowerAwareManager,
    pub console: Option<ConsoleConfig>,
}

#[derive(Debug)]
pub enum PlatformAdapterError {
    #[cfg(target_arch = "wasm32")]
    FilesystemError(String),
    #[cfg(target_arch = "wasm32")]
    InputError(String),
}

impl std::fmt::Display for PlatformAdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(target_arch = "wasm32")]
            PlatformAdapterError::FilesystemError(msg) => {
                write!(f, "Failed to create filesystem: {}", msg)
            }
            #[cfg(target_arch = "wasm32")]
            PlatformAdapterError::InputError(msg) => {
                write!(f, "Failed to create input: {}", msg)
            }
            #[cfg(not(target_arch = "wasm32"))]
            _ => {
                write!(f, "Platform adapter error occurred")
            }
        }
    }
}

impl std::error::Error for PlatformAdapterError {}

impl PlatformAdapter {
    pub fn new() -> Result<Self, PlatformAdapterError> {
        let hardware = HardwareInfo::detect();
        let power_aware = PowerAwareManager::new();
        let console = ConsoleConfig::from_hardware(&hardware);

        #[cfg(target_arch = "wasm32")]
        {
            let filesystem = Box::new(
                WebFilesystem::new()
                    .map_err(|e| PlatformAdapterError::FilesystemError(format!("{:?}", e)))?,
            ) as Box<dyn PlatformFilesystem>;
            let window =
                Box::new(crate::platform::web_window::WebWindow::new()) as Box<dyn PlatformWindow>;
            let input = Box::new(
                WebInput::new("canvas")
                    .map_err(|e| PlatformAdapterError::InputError(format!("{:?}", e)))?,
            ) as Box<dyn PlatformInput>;

            Ok(Self {
                filesystem,
                window,
                input,
                hardware,
                power_aware,
                console: Some(console),
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let filesystem = Box::new(NativeFilesystem::new()) as Box<dyn PlatformFilesystem>;
            let window = Box::new(WinitWindow::default()) as Box<dyn PlatformWindow>;
            let input = Box::new(crate::platform::native_input::NativeInput::new())
                as Box<dyn PlatformInput>;

            Ok(Self {
                filesystem,
                window,
                input,
                hardware,
                power_aware,
                console: Some(console),
            })
        }
    }

    /// Fallible initialization that provides default fallbacks on error
    pub fn new_with_fallbacks() -> Self {
        Self::new().unwrap_or_else(|err| {
            eprintln!("Platform adapter initialization error: {}", err);
            // Provide a minimal working adapter
            #[cfg(target_arch = "wasm32")]
            {
                Self {
                    filesystem: Box::new(WebFilesystem::default()),
                    window: Box::new(crate::platform::web_window::WebWindow::new()),
                    input: Box::new(WebInput::new("canvas").unwrap_or_else(|_| {
                        // Create a no-op input handler that logs errors
                        panic!("Failed to create WebInput in fallback")
                    })),
                    hardware: HardwareInfo::detect(),
                    power_aware: PowerAwareManager::new(),
                    console: None,
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                Self {
                    filesystem: Box::new(NativeFilesystem::new()),
                    window: Box::new(WinitWindow::default()),
                    input: Box::new(crate::platform::native_input::NativeInput::new()),
                    hardware: HardwareInfo::detect(),
                    power_aware: PowerAwareManager::new(),
                    console: None,
                }
            }
        })
    }

    pub fn filesystem(&self) -> &dyn PlatformFilesystem {
        &*self.filesystem
    }

    pub fn filesystem_mut(&mut self) -> &mut dyn PlatformFilesystem {
        &mut *self.filesystem
    }

    pub fn window(&self) -> &dyn PlatformWindow {
        &*self.window
    }

    pub fn window_mut(&mut self) -> &mut dyn PlatformWindow {
        &mut *self.window
    }

    pub fn input(&self) -> &dyn PlatformInput {
        &*self.input
    }

    pub fn input_mut(&mut self) -> &mut dyn PlatformInput {
        &mut *self.input
    }

    pub fn hardware(&self) -> &HardwareInfo {
        &self.hardware
    }

    pub fn power_aware(&self) -> &PowerAwareManager {
        &self.power_aware
    }

    pub fn power_aware_mut(&mut self) -> &mut PowerAwareManager {
        &mut self.power_aware
    }

    pub fn console(&self) -> Option<&ConsoleConfig> {
        self.console.as_ref()
    }

    pub fn console_mut(&mut self) -> Option<&mut ConsoleConfig> {
        self.console.as_mut()
    }
}

impl Default for PlatformAdapter {
    fn default() -> Self {
        Self::new_with_fallbacks()
    }
}
