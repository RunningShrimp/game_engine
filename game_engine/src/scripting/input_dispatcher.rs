//! 输入事件分发系统
//!
//! 将平台输入事件分发到脚本生命周期钩子

use crate::ecs::Entity;
use crate::platform::{InputBuffer, InputEvent as PlatformInputEvent, KeyCode, MouseButton};
use crate::scripting::lifecycle::{InputEvent, InputEventType, LifecycleScheduler};
use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// 输入事件分发系统
///
/// 从InputBuffer读取输入事件，并分发到所有有脚本组件的实体
pub fn input_event_dispatcher_system(
    input_buffer: Res<InputBuffer>,
    scheduler: Res<LifecycleScheduler>,
    query: Query<Entity, (With<crate::scripting::lifecycle::LifecycleHooksComponent>,)>,
) {
    let entities_with_scripts: HashSet<Entity> = query.iter().collect();

    for event in input_buffer.events.iter() {
        match event {
            PlatformInputEvent::KeyPressed { key, .. } => {
                for entity in &entities_with_scripts {
                    scheduler.queue_input(InputEvent {
                        entity: *entity,
                        event_type: InputEventType::KeyDown(*key),
                    });
                }
            }
            PlatformInputEvent::KeyReleased { key, .. } => {
                for entity in &entities_with_scripts {
                    scheduler.queue_input(InputEvent {
                        entity: *entity,
                        event_type: InputEventType::KeyUp(*key),
                    });
                }
            }
            PlatformInputEvent::MouseButtonPressed { button, .. } => {
                for entity in &entities_with_scripts {
                    scheduler.queue_input(InputEvent {
                        entity: *entity,
                        event_type: InputEventType::MouseDown(*button),
                    });
                }
            }
            PlatformInputEvent::MouseButtonReleased { button, .. } => {
                for entity in &entities_with_scripts {
                    scheduler.queue_input(InputEvent {
                        entity: *entity,
                        event_type: InputEventType::MouseUp(*button),
                    });
                }
            }
            _ => {
                // 其他输入事件暂不处理
            }
        }
    }
}

/// 应用生命周期事件分发系统
///
/// 处理应用暂停/恢复事件
pub fn app_lifecycle_dispatcher_system(
    scheduler: Res<LifecycleScheduler>,
    query: Query<Entity, (With<crate::scripting::lifecycle::LifecycleHooksComponent>,)>,
    // TODO: 需要添加应用状态资源来检测暂停/恢复
) {
    // 当应用状态资源可用时，这里会分发暂停/恢复事件
    // 目前作为占位实现
    let _ = (scheduler, query);
}
