// ============================================================================
// XR 空间映射模块
// 提供空间网格重建、平面检测、遮挡查询等功能
// ============================================================================

use super::*;
use glam::{Vec3, Quat, Mat4};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 空间网格顶点
#[derive(Debug, Clone, Copy)]
pub struct MeshVertex {
    /// 顶点位置
    pub position: Vec3,
    /// 顶点法线
    pub normal: Vec3,
    /// 顶点颜色（可选）
    pub color: Option<[u8; 4]>,
}

/// 空间网格三角形
#[derive(Debug, Clone, Copy)]
pub struct MeshTriangle {
    /// 三个顶点的索引
    pub indices: [u32; 3],
}

/// 空间网格
///
/// 表示XR环境中的3D网格重建数据
#[derive(Debug, Clone)]
pub struct SpatialMesh {
    /// 网格唯一标识符
    pub id: MeshId,
    /// 网格顶点列表
    pub vertices: Vec<MeshVertex>,
    /// 网格三角形列表
    pub triangles: Vec<MeshTriangle>,
    /// 网格的变换矩阵（相对于世界空间）
    pub transform: Mat4,
    /// 网格是否有效
    pub is_valid: bool,
    /// 网格更新时间戳（毫秒）
    pub last_updated: u64,
    /// 网格元数据
    pub metadata: HashMap<String, String>,
}

/// 网格ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MeshId(u64);

impl MeshId {
    /// 创建新的网格ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for MeshId {
    fn default() -> Self {
        Self::new()
    }
}

/// 检测到的平面
#[derive(Debug, Clone)]
pub struct DetectedPlane {
    /// 平面唯一标识符
    pub id: PlaneId,
    /// 平面中心位置
    pub center: Vec3,
    /// 平面法线
    pub normal: Vec3,
    /// 平面尺寸（宽度、高度）
    pub size: [f32; 2],
    /// 平面边界点（凸包）
    pub boundary: Vec<Vec3>,
    /// 平面类型
    pub plane_type: PlaneType,
    /// 平面是否有效
    pub is_valid: bool,
    /// 平面更新时间戳（毫秒）
    pub last_updated: u64,
    /// 平面元数据
    pub metadata: HashMap<String, String>,
}

/// 平面ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaneId(u64);

impl PlaneId {
    /// 创建新的平面ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for PlaneId {
    fn default() -> Self {
        Self::new()
    }
}

/// 平面类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneType {
    /// 水平向上（如桌面、地板）
    HorizontalUp,
    /// 水平向下（如天花板）
    HorizontalDown,
    /// 垂直（如墙壁）
    Vertical,
    /// 其他方向
    Arbitrary,
}

/// 空间映射配置
#[derive(Debug, Clone)]
pub struct SpatialMappingConfig {
    /// 是否启用空间映射
    pub enabled: bool,
    /// 网格更新频率（Hz）
    pub update_frequency: f32,
    /// 网格分辨率（米）
    pub mesh_resolution: f32,
    /// 是否启用平面检测
    pub plane_detection_enabled: bool,
    /// 平面检测阈值
    pub plane_detection_threshold: f32,
    /// 最大网格数量
    pub max_mesh_count: usize,
    /// 最大平面数量
    pub max_plane_count: usize,
}

impl Default for SpatialMappingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_frequency: 30.0, // 30 Hz
            mesh_resolution: 0.05,  // 5 cm
            plane_detection_enabled: true,
            plane_detection_threshold: 0.1,
            max_mesh_count: 100,
            max_plane_count: 50,
        }
    }
}

/// 空间映射管理器
///
/// 管理XR环境中的空间网格重建和平面检测
pub struct SpatialMappingManager {
    /// 空间映射配置
    config: SpatialMappingConfig,
    /// 所有空间网格
    meshes: HashMap<MeshId, SpatialMesh>,
    /// 所有检测到的平面
    planes: HashMap<PlaneId, DetectedPlane>,
    /// 是否已初始化
    initialized: bool,
    /// 是否支持空间映射
    supported: bool,
    /// 最后更新时间戳（毫秒）
    last_update_time: u64,
}

