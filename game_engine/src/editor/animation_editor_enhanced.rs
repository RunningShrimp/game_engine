//! 增强的动画编辑器
//!
//! 提供完整的动画编辑功能：
//! - 关键帧编辑和时间轴
//! - 曲线编辑和插值模式
//! - 动画混合和过渡
//! - 动画事件系统
//! - 动画导出/导入

use crate::animation::{AnimationClip, InterpolationMode, KeyframeTrack};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 关键帧选择
#[derive(Debug, Clone)]
pub struct KeyframeSelection {
    pub selected_keyframes: Vec<(u64, usize, usize)>, // (entity_id, track_type, keyframe_index)
    pub track_type: TrackType,
    pub entity_id: Option<u64>,
}

impl Default for KeyframeSelection {
    fn default() -> Self {
        Self {
            selected_keyframes: Vec::new(),
            track_type: TrackType::Position,
            entity_id: None,
        }
    }
}

/// 轨道类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Position,
    Rotation,
    Scale,
}

impl Default for TrackType {
    fn default() -> Self {
        TrackType::Position
    }
}

impl TrackType {
    pub fn name(&self) -> &'static str {
        match self {
            TrackType::Position => "Position",
            TrackType::Rotation => "Rotation",
            TrackType::Scale => "Scale",
        }
    }
}

/// 动画事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationEvent {
    pub time: f32,
    pub name: String,
    pub data: String,
}

/// 增强的动画编辑器
pub struct AnimationEditorEnhanced {
    /// 动画片段列表
    pub clips: Vec<AnimationClip>,
    /// 选中的动画片段索引
    pub selected_clip: Option<usize>,
    /// 时间轴缩放
    pub timeline_zoom: f32,
    /// 播放时间
    pub playback_time: f32,
    /// 是否正在播放
    pub is_playing: bool,
    /// 播放速度
    pub playback_speed: f32,
    /// 关键帧选择
    pub keyframe_selection: KeyframeSelection,
    /// 动画事件
    pub events: HashMap<usize, Vec<AnimationEvent>>, // clip_index -> events
    /// 显示网格
    pub show_grid: bool,
    /// 吸附到网格
    pub snap_to_grid: bool,
    /// 网格间隔
    pub grid_interval: f32,
    /// 显示曲线
    pub show_curves: bool,
}

impl AnimationEditorEnhanced {
    pub fn new() -> Self {
        Self {
            clips: Vec::new(),
            selected_clip: None,
            timeline_zoom: 1.0,
            playback_time: 0.0,
            is_playing: false,
            playback_speed: 1.0,
            keyframe_selection: KeyframeSelection::default(),
            events: HashMap::new(),
            show_grid: true,
            snap_to_grid: true,
            grid_interval: 0.1,
            show_curves: true,
        }
    }

    /// 添加新动画片段
    pub fn add_clip(&mut self, name: String, duration: f32) {
        let clip = AnimationClip::new(name, duration);
        self.clips.push(clip);
        self.selected_clip = Some(self.clips.len() - 1);
        self.events.insert(self.clips.len() - 1, Vec::new());
    }

    /// 删除动画片段
    pub fn delete_clip(&mut self, index: usize) {
        if index < self.clips.len() {
            self.clips.remove(index);
            self.events.remove(&index);
            if self.selected_clip == Some(index) {
                self.selected_clip = if self.clips.is_empty() {
                    None
                } else {
                    Some((index - 1).min(self.clips.len() - 1))
                };
            }
        }
    }

