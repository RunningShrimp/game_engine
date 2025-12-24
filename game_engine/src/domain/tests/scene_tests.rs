//  场景测试模块
// 
//  提供对场景聚合根的全面测试覆盖，包括状态转换、实体管理、事件溯源等。

use crate::domain::entity::{EntityId, EntityFactory, GameEntity};
use crate::domain::errors::{CompensationAction, DomainError, RecoveryStrategy, SceneError};
use crate::domain::scene::{Scene, SceneManager, SceneId, SceneMetadata, SceneState};
use crate::ecs::{Camera, PointLight, Sprite, Transform};
use glam::Vec3;

#[cfg(test)]
mod scene_id_tests {
    use super::*;

    #[test]
    fn test_scene_id_creation() {
        let id = SceneId::new(42);
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn test_scene_id_equality() {
        let id1 = SceneId::new(42);
        let id2 = SceneId::new(42);
        let id3 = SceneId::new(24);
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_scene_id_hash() {
        use std::collections::HashSet;
        
        let id1 = SceneId::new(42);
        let id2 = SceneId::new(42);
        let id3 = SceneId::new(24);
        
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id3);
        
        assert_eq!(set.len(), 2);
    }
}

#[cfg(test)]
mod scene_metadata_tests {
    use super::*;

    #[test]
    fn test_scene_metadata_default() {
        let metadata = SceneMetadata::new();
        
        assert!(metadata.author.is_none());
        assert!(metadata.description.is_none());
        assert!(metadata.created_at > 0);
        assert!(metadata.modified_at > 0);
        assert_eq!(metadata.version, 1);
    }

    #[test]
    fn test_scene_metadata_timestamps() {
        let metadata = SceneMetadata::new();
        let created = metadata.created_at;
        let modified = metadata.modified_at;
        
        assert_eq!(created, modified);
    }
}

#[cfg(test)]
mod scene_creation_tests {
    use super::*;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new(SceneId::new(1), "test_scene");
        
        assert_eq!(scene.id(), SceneId::new(1));
        assert_eq!(scene.name(), "test_scene");
        assert_eq!(scene.state(), SceneState::Unloaded);
        assert_eq!(scene.total_entity_count(), 0);
        assert_eq!(scene.active_entity_count(), 0);
    }

    #[test]
    fn test_scene_try_new_valid() {
        let scene = Scene::try_new(SceneId::new(1), "valid_name");
        
        assert!(scene.is_ok());
        let scene = scene.unwrap();
        assert_eq!(scene.name(), "valid_name");
    }

    #[test]
    fn test_scene_try_new_empty_name() {
        let scene = Scene::try_new(SceneId::new(1), "");
        
        assert!(scene.is_err());
        if let Err(DomainError::Scene(SceneError::InvalidName(msg))) = scene {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected InvalidName error");
        }
    }

    #[test]
    fn test_scene_try_new_whitespace_name() {
        let scene = Scene::try_new(SceneId::new(1), "   ");
        
        assert!(scene.is_err());
        if let Err(DomainError::Scene(SceneError::InvalidName(msg))) = scene {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected InvalidName error");
        }
    }
}

#[cfg(test)]
mod scene_state_transition_tests {
    use super::*;

    #[test]
    fn test_scene_state_unloaded_to_loading() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        assert_eq!(scene.state(), SceneState::Unloaded);
        
        let result = scene.load();
        assert!(result.is_ok());
        assert_eq!(scene.state(), SceneState::Loading);
    }

    #[test]
    fn test_scene_state_loading_to_loaded() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        
        let result = scene.load();
        assert!(result.is_ok());
        assert_eq!(scene.state(), SceneState::Loaded);
    }

    #[test]
    fn test_scene_state_loaded_to_active() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.load().unwrap();
        
        let result = scene.activate();
        assert!(result.is_ok());
        assert_eq!(scene.state(), SceneState::Active);
    }

    #[test]
    fn test_scene_state_active_to_inactive() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.load().unwrap();
        scene.activate().unwrap();
        
        let result = scene.deactivate();
        assert!(result.is_ok());
        assert_eq!(scene.state(), SceneState::Inactive);
    }

    #[test]
    fn test_scene_state_inactive_to_active() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.load().unwrap();
        scene.activate().unwrap();
        scene.deactivate().unwrap();
        
        let result = scene.activate();
        assert!(result.is_ok());
        assert_eq!(scene.state(), SceneState::Active);
    }

    #[test]
    fn test_scene_state_any_to_unloaded() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.load().unwrap();
        scene.activate().unwrap();
        
        let result = scene.unload();
        assert!(result.is_ok());
        assert_eq!(scene.state(), SceneState::Unloaded);
    }

    #[test]
    fn test_scene_state_invalid_transition() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        let result = scene.activate();
        assert!(result.is_err());
        assert_eq!(scene.state(), SceneState::Unloaded);
    }

    #[test]
    fn test_scene_state_activate_without_entities() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.load().unwrap();
        
        let result = scene.activate();
        assert!(result.is_ok());
        assert_eq!(scene.active_entity_count(), 0);
    }
}