impl SpatialMappingManager {
    /// 创建新的空间映射管理器
    ///
    /// # 返回
    ///
    /// 返回一个新的 `SpatialMappingManager` 实例。
    ///
    /// # 错误
    ///
    /// 如果初始化失败，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::xr::spatial_mapping::SpatialMappingManager;
    ///
    /// let manager = SpatialMappingManager::new()?;
    /// ```
    pub fn new() -> Result<Self, XrError> {
        Ok(Self {
            config: SpatialMappingConfig::default(),
            meshes: HashMap::new(),
            planes: HashMap::new(),
            initialized: false,
            supported: false, // 占位：实际应检查OpenXR扩展支持
            last_update_time: crate::core::utils::current_timestamp_ms(),
        })
    }

    /// 使用指定配置创建空间映射管理器
    ///
    /// # 参数
    ///
    /// * `config` - 空间映射配置
    ///
    /// # 返回
    ///
    /// 返回一个新的 `SpatialMappingManager` 实例。
    ///
    /// # 错误
    ///
    /// 如果初始化失败，返回 `XrError`。
    pub fn with_config(config: SpatialMappingConfig) -> Result<Self, XrError> {
        Ok(Self {
            config,
            meshes: HashMap::new(),
            planes: HashMap::new(),
            initialized: false,
            supported: false,
            last_update_time: crate::core::utils::current_timestamp_ms(),
        })
    }

    /// 初始化空间映射
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    ///
    /// # 错误
    ///
    /// 如果不支持空间映射，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// manager.initialize()?;
    /// ```
    pub fn initialize(&mut self) -> Result<(), XrError> {
        if !self.supported {
            return Err(XrError::FeatureNotSupported(
                "Spatial mapping not supported".to_string(),
            ));
        }

        self.initialized = true;
        Ok(())
    }

    /// 更新空间映射数据
    ///
    /// 从XR系统获取最新的空间网格和平面数据。
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    ///
    /// # 错误
    ///
    /// 如果更新失败，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// manager.update()?;
    /// ```
    pub fn update(&mut self) -> Result<(), XrError> {
        if !self.initialized || !self.config.enabled {
            return Ok(());
        }

        let now = crate::core::utils::current_timestamp_ms();
        let delta_time = (now - self.last_update_time) as f32 / 1000.0;

        // 检查是否需要更新
        if delta_time < 1.0 / self.config.update_frequency {
            return Ok(());
        }

        // 占位实现：实际应该从OpenXR获取空间映射数据
        // xr::SceneObserverMSFT::create_scene()
        // xr::SceneMSFT::compute_new_scene_components()
        // xr::SceneComponentMSFT::get_mesh_bounds()
        // xr::SceneMeshMSFT::get_vertices() / get_indices()

        self.last_update_time = now;
        Ok(())
    }

    /// 添加或更新空间网格
    ///
    /// # 参数
    ///
    /// * `mesh` - 空间网格数据
    ///
    /// # 示例
    ///
    /// ```rust
    /// let mesh = SpatialMesh { /* ... */ };
    /// manager.add_or_update_mesh(mesh);
    /// ```
    pub fn add_or_update_mesh(&mut self, mesh: SpatialMesh) {
        // 限制最大网格数量
        if self.meshes.len() >= self.config.max_mesh_count {
            // 移除最旧的网格
            if let Some(oldest_id) = self
                .meshes
                .iter()
                .min_by_key(|(_, m)| m.last_updated)
                .map(|(id, _)| *id)
            {
                self.meshes.remove(&oldest_id);
            }
        }

        self.meshes.insert(mesh.id, mesh);
    }

    /// 获取空间网格
    ///
    /// # 参数
    ///
    /// * `id` - 网格ID
    ///
    /// # 返回
    ///
    /// 如果网格存在，返回网格的只读引用；否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// if let Some(mesh) = manager.get_mesh(mesh_id) {
    ///     println!("Mesh has {} vertices", mesh.vertices.len());
    /// }
    /// ```
    pub fn get_mesh(&self, id: MeshId) -> Option<&SpatialMesh> {
        self.meshes.get(&id)
    }

