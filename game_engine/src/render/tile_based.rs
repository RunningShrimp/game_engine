//! # 移动端Tile-based渲染优化
//!
//! 为移动GPU的Tile-based渲染架构提供优化策略。
//!
//! ## 什么是Tile-based渲染?
//!
//! Tile-based渲染（TBR）将屏幕划分为小方块（tiles），每个tile独立处理：
//! - **几何阶段**: 分配三角形到tiles
//! - **渲染阶段**: 逐tile渲染到片上缓存
//! - **写入阶段**: 将tile结果写回主内存
//!
//! ## 主要厂商
//!
//! - **ARM Mali**: 使用Tile-based渲染
//! - **Qualcomm Adreno**: 使用Tile-based渲染
//! - **Apple GPU**: 使用Tile-based渲染
//! - **Intel**: 部分集成GPU使用Tile-based渲染
//!
//! ## 优化目标
//!
//! - **减少Overdraw**: TBR对overdraw敏感
//! - **优化渲染顺序**: 正确的排序至关重要
//! - **带宽优化**: 减少主内存访问
//! - **Framebuffer Fetch**: 利用片上缓存
//!
//! ## 使用场景
//!
//! - **Android设备**: Mali/Adreno GPU
//! - **iOS设备**: Apple GPU
//! - **移动游戏**: 高性能移动3D游戏
//! - **AR应用**: 移动AR/VR应用

use std::collections::HashMap;

/// Tile-based渲染配置
#[derive(Clone, Debug)]
pub struct TileBasedConfig {
    /// Tile大小（像素）
    pub tile_size: TileSize,
    /// 是否启用early-z
    pub enable_early_z: bool,
    /// 是否启用framebuffer fetch
    pub enable_framebuffer_fetch: bool,
    /// 是否启用几何排序
    pub enable_geometry_sorting: bool,
    /// 是否启用透明对象优化
    pub enable_transparency_optimization: bool,
    /// 最大overdraw阈值
    pub max_overdraw: f32,
}

impl Default for TileBasedConfig {
    fn default() -> Self {
        Self {
            tile_size: TileSize::Tile16x16,
            enable_early_z: true,
            enable_framebuffer_fetch: true,
            enable_geometry_sorting: true,
            enable_transparency_optimization: true,
            max_overdraw: 3.0,  // 允许最多3x overdraw
        }
    }
}

/// Tile大小
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileSize {
    Tile8x8 = 8,
    Tile16x16 = 16,
    Tile32x32 = 32,
}

impl TileSize {
    /// 获取tile尺寸（像素）
    pub fn size(&self) -> u32 {
        match self {
            TileSize::Tile8x8 => 8,
            TileSize::Tile16x16 => 16,
            TileSize::Tile32x32 => 32,
        }
    }

    /// 计算屏幕需要的tile数量
    pub fn calculate_tile_count(&self, screen_width: u32, screen_height: u32) -> (u32, u32) {
        let size = self.size();
        (
            (screen_width + size - 1) / size,
            (screen_height + size - 1) / size,
        )
    }
}

/// 渲染对象类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderObjectType {
    /// 不透明对象（front-to-back排序）
    Opaque,
    /// 透明对象（back-to-front排序）
    Transparent,
    /// 叠加对象（最后渲染）
    Overlay,
}

/// 渲染对象
#[derive(Clone, Debug)]
pub struct RenderObject {
    pub id: u32,
    pub object_type: RenderObjectType,
    pub depth: f32,
    pub bounds: ObjectBounds,
    pub vertex_count: u32,
    pub triangle_count: u32,
}

/// 对象边界框
#[derive(Clone, Copy, Debug)]
pub struct ObjectBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl ObjectBounds {
    /// 计算屏幕空间面积
    pub fn screen_area(&self) -> f32 {
        (self.max_x - self.min_x) * (self.max_y - self.min_y)
    }
}

/// Tile-based渲染优化器
pub struct TileBasedOptimizer {
    config: TileBasedConfig,
    render_objects: Vec<RenderObject>,
    tile_overdraw: HashMap<(u32, u32), f32>,  // (tile_x, tile_y) -> overdraw
}

impl TileBasedOptimizer {
    /// 创建新的优化器
    pub fn new(config: TileBasedConfig) -> Self {
        Self {
            config,
            render_objects: Vec::new(),
            tile_overdraw: HashMap::new(),
        }
    }

    /// 从GPU名称自动检测并创建优化器
    pub fn from_gpu_detection(gpu_name: &str) -> Self {
        let is_tbr = is_tile_based_gpu(gpu_name);
        Self::new(if is_tbr {
            TileBasedConfig::default()
        } else {
            // 非TBR GPU，禁用TBR特定优化
            TileBasedConfig {
                enable_early_z: false,
                enable_framebuffer_fetch: false,
                enable_geometry_sorting: false,
                ..Default::default()
            }
        })
    }

