//  Decal（贴花）渲染系统
//
//  提供高效的对象表面贴花功能：
//  - 投影式Decal
//  - 延迟渲染集成
//  - 法线混合
//  - 距离剔除
//
//  ## 应用场景
//
//  - 弹孔、爆炸痕迹
//  - 血迹、污渍
//  - 墙壁涂鸦
//  - 地面脚印
//
//  ## 渲染技术
//
//  1. **投影渲染**
//     - 使用正交投影将贴花投影到表面
//     - 深度测试避免不必要渲染
//
//  2. **延迟渲染集成**
//     - 使用G-buffer数据
//     - 精确的法线混合
//
//  3. **法线混合**
//     - 正确处理表面法线
//     - 保持光照一致性

use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

/// Decal配置
#[derive(Debug, Clone)]
pub struct DecalConfig {
    /// Decal大小
    pub size: Vec3,
    /// 最大数量
    pub max_decals: usize,
    /// 是否使用法线贴图
    pub use_normal_map: bool,
    /// 法线混合强度
    pub normal_strength: f32,
    /// 是否投射阴影
    pub cast_shadows: bool,
    /// 剔除距离
    pub cull_distance: f32,
}

impl Default for DecalConfig {
    fn default() -> Self {
        Self {
            size: Vec3::new(1.0, 1.0, 1.0),
            max_decals: 100,
            use_normal_map: true,
            normal_strength: 1.0,
            cast_shadows: false,
            cull_distance: 50.0,
        }
    }
}

/// Decal类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecalType {
    /// 弹孔
    BulletHole,
    /// 爆炸痕迹
    Explosion,
    /// 血迹
    Blood,
    /// 污渍
    Stain,
    /// 涂鸦
    Graffiti,
    /// 脚印
    Footprint,
    /// 自定义
    Custom,
}

/// Decal数据
#[derive(Debug, Clone)]
pub struct Decal {
    /// Decal类型
    pub decal_type: DecalType,
    /// 世界空间位置
    pub position: Vec3,
    /// 旋转（四元数）
    pub rotation: Quat,
    /// 大小
    pub size: Vec3,
    /// 颜色
    pub color: Vec4,
    /// 法线强度
    pub normal_strength: f32,
    /// 法线贴图（如果有）
    pub normal_map: Option<String>,
    /// 反照率贴图
    pub albedo_map: Option<String>,
    /// 粗糙度
    pub roughness: f32,
    /// 金属度
    pub metallic: f32,
    /// 生命周期（秒，0表示永久）
    pub lifetime: f32,
    /// 已存活时间
    pub age: f32,
    /// 是否激活
    pub active: bool,
}

impl Decal {
    /// 创建新的Decal
    pub fn new(decal_type: DecalType, position: Vec3, rotation: Quat, size: Vec3) -> Self {
        Self {
            decal_type,
            position,
            rotation,
            size,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            normal_strength: 1.0,
            normal_map: None,
            albedo_map: None,
            roughness: 0.5,
            metallic: 0.0,
            lifetime: 0.0, // 永久
            age: 0.0,
            active: true,
        }
    }

    /// 使用默认旋转创建
    pub fn at_position(decal_type: DecalType, position: Vec3) -> Self {
        Self::new(decal_type, position, Quat::IDENTITY, Vec3::ONE)
    }

    /// 设置颜色
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// 设置大小
    pub fn with_size(mut self, size: Vec3) -> Self {
        self.size = size;
        self
    }

    /// 设置生命周期
    pub fn with_lifetime(mut self, lifetime: f32) -> Self {
        self.lifetime = lifetime;
        self
    }

    /// 设置贴图
    pub fn with_textures(mut self, albedo: Option<String>, normal: Option<String>) -> Self {
        self.albedo_map = albedo;
        self.normal_map = normal;
        self
    }

    /// 更新Decal（返回是否应该移除）
    pub fn update(&mut self, dt: f32) -> bool {
        if !self.active {
            return true; // 不活跃，移除
        }

        if self.lifetime > 0.0 {
            self.age += dt;
            if self.age >= self.lifetime {
                return true; // 超过生命周期，移除
            }
        }

        false // 继续保留
    }

