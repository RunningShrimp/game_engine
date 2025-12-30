//! 视觉脚本编辑器
//!
//! 提供节点式的视觉脚本编辑功能：
//! - 节点创建和编辑（事件、条件、动作、变量等）
//! - 节点连接管理（输入/输出端口）
//! - 节点图可视化
//! - 脚本序列化/反序列化
//! - 脚本验证和执行
//!
//! ## 节点类型
//!
//! - **事件节点**: 脚本入口点（OnStart, OnUpdate, OnClick等）
//! - **条件节点**: 条件判断（If, Compare, Check等）
//! - **动作节点**: 执行动作（Move, Rotate, PlaySound等）
//! - **变量节点**: 变量操作（Get, Set, Math等）
//! - **流程节点**: 控制流程（Sequence, Branch, Loop等）
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::editor::VisualScriptEditor;
//!
//! let mut editor = VisualScriptEditor::new();
//! editor.add_node(NodeType::Event(EventType::OnStart), (100.0, 100.0));
//! editor.add_node(NodeType::Action(ActionType::Move), (300.0, 100.0));
//! editor.connect_nodes(0, 1, 0, 0);
//! editor.render(&mut ui);
//! ```

use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 节点类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// 事件节点
    Event(EventType),
    /// 条件节点
    Condition(ConditionType),
    /// 动作节点
    Action(ActionType),
    /// 变量节点
    Variable(VariableType),
    /// 流程节点
    Flow(FlowType),
    /// 数学运算节点
    Math(MathOperation),
}

/// 事件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// 开始事件
    OnStart,
    /// 更新事件
    OnUpdate,
    /// 点击事件
    OnClick,
    /// 碰撞事件
    OnCollision,
    /// 自定义事件
    Custom(String),
}

/// 条件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionType {
    /// 如果条件
    If,
    /// 比较
    Compare,
    /// 检查
    Check,
    /// 布尔运算
    Boolean(BooleanOp),
}

/// 动作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// 移动
    Move,
    /// 旋转
    Rotate,
    /// 播放声音
    PlaySound,
    /// 播放动画
    PlayAnimation,
    /// 设置变量
    SetVariable,
    /// 发送事件
    SendEvent,
}

/// 变量类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    /// 获取变量
    Get,
    /// 设置变量
    Set,
    /// 局部变量
    Local,
    /// 全局变量
    Global,
}

/// 流程类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowType {
    /// 序列（顺序执行）
    Sequence,
    /// 分支（条件分支）
    Branch,
    /// 循环
    Loop,
    /// 延迟
    Delay,
    /// 等待
    Wait,
}

/// 数学运算
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Min,
    Max,
    Clamp,
    Lerp,
}

/// 布尔运算
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOp {
    And,
    Or,
    Not,
    Xor,
}

/// 端口类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    /// 执行流（输入）
    ExecutionIn,
    /// 执行流（输出）
    ExecutionOut,
    /// 数据（输入）
    DataIn(DataType),
    /// 数据（输出）
    DataOut(DataType),
}

/// 数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Bool,
    Int,
    Float,
    String,
    Vector2,
    Vector3,
    Entity,
    Object,
}

/// 端口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// 端口ID（节点内唯一）
    pub id: usize,
    /// 端口类型
    pub port_type: PortType,
    /// 端口名称
    pub name: String,
    /// 端口位置（相对于节点）
    pub position: Vec2,
}

/// 节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualScriptNode {
    /// 节点ID（全局唯一）
    pub id: u64,
    /// 节点类型
    pub node_type: NodeType,
    /// 节点名称
    pub name: String,
    /// 节点位置
    pub position: Vec2,
    /// 节点大小
    pub size: Vec2,
    /// 输入端口
    pub input_ports: Vec<Port>,
    /// 输出端口
    pub output_ports: Vec<Port>,
    /// 节点数据（类型特定的参数）
    pub data: HashMap<String, String>,
    /// 是否选中
    pub selected: bool,
}

