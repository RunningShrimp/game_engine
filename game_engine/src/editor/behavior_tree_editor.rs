//! 行为树可视化编辑器
//!
//! 提供行为树的可视化编辑和预览功能：
//! - 树形结构可视化
//! - 节点创建和编辑
//! - 节点连接管理
//! - 执行状态预览
//! - 行为树序列化/反序列化
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::editor::BehaviorTreeEditor;
//!
//! let mut editor = BehaviorTreeEditor::new();
//! editor.create_tree("My Behavior Tree");
//! editor.add_node(BehaviorNodeType::Selector, (100.0, 100.0));
//! editor.render(&mut ui);
//! ```

#[cfg(feature = "ai-integration")]
use crate::ai::decision_tree_editor::{
    DecisionNodeData, DecisionNodeType, DecisionTreeEditor, DecisionTreeNode, NodeUpdates,
};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 行为树节点类型（与决策树节点类型对应）
pub type BehaviorNodeType = DecisionNodeType;

/// 节点执行状态（用于可视化）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeExecutionStatus {
    /// 未执行
    Idle,
    /// 正在执行
    Running,
    /// 执行成功
    Success,
    /// 执行失败
    Failure,
}

/// 行为树可视化节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualBehaviorNode {
    /// 节点ID
    pub id: u64,
    /// 节点类型
    pub node_type: BehaviorNodeType,
    /// 节点名称
    pub name: String,
    /// 节点位置
    pub position: Vec2,
    /// 节点大小
    pub size: Vec2,
    /// 子节点ID列表
    pub children: Vec<u64>,
    /// 执行状态（用于预览）
    pub execution_status: NodeExecutionStatus,
    /// 是否选中
    pub selected: bool,
    /// 节点数据
    pub data: DecisionNodeData,
}

impl VisualBehaviorNode {
    /// 从决策树节点创建可视化节点
    pub fn from_decision_node(node: &DecisionTreeNode) -> Self {
        Self {
            id: node.id,
            node_type: node.node_type,
            name: node.name.clone(),
            position: Vec2::new(node.position.0, node.position.1),
            size: Vec2::new(120.0, 60.0),
            children: node.children.clone(),
            execution_status: NodeExecutionStatus::Idle,
            selected: false,
            data: node.data.clone(),
        }
    }

    /// 获取节点的边界矩形
    pub fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(self.position.x, self.position.y),
            egui::vec2(self.size.x, self.size.y),
        )
    }

    /// 获取节点颜色（基于类型和执行状态）
    pub fn color(&self) -> egui::Color32 {
        match self.execution_status {
            NodeExecutionStatus::Running => egui::Color32::from_rgb(255, 200, 0),
            NodeExecutionStatus::Success => egui::Color32::from_rgb(0, 255, 0),
            NodeExecutionStatus::Failure => egui::Color32::from_rgb(255, 0, 0),
            NodeExecutionStatus::Idle => match self.node_type {
                BehaviorNodeType::Selector | BehaviorNodeType::Sequence => {
                    egui::Color32::from_rgb(100, 150, 255)
                }
                BehaviorNodeType::Decorator => egui::Color32::from_rgb(150, 100, 255),
                BehaviorNodeType::Condition => egui::Color32::from_rgb(255, 150, 100),
                BehaviorNodeType::Action => egui::Color32::from_rgb(100, 255, 150),
            },
        }
    }

    /// 获取节点图标文本
    pub fn icon(&self) -> &'static str {
        match self.node_type {
            BehaviorNodeType::Selector => "?",
            BehaviorNodeType::Sequence => "→",
            BehaviorNodeType::Decorator => "◇",
            BehaviorNodeType::Condition => "?",
            BehaviorNodeType::Action => "▶",
        }
    }
}

/// 连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// 连接ID
    pub id: u64,
    /// 源节点ID
    pub from_node: u64,
    /// 源端口ID
    pub from_port: usize,
    /// 目标节点ID
    pub to_node: u64,
    /// 目标端口ID
    pub to_port: usize,
    /// 连接类型（执行流或数据）
    pub connection_type: ConnectionType,
}

/// 连接类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// 执行流连接
    Execution,
    /// 数据连接
    Data(String), // 使用 String 表示数据类型
}

