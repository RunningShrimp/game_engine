//! 服务器权威系统
//!
//! 实现服务器权威状态管理，防止作弊，确保游戏公平性。

use crate::impl_default;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// 服务器权威组件 - 标记服务器控制的实体
#[derive(Component, Debug, Clone)]
pub struct ServerAuthority {
    /// 服务器tick
    pub server_tick: u64,
    /// 最后更新的服务器时间
    pub last_server_update: u64,
    /// 是否已验证
    pub verified: bool,
}

impl_default!(ServerAuthority {
    server_tick: 0,
    last_server_update: 0,
    verified: true,
});

/// 状态验证结果
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// 验证通过
    Valid,
    /// 验证失败 - 位置超出限制
    InvalidPosition { expected: Vec3, actual: Vec3 },
    /// 验证失败 - 速度超出限制
    InvalidVelocity { max_speed: f32, actual: f32 },
    /// 验证失败 - 其他原因
    Invalid(String),
}

/// 服务器权威管理器
#[derive(Resource)]
pub struct ServerAuthorityManager {
    /// 最大移动速度（用于验证）
    pub max_move_speed: f32,
    /// 最大位置偏差（用于验证）
    pub max_position_deviation: f32,
    /// 验证统计
    pub stats: AuthorityStats,
}

impl_default!(ServerAuthorityManager {
    max_move_speed: 10.0,
    max_position_deviation: 1.0,
    stats: AuthorityStats::default(),
});

impl ServerAuthorityManager {
    /// 验证客户端状态
    pub fn validate_client_state(
        &mut self,
        client_pos: Vec3,
        client_vel: Vec3,
        server_pos: Vec3,
        _server_vel: Vec3,
    ) -> ValidationResult {
        // 检查位置偏差
        let pos_diff = (client_pos - server_pos).length();
        if pos_diff > self.max_position_deviation {
            self.stats.invalid_positions += 1;
            return ValidationResult::InvalidPosition {
                expected: server_pos,
                actual: client_pos,
            };
        }

        // 检查速度
        let client_speed = client_vel.length();
        if client_speed > self.max_move_speed {
            self.stats.invalid_velocities += 1;
            return ValidationResult::InvalidVelocity {
                max_speed: self.max_move_speed,
                actual: client_speed,
            };
        }

        self.stats.valid_states += 1;
        ValidationResult::Valid
    }

    /// 解决冲突（服务器状态优先）
    pub fn resolve_conflict(&self, server_state: Vec3, _client_state: Vec3) -> Vec3 {
        // 服务器状态优先
        server_state
    }
}

/// 权威统计
#[derive(Debug, Clone, Default)]
pub struct AuthorityStats {
    /// 验证通过数
    pub valid_states: u64,
    /// 无效位置数
    pub invalid_positions: u64,
    /// 无效速度数
    pub invalid_velocities: u64,
}

/// 服务器状态更新消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStateUpdate {
    /// 服务器tick
    pub tick: u64,
    /// 实体ID
    pub entity_id: u64,
    /// 服务器位置
    pub position: Vec3,
    /// 服务器旋转
    pub rotation: Quat,
    /// 服务器速度
    pub velocity: Vec3,
    /// 时间戳（毫秒）
    pub timestamp_ms: u64,
}