    /// 添加渲染对象
    pub fn add_render_object(&mut self, object: RenderObject) {
        self.render_objects.push(object);
    }

    /// 优化渲染顺序
    pub fn optimize_render_order(&mut self, screen_width: u32, screen_height: u32) -> Vec<RenderObject> {
        if !self.config.enable_geometry_sorting {
            return self.render_objects.clone();
        }

        // 分离不透明和透明对象
        let mut opaque_objects: Vec<&RenderObject> = Vec::new();
        let mut transparent_objects: Vec<&RenderObject> = Vec::new();
        let mut overlay_objects: Vec<&RenderObject> = Vec::new();

        for obj in &self.render_objects {
            match obj.object_type {
                RenderObjectType::Opaque => opaque_objects.push(obj),
                RenderObjectType::Transparent => transparent_objects.push(obj),
                RenderObjectType::Overlay => overlay_objects.push(obj),
            }
        }

        // 不透明对象：front-to-back排序（减少overdraw）
        opaque_objects.sort_by(|a, b| {
            b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 透明对象：back-to-front排序（正确混合）
        transparent_objects.sort_by(|a, b| {
            a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 合并结果
        let mut optimized = Vec::new();
        optimized.extend(opaque_objects.into_iter().cloned());
        optimized.extend(transparent_objects.into_iter().cloned());
        optimized.extend(overlay_objects.into_iter().cloned());

        optimized
    }

    /// 计算tile overdraw
    pub fn calculate_tile_overdraw(
        &mut self,
        screen_width: u32,
        screen_height: u32,
    ) -> HashMap<(u32, u32), f32> {
        let (tiles_x, tiles_y) = self.config.tile_size.calculate_tile_count(screen_width, screen_height);
        let tile_size = self.config.tile_size.size();

        // 初始化tile覆盖计数
        let mut tile_coverage: HashMap<(u32, u32), u32> = HashMap::new();

        for obj in &self.render_objects {
            // 计算对象覆盖的tiles
            let start_tile_x = (obj.bounds.min_x as u32 / tile_size).min(tiles_x - 1);
            let end_tile_x = (obj.bounds.max_x as u32 / tile_size).min(tiles_x - 1);
            let start_tile_y = (obj.bounds.min_y as u32 / tile_size).min(tiles_y - 1);
            let end_tile_y = (obj.bounds.max_y as u32 / tile_size).min(tiles_y - 1);

            // 增加覆盖计数
            for tile_x in start_tile_x..=end_tile_x {
                for tile_y in start_tile_y..=end_tile_y {
                    *tile_coverage.entry((tile_x, tile_y)).or_insert(0) += 1;
                }
            }
        }

        // 转换为overdraw比率
        self.tile_overdraw = tile_coverage
            .into_iter()
            .map(|(tile, count)| {
                let overdraw = count as f32;
                (tile, overdraw)
            })
            .collect();

        self.tile_overdraw.clone()
    }

    /// 获取tile overdraw统计
    pub fn get_tile_overdraw_stats(&self) -> TileOverdrawStats {
        if self.tile_overdraw.is_empty() {
            return TileOverdrawStats::default();
        }

        let overdraws: Vec<f32> = self.tile_overdraw.values().cloned().collect();
        let avg = overdraws.iter().sum::<f32>() / overdraws.len() as f32;
        let max = overdraws.iter().fold(0.0f32, |a, &b| a.max(b));
        let min = overdraws.iter().fold(f32::INFINITY, |a, &b| a.min(b));

        // 高overdraw tiles占比（超过max_overdraw）
        let high_overdraw_count = overdraws
            .iter()
            .filter(|&&o| o > self.config.max_overdraw)
            .count();

        TileOverdrawStats {
            average_overdraw: avg,
            max_overdraw: max,
            min_overdraw: min,
            high_overdraw_tiles: high_overdraw_count,
            total_tiles: overdraws.len(),
        }
    }

    /// 获取超过overdraw阈值的tiles
    pub fn get_high_overdraw_tiles(&self) -> Vec<(u32, u32)> {
        self.tile_overdraw
            .iter()
            .filter(|&(_, &overdraw)| overdraw > self.config.max_overdraw)
            .map(|(&tile, _)| tile)
            .collect()
    }

    /// 清空渲染对象
    pub fn clear(&mut self) {
        self.render_objects.clear();
        self.tile_overdraw.clear();
    }
}

/// Tile overdraw统计
#[derive(Clone, Copy, Debug, Default)]
pub struct TileOverdrawStats {
    pub average_overdraw: f32,
    pub max_overdraw: f32,
    pub min_overdraw: f32,
    pub high_overdraw_tiles: usize,
    pub total_tiles: usize,
}

impl TileOverdrawStats {
    /// 获取高overdraw占比
    pub fn high_overdraw_ratio(&self) -> f32 {
        if self.total_tiles == 0 {
            return 0.0;
        }
        self.high_overdraw_tiles as f32 / self.total_tiles as f32
    }
}

/// 渲染Pass优化建议
#[derive(Clone, Debug)]
pub struct RenderPassOptimization {
    /// 是否使用early-z
    pub use_early_z: bool,
    /// 是否使用framebuffer fetch
    pub use_framebuffer_fetch: bool,
    /// 推荐的清除操作
    pub clear_ops: Vec<ClearOperation>,
    /// 推荐的渲染顺序
    pub render_order: RenderOrder,
}

/// 清除操作
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearOperation {
    Color,
    Depth,
    Stencil,
    DepthStencil,
}

/// 渲染顺序
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderOrder {
    /// Front-to-back（不透明）
    FrontToBack,
    /// Back-to-front（透明）
    BackToFront,
    /// 自定义排序
    Custom,
}

/// Tile-based渲染Pass优化器
pub struct TileBasedPassOptimizer {
    config: TileBasedConfig,
}

impl TileBasedPassOptimizer {
    pub fn new(config: TileBasedConfig) -> Self {
        Self { config }
    }

    /// 优化渲染Pass
    pub fn optimize_render_pass(&self, has_transparency: bool) -> RenderPassOptimization {
        RenderPassOptimization {
            use_early_z: self.config.enable_early_z && !has_transparency,
            use_framebuffer_fetch: self.config.enable_framebuffer_fetch,
            clear_ops: self.recommend_clear_operations(),
            render_order: self.recommend_render_order(has_transparency),
        }
    }

    /// 推荐清除操作
    fn recommend_clear_operations(&self) -> Vec<ClearOperation> {
        vec![
            ClearOperation::Color,
            ClearOperation::DepthStencil,
        ]
    }

    /// 推荐渲染顺序
    fn recommend_render_order(&self, has_transparency: bool) -> RenderOrder {
        if has_transparency {
            RenderOrder::Custom  // opaque + transparent
        } else {
            RenderOrder::FrontToBack
        }
    }
}

/// 带宽优化建议
#[derive(Clone, Debug)]
pub struct BandwidthOptimizationHints {
    /// 是否启用几何压缩
    pub enable_geometry_compression: bool,
    /// 是否启用ARM Framebuffer Fetch (AFRC)
    pub enable_afbc: bool,
    /// 是否启用Adreno优化的像素格式
    pub use_adreno_optimized_formats: bool,
    /// 是否启用Mali无损压缩
    pub use_mali_lossless_compression: bool,
}

impl BandwidthOptimizationHints {
    /// 从GPU名称获取优化建议
    pub fn from_gpu_name(gpu_name: &str) -> Self {
        let gpu_lower = gpu_name.to_lowercase();

        Self {
            enable_geometry_compression: gpu_lower.contains("adreno") || gpu_lower.contains("mali"),
            enable_afbc: gpu_lower.contains("mali"),
            use_adreno_optimized_formats: gpu_lower.contains("adreno"),
            use_mali_lossless_compression: gpu_lower.contains("mali"),
        }
    }

    /// 获取推荐的纹理格式
    pub fn recommended_texture_format(&self) -> TextureFormat {
        if self.use_adreno_optimized_formats {
            TextureFormat::Astc4x4
        } else if self.use_mali_lossless_compression {
            TextureFormat::Astc4x4
        } else {
            TextureFormat::Etc2
        }
    }
}

/// 纹理格式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureFormat {
    /// ASTC 4x4 - 高质量压缩
    Astc4x4,
    /// ETC2 - 基础压缩
    Etc2,
    /// BC3 - 桌面压缩
    Bc3,
}

/// Overdraw可视化器（用于调试）
pub struct OverdrawVisualizer {
    tile_overdraw: HashMap<(u32, u32), f32>,
    max_overdraw: f32,
}

impl OverdrawVisualizer {
    /// 创建可视化器
    pub fn new(tile_overdraw: HashMap<(u32, u32), f32>) -> Self {
        let max = tile_overdraw
            .values()
            .cloned()
            .fold(0.0f32, |a, b| a.max(b));

        Self {
            tile_overdraw,
            max_overdraw: max.max(1.0),
        }
    }

    /// 获取tile颜色（用于热力图）
    pub fn get_tile_color(&self, tile: (u32, u32)) -> [f32; 3] {
        let overdraw = *self.tile_overdraw.get(&tile).unwrap_or(&0.0);
        let ratio = (overdraw / self.max_overdraw).min(1.0);

        // 绿色 = 低overdraw，红色 = 高overdraw
        [
            ratio,           // R
            1.0 - ratio,     // G
            0.0,             // B
        ]
    }

    /// 获取最大overdraw
    pub fn max_overdraw(&self) -> f32 {
        self.max_overdraw
    }
}

// =============================================================================
// 辅助函数
// =============================================================================

/// 检测是否为Tile-based GPU
pub fn is_tile_based_gpu(gpu_name: &str) -> bool {
    let gpu_lower = gpu_name.to_lowercase();

    // ARM Mali
    if gpu_lower.contains("mali") {
        return true;
    }

    // Qualcomm Adreno
    if gpu_lower.contains("adreno") {
        return true;
    }

    // Apple GPU
    if gpu_lower.contains("apple gpu") || gpu_lower.contains("apple m") {
        return true;
    }

    // Intel集成（部分使用Tile-based）
    if gpu_lower.contains("intel") &&
       (gpu_lower.contains("hd graphics") || gpu_lower.contains("uhd")) {
        return true;
    }

    false
}

/// 获取推荐的Tile大小
pub fn recommended_tile_size(gpu_name: &str) -> TileSize {
    let gpu_lower = gpu_name.to_lowercase();

    // 高端GPU使用更小的tiles
    if gpu_lower.contains("mali-g") || gpu_lower.contains("adreno 6") || gpu_lower.contains("apple m") {
        TileSize::Tile16x16
    } else {
        TileSize::Tile32x32
    }
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_detection() {
        assert!(is_tile_based_gpu("Mali-G78"));
        assert!(is_tile_based_gpu("Adreno 650"));
        assert!(is_tile_based_gpu("Apple M1 GPU"));
        assert!(is_tile_based_gpu("Intel UHD Graphics"));
        assert!(!is_tile_based_gpu("NVIDIA GeForce RTX 3080"));
    }

    #[test]
    fn test_tile_size_calculation() {
        let tile_size = TileSize::Tile16x16;
        assert_eq!(tile_size.size(), 16);

        let (tiles_x, tiles_y) = tile_size.calculate_tile_count(1920, 1080);
        assert_eq!(tiles_x, 120);  // 1920 / 16
        assert_eq!(tiles_y, 68);   // 1080 / 16, 向上取整
    }

    #[test]
    fn test_object_bounds() {
        let bounds = ObjectBounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 100.0,
            max_y: 100.0,
        };

        assert_eq!(bounds.screen_area(), 10000.0);
    }

    #[test]
    fn test_render_order_optimization() {
        let mut optimizer = TileBasedOptimizer::new(TileBasedConfig::default());

        // 添加渲染对象
        optimizer.add_render_object(RenderObject {
            id: 1,
            object_type: RenderObjectType::Opaque,
            depth: 10.0,
            bounds: ObjectBounds { min_x: 0.0, min_y: 0.0, max_x: 100.0, max_y: 100.0 },
            vertex_count: 100,
            triangle_count: 50,
        });

        optimizer.add_render_object(RenderObject {
            id: 2,
            object_type: RenderObjectType::Opaque,
            depth: 5.0,  // 更近
            bounds: ObjectBounds { min_x: 0.0, min_y: 0.0, max_x: 100.0, max_y: 100.0 },
            vertex_count: 100,
            triangle_count: 50,
        });

        let optimized = optimizer.optimize_render_order(1920, 1080);

        // 验证front-to-back排序（depth 10在depth 5之前）
        assert_eq!(optimized[0].id, 1);  // depth 10
        assert_eq!(optimized[1].id, 2);  // depth 5
    }

    #[test]
    fn test_overdraw_stats() {
        let mut tile_overdraw = HashMap::new();
        tile_overdraw.insert((0, 0), 2.0);
        tile_overdraw.insert((1, 0), 4.0);
        tile_overdraw.insert((0, 1), 1.0);

        let visualizer = OverdrawVisualizer::new(tile_overdraw);
        assert_eq!(visualizer.max_overdraw(), 4.0);

        // 高overdraw tile应该是红色
        let color = visualizer.get_tile_color((1, 0));
        assert!(color[0] > 0.9);  // R接近1.0
        assert!(color[1] < 0.1);  // G接近0.0
    }

    #[test]
    fn test_bandwidth_optimization() {
        let hints = BandwidthOptimizationHints::from_gpu_name("Mali-G78");
        assert!(hints.enable_afbc);
        assert!(hints.use_mali_lossless_compression);
    }

    #[test]
    fn test_overdraw_stats_ratio() {
        let mut stats = TileOverdrawStats::default();
        stats.total_tiles = 100;
        stats.high_overdraw_tiles = 10;

        assert_eq!(stats.high_overdraw_ratio(), 0.1);
    }
}
