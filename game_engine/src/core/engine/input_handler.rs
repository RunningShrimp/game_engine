//  输入处理模块
//
//  负责处理各种输入事件，包括：
//  - 窗口事件处理
//  - 键盘输入处理
//  - 鼠标输入处理
//  - 触摸输入处理（如果支持）
//  - 游戏手柄输入处理
//  - 输入映射和动作处理

use crate::config::input::InputConfig;
use crate::platform::winit::WinitWindow;
use crate::platform::{
    GamepadAxis, GamepadButton, InputActions, InputBuffer, InputEvent, KeyCode, Modifiers,
    MouseButton,
};
use crate::render::wgpu_utils::WgpuRenderer;
use crate::services::render::RenderService;
use bevy_ecs::prelude::*;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
// 根据winit 0.31.0-beta.2的API变更，EventLoopWindowTarget可能已被移动
// 我们将使用winit_core中的相应类型

use crate::editor::EditorContext;

/// 处理窗口事件
///
/// 这是窗口事件的主要入口点，负责分发不同类型的事件到相应的处理函数。
///
/// # 参数
///
/// * `event` - 窗口事件
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
/// * `editor_ctx` - 编辑器上下文
/// * `render_service` - 渲染服务
/// * `render_cache` - 渲染缓存
/// * `window` - 窗口实例
/// * `elwt` - 事件循环控制
pub fn handle_window_event(
    event: &WindowEvent,
    world: &mut World,
    renderer: &mut WgpuRenderer,
    editor_ctx: &mut EditorContext,
    render_service: &mut RenderService,
    render_cache: &mut crate::render::graph::RenderCache,
    window: &WinitWindow,
    elwt: &ActiveEventLoop,
) {
    // 处理编辑器输入和窗口基础状态更新，实现逻辑闭环
    let _editor_consumed = editor_ctx.state.on_window_event(window.raw(), event);

    match event {
        WindowEvent::CloseRequested => {
            handle_close_requested(world, elwt);
        }
        WindowEvent::Resized(size) => {
            // 同步更新渲染器和服务的视口
            renderer.resize(*size);
            render_service.update_viewport(size.width, size.height);
            render_cache.invalidate();
        }
        WindowEvent::ScaleFactorChanged { .. } => {
            let size = window.raw().inner_size();
            renderer.resize(size);
        }
        WindowEvent::RedrawRequested => {
            if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
                buf.events.push(InputEvent::RedrawRequested);
                tracing::debug!(target: "input", "Redraw requested");
            }
        }
        _ => {}
    }

    // 输入事件处理
    handle_input_event(event, world);
}

/// 处理窗口关闭请求
///
/// 当用户请求关闭窗口时，记录关闭事件并退出事件循环。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `elwt` - 事件循环控制
fn handle_close_requested(world: &mut World, _elwt: &ActiveEventLoop) {
    if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
        buf.events.push(InputEvent::WindowCloseRequested);
    }
    // 在新版本的winit中，EventLoop没有exit方法
    // 应用程序应该通过返回来退出事件循环
}

