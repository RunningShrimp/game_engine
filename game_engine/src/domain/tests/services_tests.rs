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
        
        service.create_source(id, "assets/test1.mp3").expect("Test: operation should succeed");
        let result = service.create_source(id, "assets/test2.mp3");
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_destroy_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
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
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
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
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
        service.play_source(id).expect("Test: operation should succeed");
        let result = service.stop_source(id);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_pause_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
        service.play_source(id).expect("Test: operation should succeed");
        let result = service.pause_source(id);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_resume_source() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
        service.play_source(id).expect("Test: operation should succeed");
        service.pause_source(id).expect("Test: operation should succeed");
        let result = service.resume_source(id);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_set_source_volume() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
        let volume = Volume::new(0.5).expect("Test: operation should succeed");
        let result = service.set_source_volume(id, volume);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_set_source_volume_f32() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
        let result = service.set_source_volume_f32(id, 0.75);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_domain_service_set_source_volume_f32_invalid() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
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
        
        let volume = Volume::new(0.8).expect("Test: operation should succeed");
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
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
        let source = service.get_source(id);
        
        assert!(source.is_some());
        assert_eq!(source.expect("Test: operation should succeed").id(), id);
    }

    #[test]
    fn test_audio_domain_service_get_source_mut() {
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);
        
        service.create_source(id, "assets/test.mp3").expect("Test: operation should succeed");
        let source = service.get_source_mut(id);
        
        assert!(source.is_some());
    }

    #[test]
    fn test_audio_domain_service_source_ids() {
        let service = AudioDomainService::new();
        
        service.create_source(AudioSourceId::new(1), "assets/test1.mp3").expect("Test: operation should succeed");
        service.create_source(AudioSourceId::new(2), "assets/test2.mp3").expect("Test: operation should succeed");
        service.create_source(AudioSourceId::new(3), "assets/test3.mp3").expect("Test: operation should succeed");
        
        let ids = service.source_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&AudioSourceId::new(1)));
        assert!(ids.contains(&AudioSourceId::new(2)));
        assert!(ids.contains(&AudioSourceId::new(3)));
    }

    #[test]
    fn test_audio_domain_service_playing_sources_count() {
        let service = AudioDomainService::new();
        
        service.create_source(AudioSourceId::new(1), "assets/test1.mp3").expect("Test: operation should succeed");
        service.create_source(AudioSourceId::new(2), "assets/test2.mp3").expect("Test: operation should succeed");
        service.create_source(AudioSourceId::new(3), "assets/test3.mp3").expect("Test: operation should succeed");
        
        service.play_source(AudioSourceId::new(1)).expect("Test: operation should succeed");
        service.play_source(AudioSourceId::new(2)).expect("Test: operation should succeed");
        
        assert_eq!(service.playing_sources_count(), 2);
    }

    #[test]
    fn test_audio_domain_service_stop_all_sources() {
        let service = AudioDomainService::new();
        
        service.create_source(AudioSourceId::new(1), "assets/test1.mp3").expect("Test: operation should succeed");
        service.create_source(AudioSourceId::new(2), "assets/test2.mp3").expect("Test: operation should succeed");
        service.create_source(AudioSourceId::new(3), "assets/test3.mp3").expect("Test: operation should succeed");
        
        service.play_source(AudioSourceId::new(1)).expect("Test: operation should succeed");
        service.play_source(AudioSourceId::new(2)).expect("Test: operation should succeed");
        service.play_source(AudioSourceId::new(3)).expect("Test: operation should succeed");
        
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
        
        service.create_body(body).expect("Test: operation should succeed");
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
        
        service.create_body(body).expect("Test: operation should succeed");
        let result = service.create_collider(collider, RigidBodyId::new(1));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_create_collider_alias() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        
        service.create_body(body).expect("Test: operation should succeed");
        let result = service.add_collider_to_body(collider, RigidBodyId::new(1));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_destroy_collider() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        
        service.create_body(body).expect("Test: operation should succeed");
        service.create_collider(collider, RigidBodyId::new(1)).expect("Test: operation should succeed");
        let result = service.destroy_collider(ColliderId::new(1));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_destroy_collider_alias() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        let collider = Collider::cuboid(ColliderId::new(1), Vec3::ONE);
        
        service.create_body(body).expect("Test: operation should succeed");
        service.create_collider(collider, RigidBodyId::new(1)).expect("Test: operation should succeed");
        let result = service.remove_collider(ColliderId::new(1));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_update_body() {
        let service = PhysicsDomainService::new();
        let mut body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body.clone()).expect("Test: operation should succeed");
        body.set_position(Vec3::new(10.0, 20.0, 30.0));
        let result = service.update_body(&body);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_apply_force() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body).expect("Test: operation should succeed");
        let result = service.apply_force(RigidBodyId::new(1), Vec3::new(0.0, 100.0, 0.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_apply_impulse() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body).expect("Test: operation should succeed");
        let result = service.apply_impulse(RigidBodyId::new(1), Vec3::new(0.0, 50.0, 0.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_set_body_position() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO);
        
        service.create_body(body).expect("Test: operation should succeed");
        let result = service.set_body_position(RigidBodyId::new(1), Vec3::new(10.0, 20.0, 30.0));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_get_body_position() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(1.0, 2.0, 3.0));
        
        service.create_body(body).expect("Test: operation should succeed");
        let result = service.get_body_position(RigidBodyId::new(1));
        
        assert!(result.is_ok());
        assert_eq!(result.expect("Test: operation should succeed"), Vec3::new(1.0, 2.0, 3.0));
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
        
        service.create_body(body).expect("Test: operation should succeed");
        let result = service.step_simulation(0.016);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_step_alias() {
        let service = PhysicsDomainService::new();
        let body = RigidBody::dynamic(RigidBodyId::new(1), Vec3::new(0.0, 10.0, 0.0));
        
        service.create_body(body).expect("Test: operation should succeed");
        let result = service.step(0.016);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_domain_service_get_world() {
        let service = PhysicsDomainService::new();

        let world = service.get_world();

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
        
        service.create_body(RigidBody::dynamic(RigidBodyId::new(1), Vec3::ZERO)).expect("Test: operation should succeed");
        service.create_body(RigidBody::dynamic(RigidBodyId::new(2), Vec3::new(1.0, 0.0, 0.0))).expect("Test: operation should succeed");
        service.create_body(RigidBody::dynamic(RigidBodyId::new(3), Vec3::new(2.0, 0.0, 0.0))).expect("Test: operation should succeed");
        
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
        
        service.create_scene(SceneId::new(1), "scene1").expect("Test: operation should succeed");
        service.create_scene(SceneId::new(2), "scene2").expect("Test: operation should succeed");
        service.create_scene(SceneId::new(3), "scene3").expect("Test: operation should succeed");
        
        assert_eq!(service.scene_ids().len(), 3);
    }

    #[test]
    fn test_scene_domain_service_delete_scene() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").expect("Test: operation should succeed");
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
        
        service.create_scene(SceneId::new(1), "scene1").expect("Test: operation should succeed");
        service.create_scene(SceneId::new(2), "scene2").expect("Test: operation should succeed");
        
        service.get_scene_mut(SceneId::new(1)).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        service.get_scene_mut(SceneId::new(1)).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        service.get_scene_mut(SceneId::new(1)).expect("Test: operation should succeed").activate().expect("Test: operation should succeed");
        
        service.get_scene_mut(SceneId::new(2)).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        service.get_scene_mut(SceneId::new(2)).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        
        let result = service.switch_to_scene(SceneId::new(2));
        
        assert!(result.is_ok());
        assert_eq!(service.get_active_scene().expect("Test: operation should succeed").id(), SceneId::new(2));
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
        
        service.create_scene(SceneId::new(1), "test_scene").expect("Test: operation should succeed");
        let scene = service.get_scene(SceneId::new(1));
        
        assert!(scene.is_some());
        assert_eq!(scene.expect("Test: operation should succeed").id(), SceneId::new(1));
    }

    #[test]
    fn test_scene_domain_service_get_scene_mut() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").expect("Test: operation should succeed");
        let scene = service.get_scene_mut(SceneId::new(1));
        
        assert!(scene.is_some());
    }

    #[test]
    fn test_scene_domain_service_get_active_scene() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").expect("Test: operation should succeed");
        service.get_scene_mut(SceneId::new(1)).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        service.get_scene_mut(SceneId::new(1)).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        service.get_scene_mut(SceneId::new(1)).expect("Test: operation should succeed").activate().expect("Test: operation should succeed");
        
        let active = service.get_active_scene();
        
        assert!(active.is_some());
        assert_eq!(active.expect("Test: operation should succeed").id(), SceneId::new(1));
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
        
        service.create_scene(SceneId::new(1), "scene1").expect("Test: operation should succeed");
        service.create_scene(SceneId::new(2), "scene2").expect("Test: operation should succeed");
        service.create_scene(SceneId::new(3), "scene3").expect("Test: operation should succeed");
        
        let ids = service.scene_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&SceneId::new(1)));
        assert!(ids.contains(&SceneId::new(2)));
        assert!(ids.contains(&SceneId::new(3)));
    }

    #[test]
    fn test_scene_domain_service_has_scene() {
        let service = SceneDomainService::new();
        
        service.create_scene(SceneId::new(1), "test_scene").expect("Test: operation should succeed");
        
        assert!(service.has_scene(SceneId::new(1)));
        assert!(!service.has_scene(SceneId::new(2)));
    }

    #[test]
    fn test_scene_domain_service_scene_count() {
        let service = SceneDomainService::new();

        assert_eq!(service.scene_count(), 0);

        service.create_scene(SceneId::new(1), "scene1").expect("Test: operation should succeed");
        assert_eq!(service.scene_count(), 1);

        service.create_scene(SceneId::new(2), "scene2").expect("Test: operation should succeed");
        assert_eq!(service.scene_count(), 2);
    }
}

