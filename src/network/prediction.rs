//! 客户端预测系统
//!
//! 实现客户端预测、状态回滚和重放机制，减少输入延迟，提升多人游戏体验。
//!
//! ## 设计原则
//!
//! 1. **输入缓冲**: 存储本地输入历史，用于回滚和重放
//! 2. **状态快照**: 定期保存游戏状态快照，用于快速回滚
//! 3. **预测执行**: 在服务器确认前本地执行输入
//! 4. **回滚重放**: 服务器状态不一致时回滚并重放输入
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │              Client-Side Prediction Pipeline            │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. Input Capture                                       │
//! │     - Capture user input                                │
//! │     - Store in command queue                            │
//! │     - Send to server                                    │
//! │                                                          │
//! │  2. Local Prediction                                    │
//! │     - Execute input locally                             │
//! │     - Update game state                                 │
//! │     - Render immediately                                │
//! │                                                          │
//! │  3. Server Confirmation                                 │
//! │     - Receive server state                              │
//! │     - Compare with local state                          │
//! │                                                          │
//! │  4. Rollback & Replay (if needed)                      │
//! │     - Rollback to confirmed state                       │
//! │     - Replay unconfirmed inputs                         │
//! └─────────────────────────────────────────────────────────┘
//! ```

use crate::impl_default;
use crate::network::delay_compensation;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 输入命令
#[derive(Debug, Clone)]
pub struct InputCommand {
    /// 命令序列号（单调递增）
    pub sequence: u64,
    /// 输入时间戳（毫秒）
    pub timestamp_ms: u64,
    /// 输入数据（序列化）
    pub input_data: Vec<u8>,
    /// 是否已确认（服务器已处理）
    pub confirmed: bool,
    /// 确认的服务器tick
    pub confirmed_tick: Option<u64>,
}

impl InputCommand {
    /// 创建新的输入命令
    pub fn new(sequence: u64, input_data: Vec<u8>) -> Self {
        Self {
            sequence,
            timestamp_ms: crate::core::utils::current_timestamp_ms(),
            input_data,
            confirmed: false,
            confirmed_tick: None,
        }
    }

    /// 从补偿输入创建输入命令
    pub fn from_compensated(compensated: delay_compensation::CompensatedInput) -> Self {
        Self {
            sequence: compensated.sequence,
            timestamp_ms: compensated.client_timestamp,
            input_data: compensated.input_data,
            confirmed: false,
            confirmed_tick: None,
        }
    }

    /// 标记为已确认
    pub fn confirm(&mut self, server_tick: u64) {
        self.confirmed = true;
        self.confirmed_tick = Some(server_tick);
    }
}

/// 状态快照
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// 快照对应的tick
    pub tick: u64,
    /// 实体状态（实体 -> 状态数据）
    pub entity_states: Vec<(Entity, EntityState)>,
    /// 创建时间（毫秒）
    pub created_at_ms: u64,
}

/// 实体状态（用于快照）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    /// 位置
    pub position: Vec3,
    /// 旋转
    pub rotation: Quat,
    /// 缩放
    pub scale: Vec3,
    /// 速度
    pub velocity: Vec3,
    /// 其他状态数据（序列化）
    pub custom_data: Vec<u8>,
}

impl Default for EntityState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            velocity: Vec3::ZERO,
            custom_data: Vec::new(),
        }
    }
}

impl EntityState {
    /// 创建默认状态
    pub fn new() -> Self {
        Self::default()
    }
}

/// 预测组件 - 标记需要客户端预测的实体
#[derive(Component, Debug, Clone)]
pub struct PredictionComponent {
    /// 最后确认的服务器tick
    pub last_confirmed_tick: u64,
    /// 当前预测的tick
    pub current_predicted_tick: u64,
    /// 是否正在回滚
    pub is_rolling_back: bool,
    /// 回滚目标tick
    pub rollback_target_tick: Option<u64>,
}

impl_default!(PredictionComponent {
    last_confirmed_tick: 0,
    current_predicted_tick: 0,
    is_rolling_back: false,
    rollback_target_tick: None,
});

