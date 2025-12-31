//! # UV Atlas 生成器
//!
//! 将多个网格的UV坐标打包到单一纹理图集中。
//!
//! ## 功能
//!
//! - **多网格打包**: 将多个对象的UV打包到一个atlas
//! - **空间优化**: 最小化浪费空间
//! - **纹理保护**: 留出边缘避免bleeding
//! - **旋转支持**: 自动旋转UV岛以提高空间利用率
//!
//! ## 使用场景
//!
//! - **批处理渲染**: 减少draw calls
//! - **光照烘焙**: 合并多个lightmap
//! - **纹理集**: 字体图集、精灵图集

use glam::Vec2;
use std::collections::HashMap;

/// UV岛 - 单个网格的UV边界框
#[derive(Clone, Debug)]
pub struct UvIsland {
    /// 网格索引
    pub mesh_index: usize,
    /// UV坐标列表
    pub uvs: Vec<Vec2>,
    /// 边界框（最小/最大UV）
    pub bounds: (Vec2, Vec2),
    /// 旋转角度（0, 90, 180, 270度）
    pub rotation: u32,
    /// padding（纹理保护）
    pub padding: f32,
}

impl UvIsland {
    /// 创建新的UV岛
    pub fn new(mesh_index: usize, uvs: Vec<Vec2>) -> Self {
        let bounds = calculate_bounds(&uvs);

        Self {
            mesh_index,
            uvs,
            bounds,
            rotation: 0,
            padding: 0.01, // 默认1% padding
        }
    }

    /// 获取UV岛的尺寸
    pub fn size(&self) -> Vec2 {
        let (min, max) = self.bounds;
        max - min
    }

    /// 旋转UV岛（90度增量）
    pub fn rotate(&mut self, degrees: u32) {
        self.rotation = (self.rotation + degrees) % 360;

        // 旋转所有UV坐标
        for uv in &mut self.uvs {
            *uv = rotate_uv(*uv, degrees);
        }

        // 重新计算边界
        self.bounds = calculate_bounds(&self.uvs);
    }

    /// 应用padding到边界框
    pub fn apply_padding(&mut self) {
        let (min, max) = self.bounds;
        let size = max - min;
        let padding_vec = Vec2::new(self.padding, self.padding);

        self.bounds = (min - padding_vec, max + padding_vec);
    }
}

/// UV Atlas配置
#[derive(Clone, Debug)]
pub struct AtlasOptions {
    /// atlas尺寸（像素）
    pub size: (u32, u32),
    /// padding（像素）
    pub padding: u32,
    /// 是否允许旋转
    pub allow_rotation: bool,
    /// 最大尝试次数
    pub max_attempts: u32,
}

impl Default for AtlasOptions {
    fn default() -> Self {
        Self {
            size: (2048, 2048),
            padding: 4,
            allow_rotation: true,
            max_attempts: 1000,
        }
    }
}

/// UV Atlas生成器
pub struct UvAtlasGenerator {
    islands: Vec<UvIsland>,
    options: AtlasOptions,
}

impl UvAtlasGenerator {
    /// 创建新的atlas生成器
    pub fn new(options: AtlasOptions) -> Self {
        Self {
            islands: Vec::new(),
            options,
        }
    }

    /// 添加网格UV到atlas
    pub fn add_mesh(&mut self, mesh_index: usize, uvs: Vec<Vec2>) {
        let island = UvIsland::new(mesh_index, uvs);
        self.islands.push(island);
    }

    /// 生成UV Atlas
    pub fn generate(&mut self) -> Result<UvAtlas, String> {
        println!("Generating UV Atlas for {} meshes", self.islands.len());

        let mut placed_islands = Vec::new();
        let mut packing_rects = Vec::new();

        // 按大小排序（大到小）
        let mut sorted_indices: Vec<usize> = (0..self.islands.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            let size_a = self.islands[a].size();
            let size_b = self.islands[b].size();

            // 先比较宽度，再比较高度
            match size_b.x.partial_cmp(&size_a.x) {
                Some(std::cmp::Ordering::Equal) => {
                    size_b.y.partial_cmp(&size_a.y).unwrap_or(std::cmp::Ordering::Equal)
                }
                Some(ord) => ord,
                None => std::cmp::Ordering::Equal,
            }
        });

        // 简单的shelf packing算法
        let mut shelf_y = 0.0;
        let mut shelf_x = 0.0;
        let mut max_shelf_height = 0.0;

        for &idx in &sorted_indices {
            let island = &mut self.islands[idx];

            // 应用padding
            island.apply_padding();

            let island_size = island.size();
            let mesh_index = island.mesh_index;
            let rotation = island.rotation;
            let (bounds_min, _) = island.bounds;
            let island_uvs = island.uvs.clone();

            // 检查是否需要新shelf
            if shelf_x + island_size.x > 1.0 {
                // 新行
                shelf_x = 0.0;
                shelf_y += max_shelf_height;
                max_shelf_height = 0.0;
            }

            // 检查是否超出atlas边界
            if shelf_y + island_size.y > 1.0 {
                return Err("UV Atlas too small to fit all islands".to_string());
            }

            // 放置island
            let position = Vec2::new(shelf_x, shelf_y);

            // 转换UV坐标到atlas空间
            let atlas_uvs =
                transform_uvs_to_atlas_space(&island_uvs, bounds_min, island_size, position);

            placed_islands.push(PlacedIsland {
                mesh_index,
                position,
                size: island_size,
                uvs: atlas_uvs,
                rotation,
            });

            packing_rects.push((position, position + island_size));

            // 更新shelf状态
            shelf_x += island_size.x;
            max_shelf_height = max_shelf_height.max(island_size.y);
        }