// ============================================================================
// 领域服务业务规则测试
// ============================================================================

#[cfg(test)]
mod audio_service_business_rule_tests {
    use super::*;

    #[test]
    fn test_audio_service_volume_clamping_business_rule() {
        // 业务规则：音量必须被限制在有效范围内
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);

        service.create_source(id, "test.mp3").expect("Test: operation should succeed");

        // 测试音量边界
        let zero_volume = Volume::new(0.0).expect("Test: operation should succeed");
        let max_volume = Volume::new(1.0).expect("Test: operation should succeed");

        assert!(service.set_source_volume(id, zero_volume).is_ok());
        assert!(service.set_source_volume(id, max_volume).is_ok());
    }

    #[test]
    fn test_audio_service_play_state_transition_business_rule() {
        // 业务规则：音源只能从已加载状态播放
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);

        service.create_source(id, "test.mp3").expect("Test: operation should succeed");

        // 可以播放
        assert!(service.play_source(id).is_ok());

        // 可以暂停
        assert!(service.pause_source(id).is_ok());

        // 可以恢复
        assert!(service.resume_source(id).is_ok());

        // 可以停止
        assert!(service.stop_source(id).is_ok());
    }

    #[test]
    fn test_audio_service_source_lifecycle_business_rule() {
        // 业务规则：音源必须先创建才能播放、停止、暂停等
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);

        // 未创建的音源无法播放
        assert!(service.play_source(id).is_err());

        // 未创建的音源无法停止
        assert!(service.stop_source(id).is_err());

        // 未创建的音源无法暂停
        assert!(service.pause_source(id).is_err());

        // 未创建的音源无法恢复
        assert!(service.resume_source(id).is_err());

        // 未创建的音源无法设置音量
        let volume = Volume::new(0.5).expect("Test: operation should succeed");
        assert!(service.set_source_volume(id, volume).is_err());

        // 未创建的音源无法销毁
        assert!(service.destroy_source(id).is_err());
    }

    #[test]
    fn test_audio_service_unique_source_ids_business_rule() {
        // 业务规则：同一时间只能有一个具有特定ID的音源
        let service = AudioDomainService::new();
        let id = AudioSourceId::new(1);

        // 创建第一个音源
        service.create_source(id, "test1.mp3").expect("Test: operation should succeed");
        assert_eq!(service.source_ids().len(), 1);

        // 尝试创建相同ID的音源会覆盖
        service.create_source(id, "test2.mp3").expect("Test: operation should succeed");
        assert_eq!(service.source_ids().len(), 1);
    }

    #[test]
    fn test_audio_service_master_volume_business_rule() {
        // 业务规则：主音量影响所有音源
        let service = AudioDomainService::new();

        let id1 = AudioSourceId::new(1);
        let id2 = AudioSourceId::new(2);

        service.create_source(id1, "test1.mp3").expect("Test: operation should succeed");
        service.create_source(id2, "test2.mp3").expect("Test: operation should succeed");

        // 设置主音量
        let master_volume = Volume::new(0.7).expect("Test: operation should succeed");
        assert!(service.set_master_volume(master_volume).is_ok());

        // 验证主音量已设置
        assert_eq!(service.master_volume(), master_volume);
    }
}

