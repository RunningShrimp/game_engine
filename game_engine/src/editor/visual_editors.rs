//  可视化编辑器工具集
//
//  提供游戏开发所需的各种可视化编辑器：
//  - 着色器编辑器（节点-based）
//  - 动画状态机编辑器
//  - 粒子系统编辑器
//  - 性能剖析器集成
//
//  ## 设计理念
//
//  1. **实时预览**
//     - 所见即所得
//     - 热重载支持
//
//  2. **节点-based编辑**
//     - 直观的图形界面
//     - 拖拽连接
//
//  3. **版本控制友好**
//     - 文本序列化
//     - Diff友好

use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 着色器编辑器
// ============================================================================

/// 着色器节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderNodeType {
    // 输入节点
    Input,
    // 输出节点
    Output,
    // 数学运算
    Add,
    Subtract,
    Multiply,
    Divide,
    Sin,
    Cos,
    // 向量运算
    Dot,
    Cross,
    Normalize,
    // 纹理采样
    Texture2D,
    TextureCube,
    // 常量
    Float,
    Vec2,
    Vec3,
    Vec4,
    // 自定义
    Custom,
}

/// 着色器节点
#[derive(Debug, Clone)]
pub struct ShaderNode {
    /// 节点ID
    pub id: usize,
    /// 节点类型
    pub node_type: ShaderNodeType,
    /// 节点位置
    pub position: Vec2,
    /// 节点名称
    pub name: String,
    /// 输入端口
    pub inputs: Vec<NodePort>,
    /// 输出端口
    pub outputs: Vec<NodePort>,
    /// 节点数据
    pub data: ShaderNodeData,
}

/// 节点端口
#[derive(Debug, Clone)]
pub struct NodePort {
    /// 端口名称
    pub name: String,
    /// 数据类型
    pub data_type: ShaderDataType,
    /// 连接的节点（如果有）
    pub connection: Option<usize>,
}

/// 着色器数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderDataType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Texture2D,
    TextureCube,
}

/// 节点数据
#[derive(Debug, Clone)]
pub enum ShaderNodeData {
    None,
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Texture(String),
    Custom(HashMap<String, String>),
}

/// 着色器连接
#[derive(Debug, Clone)]
pub struct ShaderConnection {
    /// 源节点ID
    pub from_node: usize,
    /// 源端口索引
    pub from_port: usize,
    /// 目标节点ID
    pub to_node: usize,
    /// 目标端口索引
    pub to_port: usize,
}

/// 着色器图
#[derive(Debug, Clone)]
pub struct ShaderGraph {
    /// 节点列表
    pub nodes: Vec<ShaderNode>,
    /// 连接列表
    pub connections: Vec<ShaderConnection>,
    /// 输出节点ID
    pub output_node: Option<usize>,
    /// 图名称
    pub name: String,
}

impl ShaderGraph {
    /// 创建新的着色器图
    pub fn new(name: String) -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            output_node: None,
            name,
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: ShaderNode) -> usize {
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// 连接节点
    pub fn connect(&mut self, from_node: usize, from_port: usize, to_node: usize, to_port: usize) {
        let connection = ShaderConnection {
            from_node,
            from_port,
            to_node,
            to_port,
        };

        self.connections.push(connection);

        // 更新目标节点的输入连接
        if let Some(node) = self.nodes.get_mut(to_node)
            && let Some(input) = node.inputs.get_mut(to_port)
        {
            input.connection = Some(from_node);
        }
    }

    /// 生成着色器代码
    pub fn generate_shader_code(&self) -> Result<String, String> {
        let mut code = String::new();

        // 简化实现：生成基础结构
        code.push_str("// Generated Shader\n");
        code.push_str(&format!("// {}\n", self.name));

        // TODO: 实现完整的代码生成逻辑
        // 需要拓扑排序、类型推导、代码生成

        Ok(code)
    }
}

// ============================================================================
// 动画状态机编辑器
// ============================================================================

/// 动画状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationState {
    /// 状态ID
    pub id: String,
    /// 状态名称
    pub name: String,
    /// 动画剪辑
    pub animation_clip: String,
    /// 是否循环
    pub loop_: bool,
    /// 播放速度
    pub speed: f32,
    /// 混合时间
    pub blend_time: f32,
    /// 状态位置（用于UI显示）
    pub position: Vec2,
}

