//! # 探针管理
//!
//! 管理DDGI探针的创建、更新和查询。

use glam::{Vec2, Vec3};

/// DDGI探针
#[derive(Debug, Clone)]
pub struct DDGIProbe {
    /// 探针位置（世界空间）
    pub position: Vec3,
    /// 辐照度（RGB）
    pub irradiance: Vec3,
    /// 深度值
    pub depth: f32,
    /// 偏移量（用于优化采样）
    pub offset: Vec2,
}

impl DDGIProbe {
    /// 创建新的探针
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            irradiance: Vec3::ZERO,
            depth: 1.0,
            offset: Vec2::ZERO,
        }
    }

    /// 重置探针数据
    pub fn reset(&mut self) {
        self.irradiance = Vec3::ZERO;
        self.depth = 1.0;
        self.offset = Vec2::ZERO;
    }

    /// 设置辐照度
    pub fn set_irradiance(&mut self, irradiance: Vec3) {
        self.irradiance = irradiance;
    }

    /// 设置深度
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth;
    }

    /// 设置偏移
    pub fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }
}

/// 探针管理器
pub struct ProbeManager {
    /// DDGI体积列表
    volumes: Vec<crate::render::gi::DDGIVolume>,
    /// 活跃体积索引
    active_volume: Option<usize>,
}

impl ProbeManager {
    /// 创建新的探针管理器
    pub fn new() -> Self {
        Self {
            volumes: Vec::new(),
            active_volume: None,
        }
    }

    /// 添加DDGI体积
    pub fn add_volume(&mut self, volume: crate::render::gi::DDGIVolume) {
        self.volumes.push(volume);
        if self.active_volume.is_none() {
            self.active_volume = Some(0);
        }
    }

    /// 移除体积
    pub fn remove_volume(&mut self, index: usize) -> Option<crate::render::gi::DDGIVolume> {
        if index < self.volumes.len() {
            let volume = self.volumes.remove(index);
            // 更新活跃索引
            if self.active_volume == Some(index) {
                self.active_volume = if self.volumes.is_empty() {
                    None
                } else {
                    Some(0)
                };
            } else if self.active_volume > Some(index) {
                self.active_volume = self.active_volume.map(|i| i - 1);
            }
            Some(volume)
        } else {
            None
        }
    }

    /// 设置活跃体积
    pub fn set_active_volume(&mut self, index: usize) -> bool {
        if index < self.volumes.len() {
            self.active_volume = Some(index);
            true
        } else {
            false
        }
    }

    /// 获取活跃体积
    pub fn active_volume(&self) -> Option<&crate::render::gi::DDGIVolume> {
        self.active_volume.and_then(|idx| self.volumes.get(idx))
    }

    /// 获取活跃体积（可变）
    pub fn active_volume_mut(&mut self) -> Option<&mut crate::render::gi::DDGIVolume> {
        self.active_volume.and_then(|idx| self.volumes.get_mut(idx))
    }

    /// 获取体积
    pub fn get_volume(&self, index: usize) -> Option<&crate::render::gi::DDGIVolume> {
        self.volumes.get(index)
    }

    /// 获取体积（可变）
    pub fn get_volume_mut(&mut self, index: usize) -> Option<&mut crate::render::gi::DDGIVolume> {
        self.volumes.get_mut(index)
    }

    /// 获取体积数量
    pub fn volume_count(&self) -> usize {
        self.volumes.len()
    }