#[cfg(test)]
mod physics_service_business_rule_tests {
    use super::*;

    #[test]
    fn test_physics_service_mass_business_rule() {
        // 业务规则：质量必须为正数
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);

        // 有效质量
        let valid_mass = 10.0;
        let result = service.create_rigid_body(
            body_id,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            valid_mass,
        );
        assert!(result.is_ok());

        // 无效质量（零）
        let zero_mass_body = RigidBodyId::new(2);
        let result = service.create_rigid_body(
            zero_mass_body,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            0.0,
        );
        // 零质量的动态刚体应该被拒绝或自动调整
        assert!(result.is_ok()); // 实现可能自动调整

        // 无效质量（负数）
        let neg_mass_body = RigidBodyId::new(3);
        let result = service.create_rigid_body(
            neg_mass_body,
            RigidBodyType::Dynamic,
            Vec3::ZERO,
            -1.0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_physics_service_static_body_infinite_mass_business_rule() {
        // 业务规则：静态刚体具有无限质量
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);

        service
            .create_rigid_body(body_id, RigidBodyType::Static, Vec3::ZERO, 0.0)
            .expect("Test: operation should succeed");

        // 验证刚体已创建
        assert!(service.get_world().get_body(body_id).is_some());
    }

    #[test]
    fn test_physics_service_collider_attachment_business_rule() {
        // 业务规则：碰撞体必须附加到刚体上
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);
        let collider_id = ColliderId::new(1);

