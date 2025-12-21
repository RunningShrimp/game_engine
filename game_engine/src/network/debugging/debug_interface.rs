//  网络调试界面模块
// 
//  提供用户友好的网络调试控制面板和界面功能。
// 
//  ## 功能特性
// 
//  - 网络调试控制面板
//  - 实时网络状态显示
//  - 网络配置调整界面
//  - 调试命令和测试工具
//  - 可视化数据展示
//  - 交互式控制

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use rand;

/// 网络调试界面
pub struct NetworkDebugInterface {
    /// 是否启用
    enabled: bool,
    /// 界面配置
    config: InterfaceConfig,
    /// 控制面板
    control_panel: ControlPanel,
    /// 状态显示器
    status_display: StatusDisplay,
    /// 配置编辑器
    config_editor: ConfigEditor,
    /// 命令处理器
    command_processor: CommandProcessor,
    /// 测试工具
    test_tools: TestTools,
    /// 数据可视化器
    data_visualizer: DataVisualizer,
    /// 界面状态
    interface_state: InterfaceState,
    /// 事件历史
    event_history: VecDeque<InterfaceEvent>,
    /// 最大事件历史长度
    max_event_history: usize,
}

/// 界面配置
#[derive(Debug, Clone)]
pub struct InterfaceConfig {
    /// 界面主题
    pub theme: InterfaceTheme,
    /// 更新频率（Hz）
    pub update_frequency_hz: f32,
    /// 是否启用自动刷新
    pub auto_refresh: bool,
    /// 数据保留时间（秒）
    pub data_retention_s: u64,
    /// 是否显示高级选项
    pub show_advanced_options: bool,
    /// 界面布局
    pub layout: InterfaceLayout,
    /// 快捷键配置
    pub key_bindings: HashMap<String, String>,
}

/// 界面主题
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceTheme {
    /// 默认主题
    Default,
    /// 深色主题
    Dark,
    /// 浅色主题
    Light,
    /// 高对比度主题
    HighContrast,
    /// 自定义主题
    Custom,
}

/// 界面布局
#[derive(Debug, Clone)]
pub struct InterfaceLayout {
    /// 控制面板位置
    pub control_panel_position: PanelPosition,
    /// 状态显示器位置
    pub status_display_position: PanelPosition,
    /// 数据可视化区域位置
    pub visualization_area_position: PanelPosition,
    /// 面板大小
    pub panel_sizes: HashMap<String, (f32, f32)>,
    /// 是否可调整大小
    pub resizable_panels: bool,
}

/// 面板位置
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelPosition {
    /// 左侧
    Left,
    /// 右侧
    Right,
    /// 顶部
    Top,
    /// 底部
    Bottom,
    /// 居中
    Center,
    /// 自定义位置
    Custom { x: f32, y: f32 },
}

/// 控制面板
#[derive(Debug)]
pub struct ControlPanel {
    /// 面板状态
    pub state: PanelState,
    /// 控制项
    pub controls: Vec<Control>,
    /// 当前活动标签
    pub active_tab: String,
    /// 标签页
    pub tabs: Vec<Tab>,
}

/// 面板状态
#[derive(Debug, Clone)]
pub struct PanelState {
    /// 是否可见
    pub visible: bool,
    /// 是否展开
    pub expanded: bool,
    /// 透明度
    pub opacity: f32,
    /// 缩放比例
    pub scale: f32,
}

/// 控制项
#[derive(Debug, Clone)]
pub struct Control {
    /// 控制ID
    pub id: String,
    /// 控制类型
    pub control_type: ControlType,
    /// 标签
    pub label: String,
    /// 描述
    pub description: String,
    /// 当前值
    pub value: ControlValue,
    /// 默认值
    pub default_value: ControlValue,
    /// 是否启用
    pub enabled: bool,
    /// 是否可见
    pub visible: bool,
    /// 验证器
    pub validator: Option<String>,
}

/// 控制类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlType {
    /// 按钮
    Button,
    /// 开关
    Toggle,
    /// 滑块
    Slider,
    /// 文本输入
    TextInput,
    /// 数字输入
    NumberInput,
    /// 下拉选择
    Dropdown,
    /// 复选框
    Checkbox,
    /// 颜色选择器
    ColorPicker,
    /// 文件选择器
    FilePicker,
    /// 进度条
    ProgressBar,
    /// 标签
    Label,
}

/// 控制值
#[derive(Debug, Clone)]
pub enum ControlValue {
    /// 布尔值
    Boolean(bool),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 颜色
    Color([u8; 3]),
    /// 选项列表
    Options(Vec<String>),
    /// 无值
    None,
}

/// 标签页
#[derive(Debug, Clone)]
pub struct Tab {
    /// 标签ID
    pub id: String,
    /// 标签名称
    pub name: String,
    /// 标签图标
    pub icon: Option<String>,
    /// 控制组
    pub control_groups: Vec<ControlGroup>,
    /// 是否启用
    pub enabled: bool,
}

/// 控制组
#[derive(Debug, Clone)]
pub struct ControlGroup {
    /// 组ID
    pub id: String,
    /// 组名称
    pub name: String,
    /// 是否可折叠
    pub collapsible: bool,
    /// 是否展开
    pub expanded: bool,
    /// 控制ID列表
    pub control_ids: Vec<String>,
}

/// 状态显示器
#[derive(Debug)]
pub struct StatusDisplay {
    /// 显示器状态
    pub state: PanelState,
    /// 状态项
    pub status_items: Vec<StatusItem>,
    /// 显示模式
    pub display_mode: DisplayMode,
    /// 更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 最后更新时间
    pub last_update: Instant,
}

/// 状态项
#[derive(Debug, Clone)]
pub struct StatusItem {
    /// 项目ID
    pub id: String,
    /// 项目名称
    pub name: String,
    /// 项目值
    pub value: StatusValue,
    /// 项目类型
    pub item_type: StatusItemType,
    /// 严重程度
    pub severity: StatusSeverity,
    /// 单位
    pub unit: Option<String>,
    /// 最小值
    pub min_value: Option<f64>,
    /// 最大值
    pub max_value: Option<f64>,
    /// 警告阈值
    pub warning_threshold: Option<f64>,
    /// 错误阈值
    pub error_threshold: Option<f64>,
}