    /// 检查是否应该移除（不修改状态）
    pub fn should_remove(&self) -> bool {
        if !self.active {
            return true;
        }

        if self.lifetime > 0.0 && self.age >= self.lifetime {
            return true;
        }

        false
    }

    /// 计算世界变换矩阵
    pub fn world_matrix(&self) -> Mat4 {
        let scale = Mat4::from_scale(self.size);
        let rotation = Mat4::from_quat(self.rotation);
        let translation = Mat4::from_translation(self.position);

        translation * rotation * scale
    }

    /// 计算投影矩阵（用于投影到表面）
    pub fn projection_matrix(&self) -> Mat4 {
        // 正交投影，投影到decal的局部空间
        // glam::Mat4 doesn't have orthographic(), so we construct it manually
        let left = -0.5;
        let right = 0.5;
        let bottom = -0.5;
        let top = 0.5;
        let near = -0.5;
        let far = 0.5;

        Mat4::from_cols(
            glam::Vec4::new(2.0 / (right - left), 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, 2.0 / (top - bottom), 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, -2.0 / (far - near), 0.0),
            glam::Vec4::new(
                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                -(far + near) / (far - near),
                1.0,
            ),
        )
    }
}

/// Decal管理器
pub struct DecalManager {
    /// Decal列表
    decals: Vec<Decal>,
    /// 配置
    config: DecalConfig,
    /// 统计信息
    stats: DecalStats,
}

/// Decal统计信息
#[derive(Debug, Clone, Default)]
pub struct DecalStats {
    /// 当前Decal数量
    pub active_count: usize,
    /// 总创建数
    pub total_created: u64,
    /// 总移除数
    pub total_removed: u64,
    /// 渲染的Decal数（每帧）
    pub rendered_last_frame: usize,
}

impl DecalManager {
    /// 创建新的Decal管理器
    pub fn new(config: DecalConfig) -> Self {
        Self {
            decals: Vec::with_capacity(config.max_decals),
            config,
            stats: DecalStats::default(),
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(DecalConfig::default())
    }

    /// 添加Decal
    pub fn add_decal(&mut self, decal: Decal) {
        // 如果达到上限，移除最旧的decal
        if self.decals.len() >= self.config.max_decals {
            self.decals.remove(0);
            self.stats.total_removed += 1;
        }

        self.decals.push(decal);
        self.stats.total_created += 1;
        self.stats.active_count = self.decals.len();
    }

    /// 在位置添加Decal
    pub fn spawn_decal(&mut self, decal_type: DecalType, position: Vec3, normal: Vec3) {
        // 根据法线计算旋转
        let up = Vec3::Y;
        let rotation = if normal.dot(up) > 0.95 {
            // 接近向上，直接使用
            Quat::IDENTITY
        } else {
            // 对齐到法线
            Quat::from_rotation_arc(up, normal)
        };

        let decal = Decal::new(decal_type, position, rotation, Vec3::ONE);
        self.add_decal(decal);
    }

    /// 移除Decal
    pub fn remove_decal(&mut self, index: usize) {
        if index < self.decals.len() {
            self.decals.remove(index);
            self.stats.total_removed += 1;
            self.stats.active_count = self.decals.len();
        }
    }

    /// 清空所有Decal
    pub fn clear(&mut self) {
        self.decals.clear();
        self.stats.active_count = 0;
    }

    /// 更新所有Decal
    pub fn update(&mut self, dt: f32, camera_position: Vec3) {
        // 更新生命周期并标记需要移除的decal
        for decal in &mut self.decals {
            decal.update(dt);
        }

        // 移除应该移除的decal
        self.decals.retain(|decal| !decal.should_remove());

        // 距离剔除
        if self.config.cull_distance > 0.0 {
            let cull_distance_sq = self.config.cull_distance * self.config.cull_distance;
            self.decals.retain(|decal| {
                let dist_sq = decal.position.distance_squared(camera_position);
                dist_sq < cull_distance_sq
            });
        }

        self.stats.active_count = self.decals.len();
    }

    /// 获取可见Decal（用于渲染）
    pub fn get_visible_decals(&self, camera_position: Vec3) -> Vec<&Decal> {
        if self.config.cull_distance > 0.0 {
            let cull_distance_sq = self.config.cull_distance * self.config.cull_distance;

            self.decals
                .iter()
                .filter(|decal| {
                    decal.active
                        && decal.position.distance_squared(camera_position) < cull_distance_sq
                })
                .collect()
        } else {
            self.decals.iter().filter(|decal| decal.active).collect()
        }
    }

    /// 获取所有Decal
    pub fn decals(&self) -> &[Decal] {
        &self.decals
    }

    /// 获取可变Decal
    pub fn decals_mut(&mut self) -> &mut [Decal] {
        &mut self.decals
    }

    /// 获取统计信息
    pub fn stats(&self) -> &DecalStats {
        &self.stats
    }

    /// 获取配置
    pub fn config(&self) -> &DecalConfig {
        &self.config
    }

    /// 设置配置
    pub fn set_config(&mut self, config: DecalConfig) {
        self.config = config;
    }
}

/// Decal批量渲染器
///
/// 批量渲染多个decal，优化draw call
pub struct DecalBatchRenderer {
    /// 最大批次大小
    max_batch_size: usize,
}

impl DecalBatchRenderer {
    /// 创建新的批量渲染器
    pub fn new(max_batch_size: usize) -> Self {
        Self { max_batch_size }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(100)
    }

