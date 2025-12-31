// 剔除优化
//
// 实现视锥体剔除、遮挡剔除和LOD选择

use std::collections::HashMap;

// ============================================================================
// 视锥体剔除
// ============================================================================

/// 视锥体平面
#[derive(Debug, Clone, Copy)]
pub enum FrustumPlane {
    Left,
    Right,
    Top,
    Bottom,
    Near,
    Far,
}

/// 视锥体
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    /// 平面方程 (ax + by + cz + d = 0)
    pub planes: [[f32; 4]; 6],
}

impl Frustum {
    /// 从视图投影矩阵创建视锥体
    pub fn from_view_projection(view_projection: &[[f32; 4]; 4]) -> Self {
        let mut planes = [[0.0; 4]; 6];

        // Left plane
        planes[0][0] = view_projection[3][0] + view_projection[0][0];
        planes[0][1] = view_projection[3][1] + view_projection[0][1];
        planes[0][2] = view_projection[3][2] + view_projection[0][2];
        planes[0][3] = view_projection[3][3] + view_projection[0][3];

        // Right plane
        planes[1][0] = view_projection[3][0] - view_projection[0][0];
        planes[1][1] = view_projection[3][1] - view_projection[0][1];
        planes[1][2] = view_projection[3][2] - view_projection[0][2];
        planes[1][3] = view_projection[3][3] - view_projection[0][3];

        // Top plane
        planes[2][0] = view_projection[3][0] - view_projection[1][0];
        planes[2][1] = view_projection[3][1] - view_projection[1][1];
        planes[2][2] = view_projection[3][2] - view_projection[1][2];
        planes[2][3] = view_projection[3][3] - view_projection[1][3];

        // Bottom plane
        planes[3][0] = view_projection[3][0] + view_projection[1][0];
        planes[3][1] = view_projection[3][1] + view_projection[1][1];
        planes[3][2] = view_projection[3][2] + view_projection[1][2];
        planes[3][3] = view_projection[3][3] + view_projection[1][3];

        // Near plane
        planes[4][0] = view_projection[3][0] + view_projection[2][0];
        planes[4][1] = view_projection[3][1] + view_projection[2][1];
        planes[4][2] = view_projection[3][2] + view_projection[2][2];
        planes[4][3] = view_projection[3][3] + view_projection[2][3];

        // Far plane
        planes[5][0] = view_projection[3][0] - view_projection[2][0];
        planes[5][1] = view_projection[3][1] - view_projection[2][1];
        planes[5][2] = view_projection[3][2] - view_projection[2][2];
        planes[5][3] = view_projection[3][3] - view_projection[2][3];

        // 归一化平面
        for i in 0..6 {
            let length = (planes[i][0] * planes[i][0]
                + planes[i][1] * planes[i][1]
                + planes[i][2] * planes[i][2])
                .sqrt();
            if length > 0.0 {
                planes[i][0] /= length;
                planes[i][1] /= length;
                planes[i][2] /= length;
                planes[i][3] /= length;
            }
        }

        Self { planes }
    }

    /// 测试AABB是否在视锥体内
    pub fn test_aabb(&self, min: [f32; 3], max: [f32; 3]) -> bool {
        for i in 0..6 {
            let plane = self.planes[i];
            let normal = [plane[0], plane[1], plane[2]];
            let d = plane[3];

            // 找到AABB离平面最远的顶点
            let px = if normal[0] > 0.0 { max[0] } else { min[0] };
            let py = if normal[1] > 0.0 { max[1] } else { min[1] };
            let pz = if normal[2] > 0.0 { max[2] } else { min[2] };

            // 计算点到平面的距离
            let distance = normal[0] * px + normal[1] * py + normal[2] * pz + d;

            // 如果所有顶点都在平面外侧，则AABB在视锥体外
            if distance < 0.0 {
                return false;
            }
        }

        true
    }

    /// 测试球体是否在视锥体内
    pub fn test_sphere(&self, center: [f32; 3], radius: f32) -> bool {
        for i in 0..6 {
            let plane = self.planes[i];
            let distance = plane[0] * center[0]
                + plane[1] * center[1]
                + plane[2] * center[2]
                + plane[3];

            if distance < -radius {
                return false;
            }
        }

        true
    }
}

/// 视锥体剔除器
pub struct FrustumCuller {
    /// 当前视锥体
    frustum: Option<Frustum>,
    /// 剔除统计
    stats: CullingStats,
}

/// 剔除统计
#[derive(Debug, Clone, Copy, Default)]
pub struct CullingStats {
    /// 总对象数
    pub total_objects: u32,
    /// 可见对象数
    pub visible_objects: u32,
    /// 剔除对象数
    pub culled_objects: u32,
}