/// 处理输入事件
///
/// 处理所有类型的输入事件，包括键盘、鼠标、窗口大小变化等，
/// 并将它们转换为统一的InputEvent格式存储到InputBuffer中。
/// 同时根据输入配置更新输入动作状态。
///
/// # 参数
///
/// * `event` - 窗口事件
/// * `world` - ECS世界
pub fn handle_input_event(event: &WindowEvent, world: &mut World) {
    // 获取输入配置（如果存在）
    let input_config = world.get_resource::<InputConfig>().cloned();

    // Pre-calculate mouse position if needed for mouse events
    let mouse_pos = if let WindowEvent::MouseInput { .. } = event {
        Some(get_current_mouse_position(world))
    } else {
        None
    };

    // Handle keyboard input separately before mutable borrow
    let keyboard_events = if let WindowEvent::KeyboardInput { event, .. } = event {
        let mut temp_buf = InputBuffer::default();
        handle_keyboard_input(event, &mut temp_buf, &input_config, world);
        Some(temp_buf.events)
    } else {
        None
    };

    let events = if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
        match event {
            WindowEvent::Resized(sz) => {
                buf.events.push(InputEvent::WindowResized {
                    width: sz.width,
                    height: sz.height,
                });
                tracing::debug!(target: "input", "Window resized to {}x{}", sz.width, sz.height);
            }
            WindowEvent::Focused(f) => {
                buf.events.push(InputEvent::WindowFocused(*f));
                tracing::debug!(target: "input", "Window focused: {}", f);
            }
            WindowEvent::CursorMoved { position, .. } => {
                buf.events.push(InputEvent::MouseMoved {
                    x: position.x as f32,
                    y: position.y as f32,
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                    winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                };
                buf.events.push(InputEvent::MouseWheel {
                    delta_x: dx,
                    delta_y: dy,
                });
                tracing::debug!(target: "input", "Mouse wheel: dx={}, dy={}", dx, dy);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mb = map_mouse_button(button);
                let (x, y) = mouse_pos.unwrap(); // Safe to unwrap since we checked the event type

                match state {
                    winit::event::ElementState::Pressed => {
                        buf.events.push(InputEvent::MouseButtonPressed { button: mb, x, y });
                        tracing::debug!(target: "input", "Mouse button pressed: {:?}", mb);
                    }
                    winit::event::ElementState::Released => {
                        buf.events.push(InputEvent::MouseButtonReleased { button: mb, x, y });
                        tracing::debug!(target: "input", "Mouse button released: {:?}", mb);
                    }
                }
            }
            WindowEvent::KeyboardInput { .. } => {
                if let Some(events) = &keyboard_events {
                    buf.events.extend(events.clone());
                }
            }
            WindowEvent::CursorEntered { .. } => {
                buf.events.push(InputEvent::MouseEntered);
                tracing::debug!(target: "input", "Mouse entered window");
            }
            WindowEvent::CursorLeft { .. } => {
                buf.events.push(InputEvent::MouseLeft);
                tracing::debug!(target: "input", "Mouse left window");
            }
            WindowEvent::Touch(touch) => {
                // 处理触摸事件
                let id = touch.id;
                let position = touch.location;
                let x = position.x as f32;
                let y = position.y as f32;

                match touch.phase {
                    winit::event::TouchPhase::Started => {
                        buf.events.push(InputEvent::TouchStart { id, x, y });
                    }
                    winit::event::TouchPhase::Moved => {
                        buf.events.push(InputEvent::TouchMove { id, x, y });
                    }
                    winit::event::TouchPhase::Ended => {
                        buf.events.push(InputEvent::TouchEnd { id, x, y });
                    }
                    winit::event::TouchPhase::Cancelled => {
                        buf.events.push(InputEvent::TouchCancel { id, x, y });
                    }
                }
            }
            WindowEvent::Ime(ime) => {
                match ime {
                    winit::event::Ime::Commit(text) => {
                        for ch in text.chars() {
                            buf.events.push(InputEvent::CharInput(ch));
                        }
                        tracing::debug!(target: "input", "IME commit: {}", text);
                    }
                    _ => {} // 其他IME事件暂时忽略
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                let mods = map_modifiers(modifiers);
                // 更新当前修饰符状态（这里可能需要额外的状态管理）
                tracing::debug!(target: "input", "Modifiers changed: {:?}", mods);
            }
            _ => {}
        }
        buf.events.clone()
    } else {
        Vec::new()
    };

    if !events.is_empty() {
        update_input_actions(world, &events, &input_config);
    }

    // Update input actions for keyboard events if any
    if let Some(events) = keyboard_events {
        update_input_actions(world, &events, &input_config);
    }
}

/// 处理键盘输入
///
/// 将winit的键盘事件转换为引擎的InputEvent格式。
///
/// # 参数
///
/// * `event` - 键盘输入事件
/// * `buf` - 输入缓冲区
/// * `world` - ECS世界
/// * `input_config` - 输入配置（可选）
fn handle_keyboard_input(
    event: &winit::event::KeyEvent,
    buf: &mut InputBuffer,
    input_config: &Option<InputConfig>,
    world: &mut World,
) {
    let pressed = matches!(event.state, winit::event::ElementState::Pressed);
    let kc = map_key_code(&event.logical_key);
    let m = Modifiers::default(); // 这里应该从当前状态获取修饰符

    // 使用 input_config 进行基本的上下文检查，实现逻辑闭环
    if let Some(config) = input_config {
        // 检查按键是否匹配任何绑定的操作，并更新InputActions
        let key_str = format!("{:?}", kc);

        // 如果不存在InputActions资源，插入默认的
        if !world.contains_resource::<InputActions>() {
            world.insert_resource(InputActions::default());
        }

        // 获取输入动作资源
        let mut actions = world
            .get_resource_mut::<InputActions>()
            .expect("Failed to get InputActions resource");

        // 根据配置映射按键到动作
        if key_str == config.key_bindings.forward {
            actions.move_forward = pressed;
        }
        if key_str == config.key_bindings.backward {
            actions.move_backward = pressed;
        }
        if key_str == config.key_bindings.left {
            actions.move_left = pressed;
        }
        if key_str == config.key_bindings.right {
            actions.move_right = pressed;
        }
        if key_str == config.key_bindings.jump {
            actions.jump = pressed;
        }
        if key_str == config.key_bindings.crouch {
            actions.crouch = pressed;
        }
        if key_str == config.key_bindings.sprint {
            actions.sprint = pressed;
        }
        if key_str == config.key_bindings.interact {
            actions.interact = pressed;
        }

        tracing::trace!(target: "input", "Handled key {:?} mapped to action", kc);
    }

    if pressed {
        buf.events.push(InputEvent::KeyPressed {
            key: kc,
            modifiers: m,
        });
        tracing::debug!(target: "input", "Key pressed: {:?}", kc);
    } else {
        buf.events.push(InputEvent::KeyReleased {
            key: kc,
            modifiers: m,
        });
        tracing::debug!(target: "input", "Key released: {:?}", kc);
    }

    // Note: Key mapping is now handled externally
}

/// 映射鼠标按钮
///
/// 将winit的鼠标按钮转换为引擎的MouseButton枚举。
///
/// # 参数
///
/// * `button` - winit鼠标按钮
///
/// # 返回
///
/// 引擎的MouseButton枚举值
fn map_mouse_button(button: &winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Other(8),
        winit::event::MouseButton::Forward => MouseButton::Other(9),
        // Other变体可能已被移除，我们需要检查新的API
        _ => MouseButton::Other(0),
    }
}