/// 客户端预测管理器（增强延迟补偿）
#[derive(Resource)]
pub struct ClientPredictionManager {
    /// 输入命令队列
    input_queue: VecDeque<InputCommand>,
    /// 状态快照历史
    snapshots: VecDeque<StateSnapshot>,
    /// 下一个输入序列号
    next_sequence: u64,
    /// 最后确认的服务器tick
    last_confirmed_server_tick: u64,
    /// 当前客户端tick
    current_client_tick: u64,
    /// 最大快照数量
    max_snapshots: usize,
    /// 快照间隔（tick数）
    snapshot_interval: u64,
    /// 预测统计
    stats: PredictionStats,
    /// 延迟补偿管理器
    delay_compensation: delay_compensation::ClientDelayCompensation,
}

impl ClientPredictionManager {
    /// 创建新的预测管理器
    pub fn new(max_snapshots: usize, snapshot_interval: u64) -> Self {
        Self {
            input_queue: VecDeque::new(),
            snapshots: VecDeque::with_capacity(max_snapshots),
            next_sequence: 1,
            last_confirmed_server_tick: 0,
            current_client_tick: 0,
            max_snapshots,
            snapshot_interval,
            stats: PredictionStats::default(),
            delay_compensation: delay_compensation::ClientDelayCompensation::new(),
        }
    }

}

impl Default for ClientPredictionManager {
    fn default() -> Self {
        Self::new(128, 10) // 保存128个快照，每10tick一个快照
    }
}

impl ClientPredictionManager {
    /// 创建默认配置的预测管理器
    pub fn new_default() -> Self {
        Self::default()
    }

    /// 提交输入命令（带延迟补偿）
    pub fn submit_input_compensated(&mut self, input_data: Vec<u8>) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence += 1;

        // 创建补偿输入
        let compensated = delay_compensation::CompensatedInput::new(sequence, input_data);
        let command = InputCommand::from_compensated(compensated);

        self.input_queue.push_back(command);

        // 限制队列大小
        if self.input_queue.len() > 256 {
            self.input_queue.pop_front();
        }

        sequence
    }

    /// 获取延迟补偿管理器
    pub fn delay_compensation(&self) -> &delay_compensation::ClientDelayCompensation {
        &self.delay_compensation
    }

    /// 获取延迟补偿管理器（可变）
    pub fn delay_compensation_mut(&mut self) -> &mut delay_compensation::ClientDelayCompensation {
        &mut self.delay_compensation
    }

    /// 获取补偿后的输入时间（用于服务器回滚）
    pub fn get_compensated_input_time(&self, command: &InputCommand) -> u64 {
        // 将客户端时间转换为服务器时间
        self.delay_compensation
            .client_to_server_time(command.timestamp_ms)
    }

    /// 提交输入命令
    pub fn submit_input(&mut self, input_data: Vec<u8>) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence += 1;

        let command = InputCommand::new(sequence, input_data);
        self.input_queue.push_back(command);

        // 限制队列大小
        if self.input_queue.len() > 256 {
            self.input_queue.pop_front();
        }

        sequence
    }

    /// 获取未确认的输入命令
    pub fn get_unconfirmed_inputs(&self) -> Vec<&InputCommand> {
        self.input_queue
            .iter()
            .filter(|cmd| !cmd.confirmed)
            .collect()
    }

    /// 确认输入命令（服务器已处理）
    pub fn confirm_input(&mut self, sequence: u64, server_tick: u64) {
        for cmd in self.input_queue.iter_mut() {
            if cmd.sequence == sequence {
                cmd.confirm(server_tick);
                self.last_confirmed_server_tick = server_tick;
                self.stats.confirmed_commands += 1;
                break;
            }
        }

        // 清理已确认的旧命令
        while let Some(front) = self.input_queue.front() {
            if front.confirmed
                && front
                    .confirmed_tick
                    .map_or(false, |t| t < server_tick.saturating_sub(60))
            {
                self.input_queue.pop_front();
            } else {
                break;
            }
        }
    }

    /// 创建状态快照
    pub fn create_snapshot(&mut self, tick: u64, entity_states: Vec<(Entity, EntityState)>) {
        let snapshot = StateSnapshot {
            tick,
            entity_states,
            created_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        self.snapshots.push_back(snapshot);

        // 限制快照数量
        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.pop_front();
        }
    }

    /// 获取指定tick的快照
    pub fn get_snapshot(&self, tick: u64) -> Option<&StateSnapshot> {
        // 查找最接近的快照
        self.snapshots
            .iter()
            .filter(|s| s.tick <= tick)
            .max_by_key(|s| s.tick)
    }

    /// 回滚到指定tick
    pub fn rollback_to(&mut self, target_tick: u64) -> Option<&StateSnapshot> {
        self.stats.rollbacks += 1;
        self.get_snapshot(target_tick)
    }

    /// 重放输入命令（从指定tick到当前tick）
    pub fn replay_inputs(&mut self, from_tick: u64, to_tick: u64) -> Vec<&InputCommand> {
        self.stats.replays += 1;

        self.input_queue
            .iter()
            .filter(|cmd| {
                // 获取命令对应的tick（简化实现）
                // 实际应该根据命令时间戳计算tick
                cmd.sequence >= from_tick && cmd.sequence <= to_tick
            })
            .collect()
    }

    /// 更新当前客户端tick
    pub fn update_tick(&mut self) {
        self.current_client_tick += 1;
    }

    /// 获取当前客户端tick
    pub fn current_tick(&self) -> u64 {
        self.current_client_tick
    }

    /// 获取最后确认的服务器tick
    pub fn last_confirmed_tick(&self) -> u64 {
        self.last_confirmed_server_tick
    }

    /// 获取预测统计
    pub fn stats(&self) -> &PredictionStats {
        &self.stats
    }

    /// 检查是否需要创建快照
    pub fn should_create_snapshot(&self) -> bool {
        self.current_client_tick % self.snapshot_interval == 0
    }
}