impl CullingStats {
    /// 计算剔除效率
    pub fn culling_efficiency(&self) -> f32 {
        if self.total_objects > 0 {
            (self.culled_objects as f32 / self.total_objects as f32) * 100.0
        } else {
            0.0
        }
    }
}

impl FrustumCuller {
    /// 创建新的剔除器
    pub fn new() -> Self {
        Self {
            frustum: None,
            stats: CullingStats::default(),
        }
    }

    /// 更新视锥体
    pub fn update_frustum(&mut self, view_projection: &[[f32; 4]; 4]) {
        self.frustum = Some(Frustum::from_view_projection(view_projection));
        self.stats = CullingStats::default();
    }

    /// 测试AABB可见性
    pub fn test_aabb(&mut self, min: [f32; 3], max: [f32; 3]) -> bool {
        self.stats.total_objects += 1;

        if let Some(frustum) = &self.frustum {
            let visible = frustum.test_aabb(min, max);
            if visible {
                self.stats.visible_objects += 1;
            } else {
                self.stats.culled_objects += 1;
            }
            visible
        } else {
            true // 没有视锥体时，假设所有对象都可见
        }
    }

    /// 测试球体可见性
    pub fn test_sphere(&mut self, center: [f32; 3], radius: f32) -> bool {
        self.stats.total_objects += 1;

        if let Some(frustum) = &self.frustum {
            let visible = frustum.test_sphere(center, radius);
            if visible {
                self.stats.visible_objects += 1;
            } else {
                self.stats.culled_objects += 1;
            }
            visible
        } else {
            true
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> CullingStats {
        self.stats
    }
}

impl Default for FrustumCuller {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 遮挡剔除
// ============================================================================

/// 遮挡剔除器（基础实现）
pub struct OcclusionCuller {
    /// 启用状态
    enabled: bool,
    /// 历史剔除结果（用于时序相干性）
    history: HashMap<u32, bool>,
    /// 帧计数
    frame_count: u32,
}

impl OcclusionCuller {
    /// 创建新的遮挡剔除器
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            history: HashMap::new(),
            frame_count: 0,
        }
    }

    /// 开始新帧
    pub fn begin_frame(&mut self) {
        self.frame_count += 1;
    }

    /// 测试对象遮挡（简化实现）
    pub fn test_occlusion(&mut self, object_id: u32, bounds: ([f32; 3], [f32; 3])) -> bool {
        if !self.enabled {
            return true;
        }

        // 时序相干性优化：使用历史结果
        if let Some(&was_visible) = self.history.get(&object_id) {
            // 如果上次可见，这次也可见（保守估计）
            if was_visible {
                self.history.insert(object_id, true);
                return true;
            }
        }

        // 实际应该执行遮挡查询（简化为总是可见）
        let visible = true;
        self.history.insert(object_id, visible);
        visible
    }

    /// 获取统计信息
    pub fn stats(&self) -> OcclusionStats {
        let visible_count = self.history.values().filter(|&&v| v).count() as u32;
        let culled_count = self.history.len() as u32 - visible_count;

        OcclusionStats {
            total_queries: self.history.len() as u32,
            visible_queries: visible_count,
            culled_objects: culled_count,
        }
    }
}

/// 遮挡剔除统计
#[derive(Debug, Clone, Copy)]
pub struct OcclusionStats {
    pub total_queries: u32,
    pub visible_queries: u32,
    pub culled_objects: u32,
}

// ============================================================================
// LOD（细节层次）选择
// ============================================================================

/// LOD级别
#[derive(Debug, Clone, Copy)]
pub struct LodLevel {
    /// 距离阈值
    pub distance: f32,
    /// 网格ID
    pub mesh_id: u32,
    /// 屏幕空间大小阈值（像素）
    pub screen_size: f32,
}

/// LOD选择器
pub struct LodSelector {
    /// LOD级别（按对象ID映射）
    lod_levels: HashMap<u32, Vec<LodLevel>>,
    /// 当前相机位置
    camera_position: [f32; 3],
}

impl LodSelector {
    /// 创建新的LOD选择器
    pub fn new() -> Self {
        Self {
            lod_levels: HashMap::new(),
            camera_position: [0.0; 3],
        }
    }

    /// 添加对象的LOD级别
    pub fn add_lod_levels(&mut self, object_id: u32, levels: Vec<LodLevel>) {
        // 按距离排序（从近到远）
        let mut sorted_levels = levels;
        sorted_levels.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        self.lod_levels.insert(object_id, sorted_levels);
    }

    /// 更新相机位置
    pub fn update_camera(&mut self, position: [f32; 3]) {
        self.camera_position = position;
    }

