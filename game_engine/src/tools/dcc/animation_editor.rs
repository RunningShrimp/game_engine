//! # DCC动画编辑器
//!
//! 提供动画编辑功能，包括：
//! - 时间轴管理
//! - 关键帧编辑
//! - 动画曲线
//! - 播放控制

use crate::animation::AnimationClip;
use egui::*;
use glam::{Quat, Vec2, Vec3};
use std::collections::{HashMap, HashSet};

/// 动画ID类型
pub type AnimationID = usize;

/// 关键帧ID类型
pub type KeyframeID = usize;

/// 播放状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// 停止
    Stopped,
    /// 播放中
    Playing,
    /// 暂停
    Paused,
}

/// 时间轴
#[derive(Debug, Clone)]
pub struct Timeline {
    /// 缩放级别
    pub zoom: f32,
    /// 滚动位置
    pub scroll: f32,
    /// 当前帧
    pub current_frame: f32,
    /// 播放状态
    pub playback_state: PlaybackState,
    /// 帧率
    pub frame_rate: f32,
    /// 开始时间
    pub start_time: f32,
    /// 结束时间
    pub end_time: f32,
    /// 循环播放
    pub loop_playback: bool,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            scroll: 0.0,
            current_frame: 0.0,
            playback_state: PlaybackState::Stopped,
            frame_rate: 60.0,
            start_time: 0.0,
            end_time: 10.0,
            loop_playback: false,
        }
    }
}

/// 动画曲线
#[derive(Debug, Clone)]
pub struct AnimationCurve {
    /// 关键帧
    pub keyframes: Vec<KeyframeData>,
    /// 曲线类型
    pub curve_type: CurveType,
    /// 曲线名称
    pub name: String,
}

/// 关键帧数据
#[derive(Debug, Clone)]
pub struct KeyframeData {
    /// 关键帧ID
    pub id: KeyframeID,
    /// 时间
    pub time: f32,
    /// 值
    pub value: AnimatedValue,
    /// 切线类型
    pub tangent_type: TangentType,
    /// 左切线
    pub tangent_in: Vec2,
    /// 右切线
    pub tangent_out: Vec2,
}

/// 动画值
#[derive(Debug, Clone)]
pub enum AnimatedValue {
    /// 浮点数
    Float(f32),
    /// 向量2
    Vec2(glam::Vec2),
    /// 向量3
    Vec3(Vec3),
    /// 四元数
    Quat(glam::Quat),
    /// 颜色
    Color([f32; 4]),
}

/// 曲线类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveType {
    /// 线性
    Linear,
    /// 阶梯
    Step,
    /// 三次样条
    Cubic,
    /// 贝塞尔
    Bezier,
}

/// 切线类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TangentType {
    /// 自动
    Auto,
    /// 自由
    Free,
    /// 线性
    Linear,
    /// 常量
    Constant,
}

/// 关键帧编辑器
#[derive(Debug, Clone)]
pub struct KeyframeEditor {
    /// 选中的关键帧
    pub selected_keys: HashSet<KeyframeID>,
    /// 显示的曲线
    pub visible_curves: HashSet<String>,
    /// 曲线颜色
    pub curve_colors: HashMap<String, egui::Color32>,
    /// 编辑模式
    pub edit_mode: KeyframeEditMode,
    /// 吸附到帧
    pub snap_to_frame: bool,
    /// 吸附阈值
    pub snap_threshold: f32,
}

/// 关键帧编辑模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyframeEditMode {
    /// 选择
    Select,
    /// 移动
    Move,
    /// 缩放
    Scale,
    /// 添加
    Add,
}

impl Default for KeyframeEditor {
    fn default() -> Self {
        Self {
            selected_keys: HashSet::new(),
            visible_curves: HashSet::new(),
            curve_colors: HashMap::new(),
            edit_mode: KeyframeEditMode::Select,
            snap_to_frame: true,
            snap_threshold: 0.1,
        }
    }
}

/// DCC动画编辑器
#[derive(Debug, Clone)]
pub struct DCCAnimationEditor {
    /// 选中的动画
    pub selected_animation: Option<AnimationID>,
    /// 动画列表
    pub animations: Vec<AnimationData>,
    /// 时间轴
    pub timeline: Timeline,
    /// 关键帧编辑器
    pub keyframe_editor: KeyframeEditor,
    /// 播放速度
    pub playback_speed: f32,
}