/// 行为树可视化编辑器
#[derive(Debug)]
pub struct BehaviorTreeEditor {
    /// 决策树编辑器（底层数据）
    decision_editor: DecisionTreeEditor,
    /// 可视化节点映射
    visual_nodes: HashMap<u64, VisualBehaviorNode>,
    /// 视图偏移
    view_offset: Vec2,
    /// 缩放
    zoom: f32,
    /// 是否显示网格
    show_grid: bool,
    /// 是否吸附到网格
    snap_to_grid: bool,
    /// 网格大小
    grid_size: f32,
    /// 选中的节点ID
    selected_nodes: Vec<u64>,
    /// 正在拖拽的节点ID
    dragging_node: Option<u64>,
    /// 拖拽起始位置
    drag_start: Option<Vec2>,
    /// 是否启用执行预览
    preview_enabled: bool,
    /// 执行状态映射（节点ID -> 执行状态）
    execution_states: HashMap<u64, NodeExecutionStatus>,
}

impl Default for BehaviorTreeEditor {
    fn default() -> Self {
        Self {
            decision_editor: DecisionTreeEditor::new(),
            visual_nodes: HashMap::new(),
            view_offset: Vec2::ZERO,
            zoom: 1.0,
            show_grid: true,
            snap_to_grid: true,
            grid_size: 30.0,
            selected_nodes: Vec::new(),
            dragging_node: None,
            drag_start: None,
            preview_enabled: false,
            execution_states: HashMap::new(),
        }
    }
}

impl BehaviorTreeEditor {
    /// 创建新的行为树编辑器
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建新行为树
    pub fn create_tree(&mut self, name: String) -> Result<(), String> {
        self.decision_editor.create_tree(name).map_err(|e| e.to_string())?;
        self.sync_visual_nodes();
        Ok(())
    }

    /// 加载行为树
    pub fn load_tree(&mut self, name: &str) -> Result<(), String> {
        self.decision_editor.load_tree(name).map_err(|e| e.to_string())?;
        self.sync_visual_nodes();
        Ok(())
    }

    /// 保存当前行为树
    pub fn save_current_tree(&mut self) -> Result<(), String> {
        self.decision_editor.save_current_tree().map_err(|e| e.to_string())
    }

    /// 添加节点
    pub fn add_node(&mut self, node_type: BehaviorNodeType, position: Vec2) -> Result<u64, String> {
        let tree = self.decision_editor.get_current_tree_mut().ok_or("No current tree")?;

        let name = match node_type {
            BehaviorNodeType::Selector => "Selector",
            BehaviorNodeType::Sequence => "Sequence",
            BehaviorNodeType::Decorator => "Decorator",
            BehaviorNodeType::Condition => "Condition",
            BehaviorNodeType::Action => "Action",
        };

        // 预先计算吸附位置，避免借用冲突
        let snapped_pos = if self.snap_to_grid {
            let grid_size = self.grid_size;
            Vec2::new(
                (position.x / grid_size).round() * grid_size,
                (position.y / grid_size).round() * grid_size,
            )
        } else {
            position
        };

        let id = tree.add_node(node_type, name.to_string(), (snapped_pos.x, snapped_pos.y));

        self.sync_visual_nodes();
        Ok(id)
    }

    /// 删除选中的节点
    pub fn delete_selected_nodes(&mut self) -> Result<(), String> {
        let tree = self.decision_editor.get_current_tree_mut().ok_or("No current tree")?;

        for node_id in &self.selected_nodes {
            tree.remove_node(*node_id).map_err(|e| e.to_string())?;
        }

        self.selected_nodes.clear();
        self.sync_visual_nodes();
        Ok(())
    }

    /// 添加子节点
    pub fn add_child(&mut self, parent_id: u64, child_id: u64) -> Result<(), String> {
        let tree = self.decision_editor.get_current_tree_mut().ok_or("No current tree")?;

        tree.add_child(parent_id, child_id).map_err(|e| e.to_string())?;
        self.sync_visual_nodes();
        Ok(())
    }

    /// 移除子节点
    pub fn remove_child(&mut self, parent_id: u64, child_id: u64) -> Result<(), String> {
        let tree = self.decision_editor.get_current_tree_mut().ok_or("No current tree")?;

        tree.remove_child(parent_id, child_id).map_err(|e| e.to_string())?;
        self.sync_visual_nodes();
        Ok(())
    }

