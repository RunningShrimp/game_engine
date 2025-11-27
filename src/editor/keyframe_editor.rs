use crate::animation::{KeyframeTrack, InterpolationMode};
use glam::{Vec3, Quat};

/// 关键帧编辑器
pub struct KeyframeEditor {
    /// 当前编辑的轨道类型
    pub track_type: TrackType,
    /// 选中的关键帧索引
    pub selected_keyframe: Option<usize>,
    /// 时间轴缩放
    pub timeline_zoom: f32,
    /// 时间轴偏移
    pub timeline_offset: f32,
}

/// 轨道类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Position,
    Rotation,
    Scale,
}

impl KeyframeEditor {
    pub fn new() -> Self {
        Self {
            track_type: TrackType::Position,
            selected_keyframe: None,
            timeline_zoom: 1.0,
            timeline_offset: 0.0,
        }
    }
    
    /// 渲染关键帧编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui, entity_id: u64, clip: &mut crate::animation::AnimationClip) {
        ui.heading("Keyframe Editor");
        ui.separator();
        
        // 实体ID
        ui.label(format!("Entity ID: {}", entity_id));
        ui.separator();
        
        // 轨道类型选择
        ui.label("Track Type:");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.track_type, TrackType::Position, "Position");
            ui.selectable_value(&mut self.track_type, TrackType::Rotation, "Rotation");
            ui.selectable_value(&mut self.track_type, TrackType::Scale, "Scale");
        });
        
        ui.separator();
        
        // 根据轨道类型渲染不同的编辑器
        match self.track_type {
            TrackType::Position => {
                self.render_position_track(ui, entity_id, clip);
            }
            TrackType::Rotation => {
                self.render_rotation_track(ui, entity_id, clip);
            }
            TrackType::Scale => {
                self.render_scale_track(ui, entity_id, clip);
            }
        }
    }
    
    /// 渲染位置轨道编辑器
    fn render_position_track(&mut self, ui: &mut egui::Ui, entity_id: u64, clip: &mut crate::animation::AnimationClip) {
        ui.label("Position Track:");
        
        // 获取或创建轨道
        let track = clip.position_tracks.entry(entity_id).or_insert_with(|| {
            KeyframeTrack::new(InterpolationMode::Linear)
        });
        
        // 插值模式选择
        ui.horizontal(|ui| {
            ui.label("Interpolation:");
            ui.selectable_value(&mut track.interpolation, InterpolationMode::Linear, "Linear");
            ui.selectable_value(&mut track.interpolation, InterpolationMode::Step, "Step");
            ui.selectable_value(&mut track.interpolation, InterpolationMode::CubicBezier, "Cubic");
        });
        
        ui.separator();
        
        // 添加关键帧
        ui.label("Add Keyframe:");
        let mut new_time = 0.0;
        let mut new_value = Vec3::ZERO;
        
        ui.horizontal(|ui| {
            ui.label("Time:");
            ui.add(egui::DragValue::new(&mut new_time).suffix(" s").speed(0.1).range(0.0..=clip.duration));
        });
        
        ui.horizontal(|ui| {
            ui.label("Position:");
            ui.add(egui::DragValue::new(&mut new_value.x).prefix("X: ").speed(0.1));
            ui.add(egui::DragValue::new(&mut new_value.y).prefix("Y: ").speed(0.1));
            ui.add(egui::DragValue::new(&mut new_value.z).prefix("Z: ").speed(0.1));
        });
        
        if ui.button("Add Keyframe").clicked() {
            track.add_keyframe(new_time, new_value);
        }
        
        ui.separator();
        
        // 关键帧列表
        ui.label(format!("Keyframes ({}):", track.keyframes.len()));
        
        let mut to_remove = None;
        
        for (i, keyframe) in track.keyframes.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let is_selected = self.selected_keyframe == Some(i);
                
                if ui.selectable_label(is_selected, format!("Frame {}", i)).clicked() {
                    self.selected_keyframe = Some(i);
                }
                
                ui.label(format!("Time: {:.2}s", keyframe.time));
                
                if ui.button("🗑").clicked() {
                    to_remove = Some(i);
                }
            });
            
            // 如果选中,显示编辑器
            if self.selected_keyframe == Some(i) {
                ui.indent(i, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        ui.add(egui::DragValue::new(&mut keyframe.time).suffix(" s").speed(0.1).range(0.0..=clip.duration));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.add(egui::DragValue::new(&mut keyframe.value.x).prefix("X: ").speed(0.1));
                        ui.add(egui::DragValue::new(&mut keyframe.value.y).prefix("Y: ").speed(0.1));
                        ui.add(egui::DragValue::new(&mut keyframe.value.z).prefix("Z: ").speed(0.1));
                    });
                });
            }
        }
        
        // 删除选中的关键帧
        if let Some(index) = to_remove {
            track.keyframes.remove(index);
            if self.selected_keyframe == Some(index) {
                self.selected_keyframe = None;
            }
        }
    }
    
    /// 渲染旋转轨道编辑器
    fn render_rotation_track(&mut self, ui: &mut egui::Ui, entity_id: u64, clip: &mut crate::animation::AnimationClip) {
        ui.label("Rotation Track:");
        
        // 获取或创建轨道
        let track = clip.rotation_tracks.entry(entity_id).or_insert_with(|| {
            KeyframeTrack::new(InterpolationMode::Linear)
        });
        
        // 插值模式选择
        ui.horizontal(|ui| {
            ui.label("Interpolation:");
            ui.selectable_value(&mut track.interpolation, InterpolationMode::Linear, "Linear");
            ui.selectable_value(&mut track.interpolation, InterpolationMode::Step, "Step");
        });
        
        ui.separator();
        
        // 添加关键帧
        ui.label("Add Keyframe:");
        let mut new_time = 0.0;
        let mut new_euler = Vec3::ZERO; // 欧拉角 (度)
        
        ui.horizontal(|ui| {
            ui.label("Time:");
            ui.add(egui::DragValue::new(&mut new_time).suffix(" s").speed(0.1).range(0.0..=clip.duration));
        });
        
        ui.horizontal(|ui| {
            ui.label("Rotation (degrees):");
            ui.add(egui::DragValue::new(&mut new_euler.x).prefix("X: ").speed(1.0).range(-180.0..=180.0));
            ui.add(egui::DragValue::new(&mut new_euler.y).prefix("Y: ").speed(1.0).range(-180.0..=180.0));
            ui.add(egui::DragValue::new(&mut new_euler.z).prefix("Z: ").speed(1.0).range(-180.0..=180.0));
        });
        
        if ui.button("Add Keyframe").clicked() {
            let quat = Quat::from_euler(
                glam::EulerRot::XYZ,
                new_euler.x.to_radians(),
                new_euler.y.to_radians(),
                new_euler.z.to_radians(),
            );
            track.add_keyframe(new_time, quat);
        }
        
        ui.separator();
        
        // 关键帧列表
        ui.label(format!("Keyframes ({}):", track.keyframes.len()));
        
        let mut to_remove = None;
        
        for (i, keyframe) in track.keyframes.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let is_selected = self.selected_keyframe == Some(i);
                
                if ui.selectable_label(is_selected, format!("Frame {}", i)).clicked() {
                    self.selected_keyframe = Some(i);
                }
                
                ui.label(format!("Time: {:.2}s", keyframe.time));
                
                if ui.button("🗑").clicked() {
                    to_remove = Some(i);
                }
            });
            
            // 如果选中,显示编辑器
            if self.selected_keyframe == Some(i) {
                ui.indent(i, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        ui.add(egui::DragValue::new(&mut keyframe.time).suffix(" s").speed(0.1).range(0.0..=clip.duration));
                    });
                    
                    // 转换为欧拉角进行编辑
                    let (mut x, mut y, mut z) = keyframe.value.to_euler(glam::EulerRot::XYZ);
                    x = x.to_degrees();
                    y = y.to_degrees();
                    z = z.to_degrees();
                    
                    ui.horizontal(|ui| {
                        ui.label("Rotation (degrees):");
                        if ui.add(egui::DragValue::new(&mut x).prefix("X: ").speed(1.0).range(-180.0..=180.0)).changed() ||
                           ui.add(egui::DragValue::new(&mut y).prefix("Y: ").speed(1.0).range(-180.0..=180.0)).changed() ||
                           ui.add(egui::DragValue::new(&mut z).prefix("Z: ").speed(1.0).range(-180.0..=180.0)).changed() {
                            keyframe.value = Quat::from_euler(
                                glam::EulerRot::XYZ,
                                x.to_radians(),
                                y.to_radians(),
                                z.to_radians(),
                            );
                        }
                    });
                });
            }
        }
        
        // 删除选中的关键帧
        if let Some(index) = to_remove {
            track.keyframes.remove(index);
            if self.selected_keyframe == Some(index) {
                self.selected_keyframe = None;
            }
        }
    }
    
    /// 渲染缩放轨道编辑器
    fn render_scale_track(&mut self, ui: &mut egui::Ui, entity_id: u64, clip: &mut crate::animation::AnimationClip) {
        ui.label("Scale Track:");
        
        // 获取或创建轨道
        let track = clip.scale_tracks.entry(entity_id).or_insert_with(|| {
            KeyframeTrack::new(InterpolationMode::Linear)
        });
        
        // 插值模式选择
        ui.horizontal(|ui| {
            ui.label("Interpolation:");
            ui.selectable_value(&mut track.interpolation, InterpolationMode::Linear, "Linear");
            ui.selectable_value(&mut track.interpolation, InterpolationMode::Step, "Step");
            ui.selectable_value(&mut track.interpolation, InterpolationMode::CubicBezier, "Cubic");
        });
        
        ui.separator();
        
        // 添加关键帧
        ui.label("Add Keyframe:");
        let mut new_time = 0.0;
        let mut new_value = Vec3::ONE;
        
        ui.horizontal(|ui| {
            ui.label("Time:");
            ui.add(egui::DragValue::new(&mut new_time).suffix(" s").speed(0.1).range(0.0..=clip.duration));
        });
        
        ui.horizontal(|ui| {
            ui.label("Scale:");
            ui.add(egui::DragValue::new(&mut new_value.x).prefix("X: ").speed(0.1).range(0.01..=10.0));
            ui.add(egui::DragValue::new(&mut new_value.y).prefix("Y: ").speed(0.1).range(0.01..=10.0));
            ui.add(egui::DragValue::new(&mut new_value.z).prefix("Z: ").speed(0.1).range(0.01..=10.0));
        });
        
        if ui.button("Add Keyframe").clicked() {
            track.add_keyframe(new_time, new_value);
        }
        
        ui.separator();
        
        // 关键帧列表
        ui.label(format!("Keyframes ({}):", track.keyframes.len()));
        
        let mut to_remove = None;
        
        for (i, keyframe) in track.keyframes.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let is_selected = self.selected_keyframe == Some(i);
                
                if ui.selectable_label(is_selected, format!("Frame {}", i)).clicked() {
                    self.selected_keyframe = Some(i);
                }
                
                ui.label(format!("Time: {:.2}s", keyframe.time));
                
                if ui.button("🗑").clicked() {
                    to_remove = Some(i);
                }
            });
            
            // 如果选中,显示编辑器
            if self.selected_keyframe == Some(i) {
                ui.indent(i, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        ui.add(egui::DragValue::new(&mut keyframe.time).suffix(" s").speed(0.1).range(0.0..=clip.duration));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.add(egui::DragValue::new(&mut keyframe.value.x).prefix("X: ").speed(0.1).range(0.01..=10.0));
                        ui.add(egui::DragValue::new(&mut keyframe.value.y).prefix("Y: ").speed(0.1).range(0.01..=10.0));
                        ui.add(egui::DragValue::new(&mut keyframe.value.z).prefix("Z: ").speed(0.1).range(0.01..=10.0));
                    });
                });
            }
        }
        
        // 删除选中的关键帧
        if let Some(index) = to_remove {
            track.keyframes.remove(index);
            if self.selected_keyframe == Some(index) {
                self.selected_keyframe = None;
            }
        }
    }
}

impl Default for KeyframeEditor {
    fn default() -> Self {
        Self::new()
    }
}