/// 动画数据
#[derive(Debug, Clone)]
pub struct AnimationData {
    /// 动画名称
    pub name: String,
    /// 持续时间
    pub duration: f32,
    /// 曲线
    pub curves: HashMap<String, AnimationCurve>,
    /// 循环
    pub loops: bool,
}

impl DCCAnimationEditor {
    /// 创建新的动画编辑器
    pub fn new() -> Self {
        Self {
            selected_animation: None,
            animations: Vec::new(),
            timeline: Timeline::default(),
            keyframe_editor: KeyframeEditor::default(),
            playback_speed: 1.0,
        }
    }

    /// 添加新动画
    pub fn add_animation(&mut self, name: String, duration: f32) -> AnimationID {
        let id = self.animations.len();
        let animation = AnimationData {
            name,
            duration,
            curves: HashMap::new(),
            loops: false,
        };
        self.animations.push(animation);
        id
    }

    /// 移除动画
    pub fn remove_animation(&mut self, id: AnimationID) {
        if id < self.animations.len() {
            self.animations.remove(id);

            // 更新选中状态
            if self.selected_animation == Some(id) {
                self.selected_animation = None;
            }
        }
    }

    /// 获取动画
    pub fn get_animation(&self, id: AnimationID) -> Option<&AnimationData> {
        self.animations.get(id)
    }

    /// 获取可变动画
    pub fn get_animation_mut(&mut self, id: AnimationID) -> Option<&mut AnimationData> {
        self.animations.get_mut(id)
    }