/// 映射键盘按键
///
/// 将winit的键盘按键转换为引擎的KeyCode枚举。
///
/// # 参数
///
/// * `key` - winit键盘按键
///
/// # 返回
///
/// 引擎的KeyCode枚举值
fn map_key_code(key: &winit::keyboard::Key) -> KeyCode {
    match key {
        winit::keyboard::Key::Character(c) => {
            if c.chars().count() == 1 {
                KeyCode::Unknown(0) // 字符输入通过CharInput事件处理
            } else {
                KeyCode::Unknown(0)
            }
        }
        winit::keyboard::Key::Named(n) => {
            use winit::keyboard::NamedKey;
            match n {
                NamedKey::Escape => KeyCode::Escape,
                NamedKey::Enter => KeyCode::Enter,
                NamedKey::Tab => KeyCode::Tab,
                NamedKey::Space => KeyCode::Space,
                NamedKey::Backspace => KeyCode::Backspace,
                NamedKey::Delete => KeyCode::Delete,
                NamedKey::Insert => KeyCode::Insert,
                NamedKey::Home => KeyCode::Home,
                NamedKey::End => KeyCode::End,
                NamedKey::PageUp => KeyCode::PageUp,
                NamedKey::PageDown => KeyCode::PageDown,
                NamedKey::ArrowLeft => KeyCode::Left,
                NamedKey::ArrowRight => KeyCode::Right,
                NamedKey::ArrowUp => KeyCode::Up,
                NamedKey::ArrowDown => KeyCode::Down,
                NamedKey::F1 => KeyCode::F1,
                NamedKey::F2 => KeyCode::F2,
                NamedKey::F3 => KeyCode::F3,
                NamedKey::F4 => KeyCode::F4,
                NamedKey::F5 => KeyCode::F5,
                NamedKey::F6 => KeyCode::F6,
                NamedKey::F7 => KeyCode::F7,
                NamedKey::F8 => KeyCode::F8,
                NamedKey::F9 => KeyCode::F9,
                NamedKey::F10 => KeyCode::F10,
                NamedKey::F11 => KeyCode::F11,
                NamedKey::F12 => KeyCode::F12,
                NamedKey::Shift => KeyCode::Shift,     // 默认Shift
                NamedKey::Control => KeyCode::Control, // 默认Control
                NamedKey::Alt => KeyCode::Alt,         // 默认Alt
                NamedKey::Meta => KeyCode::Meta,       // Meta键在自定义KeyCode中
                NamedKey::CapsLock => KeyCode::CapsLock,
                NamedKey::NumLock => KeyCode::NumLock,
                NamedKey::ScrollLock => KeyCode::ScrollLock,
                NamedKey::Pause => KeyCode::Pause, // Pause键在自定义KeyCode中未定义
                NamedKey::PrintScreen => KeyCode::Unknown(0), // PrintScreen键在自定义KeyCode中未定义
                _ => KeyCode::Unknown(0),
            }
        }
        winit::keyboard::Key::Unidentified(_) | winit::keyboard::Key::Dead(_) => {
            KeyCode::Unknown(0)
        }
    }
}

