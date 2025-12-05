//! OpenXR 空间锚点扩展集成
//!
//! 实现OpenXR空间锚点扩展（XR_MSFT_spatial_anchor），提供持久化的空间定位点。
//!
//! ## 功能特性
//!
//! - 创建和销毁空间锚点
//! - 锚点持久化存储
//! - 锚点位置和旋转追踪
//! - 锚点有效性检测
//! - 锚点查询和过滤
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::xr::spatial_anchors::*;
//!
//! // 创建空间锚点管理器
//! let mut anchor_manager = SpatialAnchorManager::new()?;
//!
//! // 在当前位置创建锚点
//! let pose = Pose {
//!     position: Vec3::new(0.0, 0.0, -1.0),
//!     orientation: Quat::IDENTITY,
//! };
//! let anchor_id = anchor_manager.create_anchor(pose, "MyAnchor")?;
//!
//! // 查询锚点
//! if let Some(anchor) = anchor_manager.get_anchor(anchor_id) {
//!     println!("Anchor position: {:?}", anchor.pose.position);
//! }
//! ```

use super::*;
use crate::core::utils::current_timestamp_ms;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 空间锚点ID
///
/// 唯一标识一个空间锚点的标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnchorId(pub u64);

impl AnchorId {
    /// 生成新的锚点ID
    ///
    /// 生成一个全局唯一的锚点ID。ID从1开始递增。
    ///
    /// # 返回
    ///
    /// 返回一个新的 `AnchorId` 实例。
    ///
    /// # 安全性
    ///
    /// 此方法使用静态计数器，在多线程环境下可能不安全。
    /// 在生产环境中应使用线程安全的ID生成器。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::xr::spatial_anchors::AnchorId;
    ///
    /// let id = AnchorId::new();
    /// ```
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            AnchorId(COUNTER)
        }
    }
}

/// 空间锚点
///
/// 表示XR空间中的一个固定位置点，可以用于持久化空间定位。
/// 锚点包含位置、旋转、元数据等信息，并支持持久化存储。
#[derive(Debug, Clone)]
pub struct SpatialAnchor {
    /// 锚点ID
    pub id: AnchorId,
    /// 锚点名称
    pub name: String,
    /// 锚点姿态
    pub pose: Pose,
    /// 是否有效
    pub is_valid: bool,
    /// 创建时间（Unix时间戳，毫秒）
    pub created_at: u64,
    /// 最后更新时间
    pub last_updated: u64,
    /// 持久化存储键（如果已持久化）
    pub persistence_key: Option<String>,
    /// 锚点元数据
    pub metadata: HashMap<String, String>,
}

impl SpatialAnchor {
    /// 创建新的空间锚点
    ///
    /// # 参数
    ///
    /// * `id` - 锚点唯一标识符
    /// * `name` - 锚点名称
    /// * `pose` - 锚点的位置和旋转
    ///
    /// # 返回
    ///
    /// 返回一个新的 `SpatialAnchor` 实例，默认有效，创建时间和更新时间都设置为当前时间。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::xr::spatial_anchors::{AnchorId, SpatialAnchor};
    /// use game_engine::xr::{Pose, Vec3, Quat};
    ///
    /// let id = AnchorId::new();
    /// let pose = Pose {
    ///     position: Vec3::new(0.0, 0.0, -1.0),
    ///     orientation: Quat::IDENTITY,
    /// };
    /// let anchor = SpatialAnchor::new(id, "MyAnchor".to_string(), pose);
    /// ```
    pub fn new(id: AnchorId, name: String, pose: Pose) -> Self {
        let now = current_timestamp_ms();
        Self {
            id,
            name,
            pose,
            is_valid: true,
            created_at: now,
            last_updated: now,
            persistence_key: None,
            metadata: HashMap::new(),
        }
    }

    /// 更新锚点姿态
    ///
    /// # 参数
    ///
    /// * `pose` - 新的位置和旋转
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::xr::{Pose, Vec3, Quat};
    ///
    /// let mut anchor = SpatialAnchor::new(AnchorId::new(), "Test".to_string(), Pose {
    ///     position: Vec3::ZERO,
    ///     orientation: Quat::IDENTITY,
    /// });
    /// anchor.update_pose(Pose {
    ///     position: Vec3::new(1.0, 0.0, 0.0),
    ///     orientation: Quat::IDENTITY,
    /// });
    /// ```
    pub fn update_pose(&mut self, pose: Pose) {
        self.pose = pose;
        self.last_updated = current_timestamp_ms();
    }

