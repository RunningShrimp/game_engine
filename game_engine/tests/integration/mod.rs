//! 集成测试模块
//!
//! 测试引擎各个系统之间的集成，包括：
//! - 场景加载和序列化完整流程
//! - 事件系统端到端测试
//! - 事件溯源完整流程测试
//! - 资源加载完整流程测试
//! - 性能回归测试
//! - GPU实例化渲染性能测试
//! - ECS系统调度性能测试
//! - 物理引擎空间分区性能测试

mod scene_serialization_test;
mod event_system_e2e_test;
mod event_sourcing_e2e_test;
mod resource_loading_e2e_test;
mod performance_regression_test;
mod instance_batch_performance_test;
mod ecs_scheduling_performance_test;
mod physics_spatial_partition_test;
mod ecs_benchmark_test;
mod physics_benchmark_test;
mod resource_dependency_hotreload_test;