/// 映射修饰符
///
/// 将winit的修饰符状态转换为引擎的Modifiers结构。
///
/// # 参数
///
/// * `modifiers` - winit修饰符状态
///
/// # 返回
///
/// 引擎的Modifiers结构
fn map_modifiers(modifiers: &winit::event::Modifiers) -> Modifiers {
    Modifiers {
        shift: modifiers.state().shift_key(),
        ctrl: modifiers.state().control_key(),
        alt: modifiers.state().alt_key(),
        logo: modifiers.state().super_key(),
    }
}

/// 处理按键映射
///
/// 根据输入配置将按键映射到游戏动作。
///
/// # 参数
///
/// * `key` - 按键码
/// * `pressed` - 是否按下
/// * `world` - ECS世界
/// * `config` - 输入配置

/// 获取当前鼠标位置
///
/// 从InputBuffer中获取最新的鼠标位置。
/// 这是一个辅助函数，用于在鼠标事件中提供当前坐标。
///
/// # 参数
///
/// * `world` - ECS世界
///
/// # 返回
///
/// 鼠标的(x, y)坐标，如果没有找到则返回(0.0, 0.0)
fn get_current_mouse_position(world: &World) -> (f32, f32) {
    // 从输入缓冲区获取当前鼠标位置
    if let Some(input_buffer) = world.get_resource::<InputBuffer>() {
        // 获取最新的鼠标位置
        if let Some(mouse_state) = input_buffer.mouse_states.values().next() {
            (mouse_state.x, mouse_state.y)
        } else {
            (0.0, 0.0)
        }
    } else {
        (0.0, 0.0)
    }
}