    /// 获取所有空间网格
    ///
    /// # 返回
    ///
    /// 返回所有空间网格的列表。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let all_meshes = manager.get_all_meshes();
    /// println!("Total meshes: {}", all_meshes.len());
    /// ```
    pub fn get_all_meshes(&self) -> Vec<&SpatialMesh> {
        self.meshes.values().collect()
    }

    /// 移除空间网格
    ///
    /// # 参数
    ///
    /// * `id` - 网格ID
    ///
    /// # 示例
    ///
    /// ```rust
    /// manager.remove_mesh(mesh_id);
    /// ```
    pub fn remove_mesh(&mut self, id: MeshId) {
        self.meshes.remove(&id);
    }

    /// 添加或更新检测到的平面
    ///
    /// # 参数
    ///
    /// * `plane` - 检测到的平面数据
    ///
    /// # 示例
    ///
    /// ```rust
    /// let plane = DetectedPlane { /* ... */ };
    /// manager.add_or_update_plane(plane);
    /// ```
    pub fn add_or_update_plane(&mut self, plane: DetectedPlane) {
        // 限制最大平面数量
        if self.planes.len() >= self.config.max_plane_count {
            // 移除最旧的平面
            if let Some(oldest_id) = self
                .planes
                .iter()
                .min_by_key(|(_, p)| p.last_updated)
                .map(|(id, _)| *id)
            {
                self.planes.remove(&oldest_id);
            }
        }

        self.planes.insert(plane.id, plane);
    }

    /// 获取检测到的平面
    ///
    /// # 参数
    ///
    /// * `id` - 平面ID
    ///
    /// # 返回
    ///
    /// 如果平面存在，返回平面的只读引用；否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// if let Some(plane) = manager.get_plane(plane_id) {
    ///     println!("Plane center: {:?}", plane.center);
    /// }
    /// ```
    pub fn get_plane(&self, id: PlaneId) -> Option<&DetectedPlane> {
        self.planes.get(&id)
    }

    /// 获取所有检测到的平面
    ///
    /// # 返回
    ///
    /// 返回所有检测到的平面的列表。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let all_planes = manager.get_all_planes();
    /// println!("Total planes: {}", all_planes.len());
    /// ```
    pub fn get_all_planes(&self) -> Vec<&DetectedPlane> {
        self.planes.values().collect()
    }

    /// 查询指定类型的平面
    ///
    /// # 参数
    ///
    /// * `plane_type` - 平面类型
    ///
    /// # 返回
    ///
    /// 返回匹配指定类型的所有平面。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let horizontal_planes = manager.query_planes_by_type(PlaneType::HorizontalUp);
    /// ```
    pub fn query_planes_by_type(&self, plane_type: PlaneType) -> Vec<&DetectedPlane> {
        self.planes
            .values()
            .filter(|p| p.plane_type == plane_type)
            .collect()
    }

    /// 移除检测到的平面
    ///
    /// # 参数
    ///
    /// * `id` - 平面ID
    ///
    /// # 示例
    ///
    /// ```rust
    /// manager.remove_plane(plane_id);
    /// ```
    pub fn remove_plane(&mut self, id: PlaneId) {
        self.planes.remove(&id);
    }