    /// 添加关键帧
    pub fn add_keyframe(&mut self, entity_id: u64, track_type: TrackType, time: f32) {
        if let Some(index) = self.selected_clip {
            if let Some(clip) = self.clips.get_mut(index) {
                let time = if self.snap_to_grid {
                    (time / self.grid_interval).round() * self.grid_interval
                } else {
                    time
                };

                match track_type {
                    TrackType::Position => {
                        let track = clip
                            .position_tracks
                            .entry(entity_id)
                            .or_insert_with(|| KeyframeTrack::new(InterpolationMode::Linear));
                        track.add_keyframe(time, Vec3::ZERO);
                    }
                    TrackType::Rotation => {
                        let track = clip
                            .rotation_tracks
                            .entry(entity_id)
                            .or_insert_with(|| KeyframeTrack::new(InterpolationMode::Linear));
                        track.add_keyframe(time, Quat::IDENTITY);
                    }
                    TrackType::Scale => {
                        let track = clip
                            .scale_tracks
                            .entry(entity_id)
                            .or_insert_with(|| KeyframeTrack::new(InterpolationMode::Linear));
                        track.add_keyframe(time, Vec3::ONE);
                    }
                }
            }
        }
    }

    /// 删除关键帧
    pub fn delete_keyframe(&mut self, entity_id: u64, track_type: TrackType, keyframe_index: usize) {
        if let Some(index) = self.selected_clip {
            if let Some(clip) = self.clips.get_mut(index) {
                match track_type {
                    TrackType::Position => {
                        if let Some(track) = clip.position_tracks.get_mut(&entity_id) {
                            if keyframe_index < track.keyframes.len() {
                                track.keyframes.remove(keyframe_index);
                            }
                        }
                    }
                    TrackType::Rotation => {
                        if let Some(track) = clip.rotation_tracks.get_mut(&entity_id) {
                            if keyframe_index < track.keyframes.len() {
                                track.keyframes.remove(keyframe_index);
                            }
                        }
                    }
                    TrackType::Scale => {
                        if let Some(track) = clip.scale_tracks.get_mut(&entity_id) {
                            if keyframe_index < track.keyframes.len() {
                                track.keyframes.remove(keyframe_index);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 添加动画事件
    pub fn add_event(&mut self, time: f32, name: String, data: String) {
        if let Some(index) = self.selected_clip {
            let events = self.events.entry(index).or_insert_with(Vec::new);
            let event = AnimationEvent { time, name, data };
            events.push(event);
            events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
    }

    /// 渲染动画编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui, delta_time: f32) {
        ui.heading("Animation Editor");
        ui.separator();

        // 工具栏
        ui.horizontal(|ui| {
            if ui.button("+ New Clip").clicked() {
                self.add_clip(format!("Animation {}", self.clips.len()), 1.0);
            }
            if ui.button("🗑 Delete").clicked() {
                if let Some(index) = self.selected_clip {
                    self.delete_clip(index);
                }
            }
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.snap_to_grid, "Snap");
            ui.checkbox(&mut self.show_curves, "Curves");
        });

        ui.separator();

        // 动画片段列表
        ui.collapsing("Animation Clips", |ui| {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (i, clip) in self.clips.iter().enumerate() {
                        let is_selected = self.selected_clip == Some(i);
                        if ui.selectable_label(is_selected, &clip.name).clicked() {
                            self.selected_clip = Some(i);
                            self.playback_time = 0.0;
                            self.is_playing = false;
                        }
                    }
                });
        });

        ui.separator();

        // 动画片段编辑
        if let Some(index) = self.selected_clip {
            if let Some(clip) = self.clips.get_mut(index) {
                // 动画属性
                ui.collapsing("Clip Properties", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut clip.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Duration:");
                        ui.add(
                            egui::DragValue::new(&mut clip.duration)
                                .suffix(" s")
                                .speed(0.1)
                                .range(0.1..=60.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Looping:");
                        ui.checkbox(&mut clip.looping, "");
                    });
                });

                ui.separator();

                // 播放控制
                ui.collapsing("Playback Controls", |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(if self.is_playing {
                                "⏸ Pause"
                            } else {
                                "▶ Play"
                            })
                            .clicked()
                        {
                            self.is_playing = !self.is_playing;
                        }

                        if ui.button("⏹ Stop").clicked() {
                            self.is_playing = false;
                            self.playback_time = 0.0;
                        }

                        if ui.button("⏮ First").clicked() {
                            self.playback_time = 0.0;
                        }

                        if ui.button("⏭ Last").clicked() {
                            self.playback_time = clip.duration;
                        }

                        ui.label(format!(
                            "Time: {:.2} / {:.2} s",
                            self.playback_time, clip.duration
                        ));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        ui.add(egui::Slider::new(&mut self.playback_speed, 0.0..=2.0));
                    });

                    // 时间轴滑块
                    ui.add(
                        egui::Slider::new(&mut self.playback_time, 0.0..=clip.duration)
                            .text("Timeline"),
                    );
                });

                // 更新播放时间
                if self.is_playing {
                    self.playback_time += delta_time * self.playback_speed;
                    if self.playback_time >= clip.duration {
                        if clip.looping {
                            self.playback_time %= clip.duration;
                        } else {
                            self.playback_time = clip.duration;
                            self.is_playing = false;
                        }
                    }
                }

                ui.separator();

                // 时间轴视图
                ui.collapsing("Timeline", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Zoom:");
                        ui.add(egui::Slider::new(&mut self.timeline_zoom, 0.1..=10.0));
                    });

                    // 时间轴区域（简化实现）
                    ui.label("Timeline View:");
                    ui.label("(Timeline with keyframes will be displayed here)");

                    // 关键帧操作
                    ui.horizontal(|ui| {
                        ui.label("Add Keyframe:");
                        if ui.button("Position").clicked() {
                            if let Some(entity_id) = self.keyframe_selection.entity_id {
                                self.add_keyframe(entity_id, TrackType::Position, self.playback_time);
                            }
                        }
                        if ui.button("Rotation").clicked() {
                            if let Some(entity_id) = self.keyframe_selection.entity_id {
                                self.add_keyframe(entity_id, TrackType::Rotation, self.playback_time);
                            }
                        }
                        if ui.button("Scale").clicked() {
                            if let Some(entity_id) = self.keyframe_selection.entity_id {
                                self.add_keyframe(entity_id, TrackType::Scale, self.playback_time);
                            }
                        }
                    });
                });