/// 状态转换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// 源状态
    pub from_state: String,
    /// 目标状态
    pub to_state: String,
    /// 转换条件
    pub condition: TransitionCondition,
    /// 转换时间
    pub duration: f32,
    /// 转换模式
    pub blend_mode: BlendMode,
}

/// 转换条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    /// 时间条件
    Time { min_time: f32 },
    /// 参数条件
    Parameter {
        param: String,
        value: f32,
        comparison: ComparisonOp,
    },
    /// 事件触发
    Event { event: String },
    /// 自定义条件
    Custom { condition: String },
}

/// 比较操作符
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

/// 混合模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlendMode {
    /// 线性混合
    Linear,
    /// 调度混合
    Synchronized,
}

/// 动画状态机
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationStateMachine {
    /// 状态机ID
    pub id: String,
    /// 状态机名称
    pub name: String,
    /// 所有状态
    pub states: Vec<AnimationState>,
    /// 所有转换
    pub transitions: Vec<StateTransition>,
    /// 初始状态
    pub initial_state: String,
    /// 当前状态
    pub current_state: String,
    /// 参数（用于条件判断）
    pub parameters: HashMap<String, f32>,
}

impl AnimationStateMachine {
    /// 创建新的状态机
    pub fn new(id: String, name: String, initial_state: String) -> Self {
        Self {
            id,
            name,
            states: Vec::new(),
            transitions: Vec::new(),
            initial_state: initial_state.clone(),
            current_state: initial_state,
            parameters: HashMap::new(),
        }
    }

    /// 添加状态
    pub fn add_state(&mut self, state: AnimationState) {
        self.states.push(state);
    }

    /// 添加转换
    pub fn add_transition(&mut self, transition: StateTransition) {
        self.transitions.push(transition);
    }

    /// 设置参数
    pub fn set_parameter(&mut self, name: String, value: f32) {
        self.parameters.insert(name, value);
    }

    /// 更新状态机
    pub fn update(&mut self, dt: f32, events: &[String]) -> Vec<String> {
        let mut triggered_transitions = Vec::new();

        // 检查所有转换条件
        for transition in &self.transitions {
            if transition.from_state == self.current_state
                && self.check_transition_condition(transition, dt, events)
            {
                // 触发转换
                self.current_state = transition.to_state.clone();
                triggered_transitions.push(transition.to_state.clone());
            }
        }

        triggered_transitions
    }

    /// 检查转换条件
    fn check_transition_condition(
        &self,
        transition: &StateTransition,
        _dt: f32,
        events: &[String],
    ) -> bool {
        match &transition.condition {
            TransitionCondition::Time { min_time: _ } => {
                // TODO: 需要跟踪当前状态的时间
                true // 简化实现
            }
            TransitionCondition::Parameter {
                param,
                value,
                comparison,
            } => {
                if let Some(&param_value) = self.parameters.get(param) {
                    match comparison {
                        ComparisonOp::Equal => (param_value - *value).abs() < 0.001,
                        ComparisonOp::NotEqual => (param_value - *value).abs() >= 0.001,
                        ComparisonOp::Greater => param_value > *value,
                        ComparisonOp::Less => param_value < *value,
                        ComparisonOp::GreaterEqual => param_value >= *value,
                        ComparisonOp::LessEqual => param_value <= *value,
                    }
                } else {
                    false
                }
            }
            TransitionCondition::Event { event } => events.contains(event),
            TransitionCondition::Custom { condition: _ } => {
                // TODO: 解析和执行自定义条件
                false
            }
        }
    }

    /// 导出为JSON
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// 从JSON导入
    pub fn import_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

// ============================================================================
// 粒子系统编辑器
// ============================================================================

/// 粒子发射器类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EmitterType {
    /// 点发射器
    Point,
    /// 球形发射器
    Sphere,
    /// 圆柱发射器
    Cylinder,
    /// 圆锥发射器
    Cone,
    /// 盒子发射器
    Box,
}

/// 粒子发射器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitterConfig {
    /// 发射器类型
    pub emitter_type: EmitterType,
    /// 发射速率（粒子/秒）
    pub rate: f32,
    /// 粒子寿命（秒）
    pub lifetime: f32,
    /// 寿命随机范围
    pub lifetime_variance: f32,
    /// 初始速度
    pub initial_velocity: Vec3,
    /// 速度随机范围
    pub velocity_variance: Vec3,
    /// 初始颜色
    pub initial_color: Vec4,
    /// 结束颜色
    pub end_color: Vec4,
    /// 初始大小
    pub initial_size: f32,
    /// 结束大小
    pub end_size: f32,
    /// 重力
    pub gravity: Vec3,
    /// 空气阻力
    pub drag: f32,
    /// 发射器尺寸
    pub emitter_size: Vec3,
    /// 是否循环
    pub looping: bool,
    /// 持续时间（0表示无限）
    pub duration: f32,
}

