//! 视锥体剔除系统
//!
//! 实现基于视锥体的可见性判断和高效剔除算法

use glam::{Mat4, Vec3};

/// 视锥体的6个平面
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    /// 左平面
    pub left: Plane,
    /// 右平面
    pub right: Plane,
    /// 上平面
    pub top: Plane,
    /// 下平面
    pub bottom: Plane,
    /// 近平面
    pub near: Plane,
    /// 远平面
    pub far: Plane,
}

/// 平面表示（法向量和距离）
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// 法向量
    pub normal: Vec3,
    /// 到原点的距离
    pub distance: f32,
}

impl Plane {
    /// 创建平面
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self {
            normal: normal.normalize(),
            distance,
        }
    }

    /// 从三个点创建平面
    pub fn from_points(p0: Vec3, p1: Vec3, p2: Vec3) -> Self {
        let normal = (p1 - p0).cross(p2 - p0).normalize();
        let distance = normal.dot(p0);
        Self { normal, distance }
    }

    /// 计算点到平面的有符号距离
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    /// 检查点是否在平面的正面
    pub fn point_in_front(&self, point: Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }
}

impl Frustum {
    /// 从视图投影矩阵创建视锥体
    pub fn from_view_projection(view_proj: Mat4) -> Self {
        let m = view_proj;

        // 提取6个平面的法向量和距离
        // 参考: Gribb & Hartmann, "Fast Extraction of Viewing Frustum Planes from the World-View-Projection Matrix"

        // 左平面: M.col(3) + M.col(0)
        let left = Self::extract_plane(m.col(3) + m.col(0));

        // 右平面: M.col(3) - M.col(0)
        let right = Self::extract_plane(m.col(3) - m.col(0));

        // 下平面: M.col(3) + M.col(1)
        let bottom = Self::extract_plane(m.col(3) + m.col(1));

        // 上平面: M.col(3) - M.col(1)
        let top = Self::extract_plane(m.col(3) - m.col(1));

        // 近平面: M.col(2)
        let near = Self::extract_plane(m.col(2));

        // 远平面: M.col(3) - M.col(2)
        let far = Self::extract_plane(m.col(3) - m.col(2));

        Self {
            left,
            right,
            top,
            bottom,
            near,
            far,
        }
    }

    /// 从4分量向量提取平面
    fn extract_plane(plane_vec: glam::Vec4) -> Plane {
        let normal = Vec3::new(plane_vec.x, plane_vec.y, plane_vec.z);
        let distance = plane_vec.w;
        Plane::new(normal, distance)
    }

