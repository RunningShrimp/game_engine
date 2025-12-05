#[cfg(target_arch = "wasm32")]
use super::{Input, InputEvent, KeyCode, Modifiers, MouseButton};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent, TouchEvent, WheelEvent, Window};

/// Web平台输入处理器
pub struct WebInput {
    window: Window,
    canvas: HtmlCanvasElement,
    events: Arc<Mutex<Vec<InputEvent>>>,
    keys_pressed: Arc<Mutex<HashSet<KeyCode>>>,
    mouse_buttons: Arc<Mutex<HashSet<MouseButton>>>,
    mouse_pos: Arc<Mutex<(f32, f32)>>,
}

impl WebInput {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("No document"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("Canvas not found"))?
            .dyn_into::<HtmlCanvasElement>()?;

        let events = Arc::new(Mutex::new(Vec::new()));
        let keys_pressed = Arc::new(Mutex::new(HashSet::new()));
        let mouse_buttons = Arc::new(Mutex::new(HashSet::new()));
        let mouse_pos = Arc::new(Mutex::new((0.0, 0.0)));

        let mut input = Self {
            window,
            canvas,
            events,
            keys_pressed,
            mouse_buttons,
            mouse_pos,
        };

        input.setup_event_listeners()?;
        Ok(input)
    }

    fn setup_event_listeners(&mut self) -> Result<(), JsValue> {
        // 键盘事件
        {
            let events = self.events.clone();
            let keys = self.keys_pressed.clone();
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                if let Some(key_code) = map_key_code(&event.code()) {
                    let modifiers = get_modifiers(&event);
                    keys.lock().unwrap().insert(key_code);
                    events.lock().unwrap().push(InputEvent::KeyPressed {
                        key: key_code,
                        modifiers,
                    });
                }
            }) as Box<dyn FnMut(_)>);
            self.window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let events = self.events.clone();
            let keys = self.keys_pressed.clone();
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                if let Some(key_code) = map_key_code(&event.code()) {
                    let modifiers = get_modifiers(&event);
                    keys.lock().unwrap().remove(&key_code);
                    events.lock().unwrap().push(InputEvent::KeyReleased {
                        key: key_code,
                        modifiers,
                    });
                }
            }) as Box<dyn FnMut(_)>);
            self.window
                .add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // 鼠标事件
        {
            let events = self.events.clone();
            let pos = self.mouse_pos.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                let x = event.offset_x() as f32;
                let y = event.offset_y() as f32;
                *pos.lock().unwrap() = (x, y);
                events.lock().unwrap().push(InputEvent::MouseMoved { x, y });
            }) as Box<dyn FnMut(_)>);
            self.canvas
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let events = self.events.clone();
            let buttons = self.mouse_buttons.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                let button = map_mouse_button(event.button());
                let x = event.offset_x() as f32;
                let y = event.offset_y() as f32;
                buttons.lock().unwrap().insert(button);
                events
                    .lock()
                    .unwrap()
                    .push(InputEvent::MouseButtonPressed { button, x, y });
            }) as Box<dyn FnMut(_)>);
            self.canvas
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let events = self.events.clone();
            let buttons = self.mouse_buttons.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                let button = map_mouse_button(event.button());
                let x = event.offset_x() as f32;
                let y = event.offset_y() as f32;
                buttons.lock().unwrap().remove(&button);
                events
                    .lock()
                    .unwrap()
                    .push(InputEvent::MouseButtonReleased { button, x, y });
            }) as Box<dyn FnMut(_)>);
            self.canvas
                .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // 滚轮事件
        {
            let events = self.events.clone();
            let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
                let delta_x = event.delta_x() as f32;
                let delta_y = event.delta_y() as f32;
                events
                    .lock()
                    .unwrap()
                    .push(InputEvent::MouseWheel { delta_x, delta_y });
            }) as Box<dyn FnMut(_)>);
            self.canvas
                .add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }
}