    /// 更新节点执行状态（用于预览）
    pub fn update_execution_status(&mut self, node_id: u64, status: NodeExecutionStatus) {
        // 更新执行状态到映射
        self.execution_states.insert(node_id, status);

        // 然后更新可视化节点
        if let Some(node) = self.visual_nodes.get_mut(&node_id) {
            node.execution_status = status;
        }
    }

    /// 清除所有执行状态
    pub fn clear_execution_states(&mut self) {
        self.execution_states.clear();
        for node in self.visual_nodes.values_mut() {
            node.execution_status = NodeExecutionStatus::Idle;
        }
    }

    /// 同步可视化节点（从决策树同步）
    fn sync_visual_nodes(&mut self) {
        if let Some(tree) = self.decision_editor.get_current_tree() {
            self.visual_nodes.clear();
            for (id, node) in tree.get_all_nodes() {
                let visual_node = VisualBehaviorNode::from_decision_node(node);
                self.visual_nodes.insert(*id, visual_node);
            }
        }
    }

    /// 吸附位置到网格
    fn snap_position(&self, position: Vec2) -> Vec2 {
        if self.snap_to_grid {
            Vec2::new(
                (position.x / self.grid_size).round() * self.grid_size,
                (position.y / self.grid_size).round() * self.grid_size,
            )
        } else {
            position
        }
    }

