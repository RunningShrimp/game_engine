use raw_window_handle;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

#[derive(Clone)]
pub struct WinitWindow {
    window: Arc<dyn Window>,
}

impl WinitWindow {
    pub fn new(event_loop: &dyn ActiveEventLoop, size: (u32, u32)) -> Self {
        let win = event_loop.create_window(
            WindowAttributes::default()
                .with_min_surface_size(PhysicalSize::new(size.0, size.1))
        ).unwrap();
        Self {
            window: Arc::new(win),
        }
    }

    pub fn try_new(event_loop: &dyn ActiveEventLoop, size: (u32, u32)) -> Option<Self> {
        let win = event_loop.create_window(
            WindowAttributes::default()
                .with_min_surface_size(PhysicalSize::new(size.0, size.1))
        ).ok()?;
        Some(Self {
            window: Arc::new(win),
        })
    }
    pub fn raw(&self) -> &dyn Window {
        &*self.window
    }
    
    pub fn id(&self) -> winit::window::WindowId {
        self.window.id()
    }
    
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

impl crate::platform::Window for WinitWindow {
    fn size(&self) -> (u32, u32) {
        // TODO: Fix this after winit API is updated
        // For now, return a fixed size
        (800, 600)
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
            // 根据错误提示，Fullscreen枚举是私有的，需要从monitor模块导入
            self.window
                .set_fullscreen(Some(winit::monitor::Fullscreen::Borderless(None)));
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