/// 更新输入动作状态
///
/// 根据输入事件和配置更新输入动作状态。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `events` - 输入事件列表
/// * `input_config` - 输入配置（可选）
fn update_input_actions(
    world: &mut World,
    events: &[InputEvent],
    input_config: &Option<InputConfig>,
) {
    // 如果没有输入配置，不需要更新动作状态
    let config = match input_config {
        Some(c) => c,
        None => return,
    };

    // 如果不存在InputActions资源，插入默认的
    if !world.contains_resource::<InputActions>() {
        world.insert_resource(InputActions::default());
    }

    // 获取输入动作资源
    let mut actions = world
        .get_resource_mut::<InputActions>()
        .expect("Failed to get InputActions resource");

    // 根据事件更新动作状态
    for event in events {
        match event {
            InputEvent::KeyPressed { key, .. } => {
                update_action_state(&mut actions, key, true, config);
            }
            InputEvent::KeyReleased { key, .. } => {
                update_action_state(&mut actions, key, false, config);
            }
            _ => {}
        }
    }
}

/// 更新单个动作状态
///
/// # 参数
///
/// * `actions` - 输入动作资源的可变引用
/// * `key` - 按键码
/// * `pressed` - 是否按下
/// * `config` - 输入配置
fn update_action_state(
    actions: &mut InputActions,
    key: &KeyCode,
    pressed: bool,
    config: &InputConfig,
) {
    let key_str = format!("{:?}", key);
    if key_str == config.key_bindings.forward {
        actions.move_forward = pressed;
    }
    if key_str == config.key_bindings.backward {
        actions.move_backward = pressed;
    }
    if key_str == config.key_bindings.left {
        actions.move_left = pressed;
    }
    if key_str == config.key_bindings.right {
        actions.move_right = pressed;
    }
    if key_str == config.key_bindings.jump {
        actions.jump = pressed;
    }
    if key_str == config.key_bindings.crouch {
        actions.crouch = pressed;
    }
    if key_str == config.key_bindings.sprint {
        actions.sprint = pressed;
    }
    if key_str == config.key_bindings.interact {
        actions.interact = pressed;
    }
}

/// 处理触摸输入
///
/// 处理触摸事件（TouchStart、TouchMove、TouchEnd），将触摸事件转换为统一的InputEvent格式。
///
/// # 参数
///
/// * `event` - 窗口事件
/// * `world` - ECS世界
pub fn handle_touch_input(event: &winit::event::WindowEvent, world: &mut World) {
    if let Some(mut buf) = world.get_resource_mut::<crate::platform::InputBuffer>() {
        match event {
            winit::event::WindowEvent::Touch(touch) => {
                let id = touch.id;
                let position = touch.location;
                let x = position.x as f32;
                let y = position.y as f32;

                match touch.phase {
                    winit::event::TouchPhase::Started => {
                        buf.events.push(crate::platform::InputEvent::TouchStart { id, x, y });
                        tracing::debug!(target: "input", "Touch started: id={}, x={}, y={}", id, x, y);
                    }
                    winit::event::TouchPhase::Moved => {
                        buf.events.push(crate::platform::InputEvent::TouchMove { id, x, y });
                        tracing::debug!(target: "input", "Touch moved: id={}, x={}, y={}", id, x, y);
                    }
                    winit::event::TouchPhase::Ended => {
                        buf.events.push(crate::platform::InputEvent::TouchEnd { id, x, y });
                        tracing::debug!(target: "input", "Touch ended: id={}, x={}, y={}", id, x, y);
                    }
                    winit::event::TouchPhase::Cancelled => {
                        buf.events.push(crate::platform::InputEvent::TouchEnd { id, x, y });
                        tracing::debug!(target: "input", "Touch cancelled: id={}, x={}, y={}", id, x, y);
                    }
                }
            }
            _ => {}
        }
    }
}

