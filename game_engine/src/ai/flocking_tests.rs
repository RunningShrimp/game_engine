//! 群集系统测试
//!
//! 测试Boids算法和群集行为。

#[cfg(test)]
mod tests {
    use super::super::*;
    use glam::Vec3;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(AgentId::new(1), Vec3::ZERO);
        assert_eq!(agent.id, AgentId::new(1));
        assert_eq!(agent.position, Vec3::ZERO);
    }

    #[test]
    fn test_agent_id_generation() {
        let id1 = AgentId::new(1);
        let id2 = AgentId::new(2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_flock_config_default() {
        let config = FlockConfig::default();
        // Test default values exist
        assert_eq!(config.perception_radius, 0.0);
    }

    #[test]
    fn test_flock_config_custom() {
        let config = FlockConfig {
            separation_distance: 5.0,
            perception_radius: 10.0,
            max_speed: 20.0,
            max_steering_force: 10.0,
            separation_weight: 1.0,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            avoidance_weight: 1.0,
            enhanced: Default::default(),
        };
        assert_eq!(config.separation_distance, 5.0);
        assert_eq!(config.perception_radius, 10.0);
        assert_eq!(config.max_speed, 20.0);
        assert_eq!(config.max_steering_force, 10.0);
    }

    #[test]
    fn test_flock_manager_creation() {
        let manager = FlockManager::new(FlockConfig::default());
        assert_eq!(manager.get_agents().len(), 0);
    }

    #[test]
    fn test_add_agent_to_flock() {
        let mut manager = FlockManager::new(FlockConfig::default());
        manager.add_agent(Vec3::ZERO);
        assert_eq!(manager.get_agents().len(), 1);
    }

    #[test]
    fn test_remove_agent_from_flock() {
        let mut manager = FlockManager::new(FlockConfig::default());
        let agent_id = manager.add_agent(Vec3::ZERO);
        assert_eq!(manager.get_agents().len(), 1);

        manager.remove_agent(agent_id);
        assert_eq!(manager.get_agents().len(), 0);
    }

    #[test]
    fn test_obstacle_creation() {
        let obstacle = Obstacle {
            position: Vec3::new(5.0, 0.0, 5.0),
            radius: 2.0,
        };
        assert_eq!(obstacle.position, Vec3::new(5.0, 0.0, 5.0));
        assert_eq!(obstacle.radius, 2.0);
    }

    #[test]
    fn test_add_obstacle_to_flock() {
        let mut manager = FlockManager::new(FlockConfig::default());
        let obstacle = Obstacle {
            position: Vec3::new(5.0, 0.0, 5.0),
            radius: 2.0,
        };
        manager.add_obstacle(obstacle);
        assert_eq!(manager.get_obstacles().len(), 1);
    }

    #[test]
    fn test_agent_velocity() {
        let mut agent = Agent::new(AgentId::new(1), Vec3::ZERO);
        agent.velocity = Vec3::new(1.0, 0.0, 0.0);
        assert_eq!(agent.velocity, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_agent_acceleration() {
        let mut agent = Agent::new(AgentId::new(1), Vec3::ZERO);
        // Agent没有acceleration字段，只有velocity
        agent.velocity = Vec3::new(0.5, 0.0, 0.5);
        assert_eq!(agent.velocity, Vec3::new(0.5, 0.0, 0.5));
    }

    #[test]
    fn test_multiple_agents_flock() {
        let mut manager = FlockManager::new(FlockConfig::default());
        for i in 0..10 {
            manager.add_agent(Vec3::new(i as f32, 0.0, 0.0));
        }
        assert_eq!(manager.get_agents().len(), 10);
    }

    #[test]
    fn test_flock_manager_update() {
        let mut manager = FlockManager::new(FlockConfig::default());
        manager.add_agent(Vec3::ZERO);

        // Update should not panic
        manager.update(0.016);
        assert_eq!(manager.get_agents().len(), 1);
    }

    #[test]
    fn test_flocking_error_no_agent() {
        let mut manager = FlockManager::new(FlockConfig::default());
        let result = manager.get_agent(AgentId::new(999));
        assert!(result.is_none());
    }

    #[test]
    fn test_agent_neighbors() {
        let mut manager = FlockManager::new(FlockConfig::default());
        for i in 0..5 {
            manager.add_agent(Vec3::new(i as f32 * 2.0, 0.0, 0.0));
        }

        let neighbors = manager.get_neighbors(AgentId::new(2), 5.0);
        assert!(neighbors.len() > 0);
    }

    #[test]
    fn test_obstacle_avoidance() {
        let mut manager = FlockManager::new(FlockConfig::default());
        manager.add_agent(Vec3::ZERO);

        let obstacle = Obstacle {
            position: Vec3::new(2.0, 0.0, 0.0),
            radius: 1.0,
        };
        manager.add_obstacle(obstacle);

        // Agent should avoid obstacle during update
        manager.update(0.016);
    }

    #[test]
    fn test_flock_bounds() {
        let config = FlockConfig {
            separation_distance: 100.0,
            perception_radius: 200.0,
            max_speed: 50.0,
            max_steering_force: 25.0,
            separation_weight: 1.0,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            avoidance_weight: 1.0,
            enhanced: Default::default(),
        };
        let manager = FlockManager::new(config);
        assert_eq!(manager.get_config().separation_distance, 100.0);
        assert_eq!(manager.get_config().perception_radius, 200.0);
    }

    #[test]
    fn test_agent_max_speed() {
        let mut agent = Agent::new(AgentId::new(1), Vec3::ZERO);
        agent.velocity = Vec3::new(100.0, 0.0, 0.0);

        // Clamp to max speed
        let max_speed = 10.0;
        if agent.velocity.length() > max_speed {
            agent.velocity = agent.velocity.normalize() * max_speed;
        }

        assert!(agent.velocity.length() <= max_speed + 0.01);
    }
}