impl Default for ParticleEmitterConfig {
    fn default() -> Self {
        Self {
            emitter_type: EmitterType::Point,
            rate: 100.0,
            lifetime: 2.0,
            lifetime_variance: 0.5,
            initial_velocity: Vec3::Y,
            velocity_variance: Vec3::new(0.1, 0.1, 0.1),
            initial_color: Vec4::ONE,
            end_color: Vec4::ONE,
            initial_size: 1.0,
            end_size: 0.0,
            gravity: Vec3::NEG_Y,
            drag: 0.0,
            emitter_size: Vec3::ONE,
            looping: true,
            duration: 0.0,
        }
    }
}

/// 粒子系统数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystemData {
    /// 系统ID
    pub id: String,
    /// 系统名称
    pub name: String,
    /// 发射器配置
    pub emitter_config: ParticleEmitterConfig,
    /// 粒子纹理
    pub texture: Option<String>,
    /// 最大粒子数
    pub max_particles: usize,
    /// 是否启用
    pub enabled: bool,
    /// 预览配置（用于编辑器）
    pub preview_config: PreviewConfig,
}

/// 预览配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    /// 预览颜色
    pub background_color: Vec4,
    /// 网格显示
    pub show_grid: bool,
    /// 网格大小
    pub grid_size: f32,
    /// 发射器显示
    pub show_emitter: bool,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            background_color: Vec4::new(0.2, 0.2, 0.2, 1.0),
            show_grid: true,
            grid_size: 10.0,
            show_emitter: true,
        }
    }
}

impl ParticleSystemData {
    /// 创建新的粒子系统
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            emitter_config: ParticleEmitterConfig::default(),
            texture: None,
            max_particles: 1000,
            enabled: true,
            preview_config: PreviewConfig::default(),
        }
    }

    /// 导出为JSON
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// 从JSON导入
    pub fn import_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

// ============================================================================
// 编辑器管理器
// ============================================================================

/// 编辑器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorType {
    /// 着色器编辑器
    Shader,
    /// 动画状态机编辑器
    AnimationStateMachine,
    /// 粒子系统编辑器
    ParticleSystem,
    /// 性能剖析器
    Profiler,
}

/// 编辑器管理器
pub struct EditorManager {
    /// 当前打开的编辑器
    open_editors: HashMap<EditorType, Box<dyn Editor + Send>>,
    /// 活跃编辑器
    active_editor: Option<EditorType>,
}

/// 编辑器trait
pub trait Editor {
    /// 获取编辑器类型
    fn editor_type(&self) -> EditorType;

    /// 渲染编辑器UI
    fn render_ui(&mut self, ui: &mut egui::Ui) -> bool;

    /// 处理输入
    fn handle_input(&mut self, input: &EditorInput);

    /// 更新编辑器
    fn update(&mut self, dt: f32) -> EditorUpdateResult;
}

/// 编辑器输入
#[derive(Debug, Clone)]
pub struct EditorInput {
    /// 鼠标位置
    pub mouse_pos: Vec2,
    /// 鼠标点击
    pub mouse_clicked: bool,
    /// 键盘输入
    pub keyboard: Vec<String>,
    /// 其他输入
    pub other: HashMap<String, String>,
}

/// 编辑器更新结果
#[derive(Debug, Clone)]
pub enum EditorUpdateResult {
    /// 无变化
    None,
    /// 数据已修改
    Modified,
    /// 需要保存
    NeedsSave,
    /// 保存完成
    Saved,
}

impl EditorManager {
    /// 创建新的编辑器管理器
    pub fn new() -> Self {
        Self {
            open_editors: HashMap::new(),
            active_editor: None,
        }
    }

    /// 打开编辑器
    pub fn open_editor(&mut self, editor_type: EditorType, editor: Box<dyn Editor + Send>) {
        self.open_editors.insert(editor_type, editor);
        self.active_editor = Some(editor_type);
    }

