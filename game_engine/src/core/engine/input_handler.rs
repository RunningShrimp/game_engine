//! 输入处理模块
//!
//! 负责处理各种输入事件，包括：
//! - 窗口事件处理
//! - 键盘输入处理
//! - 鼠标输入处理
//! - 触摸输入处理（如果支持）
//! - 输入映射和动作处理

use crate::services::render::RenderService;
use crate::platform::winit::WinitWindow;
use crate::platform::{InputBuffer, InputEvent, InputActions, KeyCode, Modifiers, MouseButton};
use crate::render::wgpu_utils::WgpuRenderer;
use crate::config::input::InputConfig;
use bevy_ecs::prelude::*;
use winit::event::{WindowEvent, PointerSource, ButtonSource};
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
    elwt: &dyn ActiveEventLoop,
) {
    match event {
        WindowEvent::CloseRequested => {
            handle_close_requested(world, elwt);
        }
        WindowEvent::RedrawRequested => {
            super::game_loop::handle_redraw_request(
                world,
                renderer,
                editor_ctx,
                render_service,
                render_cache,
                window,
            );
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
fn handle_close_requested(world: &mut World, elwt: &dyn ActiveEventLoop) {
    if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
        buf.events.push(InputEvent::WindowCloseRequested);
    }
    elwt.exit();
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
    
    if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
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
                let (x, y) = get_current_mouse_position(world);
                
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
            WindowEvent::KeyboardInput { event, .. } => {
                handle_keyboard_input(event, &mut buf, world, &input_config);
            }
            WindowEvent::CursorEntered { .. } => {
                buf.events.push(InputEvent::MouseEntered);
                tracing::debug!(target: "input", "Mouse entered window");
            }
            WindowEvent::CursorLeft { .. } => {
                buf.events.push(InputEvent::MouseLeft);
                tracing::debug!(target: "input", "Mouse left window");
            }
            WindowEvent::ReceivedCharacter(ch) => {
                buf.events.push(InputEvent::CharInput(*ch));
                tracing::debug!(target: "input", "Character input: {}", ch);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                let mods = map_modifiers(modifiers);
                // 更新当前修饰符状态（这里可能需要额外的状态管理）
                tracing::debug!(target: "input", "Modifiers changed: {:?}", mods);
            }
            _ => {}
        }
        
        // 根据输入事件更新输入动作状态
        update_input_actions(world, &buf.events, &input_config);
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
    world: &mut World,
    input_config: &Option<InputConfig>,
) {
    let pressed = matches!(event.state, winit::event::ElementState::Pressed);
    let kc = map_key_code(&event.logical_key);
    let m = Modifiers::default(); // 这里应该从当前状态获取修饰符
    
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
    
    // 根据输入配置处理特殊按键映射
    if let Some(config) = input_config {
        handle_key_mapping(kc, pressed, world, config);
    }
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
                NamedKey::Shift => KeyCode::Shift, // 默认Shift
                NamedKey::Control => KeyCode::Control, // 默认Control
                NamedKey::Alt => KeyCode::Alt, // 默认Alt
                NamedKey::Super => KeyCode::Meta, // Win键在自定义KeyCode中未定义
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
fn handle_key_mapping(key: KeyCode, pressed: bool, world: &mut World, config: &InputConfig) {
    // 如果不存在InputActions资源，插入默认的
    if !world.contains_resource::<InputActions>() {
        world.insert_resource(InputActions::default());
    }
    
    // 获取输入动作资源
    let mut actions = world.get_resource_mut::<InputActions>()
        .expect("Failed to get InputActions resource");
    
    // 根据配置映射按键到动作
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
fn update_input_actions(world: &mut World, events: &[InputEvent], input_config: &Option<InputConfig>) {
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
    let mut actions = world.get_resource_mut::<InputActions>()
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
fn update_action_state(actions: &mut InputActions, key: &KeyCode, pressed: bool, config: &InputConfig) {
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
/// 使用winit 0.31.0-beta.2的新指针事件系统处理触摸输入。
///
/// # 参数
///
/// * `event` - 窗口事件
/// * `world` - ECS世界
#[allow(dead_code)]
pub fn handle_touch_input(event: &winit::event::WindowEvent, world: &mut World) {
    if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
        // 检查是否是指针移动事件中的触摸事件
        if let WindowEvent::PointerMoved { position, source, .. } = event {
            if matches!(source, PointerSource::Touch { .. }) {
                // 将触摸事件转换为鼠标事件或专门的触摸事件
                buf.events.push(InputEvent::MouseMoved {
                    x: position.x as f32,
                    y: position.y as f32,
                });
            }
        }
    }
}

/// 处理指针按钮事件（包括触摸）
///
/// 处理指针按钮事件，包括触摸开始和结束事件。
///
/// # 参数
///
/// * `event` - 指针按钮事件
/// * `world` - ECS世界
#[allow(dead_code)]
pub fn handle_pointer_button(event: &winit::event::WindowEvent, world: &mut World) {
    if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
        // 检查是否是指针按钮事件中的触摸事件
        if let WindowEvent::PointerButton { position, button, state, .. } = event {
            if matches!(button, ButtonSource::Touch { .. }) {
                match state {
                    winit::event::ElementState::Pressed => {
                        buf.events.push(InputEvent::MouseButtonPressed {
                            button: MouseButton::Left,
                            x: position.x as f32,
                            y: position.y as f32,
                        });
                    }
                    winit::event::ElementState::Released => {
                        buf.events.push(InputEvent::MouseButtonReleased {
                            button: MouseButton::Left,
                            x: position.x as f32,
                            y: position.y as f32,
                        });
                    }
                }
            }
        }
    }
}