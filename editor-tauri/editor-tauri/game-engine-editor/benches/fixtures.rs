// Test Fixtures and Utilities
//
// Common test data, mock objects, and utilities for benchmarks

use game_engine_editor_lib::{
    asset_manager::AssetManager,
    entity_manager::EntityManager,
    performance_monitor::{PerformanceMetrics, PerformanceMonitor},
    animation_system::AnimationSystem,
    behavior_tree::{BehaviorTree, BehaviorNode, NodeStatus},
};
use glam::{Mat4, Vec3, Quat};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Creates test entities for benchmarking
pub fn create_test_entities(count: usize) -> Vec<game_engine_editor_lib::entity_manager::Entity> {
    (0..count)
        .map(|i| game_engine_editor_lib::entity_manager::Entity {
            id: Uuid::new_v4(),
            name: format!("TestEntity_{}", i),
            transform: game_engine_editor_lib::entity_manager::Transform {
                position: Vec3::new(
                    (i as f32 * 10.0) % 1000.0,
                    ((i as f32 * 10.0) / 1000.0).floor() * 10.0,
                    0.0,
                ),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            parent_id: None,
            children: Vec::new(),
            components: Vec::new(),
        })
        .collect()
}

/// Creates a test frustum for culling benchmarks
pub fn create_test_frustum() -> game_engine_editor_lib::camera::Camera {
    game_engine_editor_lib::camera::Camera {
        position: Vec3::ZERO,
        target: Vec3::new(0.0, 0.0, -1.0),
        up: Vec3::Y,
        fov: 60.0,
        aspect_ratio: 16.0 / 9.0,
        near: 0.1,
        far: 1000.0,
        projection_matrix: Mat4::IDENTITY,
        view_matrix: Mat4::IDENTITY,
    }
}

/// Creates test geometry for rendering benchmarks
pub fn create_test_geometry(vertex_count: usize) -> game_engine_editor_lib::geometry::Geometry {
    let vertices = (0..vertex_count)
        .map(|i| {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (vertex_count as f32);
            game_engine_editor_lib::geometry::Vertex {
                position: Vec3::new(theta.cos(), theta.sin(), 0.0),
                normal: Vec3::Z,
                uv_coords: [0.0, 0.0],
            }
        })
        .collect();

    let indices = (0..vertex_count).collect();

    game_engine_editor_lib::geometry::Geometry {
        vertices,
        indices,
        material_id: None,
    }
}

/// Creates test commands for undo/redo benchmarks
pub fn create_test_commands(count: usize) -> Vec<game_engine_editor_lib::performance_commands::Command> {
    (0..count)
        .map(|i| game_engine_editor_lib::performance_commands::Command {
            id: Uuid::new_v4(),
            name: format!("TestCommand_{}", i),
            execute_data: json!({
                "action": "create_entity",
                "entity_id": Uuid::new_v4(),
                "position": [i as f32 * 10.0, 0.0, 0.0]
            }),
            undo_data: json!({
                "action": "delete_entity",
                "entity_id": Uuid::new_v4()
            }),
        })
        .collect()
}

/// Creates a test behavior tree
pub fn create_test_behavior_tree(depth: usize) -> BehaviorTree {
    let root = if depth == 0 {
        BehaviorNode::Sequence { children: Vec::new() }
    } else {
        BehaviorNode::Sequence {
            children: (0..3)
                .map(|_| create_test_behavior_node(depth - 1))
                .collect(),
        }
    };

    BehaviorTree {
        root,
        blackboard: serde_json::Map::new(),
    }
}

fn create_test_behavior_node(depth: usize) -> BehaviorNode {
    if depth == 0 {
        BehaviorNode::Action {
            name: "leaf_action".to_string(),
            executor: Arc::new(|_| NodeStatus::Success),
        }
    } else {
        BehaviorNode::Selector {
            children: (0..2)
                .map(|_| create_test_behavior_node(depth - 1))
                .collect(),
        }
    }
}

/// Creates test animation data
pub fn create_test_animation_data(frame_count: usize, bone_count: usize) -> game_engine_editor_lib::animation_system::AnimationClip {
    game_engine_editor_lib::animation_system::AnimationClip {
        name: "test_animation".to_string(),
        duration: frame_count as f32 / 60.0,
        tracks: (0..bone_count)
            .map(|i| game_engine_editor_lib::animation_system::AnimationTrack {
                bone_name: format!("bone_{}", i),
                position_keys: (0..frame_count)
                    .map(|f| (f as f32 / 60.0, Vec3::ZERO))
                    .collect(),
                rotation_keys: (0..frame_count)
                    .map(|f| (f as f32 / 60.0, Quat::IDENTITY))
                    .collect(),
                scale_keys: (0..frame_count)
                    .map(|f| (f as f32 / 60.0, Vec3::ONE))
                    .collect(),
            })
            .collect(),
    }
}

/// Creates test performance metrics
pub fn create_test_metrics() -> PerformanceMetrics {
    PerformanceMetrics {
        fps: 60.0,
        frame_time_ms: 16.67,
        gpu_memory_mb: 512.0,
        draw_calls: 100,
        vertex_count: 50_000,
        triangle_count: 25_000,
        texture_count: 10,
        shader_count: 5,
        entity_count: 100,
        light_count: 4,
        particle_system_count: 2,
        animation_count: 3,
        physics_objects: 50,
        audio_sources: 5,
        timestamp: chrono::Utc::now(),
    }
}

/// Mock GPU adapter for testing
pub struct MockGPUAdapter {
    pub memory_limit: usize,
    pub allocated_memory: usize,
}

impl MockGPUAdapter {
    pub fn new(memory_limit: usize) -> Self {
        Self {
            memory_limit,
            allocated_memory: 0,
        }
    }

    pub fn allocate(&mut self, size: usize) -> bool {
        if self.allocated_memory + size <= self.memory_limit {
            self.allocated_memory += size;
            true
        } else {
            false
        }
    }

    pub fn deallocate(&mut self, size: usize) {
        self.allocated_memory = self.allocated_memory.saturating_sub(size);
    }

    pub fn available_memory(&self) -> usize {
        self.memory_limit - self.allocated_memory
    }

    pub fn utilization_pct(&self) -> f64 {
        (self.allocated_memory as f64 / self.memory_limit as f64) * 100.0
    }
}

/// Performance measurement helper
pub struct PerformanceTimer {
    start: std::time::Instant,
}

impl PerformanceTimer {
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    pub fn elapsed_micros(&self) -> u128 {
        self.start.elapsed().as_micros()
    }

    pub fn elapsed_millis(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

impl Default for PerformanceTimer {
    fn default() -> Self {
        Self::start()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_entities() {
        let entities = create_test_entities(100);
        assert_eq!(entities.len(), 100);
        assert_ne!(entities[0].id, entities[1].id);
    }

    #[test]
    fn test_mock_gpu_adapter() {
        let mut adapter = MockGPUAdapter::new(1024);
        assert!(adapter.allocate(512));
        assert!(!adapter.allocate(600));
        assert_eq!(adapter.available_memory(), 512);
        assert_eq!(adapter.utilization_pct(), 50.0);
    }
}
