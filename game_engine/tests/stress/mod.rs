//! 压力测试模块
//!
//! 测试系统在高负载下的稳定性和性能，包括：
//! - 渲染压力测试（大量实体）
//! - ECS压力测试（大量组件）
//! - 网络压力测试（高并发）

mod render_stress_test;
mod ecs_stress_test;
mod network_stress_test;