    /// 设置元数据
    ///
    /// # 参数
    ///
    /// * `key` - 元数据键
    /// * `value` - 元数据值
    ///
    /// # 示例
    ///
    /// ```rust
    /// anchor.set_metadata("room".to_string(), "LivingRoom".to_string());
    /// ```
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// 获取元数据
    ///
    /// # 参数
    ///
    /// * `key` - 元数据键
    ///
    /// # 返回
    ///
    /// 如果存在对应的元数据，返回值的引用；否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// if let Some(room) = anchor.get_metadata("room") {
    ///     println!("Anchor is in room: {}", room);
    /// }
    /// ```
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 标记锚点为无效
    ///
    /// 当锚点不再有效时（例如，XR系统无法追踪该位置），调用此方法标记为无效。
    /// 无效的锚点不应再用于空间定位。
    ///
    /// # 示例
    ///
    /// ```rust
    /// anchor.invalidate(); // 标记为无效
    /// ```
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }
}

/// 空间锚点管理器
///
/// 管理XR空间中的所有锚点，提供创建、查询、更新、持久化等功能。
pub struct SpatialAnchorManager {
    /// 锚点映射
    anchors: HashMap<AnchorId, SpatialAnchor>,
    /// 持久化存储（占位实现）
    persistence_store: Arc<Mutex<HashMap<String, AnchorPersistenceData>>>,
    /// 是否支持持久化
    persistence_supported: bool,
}

/// 锚点持久化数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnchorPersistenceData {
    anchor_id: AnchorId,
    pose: Pose,
    name: String,
    metadata: HashMap<String, String>,
    created_at: u64,
}

impl SpatialAnchorManager {
    /// 创建新的空间锚点管理器
    ///
    /// # 返回
    ///
    /// 返回一个新的 `SpatialAnchorManager` 实例。
    ///
    /// # 错误
    ///
    /// 如果初始化失败，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::xr::spatial_anchors::SpatialAnchorManager;
    ///
    /// let manager = SpatialAnchorManager::new()?;
    /// ```
    pub fn new() -> Result<Self, XrError> {
        Ok(Self {
            anchors: HashMap::new(),
            persistence_store: Arc::new(Mutex::new(HashMap::new())),
            persistence_supported: false, // 占位：实际应检查OpenXR扩展支持
        })
    }

    /// 创建空间锚点
    ///
    /// 在指定位置创建一个新的空间锚点。
    ///
    /// # 参数
    ///
    /// * `pose` - 锚点的位置和旋转
    /// * `name` - 锚点名称
    ///
    /// # 返回
    ///
    /// 返回新创建的锚点ID。
    ///
    /// # 错误
    ///
    /// 如果创建失败，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::xr::{Pose, Vec3, Quat};
    ///
    /// let pose = Pose {
    ///     position: Vec3::new(0.0, 0.0, -1.0),
    ///     orientation: Quat::IDENTITY,
    /// };
    /// let anchor_id = manager.create_anchor(pose, "MyAnchor")?;
    /// ```
    pub fn create_anchor(
        &mut self,
        pose: Pose,
        name: impl Into<String>,
    ) -> Result<AnchorId, XrError> {
        let id = AnchorId::new();
        let anchor = SpatialAnchor::new(id, name.into(), pose);

        // NOTE: 实际实现中需要调用OpenXR API创建锚点
        // xr::SpatialAnchorMSFT::create(...)

        self.anchors.insert(id, anchor);
        Ok(id)
    }

    /// 从持久化存储加载锚点
    ///
    /// 从持久化存储中恢复之前保存的锚点。
    ///
    /// # 参数
    ///
    /// * `persistence_key` - 持久化存储键
    ///
    /// # 返回
    ///
    /// 返回恢复的锚点ID。
    ///
    /// # 错误
    ///
    /// 如果锚点不存在或加载失败，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let anchor_id = manager.load_anchor_from_persistence("my_saved_anchor")?;
    /// ```
    pub fn load_anchor_from_persistence(
        &mut self,
        persistence_key: &str,
    ) -> Result<AnchorId, XrError> {
        // NOTE: 实际实现中需要：
        // 1. 从持久化存储加载数据
        // 2. 调用OpenXR API恢复锚点
        // 3. 创建SpatialAnchor对象

        let store = self
            .persistence_store
            .lock()
            .map_err(|e| XrError::RuntimeFailure(format!("Lock error: {}", e)))?;

        if let Some(data) = store.get(persistence_key) {
            let id = data.anchor_id;
            let anchor = SpatialAnchor {
                id,
                name: data.name.clone(),
                pose: data.pose,
                is_valid: true,
                created_at: data.created_at,
                last_updated: current_timestamp_ms(),
                persistence_key: Some(persistence_key.to_string()),
                metadata: data.metadata.clone(),
            };

            drop(store);
            self.anchors.insert(id, anchor);
            Ok(id)
        } else {
            Err(XrError::RuntimeFailure(
                "Anchor not found in persistence store".to_string(),
            ))
        }
    }

