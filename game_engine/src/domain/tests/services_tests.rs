use crate::domain::audio::{AudioListener, AudioSource, AudioSourceId};
use crate::domain::errors::{AudioError, DomainError, PhysicsError, SceneError};
use crate::domain::physics::{Collider, ColliderId, RigidBody, RigidBodyId, RigidBodyType};
use crate::domain::scene::{Scene, SceneId};
use crate::domain::services::{AudioDomainService, PhysicsDomainService, SceneDomainService};
use crate::domain::value_objects::Volume;
use glam::{Quat, Vec3};

#[cfg(test)]
mod audio_domain_service_tests {
    use super::*;

    #[test]
    fn test_audio_domain_service_new() {
        let service = AudioDomainService::new();
        
        assert_eq!(service.source_ids().len(), 0);
        assert_eq!(service.playing_sources_count(), 0);
    }

    #[test]
    fn test_audio_domain_service_create_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        let result = service.create_source(id, "assets/test.mp3");
        
        assert!(result.is_ok());
        assert_eq!(service.source_ids().len(), 1);
        assert!(service.get_source(id).is_some());
    }

    #[test]
    fn test_audio_domain_service_create_duplicate_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test1.mp3").unwrap();
        let result = service.create_source(id, "assets/test2.mp3");
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_destroy_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        let result = service.destroy_source(id);
        
        assert!(result.is_ok());
        assert_eq!(service.source_ids().len(), 0);
        assert!(service.get_source(id).is_none());
    }

    #[test]
    fn test_audio_domain_service_destroy_nonexistent_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        let result = service.destroy_source(id);
        
        assert!(result.is_err());
        if let Err(DomainError::Audio(AudioError::SourceNotFound(msg))) = result {
            assert!(msg.contains("1"));
        } else {
            panic!("Expected SourceNotFound error");
        }
    }

    #[test]
    fn test_audio_domain_service_play_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        let result = service.play_source(id);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_play_nonexistent_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        let result = service.play_source(id);
        
        assert!(result.is_err());
        if let Err(DomainError::Audio(AudioError::SourceNotFound(msg))) = result {
            assert!(msg.contains("1"));
        } else {
            panic!("Expected SourceNotFound error");
        }
    }

    #[test]
    fn test_audio_domain_service_stop_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        service.play_source(id).unwrap();
        let result = service.stop_source(id);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_pause_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        service.play_source(id).unwrap();
        let result = service.pause_source(id);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_resume_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        service.play_source(id).unwrap();
        service.pause_source(id).unwrap();
        let result = service.resume_source(id);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_set_source_volume() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        let volume = Volume::new(0.5).unwrap();
        let result = service.set_source_volume(id, volume);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_set_source_volume_f32() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        let result = service.set_source_volume_f32(id, 0.75);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_set_source_volume_f32_invalid() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        let result = service.set_source_volume_f32(id, 1.5);
        
        assert!(result.is_err());
        if let Err(DomainError::Audio(AudioError::InvalidVolume(value))) = result {
            assert_eq!(value, 1.5);
        } else {
            panic!("Expected InvalidVolume error");
        }
    }

    #[test]
    fn test_audio_domain_service_set_master_volume() {
        let service = AudioDomainService::new();
        
        let volume = Volume::new(0.8).unwrap();
        let result = service.set_master_volume(volume);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_set_master_volume_f32() {
        let service = AudioDomainService::new();
        
        let result = service.set_master_volume_f32(0.6);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_set_master_volume_f32_invalid() {
        let service = AudioDomainService::new();
        
        let result = service.set_master_volume_f32(-0.5);
        
        assert!(result.is_err());
        if let Err(DomainError::Audio(AudioError::InvalidVolume(value))) = result {
            assert_eq!(value, -0.5);
        } else {
            panic!("Expected InvalidVolume error");
        }
    }

    #[test]
    fn test_audio_domain_service_get_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        let source = service.get_source(id);
        
        assert!(source.is_some());
        assert_eq!(source.unwrap().id(), id);
    }

    #[test]
    fn test_audio_domain_service_get_source_mut() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").unwrap();
        let source = service.get_source_mut(id);
        
        assert!(source.is_some());
    }

    #[test]
    fn test_audio_domain_service_source_ids() {
        let service = AudioDomainService::new();
        
        service.create_source(AudioSourceId::new(1), "assets/test1.mp3").unwrap();
        service.create_source(AudioSourceId::new(2), "assets/test2.mp3").unwrap();
        service.create_source(AudioSourceId::new(3), "assets/test3.mp3").unwrap();
        
        let ids = service.source_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&AudioSourceId::new(1)));
        assert!(ids.contains(&AudioSourceId::new(2)));
        assert!(ids.contains(&AudioSourceId::new(3)));
    }

    #[test]
    fn test_audio_domain_service_playing_sources_count() {
        let service = AudioDomainService::new();
        
        service.create_source(AudioSourceId::new(1), "assets/test1.mp3").unwrap();
        service.create_source(AudioSourceId::new(2), "assets/test2.mp3").unwrap();
        service.create_source(AudioSourceId::new(3), "assets/test3.mp3").unwrap();
        
        service.play_source(AudioSourceId::new(1)).unwrap();
        service.play_source(AudioSourceId::new(2)).unwrap();
        
        assert_eq!(service.playing_sources_count(), 2);
    }

    #[test]
    fn test_audio_domain_service_stop_all_sources() {
        let service = AudioDomainService::new();
        
        service.create_source(AudioSourceId::new(1), "assets/test1.mp3").unwrap();
        service.create_source(AudioSourceId::new(2), "assets/test2.mp3").unwrap();
        service.create_source(AudioSourceId::new(3), "assets/test3.mp3").unwrap();
        
        service.play_source(AudioSourceId::new(1)).unwrap();
        service.play_source(AudioSourceId::new(2)).unwrap();
        service.play_source(AudioSourceId::new(3)).unwrap();
        
        let result = service.stop_all_sources();
        
        assert!(result.is_ok());
        assert_eq!(service.playing_sources_count(), 0);
    }

    #[test]
    fn test_audio_domain_service_update_listener() {
        let service = AudioDomainService::new();
        let listener = AudioListener::default();
        
        service.update_listener(listener.clone());
        
        assert_eq!(service.get_listener().position(), listener.position());
    }

    #[test]
    fn test_audio_domain_service_get_listener() {
        let service = AudioDomainService::new();
        
        let listener = service.get_listener();
        
        assert!(listener.position() == Vec3::ZERO);
    }
}

