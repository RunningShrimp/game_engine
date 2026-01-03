/**
 * 动画系统 - Tauri后端集成
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use tauri::State;

// ==================== 类型定义 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyframeValue {
    Number(f64),
    Vector3(Vector3),
    Quaternion(Quaternion),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterpolationType {
    Constant,
    Linear,
    Cubic,
    Hermite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeData {
    pub id: String,
    pub time: f64,
    pub value: KeyframeValue,
    pub interpolation: InterpolationType,
    pub easing: EasingFunction,
    pub in_tangent: Option<Vector3>,
    pub out_tangent: Option<Vector3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationCurveData {
    pub id: String,
    pub name: String,
    pub property_path: String,
    pub keyframes: Vec<KeyframeData>,
    pub color: String,
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrackData {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub curves: Vec<AnimationCurveData>,
    pub visible: bool,
    pub locked: bool,
    pub muted: bool,
    pub expanded: bool,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClipData {
    pub id: String,
    pub name: String,
    pub duration: f64,
    pub frame_rate: f64,
    pub tracks: Vec<AnimationTrackData>,
    pub loop_enabled: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationState {
    pub clip_id: String,
    pub time: f64,
    pub values: HashMap<String, KeyframeValue>,
}

// ==================== 动画系统状态 ====================

pub struct AnimationSystem {
    clips: HashMap<String, AnimationClipData>,
    clips_directory: PathBuf,
}

impl AnimationSystem {
    pub fn new() -> Self {
        let clips_dir = std::env::current_dir()
            .unwrap()
            .join("animations");

        // 创建动画目录（如果不存在）
        if !clips_dir.exists() {
            fs::create_dir_all(&clips_dir).unwrap();
        }

        Self {
            clips: HashMap::new(),
            clips_directory: clips_dir,
        }
    }

    pub fn load_clips_from_disk(&mut self) -> Result<(), String> {
        let entries = fs::read_dir(&self.clips_directory)
            .map_err(|e| format!("Failed to read animations directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;

                let clip: AnimationClipData = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse animation file {:?}: {}", path, e))?;

                self.clips.insert(clip.id.clone(), clip);
            }
        }

        Ok(())
    }

    fn get_clip_path(&self, clip_id: &str) -> PathBuf {
        self.clips_directory.join(format!("{}.json", clip_id))
    }
}

// ==================== Tauri命令 ====================

#[tauri::command]
pub async fn create_animation_clip(
    name: String,
    state: State<'_, AnimationSystem>,
) -> Result<String, String> {
    let clip = AnimationClipData {
        id: format!("clip_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()),
        name,
        duration: 5.0,
        frame_rate: 60.0,
        tracks: vec![],
        loop_enabled: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };

    let clip_id = clip.id.clone();

    // 保存到文件
    let path = state.get_clip_path(&clip_id);
    let content = serde_json::to_string_pretty(&clip)
        .map_err(|e| format!("Failed to serialize animation clip: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write animation file {:?}: {}", path, e))?;

    // 添加到内存状态
    let mut system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;
    system.clips.insert(clip_id.clone(), clip);

    Ok(clip_id)
}

#[tauri::command]
pub async fn save_animation_clip(
    clip: AnimationClipData,
    state: State<'_, AnimationSystem>,
) -> Result<(), String> {
    // 保存到文件
    let path = state.get_clip_path(&clip.id);
    let content = serde_json::to_string_pretty(&clip)
        .map_err(|e| format!("Failed to serialize animation clip: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write animation file {:?}: {}", path, e))?;

    // 更新内存状态
    let mut system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;
    system.clips.insert(clip.id.clone(), clip);

    Ok(())
}

#[tauri::command]
pub async fn load_animation_clip(
    id: String,
    state: State<'_, Mutex<AnimationSystem>>,
) -> Result<AnimationClipData, String> {
    let system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;

    system.clips.get(&id)
        .cloned()
        .ok_or_else(|| format!("Animation clip not found: {}", id))
}

#[tauri::command]
pub async fn delete_animation_clip(
    clip_id: String,
    state: State<'_, AnimationSystem>,
) -> Result<(), String> {
    // 删除文件
    let path = state.get_clip_path(&clip_id);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete animation file {:?}: {}", path, e))?;
    }

    // 从内存状态移除
    let mut system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;
    system.clips.remove(&clip_id);

    Ok(())
}

#[tauri::command]
pub async fn list_animation_clips(
    state: State<'_, Mutex<AnimationSystem>>,
) -> Result<Vec<AnimationClipData>, String> {
    let system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;

    Ok(system.clips.values().cloned().collect())
}

#[tauri::command]
pub async fn add_keyframe(
    track_id: String,
    keyframe: KeyframeData,
    state: State<'_, Mutex<AnimationSystem>>,
) -> Result<(), String> {
    let mut system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;

    // 查找并更新轨道
    for clip in system.clips.values_mut() {
        for track in clip.tracks.iter_mut() {
            if track.id == track_id {
                for curve in track.curves.iter_mut() {
                    curve.keyframes.push(keyframe.clone());
                    curve.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

                    // 更新剪辑的更新时间
                    clip.updated_at = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    return Ok(());
                }
            }
        }
    }

    Err(format!("Track not found: {}", track_id))
}

#[tauri::command]
pub async fn update_keyframe(
    keyframe_id: String,
    data: KeyframeData,
    state: State<'_, Mutex<AnimationSystem>>,
) -> Result<(), String> {
    let mut system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;

    // 查找并更新关键帧
    for clip in system.clips.values_mut() {
        for track in clip.tracks.iter_mut() {
            for curve in track.curves.iter_mut() {
                if let Some(keyframe) = curve.keyframes.iter_mut().find(|kf| kf.id == keyframe_id) {
                    *keyframe = data;

                    // 更新剪辑的更新时间
                    clip.updated_at = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    return Ok(());
                }
            }
        }
    }

    Err(format!("Keyframe not found: {}", keyframe_id))
}

#[tauri::command]
pub async fn delete_keyframe(
    keyframe_id: String,
    state: State<'_, Mutex<AnimationSystem>>,
) -> Result<(), String> {
    let mut system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;

    // 查找并删除关键帧
    for clip in system.clips.values_mut() {
        for track in clip.tracks.iter_mut() {
            for curve in track.curves.iter_mut() {
                if let Some(pos) = curve.keyframes.iter().position(|kf| kf.id == keyframe_id) {
                    curve.keyframes.remove(pos);

                    // 更新剪辑的更新时间
                    clip.updated_at = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    return Ok(());
                }
            }
        }
    }

    Err(format!("Keyframe not found: {}", keyframe_id))
}

#[tauri::command]
pub async fn evaluate_animation_at_time(
    clip_id: String,
    time: f64,
    state: State<'_, Mutex<AnimationSystem>>,
) -> Result<AnimationState, String> {
    let system = state.lock().map_err(|e| format!("Failed to lock animation system: {}", e))?;

    let clip = system.clips.get(&clip_id)
        .ok_or_else(|| format!("Animation clip not found: {}", clip_id))?;

    let mut values = HashMap::new();

    // 评估每个轨道的曲线
    for track in &clip.tracks {
        for curve in &track.curves {
            // 找到当前时间前后的关键帧
            let mut prev_keyframe: Option<&KeyframeData> = None;
            let mut next_keyframe: Option<&KeyframeData> = None;

            for keyframe in &curve.keyframes {
                if keyframe.time <= time {
                    prev_keyframe = Some(keyframe);
                }
                if keyframe.time >= time && next_keyframe.is_none() {
                    next_keyframe = Some(keyframe);
                }
            }

            // 插值计算
            let value = if let (Some(prev), Some(next)) = (prev_keyframe, next_keyframe) {
                if prev.id == next.id {
                    // 正好在关键帧上
                    prev.value.clone()
                } else {
                    // 需要插值
                    interpolate_keyframes(time, prev, next)
                }
            } else if let Some(prev) = prev_keyframe {
                prev.value.clone()
            } else if let Some(next) = next_keyframe {
                next.value.clone()
            } else {
                continue;
            };

            values.insert(curve.property_path.clone(), value);
        }
    }

    Ok(AnimationState {
        clip_id,
        time,
        values,
    })
}

// ==================== 辅助函数 ====================

fn interpolate_keyframes(
    time: f64,
    prev: &KeyframeData,
    next: &KeyframeData,
) -> KeyframeValue {
    match &prev.value {
        KeyframeValue::Number(prev_val) => {
            if let KeyframeValue::Number(next_val) = &next.value {
                let t = (time - prev.time) / (next.time - prev.time);
                let eased_t = apply_easing(t, &prev.easing);
                KeyframeValue::Number(prev_val + (next_val - prev_val) * eased_t)
            } else {
                prev.value.clone()
            }
        }
        KeyframeValue::Vector3(prev_val) => {
            if let KeyframeValue::Vector3(next_val) = &next.value {
                let t = (time - prev.time) / (next.time - prev.time);
                let eased_t = apply_easing(t, &prev.easing);
                KeyframeValue::Vector3(Vector3 {
                    x: prev_val.x + (next_val.x - prev_val.x) * eased_t,
                    y: prev_val.y + (next_val.y - prev_val.y) * eased_t,
                    z: prev_val.z + (next_val.z - prev_val.z) * eased_t,
                })
            } else {
                prev.value.clone()
            }
        }
        KeyframeValue::Quaternion(prev_val) => {
            if let KeyframeValue::Quaternion(next_val) = &next.value {
                // 简化的四元数插值（实际应该使用slerp）
                let t = (time - prev.time) / (next.time - prev.time);
                let eased_t = apply_easing(t, &prev.easing);
                KeyframeValue::Quaternion(Quaternion {
                    x: prev_val.x + (next_val.x - prev_val.x) * eased_t,
                    y: prev_val.y + (next_val.y - prev_val.y) * eased_t,
                    z: prev_val.z + (next_val.z - prev_val.z) * eased_t,
                    w: prev_val.w + (next_val.w - prev_val.w) * eased_t,
                })
            } else {
                prev.value.clone()
            }
        }
    }
}

fn apply_easing(t: f64, easing: &EasingFunction) -> f64 {
    let t = t.max(0.0).min(1.0);

    match easing {
        EasingFunction::Linear => t,
        EasingFunction::EaseInQuad => t * t,
        EasingFunction::EaseOutQuad => t * (2.0 - t),
        EasingFunction::EaseInOutQuad => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        EasingFunction::EaseInCubic => t * t * t,
        EasingFunction::EaseOutCubic => {
            let t = t - 1.0;
            t * t * t + 1.0
        }
        EasingFunction::EaseInOutCubic => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                let t = t - 1.0;
                1.0 + 4.0 * t * t * t
            }
        }
        // 其他easing函数可以类似实现
        _ => t,
    }
}