    /// 销毁锚点
    ///
    /// 销毁指定的锚点并从管理器中移除。
    ///
    /// # 参数
    ///
    /// * `id` - 要销毁的锚点ID
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    ///
    /// # 错误
    ///
    /// 如果锚点不存在，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// manager.destroy_anchor(anchor_id)?;
    /// ```
    pub fn destroy_anchor(&mut self, id: AnchorId) -> Result<(), XrError> {
        if let Some(mut anchor) = self.anchors.remove(&id) {
            // NOTE: 实际实现中需要调用OpenXR API销毁锚点
            // xr::SpatialAnchorMSFT::destroy(...)

            anchor.invalidate();
            Ok(())
        } else {
            Err(XrError::RuntimeFailure("Anchor not found".to_string()))
        }
    }

    /// 获取锚点
    ///
    /// # 参数
    ///
    /// * `id` - 锚点ID
    ///
    /// # 返回
    ///
    /// 如果锚点存在，返回锚点的只读引用；否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// if let Some(anchor) = manager.get_anchor(anchor_id) {
    ///     println!("Anchor position: {:?}", anchor.pose.position);
    /// }
    /// ```
    pub fn get_anchor(&self, id: AnchorId) -> Option<&SpatialAnchor> {
        self.anchors.get(&id)
    }

    /// 获取锚点（可变引用）
    ///
    /// # 参数
    ///
    /// * `id` - 锚点ID
    ///
    /// # 返回
    ///
    /// 如果锚点存在，返回锚点的可变引用；否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// if let Some(anchor) = manager.get_anchor_mut(anchor_id) {
    ///     anchor.update_pose(new_pose);
    /// }
    /// ```
    pub fn get_anchor_mut(&mut self, id: AnchorId) -> Option<&mut SpatialAnchor> {
        self.anchors.get_mut(&id)
    }

    /// 更新锚点姿态
    ///
    /// # 参数
    ///
    /// * `id` - 锚点ID
    /// * `pose` - 新的位置和旋转
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    ///
    /// # 错误
    ///
    /// 如果锚点不存在，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::xr::{Pose, Vec3, Quat};
    ///
    /// let new_pose = Pose {
    ///     position: Vec3::new(1.0, 0.0, 0.0),
    ///     orientation: Quat::IDENTITY,
    /// };
    /// manager.update_anchor_pose(anchor_id, new_pose)?;
    /// ```
    pub fn update_anchor_pose(&mut self, id: AnchorId, pose: Pose) -> Result<(), XrError> {
        if let Some(anchor) = self.anchors.get_mut(&id) {
            // NOTE: 实际实现中需要调用OpenXR API更新锚点
            // xr::SpatialAnchorMSFT::locate(...)

            anchor.update_pose(pose);
            Ok(())
        } else {
            Err(XrError::RuntimeFailure("Anchor not found".to_string()))
        }
    }

    /// 查询所有锚点
    ///
    /// # 返回
    ///
    /// 返回所有锚点的列表（包括有效和无效的）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let all_anchors = manager.get_all_anchors();
    /// println!("Total anchors: {}", all_anchors.len());
    /// ```
    pub fn get_all_anchors(&self) -> Vec<&SpatialAnchor> {
        self.anchors.values().collect()
    }

    /// 查询有效锚点
    ///
    /// # 返回
    ///
    /// 返回所有有效锚点的列表。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let valid_anchors = manager.get_valid_anchors();
    /// println!("Valid anchors: {}", valid_anchors.len());
    /// ```
    pub fn get_valid_anchors(&self) -> Vec<&SpatialAnchor> {
        self.anchors.values().filter(|a| a.is_valid).collect()
    }