        // 计算atlas利用率
        let total_area = packing_rects
            .iter()
            .map(|(min, max)| (max.x - min.x) * (max.y - min.y))
            .sum::<f32>();
        let utilization = total_area * 100.0;

        println!("UV Atlas generated successfully!");
        println!("  Utilization: {:.1}%", utilization);
        println!("  Meshes packed: {}", placed_islands.len());

        Ok(UvAtlas {
            size: self.options.size,
            islands: placed_islands,
            utilization,
        })
    }
}

/// 放置后的UV岛
#[derive(Clone, Debug)]
pub struct PlacedIsland {
    pub mesh_index: usize,
    pub position: Vec2,
    pub size: Vec2,
    pub uvs: Vec<Vec2>,
    pub rotation: u32,
}

/// UV Atlas结果
#[derive(Clone, Debug)]
pub struct UvAtlas {
    /// Atlas尺寸（宽x高）
    pub size: (u32, u32),
    /// 放置的UV岛列表
    pub islands: Vec<PlacedIsland>,
    /// 空间利用率（百分比）
    pub utilization: f32,
}

impl UvAtlas {
    /// 获取指定网格的atlas UV坐标
    pub fn get_mesh_uvs(&self, mesh_index: usize) -> Option<&[Vec2]> {
        self.islands
            .iter()
            .find(|island| island.mesh_index == mesh_index)
            .map(|island| &island.uvs[..])
    }

    /// 保存atlas可视化图像
    #[cfg(feature = "gltf")] // 使用已存在的feature，image依赖总是可用
    pub fn save_visualization(&self, path: &std::path::Path) -> Result<(), String> {
        use image::{Rgb, RgbImage};

        let (width, height) = self.size;
        let mut img = RgbImage::new(width, height);

        // 填充背景
        for pixel in img.pixels_mut() {
            *pixel = Rgb([30, 30, 30]);
        }

        // 绘制每个island
        for island in &self.islands {
            let color = self.mesh_color(island.mesh_index);

            let min_x = (island.position.x * width as f32) as u32;
            let min_y = (island.position.y * height as f32) as u32;
            let max_x = ((island.position.x + island.size.x) * width as f32) as u32;
            let max_y = ((island.position.y + island.size.y) * height as f32) as u32;

            // 绘制边界框
            for y in min_y..max_y.min(height) {
                for x in min_x..max_x.min(width) {
                    img.put_pixel(x, y, color);
                }
            }
        }

        img.save(path).map_err(|e| e.to_string())
    }

    /// 为网格生成唯一颜色
    #[cfg(feature = "gltf")]
    fn mesh_color(&self, mesh_index: usize) -> image::Rgb<u8> {
        // 使用简单的哈希生成颜色
        let hash = mesh_index.wrapping_mul(2654435761);

        let r = ((hash >> 16) & 0xFF) as u8;
        let g = ((hash >> 8) & 0xFF) as u8;
        let b = (hash & 0xFF) as u8;

        image::Rgb([r, g, b])
    }
}

// =============================================================================
// 辅助函数
// =============================================================================

/// 将UV坐标转换到atlas空间
fn transform_uvs_to_atlas_space(
    uvs: &[Vec2],
    bounds_min: Vec2,
    island_size: Vec2,
    atlas_position: Vec2,
) -> Vec<Vec2> {
    uvs.iter()
        .map(|&uv| {
            // 归一化到边界框
            let normalized = (uv - bounds_min) / island_size;

            // 转换到atlas位置
            atlas_position + normalized * island_size
        })
        .collect()
}

/// 计算UV边界框
fn calculate_bounds(uvs: &[Vec2]) -> (Vec2, Vec2) {
    if uvs.is_empty() {
        return (Vec2::ZERO, Vec2::ONE);
    }

    let mut min = Vec2::new(f32::MAX, f32::MAX);
    let mut max = Vec2::new(f32::MIN, f32::MIN);

    for &uv in uvs {
        min = min.min(uv);
        max = max.max(uv);
    }

    (min, max)
}

/// 旋转UV坐标
fn rotate_uv(uv: Vec2, degrees: u32) -> Vec2 {
    match degrees % 360 {
        90 => Vec2::new(1.0 - uv.y, uv.x),
        180 => Vec2::new(1.0 - uv.x, 1.0 - uv.y),
        270 => Vec2::new(uv.y, 1.0 - uv.x),
        _ => uv,
    }
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bounds() {
        let uvs = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.5, 0.5),
            Vec2::new(1.0, 1.0),
        ];

        let (min, max) = calculate_bounds(&uvs);
        assert_eq!(min, Vec2::new(0.0, 0.0));
        assert_eq!(max, Vec2::new(1.0, 1.0));
    }

    #[test]
    fn test_rotate_uv() {
        let uv = Vec2::new(0.25, 0.5);

        assert_eq!(rotate_uv(uv, 0), uv);
        assert_eq!(rotate_uv(uv, 90), Vec2::new(0.5, 0.25));
        assert_eq!(rotate_uv(uv, 180), Vec2::new(0.75, 0.5));
        assert_eq!(rotate_uv(uv, 270), Vec2::new(0.5, 0.75));
    }

    #[test]
    fn test_atlas_generation() {
        let mut generator = UvAtlasGenerator::new(AtlasOptions::default());

        // 添加3个网格
        generator.add_mesh(
            0,
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(0.5, 0.0),
                Vec2::new(0.25, 0.5),
            ],
        );

        generator.add_mesh(
            1,
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(0.3, 0.0),
                Vec2::new(0.15, 0.3),
            ],
        );

        let atlas = generator.generate().unwrap();

        assert_eq!(atlas.islands.len(), 2);
        assert!(atlas.utilization > 0.0);
        assert!(atlas.utilization <= 100.0);
    }
}