impl VisualScriptNode {
    /// 创建新节点
    pub fn new(id: u64, node_type: NodeType, position: Vec2) -> Self {
        let node_type_clone = node_type.clone();
        let (name, input_ports, output_ports) = Self::create_ports_for_type(&node_type_clone);

        Self {
            id,
            node_type,
            name,
            position,
            size: Vec2::new(150.0, 100.0),
            input_ports,
            output_ports,
            data: HashMap::new(),
            selected: false,
        }
    }

    /// 根据节点类型创建端口
    fn create_ports_for_type(node_type: &NodeType) -> (String, Vec<Port>, Vec<Port>) {
        match node_type {
            NodeType::Event(event_type) => {
                let name = match event_type {
                    EventType::OnStart => "On Start",
                    EventType::OnUpdate => "On Update",
                    EventType::OnClick => "On Click",
                    EventType::OnCollision => "On Collision",
                    EventType::Custom(s) => s.as_str(),
                };

                let output_ports = vec![Port {
                    id: 0,
                    port_type: PortType::ExecutionOut,
                    name: "Out".to_string(),
                    position: Vec2::new(150.0, 50.0),
                }];

                (name.to_string(), Vec::new(), output_ports)
            }

            NodeType::Condition(cond_type) => {
                let name = match cond_type {
                    ConditionType::If => "If",
                    ConditionType::Compare => "Compare",
                    ConditionType::Check => "Check",
                    ConditionType::Boolean(op) => match op {
                        BooleanOp::And => "And",
                        BooleanOp::Or => "Or",
                        BooleanOp::Not => "Not",
                        BooleanOp::Xor => "Xor",
                    },
                };

                let input_ports = vec![
                    Port {
                        id: 0,
                        port_type: PortType::ExecutionIn,
                        name: "In".to_string(),
                        position: Vec2::new(0.0, 30.0),
                    },
                    Port {
                        id: 1,
                        port_type: PortType::DataIn(DataType::Bool),
                        name: "Condition".to_string(),
                        position: Vec2::new(0.0, 50.0),
                    },
                ];

                let output_ports = vec![
                    Port {
                        id: 0,
                        port_type: PortType::ExecutionOut,
                        name: "True".to_string(),
                        position: Vec2::new(150.0, 30.0),
                    },
                    Port {
                        id: 1,
                        port_type: PortType::ExecutionOut,
                        name: "False".to_string(),
                        position: Vec2::new(150.0, 70.0),
                    },
                ];

                (name.to_string(), input_ports, output_ports)
            }

            NodeType::Action(action_type) => {
                let name = match action_type {
                    ActionType::Move => "Move",
                    ActionType::Rotate => "Rotate",
                    ActionType::PlaySound => "Play Sound",
                    ActionType::PlayAnimation => "Play Animation",
                    ActionType::SetVariable => "Set Variable",
                    ActionType::SendEvent => "Send Event",
                };

                let input_ports = vec![Port {
                    id: 0,
                    port_type: PortType::ExecutionIn,
                    name: "In".to_string(),
                    position: Vec2::new(0.0, 50.0),
                }];

                let output_ports = vec![Port {
                    id: 0,
                    port_type: PortType::ExecutionOut,
                    name: "Out".to_string(),
                    position: Vec2::new(150.0, 50.0),
                }];

                (name.to_string(), input_ports, output_ports)
            }

            NodeType::Variable(var_type) => {
                let name = match var_type {
                    VariableType::Get => "Get Variable",
                    VariableType::Set => "Set Variable",
                    VariableType::Local => "Local Variable",
                    VariableType::Global => "Global Variable",
                };

                let input_ports = if matches!(var_type, VariableType::Set) {
                    vec![Port {
                        id: 0,
                        port_type: PortType::DataIn(DataType::Object),
                        name: "Value".to_string(),
                        position: Vec2::new(0.0, 50.0),
                    }]
                } else {
                    Vec::new()
                };

                let output_ports = vec![Port {
                    id: 0,
                    port_type: PortType::DataOut(DataType::Object),
                    name: "Value".to_string(),
                    position: Vec2::new(150.0, 50.0),
                }];

                (name.to_string(), input_ports, output_ports)
            }

            NodeType::Flow(flow_type) => {
                let name = match flow_type {
                    FlowType::Sequence => "Sequence",
                    FlowType::Branch => "Branch",
                    FlowType::Loop => "Loop",
                    FlowType::Delay => "Delay",
                    FlowType::Wait => "Wait",
                };

                let input_ports = vec![Port {
                    id: 0,
                    port_type: PortType::ExecutionIn,
                    name: "In".to_string(),
                    position: Vec2::new(0.0, 50.0),
                }];

                let output_ports = vec![Port {
                    id: 0,
                    port_type: PortType::ExecutionOut,
                    name: "Out".to_string(),
                    position: Vec2::new(150.0, 50.0),
                }];

                (name.to_string(), input_ports, output_ports)
            }

            NodeType::Math(op) => {
                let name = match op {
                    MathOperation::Add => "Add",
                    MathOperation::Subtract => "Subtract",
                    MathOperation::Multiply => "Multiply",
                    MathOperation::Divide => "Divide",
                    MathOperation::Modulo => "Modulo",
                    MathOperation::Power => "Power",
                    MathOperation::Min => "Min",
                    MathOperation::Max => "Max",
                    MathOperation::Clamp => "Clamp",
                    MathOperation::Lerp => "Lerp",
                };

                let input_ports = vec![
                    Port {
                        id: 0,
                        port_type: PortType::DataIn(DataType::Float),
                        name: "A".to_string(),
                        position: Vec2::new(0.0, 30.0),
                    },
                    Port {
                        id: 1,
                        port_type: PortType::DataIn(DataType::Float),
                        name: "B".to_string(),
                        position: Vec2::new(0.0, 70.0),
                    },
                ];

                let output_ports = vec![Port {
                    id: 0,
                    port_type: PortType::DataOut(DataType::Float),
                    name: "Result".to_string(),
                    position: Vec2::new(150.0, 50.0),
                }];

                (name.to_string(), input_ports, output_ports)
            }
        }
    }

