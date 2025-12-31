//! # GPU粒子系统
//!
//! 基于计算着色器的高性能粒子系统。
//!
//! ## 功能特性
//!
//! - **GPU模拟**: 数百万粒子实时模拟
//! - **计算着色器**: GPGPU加速
//! - **多种发射器**: 点、线、面、体积发射
//! - **力场**: 重力、风力、吸引力
//! - **碰撞检测**: 与场景几何体碰撞

use crate::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// =============================================================================
// 粒子
// =============================================================================

/// 粒子ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticleId(pub u64);

impl ParticleId {
    /// 创建新的粒子ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 粒子数据（GPU对齐）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ParticleData {
    /// 位置
    pub position: [f32; 3],
    /// 速度
    pub velocity: [f32; 3],
    /// 颜色
    pub color: [f32; 4],
    /// 大小
    pub size: f32,
    /// 生命值 (0.0 - 1.0)
    pub lifetime: f32,
    /// 旋转角度
    pub rotation: f32,
    /// 纹理索引
    pub texture_index: u32,
}

// =============================================================================
// 粒子发射器
// =============================================================================

/// 发射器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmitterType {
    /// 点发射器
    Point,
    /// 线发射器
    Line,
    /// 圆发射器
    Circle,
    /// 球发射器
    Sphere,
    /// 盒发射器
    Box,
    /// 圆锥发射器
    Cone,
}

/// 发射器形状
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterShape {
    /// 类型
    pub emitter_type: EmitterType,
    /// 位置
    pub position: glam::Vec3,
    /// 旋转
    pub rotation: glam::Quat,
    /// 尺寸
    pub size: glam::Vec3,
}

/// 粒子发射器
#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    /// 发射器ID
    pub id: ParticleId,
    /// 名称
    pub name: String,
    /// 形状
    pub shape: EmitterShape,
    /// 发射速率（粒子/秒）
    pub emission_rate: f32,
    /// 粒子生命周期（秒）
    pub lifetime: f32,
    /// 初始速度范围
    pub velocity_range: (f32, f32),
    /// 初始大小范围
    pub size_range: (f32, f32),
    /// 初始颜色
    pub color: glam::Vec4,
    /// 是否启用
    pub enabled: bool,
    /// 是否循环
    pub looping: bool,
    /// 最大粒子数
    pub max_particles: u32,
    /// 当前粒子数
    pub active_count: u32,
}

impl ParticleEmitter {
    /// 创建新发射器
    pub fn new(id: ParticleId, name: String) -> Self {
        Self {
            id,
            name,
            shape: EmitterShape {
                emitter_type: EmitterType::Point,
                position: glam::Vec3::ZERO,
                rotation: glam::Quat::IDENTITY,
                size: glam::Vec3::ONE,
            },
            emission_rate: 100.0,
            lifetime: 5.0,
            velocity_range: (1.0, 5.0),
            size_range: (0.1, 0.5),
            color: glam::Vec4::ONE,
            enabled: true,
            looping: true,
            max_particles: 10000,
            active_count: 0,
        }
    }

    /// 计算一帧发射数量
    pub fn calculate_emission(&self, delta_time: f32) -> u32 {
        if !self.enabled {
            return 0;
        }

        let count = (self.emission_rate * delta_time) as u32;
        count.min(self.max_particles - self.active_count)
    }
}

// =============================================================================
// 力场
// =============================================================================

/// 力场类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForceFieldType {
    /// 重力
    Gravity,
    /// 风力
    Wind,
    /// 吸引力
    Attraction,
    /// 排斥力
    Repulsion,
    /// 漩涡力
    Vortex,
    /// 阻尼
    Drag,
}

/// 力场
#[derive(Debug, Clone)]
pub struct ForceField {
    /// 力场类型
    pub field_type: ForceFieldType,
    /// 力的方向/位置
    pub direction: glam::Vec3,
    /// 力的大小
    pub magnitude: f32,
    /// 作用半径
    pub radius: f32,
    /// 是否启用
    pub enabled: bool,
}

impl ForceField {
    /// 创建重力场
    pub fn gravity(magnitude: f32) -> Self {
        Self {
            field_type: ForceFieldType::Gravity,
            direction: glam::Vec3::new(0.0, -1.0, 0.0),
            magnitude,
            radius: f32::MAX,
            enabled: true,
        }
    }

    /// 创建风力场
    pub fn wind(direction: glam::Vec3, magnitude: f32) -> Self {
        Self {
            field_type: ForceFieldType::Wind,
            direction: direction.normalize(),
            magnitude,
            radius: f32::MAX,
            enabled: true,
        }
    }

    /// 创建吸引力场
    pub fn attraction(center: glam::Vec3, magnitude: f32, radius: f32) -> Self {
        Self {
            field_type: ForceFieldType::Attraction,
            direction: center,
            magnitude,
            radius,
            enabled: true,
        }
    }
}

// =============================================================================
// 粒子系统
// =============================================================================

/// GPU粒子系统
pub struct GpuParticleSystem {
    /// 发射器
    emitters: HashMap<ParticleId, ParticleEmitter>,
    /// 力场
    force_fields: Vec<ForceField>,
    /// 粒子缓冲区（GPU）
    particle_buffer: Option<ParticleBuffer>,
    /// 计算着色器管线
    compute_pipeline: Option<ComputePipeline>,
}

/// 粒子缓冲区（GPU）
#[derive(Debug, Clone)]
struct ParticleBuffer {
    /// 缓冲区ID
    id: u32,
    /// 容量
    capacity: u32,
    /// 当前数量
    count: u32,
}

