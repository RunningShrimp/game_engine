//! # Mock Objects for Testing
//!
//! 提供模拟对象和桩实现。

use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 模拟渲染设备
#[derive(Debug, Clone)]
pub struct MockRenderDevice {
    pub frame_count: Arc<Mutex<usize>>,
    pub vertex_count: Arc<Mutex<usize>>,
    pub draw_calls: Arc<Mutex<usize>>,
}

impl MockRenderDevice {
    pub fn new() -> Self {
        Self {
            frame_count: Arc::new(Mutex::new(0)),
            vertex_count: Arc::new(Mutex::new(0)),
            draw_calls: Arc::new(Mutex::new(0)),
        }
    }

    pub fn begin_frame(&self) {
        let mut count = self.frame_count.lock().unwrap();
        *count += 1;
    }

    pub fn draw(&self, vertices: usize) {
        let mut vertex_count = self.vertex_count.lock().unwrap();
        let mut draw_calls = self.draw_calls.lock().unwrap();
        *vertex_count += vertices;
        *draw_calls += 1;
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        let frames = *self.frame_count.lock().unwrap();
        let vertices = *self.vertex_count.lock().unwrap();
        let calls = *self.draw_calls.lock().unwrap();
        (frames, vertices, calls)
    }
}

impl Default for MockRenderDevice {
    fn default() -> Self {
        Self::new()
    }
}

/// 模拟物理世界
#[derive(Debug)]
pub struct MockPhysicsWorld {
    pub bodies: Arc<Mutex<Vec<MockRigidBody>>>,
    pub gravity: Arc<Mutex<glm::Vec3>>,
}

#[derive(Debug, Clone)]
pub struct MockRigidBody {
    pub id: usize,
    pub position: glm::Vec3,
    pub velocity: glm::Vec3,
    pub mass: f32,
}

impl MockPhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: Arc::new(Mutex::new(Vec::new())),
            gravity: Arc::new(Mutex::new(glm::Vec3::new(0.0, -9.81, 0.0))),
        }
    }

    pub fn add_body(&self, position: glm::Vec3, mass: f32) -> usize {
        let mut bodies = self.bodies.lock().unwrap();
        let id = bodies.len();
        bodies.push(MockRigidBody {
            id,
            position,
            velocity: glm::Vec3::new(0.0, 0.0, 0.0),
            mass,
        });
        id
    }

    pub fn step(&self, dt: Duration) {
        let gravity = *self.gravity.lock().unwrap();
        let mut bodies = self.bodies.lock().unwrap();

        for body in bodies.iter_mut() {
            // 简单的欧拉积分
            body.velocity += gravity * dt.as_secs_f32();
            body.position += body.velocity * dt.as_secs_f32();
        }
    }

    pub fn get_body_position(&self, id: usize) -> Option<glm::Vec3> {
        let bodies = self.bodies.lock().unwrap();
        bodies.get(id).map(|b| b.position)
    }
}

impl Default for MockPhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// 模拟资源管理器
#[derive(Debug)]
pub struct MockResourceManager {
    pub loaded_resources: Arc<Mutex<Vec<String>>>,
    pub memory_usage: Arc<Mutex<usize>>,
}

impl MockResourceManager {
    pub fn new() -> Self {
        Self {
            loaded_resources: Arc::new(Mutex::new(Vec::new())),
            memory_usage: Arc::new(Mutex::new(0)),
        }
    }

    pub fn load_resource(&self, path: &str, size: usize) -> Result<(), String> {
        let mut resources = self.loaded_resources.lock().unwrap();
        let mut memory = self.memory_usage.lock().unwrap();

        resources.push(path.to_string());
        *memory += size;
        Ok(())
    }

    pub fn unload_resource(&self, path: &str, size: usize) {
        let mut resources = self.loaded_resources.lock().unwrap();
        let mut memory = self.memory_usage.lock().unwrap();

        if let Some(pos) = resources.iter().position(|r| r == path) {
            resources.remove(pos);
            *memory = memory.saturating_sub(size);
        }
    }

    pub fn get_memory_usage(&self) -> usize {
        *self.memory_usage.lock().unwrap()
    }
}

impl Default for MockResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 模拟音频系统
#[derive(Debug)]
pub struct MockAudioSystem {
    pub playing_sounds: Arc<Mutex<Vec<String>>>,
    pub volume: Arc<Mutex<f32>>,
}

impl MockAudioSystem {
    pub fn new() -> Self {
        Self {
            playing_sounds: Arc::new(Mutex::new(Vec::new())),
            volume: Arc::new(Mutex::new(1.0)),
        }
    }

    pub fn play_sound(&self, name: &str) {
        let mut sounds = self.playing_sounds.lock().unwrap();
        sounds.push(name.to_string());
    }

    pub fn stop_sound(&self, name: &str) {
        let mut sounds = self.playing_sounds.lock().unwrap();
        if let Some(pos) = sounds.iter().position(|s| s == name) {
            sounds.remove(pos);
        }
    }

    pub fn set_volume(&self, volume: f32) {
        let mut vol = self.volume.lock().unwrap();
        *vol = volume.clamp(0.0, 1.0);
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }
}

impl Default for MockAudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_render_device() {
        let device = MockRenderDevice::new();
        device.begin_frame();
        device.draw(100);

        let (frames, vertices, calls) = device.get_stats();
        assert_eq!(frames, 1);
        assert_eq!(vertices, 100);
        assert_eq!(calls, 1);
    }

    #[test]
    fn test_mock_physics_world() {
        let world = MockPhysicsWorld::new();
        let body_id = world.add_body(glm::Vec3::new(0.0, 10.0, 0.0), 1.0);

        world.step(Duration::from_secs_f32(0.016));

        let position = world.get_body_position(body_id).unwrap();
        assert!(position.y < 10.0); // 应该下落
    }

    #[test]
    fn test_mock_resource_manager() {
        let manager = MockResourceManager::new();
        manager.load_resource("test.png", 1024).unwrap();

        assert_eq!(manager.get_memory_usage(), 1024);
    }

    #[test]
    fn test_mock_audio_system() {
        let audio = MockAudioSystem::new();
        audio.play_sound("test.wav");

        assert_eq!(audio.playing_sounds.lock().unwrap().len(), 1);
    }
}
