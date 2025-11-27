use crate::animation::AnimationClip;

/// 动画编辑器
pub struct AnimationEditor {
    pub clips: Vec<AnimationClip>,
    pub selected_clip: Option<usize>,
    pub timeline_zoom: f32,
    pub playback_time: f32,
    pub is_playing: bool,
}

impl AnimationEditor {
    pub fn new() -> Self {
        Self {
            clips: Vec::new(),
            selected_clip: None,
            timeline_zoom: 1.0,
            playback_time: 0.0,
            is_playing: false,
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
                    ui.add(egui::DragValue::new(&mut clip.duration).suffix(" s").speed(0.1).range(0.1..=60.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Looping:");
                    ui.checkbox(&mut clip.looping, "");
                });
                
                ui.separator();
                
                // 播放控制
                ui.label("Playback:");
                ui.horizontal(|ui| {
                    if ui.button(if self.is_playing { "⏸ Pause" } else { "▶ Play" }).clicked() {
                        self.is_playing = !self.is_playing;
                    }
                    
                    if ui.button("⏹ Stop").clicked() {
                        self.is_playing = false;
                        self.playback_time = 0.0;
                    }
                    
                    ui.label(format!("Time: {:.2} / {:.2} s", self.playback_time, clip.duration));
                });
                
                // 更新播放时间
                if self.is_playing {
                    self.playback_time += delta_time;
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
                ui.add(egui::Slider::new(&mut self.playback_time, 0.0..=clip.duration).text("Timeline"));
                
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

impl Default for AnimationEditor {
    fn default() -> Self {
        Self::new()
    }
}