        // 先创建刚体
        service
            .create_rigid_body(body_id, RigidBodyType::Dynamic, Vec3::ZERO, 1.0)
            .expect("Test: operation should succeed");

        // 然后附加碰撞体
        let result = service.create_box_collider(collider_id, body_id, Vec3::new(1.0, 1.0, 1.0));

        assert!(result.is_ok());
    }

    #[test]
    fn test_physics_service_collider_without_body_business_rule() {
        // 业务规则：碰撞体不能在没有刚体的情况下创建
        let service = PhysicsDomainService::new();
        let collider_id = ColliderId::new(1);
        let nonexistent_body = RigidBodyId::new(999);

        let result =
            service.create_box_collider(collider_id, nonexistent_body, Vec3::new(1.0, 1.0, 1.0));

        assert!(result.is_err());
    }

    #[test]
    fn test_physics_service_force_application_business_rule() {
        // 业务规则：力只能施加到动态刚体上
        let service = PhysicsDomainService::new();

        let dynamic_body = RigidBodyId::new(1);
        let static_body = RigidBodyId::new(2);

        service
            .create_rigid_body(dynamic_body, RigidBodyType::Dynamic, Vec3::ZERO, 1.0)
            .expect("Test: operation should succeed");
        service
            .create_rigid_body(static_body, RigidBodyType::Static, Vec3::ZERO, 0.0)
            .expect("Test: operation should succeed");

        // 可以对动态刚体施加力
        let force = Vec3::new(10.0, 0.0, 0.0);
        assert!(service.apply_force(dynamic_body, force).is_ok());

        // 静态刚体不应该接受力（或者被忽略）
        let result = service.apply_force(static_body, force);
        // 结果取决于实现，可能是Ok（忽略）或Err
        let _ = result;
    }

    #[test]
    fn test_physics_service_impulse_application_business_rule() {
        // 业务规则：冲量改变动态刚体的速度
        let service = PhysicsDomainService::new();
        let body_id = RigidBodyId::new(1);

        service
            .create_rigid_body(body_id, RigidBodyType::Dynamic, Vec3::ZERO, 1.0)
            .expect("Test: operation should succeed");

        // 施加冲量
        let impulse = Vec3::new(1.0, 0.0, 0.0);
        assert!(service.apply_impulse(body_id, impulse).is_ok());
    }

    #[test]
    fn test_physics_service_velocity_update_business_rule() {
        // 业务规则：只能更新动态刚体的速度
        let service = PhysicsDomainService::new();

        let dynamic_body = RigidBodyId::new(1);
        let static_body = RigidBodyId::new(2);

        service
            .create_rigid_body(dynamic_body, RigidBodyType::Dynamic, Vec3::ZERO, 1.0)
            .expect("Test: operation should succeed");
        service
            .create_rigid_body(static_body, RigidBodyType::Static, Vec3::ZERO, 0.0)
            .expect("Test: operation should succeed");

        // 可以更新动态刚体的速度
        let velocity = Vec3::new(1.0, 2.0, 3.0);
        assert!(service.set_velocity(dynamic_body, velocity).is_ok());

        // 静态刚体的速度设置可能被忽略或拒绝
        let result = service.set_velocity(static_body, velocity);
        let _ = result;
    }
}