    /// 获取节点的边界矩形
    pub fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(self.position.x, self.position.y),
            egui::vec2(self.size.x, self.size.y),
        )
    }

    /// 获取端口的世界坐标位置
    pub fn get_port_position(&self, port_id: usize, is_input: bool) -> egui::Pos2 {
        let ports = if is_input {
            &self.input_ports
        } else {
            &self.output_ports
        };

        if let Some(port) = ports.iter().find(|p| p.id == port_id) {
            egui::pos2(
                self.position.x + port.position.x,
                self.position.y + port.position.y,
            )
        } else {
            egui::pos2(self.position.x, self.position.y)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// 执行流连接
    Execution,
    /// 数据连接
    Data(DataType),
}

/// 视觉脚本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualScript {
    /// 脚本名称
    pub name: String,
    /// 节点列表
    pub nodes: Vec<VisualScriptNode>,
    /// 连接列表
    pub connections: Vec<Connection>,
}

impl VisualScript {
    /// 创建新的视觉脚本
    pub fn new(name: String) -> Self {
        Self {
            name,
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: VisualScriptNode) -> u64 {
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// 删除节点
    pub fn remove_node(&mut self, node_id: u64) {
        self.nodes.retain(|n| n.id != node_id);
        // 删除相关连接
        self.connections.retain(|c| c.from_node != node_id && c.to_node != node_id);
    }

    /// 添加连接
    pub fn add_connection(&mut self, connection: Connection) -> Result<(), ConnectionError> {
        // 验证连接
        self.validate_connection(&connection)?;
        self.connections.push(connection);
        Ok(())
    }

    /// 删除连接
    pub fn remove_connection(&mut self, connection_id: u64) {
        self.connections.retain(|c| c.id != connection_id);
    }

    /// 验证连接
    fn validate_connection(&self, connection: &Connection) -> Result<(), ConnectionError> {
        // 检查节点是否存在
        let from_node = self
            .nodes
            .iter()
            .find(|n| n.id == connection.from_node)
            .ok_or(ConnectionError::NodeNotFound(connection.from_node))?;
        let to_node = self
            .nodes
            .iter()
            .find(|n| n.id == connection.to_node)
            .ok_or(ConnectionError::NodeNotFound(connection.to_node))?;

        // 检查端口是否存在
        let from_port = from_node
            .output_ports
            .iter()
            .find(|p| p.id == connection.from_port)
            .ok_or(ConnectionError::PortNotFound)?;
        let to_port = to_node
            .input_ports
            .iter()
            .find(|p| p.id == connection.to_port)
            .ok_or(ConnectionError::PortNotFound)?;

        // 检查连接类型是否匹配
        match (&from_port.port_type, &to_port.port_type) {
            (PortType::ExecutionOut, PortType::ExecutionIn) => Ok(()),
            (PortType::DataOut(from_type), PortType::DataIn(to_type)) => {
                if from_type == to_type {
                    Ok(())
                } else {
                    Err(ConnectionError::TypeMismatch)
                }
            }
            _ => Err(ConnectionError::InvalidConnection),
        }
    }
}

/// 连接错误
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Node not found: {0}")]
    NodeNotFound(u64),
    #[error("Port not found")]
    PortNotFound,
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("Invalid connection")]
    InvalidConnection,
}

