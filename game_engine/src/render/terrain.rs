//! 地形渲染系统
//!
//! 实现地形生成、LOD、纹理混合和渲染功能。
//!
//! ## 功能特性
//!
//! - **高度图生成**: 基于噪声函数生成高度图
//! - **网格生成**: 从高度图生成地形网格
//! - **LOD系统**: 基于距离和屏幕空间的LOD
//! - **纹理混合**: 支持多纹理混合（草地、石头、沙子等）
//! - **法线贴图**: 支持法线贴图增强细节

use crate::core::error::RenderError;
use crate::render::lod::{LodQuality, LodSelector};
use crate::render::mesh::GpuMesh;
use glam::{Vec2, Vec3};
use std::sync::Arc;

/// 地形LOD级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainLodLevel {
    /// 最高质量（原始分辨率）
    Level0,
    /// 高质量（1/2分辨率）
    Level1,
    /// 中等质量（1/4分辨率）
    Level2,
    /// 低质量（1/8分辨率）
    Level3,
    /// 极低质量（1/16分辨率）
    Level4,
}

impl TerrainLodLevel {
    /// 获取分辨率缩放因子
    pub fn resolution_scale(&self) -> usize {
        match self {
            TerrainLodLevel::Level0 => 1,
            TerrainLodLevel::Level1 => 2,
            TerrainLodLevel::Level2 => 4,
            TerrainLodLevel::Level3 => 8,
            TerrainLodLevel::Level4 => 16,
        }
    }

    /// 获取顶点数（相对于原始分辨率）
    pub fn vertex_count_ratio(&self) -> f32 {
        1.0 / (self.resolution_scale() * self.resolution_scale()) as f32
    }
}

/// 地形纹理层
#[derive(Debug, Clone)]
pub struct TerrainTextureLayer {
    /// 纹理ID
    pub texture_id: u64,
    /// 纹理缩放
    pub scale: Vec2,
    /// 混合权重
    pub weight: f32,
    /// 法线贴图ID（可选）
    pub normal_map_id: Option<u64>,
}

/// 地形数据
#[derive(Clone, Debug)]
pub struct TerrainData {
    /// 地形宽度（顶点数）
    pub width: usize,
    /// 地形高度（顶点数）
    pub height: usize,
    /// 高度图数据
    pub heightmap: Vec<f32>,
    /// 地形缩放
    pub scale: Vec3,
    /// 纹理层
    pub texture_layers: Vec<TerrainTextureLayer>,
}

impl TerrainData {
    /// 创建新的地形数据
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            heightmap: vec![0.0; width * height],
            scale: Vec3::new(1.0, 1.0, 1.0),
            texture_layers: Vec::new(),
        }
    }

    /// 获取指定位置的高度
    pub fn get_height(&self, x: usize, y: usize) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.heightmap.get(index).copied()
    }

    /// 设置指定位置的高度
    pub fn set_height(&mut self, x: usize, y: usize, height: f32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let index = y * self.width + x;
        if let Some(h) = self.heightmap.get_mut(index) {
            *h = height;
            true
        } else {
            false
        }
    }

    /// 获取插值高度（支持浮点坐标）
    pub fn get_height_interpolated(&self, x: f32, y: f32) -> f32 {
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x.ceil() as usize).min(self.width - 1);
        let y1 = (y.ceil() as usize).min(self.height - 1);

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let h00 = self.get_height(x0, y0).unwrap_or(0.0);
        let h10 = self.get_height(x1, y0).unwrap_or(0.0);
        let h01 = self.get_height(x0, y1).unwrap_or(0.0);
        let h11 = self.get_height(x1, y1).unwrap_or(0.0);

        // 双线性插值
        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;
        h0 * (1.0 - fy) + h1 * fy
    }

    /// 生成地形网格（指定LOD级别）
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `lod_level` - LOD级别
    ///
    /// # 返回
    ///
    /// 返回生成的地形网格。
    pub fn generate_mesh(
        &self,
        device: &wgpu::Device,
        lod_level: TerrainLodLevel,
    ) -> Result<GpuMesh, RenderError> {
        use crate::render::mesh::Vertex3D;

        let scale = lod_level.resolution_scale();
        let lod_width = (self.width + scale - 1) / scale;
        let lod_height = (self.height + scale - 1) / scale;

        let mut vertices = Vec::with_capacity(lod_width * lod_height);
        let mut indices = Vec::new();

        // 生成顶点
        for y in 0..lod_height {
            for x in 0..lod_width {
                let world_x = (x * scale) as f32 / self.width as f32;
                let world_z = (y * scale) as f32 / self.height as f32;

                let height = self.get_height_interpolated(
                    (x * scale) as f32,
                    (y * scale) as f32,
                );

                let position = Vec3::new(
                    world_x * self.scale.x,
                    height * self.scale.y,
                    world_z * self.scale.z,
                );

                // 计算法线
                let normal = self.calculate_normal(x * scale, y * scale);

                // 计算UV坐标
                let uv = Vec2::new(world_x, world_z);

                // 计算切线（简化：使用默认值）
                let tangent = [1.0, 0.0, 0.0, 1.0];

                vertices.push(Vertex3D {
                    pos: position.to_array(),
                    normal: normal.to_array(),
                    uv: uv.to_array(),
                    tangent,
                });
            }
        }

        // 生成索引（三角形）
        for y in 0..(lod_height - 1) {
            for x in 0..(lod_width - 1) {
                let i0 = (y * lod_width + x) as u32;
                let i1 = ((y + 1) * lod_width + x) as u32;
                let i2 = (y * lod_width + x + 1) as u32;
                let i3 = ((y + 1) * lod_width + x + 1) as u32;

                // 第一个三角形
                indices.push(i0);
                indices.push(i1);
                indices.push(i2);

                // 第二个三角形
                indices.push(i2);
                indices.push(i1);
                indices.push(i3);
            }
        }

        // 创建GpuMesh
        Ok(GpuMesh::new(device, &vertices, &indices))
    }

    /// 计算法线（简化版本）
    fn calculate_normal(&self, x: usize, y: usize) -> Vec3 {
        let h_left = self.get_height(x.saturating_sub(1), y).unwrap_or(0.0);
        let h_right = self.get_height((x + 1).min(self.width - 1), y).unwrap_or(0.0);
        let h_up = self.get_height(x, y.saturating_sub(1)).unwrap_or(0.0);
        let h_down = self.get_height(x, (y + 1).min(self.height - 1)).unwrap_or(0.0);

        let dx = (h_right - h_left) * self.scale.y;
        let dy = (h_down - h_up) * self.scale.y;

        Vec3::new(-dx, 2.0 * self.scale.x, -dy).normalize()
    }
}

