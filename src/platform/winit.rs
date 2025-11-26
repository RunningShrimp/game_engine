use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window as WinitWindowRaw;
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};

#[derive(Clone)]
pub struct WinitWindow {
    window: Arc<WinitWindowRaw>,
}

impl WinitWindow {
    pub fn new(event_loop: &EventLoop<()>, size: (u32, u32)) -> Self {
        let win = WinitWindowRaw::new(event_loop).unwrap();
        let _ = win.request_inner_size(PhysicalSize::new(size.0, size.1));
        Self { window: Arc::new(win) }
    }
    pub fn raw(&self) -> &WinitWindowRaw {
        &self.window
    }
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

impl crate::platform::Window for WinitWindow {
    fn size(&self) -> (u32, u32) {
        let s = self.window.inner_size();
        (s.width, s.height)
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
            self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
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
