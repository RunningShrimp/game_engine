//! 粒子系统管理器
//!
//! 管理多个粒子系统，提供统一的更新和渲染接口。

use crate::render::particles::emitter::{GpuParticleSystem, ParticleEmitter};
use crate::render::particles::GpuParticle;
use wgpu::{CommandEncoder, Device, Queue};

/// 粒子系统管理器
///
/// 管理多个GPU粒子系统，提供统一的更新和渲染接口。
pub struct ParticleSystemManager {
    /// 粒子系统列表
    systems: Vec<GpuParticleSystem>,
    /// 最大系统数
    max_systems: usize,
}

impl ParticleSystemManager {
    /// 创建新的粒子系统管理器
    ///
    /// # 参数
    ///
    /// * `max_systems` - 最大系统数
    ///
    /// # 返回
    ///
    /// 返回一个初始化的管理器。
    pub fn new(max_systems: usize) -> Self {
        Self {
            systems: Vec::with_capacity(max_systems),
            max_systems,
        }
    }

    /// 添加粒子系统
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `max_particles` - 最大粒子数
    ///
    /// # 返回
    ///
    /// 返回系统ID（如果成功）。
    pub fn add_system(
        &mut self,
        device: &Device,
        max_particles: u32,
    ) -> Option<usize> {
        if self.systems.len() >= self.max_systems {
            return None;
        }

        let system = GpuParticleSystem::new(device, max_particles);
        let id = self.systems.len();
        self.systems.push(system);
        Some(id)
    }

    /// 获取粒子系统（可变引用）
    ///
    /// # 参数
    ///
    /// * `id` - 系统ID
    ///
    /// # 返回
    ///
    /// 返回粒子系统的可变引用（如果存在）。
    pub fn get_system_mut(&mut self, id: usize) -> Option<&mut GpuParticleSystem> {
        self.systems.get_mut(id)
    }

    /// 更新所有粒子系统
    ///
    /// # 参数
    ///
    /// * `encoder` - 命令编码器
    /// * `device` - WGPU设备
    /// * `queue` - 命令队列
    /// * `delta_time` - 时间增量
    pub fn update_all(
        &mut self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        delta_time: f32,
    ) {
        for system in &mut self.systems {
            // 更新Uniform（需要从ParticleEmitter获取参数）
            // 这里简化处理，实际应该从ECS组件获取
        }
    }

    /// 获取系统数量
    ///
    /// # 返回
    ///
    /// 返回当前系统数量。
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// 移除系统
    ///
    /// # 参数
    ///
    /// * `id` - 系统ID
    ///
    /// # 返回
    ///
    /// 如果成功移除返回`true`，否则返回`false`。
    pub fn remove_system(&mut self, id: usize) -> bool {
        if id < self.systems.len() {
            self.systems.remove(id);
            true
        } else {
            false
        }
    }

    /// 清空所有系统
    pub fn clear(&mut self) {
        self.systems.clear();
    }
}

impl Default for ParticleSystemManager {
    fn default() -> Self {
        Self::new(64)
    }
}

// 注意：保留手动实现，因为default()调用了new(64)，有特殊逻辑

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_system_manager() {
        let manager = ParticleSystemManager::new(10);
        assert_eq!(manager.system_count(), 0);
        assert_eq!(manager.max_systems, 10);
    }
}

