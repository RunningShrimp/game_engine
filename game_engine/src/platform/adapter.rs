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

impl PlatformAdapter {
    pub fn new() -> Self {
        let hardware = HardwareInfo::detect();
        let power_aware = PowerAwareManager::new();
        let console = ConsoleConfig::from_hardware(&hardware);

        #[cfg(target_arch = "wasm32")]
        {
            let filesystem = Box::new(WebFilesystem::new().expect("Failed to create WebFilesystem"))
                as Box<dyn PlatformFilesystem>;
            let window =
                Box::new(crate::platform::web_window::WebWindow::new()) as Box<dyn PlatformWindow>;
            let input = Box::new(WebInput::new("canvas").expect("Failed to create WebInput"))
                as Box<dyn PlatformInput>;

            Self {
                filesystem,
                window,
                input,
                hardware,
                power_aware,
                console: Some(console),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let filesystem = Box::new(NativeFilesystem::new()) as Box<dyn PlatformFilesystem>;
            let window = Box::new(WinitWindow::default()) as Box<dyn PlatformWindow>;
            let input = Box::new(crate::platform::native_input::NativeInput::new())
                as Box<dyn PlatformInput>;

            Self {
                filesystem,
                window,
                input,
                hardware,
                power_aware,
                console: Some(console),
            }
        }
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
        Self::new()
    }
}