    /// 批量渲染decals
    pub fn render_batch(&self, decals: &[&Decal]) -> Vec<DecalBatch> {
        let mut batches = Vec::new();

        // 按类型分组
        let mut grouped: std::collections::HashMap<DecalType, Vec<Decal>> =
            std::collections::HashMap::new();
        for &decal in decals {
            grouped.entry(decal.decal_type).or_default().push(decal.clone());
        }

        // 为每组创建批次
        for (decal_type, group) in grouped {
            for chunk in group.chunks(self.max_batch_size) {
                batches.push(DecalBatch {
                    decal_type,
                    decals: chunk.to_vec(),
                });
            }
        }

        batches
    }
}

/// Decal批次
#[derive(Debug, Clone)]
pub struct DecalBatch {
    /// Decal类型
    pub decal_type: DecalType,
    /// Decal列表
    pub decals: Vec<Decal>,
}

/// Decal投影算法
pub struct DecalProjector;

impl DecalProjector {
    /// 将decal投影到表面
    ///
    /// # Arguments
    /// * `decal` - Decal数据
    /// * `surface_position` - 表面位置
    /// * `surface_normal` - 表面法线
    ///
    /// # Returns
    /// 投影后的UV坐标和权重
    pub fn project_to_surface(
        decal: &Decal,
        surface_position: Vec3,
        surface_normal: Vec3,
    ) -> Option<(Vec2, f32)> {
        // 将表面位置转换到decal局部空间
        let inv_matrix = decal.world_matrix().inverse();
        let local_pos = inv_matrix.transform_point3(surface_position);

        // 检查是否在decal范围内
        if local_pos.x < -0.5
            || local_pos.x > 0.5
            || local_pos.y < -0.5
            || local_pos.y > 0.5
            || local_pos.z < -0.5
            || local_pos.z > 0.5
        {
            return None;
        }

        // 计算UV坐标
        let uv = Vec2::new(local_pos.x + 0.5, local_pos.y + 0.5);

        // 计算权重（基于距离和法线对齐）
        let distance = local_pos.z.abs();
        let normal_alignment = surface_normal.dot(Vec3::Z);
        let weight = (1.0 - distance * 2.0) * normal_alignment.max(0.0);

        Some((uv, weight))
    }

    /// 批量投影多个decal
    pub fn project_batch(
        decals: &[Decal],
        surface_position: Vec3,
        surface_normal: Vec3,
    ) -> Vec<(usize, Vec2, f32)> {
        decals
            .iter()
            .enumerate()
            .filter_map(|(i, decal)| {
                Self::project_to_surface(decal, surface_position, surface_normal)
                    .map(|(uv, weight)| (i, uv, weight))
            })
            .collect()
    }
}

/// Decal池
///
/// 重用decal对象以减少分配
pub struct DecalPool {
    /// 空闲decal
    free_decals: Vec<Decal>,
    /// 池大小
    pool_size: usize,
}

impl DecalPool {
    /// 创建新的decal池
    pub fn new(pool_size: usize) -> Self {
        Self {
            free_decals: Vec::with_capacity(pool_size),
            pool_size,
        }
    }