#[cfg(test)]
mod physics_domain_service_tests {
    use super::*;

    #[test]
    fn test_physics_domain_service_new() {
        let service = PhysicsDomainService::new();
        
        assert_eq!(service.get_world().get_world().bodies.len(), 0);
    }

    #[test]
    fn test_physics_domain_service_create_body() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        let result = service.create_body(body);
        
        assert!(result.is_ok());
        assert_eq!(service.get_world().get_world().bodies.len(), 1);
    }

    #[test]
    fn test_physics_domain_service_create_body_alias() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        let result = service.add_body(body);
        
        assert!(result.is_ok());
        assert_eq!(service.get_world().get_world().bodies.len(), 1);
    }

    #[test]
    fn test_physics_domain_service_destroy_body() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body).unwrap();
        let result = service.destroy_body(RigidBodyId::new(1));
        
        assert!(result.is_ok());
        assert_eq!(service.get_world().get_world().bodies.len(), 0);
    }

    #[test]
    fn test_physics_domain_service_destroy_nonexistent_body() {
        let service = PhysicsDomainService::new();
        
        let result = service.destroy_body(RigidBodyId::new(1));
        
        assert!(result.is_err());
        if let Err(DomainError::Physics(PhysicsError::RigidBodyNotFound { body_id, .. })) = result {
            assert!(body_id.contains("1"));
        } else {
            panic!("Expected RigidBodyNotFound error");
        }
    }

    #[test]
    fn test_physics_domain_service_create_collider() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        
        service.create_body(body).unwrap();
        let result = service.create_collider(collider, RigidBodyId::new(1));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_create_collider_alias() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        
        service.create_body(body).unwrap();
        let result = service.add_collider_to_body(collider, RigidBodyId::new(1));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_destroy_collider() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        
        service.create_body(body).unwrap();
        service.create_collider(collider, RigidBodyId::new(1)).unwrap();
        let result = service.destroy_collider(ColliderId::new(1));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_destroy_collider_alias() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        
        service.create_body(body).unwrap();
        service.create_collider(collider, RigidBodyId::new(1)).unwrap();
        let result = service.remove_collider(ColliderId::new(1));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_update_body() {
        let service = PhysicsDomainService::new();
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body.clone()).unwrap();
        body.set_position(Vec3::new(10.0, 20.0, 30.0));
        let result = service.update_body(&body);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_apply_force() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body).unwrap();
        let result = service.apply_force(RigidBodyId::new(1), Vec3::new(0.0, 100.0, 0.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_apply_impulse() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body).unwrap();
        let result = service.apply_impulse(RigidBodyId::new(1), Vec3::new(0.0, 50.0, 0.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_set_body_position() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body).unwrap();
        let result = service.set_body_position(RigidBodyId::new(1), Vec3::new(10.0, 20.0, 30.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_get_body_position() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(1.0, 2.0, 3.0));
        
        service.create_body(body).unwrap();
        let result = service.get_body_position(RigidBodyId::new(1));
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_physics_domain_service_get_body_position_nonexistent() {
        let service = PhysicsDomainService::new();
        
        let result = service.get_body_position(RigidBodyId::new(1));
        
        assert!(result.is_err());
        if let Err(DomainError::Physics(PhysicsError::RigidBodyNotFound { body_id, .. })) = result {
            assert!(body_id.contains("1"));
        } else {
            panic!("Expected RigidBodyNotFound error");
        }
    }

    #[test]
    fn test_physics_domain_service_step_simulation() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(0.0, 10.0, 0.0));
        
        service.create_body(body).unwrap();
        let result = service.step_simulation(0.016);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_step_alias() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(0.0, 10.0, 0.0));
        
        service.create_body(body).unwrap();
        let result = service.step(0.016);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_get_world() {
        let service = PhysicsDomainService::new();
        
        let _world = service.get_world();
        
        assert_eq!(world.get_world().bodies.len(), 0);
    }

    #[test]
    fn test_physics_domain_service_get_world_mut() {
        let service = PhysicsDomainService::new();
        
        let world = service.get_world_mut();
        
        assert_eq!(world.get_world().bodies.len(), 0);
    }

    #[test]
    fn test_physics_domain_service_multiple_bodies() {
        let service = PhysicsDomainService::new();
        
        service.create_body(RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO)).unwrap();
        service.create_body(RigidBody::dynamic(RigidBodyId::new(2), Vec3::new(1.0, 0.0, 0.0))).unwrap();
        service.create_body(RigidBody::dynamic(RigidBodyId::new(3), Vec3::new(2.0, 0.0, 0.0))).unwrap();
        
        assert_eq!(service.get_world().get_world().bodies.len(), 3);
    }

    #[test]
    fn test_physics_domain_service_fixed_body() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::new(RigidBodyId::new(1), RigidBodyType::Fixed, Vec3::ZERO);
        
        let result = service.create_body(body);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_kinematic_body() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::new(RigidBodyId::new(1), RigidBodyType::Kinematic, Vec3::ZERO);
        
        let result = service.create_body(body);
        
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod scene_domain_service_tests {
    use super::*;

    #[test]
    fn test_scene_domain_service_new() {
        let service = SceneDomainService::new();
        
        assert_eq!(service.scene_ids().len(), 0);
    }

    #[test]
    fn test_scene_domain_service_create_scene() {
        let service = SceneDomainService::new();
        
        let result = service.create_scene(SceneId::new(1), "test_scene");
        
        assert!(result.is_ok());
        assert_eq!(service.scene_ids().len(), 1);
        assert!(service.get_scene(SceneId::new(1)).is_some());
    }

    #[test]
    fn test_scene_domain_service_create_multiple_scenes() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "scene1").unwrap();
        service.create_scene(SceneId::new(2), "scene2").unwrap();
        service.create_scene(SceneId::new(3), "scene3").unwrap();
        
        assert_eq!(service.scene_ids().len(), 3);
    }

    #[test]
    fn test_scene_domain_service_delete_scene() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").unwrap();
        let result = service.delete_scene(SceneId::new(1));
        
        assert!(result.is_ok());
        assert_eq!(service.scene_ids().len(), 0);
        assert!(service.get_scene(SceneId::new(1)).is_none());
    }

    #[test]
    fn test_scene_domain_service_delete_nonexistent_scene() {
        let service = SceneDomainService::new();
        
        let result = service.delete_scene(SceneId::new(1));
        
        assert!(result.is_err());
        if let Err(DomainError::Scene(SceneError::SceneNotFound(id))) = result {
            assert_eq!(id, SceneId::new(1));
        } else {
            panic!("Expected SceneNotFound error");
        }
    }

    #[test]
    fn test_scene_domain_service_switch_to_scene() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "scene1").unwrap();
        service.create_scene(SceneId::new(2), "scene2").unwrap();
        
        service.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        service.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        service.get_scene_mut(SceneId::new(1)).unwrap().activate().unwrap();
        
        service.get_scene_mut(SceneId::new(2)).unwrap().load().unwrap();
        service.get_scene_mut(SceneId::new(2)).unwrap().load().unwrap();
        
        let result = service.switch_to_scene(SceneId::new(2));
        
        assert!(result.is_ok());
        assert_eq!(service.get_active_scene().unwrap().id(), SceneId::new(2));
    }

    #[test]
    fn test_scene_domain_service_switch_to_nonexistent_scene() {
        let service = SceneDomainService::new();
        
        let result = service.switch_to_scene(SceneId::new(1));
        
        assert!(result.is_err());
    }

    #[test]
    fn test_scene_domain_service_get_scene() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").unwrap();
        let scene = service.get_scene(SceneId::new(1));
        
        assert!(scene.is_some());
        assert_eq!(scene.unwrap().id(), SceneId::new(1));
    }

    #[test]
    fn test_scene_domain_service_get_scene_mut() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").unwrap();
        let scene = service.get_scene_mut(SceneId::new(1));
        
        assert!(scene.is_some());
    }

    #[test]
    fn test_scene_domain_service_get_active_scene() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").unwrap();
        service.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        service.get_scene_mut(SceneId::new(1)).unwrap().load().unwrap();
        service.get_scene_mut(SceneId::new(1)).unwrap().activate().unwrap();
        
        let active = service.get_active_scene();
        
        assert!(active.is_some());
        assert_eq!(active.unwrap().id(), SceneId::new(1));
    }

    #[test]
    fn test_scene_domain_service_get_active_scene_none() {
        let service = SceneDomainService::new();
        
        let active = service.get_active_scene();
        
        assert!(active.is_none());
    }

    #[test]
    fn test_scene_domain_service_scene_ids() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "scene1").unwrap();
        service.create_scene(SceneId::new(2), "scene2").unwrap();
        service.create_scene(SceneId::new(3), "scene3").unwrap();
        
        let ids = service.scene_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&SceneId::new(1)));
        assert!(ids.contains(&SceneId::new(2)));
        assert!(ids.contains(&SceneId::new(3)));
    }

    #[test]
    fn test_scene_domain_service_has_scene() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").unwrap();
        
        assert!(service.has_scene(SceneId::new(1)));
        assert!(!service.has_scene(SceneId::new(2)));
    }

    #[test]
    fn test_scene_domain_service_scene_count() {
        let service = SceneDomainService::new();
        
        assert_eq!(service.scene_count(), 0);
        
        service.create_scene(SceneId::new(1), "scene1").unwrap();
        assert_eq!(service.scene_count(), 1);
        
        service.create_scene(SceneId::new(2), "scene2").unwrap();
        assert_eq!(service.scene_count(), 2);
    }
}