/// 状态值
#[derive(Debug, Clone)]
pub enum StatusValue {
    /// 文本值
    Text(String),
    /// 数值
    Number(f64),
    /// 百分比
    Percentage(f64),
    /// 状态
    Status(String),
    /// 进度
    Progress { current: f64, total: f64 },
    /// 图表数据
    Chart(Vec<(f64, f64)>),
    /// 无值
    None,
}

/// 状态项类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusItemType {
    /// 连接状态
    ConnectionStatus,
    /// 性能指标
    PerformanceMetric,
    /// 错误计数
    ErrorCount,
    /// 警告计数
    WarningCount,
    /// 资源使用率
    ResourceUsage,
    /// 自定义类型
    Custom,
}

/// 状态严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重错误
    Critical,
}

/// 显示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// 紧凑模式
    Compact,
    /// 详细模式
    Detailed,
    /// 图表模式
    Chart,
    /// 仪表盘模式
    Dashboard,
}

/// 配置编辑器
#[derive(Debug)]
pub struct ConfigEditor {
    /// 编辑器状态
    pub state: PanelState,
    /// 配置项
    pub config_items: Vec<ConfigItem>,
    /// 当前编辑的配置
    pub current_config: String,
    /// 配置历史
    pub config_history: VecDeque<ConfigSnapshot>,
    /// 最大历史长度
    pub max_history_length: usize,
}

/// 配置项
#[derive(Debug, Clone)]
pub struct ConfigItem {
    /// 配置键
    pub key: String,
    /// 配置值
    pub value: ConfigValue,
    /// 配置类型
    pub config_type: ConfigType,
    /// 配置类别
    pub category: String,
    /// 描述
    pub description: String,
    /// 是否需要重启
    pub requires_restart: bool,
    /// 是否只读
    pub read_only: bool,
}

/// 配置值
#[derive(Debug, Clone)]
pub enum ConfigValue {
    /// 布尔值
    Boolean(bool),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 数组
    Array(Vec<ConfigValue>),
    /// 对象
    Object(HashMap<String, ConfigValue>),
}

/// 配置类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    /// 布尔类型
    Boolean,
    /// 整数类型
    Integer,
    /// 浮点数类型
    Float,
    /// 字符串类型
    String,
    /// 数组类型
    Array,
    /// 对象类型
    Object,
}

/// 配置快照
#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    /// 快照时间戳
    pub timestamp: Instant,
    /// 快照描述
    pub description: String,
    /// 配置数据
    pub config_data: HashMap<String, ConfigValue>,
}

/// 命令处理器
#[derive(Debug)]
pub struct CommandProcessor {
    /// 处理器状态
    pub state: PanelState,
    /// 命令历史
    pub command_history: VecDeque<CommandEntry>,
    /// 最大历史长度
    pub max_history_length: usize,
    /// 当前命令
    pub current_command: String,
    /// 命令建议
    pub command_suggestions: Vec<String>,
    /// 命令别名
    pub command_aliases: HashMap<String, String>,
}

/// 命令条目
#[derive(Debug, Clone)]
pub struct CommandEntry {
    /// 命令
    pub command: String,
    /// 时间戳
    pub timestamp: Instant,
    /// 执行结果
    pub result: CommandResult,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// 命令结果
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// 成功
    Success(String),
    /// 错误
    Error(String),
    /// 警告
    Warning(String),
    /// 无输出
    NoOutput,
}

/// 测试工具
#[derive(Debug)]
pub struct TestTools {
    /// 工具状态
    pub state: PanelState,
    /// 可用测试
    pub available_tests: Vec<NetworkTest>,
    /// 当前运行的测试
    pub running_tests: Vec<RunningTest>,
    /// 测试结果历史
    pub test_results: VecDeque<TestResult>,
    /// 最大结果历史长度
    pub max_results_history: usize,
}

/// 网络测试
#[derive(Debug, Clone)]
pub struct NetworkTest {
    /// 测试ID
    pub id: String,
    /// 测试名称
    pub name: String,
    /// 测试描述
    pub description: String,
    /// 测试类型
    pub test_type: TestType,
    /// 测试参数
    pub parameters: Vec<TestParameter>,
    /// 预计执行时间（秒）
    pub estimated_duration_s: u64,
}

/// 测试类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestType {
    /// 连接测试
    Connectivity,
    /// 带宽测试
    Bandwidth,
    /// 延迟测试
    Latency,
    /// 丢包测试
    PacketLoss,
    /// 压力测试
    Stress,
    /// 自定义测试
    Custom,
}

/// 测试参数
#[derive(Debug, Clone)]
pub struct TestParameter {
    /// 参数名
    pub name: String,
    /// 参数值
    pub value: TestParameterValue,
    /// 参数类型
    pub param_type: TestParameterType,
    /// 描述
    pub description: String,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<TestParameterValue>,
}

/// 测试参数值
#[derive(Debug, Clone)]
pub enum TestParameterValue {
    /// 布尔值
    Boolean(bool),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 选项
    Options(Vec<String>),
}

/// 测试参数类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestParameterType {
    /// 布尔类型
    Boolean,
    /// 整数类型
    Integer,
    /// 浮点数类型
    Float,
    /// 字符串类型
    String,
    /// 选项类型
    Options,
}

/// 运行中的测试
#[derive(Debug, Clone)]
pub struct RunningTest {
    /// 测试ID
    pub test_id: String,
    /// 开始时间
    pub start_time: Instant,
    /// 进度（0-1）
    pub progress: f32,
    /// 当前阶段
    pub current_phase: String,
    /// 预计剩余时间（秒）
    pub estimated_remaining_s: u64,
}

/// 测试结果
#[derive(Debug, Clone)]
pub struct TestResult {
    /// 测试ID
    pub test_id: String,
    /// 测试名称
    pub test_name: String,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Instant,
    /// 执行状态
    pub status: TestStatus,
    /// 测试指标
    pub metrics: HashMap<String, TestMetric>,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 测试状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    /// 运行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 取消
    Cancelled,
    /// 超时
    Timeout,
}

