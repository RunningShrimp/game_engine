//! 聚合根不变式测试
//!
//! 本模块包含所有聚合根的不变式验证测试，确保业务规则在边界内执行。

use crate::domain::entity::{EntityFactory, EntityId};
// DomainError 未在测试中实际使用，已删除
use crate::domain::scene::{Scene, SceneId, SceneState};
use crate::ecs::Camera;
use glam::Vec3;

#[cfg(test)]
mod scene_invariants_tests {
    use super::*;

    /// 测试场景名称不能为空的不变式
    #[test]
    fn test_scene_name_not_empty() {
        let mut scene = Scene::new(SceneId(1), "Valid Scene");
        scene.name = String::new();
        assert!(scene.validate().is_err());
    }

    /// 测试实体ID唯一性不变式
    #[test]
    fn test_scene_entity_id_uniqueness() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().expect("Test: operation should succeed");

        let entity1 = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        scene.add_entity(entity1).expect("Test: operation should succeed");

        // 尝试添加重复ID的实体应该失败
        let entity2 = EntityFactory::create_basic(EntityId(1), Vec3::new(1.0, 1.0, 1.0));
        let result = scene.add_entity(entity2);
        assert!(result.is_err());
    }

    /// 测试活跃场景最多只能有一个相机的不变式
    #[test]
    fn test_scene_active_camera_limit() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().expect("Test: operation should succeed");
        scene.activate().expect("Test: operation should succeed");

        // 添加第一个相机（应该成功）
        let entity1 = EntityFactory::create_camera(EntityId(1), Vec3::ZERO, Camera::default());
        assert!(scene.add_entity(entity1).is_ok());
        assert!(scene.validate().is_ok());

        // 尝试添加第二个相机（应该失败）
        let entity2 =
            EntityFactory::create_camera(EntityId(2), Vec3::new(1.0, 0.0, 0.0), Camera::default());
        let result = scene.add_entity(entity2);
        assert!(result.is_err());
    }

    /// 测试场景激活时所有实体必须激活的不变式
    #[test]
    fn test_scene_active_entities_must_be_active() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().expect("Test: operation should succeed");

        // 添加一个非活跃实体
        let mut entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        entity.deactivate().expect("Test: operation should succeed");
        scene.entities.insert(EntityId(1), entity);

        // 激活场景（应该自动激活所有实体）
        assert!(scene.activate().is_ok());
        assert!(scene.validate().is_ok());

        // 验证实体已激活
        assert!(
            scene
                .get_entity(EntityId(1))
                .expect("Test: operation should succeed")
                .is_active()
        );
    }

    /// 测试所有实体必须有效的不变式
    #[test]
    fn test_scene_all_entities_must_be_valid() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().expect("Test: operation should succeed");

        // 创建一个违反实体不变式的实体（同时有Sprite和Camera）
        let mut invalid_entity =
            EntityFactory::create_sprite(EntityId(1), Vec3::ZERO, crate::ecs::Sprite::default());
        invalid_entity.camera = Some(Camera::default());

        // 直接插入应该导致验证失败
        scene.entities.insert(EntityId(1), invalid_entity);
        assert!(scene.validate().is_err());
    }

    /// 测试状态转换不变式
    #[test]
    fn test_scene_state_transition_invariants() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");

        // 只能从Unloaded状态加载
        assert!(scene.load().is_ok());
        assert_eq!(scene.state, SceneState::Loaded);

        // 只能从Loaded或Inactive状态激活
        assert!(scene.activate().is_ok());
        assert_eq!(scene.state, SceneState::Active);

        // 只能从Active状态停用
        assert!(scene.deactivate().is_ok());
        assert_eq!(scene.state, SceneState::Inactive);

        // 可以从任何状态卸载
        assert!(scene.unload().is_ok());
        assert_eq!(scene.state, SceneState::Unloaded);
    }
}
