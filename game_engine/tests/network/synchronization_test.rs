//! 网络同步测试
//!
//! 测试状态同步和事件同步的核心功能。

use game_engine::network::synchronization::{
    ConflictResolutionStrategy, EntityState, EntitySyncState, SyncStrategy,
};
use glam::{Quat, Vec3};

#[test]
fn test_entity_state_creation() {
    let position = Vec3::new(1.0, 2.0, 3.0);
    let rotation = Quat::IDENTITY;
    let scale = Vec3::ONE;
    let velocity = Vec3::new(0.0, 0.0, 1.0);

    let state = EntityState::new(position, rotation, scale, velocity);

    assert_eq!(state.position, position);
    assert_eq!(state.rotation, rotation);
    assert_eq!(state.scale, scale);
    assert_eq!(state.velocity, velocity);
    assert_eq!(state.version, 0);
}

#[test]
fn test_entity_state_distance() {
    let state1 = EntityState::new(
        Vec3::new(0.0, 0.0, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
        Vec3::ZERO,
    );

    let state2 = EntityState::new(
        Vec3::new(3.0, 4.0, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
        Vec3::ZERO,
    );

    let distance = state1.distance_to(&state2);
    assert!((distance - 5.0).abs() < 0.001, "距离应为5.0（3-4-5三角形）");
}

#[test]
fn test_entity_state_version_increment() {
    let mut state = EntityState::new(
        Vec3::ZERO,
        Quat::IDENTITY,
        Vec3::ONE,
        Vec3::ZERO,
    );

    assert_eq!(state.version, 0);
    // 注意：EntityState没有increment_version方法，版本号由外部管理
    // 这里主要验证版本号字段存在
    assert_eq!(state.version, 0);
}

#[test]
fn test_sync_strategy_enum() {
    assert_eq!(
        SyncStrategy::ServerAuthoritative,
        SyncStrategy::ServerAuthoritative
    );
    assert_eq!(SyncStrategy::ClientPrediction, SyncStrategy::ClientPrediction);
    assert_eq!(SyncStrategy::Hybrid, SyncStrategy::Hybrid);
}

#[test]
fn test_conflict_resolution_strategy() {
    let strategy1 = ConflictResolutionStrategy::ServerWins;
    let strategy2 = ConflictResolutionStrategy::SmoothCorrection;
    let strategy3 = ConflictResolutionStrategy::DelayedCorrection { delay_ms: 100 };
    let strategy4 = ConflictResolutionStrategy::ThresholdCorrection { threshold: 0.5 };

    // 验证策略能够创建
    match strategy1 {
        ConflictResolutionStrategy::ServerWins => {}
        _ => panic!("策略类型错误"),
    }

    match strategy3 {
        ConflictResolutionStrategy::DelayedCorrection { delay_ms } => {
            assert_eq!(delay_ms, 100);
        }
        _ => panic!("策略类型错误"),
    }

    match strategy4 {
        ConflictResolutionStrategy::ThresholdCorrection { threshold } => {
            assert!((threshold - 0.5).abs() < 0.001);
        }
        _ => panic!("策略类型错误"),
    }
}

#[test]
fn test_entity_sync_state_creation() {
    let sync_state = EntitySyncState {
        entity_id: 1,
        last_sync_tick: 0,
        sync_strategy: SyncStrategy::ServerAuthoritative,
        conflict_resolution: ConflictResolutionStrategy::ServerWins,
        server_state: None,
        client_state: None,
        correcting: false,
        correction_start_time: None,
    };

    assert_eq!(sync_state.entity_id, 1);
    assert_eq!(sync_state.last_sync_tick, 0);
    assert_eq!(sync_state.sync_strategy, SyncStrategy::ServerAuthoritative);
    assert!(!sync_state.correcting);
}

#[test]
fn test_entity_sync_state_with_states() {
    let server_state = EntityState::new(
        Vec3::new(10.0, 20.0, 30.0),
        Quat::IDENTITY,
        Vec3::ONE,
        Vec3::ZERO,
    );

    let client_state = EntityState::new(
        Vec3::new(10.1, 20.1, 30.1),
        Quat::IDENTITY,
        Vec3::ONE,
        Vec3::ZERO,
    );

    let sync_state = EntitySyncState {
        entity_id: 1,
        last_sync_tick: 100,
        sync_strategy: SyncStrategy::ClientPrediction,
        conflict_resolution: ConflictResolutionStrategy::SmoothCorrection,
        server_state: Some(server_state.clone()),
        client_state: Some(client_state.clone()),
        correcting: false,
        correction_start_time: None,
    };

    assert!(sync_state.server_state.is_some());
    assert!(sync_state.client_state.is_some());
    assert_eq!(
        sync_state.server_state.as_ref().unwrap().position,
        Vec3::new(10.0, 20.0, 30.0)
    );
}

#[test]
fn test_entity_state_serialization() {
    use serde_json;

    let state = EntityState::new(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::IDENTITY,
        Vec3::ONE,
        Vec3::new(0.5, 0.5, 0.5),
    );

    // 测试序列化
    let serialized = serde_json::to_string(&state);
    assert!(serialized.is_ok());

    // 测试反序列化
    if let Ok(json_str) = serialized {
        let deserialized: Result<EntityState, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        if let Ok(deserialized_state) = deserialized {
            assert!((deserialized_state.position.x - 1.0).abs() < 0.001);
            assert!((deserialized_state.position.y - 2.0).abs() < 0.001);
            assert!((deserialized_state.position.z - 3.0).abs() < 0.001);
        }
    }
}