#[cfg(test)]
mod scene_entity_management_tests {
    use super::*;

    #[test]
    fn test_scene_add_entity() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        let entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        let result = scene.add_entity(entity);
        assert!(result.is_ok());
        assert_eq!(scene.total_entity_count(), 1);
        assert!(scene.contains_entity(EntityId::new(1)));
    }

    #[test]
    fn test_scene_add_duplicate_entity() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        let entity1 = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        let entity2 = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        scene.add_entity(entity1).unwrap();
        let result = scene.add_entity(entity2);
        
        assert!(result.is_err());
        if let Err(DomainError::Scene(SceneError::DuplicateEntity(id))) = result {
            assert_eq!(id, EntityId::new(1));
        } else {
            panic!("Expected DuplicateEntity error");
        }
    }

    #[test]
    fn test_scene_remove_entity() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        let entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        scene.add_entity(entity).unwrap();
        
        let result = scene.remove_entity(EntityId::new(1));
        assert!(result.is_ok());
        assert_eq!(scene.total_entity_count(), 0);
        assert!(!scene.contains_entity(EntityId::new(1)));
    }

    #[test]
    fn test_scene_remove_nonexistent_entity() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        let result = scene.remove_entity(EntityId::new(1));
        assert!(result.is_err());
        if let Err(DomainError::Scene(SceneError::EntityNotFound(id))) = result {
            assert_eq!(id, EntityId::new(1));
        } else {
            panic!("Expected EntityNotFound error");
        }
    }

    #[test]
    fn test_scene_get_entity() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        let entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        scene.add_entity(entity).unwrap();
        
        let retrieved = scene.get_entity(EntityId::new(1));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, EntityId::new(1));
    }

    #[test]
    fn test_scene_get_entity_mut() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        let entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        scene.add_entity(entity).unwrap();
        
        if let Some(entity) = scene.get_entity_mut(EntityId::new(1)) {
            entity.set_name("Updated Name").unwrap();
        }
        
        let retrieved = scene.get_entity(EntityId::new(1));
        assert_eq!(retrieved.unwrap().name, Some("Updated Name".to_string()));
    }

    #[test]
    fn test_scene_find_entity_by_name() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        entity.set_name("Player").unwrap();
        scene.add_entity(entity).unwrap();
        
        let found = scene.find_entity_by_name("Player");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, EntityId::new(1));
    }

    #[test]
    fn test_scene_find_entity_by_name_not_found() {
        let scene = Scene::new(SceneId::new(1), "test");
        
        let found = scene.find_entity_by_name("Nonexistent");
        assert!(found.is_none());
    }

    #[test]
    fn test_scene_entity_ids() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        scene.add_entity(EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId::new(2), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId::new(3), Vec3::ZERO)).unwrap();
        
        let ids = scene.entity_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&EntityId::new(1)));
        assert!(ids.contains(&EntityId::new(2)));
        assert!(ids.contains(&EntityId::new(3)));
    }

    #[test]
    fn test_scene_add_multiple_entities() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        let entities = vec![
            EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO),
            EntityFactory::create_basic(EntityId::new(2), Vec3::ZERO),
            EntityFactory::create_basic(EntityId::new(3), Vec3::ZERO),
        ];
        
        let result = scene.add_entities(entities);
        assert!(result.is_ok());
        assert_eq!(scene.total_entity_count(), 3);
    }

    #[test]
    fn test_scene_remove_multiple_entities() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        scene.add_entity(EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId::new(2), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId::new(3), Vec3::ZERO)).unwrap();
        
        let ids = vec![EntityId::new(1), EntityId::new(3)];
        let result = scene.remove_entities(ids);
        assert!(result.is_ok());
        assert_eq!(scene.total_entity_count(), 1);
        assert!(scene.contains_entity(EntityId::new(2)));
    }
}