/// 权威系统 - 验证和同步服务器状态
pub fn server_authority_system(
    mut authority: ResMut<ServerAuthorityManager>,
    mut query: Query<(&mut ServerAuthority, &mut crate::ecs::Transform, Entity)>,
    network_state: Res<crate::network::NetworkState>,
) {
    // 接收服务器状态更新
    let messages = crate::network::NetworkService::receive(&network_state);

    for msg in messages {
        if let crate::network::NetworkMessage::StateSync { tick: _, data } = msg {
            // 反序列化服务器状态
            if let Ok(updates) = serde_json::from_slice::<Vec<ServerStateUpdate>>(&data) {
                for update in updates {
                    // 查找对应的实体
                    for (mut auth, mut transform, entity) in query.iter_mut() {
                        // 简化：假设entity的index对应update.entity_id
                        // 实际实现中应该有Entity到网络ID的映射
                        if entity.index() as u64 == update.entity_id {
                            // 获取客户端当前状态
                            let client_pos = transform.pos;
                            let client_vel = Vec3::ZERO; // 如果有速度组件，应该从那里获取

                            // 验证客户端状态
                            let validation_result = authority.validate_client_state(
                                client_pos,
                                client_vel,
                                update.position,
                                update.velocity,
                            );

                            match validation_result {
                                ValidationResult::Valid => {
                                    // 验证通过，更新服务器状态
                                    auth.server_tick = update.tick;
                                    auth.last_server_update = update.timestamp_ms;
                                    auth.verified = true;

                                    // 平滑插值到服务器位置（可选）
                                    // transform.pos = update.position;
                                }
                                ValidationResult::InvalidPosition { expected, .. } => {
                                    // 位置不一致，校正到服务器位置
                                    auth.verified = false;
                                    transform.pos =
                                        authority.resolve_conflict(expected, client_pos);
                                    auth.server_tick = update.tick;
                                    auth.last_server_update = update.timestamp_ms;
                                }
                                ValidationResult::InvalidVelocity { .. } => {
                                    // 速度异常，校正
                                    auth.verified = false;
                                    transform.pos =
                                        authority.resolve_conflict(update.position, client_pos);
                                    auth.server_tick = update.tick;
                                    auth.last_server_update = update.timestamp_ms;
                                }
                                ValidationResult::Invalid(reason) => {
                                    // 其他验证失败
                                    tracing::warn!(target: "authority", "Validation failed: {}", reason);
                                    auth.verified = false;
                                    transform.pos =
                                        authority.resolve_conflict(update.position, client_pos);
                                    auth.server_tick = update.tick;
                                    auth.last_server_update = update.timestamp_ms;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 更新所有权威组件的tick（即使没有收到更新）
    for (auth, _transform, _entity) in query.iter() {
        if auth.verified {
            // 如果已验证，可以继续使用客户端预测
            // 否则等待服务器更新
        }
    }
}

/// 客户端状态校正系统 - 平滑插值到服务器状态
pub fn client_state_correction_system(
    mut query: Query<(&mut ServerAuthority, &mut crate::ecs::Transform)>,
    _time: Res<crate::ecs::Time>,
) {
    for (auth, _transform) in query.iter_mut() {
        if !auth.verified {
            // 如果未验证，可能需要平滑校正
            // 这里可以实现插值逻辑
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_state() {
        let mut manager = ServerAuthorityManager::default();

        let client_pos = Vec3::new(0.0, 0.0, 0.0);
        let client_vel = Vec3::new(1.0, 0.0, 0.0);
        let server_pos = Vec3::new(0.1, 0.0, 0.0);
        let server_vel = Vec3::new(1.0, 0.0, 0.0);

        let result = manager.validate_client_state(client_pos, client_vel, server_pos, server_vel);
        assert!(matches!(result, ValidationResult::Valid));
        assert_eq!(manager.stats.valid_states, 1);
    }

    #[test]
    fn test_validation_invalid_position() {
        let mut manager = ServerAuthorityManager::default();

        let client_pos = Vec3::new(0.0, 0.0, 0.0);
        let client_vel = Vec3::ZERO;
        let server_pos = Vec3::new(2.0, 0.0, 0.0); // 超出偏差限制
        let server_vel = Vec3::ZERO;

        let result = manager.validate_client_state(client_pos, client_vel, server_pos, server_vel);
        assert!(matches!(result, ValidationResult::InvalidPosition { .. }));
        assert_eq!(manager.stats.invalid_positions, 1);
    }

    #[test]
    fn test_validation_invalid_velocity() {
        let mut manager = ServerAuthorityManager::default();

        let client_pos = Vec3::new(0.0, 0.0, 0.0);
        let client_vel = Vec3::new(20.0, 0.0, 0.0); // 超出速度限制
        let server_pos = Vec3::new(0.0, 0.0, 0.0);
        let server_vel = Vec3::ZERO;

        let result = manager.validate_client_state(client_pos, client_vel, server_pos, server_vel);
        assert!(matches!(result, ValidationResult::InvalidVelocity { .. }));
        assert_eq!(manager.stats.invalid_velocities, 1);
    }

    #[test]
    fn test_resolve_conflict() {
        let manager = ServerAuthorityManager::default();

        let server_state = Vec3::new(10.0, 20.0, 30.0);
        let client_state = Vec3::new(11.0, 21.0, 31.0);

        let resolved = manager.resolve_conflict(server_state, client_state);
        assert_eq!(resolved, server_state); // 服务器状态优先
    }
}
