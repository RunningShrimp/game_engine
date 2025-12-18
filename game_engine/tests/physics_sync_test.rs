//! 测试 PhysicsDomainService 的线程安全性

use game_engine::domain::services::PhysicsDomainService;

// 测试 PhysicsDomainService 是否实现了 Send 特性
#[test]
fn test_physics_domain_service_send() {
    fn assert_send<T: Send>() {}
    assert_send::<PhysicsDomainService>();
}

// 测试 PhysicsDomainService 是否实现了 Sync 特性
#[test]
fn test_physics_domain_service_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<PhysicsDomainService>();
}