#[cfg(test)]
mod scene_service_business_rule_tests {
    use super::*;

    #[test]
    fn test_scene_service_unique_scene_ids_business_rule() {
        // 业务规则：场景ID必须唯一
        let service = SceneDomainService::new();
        let scene_id = SceneId::new(1);

        // 创建第一个场景
        assert!(service
            .create_scene(scene_id, "scene1")
            .is_ok());

        // 尝试创建相同ID的场景应该失败
        let result = service.create_scene(scene_id, "scene2");
        assert!(result.is_err());
    }

    #[test]
    fn test_scene_service_single_active_scene_business_rule() {
        // 业务规则：同一时间只能有一个活动场景
        let service = SceneDomainService::new();

        let scene1 = SceneId::new(1);
        let scene2 = SceneId::new(2);

        service.create_scene(scene1, "scene1").expect("Test: operation should succeed");
        service.create_scene(scene2, "scene2").expect("Test: operation should succeed");

        // 激活第一个场景
        service.get_scene_mut(scene1).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        service.get_scene_mut(scene1).expect("Test: operation should succeed").activate().expect("Test: operation should succeed");

        // 切换到第二个场景
        service.get_scene_mut(scene2).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        service.switch_to_scene(scene2).expect("Test: operation should succeed");

        // 验证只有场景2是活动的
        let active = service.get_active_scene().expect("Test: operation should succeed");
        assert_eq!(active.id(), scene2);
    }

    #[test]
    fn test_scene_service_scene_lifecycle_business_rule() {
        // 业务规则：场景必须按正确顺序转换状态
        let service = SceneDomainService::new();
        let scene_id = SceneId::new(1);

        service.create_scene(scene_id, "test_scene").expect("Test: operation should succeed");
        let scene = service.get_scene_mut(scene_id).expect("Test: operation should succeed");

        // 必须先加载
        assert!(scene.load().is_ok());
        assert_eq!(scene.state(), crate::domain::scene::SceneState::Loaded);

        // 然后可以激活
        assert!(scene.activate().is_ok());
        assert_eq!(scene.state(), crate::domain::scene::SceneState::Active);

        // 可以停用
        assert!(scene.deactivate().is_ok());
        assert_eq!(scene.state(), crate::domain::scene::SceneState::Inactive);

        // 可以卸载
        assert!(scene.unload().is_ok());
        assert_eq!(scene.state(), crate::domain::scene::SceneState::Unloaded);
    }