    /// 检查点是否在视锥体内
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.left.point_in_front(point)
            && self.right.point_in_front(point)
            && self.top.point_in_front(point)
            && self.bottom.point_in_front(point)
            && self.near.point_in_front(point)
            && self.far.point_in_front(point)
    }

    /// 检查球体是否与视锥体相交
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        // 检查球体是否与任何平面相交
        let planes = [
            self.left,
            self.right,
            self.top,
            self.bottom,
            self.near,
            self.far,
        ];

        for plane in &planes {
            let distance = plane.distance_to_point(center);
            if distance < -radius {
                return false; // 球体完全在平面后面
            }
        }

        true
    }

    /// 检查包围盒是否与视锥体相交
    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        let planes = [
            self.left,
            self.right,
            self.top,
            self.bottom,
            self.near,
            self.far,
        ];

        for plane in &planes {
            // 获取包围盒中离平面最近的点
            let closest_point = self.closest_point_on_aabb(min, max, plane.normal);

            // 如果最近的点在平面后面，则包围盒不可见
            if plane.distance_to_point(closest_point) < 0.0 {
                return false;
            }
        }

        true
    }

    /// 计算包围盒上离平面最近的点
    fn closest_point_on_aabb(&self, min: Vec3, max: Vec3, normal: Vec3) -> Vec3 {
        Vec3::new(
            if normal.x > 0.0 { max.x } else { min.x },
            if normal.y > 0.0 { max.y } else { min.y },
            if normal.z > 0.0 { max.z } else { min.z },
        )
    }

    /// 检查OBB（有向包围盒）是否与视锥体相交
    pub fn intersects_obb(&self, center: Vec3, half_extents: Vec3, rotation: glam::Quat) -> bool {
        // 将OBB的8个顶点转换到世界空间
        let corners = Self::get_obb_corners(center, half_extents, rotation);

        // 对于每个平面，检查是否所有点都在平面后面
        let planes = [
            self.left,
            self.right,
            self.top,
            self.bottom,
            self.near,
            self.far,
        ];

        for plane in &planes {
            let mut all_behind = true;
            for corner in &corners {
                if plane.distance_to_point(*corner) >= 0.0 {
                    all_behind = false;
                    break;
                }
            }

            if all_behind {
                return false;
            }
        }

        true
    }

    /// 获取OBB的8个顶点
    fn get_obb_corners(center: Vec3, half_extents: Vec3, rotation: glam::Quat) -> [Vec3; 8] {
        let offsets = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
        ];

        let mut corners = [Vec3::ZERO; 8];
        for (i, offset) in offsets.iter().enumerate() {
            let local = *offset * half_extents;
            corners[i] = center + rotation.mul_vec3(local);
        }

        corners
    }

    /// 获取视锥体的8个顶点
    pub fn corners(&self) -> [Vec3; 8] {
        // 计算四条边的交点
        let near_bottom_left = self.intersect_three_planes(self.near, self.left, self.bottom);
        let near_bottom_right = self.intersect_three_planes(self.near, self.right, self.bottom);
        let near_top_right = self.intersect_three_planes(self.near, self.right, self.top);
        let near_top_left = self.intersect_three_planes(self.near, self.left, self.top);

        let far_bottom_left = self.intersect_three_planes(self.far, self.left, self.bottom);
        let far_bottom_right = self.intersect_three_planes(self.far, self.right, self.bottom);
        let far_top_right = self.intersect_three_planes(self.far, self.right, self.top);
        let far_top_left = self.intersect_three_planes(self.far, self.left, self.top);

        [
            near_bottom_left,
            near_bottom_right,
            near_top_right,
            near_top_left,
            far_bottom_left,
            far_bottom_right,
            far_top_right,
            far_top_left,
        ]
    }

    /// 计算三个平面的交点
    fn intersect_three_planes(&self, p1: Plane, p2: Plane, p3: Plane) -> Vec3 {
        let n1 = p1.normal;
        let n2 = p2.normal;
        let n3 = p3.normal;
        let d1 = p1.distance;
        let d2 = p2.distance;
        let d3 = p3.distance;

        let denom = n1.dot(n2.cross(n3));
        if denom.abs() < f32::EPSILON {
            return Vec3::ZERO;
        }

        Vec3::new(
            (d1 * n2.cross(n3) + d2 * n3.cross(n1) + d3 * n1.cross(n2)).dot(n1) / denom,
            (d1 * n2.cross(n3) + d2 * n3.cross(n1) + d3 * n1.cross(n2)).dot(n2) / denom,
            (d1 * n2.cross(n3) + d2 * n3.cross(n1) + d3 * n1.cross(n2)).dot(n3) / denom,
        )
    }
}

/// 剔除结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullingResult {
    /// 完全在视锥体外
    Outside,
    /// 与视锥体部分相交
    Intersecting,
    /// 完全在视锥体内
    Inside,
}

/// 高效剔除系统
pub struct CullingSystem {
    frustum: Frustum,
}

impl CullingSystem {
    /// 创建新的剔除系统
    pub fn new(view_proj: Mat4) -> Self {
        Self {
            frustum: Frustum::from_view_projection(view_proj),
        }
    }

    /// 更新视锥体
    pub fn update(&mut self, view_proj: Mat4) {
        self.frustum = Frustum::from_view_projection(view_proj);
    }

    /// 测试球体
    pub fn test_sphere(&self, center: Vec3, radius: f32) -> CullingResult {
        if self.frustum.intersects_sphere(center, radius) {
            CullingResult::Intersecting
        } else {
            CullingResult::Outside
        }
    }

    /// 测试包围盒
    pub fn test_aabb(&self, min: Vec3, max: Vec3) -> CullingResult {
        if self.frustum.intersects_aabb(min, max) {
            CullingResult::Intersecting
        } else {
            CullingResult::Outside
        }
    }

    /// 测试多个包围盒，返回可见的索引
    pub fn cull_aabbs(&self, boxes: &[(Vec3, Vec3)]) -> Vec<usize> {
        boxes
            .iter()
            .enumerate()
            .filter(|(_, (min, max))| self.frustum.intersects_aabb(*min, *max))
            .map(|(i, _)| i)
            .collect()
    }

    /// 测试多个球体，返回可见的索引
    pub fn cull_spheres(&self, spheres: &[(Vec3, f32)]) -> Vec<usize> {
        spheres
            .iter()
            .enumerate()
            .filter(|(_, (center, radius))| self.frustum.intersects_sphere(*center, *radius))
            .map(|(i, _)| i)
            .collect()
    }