#[cfg(test)]
mod scene_validation_tests {
    use super::*;

    #[test]
    fn test_scene_validate_valid_scene() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.load().unwrap();
        
        let result = scene.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_scene_validate_duplicate_entity_ids() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        let entity1 = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        let entity2 = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        scene.entities.insert(EntityId::new(1), entity1);
        scene.entities.insert(EntityId::new(2), entity2);
        
        let result = scene.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_scene_validate_invalid_entity() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        entity.scale(Vec3::new(-1.0, 1.0, 1.0)).unwrap();
        
        scene.add_entity(entity).unwrap();
        
        let result = scene.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_scene_validate_multiple_active_cameras() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.load().unwrap();
        scene.activate().unwrap();
        
        let camera1 = EntityFactory::create_camera(EntityId::new(1), Vec3::ZERO, Camera::default());
        let camera2 = EntityFactory::create_camera(EntityId::new(2), Vec3::ZERO, Camera::default());
        
        scene.add_entity(camera1).unwrap();
        scene.add_entity(camera2).unwrap();
        
        let result = scene.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_scene_validate_inactive_entities_in_active_scene() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.load().unwrap();
        
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        entity.deactivate().unwrap();
        
        scene.add_entity(entity).unwrap();
        
        let result = scene.activate();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod scene_event_sourcing_tests {
    use super::*;

    #[test]
    fn test_scene_uncommitted_event_count() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        assert_eq!(scene.uncommitted_event_count(), 0);
        
        scene.load().unwrap();
        assert_eq!(scene.uncommitted_event_count(), 1);
        
        scene.load().unwrap();
        assert_eq!(scene.uncommitted_event_count(), 2);
    }

    #[test]
    fn test_scene_take_uncommitted_events() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        scene.load().unwrap();
        scene.load().unwrap();
        
        let events = scene.take_uncommitted_events();
        assert_eq!(events.len(), 2);
        assert_eq!(scene.uncommitted_event_count(), 0);
    }

    #[test]
    fn test_scene_events_after_entity_operations() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        scene.add_entity(EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO)).unwrap();
        assert_eq!(scene.uncommitted_event_count(), 1);
        
        scene.remove_entity(EntityId::new(1)).unwrap();
        assert_eq!(scene.uncommitted_event_count(), 2);
    }
}

#[cfg(test)]
mod scene_recovery_tests {
    use super::*;

    #[test]
    fn test_scene_recover_from_error() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        
        let error = SceneError::EntityNotFound(EntityId::new(999));
        let result = scene.recover_from_error(&error);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_scene_create_compensation() {
        let scene = Scene::new(SceneId::new(1), "test");
        
        let compensation = scene.create_compensation();
        assert!(matches!(compensation, CompensationAction::Retry { .. }));
    }

    #[test]
    fn test_scene_set_recovery_strategy() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        let strategy = RecoveryStrategy::Retry {
            max_attempts: 5,
            delay_ms: 200,
        };
        scene.set_recovery_strategy(strategy.clone());
        
        assert_eq!(scene.recovery_strategy(), &strategy);
    }
}

#[cfg(test)]
mod scene_snapshot_tests {
    use super::*;

    #[test]
    fn test_scene_create_snapshot() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        scene.load().unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO)).unwrap();
        
        let snapshot = scene.create_snapshot();
        
        assert_eq!(snapshot.scene_id, scene.id());
        assert_eq!(snapshot.state, scene.state());
        assert_eq!(snapshot.entity_count, scene.total_entity_count());
    }
}

#[cfg(test)]
mod scene_manager_tests {
    use super::*;