    /// 获取decal（从池中）
    pub fn acquire(&mut self) -> Decal {
        self.free_decals.pop().unwrap_or_else(|| {
            // Pool exhausted, creating new decal
            tracing::warn!("Decal pool exhausted, creating new decal instance");
            Decal::at_position(DecalType::Custom, Vec3::ZERO)
        })
    }

    /// 归还decal到池
    pub fn release(&mut self, mut decal: Decal) {
        // 重置状态
        decal.active = false;
        decal.age = 0.0;

        if self.free_decals.len() < self.pool_size {
            self.free_decals.push(decal);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decal_creation() {
        let decal = Decal::at_position(DecalType::BulletHole, Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(decal.decal_type, DecalType::BulletHole);
        assert_eq!(decal.position, Vec3::new(1.0, 2.0, 3.0));
        assert!(decal.active);
    }

    #[test]
    fn test_decal_lifetime() {
        let mut decal = Decal::at_position(DecalType::Blood, Vec3::ZERO).with_lifetime(1.0);

        assert!(!decal.update(0.5)); // 0.5秒，仍然存活
        assert!(!decal.update(0.3)); // 0.8秒，仍然存活
        assert!(decal.update(0.3)); // 1.1秒，应该移除
    }

    #[test]
    fn test_decal_manager() {
        let mut manager = DecalManager::default_config();

        manager.spawn_decal(DecalType::BulletHole, Vec3::ZERO, Vec3::Y);

        assert_eq!(manager.stats().active_count, 1);
        assert_eq!(manager.stats().total_created, 1);
    }

    #[test]
    fn test_decal_max_limit() {
        let config = DecalConfig {
            max_decals: 3,
            ..Default::default()
        };

        let mut manager = DecalManager::new(config);

        // 添加超过限制的decal
        for i in 0..5 {
            manager.spawn_decal(
                DecalType::BulletHole,
                Vec3::new(i as f32, 0.0, 0.0),
                Vec3::Y,
            );
        }

        // 应该保留最新的3个
        assert_eq!(manager.stats().active_count, 3);
    }

    #[test]
    fn test_decal_culling() {
        let config = DecalConfig {
            cull_distance: 10.0,
            ..Default::default()
        };

        let mut manager = DecalManager::new(config);

        // 添加近距离decal
        manager.spawn_decal(DecalType::BulletHole, Vec3::ZERO, Vec3::Y);
        // 添加远距离decal
        manager.spawn_decal(DecalType::BulletHole, Vec3::new(100.0, 0.0, 0.0), Vec3::Y);

        manager.update(0.0, Vec3::ZERO);

        // 应该剔除远处的decal
        assert_eq!(manager.stats().active_count, 1);
    }

    #[test]
    fn test_decal_projection() {
        let decal = Decal::at_position(DecalType::Graffiti, Vec3::ZERO);
        let surface_pos = Vec3::new(0.1, 0.1, 0.0);
        let surface_normal = Vec3::Z;

        let result = DecalProjector::project_to_surface(&decal, surface_pos, surface_normal);

        assert!(result.is_some());
        let (uv, weight) = result.unwrap_or_else(|| {
            tracing::error!("Decal projection failed unexpectedly");
            panic!("Expected projection to succeed");
        });
        assert!(uv.x >= 0.0 && uv.x <= 1.0);
        assert!(uv.y >= 0.0 && uv.y <= 1.0);
        assert!(weight > 0.0);
    }

    #[test]
    fn test_decal_batch_renderer() {
        let renderer = DecalBatchRenderer::default_config();

        // 创建 decals 并存储在变量中以延长生命周期
        let decal1 = Decal::at_position(DecalType::BulletHole, Vec3::ZERO);
        let decal2 = Decal::at_position(DecalType::Explosion, Vec3::X);

        let decals = vec![&decal1, &decal2];

        let batches = renderer.render_batch(&decals);

        // 应该有2个批次（不同类型）
        assert_eq!(batches.len(), 2);
    }
}