/// 预测统计信息
#[derive(Debug, Clone, Default)]
pub struct PredictionStats {
    /// 回滚次数
    pub rollbacks: u64,
    /// 重放次数
    pub replays: u64,
    /// 已确认的命令数
    pub confirmed_commands: u64,
    /// 未确认的命令数
    pub unconfirmed_commands: u64,
    /// 平均回滚距离（tick）
    pub avg_rollback_distance: f32,
}

/// 预测系统 - 处理客户端预测逻辑
pub fn client_prediction_system(
    mut prediction: ResMut<ClientPredictionManager>,
    mut query: Query<(&mut PredictionComponent, Entity, &crate::ecs::Transform)>,
) {
    prediction.update_tick();

    // 更新预测组件
    for (mut pred, _entity, _transform) in query.iter_mut() {
        pred.current_predicted_tick = prediction.current_tick();
    }

    // 检查是否需要创建快照
    let should_create = prediction.should_create_snapshot();
    let current_tick = prediction.current_tick();

    if should_create {
        // 收集实体状态并创建快照
        let mut entity_states = Vec::new();
        for (_pred, entity, transform) in query.iter() {
            let state = EntityState {
                position: transform.pos,
                rotation: transform.rot,
                scale: transform.scale,
                velocity: Vec3::ZERO, // 如果有速度组件，应该从那里获取
                custom_data: Vec::new(),
            };
            entity_states.push((entity, state));
        }

        prediction.create_snapshot(current_tick, entity_states);
    }
}