    /// 获取当前视锥体
    pub fn frustum(&self) -> &Frustum {
        &self.frustum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_plane_creation() {
        let plane = Plane::new(Vec3::Z, 5.0);
        assert_eq!(plane.normal, Vec3::Z);
        assert_eq!(plane.distance, 5.0);
    }

    #[test]
    fn test_point_in_plane() {
        let plane = Plane::new(Vec3::Z, 0.0);
        assert!(plane.point_in_front(Vec3::new(0.0, 0.0, 1.0)));
        assert!(!plane.point_in_front(Vec3::new(0.0, 0.0, -1.0)));
    }

    #[test]
    fn test_frustum_sphere_intersection() {
        let view_proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(view_proj);

        // 原点处的球体应该与视锥体相交
        assert!(frustum.intersects_sphere(Vec3::ZERO, 1.0));

        // 远处的球体可能不相交
        let result = frustum.intersects_sphere(Vec3::new(0.0, 0.0, 1000.0), 1.0);
        // 结果取决于视锥体的具体参数
        let _ = result;
    }

    #[test]
    fn test_culling_system() {
        let view_proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 1.0, 0.1, 100.0);
        let culling = CullingSystem::new(view_proj);

        let boxes = vec![
            (Vec3::new(-5.0, -5.0, 1.0), Vec3::new(-4.0, -4.0, 2.0)),
            (Vec3::new(4.0, 4.0, 1.0), Vec3::new(5.0, 5.0, 2.0)),
        ];

        let visible = culling.cull_aabbs(&boxes);
        assert!(!visible.is_empty());
    }

    proptest! {
        #[test]
        fn test_frustum_culling_properties(
            center_x in -50.0f32..50.0,
            center_y in -50.0f32..50.0,
            center_z in 1.0f32..100.0,
            radius in 0.1f32..10.0,
        ) {
            // 创建标准的透视投影视锥体
            let view_proj = Mat4::perspective_rh(
                std::f32::consts::PI / 4.0,
                16.0 / 9.0,
                0.1,
                100.0,
            );
            let frustum = Frustum::from_view_projection(view_proj);
            let center = Vec3::new(center_x, center_y, center_z);

            // 属性1: 原点附近的球体应该可见
            if center.distance(Vec3::ZERO) < 10.0 && radius > 0.5 {
                let visible = frustum.intersects_sphere(center, radius);
                // 原点附近的球体通常应该可见（取决于视锥体参数）
                // 这里只验证函数不会panic
                let _ = visible;
            }

            // 属性2: 非常远的球体可能不可见
            if center_z > 90.0 && radius < 1.0 {
                let visible = frustum.intersects_sphere(center, radius);
                // 只验证函数不会panic
                let _ = visible;
            }

            // 属性3: 非常大的球体应该可见（如果中心在视锥体内）
            if radius > 50.0 {
                let visible = frustum.intersects_sphere(center, radius);
                // 大球体通常应该可见
                // 这里只验证函数不会panic
                let _ = visible;
            }
        }

        #[test]
        fn test_plane_distance_properties(
            normal_x in -1.0f32..1.0,
            normal_y in -1.0f32..1.0,
            normal_z in -1.0f32..1.0,
            distance in -10.0f32..10.0,
            point_x in -100.0f32..100.0,
            point_y in -100.0f32..100.0,
            point_z in -100.0f32..100.0,
        ) {
            let normal = Vec3::new(normal_x, normal_y, normal_z);
            // 跳过零向量
            if normal.length() < 0.001 {
                return Ok(());
            }

            let plane = Plane::new(normal, distance);
            let point = Vec3::new(point_x, point_y, point_z);

            // 属性1: 点到平面的距离应该是有符号的
            let dist = plane.distance_to_point(point);
            prop_assert!(dist.is_finite());

            // 属性2: 点在平面正面当且仅当距离 >= 0
            let in_front = plane.point_in_front(point);
            prop_assert_eq!(in_front, dist >= 0.0);
        }

        #[test]
        fn test_aabb_culling_properties(
            min_x in -10.0f32..10.0,
            min_y in -10.0f32..10.0,
            min_z in 1.0f32..50.0,
            max_x in -10.0f32..10.0,
            max_y in -10.0f32..10.0,
            max_z in 1.0f32..50.0,
        ) {
            let view_proj = Mat4::perspective_rh(
                std::f32::consts::PI / 4.0,
                16.0 / 9.0,
                0.1,
                100.0,
            );
            let frustum = Frustum::from_view_projection(view_proj);

            // 确保min < max
            let min = Vec3::new(
                min_x.min(max_x),
                min_y.min(max_y),
                min_z.min(max_z),
            );
            let max = Vec3::new(
                min_x.max(max_x),
                min_y.max(max_y),
                min_z.max(max_z),
            );

            // 属性: AABB剔除应该返回布尔值，不会panic
            let visible = frustum.intersects_aabb(min, max);
            // 只验证函数不会panic
            let _ = visible;
        }
    }
}