    #[test]
    fn test_scene_manager_create_scene() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId::new(1), "test_scene").unwrap();
        
        let scene = manager.get_scene(SceneId::new(1));
        assert!(scene.is_some());
        assert_eq!(scene.unwrap().id(), SceneId::new(1));
    }

    #[test]
    fn test_scene_manager_delete_scene() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId::new(1), "test_scene").unwrap();
        
        let result = manager.delete_scene(SceneId::new(1));
        assert!(result.is_ok());
        
        let retrieved = manager.get_scene(SceneId::new(1));
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_scene_manager_get_scene() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId::new(1), "test_scene").unwrap();
        
        let scene = manager.get_scene(SceneId::new(1));
        assert!(scene.is_some());
        assert_eq!(scene.unwrap().id(), SceneId::new(1));
    }

    #[test]
    fn test_scene_manager_get_scene_mut() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId::new(1), "test_scene").unwrap();
        
        if let Some(scene) = manager.get_scene_mut(SceneId::new(1)) {
            scene.load().unwrap();
        }
        
        let scene = manager.get_scene(SceneId::new(1));
        assert_eq!(scene.unwrap().state(), SceneState::Loading);
    }

    #[test]
    fn test_scene_manager_switch_to_scene() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId::new(1), "scene1").unwrap();
        manager.create_scene(SceneId::new(2), "scene2").unwrap();
        
        manager.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        manager.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        manager.get_scene_mut(SceneId::new(1)).unwrap().activate().unwrap();
        
        let result = manager.switch_to_scene(SceneId::new(2));
        assert!(result.is_ok());
        
        assert_eq!(manager.active_scene().unwrap().id(), SceneId::new(2));
    }

    #[test]
    fn test_scene_manager_active_scene() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId::new(1), "test").unwrap();
        
        manager.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        manager.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        manager.get_scene_mut(SceneId::new(1)).unwrap().activate().unwrap();
        
        let active = manager.active_scene();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id(), SceneId::new(1));
    }

    #[test]
    fn test_scene_manager_current_scene() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId::new(1), "test").unwrap();
        
        manager.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        
        let current = manager.current_scene();
        assert!(current.is_some());
        assert_eq!(current.unwrap().id(), SceneId::new(1));
    }

    #[test]
    fn test_scene_manager_scene_ids() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId::new(1), "scene1").unwrap();
        manager.create_scene(SceneId::new(2), "scene2").unwrap();
        manager.create_scene(SceneId::new(3), "scene3").unwrap();
        
        let ids = manager.scene_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&SceneId::new(1)));
        assert!(ids.contains(&SceneId::new(2)));
        assert!(ids.contains(&SceneId::new(3)));
    }
}

#[cfg(test)]
mod scene_edge_cases_tests {
    use super::*;

    #[test]
    fn test_scene_large_number_of_entities() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        for i in 0..1000 {
            let entity = EntityFactory::create_basic(EntityId::new(i), Vec3::ZERO);
            scene.add_entity(entity).unwrap();
        }
        
        assert_eq!(scene.total_entity_count(), 1000);
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn test_scene_entity_iterators() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        scene.add_entity(EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId::new(2), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId::new(3), Vec3::ZERO)).unwrap();
        
        let count = scene.entities_iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_scene_metadata_mut() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        
        scene.metadata_mut().author = Some("Test Author".to_string());
        scene.metadata_mut().description = Some("Test Description".to_string());
        
        assert_eq!(scene.metadata().author, Some("Test Author".to_string()));
        assert_eq!(scene.metadata().description, Some("Test Description".to_string()));
    }

    #[test]
    fn test_scene_last_modified() {
        let mut scene = Scene::new(SceneId::new(1), "test");
        let initial = scene.last_modified();
        
        scene.load().unwrap();
        
        assert!(scene.last_modified() > initial);
    }
}

#[cfg(test)]
mod scene_serialization_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_scene_serialization() {
        let mut scene = Scene::new(SceneId::new(1), "test_scene");
        scene.load().unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO)).unwrap();
        
        let serialized = serde_json::to_string(&scene).unwrap();
        let deserialized: Scene = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(scene.id(), deserialized.id());
        assert_eq!(scene.name(), deserialized.name());
        assert_eq!(scene.total_entity_count(), deserialized.total_entity_count());
    }

    #[test]
    fn test_scene_id_serialization() {
        let id = SceneId::new(42);
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: SceneId = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_scene_state_serialization() {
        let state = SceneState::Active;
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: SceneState = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(state, deserialized);
    }
}