    /// 计算三次贝塞尔曲线点
    fn cubic_bezier(
        &self,
        p0: egui::Pos2,
        p1: egui::Pos2,
        p2: egui::Pos2,
        p3: egui::Pos2,
        t: f32,
    ) -> egui::Pos2 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        egui::pos2(
            uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
            uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
        )
    }

    /// 渲染编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Behavior Tree Editor");
        ui.separator();

        // 工具栏
        ui.horizontal(|ui| {
            if let Some(tree) = self.decision_editor.get_current_tree() {
                ui.label(format!("Tree: {}", tree.name));
            } else {
                ui.label("No tree loaded");
            }
        });

        ui.separator();

        if ui.button("New").clicked()
            && let Err(e) = self.create_tree("New Behavior Tree".to_string())
        {
            tracing::warn!("Failed to create tree: {}", e);
        }

        if ui.button("Save").clicked()
            && let Err(e) = self.save_current_tree()
        {
            tracing::warn!("Failed to save tree: {}", e);
        }

        ui.separator();

        ui.menu_button("Add Node", |ui| {
            if ui.button("Selector").clicked() {
                let pos = self.view_offset + Vec2::new(100.0, 100.0);
                let _ = self.add_node(BehaviorNodeType::Selector, pos);
            }
            if ui.button("Sequence").clicked() {
                let pos = self.view_offset + Vec2::new(100.0, 100.0);
                let _ = self.add_node(BehaviorNodeType::Sequence, pos);
            }
            if ui.button("Decorator").clicked() {
                let pos = self.view_offset + Vec2::new(100.0, 100.0);
                let _ = self.add_node(BehaviorNodeType::Decorator, pos);
            }
            if ui.button("Condition").clicked() {
                let pos = self.view_offset + Vec2::new(100.0, 100.0);
                let _ = self.add_node(BehaviorNodeType::Condition, pos);
            }
            if ui.button("Action").clicked() {
                let pos = self.view_offset + Vec2::new(100.0, 100.0);
                let _ = self.add_node(BehaviorNodeType::Action, pos);
            }
        });

        ui.separator();

        ui.checkbox(&mut self.show_grid, "Grid");
        ui.checkbox(&mut self.snap_to_grid, "Snap");
        ui.checkbox(&mut self.preview_enabled, "Preview");

        ui.label("Zoom:");
        ui.add(egui::Slider::new(&mut self.zoom, 0.1..=3.0));

        ui.separator();

        // 树视图
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), ui.available_height() - 100.0),
            egui::Sense::click_and_drag(),
        );

        let rect = response.rect;

        // 绘制背景
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));

        // 绘制网格
        if self.show_grid {
            self.draw_grid(&painter, rect);
        }

        // 绘制连接
        self.draw_connections(&painter);

        // 绘制节点
        self.draw_nodes(&painter, rect);

        // 处理交互
        self.handle_interaction(&response, rect);
    }

    /// 绘制网格
    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let grid_size = self.grid_size * self.zoom;
        let grid_color = egui::Color32::from_gray(50);

        // 垂直网格线
        let start_x = (rect.left() - self.view_offset.x * self.zoom) % grid_size;
        let mut x = rect.left() - start_x;
        while x < rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
            x += grid_size;
        }

        // 水平网格线
        let start_y = (rect.top() - self.view_offset.y * self.zoom) % grid_size;
        let mut y = rect.top() - start_y;
        while y < rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, grid_color),
            );
            y += grid_size;
        }
    }

    /// 绘制节点
    fn draw_nodes(&self, painter: &egui::Painter, viewport: egui::Rect) {
        for node in self.visual_nodes.values() {
            let node_rect = node.rect();
            let screen_rect = egui::Rect::from_min_size(
                egui::pos2(
                    node_rect.min.x * self.zoom + self.view_offset.x,
                    node_rect.min.y * self.zoom + self.view_offset.y,
                ),
                egui::vec2(
                    node_rect.width() * self.zoom,
                    node_rect.height() * self.zoom,
                ),
            );

            // 只绘制可见的节点
            if !viewport.intersects(screen_rect) {
                continue;
            }

            let color = node.color();
            let bg_color = if node.selected {
                egui::Color32::from_rgb(60, 80, 100)
            } else {
                color
            };

            // 绘制节点背景
            painter.rect_filled(screen_rect, egui::CornerRadius::same(4), bg_color);
            painter.rect_stroke(
                screen_rect,
                egui::CornerRadius::same(4),
                egui::Stroke::new(2.0, egui::Color32::WHITE),
                egui::StrokeKind::Inside,
            );

            // 绘制节点图标
            painter.text(
                egui::pos2(
                    screen_rect.min.x + 10.0 * self.zoom,
                    screen_rect.min.y + 20.0 * self.zoom,
                ),
                egui::Align2::LEFT_CENTER,
                node.icon(),
                egui::FontId::proportional(20.0 * self.zoom),
                egui::Color32::WHITE,
            );

            // 绘制节点名称
            let name = node.name.clone();
            painter.text(
                egui::pos2(
                    screen_rect.min.x + 30.0 * self.zoom,
                    screen_rect.min.y + 20.0 * self.zoom,
                ),
                egui::Align2::LEFT_CENTER,
                &name,
                egui::FontId::proportional(12.0 * self.zoom),
                egui::Color32::WHITE,
            );

            // 绘制执行状态指示器
            if self.preview_enabled {
                let status_text = match node.execution_status {
                    NodeExecutionStatus::Running => "●",
                    NodeExecutionStatus::Success => "✓",
                    NodeExecutionStatus::Failure => "✗",
                    NodeExecutionStatus::Idle => "",
                };
                if !status_text.is_empty() {
                    painter.text(
                        egui::pos2(
                            screen_rect.max.x - 15.0 * self.zoom,
                            screen_rect.min.y + 15.0 * self.zoom,
                        ),
                        egui::Align2::RIGHT_CENTER,
                        status_text,
                        egui::FontId::proportional(16.0 * self.zoom),
                        egui::Color32::WHITE,
                    );
                }
            }
        }
    }

    /// 绘制连接
    fn draw_connections(&self, painter: &egui::Painter) {
        for node in self.visual_nodes.values() {
            let parent_rect = node.rect();
            let parent_screen = egui::pos2(
                parent_rect.center().x * self.zoom + self.view_offset.x,
                parent_rect.max.y * self.zoom + self.view_offset.y,
            );

            for &child_id in &node.children {
                if let Some(child) = self.visual_nodes.get(&child_id) {
                    let child_rect = child.rect();
                    let child_screen = egui::pos2(
                        child_rect.center().x * self.zoom + self.view_offset.x,
                        child_rect.min.y * self.zoom + self.view_offset.y,
                    );

                    // 绘制连接线
                    painter.line_segment(
                        [parent_screen, child_screen],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(150, 150, 150)),
                    );
                }
            }
        }
    }

    /// 绘制贝塞尔曲线连接
    fn draw_bezier_connection(
        &self,
        painter: &egui::Painter,
        from: egui::Pos2,
        to: egui::Pos2,
        color: egui::Color32,
    ) {
        let dx = (to.x - from.x).abs();
        let cp1 = egui::pos2(from.x + dx * 0.5, from.y);
        let cp2 = egui::pos2(to.x - dx * 0.5, to.y);

        let mut points = Vec::new();
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            let point = self.cubic_bezier(from, cp1, cp2, to, t);
            points.push(point);
        }

        painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, color)));
    }

    /// 处理交互
    fn handle_interaction(&mut self, response: &egui::Response, _viewport: egui::Rect) {
        // 处理点击
        if response.clicked()
            && let Some(click_pos) = response.interact_pointer_pos()
        {
            let graph_pos = egui::pos2(
                (click_pos.x - self.view_offset.x) / self.zoom,
                (click_pos.y - self.view_offset.y) / self.zoom,
            );

            // 检查是否点击了节点
            let mut clicked_node = None;
            for node in self.visual_nodes.values() {
                if node.rect().contains(graph_pos) {
                    clicked_node = Some(node.id);
                    break;
                }
            }

            if let Some(node_id) = clicked_node {
                if !response.ctx.input(|i| i.modifiers.ctrl) {
                    self.selected_nodes.clear();
                }
                if !self.selected_nodes.contains(&node_id) {
                    self.selected_nodes.push(node_id);
                }
            }
        }

        // 处理拖拽
        if response.dragged() {
            let delta = response.drag_delta();
            let graph_delta = Vec2::new(delta.x / self.zoom, delta.y / self.zoom);

            if let Some(dragging_node_id) = self.dragging_node {
                // 预先计算 snap_to_grid 和 grid_size，避免借用冲突
                let snap_enabled = self.snap_to_grid;
                let grid_size = self.grid_size;

                // 预先获取节点的当前位置，避免借用冲突
                let current_position = self.visual_nodes.get(&dragging_node_id).map(|n| n.position);

                // 更新节点位置
                if let Some(pos) = current_position {
                    let mut new_position = pos + graph_delta;

                    // 应用网格吸附
                    if snap_enabled {
                        new_position = Vec2::new(
                            (new_position.x / grid_size).round() * grid_size,
                            (new_position.y / grid_size).round() * grid_size,
                        );
                    }

                    // 更新可视化节点位置
                    if let Some(node) = self.visual_nodes.get_mut(&dragging_node_id) {
                        node.position = new_position;
                    }

                    // 同步到决策树
                    if let Some(tree) = self.decision_editor.get_current_tree_mut() {
                        let updates = NodeUpdates {
                            name: None,
                            description: None,
                            position: Some((new_position.x, new_position.y)),
                            data: None,
                        };
                        if let Err(e) = tree.update_node(dragging_node_id, updates) {
                            tracing::warn!("Failed to update node position: {}", e);
                        }
                    }
                }
            } else {
                // 拖拽视图
                self.view_offset += Vec2::new(delta.x, delta.y);
            }
        }

        // 开始拖拽
        if response.drag_started()
            && let Some(click_pos) = response.interact_pointer_pos()
        {
            let graph_pos = egui::pos2(
                (click_pos.x - self.view_offset.x) / self.zoom,
                (click_pos.y - self.view_offset.y) / self.zoom,
            );

            // 检查是否开始拖拽节点
            for node in self.visual_nodes.values() {
                if node.rect().contains(graph_pos) {
                    self.dragging_node = Some(node.id);
                    self.drag_start = Some(Vec2::new(graph_pos.x, graph_pos.y));
                    break;
                }
            }
        }

        // 结束拖拽
        if response.drag_stopped() {
            self.dragging_node = None;
            self.drag_start = None;
        }

        // 处理删除键
        if response.ctx.input(|i| i.key_pressed(egui::Key::Delete))
            && let Err(e) = self.delete_selected_nodes()
        {
            tracing::warn!("Failed to delete nodes: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_tree_editor_creation() {
        let mut editor = BehaviorTreeEditor::new();
        editor
            .create_tree("Test Tree".to_string())
            .expect("Failed to create tree in test");
        assert!(editor.decision_editor.get_current_tree().is_some());
    }

    #[test]
    fn test_add_node() {
        let mut editor = BehaviorTreeEditor::new();
        editor
            .create_tree("Test Tree".to_string())
            .expect("Failed to create tree in test");
        let result = editor.add_node(BehaviorNodeType::Selector, Vec2::new(100.0, 100.0));
        assert!(result.is_ok());
        assert_eq!(editor.visual_nodes.len(), 1);
    }
}
