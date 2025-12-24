use raw_window_handle;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::Fullscreen,
    window::{Window, WindowAttributes},
};

pub struct WinitWindow {
    window: Option<Arc<Window>>,
}

impl Default for WinitWindow {
    fn default() -> Self {
        Self { window: None }
    }
}

impl WinitWindow {
    pub fn new(event_loop: &ActiveEventLoop, size: (u32, u32)) -> Self {
        let win = event_loop
            .create_window(
                WindowAttributes::default().with_inner_size(PhysicalSize::new(size.0, size.1)),
            )
            .unwrap();
        Self {
            window: Some(Arc::new(win)),
        }
    }

    pub fn try_new(event_loop: &ActiveEventLoop, size: (u32, u32)) -> Option<Self> {
        let win = event_loop
            .create_window(
                WindowAttributes::default().with_inner_size(PhysicalSize::new(size.0, size.1)),
            )
            .ok()?;
        Some(Self {
            window: Some(Arc::new(win)),
        })
    }
    pub fn raw(&self) -> &Window {
        self.window.as_ref().expect("Window not initialized")
    }

    pub fn id(&self) -> winit::window::WindowId {
        self.raw().id()
    }

    pub fn request_redraw(&self) {
        self.raw().request_redraw();
    }

    pub fn outer_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.raw().outer_size()
    }

    /// 从Arc<Window>创建WinitWindow
    ///
    /// 用于在事件循环中从已存在的窗口创建WinitWindow包装器
    pub fn from_arc(window: Arc<Window>) -> Self {
        Self {
            window: Some(window),
        }
    }
}

impl crate::platform::Window for WinitWindow {
    fn size(&self) -> (u32, u32) {
        if self.window.is_none() {
            return (800, 600);
        }
        let size = self.raw().inner_size();
        (size.width, size.height)
    }
    fn scale_factor(&self) -> f64 {
        if self.window.is_none() {
            return 1.0;
        }
        self.raw().scale_factor()
    }
    fn request_redraw(&self) {
        if self.window.is_some() {
            self.raw().request_redraw();
        }
    }
    fn set_title(&self, title: &str) {
        if self.window.is_some() {
            self.raw().set_title(title);
        }
    }
    fn set_fullscreen(&self, fullscreen: bool) {
        if self.window.is_some() {
            if fullscreen {
                self.raw().set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                self.raw().set_fullscreen(None);
            }
        }
    }
    fn set_cursor_visible(&self, visible: bool) {
        if self.window.is_some() {
            self.raw().set_cursor_visible(visible);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        use raw_window_handle::HasWindowHandle;
        self.raw().window_handle().unwrap().as_raw()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        use raw_window_handle::HasDisplayHandle;
        self.raw().display_handle().unwrap().as_raw()
    }
}
