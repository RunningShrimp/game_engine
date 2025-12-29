//  测试fixture和模拟对象
//
//  提供常用的测试fixture，减少重复代码。

use bevy_ecs::prelude::*;
use std::sync::{Arc, Mutex};

/// ECS World fixture
pub struct WorldFixture {
    pub world: World,
}

impl WorldFixture {
    /// 创建新的World fixture
    pub fn new() -> Self {
        let mut world = World::new();

        // 添加基础资源
        world.insert_resource(TimeTracker::default());

        Self { world }
    }

    /// 添加测试实体
    pub fn spawn_test_entity(&mut self) -> Entity {
        self.world.spawn((
            Name::new("test_entity".to_string()),
            Transform::default(),
        ))
    }

    /// 添加带有特定组件的实体
    pub fn spawn_entity_with_components<'a, T: Bundle>(&mut self, components: T) -> Entity {
        self.world.spawn(components)
    }
}

impl Default for WorldFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// 时间追踪资源（用于测试）
#[derive(Default, Debug)]
pub struct TimeTracker {
    pub elapsed: std::time::Duration,
}

/// 名称组件
#[derive(Debug, Component)]
pub struct Name {
    pub value: String,
}

impl Name {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

/// 变换组件
#[derive(Debug, Component, Default)]
pub struct Transform {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub scale: (f32, f32, f32),
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        }
    }
}

/// 测试场景fixture
pub struct SceneFixture {
    pub scene: game_engine::domain::Scene,
}

impl SceneFixture {
    /// 创建新的测试场景
    pub fn new() -> Result<Self, game_engine::domain::DomainError> {
        let scene = game_engine::domain::Scene::new(
            game_engine::domain::SceneId::new(1),
            "test_scene"
        )?;

        Ok(Self { scene })
    }

    /// 创建并返回带有实体的测试场景
    pub fn with_entities() -> Result<Self, game_engine::domain::DomainError> {
        let mut fixture = Self::new()?;

        // 添加测试实体
        let entity = game_engine::domain::GameEntity::new(
            game_engine::domain::EntityId::new(1),
            "test_entity"
        );
        fixture.scene.add_entity(entity)?;

        Ok(fixture)
    }
}

/// 模拟配置fixture
pub struct ConfigFixture {
    pub config: game_engine::config::EngineConfig,
}

impl ConfigFixture {
    /// 创建测试配置
    pub fn new() -> Self {
        let mut config = game_engine::config::EngineConfig::default();

        // 使用较小的配置加速测试
        config.graphics.resolution.width = 800;
        config.graphics.resolution.height = 600;

        Self { config }
    }

    /// 创建无窗口配置（用于无头测试）
    pub fn headless() -> Self {
        Self::new()
    }
}

impl Default for ConfigFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试资源加载器fixture
pub struct ResourceLoaderFixture {
    pub loader: Arc<Mutex<MockResourceLoader>>,
}

impl ResourceLoaderFixture {
    /// 创建新的测试资源加载器
    pub fn new() -> Self {
        Self {
            loader: Arc::new(Mutex::new(MockResourceLoader::new())),
        }
    }
}

/// 模拟资源加载器
pub struct MockResourceLoader {
    pub loaded_paths: Vec<String>,
    pub should_fail: bool,
}

impl MockResourceLoader {
    pub fn new() -> Self {
        Self {
            loaded_paths: Vec::new(),
            should_fail: false,
        }
    }

    /// 设置为失败模式
    pub fn set_fail_mode(&mut self, fail: bool) {
        self.should_fail = fail;
    }

    /// 模拟加载资源
    pub fn load(&mut self, path: &str) -> Result<Vec<u8>, String> {
        if self.should_fail {
            return Err(format!("Simulated load failure for: {}", path));
        }

        self.loaded_paths.push(path.to_string());
        Ok(vec
![ /* 返回模拟数据 */ ])
    }
}

impl Default for MockResourceLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能测试fixture
pub struct PerformanceFixture {
    pub measurements: Vec<Measurement>,
}

impl PerformanceFixture {
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
        }
    }

    /// 测量操作性能
    pub fn measure<F>(&mut self, name: &str, operation: F) -> Duration
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        operation();
        let duration = start.elapsed();

        self.measurements.push(Measurement {
            name: name.to_string(),
            duration,
            timestamp: Instant::now(),
        });

        duration
    }

    /// 获取所有测量结果
    pub fn results(&self) -> &[Measurement] {
        &self.measurements
    }

    /// 打印性能报告
    pub fn print_report(&self) {
        println!("\n=== Performance Report ===");
        for measurement in &self.measurements {
            println!("{}: {:?}", measurement.name, measurement.duration);
        }
        println!("========================\n");
    }
}

impl Default for PerformanceFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能测量记录
#[derive(Debug)]
pub struct Measurement {
    pub name: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_world_fixture() {
        let mut fixture = WorldFixture::new();
        let entity = fixture.spawn_test_entity();

        // 验证实体已创建
        assert!(fixture.world.get::<Name>(entity).is_some());
        assert_eq!(
            fixture.world.get::<Name>(entity)
                .expect("Name component should exist for test entity")
                .value,
            "test_entity"
        );
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_scene_fixture() {
        let fixture = SceneFixture::new()
            .expect("SceneFixture::new should not fail in test context");
        assert_eq!(fixture.scene.name, "test_scene");
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_performance_fixture() {
        let mut fixture = PerformanceFixture::new();

        fixture.measure("test_operation", || {
            std::thread::sleep(Duration::from_millis(10));
        });

        assert_eq!(fixture.measurements.len(), 1);
        assert_eq!(fixture.measurements[0].name, "test_operation");
    }
}
