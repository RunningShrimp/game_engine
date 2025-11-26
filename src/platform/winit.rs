use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window as WinitWindowRaw;

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
}