    /// 按名称查询锚点
    ///
    /// # 参数
    ///
    /// * `name` - 锚点名称
    ///
    /// # 返回
    ///
    /// 如果找到匹配的锚点，返回锚点的引用；否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// if let Some(anchor) = manager.find_anchor_by_name("MyAnchor") {
    ///     println!("Found anchor: {:?}", anchor.id);
    /// }
    /// ```
    pub fn find_anchor_by_name(&self, name: &str) -> Option<&SpatialAnchor> {
        self.anchors.values().find(|a| a.name == name)
    }

    /// 按位置查询锚点（在指定半径内）
    ///
    /// 查找指定位置附近的所有有效锚点。
    ///
    /// # 参数
    ///
    /// * `position` - 查询位置
    /// * `radius` - 搜索半径
    ///
    /// # 返回
    ///
    /// 返回在指定半径内的所有有效锚点列表。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::xr::Vec3;
    ///
    /// let nearby = manager.find_anchors_near(Vec3::new(0.0, 0.0, 0.0), 5.0);
    /// println!("Found {} anchors nearby", nearby.len());
    /// ```
    pub fn find_anchors_near(&self, position: Vec3, radius: f32) -> Vec<&SpatialAnchor> {
        self.anchors
            .values()
            .filter(|a| a.is_valid && (a.pose.position - position).length() <= radius)
            .collect()
    }

    /// 持久化锚点
    ///
    /// 将锚点保存到持久化存储，以便在应用重启后恢复。
    ///
    /// # 参数
    ///
    /// * `id` - 要持久化的锚点ID
    /// * `persistence_key` - 持久化存储键
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    ///
    /// # 错误
    ///
    /// 如果持久化不支持、锚点不存在或保存失败，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// manager.persist_anchor(anchor_id, "my_anchor_key")?;
    /// ```
    pub fn persist_anchor(
        &mut self,
        id: AnchorId,
        persistence_key: impl Into<String>,
    ) -> Result<(), XrError> {
        if !self.persistence_supported {
            return Err(XrError::NotSupported);
        }

        if let Some(anchor) = self.anchors.get(&id) {
            let key = persistence_key.into();

            // NOTE: 实际实现中需要调用OpenXR API持久化锚点
            // xr::SpatialAnchorStoreMSFT::persist_anchor(...)

            let persistence_data = AnchorPersistenceData {
                anchor_id: anchor.id,
                pose: anchor.pose,
                name: anchor.name.clone(),
                metadata: anchor.metadata.clone(),
                created_at: anchor.created_at,
            };

            let mut store = self
                .persistence_store
                .lock()
                .map_err(|e| XrError::RuntimeFailure(format!("Lock error: {}", e)))?;
            store.insert(key.clone(), persistence_data);

            // 更新锚点的持久化键
            if let Some(anchor_mut) = self.anchors.get_mut(&id) {
                anchor_mut.persistence_key = Some(key);
            }

            Ok(())
        } else {
            Err(XrError::RuntimeFailure("Anchor not found".to_string()))
        }
    }

    /// 取消持久化锚点
    /// 取消锚点持久化
    ///
    /// 从持久化存储中移除锚点，但不会销毁锚点本身。
    ///
    /// # 参数
    ///
    /// * `id` - 要取消持久化的锚点ID
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`。
    ///
    /// # 错误
    ///
    /// 如果锚点不存在或未持久化，返回 `XrError`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// manager.unpersist_anchor(anchor_id)?;
    /// ```
    pub fn unpersist_anchor(&mut self, id: AnchorId) -> Result<(), XrError> {
        if let Some(anchor) = self.anchors.get_mut(&id) {
            if let Some(ref key) = anchor.persistence_key {
                // NOTE: 实际实现中需要调用OpenXR API取消持久化
                // xr::SpatialAnchorStoreMSFT::unpersist_anchor(...)

                let mut store = self
                    .persistence_store
                    .lock()
                    .map_err(|e| XrError::RuntimeFailure(format!("Lock error: {}", e)))?;
                store.remove(key);

                anchor.persistence_key = None;
            }
            Ok(())
        } else {
            Err(XrError::RuntimeFailure("Anchor not found".to_string()))
        }
    }

    /// 清除所有锚点
    /// 清空所有锚点
    ///
    /// 销毁并移除所有锚点。此操作不可逆。
    ///
    /// # 示例
    ///
    /// ```rust
    /// manager.clear_all(); // 清空所有锚点
    /// ```
    pub fn clear_all(&mut self) {
        // NOTE: 实际实现中需要销毁所有OpenXR锚点
        self.anchors.clear();
    }

