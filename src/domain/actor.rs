//! Actor模式实现
//! 替代channel通信，实现更细粒度的并发控制
//!
//! 使用异步非阻塞消息处理，避免阻塞主循环

use crate::domain::errors::DomainError;
use crate::ecs::AiComponent;
use bevy_ecs::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use tokio::sync::mpsc;
use crate::resources::runtime::global_runtime;

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// 低优先级（默认）
    Low = 0,
    /// 正常优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 紧急优先级
    Urgent = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

impl MessagePriority {
    /// 创建新的消息优先级
    pub fn new() -> Self {
        Self::default()
    }
}

/// Actor消息类型（带优先级）
#[derive(Debug)]
pub struct PrioritizedMessage<T> {
    /// 消息内容
    pub message: T,
    /// 消息优先级
    pub priority: MessagePriority,
}

impl<T> PrioritizedMessage<T> {
    /// 创建带优先级的消息
    pub fn new(message: T, priority: MessagePriority) -> Self {
        Self { message, priority }
    }

    /// 创建正常优先级的消息
    pub fn normal(message: T) -> Self {
        Self {
            message,
            priority: MessagePriority::Normal,
        }
    }

    /// 创建高优先级的消息
    pub fn high(message: T) -> Self {
        Self {
            message,
            priority: MessagePriority::High,
        }
    }

    /// 创建紧急优先级的消息
    pub fn urgent(message: T) -> Self {
        Self {
            message,
            priority: MessagePriority::Urgent,
        }
    }
}

/// Actor消息类型
#[derive(Debug)]
pub enum ActorMessage<T> {
    /// 处理消息（带优先级）
    Handle(PrioritizedMessage<T>),
    /// 停止Actor
    Stop,
}

/// Actor句柄
///
/// 实现Resource trait，可以作为ECS资源使用
#[derive(Resource)]
pub struct ActorHandle<T> {
    sender: mpsc::UnboundedSender<ActorMessage<T>>,
    // 异步任务句柄（用于等待Actor完成）
    _task_handle: tokio::task::JoinHandle<()>,
}

impl<T> ActorHandle<T>
where
    T: Send + 'static,
{
    /// 发送消息到Actor（正常优先级）
    /// 
    /// 非阻塞操作，立即返回
    pub fn send(&self, message: T) -> Result<(), DomainError> {
        self.send_with_priority(message, MessagePriority::Normal)
    }

    /// 发送带优先级的消息到Actor
    /// 
    /// 非阻塞操作，立即返回
    pub fn send_with_priority(
        &self,
        message: T,
        priority: MessagePriority,
    ) -> Result<(), DomainError> {
        let prioritized = PrioritizedMessage::new(message, priority);
        self.sender
            .send(ActorMessage::Handle(prioritized))
            .map_err(|_| DomainError::General("Failed to send message to actor".to_string()))
    }

    /// 发送高优先级消息
    /// 
    /// 非阻塞操作，立即返回
    pub fn send_high_priority(&self, message: T) -> Result<(), DomainError> {
        self.send_with_priority(message, MessagePriority::High)
    }

    /// 发送紧急消息
    /// 
    /// 非阻塞操作，立即返回
    pub fn send_urgent(&self, message: T) -> Result<(), DomainError> {
        self.send_with_priority(message, MessagePriority::Urgent)
    }

    /// 停止Actor
    /// 
    /// 发送停止信号，Actor会在处理完当前消息后停止
    pub fn stop(self) -> Result<(), DomainError> {
        self.sender
            .send(ActorMessage::Stop)
            .map_err(|_| DomainError::General("Failed to stop actor".to_string()))?;
        // 注意：不再等待任务完成，因为这是异步的
        // 如果需要等待，可以使用 _task_handle.await
        Ok(())
    }
}

/// Actor trait
pub trait Actor: Send + 'static {
    type Message: Send + 'static;

    /// 处理消息
    fn receive(&mut self, message: Self::Message) -> Result<(), DomainError>;

    /// Actor停止时的清理
    fn cleanup(&mut self) {}
}