    /// 关闭编辑器
    pub fn close_editor(&mut self, editor_type: EditorType) {
        self.open_editors.remove(&editor_type);
        if self.active_editor == Some(editor_type) {
            self.active_editor = None;
        }
    }

    /// 渲染活跃编辑器
    pub fn render_active_editor(&mut self, ui: &mut egui::Ui) {
        if let Some(editor_type) = self.active_editor
            && let Some(editor) = self.open_editors.get_mut(&editor_type)
        {
            editor.render_ui(ui);
        }
    }

    /// 更新所有编辑器
    pub fn update(&mut self, dt: f32, input: &EditorInput) {
        for editor in self.open_editors.values_mut() {
            editor.handle_input(input);
            editor.update(dt);
        }
    }
}

impl Default for EditorManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 便捷类型和辅助函数
// ============================================================================

/// 着色器编辑器便捷类型
pub type ShaderEditor = ShaderGraph;

/// 动画编辑器便捷类型
pub type AnimationEditor = AnimationStateMachine;

/// 粒子编辑器便捷类型
pub type ParticleEditor = ParticleSystemData;

/// 创建默认着色器图
pub fn create_default_shader_graph() -> ShaderGraph {
    let mut graph = ShaderGraph::new("DefaultShader".to_string());

    // 添加输出节点
    let output_node = ShaderNode {
        id: 0,
        node_type: ShaderNodeType::Output,
        position: Vec2::new(400.0, 300.0),
        name: "Output".to_string(),
        inputs: vec![
            NodePort {
                name: "BaseColor".to_string(),
                data_type: ShaderDataType::Color,
                connection: None,
            },
            NodePort {
                name: "Normal".to_string(),
                data_type: ShaderDataType::Vec3,
                connection: None,
            },
        ],
        outputs: Vec::new(),
        data: ShaderNodeData::None,
    };

    graph.add_node(output_node);
    graph.output_node = Some(0);

    graph
}

/// 创建默认状态机
pub fn create_default_state_machine() -> AnimationStateMachine {
    let mut sm = AnimationStateMachine::new(
        "default_sm".to_string(),
        "Default StateMachine".to_string(),
        "Idle".to_string(),
    );

    // 添加Idle状态
    sm.add_state(AnimationState {
        id: "idle".to_string(),
        name: "Idle".to_string(),
        animation_clip: "idle.anim".to_string(),
        loop_: true,
        speed: 1.0,
        blend_time: 0.2,
        position: Vec2::new(100.0, 100.0),
    });

    // 添加Walk状态
    sm.add_state(AnimationState {
        id: "walk".to_string(),
        name: "Walk".to_string(),
        animation_clip: "walk.anim".to_string(),
        loop_: true,
        speed: 1.0,
        blend_time: 0.2,
        position: Vec2::new(300.0, 100.0),
    });

    sm
}

/// 创建默认粒子系统
pub fn create_default_particle_system() -> ParticleSystemData {
    ParticleSystemData::new("default_ps".to_string(), "Default System".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_graph() {
        let graph = create_default_shader_graph();
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.output_node.is_some());
    }

    #[test]
    fn test_state_machine() {
        let sm = create_default_state_machine();
        assert_eq!(sm.current_state, "Idle");
        assert_eq!(sm.states.len(), 2);
    }

    #[test]
    fn test_particle_system() {
        let ps = create_default_particle_system();
        assert_eq!(ps.id, "default_ps");
        assert!(ps.enabled);
    }

    #[test]
    fn test_state_machine_transition() {
        let mut sm = create_default_state_machine();
        sm.set_parameter("Speed".to_string(), 5.0);

        // 测试参数条件转换
        let transition = StateTransition {
            from_state: "Idle".to_string(),
            to_state: "Walk".to_string(),
            condition: TransitionCondition::Parameter {
                param: "Speed".to_string(),
                value: 3.0,
                comparison: ComparisonOp::Greater,
            },
            duration: 0.2,
            blend_mode: BlendMode::Linear,
        };

        sm.add_transition(transition);
        sm.update(0.0, &[]);

        // 应该触发转换
        assert_eq!(sm.current_state, "Walk");
    }

    #[test]
    fn test_json_export_import() {
        let sm = create_default_state_machine();
        let json = sm.export_json().expect("Test: operation should succeed");
        let sm2 =
            AnimationStateMachine::import_json(&json).expect("Test: operation should succeed");

        assert_eq!(sm.name, sm2.name);
        assert_eq!(sm.states.len(), sm2.states.len());
    }
}