    /// 更新所有体积
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), crate::render::gi::DDGIError> {
        for volume in &mut self.volumes {
            volume.update(device, queue, encoder)?;
        }
        Ok(())
    }

    /// 查找最近的探针
    pub fn find_nearest_probe(&self, world_pos: Vec3) -> Option<(&DDGIProbe, usize, usize)> {
        self.active_volume.and_then(|volume_idx| {
            self.volumes.get(volume_idx).and_then(|volume| {
                let mut min_dist = f32::MAX;
                let mut nearest_probe = None;
                let mut nearest_idx = 0;

                for idx in 0..volume.probe_count() {
                    if let Some(probe) = volume.get_probe(idx) {
                        let dist = probe.position.distance(world_pos);
                        if dist < min_dist {
                            min_dist = dist;
                            nearest_probe = Some(probe);
                            nearest_idx = idx;
                        }
                    }
                }

                nearest_probe.map(|probe| (probe, nearest_idx, volume_idx))
            })
        })
    }

    /// 查找探针的8个邻居（用于三线性插值）
    pub fn find_neighbor_probes(&self, world_pos: Vec3) -> Option<[Option<&DDGIProbe>; 8]> {
        self.active_volume.and_then(|volume_idx| {
            self.volumes.get(volume_idx).map(|volume| {
                let config = volume.config();
                let local_pos = world_pos - volume.volume_origin();
                let probe_spacing = config.probe_spacing;

                // 计算探针网格索引
                let base_x = (local_pos.x / probe_spacing).floor() as i32;
                let base_y = (local_pos.y / probe_spacing).floor() as i32;
                let base_z = (local_pos.z / probe_spacing).floor() as i32;

                // 获取8个角点的探针
                let mut neighbors = [None; 8];
                let mut idx = 0;

                for dz in 0..2 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let probe_x = base_x + dx;
                            let probe_y = base_y + dy;
                            let probe_z = base_z + dz;

                            // 计算探针索引
                            let probe_idx = ((probe_z * config.probe_counts.y as i32 + probe_y)
                                * config.probe_counts.x as i32
                                + probe_x) as usize;

                            neighbors[idx] = volume.get_probe(probe_idx);
                            idx += 1;
                        }
                    }
                }

                neighbors
            })
        })
    }

    /// 采样辐照度（使用三线性插值）
    pub fn sample_irradiance(&self, world_pos: Vec3, normal: Vec3) -> Vec3 {
        if let Some(neighbors) = self.find_neighbor_probes(world_pos) {
            let config = self.active_volume.and_then(|v| self.volumes.get(v)).map(|v| v.config());

            if let Some(config) = config {
                let volume = self.active_volume.and_then(|v| self.volumes.get(v));
                if let Some(volume) = volume {
                    let local_pos = world_pos - volume.volume_origin();
                    let probe_spacing = config.probe_spacing;

                    // 计算插值权重
                    let base_x = (local_pos.x / probe_spacing).floor() as f32;
                    let base_y = (local_pos.y / probe_spacing).floor() as f32;
                    let base_z = (local_pos.z / probe_spacing).floor() as f32;

                    let tx = (local_pos.x / probe_spacing - base_x).clamp(0.0, 1.0);
                    let ty = (local_pos.y / probe_spacing - base_y).clamp(0.0, 1.0);
                    let tz = (local_pos.z / probe_spacing - base_z).clamp(0.0, 1.0);

                    // 三线性插值
                    let mut result = Vec3::ZERO;
                    let mut weight_sum = 0.0;

                    for (i, probe_opt) in neighbors.iter().enumerate() {
                        if let Some(probe) = probe_opt {
                            let dx = if i & 1 != 0 { tx } else { 1.0 - tx };
                            let dy = if i & 2 != 0 { ty } else { 1.0 - ty };
                            let dz = if i & 4 != 0 { tz } else { 1.0 - tz };

                            let weight = dx * dy * dz;
                            result += probe.irradiance * weight;
                            weight_sum += weight;
                        }
                    }

                    if weight_sum > 0.0 {
                        result / weight_sum
                    } else {
                        Vec3::ZERO
                    }
                } else {
                    Vec3::ZERO
                }
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        }
    }

    /// 清空所有体积
    pub fn clear(&mut self) {
        self.volumes.clear();
        self.active_volume = None;
    }
}

impl Default for ProbeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_creation() {
        let position = Vec3::new(1.0, 2.0, 3.0);
        let probe = DDGIProbe::new(position);

        assert_eq!(probe.position, position);
        assert_eq!(probe.irradiance, Vec3::ZERO);
        assert_eq!(probe.depth, 1.0);
        assert_eq!(probe.offset, Vec2::ZERO);
    }

    #[test]
    fn test_probe_reset() {
        let mut probe = DDGIProbe::new(Vec3::ZERO);
        probe.set_irradiance(Vec3::new(0.5, 0.5, 0.5));
        probe.set_depth(0.5);
        probe.set_offset(Vec2::new(0.1, 0.2));

        probe.reset();

        assert_eq!(probe.irradiance, Vec3::ZERO);
        assert_eq!(probe.depth, 1.0);
        assert_eq!(probe.offset, Vec2::ZERO);
    }

    #[test]
    fn test_probe_manager() {
        let manager = ProbeManager::new();
        assert_eq!(manager.volume_count(), 0);
        assert!(manager.active_volume().is_none());
    }
}
