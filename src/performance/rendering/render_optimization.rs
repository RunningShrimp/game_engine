use crate::impl_default;
use glam::Mat4;

/// 平面
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: glam::Vec3,
    pub distance: f32,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            normal: glam::Vec3::ZERO,
            distance: 0.0,
        }
    }
}

impl Plane {
    pub fn new(normal: glam::Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// 计算点到平面的距离
    pub fn distance_to_point(&self, point: glam::Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

/// 视锥体剔除
#[derive(Default)]
pub struct FrustumCulling {
    /// 视锥体平面
    planes: [Plane; 6],
}

impl FrustumCulling {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从视图投影矩阵更新视锥体
    pub fn update_from_matrix(&mut self, view_proj: &Mat4) {
        let m = view_proj.to_cols_array();

        // 左平面
        self.planes[0] = Plane::new(
            glam::Vec3::new(m[3] + m[0], m[7] + m[4], m[11] + m[8]),
            m[15] + m[12],
        );

        // 右平面
        self.planes[1] = Plane::new(
            glam::Vec3::new(m[3] - m[0], m[7] - m[4], m[11] - m[8]),
            m[15] - m[12],
        );

        // 下平面
        self.planes[2] = Plane::new(
            glam::Vec3::new(m[3] + m[1], m[7] + m[5], m[11] + m[9]),
            m[15] + m[13],
        );

        // 上平面
        self.planes[3] = Plane::new(
            glam::Vec3::new(m[3] - m[1], m[7] - m[5], m[11] - m[9]),
            m[15] - m[13],
        );

        // 近平面
        self.planes[4] = Plane::new(
            glam::Vec3::new(m[3] + m[2], m[7] + m[6], m[11] + m[10]),
            m[15] + m[14],
        );

        // 远平面
        self.planes[5] = Plane::new(
            glam::Vec3::new(m[3] - m[2], m[7] - m[6], m[11] - m[10]),
            m[15] - m[14],
        );

        // 归一化平面
        for plane in &mut self.planes {
            let length = plane.normal.length();
            plane.normal /= length;
            plane.distance /= length;
        }
    }

    /// 检查球体是否在视锥体内
    pub fn is_sphere_visible(&self, center: glam::Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
                return false;
            }
        }
        true
    }

    /// 检查AABB是否在视锥体内
    pub fn is_aabb_visible(&self, min: glam::Vec3, max: glam::Vec3) -> bool {
        for plane in &self.planes {
            let mut out = 0;

            // 检查AABB的8个顶点
            for i in 0..8 {
                let point = glam::Vec3::new(
                    if i & 1 != 0 { max.x } else { min.x },
                    if i & 2 != 0 { max.y } else { min.y },
                    if i & 4 != 0 { max.z } else { min.z },
                );

                if plane.distance_to_point(point) < 0.0 {
                    out += 1;
                }
            }

            // 如果所有顶点都在平面外侧,则AABB不可见
            if out == 8 {
                return false;
            }
        }

        true
    }
}

/// 遮挡剔除 (简化版)
#[derive(Default)]
pub struct OcclusionCulling {
    /// 遮挡物列表 (简化为球体)
    occluders: Vec<(glam::Vec3, f32)>, // (center, radius)
}

impl OcclusionCulling {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加遮挡物
    pub fn add_occluder(&mut self, center: glam::Vec3, radius: f32) {
        self.occluders.push((center, radius));
    }

    /// 清空遮挡物
    pub fn clear(&mut self) {
        self.occluders.clear();
    }

    /// 检查物体是否被遮挡 (简化版,仅检查是否被遮挡物完全包含)
    pub fn is_occluded(&self, center: glam::Vec3, radius: f32) -> bool {
        for (occluder_center, occluder_radius) in &self.occluders {
            let distance = center.distance(*occluder_center);
            if distance + radius < *occluder_radius {
                return true; // 被完全包含
            }
        }
        false
    }
}

/// LOD (Level of Detail) 管理器
pub struct LodManager {
    /// LOD距离阈值
    lod_distances: Vec<f32>,
}

impl LodManager {
    pub fn new(lod_distances: Vec<f32>) -> Self {
        Self { lod_distances }
    }

    /// 根据距离选择LOD级别
    pub fn select_lod(&self, distance: f32) -> usize {
        for (i, &threshold) in self.lod_distances.iter().enumerate() {
            if distance < threshold {
                return i;
            }
        }
        self.lod_distances.len()
    }
}

impl Default for LodManager {
    fn default() -> Self {
        Self::new(vec![10.0, 50.0, 100.0]) // 默认3个LOD级别
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_culling() {
        let mut frustum = FrustumCulling::new();

        // 创建一个简单的视图投影矩阵
        let view_proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 100.0);

        frustum.update_from_matrix(&view_proj);

        // 测试球体可见性
        let visible = frustum.is_sphere_visible(glam::Vec3::new(0.0, 0.0, -10.0), 1.0);
        assert!(visible);
    }

    #[test]
    fn test_lod_manager() {
        let lod_manager = LodManager::default();

        assert_eq!(lod_manager.select_lod(5.0), 0);
        assert_eq!(lod_manager.select_lod(30.0), 1);
        assert_eq!(lod_manager.select_lod(80.0), 2);
        assert_eq!(lod_manager.select_lod(150.0), 3);
    }
}