/// 输入处理系统 - 捕获输入并提交到预测系统
pub fn input_capture_system(
    mut prediction: ResMut<ClientPredictionManager>,
    input_buffer: Res<crate::platform::InputBuffer>,
) {
    // 从输入系统获取输入并提交
    // 只处理与游戏逻辑相关的输入（移动、动作等）
    let mut relevant_inputs = Vec::new();

    for event in input_buffer.events.iter() {
        match event {
            crate::platform::InputEvent::KeyPressed { key, modifiers: _ }
            | crate::platform::InputEvent::KeyReleased { key, modifiers: _ } => {
                // 只处理WASD等移动键
                match key {
                    crate::platform::KeyCode::W
                    | crate::platform::KeyCode::A
                    | crate::platform::KeyCode::S
                    | crate::platform::KeyCode::D
                    | crate::platform::KeyCode::Space => {
                        relevant_inputs.push(*key);
                    }
                    _ => {}
                }
            }
            crate::platform::InputEvent::MouseMoved { x: _, y: _ } => {
                // 鼠标移动用于视角控制
                relevant_inputs.push(crate::platform::KeyCode::Unknown(0)); // 标记为鼠标输入
            }
            crate::platform::InputEvent::MouseButtonPressed { button, x: _, y: _ }
            | crate::platform::InputEvent::MouseButtonReleased { button, x: _, y: _ } => {
                // 鼠标按钮用于动作
                let _ = button; // 标记为鼠标按钮
                relevant_inputs.push(crate::platform::KeyCode::Unknown(1)); // 标记为鼠标按钮
            }
            _ => {}
        }
    }

    // 序列化输入数据
    if !relevant_inputs.is_empty() {
        // 简单的序列化：将KeyCode转换为字节
        let input_data: Vec<u8> = relevant_inputs
            .iter()
            .flat_map(|k| {
                let code = match k {
                    crate::platform::KeyCode::W => 1,
                    crate::platform::KeyCode::A => 2,
                    crate::platform::KeyCode::S => 3,
                    crate::platform::KeyCode::D => 4,
                    crate::platform::KeyCode::Space => 5,
                    _ => 0,
                };
                (code as i32).to_le_bytes().to_vec()
            })
            .collect();

        prediction.submit_input(input_data);
    }
}

/// 预测执行系统 - 执行未确认的输入
pub fn prediction_execute_system(
    prediction: Res<ClientPredictionManager>,
    mut query: Query<(&mut PredictionComponent, &mut crate::ecs::Transform)>,
    time: Res<crate::ecs::Time>,
) {
    // 获取未确认的输入
    let unconfirmed = prediction.get_unconfirmed_inputs();

    // 执行预测逻辑
    for cmd in unconfirmed {
        // 反序列化输入并应用到实体
        // 简单的反序列化：从字节恢复KeyCode
        let mut input_keys = Vec::new();
        for chunk in cmd.input_data.chunks(8) {
            if chunk.len() == 8 {
                let code = u64::from_le_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ]);
                match code {
                    1 => input_keys.push(crate::platform::KeyCode::W),
                    2 => input_keys.push(crate::platform::KeyCode::A),
                    3 => input_keys.push(crate::platform::KeyCode::S),
                    4 => input_keys.push(crate::platform::KeyCode::D),
                    5 => input_keys.push(crate::platform::KeyCode::Space),
                    _ => {}
                }
            }
        }

        // 根据输入更新实体transform
        let move_speed = 5.0 * time.delta_seconds;
        for (_pred, mut transform) in query.iter_mut() {
            for key in &input_keys {
                match key {
                    crate::platform::KeyCode::W => {
                        transform.pos.z -= move_speed; // 向前移动
                    }
                    crate::platform::KeyCode::S => {
                        transform.pos.z += move_speed; // 向后移动
                    }
                    crate::platform::KeyCode::A => {
                        transform.pos.x -= move_speed; // 向左移动
                    }
                    crate::platform::KeyCode::D => {
                        transform.pos.x += move_speed; // 向右移动
                    }
                    crate::platform::KeyCode::Space => {
                        transform.pos.y += move_speed; // 向上移动（跳跃）
                    }
                    _ => {}
                }
            }
        }
    }
}

