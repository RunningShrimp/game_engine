//! 场景管理器
//!
//! 提供场景的加载、保存、切换和管理功能。

use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 场景ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(pub u64);

/// 场景
#[derive(Debug, Clone)]
pub struct Scene {
    pub id: SceneId,
    pub name: String,
    pub entities: Vec<Entity>,
    pub metadata: HashMap<String, String>,
}

/// 场景过渡类型
#[derive(Debug, Clone)]
pub enum SceneTransition {
    /// 立即切换
    Immediate,
    /// 淡入淡出
    Fade { duration: f32 },
    /// 滑动过渡
    Slide {
        direction: SlideDirection,
        duration: f32,
    },
}

/// 滑动方向
#[derive(Debug, Clone, Copy)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

/// 场景管理器
#[derive(Resource)]
pub struct SceneManager {
    scenes: HashMap<SceneId, Scene>,
    current_scene: Option<SceneId>,
    next_id: u64,
    transition_state: Option<SceneTransitionState>,
}

#[derive(Debug)]
struct SceneTransitionState {
    from_scene: Option<SceneId>,
    to_scene: SceneId,
    transition: SceneTransition,
    elapsed: f32,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self {
            scenes: HashMap::new(),
            current_scene: None,
            next_id: 1,
            transition_state: None,
        }
    }
}

impl SceneManager {
    /// 创建新的场景管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建新场景
    pub fn create_scene(&mut self, name: String) -> SceneId {
        let id = SceneId(self.next_id);
        self.next_id += 1;

        let scene = Scene {
            id,
            name,
            entities: Vec::new(),
            metadata: HashMap::new(),
        };

        self.scenes.insert(id, scene);
        id
    }

    /// 添加实体到场景
    pub fn add_entity_to_scene(&mut self, scene_id: SceneId, entity: Entity) {
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.entities.push(entity);
        }
    }

    /// 从场景移除实体
    pub fn remove_entity_from_scene(&mut self, scene_id: SceneId, entity: Entity) {
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.entities.retain(|&e| e != entity);
        }
    }

    /// 切换到场景
    pub fn switch_to_scene(&mut self, scene_id: SceneId, transition: SceneTransition) {
        if self.scenes.contains_key(&scene_id) {
            self.transition_state = Some(SceneTransitionState {
                from_scene: self.current_scene,
                to_scene: scene_id,
                transition,
                elapsed: 0.0,
            });
        }
    }

    /// 立即切换场景（无过渡）
    pub fn switch_to_scene_immediate(&mut self, scene_id: SceneId) {
        if self.scenes.contains_key(&scene_id) {
            self.current_scene = Some(scene_id);
            self.transition_state = None;
        }
    }

    /// 获取当前场景
    pub fn current_scene(&self) -> Option<&Scene> {
        self.current_scene.and_then(|id| self.scenes.get(&id))
    }

    /// 获取场景
    pub fn get_scene(&self, scene_id: SceneId) -> Option<&Scene> {
        self.scenes.get(&scene_id)
    }

    /// 获取所有场景
    pub fn all_scenes(&self) -> Vec<&Scene> {
        self.scenes.values().collect()
    }

    /// 删除场景
    pub fn delete_scene(&mut self, scene_id: SceneId) {
        if self.current_scene == Some(scene_id) {
            self.current_scene = None;
        }
        self.scenes.remove(&scene_id);
    }

    /// 重命名场景
    pub fn rename_scene(&mut self, scene_id: SceneId, new_name: String) {
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.name = new_name;
        }
    }

    /// 设置场景元数据
    pub fn set_scene_metadata(&mut self, scene_id: SceneId, key: String, value: String) {
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.metadata.insert(key, value);
        }
    }

    /// 获取场景元数据
    pub fn get_scene_metadata(&self, scene_id: SceneId, key: &str) -> Option<&String> {
        self.scenes
            .get(&scene_id)
            .and_then(|scene| scene.metadata.get(key))
    }

    /// 更新过渡状态
    pub fn update_transition(&mut self, delta_time: f32) {
        if let Some(ref mut state) = self.transition_state {
            state.elapsed += delta_time;

            let duration = match state.transition {
                SceneTransition::Immediate => 0.0,
                SceneTransition::Fade { duration } => duration,
                SceneTransition::Slide { duration, .. } => duration,
            };

            if state.elapsed >= duration {
                // 过渡完成
                self.current_scene = Some(state.to_scene);
                self.transition_state = None;
            }
        }
    }

    /// 获取过渡进度 (0.0 - 1.0)
    pub fn transition_progress(&self) -> f32 {
        if let Some(ref state) = self.transition_state {
            let duration = match state.transition {
                SceneTransition::Immediate => 1.0,
                SceneTransition::Fade { duration } => duration,
                SceneTransition::Slide { duration, .. } => duration,
            };
            (state.elapsed / duration).min(1.0)
        } else {
            1.0
        }
    }

    /// 检查是否正在过渡
    pub fn is_transitioning(&self) -> bool {
        self.transition_state.is_some()
    }

    /// 获取场景数量
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }
}

/// 场景系统 - ECS系统函数
pub fn scene_update_system(mut scene_manager: ResMut<SceneManager>, time: Res<crate::ecs::Time>) {
    scene_manager.update_transition(time.delta_seconds);
}

/// 场景加载系统
pub fn scene_load_system(scene_manager: Res<SceneManager>, mut commands: Commands) {
    // 加载当前场景的实体
    if let Some(current_scene) = scene_manager.current_scene() {
        // 这里可以实现场景实体的实际加载逻辑
        tracing::info!(target: "scene", "Loading scene: {}", current_scene.name);
    }
}

/// 场景清理系统
pub fn scene_cleanup_system(scene_manager: Res<SceneManager>, mut commands: Commands) {
    // 清理前一个场景的实体
    // 实际实现需要跟踪哪些实体属于哪个场景
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_manager() {
        let mut manager = SceneManager::new();

        // 创建场景
        let scene1_id = manager.create_scene("Main Menu".to_string());
        let _scene2_id = manager.create_scene("Game Level".to_string());

        assert_eq!(manager.scene_count(), 2);

        // 切换场景
        manager.switch_to_scene_immediate(scene1_id);
        assert_eq!(manager.current_scene().unwrap().name, "Main Menu");

        // 重命名场景
        manager.rename_scene(scene1_id, "Updated Menu".to_string());
        assert_eq!(manager.current_scene().unwrap().name, "Updated Menu");

        // 设置元数据
        manager.set_scene_metadata(scene1_id, "difficulty".to_string(), "easy".to_string());
        assert_eq!(
            manager.get_scene_metadata(scene1_id, "difficulty"),
            Some(&"easy".to_string())
        );
    }

    #[test]
    fn test_scene_transition() {
        let mut manager = SceneManager::new();

        let scene1_id = manager.create_scene("Scene 1".to_string());
        let scene2_id = manager.create_scene("Scene 2".to_string());

        manager.switch_to_scene_immediate(scene1_id);

        // 开始过渡
        manager.switch_to_scene(scene2_id, SceneTransition::Fade { duration: 1.0 });

        assert!(manager.is_transitioning());

        // 更新过渡
        manager.update_transition(0.5);
        assert!(manager.transition_progress() < 1.0);

        // 完成过渡
        manager.update_transition(0.6);
        assert_eq!(manager.transition_progress(), 1.0);
        assert!(!manager.is_transitioning());
        assert_eq!(manager.current_scene().unwrap().name, "Scene 2");
    }
}