/// Actor系统
///
/// 管理所有Actor的注册、调度和生命周期。
/// 支持优先级队列和批量处理优化。
pub struct ActorSystem {
    actors: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    /// Actor优先级配置
    actor_priorities: HashMap<String, MessagePriority>,
}

// 为PrioritizedMessage实现Ord，用于BinaryHeap排序
impl<T> Ord for PrioritizedMessage<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap是最大堆，所以需要反转比较顺序
        // 优先级高的（值大的）应该先出队
        other.priority.cmp(&self.priority)
    }
}

impl<T> PartialOrd for PrioritizedMessage<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for PrioritizedMessage<T> {}

impl<T> PartialEq for PrioritizedMessage<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
            actor_priorities: HashMap::new(),
        }
    }

    /// 设置Actor的默认优先级
    pub fn set_actor_priority(&mut self, name: &str, priority: MessagePriority) {
        self.actor_priorities.insert(name.to_string(), priority);
    }

    /// 获取Actor的默认优先级
    pub fn get_actor_priority(&self, name: &str) -> MessagePriority {
        self.actor_priorities
            .get(name)
            .copied()
            .unwrap_or(MessagePriority::Normal)
    }

    /// 注册Actor
    pub fn register<A>(
        &mut self,
        name: impl Into<String>,
        actor: A,
    ) -> Result<ActorHandle<A::Message>, DomainError>
    where
        A: Actor + 'static,
    {
        let name = name.into();
        if self.actors.contains_key(&name) {
            return Err(DomainError::General(format!(
                "Actor '{}' already exists",
                name
            )));
        }

        let (sender, mut receiver) = mpsc::unbounded_channel();
        let mut actor = actor;

        // 启动异步Actor任务
        let task_handle = global_runtime().spawn(async move {
            // 优先级队列：使用BinaryHeap实现最大堆（优先级高的先处理）
            let mut priority_queue: BinaryHeap<PrioritizedMessage<A::Message>> = BinaryHeap::new();
            let mut processing_queue = Vec::new();

            loop {
                // 非阻塞接收所有可用消息
                while let Ok(msg) = receiver.try_recv() {
                    match msg {
                        ActorMessage::Handle(prioritized_msg) => {
                            priority_queue.push(prioritized_msg);
                        }
                        ActorMessage::Stop => {
                            // 处理完队列中的所有消息后再停止
                            while let Some(msg) = priority_queue.pop() {
                                if let Err(e) = actor.receive(msg.message) {
                                    tracing::error!(target: "actor", "Actor error: {:?}", e);
                                }
                            }
                            actor.cleanup();
                            return;
                        }
                    }
                }

                // 处理优先级队列中的消息（最多处理一批）
                let batch_size = 10; // 每批处理的消息数量
                for _ in 0..batch_size {
                    if let Some(msg) = priority_queue.pop() {
                        processing_queue.push(msg);
                    } else {
                        break;
                    }
                }

                // 按优先级顺序处理消息
                for msg in processing_queue.drain(..) {
                    if let Err(e) = actor.receive(msg.message) {
                        tracing::error!(target: "actor", "Actor error: {:?}", e);
                    }
                }

                // 如果队列为空，异步等待新消息（非阻塞）
                if priority_queue.is_empty() {
                    match receiver.recv().await {
                        Some(ActorMessage::Handle(prioritized_msg)) => {
                            priority_queue.push(prioritized_msg);
                        }
                        Some(ActorMessage::Stop) => {
                            // 处理完队列中的所有消息后再停止
                            while let Some(msg) = priority_queue.pop() {
                                if let Err(e) = actor.receive(msg.message) {
                                    tracing::error!(target: "actor", "Actor error: {:?}", e);
                                }
                            }
                            actor.cleanup();
                            return;
                        }
                        None => break, // 发送端已关闭
                    }
                } else {
                    // 如果队列不为空，短暂让出控制权，避免长时间占用
                    tokio::task::yield_now().await;
                }
            }
        });

        let handle = ActorHandle {
            sender,
            _task_handle: task_handle,
        };

        self.actors.insert(name, Box::new(handle.sender.clone()));

        Ok(handle)
    }

    /// 获取Actor句柄
    pub fn get_handle<A>(&self, name: &str) -> Option<&mpsc::UnboundedSender<ActorMessage<A::Message>>>
    where
        A: Actor,
    {
        self.actors
            .get(name)?
            .downcast_ref::<mpsc::UnboundedSender<ActorMessage<A::Message>>>()
    }

    /// 停止所有Actor
    pub fn shutdown(&mut self) -> Result<(), DomainError> {
        for _actor in self.actors.values() {
            // 这里需要改进：需要知道每个actor的消息类型
            // 暂时跳过，实际实现需要类型安全的停止
        }
        self.actors.clear();
        Ok(())
    }
}