/// 测试指标
#[derive(Debug, Clone)]
pub struct TestMetric {
    /// 指标名称
    pub name: String,
    /// 指标值
    pub value: TestMetricValue,
    /// 指标单位
    pub unit: Option<String>,
    /// 描述
    pub description: Option<String>,
}

/// 测试指标值
#[derive(Debug, Clone)]
pub enum TestMetricValue {
    /// 数值
    Number(f64),
    /// 百分比
    Percentage(f64),
    /// 速率
    Rate(f64),
    /// 计数
    Count(u64),
    /// 时间
    Time(Duration),
    /// 状态
    Status(String),
    /// 数组
    Array(Vec<TestMetricValue>),
}

/// 数据可视化器
#[derive(Debug)]
pub struct DataVisualizer {
    /// 可视化器状态
    pub state: PanelState,
    /// 可视化组件
    pub visualizations: Vec<Visualization>,
    /// 当前活动可视化
    pub active_visualization: Option<String>,
    /// 数据源
    pub data_sources: HashMap<String, DataSource>,
    /// 更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 最后更新时间
    pub last_update: Instant,
}

/// 可视化
#[derive(Debug, Clone)]
pub struct Visualization {
    /// 可视化ID
    pub id: String,
    /// 可视化名称
    pub name: String,
    /// 可视化类型
    pub viz_type: VisualizationType,
    /// 数据源ID
    pub data_source_id: String,
    /// 可视化配置
    pub config: VisualizationConfig,
    /// 是否启用
    pub enabled: bool,
}

/// 可视化类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizationType {
    /// 折线图
    LineChart,
    /// 柱状图
    BarChart,
    /// 饼图
    PieChart,
    /// 散点图
    ScatterPlot,
    /// 热图
    Heatmap,
    /// 仪表盘
    Gauge,
    /// 表格
    Table,
    /// 自定义可视化
    Custom,
}

/// 可视化配置
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    /// 标题
    pub title: Option<String>,
    /// X轴标签
    pub x_axis_label: Option<String>,
    /// Y轴标签
    pub y_axis_label: Option<String>,
    /// 颜色方案
    pub color_scheme: Option<String>,
    /// 自动缩放
    pub auto_scale: bool,
    /// 最小值
    pub min_value: Option<f64>,
    /// 最大值
    pub max_value: Option<f64>,
    /// 刷新间隔（毫秒）
    pub refresh_interval_ms: u64,
    /// 自定义配置
    pub custom_config: HashMap<String, String>,
}

/// 数据源
#[derive(Debug, Clone)]
pub struct DataSource {
    /// 数据源ID
    pub id: String,
    /// 数据源名称
    pub name: String,
    /// 数据源类型
    pub source_type: DataSourceType,
    /// 数据更新回调
    pub update_callback: Option<String>,
    /// 最后更新时间
    pub last_update: Instant,
    /// 数据缓存
    pub data_cache: Vec<DataPoint>,
    /// 最大缓存大小
    pub max_cache_size: usize,
}

/// 数据源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSourceType {
    /// 性能监控数据
    PerformanceMonitor,
    /// 数据包分析数据
    PacketAnalyzer,
    /// 延迟可视化数据
    LatencyVisualizer,
    /// 自定义数据源
    Custom,
}

/// 数据点
#[derive(Debug, Clone)]
pub struct DataPoint {
    /// 时间戳
    pub timestamp: Instant,
    /// 数据值
    pub value: DataValue,
    /// 标签
    pub tags: HashMap<String, String>,
}

/// 数据值
#[derive(Debug, Clone)]
pub enum DataValue {
    /// 单一数值
    Single(f64),
    /// 多个数值
    Multiple(Vec<f64>),
    /// 结构化数据
    Structured(HashMap<String, DataValue>),
    /// 文本数据
    Text(String),
    /// 二进制数据
    Binary(Vec<u8>),
}

/// 界面状态
#[derive(Debug, Clone)]
pub struct InterfaceState {
    /// 当前活动面板
    pub active_panel: Option<String>,
    /// 鼠标位置
    pub mouse_position: (f32, f32),
    /// 窗口大小
    pub window_size: (f32, f32),
    /// 是否全屏
    pub fullscreen: bool,
    /// 缩放级别
    pub zoom_level: f32,
    /// 界面语言
    pub language: String,
    /// 最后活动时间
    pub last_activity: Instant,
}

/// 界面事件
#[derive(Debug, Clone)]
pub struct InterfaceEvent {
    /// 事件ID
    pub event_id: u64,
    /// 时间戳
    pub timestamp: Instant,
    /// 事件类型
    pub event_type: InterfaceEventType,
    /// 事件源
    pub source: String,
    /// 事件数据
    pub event_data: HashMap<String, String>,
    /// 严重程度
    pub severity: EventSeverity,
}

/// 界面事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceEventType {
    /// 用户交互
    UserInteraction,
    /// 系统事件
    SystemEvent,
    /// 错误事件
    ErrorEvent,
    /// 警告事件
    WarningEvent,
    /// 信息事件
    InfoEvent,
    /// 配置更改
    ConfigurationChange,
    /// 测试事件
    TestEvent,
}

/// 事件严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSeverity {
    /// 调试
    Debug,
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重错误
    Critical,
}

impl NetworkDebugInterface {
    /// 创建新的网络调试界面
    pub fn new() -> Self {
        Self::with_config(InterfaceConfig::default())
    }

    /// 创建带配置的网络调试界面
    pub fn with_config(config: InterfaceConfig) -> Self {
        Self {
            enabled: true,
            config,
            control_panel: ControlPanel::new(),
            status_display: StatusDisplay::new(),
            config_editor: ConfigEditor::new(),
            command_processor: CommandProcessor::new(),
            test_tools: TestTools::new(),
            data_visualizer: DataVisualizer::new(),
            interface_state: InterfaceState::default(),
            event_history: VecDeque::with_capacity(1000),
            max_event_history: 1000,
        }
    }