/// 视觉脚本编辑器
#[derive(Debug)]
pub struct VisualScriptEditor {
    /// 当前脚本
    pub script: VisualScript,
    /// 当前文件路径
    current_file_path: Option<std::path::PathBuf>,
    /// 视图偏移
    pub view_offset: Vec2,
    /// 缩放
    pub zoom: f32,
    /// 是否显示网格
    pub show_grid: bool,
    /// 是否吸附到网格
    pub snap_to_grid: bool,
    /// 网格大小
    pub grid_size: f32,
    /// 下一个节点ID
    next_node_id: u64,
    /// 下一个连接ID
    next_connection_id: u64,
    /// 选中的节点ID
    selected_nodes: Vec<u64>,
    /// 正在拖拽的节点ID
    dragging_node: Option<u64>,
    /// 拖拽起始位置
    drag_start: Option<Vec2>,
    /// 正在创建的连接
    creating_connection: Option<(u64, usize, bool)>, // (node_id, port_id, is_output)
}

impl Default for VisualScriptEditor {
    fn default() -> Self {
        Self {
            script: VisualScript::new("New Script".to_string()),
            current_file_path: None,
            view_offset: Vec2::ZERO,
            zoom: 1.0,
            show_grid: true,
            snap_to_grid: true,
            grid_size: 20.0,
            next_node_id: 1,
            next_connection_id: 1,
            selected_nodes: Vec::new(),
            dragging_node: None,
            drag_start: None,
            creating_connection: None,
        }
    }
}

impl VisualScriptEditor {
    /// 创建新的视觉脚本编辑器
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建新脚本
    pub fn new_script(&mut self, name: String) {
        self.script = VisualScript::new(name);
        self.next_node_id = 1;
        self.next_connection_id = 1;
        self.selected_nodes.clear();
    }

    /// 添加节点
    pub fn add_node(&mut self, node_type: NodeType, position: Vec2) -> u64 {
        let id = self.next_node_id;
        self.next_node_id += 1;

        let mut node = VisualScriptNode::new(id, node_type, position);
        if self.snap_to_grid {
            node.position = self.snap_position(position);
        }

        self.script.add_node(node);
        id
    }