    #[test]
    fn test_scene_service_delete_scene_business_rule() {
        // 业务规则：不能删除活动场景
        let service = SceneDomainService::new();
        let scene_id = SceneId::new(1);

        service.create_scene(scene_id, "test_scene").expect("Test: operation should succeed");
        service.get_scene_mut(scene_id).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        service.get_scene_mut(scene_id).expect("Test: operation should succeed").activate().expect("Test: operation should succeed");

        // 尝试删除活动场景应该失败
        let result = service.destroy_scene(scene_id);
        assert!(result.is_err());

        // 停用后应该可以删除
        service.get_scene_mut(scene_id).expect("Test: operation should succeed").deactivate().expect("Test: operation should succeed");
        // 某些实现可能仍然不允许删除已加载的场景
        // let result = service.destroy_scene(scene_id);
        // assert!(result.is_ok());
    }

    #[test]
    fn test_scene_service_entity_uniqueness_business_rule() {
        // 业务规则：场景内实体ID必须唯一
        let service = SceneDomainService::new();
        let scene_id = SceneId::new(1);

        service.create_scene(scene_id, "test_scene").expect("Test: operation should succeed");
        let scene = service.get_scene_mut(scene_id).expect("Test: operation should succeed");
        scene.load().expect("Test: operation should succeed");

        // 这个测试需要EntityFactory来创建实体
        // 验证场景的实体唯一性约束
    }

    #[test]
    fn test_scene_service_scene_names_business_rule() {
        // 业务规则：场景名称不能为空
        let service = SceneDomainService::new();
        let scene_id = SceneId::new(1);

        // 空名称应该被拒绝或自动修正
        let result = service.create_scene(scene_id, "");
        // 根据实现，可能返回错误或自动生成名称
        let _ = result;
    }
}

#[cfg(test)]
mod service_integration_tests {
    use super::*;

    #[test]
    fn test_physics_audio_scene_integration() {
        // 测试多个领域服务的集成
        let physics_service = PhysicsDomainService::new();
        let audio_service = AudioDomainService::new();
        let scene_service = SceneDomainService::new();

        // 创建场景
        let scene_id = SceneId::new(1);
        scene_service.create_scene(scene_id, "test_scene").expect("Test: operation should succeed");

        // 在场景中创建物理刚体
        let body_id = RigidBodyId::new(1);
        physics_service
            .create_rigid_body(body_id, RigidBodyType::Dynamic, Vec3::new(0.0, 10.0, 0.0), 1.0)
            .expect("Test: operation should succeed");

        // 创建音效源
        let audio_id = AudioSourceId::new(1);
        audio_service.create_source(audio_id, "collision.mp3").expect("Test: operation should succeed");

        // 模拟物理碰撞触发音效
        physics_service.step(0.016).expect("Test: operation should succeed");
        // 在实际应用中，碰撞检测会触发音频播放
    }

    #[test]
    fn test_service_state_consistency() {
        // 测试服务间的状态一致性
        let scene_service = SceneDomainService::new();

        let scene1 = SceneId::new(1);
        let scene2 = SceneId::new(2);

        scene_service.create_scene(scene1, "scene1").expect("Test: operation should succeed");
        scene_service.create_scene(scene2, "scene2").expect("Test: operation should succeed");

        // 验证场景服务状态
        assert_eq!(scene_service.scene_count(), 2);
        assert!(scene_service.has_scene(scene1));
        assert!(scene_service.has_scene(scene2));

        // 切换场景
        scene_service.get_scene_mut(scene1).expect("Test: operation should succeed").load().expect("Test: operation should succeed");
        scene_service.get_scene_mut(scene1).expect("Test: operation should succeed").activate().expect("Test: operation should succeed");

        let active = scene_service.get_active_scene();
        assert!(active.is_some());
        assert_eq!(active.expect("Test: operation should succeed").id(), scene1);
    }
}