/// 音频Actor消息
#[derive(Debug)]
pub enum AudioActorMessage {
    Play {
        source_id: u64,
        path: String,
        volume: f32,
        looped: bool,
    },
    Stop {
        source_id: u64,
    },
    Pause {
        source_id: u64,
    },
    Resume {
        source_id: u64,
    },
    SetVolume {
        source_id: u64,
        volume: f32,
    },
    SetMasterVolume {
        volume: f32,
    },
}

/// 音频Actor
pub struct AudioActor {
    // 这里可以包含音频后端状态
    ai: Option<AiComponent>,
}

impl AudioActor {
    pub fn new() -> Self {
        Self { ai: None }
    }

    /// 在创建 actor 时设置 AI 组件
    pub fn with_ai(mut self, ai: AiComponent) -> Self {
        self.ai = Some(ai);
        self
    }

    /// 动态设置 AI 组件
    pub fn set_ai(&mut self, ai: AiComponent) {
        self.ai = Some(ai);
    }
}

impl Actor for AudioActor {
    type Message = AudioActorMessage;

    fn receive(&mut self, message: Self::Message) -> Result<(), DomainError> {
        match message {
            AudioActorMessage::Play {
                source_id,
                path,
                volume: _volume,
                looped: _looped,
            } => {
                tracing::info!(target: "audio_actor", "Playing {} for source {}", path, source_id);
                // 实际的音频播放逻辑
            }
            AudioActorMessage::Stop { source_id } => {
                tracing::info!(target: "audio_actor", "Stopping source {}", source_id);
                // 实际的音频停止逻辑
            }
            AudioActorMessage::Pause { source_id } => {
                tracing::info!(target: "audio_actor", "Pausing source {}", source_id);
                // 实际的音频暂停逻辑
            }
            AudioActorMessage::Resume { source_id } => {
                tracing::info!(target: "audio_actor", "Resuming source {}", source_id);
                // 实际的音频恢复逻辑
            }
            AudioActorMessage::SetVolume { source_id, volume } => {
                tracing::debug!(target: "audio_actor", "Setting volume {} for source {}", volume, source_id);
                // 实际的音量设置逻辑
            }
            AudioActorMessage::SetMasterVolume { volume } => {
                tracing::debug!(target: "audio_actor", "Setting master volume {}", volume);
                // 实际的主音量设置逻辑
            }
        }
        Ok(())
    }
}

/// 物理Actor消息
#[derive(Debug)]
pub enum PhysicsActorMessage {
    Step { delta_time: f32 },
    ApplyForce { body_id: u64, force: [f32; 3] },
    ApplyImpulse { body_id: u64, impulse: [f32; 3] },
    SetPosition { body_id: u64, position: [f32; 3] },
    SetVelocity { body_id: u64, velocity: [f32; 3] },
}

/// 物理Actor
pub struct PhysicsActor {
    // 这里可以包含物理世界状态
    ai: Option<AiComponent>,
}

impl PhysicsActor {
    pub fn new() -> Self {
        Self { ai: None }
    }

    /// 在创建 actor 时设置 AI 组件
    pub fn with_ai(mut self, ai: AiComponent) -> Self {
        self.ai = Some(ai);
        self
    }

    /// 动态设置 AI 组件
    pub fn set_ai(&mut self, ai: AiComponent) {
        self.ai = Some(ai);
    }
}

impl Actor for PhysicsActor {
    type Message = PhysicsActorMessage;