                ui.separator();

                // 轨道列表
                ui.collapsing("Tracks", |ui| {
                    ui.label(format!("Position Tracks: {}", clip.position_tracks.len()));
                    for (entity_id, track) in &clip.position_tracks {
                        ui.horizontal(|ui| {
                            ui.label(format!("  Entity {}: {} keyframes", entity_id, track.keyframes.len()));
                            if ui.button("Edit").clicked() {
                                self.keyframe_selection.entity_id = Some(*entity_id);
                                self.keyframe_selection.track_type = TrackType::Position;
                            }
                        });
                    }

                    ui.label(format!("Rotation Tracks: {}", clip.rotation_tracks.len()));
                    for (entity_id, track) in &clip.rotation_tracks {
                        ui.horizontal(|ui| {
                            ui.label(format!("  Entity {}: {} keyframes", entity_id, track.keyframes.len()));
                            if ui.button("Edit").clicked() {
                                self.keyframe_selection.entity_id = Some(*entity_id);
                                self.keyframe_selection.track_type = TrackType::Rotation;
                            }
                        });
                    }

                    ui.label(format!("Scale Tracks: {}", clip.scale_tracks.len()));
                    for (entity_id, track) in &clip.scale_tracks {
                        ui.horizontal(|ui| {
                            ui.label(format!("  Entity {}: {} keyframes", entity_id, track.keyframes.len()));
                            if ui.button("Edit").clicked() {
                                self.keyframe_selection.entity_id = Some(*entity_id);
                                self.keyframe_selection.track_type = TrackType::Scale;
                            }
                        });
                    }
                });

                ui.separator();