    /// 查询遮挡
    ///
    /// 检查从起点到终点的射线是否被空间网格遮挡。
    ///
    /// # 参数
    ///
    /// * `origin` - 射线起点
    /// * `direction` - 射线方向（归一化）
    /// * `max_distance` - 最大查询距离
    ///
    /// # 返回
    ///
    /// 如果被遮挡，返回 `Some(碰撞点)`；否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use glam::Vec3;
    ///
    /// let origin = Vec3::new(0.0, 0.0, 0.0);
    /// let direction = Vec3::new(0.0, 0.0, -1.0);
    /// if let Some(hit_point) = manager.query_occlusion(origin, direction, 10.0) {
    ///     println!("Hit at: {:?}", hit_point);
    /// }
    /// ```
    pub fn query_occlusion(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<Vec3> {
        // 占位实现：实际应该进行射线-三角形相交测试
        // 遍历所有网格，对每个网格的三角形进行相交测试
        // 返回最近的碰撞点

        None
    }

    /// 获取配置
    ///
    /// # 返回
    ///
    /// 返回空间映射配置的只读引用。
    pub fn config(&self) -> &SpatialMappingConfig {
        &self.config
    }

    /// 更新配置
    ///
    /// # 参数
    ///
    /// * `config` - 新的空间映射配置
    ///
    /// # 示例
    ///
    /// ```rust
    /// let mut config = manager.config().clone();
    /// config.mesh_resolution = 0.1;
    /// manager.update_config(config);
    /// ```
    pub fn update_config(&mut self, config: SpatialMappingConfig) {
        self.config = config;
    }

    /// 检查是否支持空间映射
    ///
    /// # 返回
    ///
    /// 如果支持空间映射，返回 `true`；否则返回 `false`。
    pub fn is_supported(&self) -> bool {
        self.supported
    }

    /// 检查是否已初始化
    ///
    /// # 返回
    ///
    /// 如果已初始化，返回 `true`；否则返回 `false`。
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 获取网格数量
    ///
    /// # 返回
    ///
    /// 返回当前管理的网格数量。
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// 获取平面数量
    ///
    /// # 返回
    ///
    /// 返回当前检测到的平面数量。
    pub fn plane_count(&self) -> usize {
        self.planes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_mapping_manager_creation() {
        let manager = SpatialMappingManager::new().unwrap();
        assert!(!manager.is_initialized());
        assert!(!manager.is_supported());
        assert_eq!(manager.mesh_count(), 0);
        assert_eq!(manager.plane_count(), 0);
    }

    #[test]
    fn test_add_and_get_mesh() {
        let mut manager = SpatialMappingManager::new().unwrap();
        let mesh_id = MeshId::new();
        let mesh = SpatialMesh {
            id: mesh_id,
            vertices: vec![],
            triangles: vec![],
            transform: Mat4::IDENTITY,
            is_valid: true,
            last_updated: crate::core::utils::current_timestamp_ms(),
            metadata: HashMap::new(),
        };

        manager.add_or_update_mesh(mesh);
        assert_eq!(manager.mesh_count(), 1);
        assert!(manager.get_mesh(mesh_id).is_some());
    }

    #[test]
    fn test_add_and_get_plane() {
        let mut manager = SpatialMappingManager::new().unwrap();
        let plane_id = PlaneId::new();
        let plane = DetectedPlane {
            id: plane_id,
            center: Vec3::ZERO,
            normal: Vec3::Y,
            size: [1.0, 1.0],
            boundary: vec![],
            plane_type: PlaneType::HorizontalUp,
            is_valid: true,
            last_updated: crate::core::utils::current_timestamp_ms(),
            metadata: HashMap::new(),
        };

        manager.add_or_update_plane(plane);
        assert_eq!(manager.plane_count(), 1);
        assert!(manager.get_plane(plane_id).is_some());
    }

    #[test]
    fn test_query_planes_by_type() {
        let mut manager = SpatialMappingManager::new().unwrap();

        let plane1 = DetectedPlane {
            id: PlaneId::new(),
            center: Vec3::ZERO,
            normal: Vec3::Y,
            size: [1.0, 1.0],
            boundary: vec![],
            plane_type: PlaneType::HorizontalUp,
            is_valid: true,
            last_updated: crate::core::utils::current_timestamp_ms(),
            metadata: HashMap::new(),
        };

        let plane2 = DetectedPlane {
            id: PlaneId::new(),
            center: Vec3::ZERO,
            normal: Vec3::X,
            size: [1.0, 1.0],
            boundary: vec![],
            plane_type: PlaneType::Vertical,
            is_valid: true,
            last_updated: crate::core::utils::current_timestamp_ms(),
            metadata: HashMap::new(),
        };

        manager.add_or_update_plane(plane1);
        manager.add_or_update_plane(plane2);

        let horizontal_planes = manager.query_planes_by_type(PlaneType::HorizontalUp);
        assert_eq!(horizontal_planes.len(), 1);

        let vertical_planes = manager.query_planes_by_type(PlaneType::Vertical);
        assert_eq!(vertical_planes.len(), 1);
    }
}