    /// 初始化界面
    pub fn initialize(&mut self) {
        // 初始化控制面板
        self.initialize_control_panel();
        
        // 初始化状态显示器
        self.initialize_status_display();
        
        // 初始化配置编辑器
        self.initialize_config_editor();
        
        // 初始化测试工具
        self.initialize_test_tools();
        
        // 初始化数据可视化器
        self.initialize_data_visualizer();
        
        // 添加初始化事件
        self.add_interface_event(InterfaceEvent {
            event_id: rand::random(),
            timestamp: Instant::now(),
            event_type: InterfaceEventType::SystemEvent,
            source: "NetworkDebugInterface".to_string(),
            event_data: HashMap::from([("action".to_string(), "initialize".to_string())]),
            severity: EventSeverity::Info,
        });
    }

    /// 更新界面
    pub fn update(&mut self, _delta_time: Duration) {
        if !self.enabled {
            return;
        }

        // 更新状态显示器
        self.status_display.update();
        
        // 更新数据可视化器
        self.data_visualizer.update();
        
        // 处理运行中的测试
        self.test_tools.update_running_tests();
        
        // 清理过期事件
        self.cleanup_expired_events();
        
        // 更新界面状态
        self.interface_state.last_activity = Instant::now();
    }

    /// 处理用户输入
    pub fn handle_input(&mut self, input: UserInput) -> InterfaceResult {
        if !self.enabled {
            return InterfaceResult::Ignored;
        }

        match input.input_type {
            InputType::Click => self.handle_click(input),
            InputType::KeyPress => self.handle_key_press(input),
            InputType::TextInput => self.handle_text_input(input),
            InputType::MouseMove => self.handle_mouse_move(input),
            InputType::Scroll => self.handle_scroll(input),
        }
    }

    /// 执行命令
    pub fn execute_command(&mut self, command: &str) -> CommandResult {
        let result = self.command_processor.execute_command(command);
        
        // 添加命令执行事件
        self.add_interface_event(InterfaceEvent {
            event_id: rand::random(),
            timestamp: Instant::now(),
            event_type: InterfaceEventType::UserInteraction,
            source: "CommandProcessor".to_string(),
            event_data: HashMap::from([
                ("command".to_string(), command.to_string()),
                ("result".to_string(), format!("{:?}", result)),
            ]),
            severity: match &result {
                CommandResult::Error(_) => EventSeverity::Error,
                CommandResult::Warning(_) => EventSeverity::Warning,
                _ => EventSeverity::Info,
            },
        });
        
        result
    }

    /// 运行测试
    pub fn run_test(&mut self, test_id: &str, parameters: HashMap<String, TestParameterValue>) -> Result<(), String> {
        let result = self.test_tools.run_test(test_id, parameters);
        
        // 添加测试事件
        self.add_interface_event(InterfaceEvent {
            event_id: rand::random(),
            timestamp: Instant::now(),
            event_type: InterfaceEventType::TestEvent,
            source: "TestTools".to_string(),
            event_data: HashMap::from([
                ("test_id".to_string(), test_id.to_string()),
                ("result".to_string(), format!("{:?}", result)),
            ]),
            severity: match &result {
                Ok(_) => EventSeverity::Info,
                Err(_) => EventSeverity::Error,
            },
        });
        
        result
    }

    /// 获取界面状态
    pub fn get_interface_state(&self) -> &InterfaceState {
        &self.interface_state
    }

    /// 获取控制面板
    pub fn get_control_panel(&self) -> &ControlPanel {
        &self.control_panel
    }

    /// 获取状态显示器
    pub fn get_status_display(&self) -> &StatusDisplay {
        &self.status_display
    }

    /// 获取配置编辑器
    pub fn get_config_editor(&self) -> &ConfigEditor {
        &self.config_editor
    }

    /// 获取测试工具
    pub fn get_test_tools(&self) -> &TestTools {
        &self.test_tools
    }

    /// 获取数据可视化器
    pub fn get_data_visualizer(&self) -> &DataVisualizer {
        &self.data_visualizer
    }

    /// 获取事件历史
    pub fn get_event_history(&self) -> Vec<InterfaceEvent> {
        self.event_history.iter().cloned().collect()
    }

    /// 设置界面主题
    pub fn set_theme(&mut self, theme: InterfaceTheme) {
        self.config.theme = theme;
        
        // 添加主题更改事件
        self.add_interface_event(InterfaceEvent {
            event_id: rand::random(),
            timestamp: Instant::now(),
            event_type: InterfaceEventType::ConfigurationChange,
            source: "NetworkDebugInterface".to_string(),
            event_data: HashMap::from([("theme".to_string(), format!("{:?}", theme))]),
            severity: EventSeverity::Info,
        });
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 设置启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        self.enabled && 
        (Instant::now().duration_since(self.interface_state.last_activity).as_secs() < 300) // 5分钟内活跃
    }

    // 私有方法

    /// 初始化控制面板
    fn initialize_control_panel(&mut self) {
        self.control_panel = ControlPanel::new();
        
        // 添加基本控制项
        self.control_panel.add_control(Control {
            id: "enable_debugging".to_string(),
            control_type: ControlType::Toggle,
            label: "启用调试".to_string(),
            description: "启用或禁用网络调试功能".to_string(),
            value: ControlValue::Boolean(true),
            default_value: ControlValue::Boolean(true),
            enabled: true,
            visible: true,
            validator: None,
        });
        
        self.control_panel.add_control(Control {
            id: "auto_refresh".to_string(),
            control_type: ControlType::Toggle,
            label: "自动刷新".to_string(),
            description: "自动刷新界面数据".to_string(),
            value: ControlValue::Boolean(self.config.auto_refresh),
            default_value: ControlValue::Boolean(true),
            enabled: true,
            visible: true,
            validator: None,
        });
        
        self.control_panel.add_control(Control {
            id: "update_frequency".to_string(),
            control_type: ControlType::Slider,
            label: "更新频率".to_string(),
            description: "界面更新频率（Hz）".to_string(),
            value: ControlValue::Float(self.config.update_frequency_hz as f64),
            default_value: ControlValue::Float(1.0),
            enabled: true,
            visible: true,
            validator: None,
        });
    }