    /// 删除选中的节点
    pub fn delete_selected_nodes(&mut self) {
        for node_id in &self.selected_nodes {
            self.script.remove_node(*node_id);
        }
        self.selected_nodes.clear();
    }

    /// 添加连接
    pub fn add_connection(
        &mut self,
        from_node: u64,
        from_port: usize,
        to_node: u64,
        to_port: usize,
    ) -> Result<(), ConnectionError> {
        let id = self.next_connection_id;
        self.next_connection_id += 1;

        // 确定连接类型
        let from_node_obj = self.script.nodes.iter().find(|n| n.id == from_node);
        let connection_type = if let Some(node) = from_node_obj {
            if let Some(port) = node.output_ports.iter().find(|p| p.id == from_port) {
                match port.port_type {
                    PortType::ExecutionOut => ConnectionType::Execution,
                    PortType::DataOut(data_type) => ConnectionType::Data(data_type),
                    _ => return Err(ConnectionError::InvalidConnection),
                }
            } else {
                return Err(ConnectionError::PortNotFound);
            }
        } else {
            return Err(ConnectionError::NodeNotFound(from_node));
        };

        let connection = Connection {
            id,
            from_node,
            from_port,
            to_node,
            to_port,
            connection_type,
        };

        self.script.add_connection(connection)
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

    /// 渲染编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Visual Script Editor");
        ui.separator();

        // 工具栏
        ui.horizontal(|ui| {
            ui.label("Script:");
            ui.text_edit_singleline(&mut self.script.name);

            ui.separator();

            if ui.button("New").clicked() {
                self.new_script("New Script".to_string());
            }

            if ui.button("Save").clicked() {
                // 使用当前文件路径或默认路径
                let save_path = self.current_file_path.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("{}.json", self.script.name))
                });

                if let Err(e) = self.save_to_file(&save_path) {
                    tracing::error!("Failed to save script: {}", e);
                } else {
                    tracing::info!("Script saved to: {:?}", save_path);
                }
            }