/// 回滚系统 - 处理状态回滚
pub fn rollback_system(
    mut prediction: ResMut<ClientPredictionManager>,
    mut query: Query<(&mut PredictionComponent, &mut crate::ecs::Transform, Entity)>,
    server_tick: Res<crate::network::NetworkState>,
) {
    let confirmed_tick = server_tick.current_tick;
    let last_confirmed = prediction.last_confirmed_tick();

    // 检查是否需要回滚
    if confirmed_tick > last_confirmed {
        // 服务器已确认新状态，检查是否需要回滚
        for (mut pred, mut transform, entity) in query.iter_mut() {
            if pred.current_predicted_tick > confirmed_tick {
                // 需要回滚
                pred.is_rolling_back = true;
                pred.rollback_target_tick = Some(confirmed_tick);

                // 获取回滚目标快照
                if let Some(snapshot) = prediction.rollback_to(confirmed_tick) {
                    // 恢复实体状态
                    if let Some((_snapshot_entity, state)) =
                        snapshot.entity_states.iter().find(|(e, _)| *e == entity)
                    {
                        transform.pos = state.position;
                        transform.rot = state.rotation;
                        transform.scale = state.scale;
                    }

                    // 重放未确认的输入
                    let replay_commands =
                        prediction.replay_inputs(confirmed_tick, pred.current_predicted_tick);
                    for cmd in replay_commands {
                        // 重放输入命令
                        // 反序列化输入
                        let mut input_keys = Vec::new();
                        for chunk in cmd.input_data.chunks(8) {
                            if chunk.len() == 8 {
                                let code = u64::from_le_bytes([
                                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5],
                                    chunk[6], chunk[7],
                                ]);
                                match code {
                                    1 => input_keys.push(crate::platform::KeyCode::W),
                                    2 => input_keys.push(crate::platform::KeyCode::A),
                                    3 => input_keys.push(crate::platform::KeyCode::S),
                                    4 => input_keys.push(crate::platform::KeyCode::D),
                                    5 => input_keys.push(crate::platform::KeyCode::Space),
                                    _ => {}
                                }
                            }
                        }

                        // 重新执行输入（使用固定的时间步长）
                        let move_speed = 5.0 * (1.0 / 60.0); // 假设60fps
                        for key in &input_keys {
                            match key {
                                crate::platform::KeyCode::W => {
                                    transform.pos.z -= move_speed;
                                }
                                crate::platform::KeyCode::S => {
                                    transform.pos.z += move_speed;
                                }
                                crate::platform::KeyCode::A => {
                                    transform.pos.x -= move_speed;
                                }
                                crate::platform::KeyCode::D => {
                                    transform.pos.x += move_speed;
                                }
                                crate::platform::KeyCode::Space => {
                                    transform.pos.y += move_speed;
                                }
                                _ => {}
                            }
                        }
                    }

                    pred.is_rolling_back = false;
                    pred.last_confirmed_tick = confirmed_tick;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_command_creation() {
        let cmd = InputCommand::new(1, vec![1, 2, 3]);
        assert_eq!(cmd.sequence, 1);
        assert!(!cmd.confirmed);
    }

    #[test]
    fn test_prediction_manager() {
        let mut manager = ClientPredictionManager::default();

        // 提交输入
        let seq = manager.submit_input(vec![1, 2, 3]);
        assert_eq!(seq, 1);

        // 确认输入
        manager.confirm_input(1, 10);
        assert_eq!(manager.last_confirmed_tick(), 10);
    }

    #[test]
    fn test_snapshot_creation() {
        let mut manager = ClientPredictionManager::default();
        let mut world = World::new();

        let entity1 = world.spawn_empty().id();
        let entity2 = world.spawn_empty().id();

        let states = vec![
            (entity1, EntityState::default()),
            (entity2, EntityState::default()),
        ];

        manager.create_snapshot(5, states);
        assert_eq!(manager.snapshots.len(), 1);

        let snapshot = manager.get_snapshot(5);
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().tick, 5);
    }

    #[test]
    fn test_input_capture_and_prediction() {
        let mut manager = ClientPredictionManager::default();

        // 提交输入
        let input_data = vec![1, 0, 0, 0, 0, 0, 0, 0]; // W键
        let seq = manager.submit_input(input_data);
        assert_eq!(seq, 1);

        // 验证未确认输入
        let unconfirmed = manager.get_unconfirmed_inputs();
        assert_eq!(unconfirmed.len(), 1);
        assert_eq!(unconfirmed[0].sequence, 1);

        // 确认输入
        manager.confirm_input(1, 10);
        let unconfirmed_after = manager.get_unconfirmed_inputs();
        assert_eq!(unconfirmed_after.len(), 0);
    }

    #[test]
    fn test_rollback_and_replay() {
        let mut manager = ClientPredictionManager::default();
        let mut world = World::new();

        let entity = world.spawn_empty().id();
        let state = EntityState {
            position: glam::Vec3::new(10.0, 20.0, 30.0),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
            velocity: glam::Vec3::ZERO,
            custom_data: Vec::new(),
        };

        // 创建快照
        manager.create_snapshot(5, vec![(entity, state.clone())]);

        // 回滚到快照
        let snapshot = manager.rollback_to(5);
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().tick, 5);
        assert_eq!(snapshot.unwrap().entity_states.len(), 1);
    }
}