    /// 显示UI
    pub fn show_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Animation Editor")
            .default_size([800.0, 600.0])
            .show(ctx, |ui| {
                self.show_editor_ui(ui);
            });
    }

    /// 显示编辑器UI
    fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        // 播放控制
        ui.horizontal(|ui| {
            // 播放/暂停按钮
            match self.timeline.playback_state {
                PlaybackState::Stopped | PlaybackState::Paused => {
                    if ui.button("▶").clicked() {
                        self.timeline.playback_state = PlaybackState::Playing;
                    }
                }
                PlaybackState::Playing => {
                    if ui.button("⏸").clicked() {
                        self.timeline.playback_state = PlaybackState::Paused;
                    }
                }
            }

            if ui.button("⏹").clicked() {
                self.timeline.playback_state = PlaybackState::Stopped;
                self.timeline.current_frame = 0.0;
            }

            ui.separator();

            // 循环按钮
            if ui.selectable_label(self.timeline.loop_playback, "🔄").clicked() {
                self.timeline.loop_playback = !self.timeline.loop_playback;
            }

            ui.separator();

            // 当前帧显示
            ui.label(format!("Frame: {:.2}", self.timeline.current_frame));
        });

        ui.separator();

        // 时间轴
        self.show_timeline(ui);

        ui.separator();

        // 关键帧编辑器
        if let Some(anim_idx) = self.selected_animation {
            // 需要先克隆动画数据以避免借用冲突
            let animation_clone = self.animations.get(anim_idx).cloned();
            if let Some(animation) = animation_clone {
                self.show_keyframe_editor(ui, &animation);
            }
        }

        ui.separator();

        // 动画列表
        self.show_animation_list(ui);
    }

    /// 显示时间轴
    fn show_timeline(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let height = 120.0;

        let response = ui.allocate_response(
            egui::vec2(available_width, height),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter();
        let rect = response.rect;

        // 背景
        painter.rect_filled(
            rect,
            egui::Rounding::same(4),
            egui::Color32::from_rgb(40, 40, 40),
        );

        // 时间标尺
        let ruler_height = 20.0;
        let ruler_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.top()),
            egui::vec2(rect.width(), ruler_height),
        );
        painter.rect_filled(
            ruler_rect,
            egui::Rounding::ZERO,
            egui::Color32::from_rgb(60, 60, 60),
        );

        // 绘制刻度
        let frame_width = 10.0 * self.timeline.zoom;
        let start_frame = (self.timeline.scroll / frame_width).floor() as i32;
        let end_frame = start_frame + (rect.width() / frame_width).ceil() as i32 + 1;

        for frame in start_frame..=end_frame {
            let x = rect.left() + (frame as f32 * frame_width) - self.timeline.scroll;

            if x >= rect.left() && x <= rect.right() {
                painter.line(
                    vec![
                        egui::pos2(x, ruler_rect.top()),
                        egui::pos2(x, ruler_rect.bottom()),
                    ],
                    (1.0, egui::Color32::DARK_GRAY),
                );

                // 每10帧显示数字
                if frame % 10 == 0 {
                    let text = frame.to_string();
                    painter.text(
                        egui::pos2(x + 2.0, ruler_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        text,
                        egui::FontId::monospace(10.0),
                        egui::Color32::LIGHT_GRAY,
                    );
                }
            }
        }

        // 绘制播放头
        let playhead_x =
            rect.left() + (self.timeline.current_frame * frame_width) - self.timeline.scroll;
        if playhead_x >= rect.left() && playhead_x <= rect.right() {
            painter.line_segment(
                [
                    egui::pos2(playhead_x, ruler_rect.bottom()),
                    egui::pos2(playhead_x, rect.bottom()),
                ],
                (2.0, egui::Color32::RED),
            );

            // 播放头三角形
            let triangle = vec![
                egui::pos2(playhead_x - 6.0, ruler_rect.top()),
                egui::pos2(playhead_x + 6.0, ruler_rect.top()),
                egui::pos2(playhead_x, ruler_rect.top() + 8.0),
            ];
            painter.add(egui::epaint::PathShape::line(
                triangle.clone(),
                egui::Stroke::new(2.0, egui::Color32::RED),
            ));
            painter.add(egui::epaint::PathShape::convex_polygon(
                triangle,
                egui::Color32::RED,
                egui::Stroke::NONE,
            ));
        }

        // 绘制关键帧
        if let Some(anim_idx) = self.selected_animation {
            if let Some(animation) = self.animations.get(anim_idx) {
                for (curve_name, curve) in &animation.curves {
                    if self.keyframe_editor.visible_curves.contains(curve_name) {
                        for keyframe in &curve.keyframes {
                            let x =
                                rect.left() + (keyframe.time * frame_width) - self.timeline.scroll;
                            if x >= rect.left() && x <= rect.right() {
                                let color = self
                                    .keyframe_editor
                                    .curve_colors
                                    .get(curve_name)
                                    .copied()
                                    .unwrap_or(egui::Color32::YELLOW);

                                painter.circle_filled(egui::pos2(x, rect.center().y), 5.0, color);
                            }
                        }
                    }
                }
            }
        }

        // 处理交互
        if response.dragged() {
            let delta = response.drag_delta();
            self.timeline.scroll -= delta.x;

            // 限制滚动范围
            self.timeline.scroll = self.timeline.scroll.max(0.0);
        }

        if response.clicked() {
            let click_x = response.interact_pointer_pos().unwrap().x - rect.left();
            let frame = ((click_x + self.timeline.scroll) / frame_width).round();
            self.timeline.current_frame = frame.max(0.0);
        }
    }

    /// 显示关键帧编辑器
    fn show_keyframe_editor(&mut self, ui: &mut egui::Ui, animation: &AnimationData) {
        ui.label("Keyframe Editor:");

        // 编辑模式
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(
                &mut self.keyframe_editor.edit_mode,
                KeyframeEditMode::Select,
                "Select",
            );
            ui.selectable_value(
                &mut self.keyframe_editor.edit_mode,
                KeyframeEditMode::Move,
                "Move",
            );
            ui.selectable_value(
                &mut self.keyframe_editor.edit_mode,
                KeyframeEditMode::Add,
                "Add",
            );
        });

        // 吸附设置
        ui.checkbox(&mut self.keyframe_editor.snap_to_frame, "Snap to Frame");

        // 曲线列表
        ui.separator();
        ui.label("Curves:");

        for (curve_name, curve) in &animation.curves {
            let mut is_visible = self.keyframe_editor.visible_curves.contains(curve_name);

            ui.horizontal(|ui| {
                if ui.checkbox(&mut is_visible, "").changed() {
                    if is_visible {
                        self.keyframe_editor.visible_curves.insert(curve_name.clone());
                    } else {
                        self.keyframe_editor.visible_curves.remove(curve_name);
                    }
                }

                ui.label(&curve.name);
                ui.label(format!("{} keys", curve.keyframes.len()));
            });
        }

        // 关键帧列表
        ui.separator();
        ui.label(format!(
            "Selected Keys: {}",
            self.keyframe_editor.selected_keys.len()
        ));

        if ui.button("Delete Selected").clicked() {
            self.delete_selected_keyframes();
        }
    }

    /// 显示动画列表
    fn show_animation_list(&mut self, ui: &mut egui::Ui) {
        ui.label("Animations:");

        let mut selected_to_remove = None;
        for (i, animation) in self.animations.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.selected_animation == Some(i), &animation.name)
                    .clicked()
                {
                    self.selected_animation = Some(i);
                }

                if ui.button("×").clicked() {
                    selected_to_remove = Some(i);
                }
            });
        }

        if let Some(id) = selected_to_remove {
            self.remove_animation(id);
        }

        ui.separator();

        // 添加动画按钮
        if ui.button("Add Animation").clicked() {
            let name = format!("Animation {}", self.animations.len());
            self.add_animation(name, 5.0);
        }
    }

    /// 添加关键帧
    pub fn add_keyframe(
        &mut self,
        animation_id: AnimationID,
        curve_name: String,
        time: f32,
        value: AnimatedValue,
    ) -> Option<KeyframeID> {
        if let Some(animation) = self.animations.get_mut(animation_id) {
            let curve =
                animation.curves.entry(curve_name.clone()).or_insert_with(|| AnimationCurve {
                    keyframes: Vec::new(),
                    curve_type: CurveType::Linear,
                    name: curve_name.clone(),
                });

            let id = curve.keyframes.len();
            let keyframe = KeyframeData {
                id,
                time,
                value,
                tangent_type: TangentType::Auto,
                tangent_in: Vec2::ZERO,
                tangent_out: Vec2::ZERO,
            };

            curve.keyframes.push(keyframe);
            Some(id)
        } else {
            None
        }
    }

    /// 删除选中的关键帧
    pub fn delete_selected_keyframes(&mut self) {
        if let Some(anim_id) = self.selected_animation {
            if let Some(animation) = self.animations.get_mut(anim_id) {
                for (_, curve) in animation.curves.iter_mut() {
                    curve.keyframes.retain(|k| !self.keyframe_editor.selected_keys.contains(&k.id));
                }

                self.keyframe_editor.selected_keys.clear();
            }
        }
    }

    /// 更新播放
    pub fn update(&mut self, delta_time: f32) {
        if self.timeline.playback_state == PlaybackState::Playing {
            self.timeline.current_frame +=
                delta_time * self.timeline.frame_rate * self.playback_speed;

            // 检查是否到达结束时间
            let duration_frames = self.timeline.end_time * self.timeline.frame_rate;
            if self.timeline.current_frame >= duration_frames {
                if self.timeline.loop_playback {
                    self.timeline.current_frame = 0.0;
                } else {
                    self.timeline.playback_state = PlaybackState::Stopped;
                    self.timeline.current_frame = duration_frames;
                }
            }
        }
    }

    /// 获取当前时间（秒）
    pub fn get_current_time(&self) -> f32 {
        self.timeline.current_frame / self.timeline.frame_rate
    }

    /// 设置当前时间
    pub fn set_current_time(&mut self, time: f32) {
        self.timeline.current_frame = time * self.timeline.frame_rate;
    }

    /// 获取操作历史
    pub fn get_animations(&self) -> &[AnimationData] {
        &self.animations
    }
}

