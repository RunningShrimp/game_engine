//! GPU 粒子系统模块
//!
//! 支持百万级粒子的实时模拟和渲染，完全在 GPU 上执行。
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  GPU Particle System                     │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. Emission (Compute Shader)                            │
//! │     - 从死亡粒子池分配新粒子                               │
//! │     - 初始化位置、速度、生命周期                           │
//! │                                                          │
//! │  2. Simulation (Compute Shader)                          │
//! │     - 物理更新（重力、风力、碰撞）                         │
//! │     - 生命周期更新                                        │
//! │     - 颜色/大小随生命周期变化                             │
//! │                                                          │
//! │  3. Rendering (Vertex + Fragment Shader)                 │
//! │     - Billboard 渲染或 Point Sprite                      │
//! │     - 基于深度的软粒子                                    │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! // 创建粒子发射器
//! let emitter = ParticleEmitter {
//!     max_particles: 100_000,
//!     emission_rate: 1000.0,
//!     lifetime: 2.0..5.0,
//!     initial_velocity: Vec3::new(0.0, 5.0, 0.0),
//!     gravity: Vec3::new(0.0, -9.81, 0.0),
//!     ..Default::default()
//! };
//!
//! // 添加到实体
//! commands.spawn((emitter, Transform::default()));
//! ```

pub mod emitter;
pub mod system;

pub use emitter::{
    ColorGradient, ColorStop, GpuParticle, GpuParticleSystem, ParticleEmitter, ParticleEmitterConfig,
    ParticleShape, ParticleSystemStats, SizeOverLifetime,
};
pub use system::ParticleSystemManager;
