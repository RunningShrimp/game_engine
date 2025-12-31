# 统一平台输入抽象层

**版本**: v1.0
**日期**: 2025-12-31
**状态**: 已实现
**目标**: 跨平台统一输入处理接口

---

## 概述

统一平台输入抽象层为游戏引擎提供了一套跨平台的输入处理接口，支持：
- **桌面平台**: Windows, macOS, Linux
- **移动平台**: Android, iOS
- **Web平台**: WebGL/WebAssembly
- **XR平台**: VR/AR设备

---

## 架构设计

### 核心组件

```
┌─────────────────────────────────────────┐
│          游戏引擎核心                    │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│       Input Trait (统一接口)            │
│  - poll_events()                         │
│  - is_key_pressed()                      │
│  - is_mouse_button_pressed()             │
│  - mouse_position()                      │
└─────────────────┬───────────────────────┘
                  │
    ┌─────────────┼─────────────┬──────────────┐
    │             │             │              │
┌───▼────┐  ┌───▼────┐  ┌───▼─────┐  ┌───▼────┐
│ Native │  │  Web   │  │ Mobile │  │   XR   │
│ Input  │  │ Input  │  │ Input  │  │ Input  │
└────────┘  └────────┘  └─────────┘  └────────┘
```

---

## Input Trait

### 核心接口

```rust
/// 统一输入trait - 所有平台输入系统都应实现此trait
pub trait Input {
    /// 轮询输入事件
    fn poll_events(&mut self) -> Vec<InputEvent>;

    /// 键盘状态
    fn is_key_pressed(&self, key: KeyCode) -> bool;

    /// 鼠标状态
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;
    fn mouse_position(&self) -> (f32, f32);

    /// 光标控制
    fn set_cursor_grab(&mut self, grab: bool);
    fn set_cursor_visible(&mut self, visible: bool);
}
```

### 实现的平台

| 平台 | 实现类型 | 文件位置 |
|------|---------|---------|
| Desktop | `NativeInput` | `platform/native_input.rs` |
| Web | `WebInput` | `platform/web_input.rs` |
| Mobile | `MobileInput` | `platform/mobile/mod.rs` |
| XR | `XrInput` | `xr/input.rs` |

---

## 输入事件类型

### InputEvent 枚举

```rust
pub enum InputEvent {
    // ========== 键盘事件 ==========
    KeyPressed {
        key: KeyCode,
        modifiers: Modifiers,
    },
    KeyReleased {
        key: KeyCode,
        modifiers: Modifiers,
    },
    CharInput(char),

    // ========== 鼠标事件 ==========
    MouseMoved { x: f32, y: f32 },
    MouseButtonPressed {
        button: MouseButton,
        x: f32,
        y: f32,
    },
    MouseButtonReleased {
        button: MouseButton,
        x: f32,
        y: f32,
    },
    MouseWheel {
        delta_x: f32,
        delta_y: f32,
    },
    MouseEntered,
    MouseLeft,

    // ========== 触摸事件 ==========
    TouchStart {
        id: u64,
        x: f32,
        y: f32,
    },
    TouchMove {
        id: u64,
        x: f32,
        y: f32,
    },
    TouchEnd {
        id: u64,
        x: f32,
        y: f32,
    },
    TouchCancel {
        id: u64,
        x: f32,
        y: f32,
    },

    // ========== 手柄事件 ==========
    GamepadConnected(u32),
    GamepadDisconnected(u32),
    GamepadAxis {
        id: u32,
        axis: GamepadAxis,
        value: f32,
    },
    GamepadButton {
        id: u32,
        button: GamepadButton,
        pressed: bool,
    },

    // ========== 窗口事件 ==========
    WindowResized {
        width: u32,
        height: u32,
    },
    WindowFocused(bool),
    WindowCloseRequested,
    RedrawRequested,
}
```

---

## KeyCode 枚举

### 支持的按键

```rust
pub enum KeyCode {
    // 字母键
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // 数字键
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,

    // 功能键
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // 特殊键
    Escape, Enter, Tab, Space, Backspace, Insert,
    Delete, Home, End, PageUp, PageDown,

    // 方向键
    Left, Up, Right, Down,

    // 控制键
    LeftShift, RightShift, LeftControl, RightControl,
    LeftAlt, RightAlt, LeftSuper, RightSuper,

    // 符号键
    Minus, Equals, BracketLeft, BracketRight,
    Semicolon, Quote, Backslash, Comma, Period,
    Slash, Backquote,

    // 其他
    CapsLock, ScrollLock, NumLock, PrintScreen,
    Pause, ScreenLock, Menu,

    // 未知按键
    Unknown,
}
```

---

## MouseButton 枚举

```rust
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}
```

---

## Gamepad 支持

### GamepadAxis 枚举