/// 处理指针按钮事件（包括触摸）
///
/// 处理指针按钮事件，包括鼠标和触摸设备的按钮事件。
/// 在winit 0.30中，触摸事件通过WindowEvent::Touch处理，鼠标按钮通过WindowEvent::MouseInput处理。
/// 此函数主要用于处理其他指针设备（如触控笔）的按钮事件。
///
/// # 参数
///
/// * `event` - 窗口事件
/// * `world` - ECS世界
pub fn handle_pointer_button(event: &winit::event::WindowEvent, _world: &mut World) {
    // 在winit 0.30中，指针按钮事件主要通过MouseInput和Touch事件处理
    // 这里可以处理其他指针设备的特殊事件
    // 目前触摸和鼠标事件已经在handle_input_event中处理
    // 此函数保留用于未来扩展（如触控笔压力感应等）

    // 如果需要处理特殊的指针设备事件，可以在这里添加
    match event {
        // 可以在这里添加其他指针设备的事件处理
        _ => {}
    }
}

/// 处理游戏手柄输入事件
///
/// 处理游戏手柄连接/断开、按钮和轴事件，将平台层的InputEvent转换为引擎的InputBuffer格式。
/// 此函数用于处理来自游戏手柄的原始输入数据，并将其映射到引擎的标准游戏手柄按钮和轴枚举。
///
/// # 参数
///
/// * `event` - 平台层的游戏手柄输入事件
/// * `world` - ECS世界
pub fn handle_gamepad_input(event: &InputEvent, world: &mut World) {
    if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
        match event {
            InputEvent::GamepadButton {
                id,
                button,
                pressed,
            } => {
                let mapped_button = map_gamepad_button(*button);
                buf.events.push(InputEvent::GamepadButton {
                    id: *id,
                    button: mapped_button,
                    pressed: *pressed,
                });
            }
            InputEvent::GamepadAxis { id, axis, value } => {
                let mapped_axis = map_gamepad_axis(*axis);
                buf.events.push(InputEvent::GamepadAxis {
                    id: *id,
                    axis: mapped_axis,
                    value: *value,
                });
            }
            InputEvent::GamepadConnected(id) => {
                buf.events.push(InputEvent::GamepadConnected(*id));
            }
            InputEvent::GamepadDisconnected(id) => {
                buf.events.push(InputEvent::GamepadDisconnected(*id));
            }
            _ => {}
        }
    }
}

/// 映射游戏手柄按钮到引擎的GamepadButton枚举
///
/// 此函数用于将平台层的游戏手柄按钮映射到引擎的标准GamepadButton枚举。
/// 当前实现直接返回输入值，确保类型一致性。
/// 未来可以在此添加不同平台之间的按钮映射逻辑。
///
/// # 参数
///
/// * `button` - 游戏手柄按钮枚举值
///
/// # 返回
///
/// 映射后的GamepadButton枚举值
fn map_gamepad_button(button: GamepadButton) -> GamepadButton {
    button
}