    /// 选择LOD级别
    pub fn select_lod(&self, object_id: u32, object_position: [f32; 3]) -> Option<usize> {
        if let Some(levels) = self.lod_levels.get(&object_id) {
            // 计算到相机的距离
            let dx = object_position[0] - self.camera_position[0];
            let dy = object_position[1] - self.camera_position[1];
            let dz = object_position[2] - self.camera_position[2];
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();

            // 找到合适的LOD级别
            for (i, level) in levels.iter().enumerate() {
                if distance <= level.distance {
                    return Some(i);
                }
            }

            // 返回最低级别
            Some(levels.len() - 1)
        } else {
            None
        }
    }

    /// 获取LOD级别的网格ID
    pub fn get_mesh_id(&self, object_id: u32, lod_index: usize) -> Option<u32> {
        if let Some(levels) = self.lod_levels.get(&object_id) {
            if lod_index < levels.len() {
                Some(levels[lod_index].mesh_id)
            } else {
                levels.last().map(|l| l.mesh_id)
            }
        } else {
            None
        }
    }
}

impl Default for LodSelector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 综合剔除系统
// ============================================================================

/// 剔除系统
pub struct CullingSystem {
    /// 视锥体剔除器
    frustum_culler: FrustumCuller,
    /// 遮挡剔除器
    occlusion_culler: OcclusionCuller,
    /// LOD选择器
    lod_selector: LodSelector,
}

impl CullingSystem {
    /// 创建新的剔除系统
    pub fn new(enable_occlusion: bool) -> Self {
        Self {
            frustum_culler: FrustumCuller::new(),
            occlusion_culler: OcclusionCuller::new(enable_occlusion),
            lod_selector: LodSelector::new(),
        }
    }

    /// 更新视锥体
    pub fn update_frustum(&mut self, view_projection: &[[f32; 4]; 4]) {
        self.frustum_culler.update_frustum(view_projection);
    }

    /// 更新相机位置
    pub fn update_camera(&mut self, position: [f32; 3]) {
        self.lod_selector.update_camera(position);
    }

    /// 开始新帧
    pub fn begin_frame(&mut self) {
        self.occlusion_culler.begin_frame();
    }

    /// 测试对象可见性（综合剔除）
    pub fn test_visibility(&mut self, object_id: u32, bounds: ([f32; 3], [f32; 3])) -> bool {
        let (min, max) = bounds;

        // 视锥体剔除
        if !self.frustum_culler.test_aabb(min, max) {
            return false;
        }

        // 遮挡剔除
        if !self.occlusion_culler.test_occlusion(object_id, bounds) {
            return false;
        }

        true
    }

    /// 选择LOD级别
    pub fn select_lod(&self, object_id: u32, position: [f32; 3]) -> Option<usize> {
        self.lod_selector.select_lod(object_id, position)
    }

    /// 添加LOD级别
    pub fn add_lod_levels(&mut self, object_id: u32, levels: Vec<LodLevel>) {
        self.lod_selector.add_lod_levels(object_id, levels);
    }

    /// 获取视锥体剔除统计
    pub fn frustum_stats(&self) -> CullingStats {
        self.frustum_culler.stats()
    }

    /// 获取遮挡剔除统计
    pub fn occlusion_stats(&self) -> OcclusionStats {
        self.occlusion_culler.stats()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_creation() {
        // 单位矩阵
        let view_proj = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let frustum = Frustum::from_view_projection(&view_proj);
        // 应该包含原点附近的对象
        assert!(frustum.test_aabb([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]));
    }

    #[test]
    fn test_frustum_culler() {
        let mut culler = FrustumCuller::new();

        let view_proj = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        culler.update_frustum(&view_proj);

        // 测试可见对象
        assert!(culler.test_aabb([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]));

        let stats = culler.stats();
        assert_eq!(stats.total_objects, 1);
        assert_eq!(stats.visible_objects, 1);
    }

    #[test]
    fn test_lod_selector() {
        let mut selector = LodSelector::new();

        let levels = vec![
            LodLevel {
                distance: 10.0,
                mesh_id: 1,
                screen_size: 100.0,
            },
            LodLevel {
                distance: 20.0,
                mesh_id: 2,
                screen_size: 50.0,
            },
        ];

        selector.add_lod_levels(0, levels);
        selector.update_camera([0.0, 0.0, 0.0]);

        // 近距离应该选择LOD 0
        assert_eq!(selector.select_lod(0, [5.0, 0.0, 0.0]), Some(0));

        // 远距离应该选择LOD 1
        assert_eq!(selector.select_lod(0, [15.0, 0.0, 0.0]), Some(1));
    }

    #[test]
    fn test_culling_stats() {
        let stats = CullingStats {
            total_objects: 100,
            visible_objects: 60,
            culled_objects: 40,
        };

        assert!((stats.culling_efficiency() - 40.0).abs() < 0.01);
    }
}