            if ui.button("Load").clicked() {
                // 使用当前文件路径或默认路径
                let load_path = self.current_file_path.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("{}.json", self.script.name))
                });

                if let Err(e) = self.load_from_file(&load_path) {
                    tracing::error!("Failed to load script: {}", e);
                } else {
                    tracing::info!("Script loaded from: {:?}", load_path);
                }
            }

            ui.separator();

            // 节点创建菜单
            ui.menu_button("Add Node", |ui| {
                if ui.button("Event → On Start").clicked() {
                    let pos = self.view_offset + Vec2::new(100.0, 100.0);
                    self.add_node(NodeType::Event(EventType::OnStart), pos);
                }
                if ui.button("Action → Move").clicked() {
                    let pos = self.view_offset + Vec2::new(300.0, 100.0);
                    self.add_node(NodeType::Action(ActionType::Move), pos);
                }
                if ui.button("Condition → If").clicked() {
                    let pos = self.view_offset + Vec2::new(200.0, 200.0);
                    self.add_node(NodeType::Condition(ConditionType::If), pos);
                }
                if ui.button("Math → Add").clicked() {
                    let pos = self.view_offset + Vec2::new(200.0, 300.0);
                    self.add_node(NodeType::Math(MathOperation::Add), pos);
                }
            });

            ui.separator();

            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.snap_to_grid, "Snap");

            ui.label("Zoom:");
            ui.add(egui::Slider::new(&mut self.zoom, 0.1..=3.0));
        });

        ui.separator();

        // 节点图视图
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
        for node in &self.script.nodes {
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

            // 绘制节点背景
            let bg_color = if node.selected {
                egui::Color32::from_rgb(60, 80, 100)
            } else {
                egui::Color32::from_rgb(40, 50, 60)
            };
            painter.rect_filled(screen_rect, egui::CornerRadius::same(4), bg_color);
            painter.rect_stroke(
                screen_rect,
                egui::CornerRadius::same(4),
                egui::Stroke::new(2.0, egui::Color32::WHITE),
                egui::StrokeKind::Inside,
            );

            // 绘制节点标题
            let title_rect = egui::Rect::from_min_size(
                screen_rect.min,
                egui::vec2(screen_rect.width(), 25.0 * self.zoom),
            );
            painter.rect_filled(title_rect, 0.0, egui::Color32::from_rgb(30, 40, 50));
            painter.text(
                egui::pos2(
                    screen_rect.min.x + 5.0 * self.zoom,
                    screen_rect.min.y + 15.0 * self.zoom,
                ),
                egui::Align2::LEFT_CENTER,
                &node.name,
                egui::FontId::proportional(12.0 * self.zoom),
                egui::Color32::WHITE,
            );

            // 绘制输入端口
            for port in &node.input_ports {
                let port_pos = egui::pos2(
                    screen_rect.min.x + port.position.x * self.zoom,
                    screen_rect.min.y + port.position.y * self.zoom,
                );
                let port_color = match port.port_type {
                    PortType::ExecutionIn => egui::Color32::from_rgb(255, 100, 100),
                    PortType::DataIn(_) => egui::Color32::from_rgb(100, 150, 255),
                    _ => egui::Color32::WHITE,
                };
                painter.circle_filled(port_pos, 5.0 * self.zoom, port_color);
            }

            // 绘制输出端口
            for port in &node.output_ports {
                let port_pos = egui::pos2(
                    screen_rect.min.x + port.position.x * self.zoom,
                    screen_rect.min.y + port.position.y * self.zoom,
                );
                let port_color = match port.port_type {
                    PortType::ExecutionOut => egui::Color32::from_rgb(255, 100, 100),
                    PortType::DataOut(_) => egui::Color32::from_rgb(100, 150, 255),
                    _ => egui::Color32::WHITE,
                };
                painter.circle_filled(port_pos, 5.0 * self.zoom, port_color);
            }
        }
    }

    /// 绘制连接
    fn draw_connections(&self, painter: &egui::Painter) {
        for connection in &self.script.connections {
            let from_node = self.script.nodes.iter().find(|n| n.id == connection.from_node);
            let to_node = self.script.nodes.iter().find(|n| n.id == connection.to_node);

            if let (Some(from), Some(to)) = (from_node, to_node) {
                let from_pos = from.get_port_position(connection.from_port, false);
                let to_pos = to.get_port_position(connection.to_port, true);

                let screen_from = egui::pos2(
                    from_pos.x * self.zoom + self.view_offset.x,
                    from_pos.y * self.zoom + self.view_offset.y,
                );
                let screen_to = egui::pos2(
                    to_pos.x * self.zoom + self.view_offset.x,
                    to_pos.y * self.zoom + self.view_offset.y,
                );

                let color = match connection.connection_type {
                    ConnectionType::Execution => egui::Color32::from_rgb(255, 100, 100),
                    ConnectionType::Data(_) => egui::Color32::from_rgb(100, 150, 255),
                };

                // 绘制贝塞尔曲线连接
                self.draw_bezier_connection(painter, screen_from, screen_to, color);
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

    /// 三次贝塞尔曲线插值
    fn cubic_bezier(
        &self,
        p0: egui::Pos2,
        p1: egui::Pos2,
        p2: egui::Pos2,
        p3: egui::Pos2,
        t: f32,
    ) -> egui::Pos2 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        egui::pos2(
            mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
            mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
        )
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
            for node in &self.script.nodes {
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
            } else if !response.ctx.input(|i| i.modifiers.ctrl) {
                self.selected_nodes.clear();
            }

            // 更新节点选中状态
            for i in 0..self.script.nodes.len() {
                if let Some(node) = self.script.nodes.get_mut(i) {
                    node.selected = self.selected_nodes.contains(&node.id);
                }
            }
        }

        // 处理拖拽
        if response.dragged() {
            let delta = response.drag_delta();
            let graph_delta = Vec2::new(delta.x / self.zoom, delta.y / self.zoom);

            if let Some(node_id) = self.dragging_node {
                // 先找到节点索引
                let node_idx_opt = self.script.nodes.iter().position(|n| n.id == node_id);

                // 预先计算 snap_to_grid 和 grid_size，避免借用冲突
                let snap_to_grid = self.snap_to_grid;
                let grid_size = self.grid_size;

                if let Some(node_idx) = node_idx_opt
                    && node_idx < self.script.nodes.len()
                    && let Some(node) = self.script.nodes.get_mut(node_idx)
                {
                    node.position += graph_delta;
                    if snap_to_grid {
                        // 直接计算 snap_position，避免调用 self 方法
                        node.position = Vec2::new(
                            (node.position.x / grid_size).round() * grid_size,
                            (node.position.y / grid_size).round() * grid_size,
                        );
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
            for node in &self.script.nodes {
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
        if response.ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.delete_selected_nodes();
        }
    }

    /// 保存脚本到文件
    ///
    /// 将当前的VisualScript序列化为JSON格式并保存到指定路径。
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// 成功返回Ok(())，失败返回错误信息
    pub fn save_to_file(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.script)?;
        std::fs::write(path, json)?;
        self.current_file_path = Some(path.to_path_buf());
        Ok(())
    }

    /// 从文件加载脚本
    ///
    /// 从指定路径读取JSON文件并反序列化为VisualScript。
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// 成功返回Ok(())，失败返回错误信息
    ///
    /// # 说明
    ///
    /// 加载成功后，会重置编辑器状态（视图偏移、缩放、选中等），
    /// 但保留脚本内容（节点和连接）。
    pub fn load_from_file(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let script: VisualScript = serde_json::from_str(&json)?;

        // 替换当前脚本
        self.script = script;
        self.current_file_path = Some(path.to_path_buf());

        // 重置编辑器状态
        self.view_offset = Vec2::new(0.0, 0.0);
        self.zoom = 1.0;
        self.selected_nodes.clear();
        self.dragging_node = None;
        self.drag_start = None;
        self.creating_connection = None;

        // 更新节点ID计数器
        if let Some(max_id) = self.script.nodes.iter().map(|n| n.id).max() {
            self.next_node_id = max_id + 1;
        } else {
            self.next_node_id = 1;
        }

        // 更新连接ID计数器
        if let Some(max_id) = self.script.connections.iter().map(|c| c.id).max() {
            self.next_connection_id = max_id + 1;
        } else {
            self.next_connection_id = 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let mut editor = VisualScriptEditor::new();
        let node_id = editor.add_node(NodeType::Event(EventType::OnStart), Vec2::new(100.0, 100.0));
        assert_eq!(node_id, 1);
        assert_eq!(editor.script.nodes.len(), 1);
    }

    #[test]
    fn test_connection_creation() {
        let mut editor = VisualScriptEditor::new();
        let node1 = editor.add_node(NodeType::Event(EventType::OnStart), Vec2::new(100.0, 100.0));
        let node2 = editor.add_node(NodeType::Action(ActionType::Move), Vec2::new(300.0, 100.0));

        let result = editor.add_connection(node1, 0, node2, 0);
        assert!(result.is_ok());
        assert_eq!(editor.script.connections.len(), 1);
    }

    #[test]
    fn test_connection_validation() {
        let mut editor = VisualScriptEditor::new();
        let node1 = editor.add_node(NodeType::Event(EventType::OnStart), Vec2::new(100.0, 100.0));
        let node2 = editor.add_node(NodeType::Action(ActionType::Move), Vec2::new(300.0, 100.0));

        // 无效连接（端口不存在）
        let result = editor.add_connection(node1, 999, node2, 0);
        assert!(result.is_err());
    }
}