```rust
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}
```

### GamepadButton 枚举

```rust
pub enum GamepadButton {
    // Face buttons
    A,
    B,
    X,
    Y,

    // Bumpers
    LeftBumper,
    RightBumper,

    // Triggers
    LeftTrigger,
    RightTrigger,

    // Menu buttons
    Select,
    Start,
    Mode,

    // Sticks
    LeftStick,
    RightStick,

    // D-Pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}
```

---

## 使用示例

### 基础输入处理

```rust
use crate::platform::{Input, InputEvent, KeyCode};

struct GameState {
    input: Box<dyn Input>,
}

impl GameState {
    fn update(&mut self) {
        // 轮询输入事件
        let events = self.input.poll_events();

        for event in events {
            match event {
                InputEvent::KeyPressed { key, .. } => {
                    self.handle_key_press(key);
                }
                InputEvent::MouseButtonPressed { button, x, y } => {
                    self.handle_mouse_press(button, x, y);
                }
                _ => {}
            }
        }

        // 检查持续按键状态
        if self.input.is_key_pressed(KeyCode::W) {
            self.move_forward();
        }
    }

    fn handle_key_press(&mut self, key: KeyCode) {
        match key {
            KeyCode::Escape => self.quit_game(),
            KeyCode::Space => self.jump(),
            _ => {}
        }
    }

    fn handle_mouse_press(&mut self, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => {
                if let Some(entity) = self.get_entity_at(x, y) {
                    self.select_entity(entity);
                }
            }
            MouseButton::Right => {
                self.show_context_menu(x, y);
            }
            _ => {}
        }
    }
}
```

### 触摸输入处理（移动平台）

```rust
#[cfg(any(target_os = "android", target_os = "ios"))]
impl GameState {
    fn handle_touch(&mut self, event: InputEvent) {
        match event {
            InputEvent::TouchStart { id, x, y } => {
                self.touches.insert(id, (x, y));
            }
            InputEvent::TouchMove { id, x, y } => {
                if let Some(touch) = self.touches.get_mut(&id) {
                    *touch = (x, y);
                    self.handle_gesture(id, x, y);
                }
            }
            InputEvent::TouchEnd { id, .. } => {
                self.touches.remove(&id);
            }
            _ => {}
        }
    }

    fn handle_gesture(&mut self, id: u64, x: f32, y: f32) {
        if self.touches.len() == 2 {
            // 双指缩放
            let positions: Vec<_> = self.touches.values().collect();
            if let (Some(&p1), Some(&p2)) = (positions.first(), positions.get(1)) {
                let distance = ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt();
                self.zoom_level = distance / self.initial_distance;
            }
        }
    }
}
```

### 手柄输入处理

```rust
impl GameState {
    fn handle_gamepad(&self, event: InputEvent) {
        match event {
            InputEvent::GamepadConnected(id) => {
                println!("Gamepad {} connected", id);
            }
            InputEvent::GamepadAxis { id, axis, value } => {
                match axis {
                    GamepadAxis::LeftStickX => {
                        self.player_movement.x = value;
                    }
                    GamepadAxis::LeftStickY => {
                        self.player_movement.y = value;
                    }
                    _ => {}
                }
            }
            InputEvent::GamepadButton { id, button, pressed } => {
                if pressed {
                    match button {
                        GamepadButton::A => self.jump(),
                        GamepadButton::B => self.attack(),
                        GamepadButton::X => self.use_item(),
                        GamepadButton::Y => self.interact(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
```

### 光标控制

```rust
impl GameState {
    fn enable_fps_mode(&mut self) {
        // 锁定光标到窗口中心
        self.input.set_cursor_grab(true);
        self.input.set_cursor_visible(false);
    }

    fn disable_fps_mode(&mut self) {
        // 释放光标
        self.input.set_cursor_grab(false);
        self.input.set_cursor_visible(true);
    }
}
```

---

## 平台特定实现

### NativeInput (桌面平台)

**文件**: `platform/native_input.rs`

```rust
pub struct NativeInput {
    events: Vec<InputEvent>,
    keys_pressed: HashSet<KeyCode>,
    mouse_buttons: HashSet<MouseButton>,
    mouse_pos: (f32, f32),
    cursor_grabbed: bool,
    cursor_visible: bool,
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

    fn set_cursor_grab(&mut self, grab: bool) {
        self.cursor_grabbed = grab;
    }

    fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }
}
```

### WebInput (Web平台)

**文件**: `platform/web_input.rs`

```rust
pub struct WebInput {
    // 使用JavaScript互操作处理Web事件
}

impl Input for WebInput {
    fn poll_events(&mut self) -> Vec<InputEvent> {
        // 从JavaScript事件队列获取事件
        // 通过wasm-bindge实现
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        // 通过JavaScript检查按键状态
    }

    // ... 其他方法实现
}
```