    /// 检查持久化是否支持
    /// 检查是否支持持久化
    ///
    /// # 返回
    ///
    /// 如果当前XR系统支持锚点持久化，返回 `true`；否则返回 `false`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// if manager.is_persistence_supported() {
    ///     manager.persist_anchor(anchor_id, "key")?;
    /// }
    /// ```
    pub fn is_persistence_supported(&self) -> bool {
        self.persistence_supported
    }

    /// 获取锚点数量
    /// 获取锚点总数
    ///
    /// # 返回
    ///
    /// 返回所有锚点的数量（包括有效和无效的）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// println!("Total anchors: {}", manager.anchor_count());
    /// ```
    pub fn anchor_count(&self) -> usize {
        self.anchors.len()
    }

    /// 获取有效锚点数量
    /// 获取有效锚点数量
    ///
    /// # 返回
    ///
    /// 返回所有有效锚点的数量。
    ///
    /// # 示例
    ///
    /// ```rust
    /// println!("Valid anchors: {}", manager.valid_anchor_count());
    /// ```
    pub fn valid_anchor_count(&self) -> usize {
        self.anchors.values().filter(|a| a.is_valid).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_anchor() {
        let mut manager = SpatialAnchorManager::new().unwrap();

        let pose = Pose {
            position: Vec3::new(0.0, 0.0, -1.0),
            orientation: Quat::IDENTITY,
        };

        let id = manager.create_anchor(pose, "TestAnchor").unwrap();
        assert!(manager.get_anchor(id).is_some());
        assert_eq!(manager.anchor_count(), 1);
    }

    #[test]
    fn test_destroy_anchor() {
        let mut manager = SpatialAnchorManager::new().unwrap();

        let pose = Pose::default();
        let id = manager.create_anchor(pose, "TestAnchor").unwrap();

        assert_eq!(manager.anchor_count(), 1);
        manager.destroy_anchor(id).unwrap();
        assert_eq!(manager.anchor_count(), 0);
    }

    #[test]
    fn test_update_anchor_pose() {
        let mut manager = SpatialAnchorManager::new().unwrap();

        let pose1 = Pose {
            position: Vec3::new(0.0, 0.0, -1.0),
            orientation: Quat::IDENTITY,
        };
        let id = manager.create_anchor(pose1, "TestAnchor").unwrap();

        let pose2 = Pose {
            position: Vec3::new(1.0, 0.0, -1.0),
            orientation: Quat::IDENTITY,
        };
        manager.update_anchor_pose(id, pose2).unwrap();

        let anchor = manager.get_anchor(id).unwrap();
        assert_eq!(anchor.pose.position, Vec3::new(1.0, 0.0, -1.0));
    }

    #[test]
    fn test_find_anchor_by_name() {
        let mut manager = SpatialAnchorManager::new().unwrap();

        let pose = Pose::default();
        let _id = manager.create_anchor(pose, "MyAnchor").unwrap();

        let anchor = manager.find_anchor_by_name("MyAnchor").unwrap();
        assert_eq!(anchor.name, "MyAnchor");
    }

    #[test]
    fn test_find_anchors_near() {
        let mut manager = SpatialAnchorManager::new().unwrap();

        let pose1 = Pose {
            position: Vec3::new(0.0, 0.0, -1.0),
            orientation: Quat::IDENTITY,
        };
        let _id1 = manager.create_anchor(pose1, "Anchor1").unwrap();

        let pose2 = Pose {
            position: Vec3::new(10.0, 0.0, -1.0),
            orientation: Quat::IDENTITY,
        };
        let _id2 = manager.create_anchor(pose2, "Anchor2").unwrap();

        let nearby = manager.find_anchors_near(Vec3::new(0.5, 0.0, -1.0), 1.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].name, "Anchor1");
    }

    #[test]
    fn test_anchor_metadata() {
        let mut manager = SpatialAnchorManager::new().unwrap();

        let pose = Pose::default();
        let id = manager.create_anchor(pose, "TestAnchor").unwrap();

        let anchor = manager.get_anchor_mut(id).unwrap();
        anchor.set_metadata("type".to_string(), "waypoint".to_string());

        assert_eq!(anchor.get_metadata("type"), Some(&"waypoint".to_string()));
    }
}