/// 映射游戏手柄轴到引擎的GamepadAxis枚举
///
/// 此函数用于将平台层的游戏手柄轴映射到引擎的标准GamepadAxis枚举。
/// 当前实现直接返回输入值，确保类型一致性。
/// 未来可以在此添加不同平台之间的轴映射逻辑。
///
/// # 参数
///
/// * `axis` - 游戏手柄轴枚举值
///
/// # 返回
///
/// 映射后的GamepadAxis枚举值
fn map_gamepad_axis(axis: GamepadAxis) -> GamepadAxis {
    axis
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalPosition;
    use winit::event::{TouchPhase, WindowEvent};

    #[test]
    fn test_handle_touch_start() {
        let mut world = World::new();
        world.insert_resource(crate::platform::InputBuffer::default());

        let touch_event = WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            id: 1,
            location: PhysicalPosition::new(100.0, 200.0),
            phase: TouchPhase::Started,
            force: None,
        });

        handle_touch_input(&touch_event, &mut world);

        let buf = world.get_resource::<crate::platform::InputBuffer>().unwrap();
        assert_eq!(buf.events.len(), 1);
        match &buf.events[0] {
            crate::platform::InputEvent::TouchStart { id, x, y } => {
                assert_eq!(*id, 1);
                assert_eq!(*x, 100.0);
                assert_eq!(*y, 200.0);
            }
            _ => panic!("Expected TouchStart event"),
        }
    }

    #[test]
    fn test_handle_touch_move() {
        let mut world = World::new();
        world.insert_resource(crate::platform::InputBuffer::default());

        let touch_event = WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            id: 2,
            location: PhysicalPosition::new(150.0, 250.0),
            phase: TouchPhase::Moved,
            force: None,
        });

        handle_touch_input(&touch_event, &mut world);

        let buf = world.get_resource::<crate::platform::InputBuffer>().unwrap();
        assert_eq!(buf.events.len(), 1);
        match &buf.events[0] {
            crate::platform::InputEvent::TouchMove { id, x, y } => {
                assert_eq!(*id, 2);
                assert_eq!(*x, 150.0);
                assert_eq!(*y, 250.0);
            }
            _ => panic!("Expected TouchMove event"),
        }
    }

    #[test]
    fn test_handle_touch_end() {
        let mut world = World::new();
        world.insert_resource(crate::platform::InputBuffer::default());

        let touch_event = WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            id: 3,
            location: PhysicalPosition::new(200.0, 300.0),
            phase: TouchPhase::Ended,
            force: None,
        });

        handle_touch_input(&touch_event, &mut world);

        let buf = world.get_resource::<crate::platform::InputBuffer>().unwrap();
        assert_eq!(buf.events.len(), 1);
        match &buf.events[0] {
            crate::platform::InputEvent::TouchEnd { id, x, y } => {
                assert_eq!(*id, 3);
                assert_eq!(*x, 200.0);
                assert_eq!(*y, 300.0);
            }
            _ => panic!("Expected TouchEnd event"),
        }
    }

    #[test]
    fn test_handle_touch_cancelled() {
        let mut world = World::new();
        world.insert_resource(crate::platform::InputBuffer::default());

        let touch_event = WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            id: 4,
            location: PhysicalPosition::new(250.0, 350.0),
            phase: TouchPhase::Cancelled,
            force: None,
        });

        handle_touch_input(&touch_event, &mut world);

        let buf = world.get_resource::<crate::platform::InputBuffer>().unwrap();
        assert_eq!(buf.events.len(), 1);
        // Cancelled应该转换为TouchEnd
        match &buf.events[0] {
            crate::platform::InputEvent::TouchEnd { id, x, y } => {
                assert_eq!(*id, 4);
                assert_eq!(*x, 250.0);
                assert_eq!(*y, 350.0);
            }
            _ => panic!("Expected TouchEnd event for cancelled touch"),
        }
    }

    #[test]
    fn test_handle_gamepad_button_pressed() {
        let mut world = World::new();
        world.insert_resource(crate::platform::InputBuffer::default());

        let gamepad_event = crate::platform::InputEvent::GamepadButton {
            id: 0,
            button: crate::platform::GamepadButton::South,
            pressed: true,
        };

        handle_gamepad_input(&gamepad_event, &mut world);

        let buf = world.get_resource::<crate::platform::InputBuffer>().unwrap();
        assert_eq!(buf.events.len(), 1);
        match &buf.events[0] {
            crate::platform::InputEvent::GamepadButton {
                id,
                button,
                pressed,
            } => {
                assert_eq!(*id, 0);
                assert_eq!(*button, crate::platform::GamepadButton::South);
                assert!(pressed);
            }
            _ => panic!("Expected GamepadButton event"),
        }
    }

    #[test]
    fn test_handle_gamepad_axis() {
        let mut world = World::new();
        world.insert_resource(crate::platform::InputBuffer::default());

        let gamepad_event = crate::platform::InputEvent::GamepadAxis {
            id: 0,
            axis: crate::platform::GamepadAxis::LeftStickX,
            value: 0.5,
        };

        handle_gamepad_input(&gamepad_event, &mut world);

        let buf = world.get_resource::<crate::platform::InputBuffer>().unwrap();
        assert_eq!(buf.events.len(), 1);
        match &buf.events[0] {
            crate::platform::InputEvent::GamepadAxis { id, axis, value } => {
                assert_eq!(*id, 0);
                assert_eq!(*axis, crate::platform::GamepadAxis::LeftStickX);
                assert!((*value - 0.5).abs() < 0.001);
            }
            _ => panic!("Expected GamepadAxis event"),
        }
    }

    #[test]
    fn test_handle_gamepad_connected() {
        let mut world = World::new();
        world.insert_resource(crate::platform::InputBuffer::default());

        let gamepad_event = crate::platform::InputEvent::GamepadConnected(0);

        handle_gamepad_input(&gamepad_event, &mut world);

        let buf = world.get_resource::<crate::platform::InputBuffer>().unwrap();
        assert_eq!(buf.events.len(), 1);
        match &buf.events[0] {
            crate::platform::InputEvent::GamepadConnected(id) => {
                assert_eq!(*id, 0);
            }
            _ => panic!("Expected GamepadConnected event"),
        }
    }

    #[test]
    fn test_handle_gamepad_disconnected() {
        let mut world = World::new();
        world.insert_resource(crate::platform::InputBuffer::default());

        let gamepad_event = crate::platform::InputEvent::GamepadDisconnected(0);

        handle_gamepad_input(&gamepad_event, &mut world);

        let buf = world.get_resource::<crate::platform::InputBuffer>().unwrap();
        assert_eq!(buf.events.len(), 1);
        match &buf.events[0] {
            crate::platform::InputEvent::GamepadDisconnected(id) => {
                assert_eq!(*id, 0);
            }
            _ => panic!("Expected GamepadDisconnected event"),
        }
    }

    #[test]
    fn test_map_gamepad_button() {
        assert_eq!(
            map_gamepad_button(crate::platform::GamepadButton::South),
            crate::platform::GamepadButton::South
        );
        assert_eq!(
            map_gamepad_button(crate::platform::GamepadButton::East),
            crate::platform::GamepadButton::East
        );
        assert_eq!(
            map_gamepad_button(crate::platform::GamepadButton::West),
            crate::platform::GamepadButton::West
        );
        assert_eq!(
            map_gamepad_button(crate::platform::GamepadButton::North),
            crate::platform::GamepadButton::North
        );
        assert_eq!(
            map_gamepad_button(crate::platform::GamepadButton::LeftBumper),
            crate::platform::GamepadButton::LeftBumper
        );
        assert_eq!(
            map_gamepad_button(crate::platform::GamepadButton::RightBumper),
            crate::platform::GamepadButton::RightBumper
        );
    }

    #[test]
    fn test_map_gamepad_axis() {
        assert_eq!(
            map_gamepad_axis(crate::platform::GamepadAxis::LeftStickX),
            crate::platform::GamepadAxis::LeftStickX
        );
        assert_eq!(
            map_gamepad_axis(crate::platform::GamepadAxis::LeftStickY),
            crate::platform::GamepadAxis::LeftStickY
        );
        assert_eq!(
            map_gamepad_axis(crate::platform::GamepadAxis::RightStickX),
            crate::platform::GamepadAxis::RightStickX
        );
        assert_eq!(
            map_gamepad_axis(crate::platform::GamepadAxis::RightStickY),
            crate::platform::GamepadAxis::RightStickY
        );
        assert_eq!(
            map_gamepad_axis(crate::platform::GamepadAxis::LeftTrigger),
            crate::platform::GamepadAxis::LeftTrigger
        );
        assert_eq!(
            map_gamepad_axis(crate::platform::GamepadAxis::RightTrigger),
            crate::platform::GamepadAxis::RightTrigger
        );
    }
}