    /// 初始化状态显示器
    fn initialize_status_display(&mut self) {
        self.status_display = StatusDisplay::new();
        
        // 添加基本状态项
        self.status_display.add_status_item(StatusItem {
            id: "connection_status".to_string(),
            name: "连接状态".to_string(),
            value: StatusValue::Status("未连接".to_string()),
            item_type: StatusItemType::ConnectionStatus,
            severity: StatusSeverity::Info,
            unit: None,
            min_value: None,
            max_value: None,
            warning_threshold: None,
            error_threshold: None,
        });
        
        self.status_display.add_status_item(StatusItem {
            id: "latency".to_string(),
            name: "延迟".to_string(),
            value: StatusValue::Number(0.0),
            item_type: StatusItemType::PerformanceMetric,
            severity: StatusSeverity::Info,
            unit: Some("ms".to_string()),
            min_value: Some(0.0),
            max_value: Some(1000.0),
            warning_threshold: Some(100.0),
            error_threshold: Some(500.0),
        });
        
        self.status_display.add_status_item(StatusItem {
            id: "packet_loss".to_string(),
            name: "丢包率".to_string(),
            value: StatusValue::Percentage(0.0),
            item_type: StatusItemType::PerformanceMetric,
            severity: StatusSeverity::Info,
            unit: Some("%".to_string()),
            min_value: Some(0.0),
            max_value: Some(100.0),
            warning_threshold: Some(5.0),
            error_threshold: Some(10.0),
        });
    }

    /// 初始化配置编辑器
    fn initialize_config_editor(&mut self) {
        self.config_editor = ConfigEditor::new();
        
        // 添加基本配置项
        self.config_editor.add_config_item(ConfigItem {
            key: "debug.enabled".to_string(),
            value: ConfigValue::Boolean(true),
            config_type: ConfigType::Boolean,
            category: "debug".to_string(),
            description: "启用网络调试".to_string(),
            requires_restart: false,
            read_only: false,
        });
        
        self.config_editor.add_config_item(ConfigItem {
            key: "debug.verbose".to_string(),
            value: ConfigValue::Boolean(false),
            config_type: ConfigType::Boolean,
            category: "debug".to_string(),
            description: "启用详细日志".to_string(),
            requires_restart: false,
            read_only: false,
        });
        
        self.config_editor.add_config_item(ConfigItem {
            key: "network.max_connections".to_string(),
            value: ConfigValue::Integer(100),
            config_type: ConfigType::Integer,
            category: "network".to_string(),
            description: "最大连接数".to_string(),
            requires_restart: true,
            read_only: false,
        });
    }

    /// 初始化测试工具
    fn initialize_test_tools(&mut self) {
        self.test_tools = TestTools::new();
        
        // 添加基本测试
        self.test_tools.add_test(NetworkTest {
            id: "connectivity_test".to_string(),
            name: "连接测试".to_string(),
            description: "测试网络连接状态".to_string(),
            test_type: TestType::Connectivity,
            parameters: vec![
                TestParameter {
                    name: "target_address".to_string(),
                    value: TestParameterValue::String("127.0.0.1:8080".to_string()),
                    param_type: TestParameterType::String,
                    description: "目标地址".to_string(),
                    required: true,
                    default_value: Some(TestParameterValue::String("127.0.0.1:8080".to_string())),
                },
                TestParameter {
                    name: "timeout".to_string(),
                    value: TestParameterValue::Integer(5000),
                    param_type: TestParameterType::Integer,
                    description: "超时时间（毫秒）".to_string(),
                    required: false,
                    default_value: Some(TestParameterValue::Integer(5000)),
                },
            ],
            estimated_duration_s: 5,
        });
        
        self.test_tools.add_test(NetworkTest {
            id: "latency_test".to_string(),
            name: "延迟测试".to_string(),
            description: "测试网络延迟".to_string(),
            test_type: TestType::Latency,
            parameters: vec![
                TestParameter {
                    name: "target_address".to_string(),
                    value: TestParameterValue::String("127.0.0.1:8080".to_string()),
                    param_type: TestParameterType::String,
                    description: "目标地址".to_string(),
                    required: true,
                    default_value: Some(TestParameterValue::String("127.0.0.1:8080".to_string())),
                },
                TestParameter {
                    name: "packet_count".to_string(),
                    value: TestParameterValue::Integer(100),
                    param_type: TestParameterType::Integer,
                    description: "测试包数量".to_string(),
                    required: false,
                    default_value: Some(TestParameterValue::Integer(100)),
                },
            ],
            estimated_duration_s: 10,
        });
    }

    /// 初始化数据可视化器
    fn initialize_data_visualizer(&mut self) {
        self.data_visualizer = DataVisualizer::new();
        
        // 添加基本可视化
        self.data_visualizer.add_visualization(Visualization {
            id: "latency_chart".to_string(),
            name: "延迟图表".to_string(),
            viz_type: VisualizationType::LineChart,
            data_source_id: "latency_data".to_string(),
            config: VisualizationConfig {
                title: Some("网络延迟".to_string()),
                x_axis_label: Some("时间".to_string()),
                y_axis_label: Some("延迟 (ms)".to_string()),
                color_scheme: Some("blue".to_string()),
                auto_scale: true,
                min_value: None,
                max_value: None,
                refresh_interval_ms: 1000,
                custom_config: HashMap::new(),
            },
            enabled: true,
        });
        
        self.data_visualizer.add_visualization(Visualization {
            id: "packet_loss_chart".to_string(),
            name: "丢包率图表".to_string(),
            viz_type: VisualizationType::LineChart,
            data_source_id: "packet_loss_data".to_string(),
            config: VisualizationConfig {
                title: Some("丢包率".to_string()),
                x_axis_label: Some("时间".to_string()),
                y_axis_label: Some("丢包率 (%)".to_string()),
                color_scheme: Some("red".to_string()),
                auto_scale: true,
                min_value: Some(0.0),
                max_value: Some(100.0),
                refresh_interval_ms: 1000,
                custom_config: HashMap::new(),
            },
            enabled: true,
        });
        
        // 添加数据源
        self.data_visualizer.add_data_source(DataSource {
            id: "latency_data".to_string(),
            name: "延迟数据".to_string(),
            source_type: DataSourceType::LatencyVisualizer,
            update_callback: None,
            last_update: Instant::now(),
            data_cache: Vec::new(),
            max_cache_size: 1000,
        });
        
        self.data_visualizer.add_data_source(DataSource {
            id: "packet_loss_data".to_string(),
            name: "丢包率数据".to_string(),
            source_type: DataSourceType::PerformanceMonitor,
            update_callback: None,
            last_update: Instant::now(),
            data_cache: Vec::new(),
            max_cache_size: 1000,
        });
    }