/// 计算着色器管线
#[derive(Debug, Clone)]
struct ComputePipeline {
    /// 着色器模块
    shader_module: u32,
    /// 管线布局
    pipeline_layout: u32,
}

impl GpuParticleSystem {
    /// 创建新系统
    pub fn new() -> Self {
        Self {
            emitters: HashMap::new(),
            force_fields: Vec::new(),
            particle_buffer: None,
            compute_pipeline: None,
        }
    }

    /// 添加发射器
    pub fn add_emitter(&mut self, emitter: ParticleEmitter) {
        self.emitters.insert(emitter.id, emitter);
    }

    /// 移除发射器
    pub fn remove_emitter(&mut self, id: ParticleId) -> bool {
        self.emitters.remove(&id).is_some()
    }

    /// 获取发射器
    pub fn get_emitter(&self, id: ParticleId) -> Option<&ParticleEmitter> {
        self.emitters.get(&id)
    }

    /// 获取可变发射器
    pub fn get_emitter_mut(&mut self, id: ParticleId) -> Option<&mut ParticleEmitter> {
        self.emitters.get_mut(&id)
    }

    /// 添加力场
    pub fn add_force_field(&mut self, field: ForceField) {
        self.force_fields.push(field);
    }

    /// 移除力场
    pub fn remove_force_field(&mut self, index: usize) -> bool {
        if index < self.force_fields.len() {
            self.force_fields.remove(index);
            true
        } else {
            false
        }
    }

    /// 更新系统
    pub fn update(&mut self, delta_time: f32) {
        // 更新所有发射器
        for emitter in self.emitters.values_mut() {
            if emitter.enabled {
                let emitted = emitter.calculate_emission(delta_time);
                emitter.active_count = emitter.active_count.saturating_add(emitted);
            }
        }

        // TODO: 在GPU上运行粒子模拟
        self.simulate_particles_gpu(delta_time);
    }

    /// GPU粒子模拟
    fn simulate_particles_gpu(&mut self, delta_time: f32) {
        // TODO: 实现实际的GPU模拟
        // 这里使用简化实现

        // 计算粒子数量
        let total_particles: u32 = self.emitters.values().map(|e| e.active_count).sum();

        // 如果没有GPU缓冲区，创建一个
        if self.particle_buffer.is_none() && total_particles > 0 {
            self.particle_buffer = Some(ParticleBuffer {
                id: 1,
                capacity: 100000,
                count: total_particles,
            });
        }
    }

    /// 获取总粒子数
    pub fn total_particles(&self) -> u32 {
        self.emitters.values().map(|e| e.active_count).sum()
    }

    /// 获取发射器数量
    pub fn emitter_count(&self) -> usize {
        self.emitters.len()
    }
}

impl Default for GpuParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// 粒子事件
// =============================================================================

/// 粒子事件
#[derive(Debug, Clone)]
pub enum ParticleEvent {
    /// 发射器创建
    EmitterCreated {
        emitter_id: ParticleId,
        name: String,
    },
    /// 发射器销毁
    EmitterDestroyed { emitter_id: ParticleId },
    /// 粒子发射
    ParticlesEmitted { emitter_id: ParticleId, count: u32 },
    /// 力场添加
    ForceFieldAdded { field_type: ForceFieldType },
}

impl DomainEvent for ParticleEvent {
    fn event_type(&self) -> &'static str {
        match self {
            ParticleEvent::EmitterCreated { .. } => "EmitterCreated",
            ParticleEvent::EmitterDestroyed { .. } => "EmitterDestroyed",
            ParticleEvent::ParticlesEmitted { .. } => "ParticlesEmitted",
            ParticleEvent::ForceFieldAdded { .. } => "ForceFieldAdded",
        }
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// =============================================================================
// ECS集成
// =============================================================================

/// GPU粒子系统资源
#[derive(Resource)]
pub struct GpuParticleSystemResource {
    pub system: GpuParticleSystem,
}

/// 发射器组件
#[derive(Component, Debug, Clone)]
pub struct ParticleEmitterComponent {
    pub emitter_id: ParticleId,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_creation() {
        let emitter = ParticleEmitter::new(ParticleId::new(1), "test_emitter".to_string());

        assert_eq!(emitter.active_count, 0);
        assert!(emitter.enabled);
    }

    #[test]
    fn test_emission_calculation() {
        let mut emitter = ParticleEmitter::new(ParticleId::new(1), "test_emitter".to_string());

        emitter.emission_rate = 100.0;

        let count = emitter.calculate_emission(0.1); // 100 Hz * 0.1s = 10 particles
        assert_eq!(count, 10);
    }

    #[test]
    fn test_force_field_creation() {
        let gravity = ForceField::gravity(9.8);
        assert_eq!(gravity.field_type, ForceFieldType::Gravity);
        assert_eq!(gravity.magnitude, 9.8);

        let wind = ForceField::wind(glam::Vec3::new(1.0, 0.0, 0.0), 5.0);
        assert_eq!(wind.field_type, ForceFieldType::Wind);
    }

    #[test]
    fn test_system_creation() {
        let system = GpuParticleSystem::new();
        assert_eq!(system.total_particles(), 0);
        assert_eq!(system.emitter_count(), 0);
    }

    #[test]
    fn test_add_emitter() {
        let mut system = GpuParticleSystem::new();
        let emitter = ParticleEmitter::new(ParticleId::new(1), "test".to_string());

        system.add_emitter(emitter);
        assert_eq!(system.emitter_count(), 1);
    }
}