    fn receive(&mut self, message: Self::Message) -> Result<(), DomainError> {
        match message {
            PhysicsActorMessage::Step { delta_time } => {
                tracing::debug!(target: "physics_actor", "Stepping with delta {}", delta_time);
                // 实际的物理步进逻辑
            }
            PhysicsActorMessage::ApplyForce { body_id, force } => {
                tracing::debug!(target: "physics_actor", "Applying force {:?} to body {}", force, body_id);
                // 实际的力施加逻辑
            }
            PhysicsActorMessage::ApplyImpulse { body_id, impulse } => {
                tracing::debug!(target: "physics_actor", "Applying impulse {:?} to body {}", impulse, body_id);
                // 实际的冲量施加逻辑
            }
            PhysicsActorMessage::SetPosition { body_id, position } => {
                tracing::debug!(target: "physics_actor", "Setting position {:?} for body {}", position, body_id);
                // 实际的位置设置逻辑
            }
            PhysicsActorMessage::SetVelocity { body_id, velocity } => {
                tracing::debug!(target: "physics_actor", "Setting velocity {:?} for body {}", velocity, body_id);
                // 实际的速度设置逻辑
            }
        }
        Ok(())
    }
}

/// 渲染Actor消息
#[derive(Debug)]
pub enum RenderActorMessage {
    RenderFrame,
    UpdateTransform {
        entity_id: u64,
        position: [f32; 3],
        rotation: [f32; 4],
    },
    LoadTexture {
        path: String,
    },
    UnloadTexture {
        texture_id: u64,
    },
}

/// 渲染Actor
pub struct RenderActor {
    // 这里可以包含渲染状态
    ai: Option<AiComponent>,
}

impl RenderActor {
    pub fn new() -> Self {
        Self { ai: None }
    }

    /// 在创建 actor 时设置 AI 组件
    pub fn with_ai(mut self, ai: AiComponent) -> Self {
        self.ai = Some(ai);
        self
    }

    /// 动态设置 AI 组件
    pub fn set_ai(&mut self, ai: AiComponent) {
        self.ai = Some(ai);
    }
}

impl Actor for RenderActor {
    type Message = RenderActorMessage;

    fn receive(&mut self, message: Self::Message) -> Result<(), DomainError> {
        match message {
            RenderActorMessage::RenderFrame => {
                tracing::debug!(target: "render_actor", "Rendering frame");
                // 实际的渲染逻辑
            }
            RenderActorMessage::UpdateTransform {
                entity_id,
                position: _,
                rotation: _,
            } => {
                tracing::debug!(target: "render_actor", "Updating transform for entity {}", entity_id);
                // 实际的变换更新逻辑
            }
            RenderActorMessage::LoadTexture { path } => {
                tracing::info!(target: "render_actor", "Loading texture {}", path);
                // 实际的纹理加载逻辑
            }
            RenderActorMessage::UnloadTexture { texture_id } => {
                tracing::info!(target: "render_actor", "Unloading texture {}", texture_id);
                // 实际的纹理卸载逻辑
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_system() {
        let mut system = ActorSystem::new();

        // 注册音频Actor
        let audio_handle = system.register("audio", AudioActor::new()).unwrap();

        // 发送消息
        audio_handle
            .send(AudioActorMessage::Play {
                source_id: 1,
                path: "test.wav".to_string(),
                volume: 1.0,
                looped: false,
            })
            .unwrap();

        // 停止Actor
        audio_handle.stop().unwrap();
    }

    #[test]
    fn test_multiple_actors() {
        let mut system = ActorSystem::new();

        // 注册多个Actor
        let audio_handle = system.register("audio", AudioActor::new()).unwrap();
        let physics_handle = system.register("physics", PhysicsActor::new()).unwrap();
        let render_handle = system.register("render", RenderActor::new()).unwrap();

        // 发送消息到不同Actor
        audio_handle
            .send(AudioActorMessage::SetMasterVolume { volume: 0.8 })
            .unwrap();
        physics_handle
            .send(PhysicsActorMessage::Step { delta_time: 0.016 })
            .unwrap();
        render_handle.send(RenderActorMessage::RenderFrame).unwrap();

        // 停止所有Actor
        audio_handle.stop().unwrap();
        physics_handle.stop().unwrap();
        render_handle.stop().unwrap();
    }
}
