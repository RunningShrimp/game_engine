use crate::animation::{AnimationClip, InterpolationMode, KeyframeTrack};
use crate::impl_default;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 关键帧选择（增强功能）
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

/// 轨道类型（增强功能）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackType {
    #[default]
    Position,
    Rotation,
    Scale,
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

/// 动画事件（增强功能）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationEvent {
    pub time: f32,
    pub name: String,
    pub data: String,
}

/// 动画编辑器
pub struct AnimationEditor {
    pub clips: Vec<AnimationClip>,
    pub selected_clip: Option<usize>,
    pub timeline_zoom: f32,
    pub playback_time: f32,
    pub is_playing: bool,
    /// 播放速度（增强功能）
    pub playback_speed: f32,
    /// 关键帧选择（增强功能）
    pub keyframe_selection: KeyframeSelection,
    /// 动画事件（增强功能）
    pub events: HashMap<usize, Vec<AnimationEvent>>, // clip_index -> events
    /// 显示网格（增强功能）
    pub show_grid: bool,
    /// 吸附到网格（增强功能）
    pub snap_to_grid: bool,
    /// 网格间隔（增强功能）
    pub grid_interval: f32,
    /// 显示曲线（增强功能）
    pub show_curves: bool,
}

impl AnimationEditor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加新动画片段（增强功能）
    pub fn add_clip(&mut self, name: String, duration: f32) {
        let clip = AnimationClip::new(name, duration);
        self.clips.push(clip);
        self.selected_clip = Some(self.clips.len() - 1);
        self.events.insert(self.clips.len() - 1, Vec::new());
    }

    /// 删除动画片段（增强功能）
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

    /// 添加关键帧（增强功能）
    pub fn add_keyframe(&mut self, entity_id: u64, track_type: TrackType, time: f32) {
        if let Some(index) = self.selected_clip
            && let Some(clip) = self.clips.get_mut(index)
        {
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

    /// 删除关键帧（增强功能）
    pub fn delete_keyframe(
        &mut self,
        entity_id: u64,
        track_type: TrackType,
        keyframe_index: usize,
    ) {
        if let Some(index) = self.selected_clip
            && let Some(clip) = self.clips.get_mut(index)
        {
            match track_type {
                TrackType::Position => {
                    if let Some(track) = clip.position_tracks.get_mut(&entity_id)
                        && keyframe_index < track.keyframes.len()
                    {
                        track.keyframes.remove(keyframe_index);
                    }
                }
                TrackType::Rotation => {
                    if let Some(track) = clip.rotation_tracks.get_mut(&entity_id)
                        && keyframe_index < track.keyframes.len()
                    {
                        track.keyframes.remove(keyframe_index);
                    }
                }
                TrackType::Scale => {
                    if let Some(track) = clip.scale_tracks.get_mut(&entity_id)
                        && keyframe_index < track.keyframes.len()
                    {
                        track.keyframes.remove(keyframe_index);
                    }
                }
            }
        }
    }

    /// 添加动画事件（增强功能）
    pub fn add_event(&mut self, time: f32, name: String, data: String) {
        if let Some(index) = self.selected_clip {
            let events = self.events.entry(index).or_default();
            let event = AnimationEvent { time, name, data };
            events.push(event);
            events.sort_by(|a, b| {
                a.time.partial_cmp(&b.time).expect("Test: operation should succeed")
            });
        }
    }

    /// 渲染动画编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui, delta_time: f32) {
        ui.heading("Animation Editor");
        ui.separator();

        // 动画片段列表
        ui.label("Animation Clips:");
        for (i, clip) in self.clips.iter().enumerate() {
            let is_selected = self.selected_clip == Some(i);
            if ui.selectable_label(is_selected, &clip.name).clicked() {
                self.selected_clip = Some(i);
                self.playback_time = 0.0;
                self.is_playing = false;
            }
        }

        // 添加新动画片段按钮
        if ui.button("+ Add Animation Clip").clicked() {
            let clip = AnimationClip::new(format!("Animation {}", self.clips.len()), 1.0);
            self.clips.push(clip);
            self.selected_clip = Some(self.clips.len() - 1);
        }

        ui.separator();

        // 动画片段编辑
        if let Some(index) = self.selected_clip {
            if let Some(clip) = self.clips.get_mut(index) {
                ui.label(format!("Editing: {}", clip.name));
                ui.separator();

                // 动画属性
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

                ui.separator();

                // 播放控制
                ui.label("Playback:");
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

                    ui.label(format!(
                        "Time: {:.2} / {:.2} s",
                        self.playback_time, clip.duration
                    ));
                });

                // 更新播放时间（支持播放速度）
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

                // 时间轴滑块
                ui.add(
                    egui::Slider::new(&mut self.playback_time, 0.0..=clip.duration)
                        .text("Timeline"),
                );

                ui.separator();

                // 轨道列表
                ui.label("Tracks:");
                ui.label(format!("  Position Tracks: {}", clip.position_tracks.len()));
                ui.label(format!("  Rotation Tracks: {}", clip.rotation_tracks.len()));
                ui.label(format!("  Scale Tracks: {}", clip.scale_tracks.len()));

                ui.separator();

                // 时间轴缩放
                ui.horizontal(|ui| {
                    ui.label("Timeline Zoom:");
                    ui.add(egui::Slider::new(&mut self.timeline_zoom, 0.1..=10.0));
                });

                ui.separator();

                // 关键帧编辑 (占位)
                ui.label("Keyframe Editor:");
                ui.label("(Keyframe editing interface will be displayed here)");
            }
        } else {
            ui.label("No animation clip selected");
        }
    }
}

impl_default!(AnimationEditor {
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
});