---

## 设计原则

### 1. 平台无关性

游戏代码不应关心底层平台，所有平台差异通过Input trait抽象。

```rust
// ✅ 好的做法 - 平台无关
fn handle_input(input: &dyn Input) {
    if input.is_key_pressed(KeyCode::Space) {
        player.jump();
    }
}

// ❌ 不好的做法 - 平台相关
#[cfg(target_os = "windows")]
fn handle_windows_input() { }

#[cfg(target_os = "linux")]
fn handle_linux_input() { }
```

### 2. 事件驱动

使用轮询模式获取事件列表，适合游戏主循环。

```rust
fn game_loop(input: &mut dyn Input) {
    loop {
        let events = input.poll_events();
        for event in events {
            handle_event(event);
        }

        update_game();
        render();

        std::thread::sleep(Duration::from_millis(16));
    }
}
```

### 3. 状态查询

除了事件，还提供实时状态查询。

```rust
// 检查按键持续按下
while input.is_key_pressed(KeyCode::W) {
    player.move_forward();
}
```

### 4. 触摸支持

移动平台的多点触摸支持。

```rust
struct TouchState {
    active_touches: HashMap<u64, (f32, f32)>,
}

impl TouchState {
    fn handle_touch_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::TouchStart { id, x, y } => {
                self.active_touches.insert(id, (x, y));
            }
            InputEvent::TouchEnd { id, .. } => {
                self.active_touches.remove(&id);
            }
            _ => {}
        }
    }

    fn detect_gesture(&self) -> Option<Gesture> {
        match self.active_touches.len() {
            1 => Some(Gesture::Tap),
            2 => Some(Gesture::Pinch),
            _ => None,
        }
    }
}
```

---

## 扩展性

### 添加新输入设备

1. **定义新的InputEvent变体**
2. **在Input trait中添加查询方法**
3. **在各平台实现中处理新设备**

示例：添加VR控制器支持

```rust
// 1. 扩展InputEvent
pub enum InputEvent {
    // ... 现有事件

    // VR控制器事件
    VrControllerConnected {
        controller_id: u32,
        hand: VrHand,
    },
    VrControllerPose {
        controller_id: u32,
        position: Vec3,
        rotation: Quat,
    },
    VrControllerButton {
        controller_id: u32,
        button: VrButton,
        pressed: bool,
    },
}

// 2. 扩展Input trait
pub trait Input {
    // ... 现有方法

    fn get_controller_pose(&self, id: u32) -> Option<(Vec3, Quat)>;
    fn is_controller_button_pressed(&self, id: u32, button: VrButton) -> bool;
}
```

---

## 性能考虑

### 事件批处理

批量处理事件以提高缓存效率：

```rust
const MAX_EVENTS_PER_FRAME: usize = 100;

fn process_events(input: &mut dyn Input) {
    let events = input.poll_events();
    let events = events.into_iter().take(MAX_EVENTS_PER_FRAME);

    for event in events {
        handle_event(event);
    }
}
```

### 避免频繁轮询

只在主循环中轮询事件：

```rust
struct Game {
    input_events: Vec<InputEvent>,
}

impl Game {
    fn update(&mut self, input: &mut dyn Input) {
        // 只在帧开始时轮询一次
        if self.input_events.is_empty() {
            self.input_events = input.poll_events();
        }

        // 使用缓存的事件
        for event in &self.input_events {
            self.handle_event(event);
        }

        // 帧结束时清空
        self.input_events.clear();
    }
}
```

---

## 测试

### 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state() {
        let mut input = NativeInput::new();
        input.set_key_pressed(KeyCode::Space, true);

        assert!(input.is_key_pressed(KeyCode::Space));
    }

    #[test]
    fn test_mouse_position() {
        let mut input = NativeInput::new();
        input.set_mouse_position(100.0, 200.0);

        assert_eq!(input.mouse_position(), (100.0, 200.0));
    }

    #[test]
    fn test_event_polling() {
        let mut input = NativeInput::new();
        input.push_event(InputEvent::KeyPressed {
            key: KeyCode::A,
            modifiers: Modifiers::empty(),
        });

        let events = input.poll_events();
        assert_eq!(events.len(), 1);
    }
}
```

---

## 总结

统一平台输入抽象层提供了：

✅ **跨平台一致性** - 相同的API适用于所有平台
✅ **易于扩展** - 添加新输入设备无需修改游戏代码
✅ **性能优化** - 事件批处理和状态缓存
✅ **类型安全** - 强类型枚举避免运行时错误
✅ **完整覆盖** - 键盘、鼠标、触摸、手柄、XR全支持

---

**文档版本**: v1.0
**最后更新**: 2025-12-31
**维护者**: 游戏引擎团队