                // 关键帧编辑
                if let Some(entity_id) = self.keyframe_selection.entity_id {
                    ui.collapsing("Keyframe Editor", |ui| {
                        ui.label(format!(
                            "Editing: {} Track for Entity {}",
                            self.keyframe_selection.track_type.name(),
                            entity_id
                        ));

                        match self.keyframe_selection.track_type {
                            TrackType::Position => {
                                if let Some(track) = clip.position_tracks.get(&entity_id) {
                                    self.render_keyframe_list_vec3(ui, track, entity_id, TrackType::Position);
                                }
                            }
                            TrackType::Rotation => {
                                // Rotation tracks use Quat, need separate rendering
                                ui.label("Rotation keyframes (Quat) - editing interface coming soon");
                            }
                            TrackType::Scale => {
                                if let Some(track) = clip.scale_tracks.get(&entity_id) {
                                    self.render_keyframe_list_vec3(ui, track, entity_id, TrackType::Scale);
                                }
                            }
                        }
                    });
                }

                ui.separator();

                // 动画事件
                ui.collapsing("Animation Events", |ui| {
                    let events = self.events.entry(index).or_insert_with(Vec::new);
                    let mut to_remove = Vec::new();
                    
                    for (i, event) in events.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut event.time)
                                    .prefix("Time: ")
                                    .suffix(" s")
                                    .speed(0.1)
                                    .range(0.0..=clip.duration),
                            );
                            ui.text_edit_singleline(&mut event.name);
                            ui.text_edit_singleline(&mut event.data);
                            if ui.button("🗑").clicked() {
                                to_remove.push(i);
                            }
                        });
                    }

                    // 删除事件（从后往前删除）
                    to_remove.sort();
                    to_remove.reverse();
                    for index in to_remove {
                        events.remove(index);
                    }

                    if ui.button("+ Add Event").clicked() {
                        self.add_event(self.playback_time, "Event".to_string(), "".to_string());
                    }
                });

                ui.separator();

                // 曲线编辑器（占位）
                if self.show_curves {
                    ui.collapsing("Curve Editor", |ui| {
                        ui.label("Curve Editor:");
                        ui.label("(Curve editing interface will be displayed here)");
                    });
                }
            }
        } else {
            ui.label("No animation clip selected");
        }
    }

    /// 渲染关键帧列表（Vec3）
    fn render_keyframe_list_vec3(
        &mut self,
        ui: &mut egui::Ui,
        track: &KeyframeTrack<Vec3>,
        entity_id: u64,
        track_type: TrackType,
    ) {
        ui.label(format!("Keyframes: {}", track.keyframes.len()));

        // 收集要删除的关键帧索引
        let mut to_delete = Vec::new();

        // 显示关键帧列表（只读，编辑需要通过其他方法）
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for (i, keyframe) in track.keyframes.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Keyframe {}:", i));
                        ui.label(format!("Time: {:.2} s", keyframe.time));
                        ui.label(format!("Value: ({:.2}, {:.2}, {:.2})", 
                            keyframe.value.x, keyframe.value.y, keyframe.value.z));
                        if ui.button("🗑").clicked() {
                            to_delete.push(i);
                        }
                    });
                }
            });

        // 删除关键帧（从后往前删除以避免索引问题）
        to_delete.sort();
        to_delete.reverse();
        for index in to_delete {
            self.delete_keyframe(entity_id, track_type, index);
        }
    }
}

impl Default for AnimationEditorEnhanced {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_editor() {
        let mut editor = AnimationEditorEnhanced::new();
        editor.add_clip("Test Animation".to_string(), 2.0);
        assert!(!editor.clips.is_empty());
        assert_eq!(editor.clips[0].name, "Test Animation");
    }

    #[test]
    fn test_keyframe_operations() {
        let mut editor = AnimationEditorEnhanced::new();
        editor.add_clip("Test".to_string(), 1.0);
        editor.keyframe_selection.entity_id = Some(1);
        editor.add_keyframe(1, TrackType::Position, 0.5);
        
        if let Some(clip) = editor.clips.get(0) {
            assert!(clip.position_tracks.contains_key(&1));
        }
    }
}