/// 地形块（用于LOD）
#[derive(Debug, Clone)]
pub struct TerrainChunk {
    /// 块位置（世界坐标）
    pub position: Vec3,
    /// 块大小
    pub size: f32,
    /// LOD级别
    pub lod_level: TerrainLodLevel,
    /// 网格（可选，按需加载）
    pub mesh: Option<Arc<GpuMesh>>,
    /// 是否可见
    pub visible: bool,
}

/// 地形渲染器
pub struct TerrainRenderer {
    /// 地形数据
    terrain_data: TerrainData,
    /// 地形块（用于LOD）
    chunks: Vec<TerrainChunk>,
    /// LOD选择器
    lod_selector: Option<LodSelector>,
    /// 块大小
    chunk_size: f32,
}

impl TerrainRenderer {
    /// 创建新的地形渲染器
    pub fn new(terrain_data: TerrainData, chunk_size: f32) -> Self {
        Self {
            terrain_data,
            chunks: Vec::new(),
            lod_selector: None,
            chunk_size,
        }
    }

    /// 设置LOD选择器
    pub fn set_lod_selector(&mut self, selector: LodSelector) {
        self.lod_selector = Some(selector);
    }

    /// 更新地形块LOD
    pub fn update_lod(&mut self, camera_pos: Vec3, delta_time: f32) -> Result<(), RenderError> {
        if let Some(ref mut lod_selector) = self.lod_selector {
            for chunk in &mut self.chunks {
                let distance = (chunk.position - camera_pos).length();
                let selection = lod_selector.select_stateless(
                    distance,
                    chunk.size,
                    &glam::Mat4::IDENTITY, // 简化：使用单位矩阵
                );

                // 根据LOD选择结果更新块级别
                chunk.lod_level = match selection.quality {
                    crate::render::lod::LodQuality::High => TerrainLodLevel::Level0,
                    crate::render::lod::LodQuality::Medium => TerrainLodLevel::Level1,
                    crate::render::lod::LodQuality::Low => TerrainLodLevel::Level2,
                    crate::render::lod::LodQuality::VeryLow => TerrainLodLevel::Level3,
                    crate::render::lod::LodQuality::Culled => {
                        chunk.visible = false;
                        continue;
                    }
                };
                chunk.visible = true;
            }
        }
        Ok(())
    }

    /// 获取可见的地形块
    pub fn visible_chunks(&self) -> impl Iterator<Item = &TerrainChunk> {
        self.chunks.iter().filter(|chunk| chunk.visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_lod_level() {
        assert_eq!(TerrainLodLevel::Level0.resolution_scale(), 1);
        assert_eq!(TerrainLodLevel::Level1.resolution_scale(), 2);
        assert_eq!(TerrainLodLevel::Level2.resolution_scale(), 4);
    }

    #[test]
    fn test_terrain_data() {
        let terrain = TerrainData::new(10, 10);
        assert_eq!(terrain.width, 10);
        assert_eq!(terrain.height, 10);
        assert_eq!(terrain.heightmap.len(), 100);
    }

    #[test]
    fn test_height_interpolation() {
        let mut terrain = TerrainData::new(10, 10);
        assert!(terrain.set_height(5, 5, 10.0));
        let height = terrain.get_height_interpolated(5.0, 5.0);
        assert!((height - 10.0).abs() < 0.1);
    }
}