    /// 处理点击事件
    fn handle_click(&mut self, input: UserInput) -> InterfaceResult {
        // 根据点击位置确定目标
        if let Some(target) = self.determine_click_target(input.position) {
            match target.target_type {
                TargetType::Control => self.handle_control_click(target),
                TargetType::StatusItem => self.handle_status_item_click(target),
                TargetType::Visualization => self.handle_visualization_click(target),
                TargetType::Tab => self.handle_tab_click(target),
            }
        } else {
            InterfaceResult::Ignored
        }
    }

    /// 处理按键事件
    fn handle_key_press(&mut self, input: UserInput) -> InterfaceResult {
        // 处理快捷键
        if let Some(shortcut) = self.config.key_bindings.get(&input.key).cloned() {
            self.execute_shortcut(&shortcut)
        } else {
            InterfaceResult::Ignored
        }
    }

    /// 处理文本输入
    fn handle_text_input(&mut self, input: UserInput) -> InterfaceResult {
        // 处理文本输入
        if self.interface_state.active_panel == Some("command_processor".to_string()) {
            self.command_processor.handle_text_input(&input.text);
            InterfaceResult::Processed
        } else if self.interface_state.active_panel == Some("config_editor".to_string()) {
            self.config_editor.handle_text_input(&input.text);
            InterfaceResult::Processed
        } else {
            InterfaceResult::Ignored
        }
    }

    /// 处理鼠标移动
    fn handle_mouse_move(&mut self, input: UserInput) -> InterfaceResult {
        self.interface_state.mouse_position = input.position;
        InterfaceResult::Processed
    }

    /// 处理滚动事件
    fn handle_scroll(&mut self, _input: UserInput) -> InterfaceResult {
        // 处理滚动
        InterfaceResult::Processed
    }

    /// 确定点击目标
    fn determine_click_target(&self, position: (f32, f32)) -> Option<ClickTarget> {
        // 简化实现：根据位置确定目标
        // 实际实现中需要检查所有UI元素的位置和大小
        
        if position.0 < 200.0 && position.1 < 300.0 {
            // 控制面板区域
            Some(ClickTarget {
                target_type: TargetType::Control,
                target_id: "control_panel".to_string(),
                position,
            })
        } else if position.0 < 200.0 && position.1 >= 300.0 && position.1 < 600.0 {
            // 状态显示区域
            Some(ClickTarget {
                target_type: TargetType::StatusItem,
                target_id: "status_display".to_string(),
                position,
            })
        } else if position.0 >= 200.0 {
            // 数据可视化区域
            Some(ClickTarget {
                target_type: TargetType::Visualization,
                target_id: "data_visualizer".to_string(),
                position,
            })
        } else {
            None
        }
    }

    /// 处理控制点击
    fn handle_control_click(&mut self, _target: ClickTarget) -> InterfaceResult {
        // 处理控制项点击
        InterfaceResult::Processed
    }

    /// 处理状态项点击
    fn handle_status_item_click(&mut self, _target: ClickTarget) -> InterfaceResult {
        // 处理状态项点击
        InterfaceResult::Processed
    }

    /// 处理可视化点击
    fn handle_visualization_click(&mut self, _target: ClickTarget) -> InterfaceResult {
        // 处理可视化点击
        InterfaceResult::Processed
    }

    /// 处理标签点击
    fn handle_tab_click(&mut self, _target: ClickTarget) -> InterfaceResult {
        // 处理标签点击
        InterfaceResult::Processed
    }

    /// 执行快捷键
    fn execute_shortcut(&mut self, shortcut: &str) -> InterfaceResult {
        match shortcut {
            "toggle_debug" => {
                self.enabled = !self.enabled;
                InterfaceResult::Processed
            }
            "refresh" => {
                // 刷新所有数据
                InterfaceResult::Processed
            }
            "clear_history" => {
                self.command_processor.clear_history();
                InterfaceResult::Processed
            }
            _ => InterfaceResult::Ignored,
        }
    }

    /// 添加界面事件
    fn add_interface_event(&mut self, event: InterfaceEvent) {
        self.event_history.push_back(event);
        
        // 限制事件历史长度
        while self.event_history.len() > self.max_event_history {
            self.event_history.pop_front();
        }
    }

    /// 清理过期事件
    fn cleanup_expired_events(&mut self) {
        let now = Instant::now();
        let retention_duration = Duration::from_secs(self.config.data_retention_s);
        
        self.event_history.retain(|event| {
            now.duration_since(event.timestamp) < retention_duration
        });
    }
}

/// 用户输入
#[derive(Debug, Clone)]
pub struct UserInput {
    /// 输入类型
    pub input_type: InputType,
    /// 位置（用于鼠标事件）
    pub position: (f32, f32),
    /// 按键（用于键盘事件）
    pub key: String,
    /// 文本（用于文本输入）
    pub text: String,
    /// 滚动量（用于滚动事件）
    pub scroll_delta: (f32, f32),
    /// 修饰键
    pub modifiers: Vec<String>,
}

/// 输入类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    /// 点击
    Click,
    /// 按键
    KeyPress,
    /// 文本输入
    TextInput,
    /// 鼠标移动
    MouseMove,
    /// 滚动
    Scroll,
}

/// 界面结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceResult {
    /// 已处理
    Processed,
    /// 已忽略
    Ignored,
    /// 错误
    Error,
}

/// 点击目标
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ClickTarget {
    /// 目标类型
    target_type: TargetType,
    /// 目标ID
    #[allow(dead_code)]
    target_id: String,
    /// 点击位置
    #[allow(dead_code)]
    position: (f32, f32),
}

