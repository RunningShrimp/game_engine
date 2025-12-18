//! Actor系统集成
//!
//! 提供ECS系统来与Actor系统交互

use crate::domain::actor::{
    ActorHandle, AudioActorMessage, PhysicsActorMessage, RenderActorMessage,
};
use crate::ecs::Time;
use bevy_ecs::prelude::*;

/// Actor消息队列系统
///
/// 在每帧更新时，系统从ECS资源中获取Actor句柄并发送消息。
/// Actor在独立线程中异步处理消息，实现细粒度的并发控制。
///
/// ## 集成说明
///
/// 此系统替换了原有的channel通信方式，使用Actor模式实现：
/// - **音频Actor**：处理音频播放、暂停、停止等操作
/// - **物理Actor**：处理物理步进和同步
/// - **渲染Actor**：处理渲染命令和状态更新
///
/// ## 优势
///
/// - 细粒度并发控制：每个Actor在独立线程中运行
/// - 消息驱动：通过消息传递实现异步通信
/// - 类型安全：每个Actor有明确的消息类型
/// - 错误隔离：Actor错误不会影响主线程
pub fn actor_message_system(
    time: Res<Time>,
    audio_handle: Option<Res<ActorHandle<AudioActorMessage>>>,
    physics_handle: Option<Res<ActorHandle<PhysicsActorMessage>>>,
    render_handle: Option<Res<ActorHandle<RenderActorMessage>>>,
) {
    // 每帧发送物理步进消息到物理Actor
    if let Some(handle) = physics_handle {
        if let Err(e) = handle.send(PhysicsActorMessage::Step {
            delta_time: time.delta_seconds,
        }) {
            tracing::warn!(target: "actor", "Failed to send physics step message: {:?}", e);
        }
    }

    // 音频Actor消息处理
    // 注意：音频操作通常由事件触发，而不是每帧更新
    // 如果需要每帧更新，可以添加Update消息类型
    // 示例：发送高优先级音频更新消息
    // if let Some(handle) = audio_handle {
    //     handle.send_high_priority(AudioActorMessage::Update { ... }).ok();
    // }
    let _audio_handle = audio_handle;

    // 渲染Actor消息处理
    // 注意：渲染操作通常由事件触发，而不是每帧更新
    // 如果需要每帧更新，可以添加Update消息类型
    // 示例：发送高优先级渲染更新消息
    // if let Some(handle) = render_handle {
    //     handle.send_high_priority(RenderActorMessage::Update { ... }).ok();
    // }
    let _render_handle = render_handle;
}

/// 注册Actor系统到调度器
pub fn register_actor_systems(schedule: &mut Schedule) {
    schedule.add_systems(actor_message_system);
}
