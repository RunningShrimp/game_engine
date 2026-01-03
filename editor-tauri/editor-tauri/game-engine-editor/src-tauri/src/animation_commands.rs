// 动画管理Tauri命令实现
// 为Timeline组件提供简化的动画管理功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

// 简化的动画剪辑结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClip {
    pub id: String,
    pub name: String,
    pub duration: f32,
    pub framerate: u32,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    pub curve_type: String, // "linear", "ease-in", "ease-out", etc.
}

// 动画管理器（内存存储）
pub struct AnimationManager {
    clips: Mutex<HashMap<String, AnimationClip>>,
    project_path: Mutex<Option<PathBuf>>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            clips: Mutex::new(HashMap::new()),
            project_path: Mutex::new(None),
        }
    }

    /// 保存动画剪辑
    pub fn save_clip(&self, clip: AnimationClip) -> Result<(), String> {
        let mut clips = self.clips.lock().unwrap();
        clips.insert(clip.id.clone(), clip);

        // 简化实现：仅内存存储，生产环境应持久化到文件
        Ok(())
    }

    /// 删除动画剪辑
    pub fn delete_clip(&self, clip_id: &str) -> Result<(), String> {
        let mut clips = self.clips.lock().unwrap();
        clips.remove(clip_id)
            .ok_or_else(|| format!("动画剪辑 '{}' 不存在", clip_id))?;
        Ok(())
    }

    /// 获取所有动画剪辑
    pub fn get_all_clips(&self) -> Vec<AnimationClip> {
        let clips = self.clips.lock().unwrap();
        clips.values().cloned().collect()
    }

    /// 获取特定动画剪辑
    pub fn get_clip(&self, clip_id: &str) -> Option<AnimationClip> {
        let clips = self.clips.lock().unwrap();
        clips.get(clip_id).cloned()
    }

    /// 设置项目路径
    pub fn set_project_path(&self, path: PathBuf) {
        *self.project_path.lock().unwrap() = Some(path);
    }
}

// 全局动画管理器实例
lazy_static::lazy_static! {
    pub static ref ANIMATION_MANAGER: AnimationManager = AnimationManager::new();
}

// Tauri命令实现
#[tauri::command]
pub async fn save_animation_clip(clip: AnimationClip) -> Result<(), String> {
    ANIMATION_MANAGER.save_clip(clip)
}

#[tauri::command]
pub async fn delete_animation_clip(clip_id: String) -> Result<(), String> {
    ANIMATION_MANAGER.delete_clip(&clip_id)
}

#[tauri::command]
pub async fn list_animation_clips() -> Result<Vec<AnimationClip>, String> {
    Ok(ANIMATION_MANAGER.get_all_clips())
}

#[tauri::command]
pub async fn load_animation_clip(clip_id: String) -> Result<AnimationClip, String> {
    ANIMATION_MANAGER.get_clip(&clip_id)
        .ok_or_else(|| format!("动画剪辑 '{}' 不存在", clip_id))
}