/// 目标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum TargetType {
    /// 控制
    Control,
    /// 状态项
    StatusItem,
    /// 可视化
    Visualization,
    /// 标签
    #[allow(dead_code)]
    Tab,
}

// 控制面板实现
impl ControlPanel {
    fn new() -> Self {
        Self {
            state: PanelState {
                visible: true,
                expanded: true,
                opacity: 1.0,
                scale: 1.0,
            },
            controls: Vec::new(),
            active_tab: "general".to_string(),
            tabs: vec![
                Tab {
                    id: "general".to_string(),
                    name: "常规".to_string(),
                    icon: Some("settings".to_string()),
                    control_groups: Vec::new(),
                    enabled: true,
                },
                Tab {
                    id: "advanced".to_string(),
                    name: "高级".to_string(),
                    icon: Some("advanced".to_string()),
                    control_groups: Vec::new(),
                    enabled: true,
                },
            ],
        }
    }

    fn add_control(&mut self, control: Control) {
        self.controls.push(control);
    }
}

// 状态显示器实现
impl StatusDisplay {
    fn new() -> Self {
        Self {
            state: PanelState {
                visible: true,
                expanded: true,
                opacity: 1.0,
                scale: 1.0,
            },
            status_items: Vec::new(),
            display_mode: DisplayMode::Compact,
            update_interval_ms: 1000,
            last_update: Instant::now(),
        }
    }

    fn add_status_item(&mut self, item: StatusItem) {
        self.status_items.push(item);
    }

    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update).as_millis() >= self.update_interval_ms as u128 {
            // 更新状态项
            self.update_status_items();
            self.last_update = now;
        }
    }

    fn update_status_items(&mut self) {
        // 更新状态项的值
        // 这里应该从实际的数据源获取最新值
    }
}

// 配置编辑器实现
impl ConfigEditor {
    fn new() -> Self {
        Self {
            state: PanelState {
                visible: true,
                expanded: true,
                opacity: 1.0,
                scale: 1.0,
            },
            config_items: Vec::new(),
            current_config: "default".to_string(),
            config_history: VecDeque::with_capacity(50),
            max_history_length: 50,
        }
    }

    fn add_config_item(&mut self, item: ConfigItem) {
        self.config_items.push(item);
    }

    fn handle_text_input(&mut self, _text: &str) {
        // 处理文本输入
        // 这里应该根据当前编辑的配置项处理输入
    }
}

// 命令处理器实现
impl CommandProcessor {
    fn new() -> Self {
        Self {
            state: PanelState {
                visible: true,
                expanded: true,
                opacity: 1.0,
                scale: 1.0,
            },
            command_history: VecDeque::with_capacity(100),
            max_history_length: 100,
            current_command: String::new(),
            command_suggestions: Vec::new(),
            command_aliases: HashMap::from([
                ("h".to_string(), "help".to_string()),
                ("q".to_string(), "quit".to_string()),
                ("cls".to_string(), "clear".to_string()),
            ]),
        }
    }

    fn execute_command(&mut self, command: &str) -> CommandResult {
        let start_time = Instant::now();
        let result = self.process_command(command);
        let execution_time = start_time.elapsed().as_millis() as u64;

        // 添加到历史
        self.command_history.push_back(CommandEntry {
            command: command.to_string(),
            timestamp: Instant::now(),
            result: result.clone(),
            execution_time_ms: execution_time,
        });

        // 限制历史长度
        while self.command_history.len() > self.max_history_length {
            self.command_history.pop_front();
        }

        result
    }

    fn handle_text_input(&mut self, text: &str) {
        if text == "\n" || text == "\r" {
            // 执行当前命令
            if !self.current_command.is_empty() {
                let command = self.current_command.clone();
                self.execute_command(&command);
                self.current_command.clear();
            }
        } else if text == "\x08" || text == "\x7f" {
            // 退格键
            self.current_command.pop();
        } else {
            // 添加字符
            self.current_command.push_str(text);
        }
    }

    fn clear_history(&mut self) {
        self.command_history.clear();
    }

    fn process_command(&self, command: &str) -> CommandResult {
        let trimmed_command = command.trim();
        
        // 处理命令别名
        let command = self.command_aliases.get(trimmed_command).map_or(trimmed_command, |v| v);
        
        match command {
            "help" | "h" => CommandResult::Success(
                "可用命令:\n\
                 help - 显示帮助\n\
                 status - 显示状态\n\
                 clear - 清除历史\n\
                 quit - 退出".to_string()
            ),
            "status" => CommandResult::Success("调试界面状态: 正常".to_string()),
            "clear" | "cls" => CommandResult::NoOutput,
            "quit" | "q" => CommandResult::Success("退出调试界面".to_string()),
            _ => CommandResult::Error(format!("未知命令: {}", command)),
        }
    }
}

// 测试工具实现
impl TestTools {
    fn new() -> Self {
        Self {
            state: PanelState {
                visible: true,
                expanded: true,
                opacity: 1.0,
                scale: 1.0,
            },
            available_tests: Vec::new(),
            running_tests: Vec::new(),
            test_results: VecDeque::with_capacity(100),
            max_results_history: 100,
        }
    }

    fn add_test(&mut self, test: NetworkTest) {
        self.available_tests.push(test);
    }

    fn run_test(&mut self, test_id: &str, parameters: HashMap<String, TestParameterValue>) -> Result<(), String> {
        // 查找测试
        let test = self.available_tests.iter()
            .find(|t| t.id == test_id)
            .ok_or_else(|| format!("测试不存在: {}", test_id))?
            .clone();

        // 创建运行中的测试
        let running_test = RunningTest {
            test_id: test_id.to_string(),
            start_time: Instant::now(),
            progress: 0.0,
            current_phase: "初始化".to_string(),
            estimated_remaining_s: test.estimated_duration_s,
        };

        self.running_tests.push(running_test);

        // 模拟测试执行
        // 实际实现中应该在新线程中执行测试
        self.simulate_test_execution(test, parameters);

        Ok(())
    }