impl Default for DCCAnimationEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_editor_creation() {
        let editor = DCCAnimationEditor::new();
        assert!(editor.animations.is_empty());
        assert_eq!(editor.timeline.playback_state, PlaybackState::Stopped);
    }

    #[test]
    fn test_add_animation() {
        let mut editor = DCCAnimationEditor::new();
        let id = editor.add_animation("TestAnimation".to_string(), 5.0);
        assert_eq!(id, 0);
        assert_eq!(editor.animations.len(), 1);
    }

    #[test]
    fn test_add_keyframe() {
        let mut editor = DCCAnimationEditor::new();
        let anim_id = editor.add_animation("TestAnimation".to_string(), 5.0);

        let keyframe_id = editor.add_keyframe(
            anim_id,
            "position.x".to_string(),
            0.0,
            AnimatedValue::Float(0.0),
        );

        assert!(keyframe_id.is_some());
        assert_eq!(keyframe_id.unwrap(), 0);
    }

    #[test]
    fn test_playback() {
        let mut editor = DCCAnimationEditor::new();
        editor.timeline.playback_state = PlaybackState::Playing;

        editor.update(0.016); // ~60fps
        assert!(editor.timeline.current_frame > 0.0);
    }

    #[test]
    fn test_time_conversion() {
        let mut editor = DCCAnimationEditor::new();

        editor.set_current_time(1.0);
        assert!((editor.get_current_time() - 1.0).abs() < 0.001);

        editor.timeline.current_frame = 30.0;
        assert!((editor.get_current_time() - 0.5).abs() < 0.001);
    }
}
