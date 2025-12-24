use crate::platform::{Input, InputEvent, KeyCode, MouseButton};

#[derive(Default)]
pub struct NativeInput {
    events: Vec<InputEvent>,
    keys_pressed: std::collections::HashSet<KeyCode>,
    mouse_buttons: std::collections::HashSet<MouseButton>,
    mouse_pos: (f32, f32),
}

impl NativeInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_event(&mut self, event: InputEvent) {
        self.events.push(event);
    }

    pub fn set_key_pressed(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.keys_pressed.insert(key);
        } else {
            self.keys_pressed.remove(&key);
        }
    }

    pub fn set_mouse_button_pressed(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            self.mouse_buttons.insert(button);
        } else {
            self.mouse_buttons.remove(&button);
        }
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_pos = (x, y);
    }
}

impl Input for NativeInput {
    fn poll_events(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.events)
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    fn mouse_position(&self) -> (f32, f32) {
        self.mouse_pos
    }

    fn set_cursor_grab(&mut self, _grab: bool) {
        // TODO: Implement cursor grab for native platforms
    }

    fn set_cursor_visible(&mut self, _visible: bool) {
        // TODO: Implement cursor visibility for native platforms
    }

    #[cfg(feature = "xr")]
    fn xr_actions(&self) -> Option<&crate::platform::XrActionSet> {
        None
    }
}
