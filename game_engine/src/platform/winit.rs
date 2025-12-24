use raw_window_handle;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::Fullscreen,
    window::{Window, WindowAttributes},
};

pub struct WinitWindow {
    window: Arc<Window>,
}

impl WinitWindow {
    pub fn new(event_loop: &ActiveEventLoop, size: (u32, u32)) -> Self {
        let win = event_loop
            .create_window(
                WindowAttributes::default().with_inner_size(PhysicalSize::new(size.0, size.1)),
            )
            .unwrap();
        Self { window: Arc::new(win) }
    }

    pub fn try_new(event_loop: &ActiveEventLoop, size: (u32, u32)) -> Option<Self> {
        let win = event_loop
            .create_window(
                WindowAttributes::default().with_inner_size(PhysicalSize::new(size.0, size.1)),
            )
            .ok()?;
        Some(Self {
            window: Arc::new(win),
        })
    }
    pub fn raw(&self) -> &Window {
        &*self.window
    }

    pub fn id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn outer_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.outer_size()
    }

    /// 从Arc<Window>创建WinitWindow
    ///
    /// 用于在事件循环中从已存在的窗口创建WinitWindow包装器
    pub fn from_arc(window: Arc<Window>) -> Self {
        Self { window }
    }
}

impl crate::platform::Window for WinitWindow {
    fn size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }
    fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }
    fn request_redraw(&self) {
        self.window.request_redraw();
    }
    fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }
    fn set_fullscreen(&self, fullscreen: bool) {
        if fullscreen {
            self.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        } else {
            self.window.set_fullscreen(None);
        }
    }
    fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        use raw_window_handle::HasWindowHandle;
        self.window.window_handle().unwrap().as_raw()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        use raw_window_handle::HasDisplayHandle;
        self.window.display_handle().unwrap().as_raw()
    }
}