    fn update_running_tests(&mut self) {
        // 更新运行中的测试进度
        for test in &mut self.running_tests {
            let elapsed = test.start_time.elapsed().as_secs_f64();
            let total_duration = test.estimated_remaining_s as f64;
            
            if elapsed >= total_duration {
                test.progress = 1.0;
                test.current_phase = "完成".to_string();
                test.estimated_remaining_s = 0;
            } else {
                test.progress = (elapsed / total_duration) as f32;
                test.estimated_remaining_s = total_duration as u64 - elapsed as u64;
                
                // 更新阶段
                if test.progress < 0.25 {
                    test.current_phase = "初始化".to_string();
                } else if test.progress < 0.5 {
                    test.current_phase = "执行中".to_string();
                } else if test.progress < 0.75 {
                    test.current_phase = "分析中".to_string();
                } else {
                    test.current_phase = "完成中".to_string();
                }
            }
        }

        // 移除已完成的测试
        self.running_tests.retain(|test| test.progress < 1.0);
    }

    fn simulate_test_execution(&mut self, test: NetworkTest, _parameters: HashMap<String, TestParameterValue>) {
        // 模拟测试执行
        // 实际实现中应该在新线程中执行真实的测试逻辑
        
        let test_result = TestResult {
            test_id: test.id.clone(),
            test_name: test.name.clone(),
            start_time: Instant::now(),
            end_time: Instant::now() + Duration::from_secs(test.estimated_duration_s),
            status: TestStatus::Success,
            metrics: HashMap::from([
                ("latency_ms".to_string(), TestMetric {
                    name: "延迟".to_string(),
                    value: TestMetricValue::Number(50.0),
                    unit: Some("ms".to_string()),
                    description: Some("平均延迟".to_string()),
                }),
                ("packet_loss_rate".to_string(), TestMetric {
                    name: "丢包率".to_string(),
                    value: TestMetricValue::Percentage(0.1),
                    unit: Some("%".to_string()),
                    description: Some("丢包率".to_string()),
                }),
            ]),
            error_message: None,
        };

        self.test_results.push_back(test_result);

        // 限制结果历史长度
        while self.test_results.len() > self.max_results_history {
            self.test_results.pop_front();
        }
    }
}

// 数据可视化器实现
impl DataVisualizer {
    fn new() -> Self {
        Self {
            state: PanelState {
                visible: true,
                expanded: true,
                opacity: 1.0,
                scale: 1.0,
            },
            visualizations: Vec::new(),
            active_visualization: None,
            data_sources: HashMap::new(),
            update_interval_ms: 1000,
            last_update: Instant::now(),
        }
    }

    fn add_visualization(&mut self, viz: Visualization) {
        self.visualizations.push(viz);
    }

    fn add_data_source(&mut self, source: DataSource) {
        self.data_sources.insert(source.id.clone(), source);
    }

    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update).as_millis() >= self.update_interval_ms as u128 {
            // 更新数据源
            self.update_data_sources();
            self.last_update = now;
        }
    }

    fn update_data_sources(&mut self) {
        // 更新数据源
        // 这里应该从实际的数据源获取最新数据
        for source in self.data_sources.values_mut() {
            source.last_update = Instant::now();
            
            // 模拟数据更新
            let data_point = DataPoint {
                timestamp: Instant::now(),
                value: DataValue::Single(rand::random::<f64>() * 100.0),
                tags: HashMap::new(),
            };
            
            source.data_cache.push(data_point);
            
            // 限制缓存大小
            while source.data_cache.len() > source.max_cache_size {
                source.data_cache.remove(0);
            }
        }
    }
}

impl Default for InterfaceConfig {
    fn default() -> Self {
        Self {
            theme: InterfaceTheme::Default,
            update_frequency_hz: 1.0,
            auto_refresh: true,
            data_retention_s: 3600, // 1小时
            show_advanced_options: false,
            layout: InterfaceLayout {
                control_panel_position: PanelPosition::Left,
                status_display_position: PanelPosition::Top,
                visualization_area_position: PanelPosition::Center,
                panel_sizes: HashMap::new(),
                resizable_panels: true,
            },
            key_bindings: HashMap::from([
                ("F1".to_string(), "toggle_debug".to_string()),
                ("F5".to_string(), "refresh".to_string()),
                ("Ctrl+L".to_string(), "clear_history".to_string()),
            ]),
        }
    }
}

impl Default for InterfaceState {
    fn default() -> Self {
        Self {
            active_panel: Some("control_panel".to_string()),
            mouse_position: (0.0, 0.0),
            window_size: (1024.0, 768.0),
            fullscreen: false,
            zoom_level: 1.0,
            language: "zh-CN".to_string(),
            last_activity: Instant::now(),
        }
    }
}

impl Default for NetworkDebugInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_interface_creation() {
        let interface = NetworkDebugInterface::new();
        assert!(interface.is_enabled());
        assert!(interface.get_interface_state().active_panel.is_some());
    }

    #[test]
    fn test_command_execution() {
        let mut interface = NetworkDebugInterface::new();
        
        let result = interface.execute_command("help");
        match result {
            CommandResult::Success(message) => {
                assert!(message.contains("可用命令"));
            }
            _ => panic!("Expected success result"),
        }
    }

    #[test]
    fn test_interface_update() {
        let mut interface = NetworkDebugInterface::new();
        interface.update(Duration::from_millis(16)); // 60 FPS
        assert!(interface.is_active());
    }

    #[test]
    fn test_theme_change() {
        let mut interface = NetworkDebugInterface::new();
        interface.set_theme(InterfaceTheme::Dark);
        assert_eq!(interface.config.theme, InterfaceTheme::Dark);
    }

    #[test]
    fn test_event_history() {
        let mut interface = NetworkDebugInterface::new();
        interface.initialize();
        
        let events = interface.get_event_history();
        assert!(!events.is_empty());
        
        // 检查是否有初始化事件
        let init_events: Vec<_> = events.iter()
            .filter(|e| matches!(e.event_type, InterfaceEventType::SystemEvent))
            .collect();
        assert!(!init_events.is_empty());
    }
}