impl Input for WebInput {
    fn poll_events(&mut self) -> Vec<InputEvent> {
        self.events.lock().unwrap().drain(..).collect()
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.lock().unwrap().contains(&key)
    }

    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.lock().unwrap().contains(&button)
    }

    fn mouse_position(&self) -> (f32, f32) {
        *self.mouse_pos.lock().unwrap()
    }

    fn set_cursor_grab(&mut self, _grab: bool) {
        // Web平台通过Pointer Lock API实现
        // 需要用户交互才能触发,这里暂不实现
    }

    fn set_cursor_visible(&mut self, visible: bool) {
        let style = if visible { "default" } else { "none" };
        let _ = self.canvas.style().set_property("cursor", style);
    }
}

// 辅助函数

fn map_key_code(code: &str) -> Option<KeyCode> {
    Some(match code {
        "KeyA" => KeyCode::A,
        "KeyB" => KeyCode::B,
        "KeyC" => KeyCode::C,
        "KeyD" => KeyCode::D,
        "KeyE" => KeyCode::E,
        "KeyF" => KeyCode::F,
        "KeyG" => KeyCode::G,
        "KeyH" => KeyCode::H,
        "KeyI" => KeyCode::I,
        "KeyJ" => KeyCode::J,
        "KeyK" => KeyCode::K,
        "KeyL" => KeyCode::L,
        "KeyM" => KeyCode::M,
        "KeyN" => KeyCode::N,
        "KeyO" => KeyCode::O,
        "KeyP" => KeyCode::P,
        "KeyQ" => KeyCode::Q,
        "KeyR" => KeyCode::R,
        "KeyS" => KeyCode::S,
        "KeyT" => KeyCode::T,
        "KeyU" => KeyCode::U,
        "KeyV" => KeyCode::V,
        "KeyW" => KeyCode::W,
        "KeyX" => KeyCode::X,
        "KeyY" => KeyCode::Y,
        "KeyZ" => KeyCode::Z,
        "Digit0" => KeyCode::Num0,
        "Digit1" => KeyCode::Num1,
        "Digit2" => KeyCode::Num2,
        "Digit3" => KeyCode::Num3,
        "Digit4" => KeyCode::Num4,
        "Digit5" => KeyCode::Num5,
        "Digit6" => KeyCode::Num6,
        "Digit7" => KeyCode::Num7,
        "Digit8" => KeyCode::Num8,
        "Digit9" => KeyCode::Num9,
        "F1" => KeyCode::F1,
        "F2" => KeyCode::F2,
        "F3" => KeyCode::F3,
        "F4" => KeyCode::F4,
        "F5" => KeyCode::F5,
        "F6" => KeyCode::F6,
        "F7" => KeyCode::F7,
        "F8" => KeyCode::F8,
        "F9" => KeyCode::F9,
        "F10" => KeyCode::F10,
        "F11" => KeyCode::F11,
        "F12" => KeyCode::F12,
        "Escape" => KeyCode::Escape,
        "Tab" => KeyCode::Tab,
        "CapsLock" => KeyCode::CapsLock,
        "ShiftLeft" | "ShiftRight" => KeyCode::Shift,
        "ControlLeft" | "ControlRight" => KeyCode::Control,
        "AltLeft" | "AltRight" => KeyCode::Alt,
        "Space" => KeyCode::Space,
        "Enter" => KeyCode::Enter,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "ArrowLeft" => KeyCode::Left,
        "ArrowRight" => KeyCode::Right,
        "ArrowUp" => KeyCode::Up,
        "ArrowDown" => KeyCode::Down,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Insert" => KeyCode::Insert,
        _ => return None,
    })
}

fn map_mouse_button(button: i16) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        n => MouseButton::Other(n as u16),
    }
}

fn get_modifiers(event: &KeyboardEvent) -> Modifiers {
    Modifiers {
        shift: event.shift_key(),
        ctrl: event.ctrl_key(),
        alt: event.alt_key(),
        logo: event.meta_key(),
    }
}
