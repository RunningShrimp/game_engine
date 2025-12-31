//! # Particle System
//!
//! 本模块提供高性能粒子系统，支持SIMD加速批量处理。
//!
//! ## 功能特性
//!
//! - **SIMD批量处理** - 使用SIMD指令加速粒子更新
//! - **力场效果** - 支持各种力场（引力、涡流等）
//! - **碰撞检测** - 粒子与平面的碰撞检测
//! - **ECS集成** - 完全集成Bevy ECS
//!
//! ## 主要组件
//!
//! - [`SimdParticle`] - 粒子组件
//! - [`SimdParticleProcessor`] - SIMD批量处理器
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::particles::{SimdParticle, simd_particle_update_system};
//! use bevy_ecs::prelude::*;
//!
//! // 创建粒子实体
//! let mut world = World::new();
//! world.spawn(SimdParticle::default());
//! ```

/// SIMD粒子批量处理集成
pub mod simd_integration;

pub use simd_integration::{
    SimdParticle, SimdParticleProcessor, simd_particle_force_field_system,
    simd_particle_update_system,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_particle_default() {
        let particle = SimdParticle::default();
        assert_eq!(particle.life, 1.0);
    }
}
