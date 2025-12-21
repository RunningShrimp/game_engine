//  网络模拟工具模块
// 
//  实现网络状况模拟、负载测试、故障模拟和恢复测试功能。
// 
//  ## 功能特性
// 
//  - 网络状况模拟器（延迟、丢包、抖动）
//  - 网络负载测试工具
//  - 网络故障模拟
//  - 网络恢复测试
//  - 自定义网络场景
//  - 模拟结果分析

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::thread;
use rand;

/// 网络模拟器
pub struct NetworkSimulator {
    /// 是否启用
    enabled: bool,
    /// 模拟器配置
    config: SimulatorConfig,
    /// 当前模拟状态
    simulation_state: SimulationState,
    /// 网络状况模拟器
    condition_simulator: NetworkConditionSimulator,
    /// 负载测试器
    load_tester: LoadTester,
    /// 故障模拟器
    failure_simulator: FailureSimulator,
    /// 恢复测试器
    recovery_tester: RecoveryTester,
    /// 模拟结果
    simulation_results: VecDeque<SimulationResult>,
    /// 最大结果历史长度
    max_results_history: usize,
    /// 活跃的模拟场景
    #[allow(dead_code)]
    active_scenarios: Vec<SimulationScenario>,
    /// 模拟统计
    statistics: SimulationStatistics,
}

/// 模拟器配置
#[derive(Debug, Clone)]
pub struct SimulatorConfig {
    /// 默认模拟参数
    pub default_parameters: SimulationParameters,
    /// 最大并发模拟数
    pub max_concurrent_simulations: usize,
    /// 模拟更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 是否启用实时模拟
    pub enable_realtime_simulation: bool,
    /// 是否保存模拟结果
    pub save_simulation_results: bool,
    /// 结果保存路径
    pub results_save_path: Option<String>,
    /// 是否启用详细日志
    pub enable_detailed_logging: bool,
}

/// 模拟参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParameters {
    /// 延迟参数
    pub latency: LatencyParameters,
    /// 丢包参数
    pub packet_loss: PacketLossParameters,
    /// 带宽参数
    pub bandwidth: BandwidthParameters,
    /// 抖动参数
    pub jitter: JitterParameters,
    /// 网络拥塞参数
    pub congestion: CongestionParameters,
}

/// 延迟参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyParameters {
    /// 基础延迟（毫秒）
    pub base_latency_ms: f32,
    /// 延迟变化范围（毫秒）
    pub latency_variation_ms: f32,
    /// 延迟分布类型
    pub distribution_type: LatencyDistribution,
    /// 是否启用突发延迟
    pub enable_burst_latency: bool,
    /// 突发延迟概率
    pub burst_probability: f32,
    /// 突发延迟倍数
    pub burst_multiplier: f32,
}

/// 延迟分布类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatencyDistribution {
    /// 固定延迟
    Fixed,
    /// 均匀分布
    Uniform,
    /// 正态分布
    Normal,
    /// 指数分布
    Exponential,
    /// 自定义分布
    Custom,
}

/// 丢包参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketLossParameters {
    /// 基础丢包率（0-1）
    pub base_loss_rate: f32,
    /// 丢包突发概率
    pub burst_probability: f32,
    /// 突发丢包率
    pub burst_loss_rate: f32,
    /// 突发持续时间（毫秒）
    pub burst_duration_ms: u64,
    /// 丢包模式
    pub loss_pattern: PacketLossPattern,
}

/// 丢包模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketLossPattern {
    /// 随机丢包
    Random,
    /// 连续丢包
    Consecutive,
    /// 周期性丢包
    Periodic,
    /// 突发丢包
    Burst,
    /// 自定义模式
    Custom,
}

/// 带宽参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthParameters {
    /// 上行带宽（Kbps）
    pub upload_bandwidth_kbps: f32,
    /// 下行带宽（Kbps）
    pub download_bandwidth_kbps: f32,
    /// 带宽波动幅度（0-1）
    pub bandwidth_fluctuation: f32,
    /// 波动周期（秒）
    pub fluctuation_period_s: u64,
    /// 是否启用带宽限制
    pub enable_throttling: bool,
    /// 限制策略
    pub throttling_strategy: ThrottlingStrategy,
}

/// 限制策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThrottlingStrategy {
    /// 固定限制
    Fixed,
    /// 动态限制
    Dynamic,
    /// 优先级限制
    Priority,
    /// 自适应限制
    Adaptive,
}

/// 抖动参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitterParameters {
    /// 抖动幅度（毫秒）
    pub jitter_amplitude_ms: f32,
    /// 抖动频率（Hz）
    pub jitter_frequency_hz: f32,
    /// 抖动分布类型
    pub jitter_distribution: JitterDistribution,
    /// 是否启用随机抖动
    pub enable_random_jitter: bool,
    /// 随机抖动概率
    pub random_probability: f32,
}

/// 抖动分布类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JitterDistribution {
    /// 均匀分布
    Uniform,
    /// 正态分布
    Normal,
    /// 指数分布
    Exponential,
}

/// 网络拥塞参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionParameters {
    /// 拥塞程度（0-1）
    pub congestion_level: f32,
    /// 拥塞持续时间（毫秒）
    pub congestion_duration_ms: u64,
    /// 拥塞间隔（毫秒）
    pub congestion_interval_ms: u64,
    /// 拥塞模式
    pub congestion_pattern: CongestionPattern,
}

/// 拥塞模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CongestionPattern {
    /// 恒定拥塞
    Constant,
    /// 周期性拥塞
    Periodic,
    /// 随机拥塞
    Random,
    /// 自适应拥塞
    Adaptive,
}

/// 模拟状态
#[derive(Debug, Clone)]
pub struct SimulationState {
    /// 是否正在模拟
    pub is_simulating: bool,
    /// 当前模拟时间
    pub simulation_time: Instant,
    /// 模拟进度（0-1）
    pub progress: f32,
    /// 当前模拟场景
    pub current_scenario: Option<String>,
    /// 活跃的模拟参数
    pub active_parameters: SimulationParameters,
    /// 模拟开始时间
    pub start_time: Option<Instant>,
    /// 预计结束时间
    pub estimated_end_time: Option<Instant>,
}

/// 网络状况模拟器
#[derive(Debug)]
pub struct NetworkConditionSimulator {
    /// 模拟器状态
    pub state: SimulationState,
    /// 当前网络状况
    pub current_condition: NetworkCondition,
    /// 预定义场景
    pub predefined_scenarios: HashMap<String, SimulationScenario>,
    /// 自定义场景
    pub custom_scenarios: HashMap<String, SimulationScenario>,
    /// 场景历史
    pub scenario_history: VecDeque<ScenarioExecution>,
    /// 最大历史长度
    pub max_history_length: usize,
}

/// 网络状况
#[derive(Debug, Clone)]
pub struct NetworkCondition {
    /// 当前延迟（毫秒）
    pub current_latency_ms: f32,
    /// 当前丢包率（0-1）
    pub current_packet_loss_rate: f32,
    /// 当前带宽利用率（0-1）
    pub current_bandwidth_utilization: f32,
    /// 当前抖动（毫秒）
    pub current_jitter_ms: f32,
    /// 拥塞程度（0-1）
    pub congestion_level: f32,
    /// 网络质量评分（0-100）
    pub quality_score: f32,
    /// 最后更新时间
    pub last_update: Instant,
}

/// 模拟场景
#[derive(Debug, Clone)]
pub struct SimulationScenario {
    /// 场景ID
    pub id: String,
    /// 场景名称
    pub name: String,
    /// 场景描述
    pub description: String,
    /// 场景类型
    pub scenario_type: ScenarioType,
    /// 模拟参数
    pub parameters: SimulationParameters,
    /// 持续时间（秒）
    pub duration_s: u64,
    /// 是否循环
    pub loop_scenario: bool,
    /// 场景阶段
    pub phases: Vec<ScenarioPhase>,
}

/// 场景类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioType {
    /// 预设场景
    Preset,
    /// 自定义场景
    Custom,
    /// 压力测试场景
    StressTest,
    /// 性能测试场景
    PerformanceTest,
    /// 故障恢复测试场景
    FailureRecoveryTest,
}

/// 场景阶段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioPhase {
    /// 阶段ID
    pub id: String,
    /// 阶段名称
    pub name: String,
    /// 阶段参数
    pub parameters: SimulationParameters,
    /// 阶段持续时间（秒）
    pub duration_s: u64,
    /// 阶段转换条件
    pub transition_condition: Option<String>,
}

/// 场景执行记录
#[derive(Debug, Clone)]
pub struct ScenarioExecution {
    /// 执行ID
    pub execution_id: u64,
    /// 场景ID
    pub scenario_id: String,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 执行状态
    pub status: ExecutionStatus,
    /// 执行结果
    pub result: Option<ExecutionResult>,
}

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// 准备中
    Preparing,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 失败
    Failed,
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 成功
    pub success: bool,
    /// 执行时间（秒）
    pub execution_time_s: f64,
    /// 错误信息
    pub error_message: Option<String>,
    /// 性能指标
    pub performance_metrics: HashMap<String, f64>,
}

/// 负载测试器
#[derive(Debug)]
pub struct LoadTester {
    /// 测试器状态
    pub state: SimulationState,
    /// 当前测试配置
    pub current_test_config: LoadTestConfig,
    /// 测试历史
    pub test_history: VecDeque<LoadTestResult>,
    /// 最大历史长度
    pub max_history_length: usize,
    /// 活跃的测试线程
    pub active_test_threads: Vec<thread::JoinHandle<()>>,
    /// 测试统计
    pub test_statistics: LoadTestStatistics,
}

/// 负载测试配置
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// 测试类型
    pub test_type: LoadTestType,
    /// 目标地址
    pub target_address: String,
    /// 并发连接数
    pub concurrent_connections: u32,
    /// 测试持续时间（秒）
    pub duration_s: u64,
    /// 请求间隔（毫秒）
    pub request_interval_ms: u64,
    /// 数据包大小
    pub packet_size: usize,
    /// 测试模式
    pub test_mode: TestMode,
    /// 是否启用逐步增加负载
    pub enable_ramp_up: bool,
    /// 负载增加时间（秒）
    pub ramp_up_time_s: u64,
}

/// 负载测试类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadTestType {
    /// 连接测试
    Connection,
    /// 带宽测试
    Bandwidth,
    /// 吞吐量测试
    Throughput,
    /// 延迟测试
    Latency,
    /// 综合测试
    Comprehensive,
}

/// 测试模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    /// 恒定负载
    Constant,
    /// 阶梯负载
    Step,
    /// 峰值负载
    Spike,
    /// 正弦负载
    Sine,
    /// 自定义负载
    Custom,
}

/// 负载测试结果
#[derive(Debug, Clone)]
pub struct LoadTestResult {
    /// 测试ID
    pub test_id: u64,
    /// 测试类型
    pub test_type: LoadTestType,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Instant,
    /// 测试状态
    pub status: TestStatus,
    /// 测试指标
    pub metrics: LoadTestMetrics,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 测试状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    /// 准备中
    Preparing,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 负载测试指标
#[derive(Debug, Clone)]
pub struct LoadTestMetrics {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f32,
    /// 最小延迟（毫秒）
    pub min_latency_ms: f32,
    /// 最大延迟（毫秒）
    pub max_latency_ms: f32,
    /// 延迟标准差
    pub latency_std_dev: f32,
    /// 吞吐量（请求/秒）
    pub throughput_rps: f32,
    /// 带宽利用率（Mbps）
    pub bandwidth_utilization_mbps: f32,
    /// 错误率（0-1）
    pub error_rate: f32,
}

/// 负载测试统计
#[derive(Debug, Clone, Default)]
pub struct LoadTestStatistics {
    /// 总测试数
    pub total_tests: u64,
    /// 成功测试数
    pub successful_tests: u64,
    /// 失败测试数
    pub failed_tests: u64,
    /// 平均测试时间（秒）
    pub average_test_time_s: f64,
    /// 最佳性能指标
    pub best_performance: Option<LoadTestMetrics>,
    /// 最差性能指标
    pub worst_performance: Option<LoadTestMetrics>,
}

/// 故障模拟器
#[derive(Debug)]
pub struct FailureSimulator {
    /// 模拟器状态
    pub state: SimulationState,
    /// 当前故障场景
    pub current_failure_scenario: Option<FailureScenario>,
    /// 预定义故障类型
    pub predefined_failures: HashMap<String, FailureType>,
    /// 故障历史
    pub failure_history: VecDeque<FailureExecution>,
    /// 最大历史长度
    pub max_history_length: usize,
    /// 故障注入器
    pub failure_injector: FailureInjector,
}

/// 故障场景
#[derive(Debug, Clone)]
pub struct FailureScenario {
    /// 场景ID
    pub id: String,
    /// 场景名称
    pub name: String,
    /// 场景描述
    pub description: String,
    /// 故障类型
    pub failure_type: FailureType,
    /// 故障参数
    pub parameters: FailureParameters,
    /// 触发条件
    pub trigger_condition: Option<String>,
    /// 恢复条件
    pub recovery_condition: Option<String>,
}

/// 故障类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureType {
    /// 连接中断
    ConnectionInterruption,
    /// 网络分区
    NetworkPartition,
    /// 延迟峰值
    LatencySpike,
    /// 丢包突发
    PacketLossBurst,
    /// 带宽限制
    BandwidthThrottling,
    /// 服务器过载
    ServerOverload,
    /// 自定义故障
    Custom,
}

/// 故障参数
#[derive(Debug, Clone)]
pub struct FailureParameters {
    /// 故障持续时间（毫秒）
    pub duration_ms: u64,
    /// 故障强度（0-1）
    pub intensity: f32,
    /// 故障频率
    pub frequency: f32,
    /// 影响范围
    pub affected_components: Vec<String>,
    /// 自定义参数
    pub custom_parameters: HashMap<String, String>,
}

/// 故障执行记录
#[derive(Debug, Clone)]
pub struct FailureExecution {
    /// 执行ID
    pub execution_id: u64,
    /// 故障类型
    pub failure_type: FailureType,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 执行状态
    pub status: ExecutionStatus,
    /// 执行结果
    pub result: Option<ExecutionResult>,
}

/// 故障注入器
#[derive(Debug)]
pub struct FailureInjector {
    /// 注入方法
    pub injection_method: InjectionMethod,
    /// 注入目标
    pub injection_targets: Vec<String>,
    /// 注入状态
    pub injection_state: HashMap<String, InjectionState>,
}

/// 注入方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionMethod {
    /// 直接注入
    Direct,
    /// 代理注入
    Proxy,
    /// 拦截注入
    Interception,
    /// 模拟注入
    Simulation,
}

/// 注入状态
#[derive(Debug, Clone)]
pub struct InjectionState {
    /// 是否已注入
    pub injected: bool,
    /// 注入时间
    pub injection_time: Instant,
    /// 注入参数
    pub injection_parameters: HashMap<String, String>,
}

/// 恢复测试器
#[derive(Debug)]
pub struct RecoveryTester {
    /// 测试器状态
    pub state: SimulationState,
    /// 当前测试配置
    pub current_test_config: RecoveryTestConfig,
    /// 测试历史
    pub test_history: VecDeque<RecoveryTestResult>,
    /// 最大历史长度
    pub max_history_length: usize,
    /// 恢复策略
    pub recovery_strategies: HashMap<String, RecoveryStrategy>,
}

/// 恢复测试配置
#[derive(Debug, Clone)]
pub struct RecoveryTestConfig {
    /// 测试类型
    pub test_type: RecoveryTestType,
    /// 故障类型
    pub failure_type: FailureType,
    /// 故障参数
    pub failure_parameters: FailureParameters,
    /// 恢复策略
    pub recovery_strategy: String,
    /// 恢复超时时间（秒）
    pub recovery_timeout_s: u64,
    /// 测试轮数
    pub test_rounds: u32,
    /// 是否启用自动恢复
    pub enable_auto_recovery: bool,
}

/// 恢复测试类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryTestType {
    /// 连接恢复
    ConnectionRecovery,
    /// 服务恢复
    ServiceRecovery,
    /// 数据恢复
    DataRecovery,
    /// 性能恢复
    PerformanceRecovery,
    /// 综合恢复
    Comprehensive,
}

/// 恢复策略
#[derive(Debug, Clone)]
pub struct RecoveryStrategy {
    /// 策略ID
    pub id: String,
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 恢复步骤
    pub recovery_steps: Vec<RecoveryStep>,
    /// 预计恢复时间（秒）
    pub estimated_recovery_time_s: u64,
    /// 成功率（0-1）
    pub success_rate: f32,
}

/// 恢复步骤
#[derive(Debug, Clone)]
pub struct RecoveryStep {
    /// 步骤ID
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 步骤类型
    pub step_type: RecoveryStepType,
    /// 步骤参数
    pub parameters: HashMap<String, String>,
    /// 超时时间（秒）
    pub timeout_s: u64,
    /// 重试次数
    pub retry_count: u32,
}

/// 恢复步骤类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStepType {
    /// 重试连接
    RetryConnection,
    /// 重置状态
    ResetState,
    /// 重新同步
    Resynchronize,
    /// 重建连接
    RebuildConnection,
    /// 切换服务器
    SwitchServer,
    /// 自定义步骤
    Custom,
}

/// 恢复测试结果
#[derive(Debug, Clone)]
pub struct RecoveryTestResult {
    /// 测试ID
    pub test_id: u64,
    /// 测试类型
    pub test_type: RecoveryTestType,
    /// 故障类型
    pub failure_type: FailureType,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Instant,
    /// 测试状态
    pub status: TestStatus,
    /// 恢复指标
    pub recovery_metrics: RecoveryMetrics,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 恢复指标
#[derive(Debug, Clone)]
pub struct RecoveryMetrics {
    /// 恢复时间（秒）
    pub recovery_time_s: f64,
    /// 恢复成功率（0-1）
    pub recovery_success_rate: f32,
    /// 平均重试次数
    pub average_retry_count: f32,
    /// 数据丢失量
    pub data_loss_amount: u64,
    /// 服务中断时间（秒）
    pub service_downtime_s: f64,
}

/// 模拟结果
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// 结果ID
    pub result_id: u64,
    /// 模拟类型
    pub simulation_type: SimulationType,
    /// 开始时间
    pub start_time: Instant,
    /// 结束时间
    pub end_time: Instant,
    /// 模拟参数
    pub parameters: SimulationParameters,
    /// 模拟指标
    pub metrics: SimulationMetrics,
    /// 成功状态
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 模拟类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Hash)]
pub enum SimulationType {
    /// 网络状况模拟
    NetworkCondition,
    /// 负载测试
    LoadTest,
    /// 故障模拟
    FailureSimulation,
    /// 恢复测试
    RecoveryTest,
    /// 综合测试
    Comprehensive,
}

/// 模拟指标
#[derive(Debug, Clone)]
pub struct SimulationMetrics {
    /// 平均延迟（毫秒）
    pub average_latency_ms: f32,
    /// 延迟分布
    pub latency_distribution: LatencyDistributionStats,
    /// 丢包率（0-1）
    pub packet_loss_rate: f32,
    /// 带宽利用率（0-1）
    pub bandwidth_utilization: f32,
    /// 吞吐量（请求/秒）
    pub throughput_rps: f32,
    /// 错误率（0-1）
    pub error_rate: f32,
    /// 网络质量评分（0-100）
    pub quality_score: f32,
    /// 自定义指标
    pub custom_metrics: HashMap<String, f64>,
}

/// 延迟分布统计
#[derive(Debug, Clone)]
pub struct LatencyDistributionStats {
    /// 最小延迟
    pub min_latency_ms: f32,
    /// 最大延迟
    pub max_latency_ms: f32,
    /// 平均延迟
    pub average_latency_ms: f32,
    /// 中位数延迟
    pub median_latency_ms: f32,
    /// 95%分位数延迟
    pub p95_latency_ms: f32,
    /// 99%分位数延迟
    pub p99_latency_ms: f32,
}

/// 模拟统计
#[derive(Debug, Clone, Default)]
pub struct SimulationStatistics {
    /// 总模拟数
    pub total_simulations: u64,
    /// 成功模拟数
    pub successful_simulations: u64,
    /// 失败模拟数
    pub failed_simulations: u64,
    /// 平均模拟时间（秒）
    pub average_simulation_time_s: f64,
    /// 最佳性能指标
    pub best_performance: Option<SimulationMetrics>,
    /// 最差性能指标
    pub worst_performance: Option<SimulationMetrics>,
    /// 按类型分类的统计
    pub statistics_by_type: HashMap<SimulationType, TypeStatistics>,
}

/// 按类型分类的统计
#[derive(Debug, Clone, Default)]
pub struct TypeStatistics {
    /// 模拟次数
    pub simulation_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 平均指标
    pub average_metrics: Option<SimulationMetrics>,
}

impl NetworkSimulator {
    /// 创建新的网络模拟器
    pub fn new() -> Self {
        Self::with_config(SimulatorConfig::default())
    }

    /// 创建带配置的网络模拟器
    pub fn with_config(config: SimulatorConfig) -> Self {
        Self {
            enabled: true,
            config: config.clone(),
            simulation_state: SimulationState::default(),
            condition_simulator: NetworkConditionSimulator::new(),
            load_tester: LoadTester::new(),
            failure_simulator: FailureSimulator::new(),
            recovery_tester: RecoveryTester::new(),
            simulation_results: VecDeque::with_capacity(1000),
            max_results_history: 1000,
            active_scenarios: Vec::new(),
            statistics: SimulationStatistics::default(),
        }
    }

    /// 初始化模拟器
    pub fn initialize(&mut self) {
        // 初始化各个子模拟器
        self.condition_simulator.initialize();
        self.load_tester.initialize();
        self.failure_simulator.initialize();
        self.recovery_tester.initialize();

        // 加载预定义场景
        self.load_predefined_scenarios();

        // 加载预定义故障类型
        self.load_predefined_failures();

        // 加载恢复策略
        self.load_recovery_strategies();
    }

    /// 开始网络状况模拟
    pub fn start_condition_simulation(&mut self, scenario_id: &str) -> Result<(), String> {
        if self.simulation_state.is_simulating {
            return Err("已有模拟正在运行".to_string());
        }

        // 查找场景
        let scenario = self.find_scenario(scenario_id)?;
        
        // 设置模拟状态
        self.simulation_state = SimulationState {
            is_simulating: true,
            simulation_time: Instant::now(),
            progress: 0.0,
            current_scenario: Some(scenario_id.to_string()),
            active_parameters: scenario.parameters.clone(),
            start_time: Some(Instant::now()),
            estimated_end_time: Some(Instant::now() + Duration::from_secs(scenario.duration_s)),
        };

        // 开始场景模拟
        self.condition_simulator.start_scenario(scenario)?;

        Ok(())
    }

    /// 开始负载测试
    pub fn start_load_test(&mut self, test_config: LoadTestConfig) -> Result<u64, String> {
        if self.simulation_state.is_simulating {
            return Err("已有模拟正在运行".to_string());
        }

        // 设置模拟状态
        self.simulation_state = SimulationState {
            is_simulating: true,
            simulation_time: Instant::now(),
            progress: 0.0,
            current_scenario: Some("load_test".to_string()),
            active_parameters: self.config.default_parameters.clone(),
            start_time: Some(Instant::now()),
            estimated_end_time: Some(Instant::now() + Duration::from_secs(test_config.duration_s)),
        };

        // 开始负载测试
        let test_id = self.load_tester.start_test(test_config)?;

        Ok(test_id)
    }

    /// 开始故障模拟
    pub fn start_failure_simulation(&mut self, scenario_id: &str) -> Result<u64, String> {
        if self.simulation_state.is_simulating {
            return Err("已有模拟正在运行".to_string());
        }

        // 查找故障场景
        let scenario = self.find_failure_scenario(scenario_id)?;
        
        // 设置模拟状态
        self.simulation_state = SimulationState {
            is_simulating: true,
            simulation_time: Instant::now(),
            progress: 0.0,
            current_scenario: Some(scenario_id.to_string()),
            active_parameters: self.config.default_parameters.clone(),
            start_time: Some(Instant::now()),
            estimated_end_time: Some(Instant::now() + Duration::from_millis(scenario.parameters.duration_ms)),
        };

        // 开始故障模拟
        let execution_id = self.failure_simulator.start_failure(scenario)?;

        Ok(execution_id)
    }

    /// 开始恢复测试
    pub fn start_recovery_test(&mut self, test_config: RecoveryTestConfig) -> Result<u64, String> {
        if self.simulation_state.is_simulating {
            return Err("已有模拟正在运行".to_string());
        }

        // 设置模拟状态
        self.simulation_state = SimulationState {
            is_simulating: true,
            simulation_time: Instant::now(),
            progress: 0.0,
            current_scenario: Some("recovery_test".to_string()),
            active_parameters: self.config.default_parameters.clone(),
            start_time: Some(Instant::now()),
            estimated_end_time: Some(Instant::now() + Duration::from_secs(test_config.recovery_timeout_s)),
        };

        // 开始恢复测试
        let test_id = self.recovery_tester.start_test(test_config)?;

        Ok(test_id)
    }

    /// 停止当前模拟
    pub fn stop_simulation(&mut self) -> Result<(), String> {
        if !self.simulation_state.is_simulating {
            return Err("没有正在运行的模拟".to_string());
        }

        // 停止各个子模拟器
        self.condition_simulator.stop_simulation();
        self.load_tester.stop_test();
        self.failure_simulator.stop_failure();
        self.recovery_tester.stop_test();

        // 重置模拟状态
        self.simulation_state = SimulationState::default();

        Ok(())
    }

    /// 更新模拟器
    pub fn update(&mut self, delta_time: Duration) {
        if !self.enabled {
            return;
        }

        if self.simulation_state.is_simulating {
            // 更新模拟进度
            self.update_simulation_progress();

            // 更新各个子模拟器
            self.condition_simulator.update(delta_time);
            self.load_tester.update(delta_time);
            self.failure_simulator.update(delta_time);
            self.recovery_tester.update(delta_time);

            // 检查模拟是否完成
            self.check_simulation_completion();
        }
    }

    /// 获取当前模拟状态
    pub fn get_simulation_state(&self) -> &SimulationState {
        &self.simulation_state
    }

    /// 获取模拟结果历史
    pub fn get_simulation_results(&self) -> Vec<SimulationResult> {
        self.simulation_results.iter().cloned().collect()
    }

    /// 获取模拟统计
    pub fn get_statistics(&self) -> &SimulationStatistics {
        &self.statistics
    }

    /// 获取网络状况
    pub fn get_current_condition(&self) -> Option<&NetworkCondition> {
        self.condition_simulator.get_current_condition()
    }

    /// 获取负载测试结果
    pub fn get_load_test_results(&self) -> Vec<LoadTestResult> {
        self.load_tester.get_test_results()
    }

    /// 获取故障模拟结果
    pub fn get_failure_results(&self) -> Vec<FailureExecution> {
        self.failure_simulator.get_failure_results()
    }

    /// 获取恢复测试结果
    pub fn get_recovery_test_results(&self) -> Vec<RecoveryTestResult> {
        self.recovery_tester.get_test_results()
    }

    /// 添加自定义场景
    pub fn add_custom_scenario(&mut self, scenario: SimulationScenario) {
        self.condition_simulator.add_custom_scenario(scenario);
    }

    /// 添加自定义故障场景
    pub fn add_custom_failure_scenario(&mut self, scenario: FailureScenario) {
        self.failure_simulator.add_custom_scenario(scenario);
    }

    /// 添加自定义恢复策略
    pub fn add_custom_recovery_strategy(&mut self, strategy: RecoveryStrategy) {
        self.recovery_tester.add_custom_strategy(strategy);
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
        self.enabled && self.simulation_state.is_simulating
    }

    // 私有方法

    /// 查找场景
    fn find_scenario(&self, scenario_id: &str) -> Result<SimulationScenario, String> {
        self.condition_simulator.find_scenario(scenario_id)
            .ok_or_else(|| format!("场景不存在: {}", scenario_id))
    }

    /// 查找故障场景
    fn find_failure_scenario(&self, scenario_id: &str) -> Result<FailureScenario, String> {
        self.failure_simulator.find_scenario(scenario_id)
            .ok_or_else(|| format!("故障场景不存在: {}", scenario_id))
    }

    /// 更新模拟进度
    fn update_simulation_progress(&mut self) {
        if let (Some(start_time), Some(end_time)) = 
            (self.simulation_state.start_time, self.simulation_state.estimated_end_time) {
            let total_duration = end_time.duration_since(start_time);
            let elapsed = start_time.elapsed();
            
            if total_duration.as_secs() > 0 {
                self.simulation_state.progress = 
                    (elapsed.as_secs_f64() / total_duration.as_secs_f64()) as f32;
            }
        }
    }

    /// 检查模拟完成
    fn check_simulation_completion(&mut self) {
        if let Some(end_time) = self.simulation_state.estimated_end_time {
            if Instant::now() >= end_time {
                // 模拟完成，生成结果
                self.generate_simulation_result();
                
                // 停止模拟
                let _ = self.stop_simulation();
            }
        }
    }

    /// 生成模拟结果
    fn generate_simulation_result(&mut self) {
        let result = SimulationResult {
            result_id: rand::random(),
            simulation_type: self.determine_simulation_type(),
            start_time: self.simulation_state.start_time.unwrap_or_else(Instant::now),
            end_time: Instant::now(),
            parameters: self.simulation_state.active_parameters.clone(),
            metrics: self.collect_simulation_metrics(),
            success: true,
            error_message: None,
        };

        // 先更新统计信息再移动结果
        self.update_statistics(&result);
        
        // 保存结果
        self.simulation_results.push_back(result);

        // 限制结果历史长度
        while self.simulation_results.len() > self.max_results_history {
            self.simulation_results.pop_front();
        }
    }

    /// 确定模拟类型
    fn determine_simulation_type(&self) -> SimulationType {
        if let Some(scenario_id) = &self.simulation_state.current_scenario {
            if scenario_id == "load_test" {
                SimulationType::LoadTest
            } else if scenario_id == "recovery_test" {
                SimulationType::RecoveryTest
            } else if self.failure_simulator.is_simulating() {
                SimulationType::FailureSimulation
            } else {
                SimulationType::NetworkCondition
            }
        } else {
            SimulationType::NetworkCondition
        }
    }

    /// 收集模拟指标
    fn collect_simulation_metrics(&self) -> SimulationMetrics {
        // 从各个子模拟器收集指标
        let mut metrics = SimulationMetrics::default();

        // 从网络状况模拟器收集指标
        if let Some(condition) = self.get_current_condition() {
            metrics.average_latency_ms = condition.current_latency_ms;
            metrics.packet_loss_rate = condition.current_packet_loss_rate;
            metrics.bandwidth_utilization = condition.current_bandwidth_utilization;
            metrics.quality_score = condition.quality_score;
        }

        // 从负载测试器收集指标
        let load_test_results = self.get_load_test_results();
        if !load_test_results.is_empty() {
            let latest_result = &load_test_results[load_test_results.len() - 1];
            metrics.throughput_rps = latest_result.metrics.throughput_rps;
            metrics.error_rate = latest_result.metrics.error_rate;
        }

        metrics
    }

    /// 更新统计信息
    fn update_statistics(&mut self, result: &SimulationResult) {
        self.statistics.total_simulations += 1;
        
        if result.success {
            self.statistics.successful_simulations += 1;
        } else {
            self.statistics.failed_simulations += 1;
        }

        // 更新平均模拟时间
        let simulation_time = result.end_time.duration_since(result.start_time).as_secs_f64();
        let total_simulations = self.statistics.total_simulations as f64;
        self.statistics.average_simulation_time_s = 
            (self.statistics.average_simulation_time_s * (total_simulations - 1.0) + simulation_time) / total_simulations;

        // 更新最佳/最差性能
        self.update_performance_extremes(&result.metrics);

        // 更新按类型分类的统计
        self.update_type_statistics(result);
    }

    /// 更新性能极值
    fn update_performance_extremes(&mut self, metrics: &SimulationMetrics) {
        // 更新最佳性能
        if self.statistics.best_performance.is_none() || 
           self.is_better_performance(metrics, self.statistics.best_performance.as_ref().unwrap()) {
            self.statistics.best_performance = Some(metrics.clone());
        }

        // 更新最差性能
        if self.statistics.worst_performance.is_none() || 
           self.is_worse_performance(metrics, self.statistics.worst_performance.as_ref().unwrap()) {
            self.statistics.worst_performance = Some(metrics.clone());
        }
    }

    /// 判断是否为更好的性能
    fn is_better_performance(&self, current: &SimulationMetrics, best: &SimulationMetrics) -> bool {
        // 简化的性能比较：延迟越低、丢包率越低、质量分数越高越好
        current.average_latency_ms < best.average_latency_ms ||
        current.packet_loss_rate < best.packet_loss_rate ||
        current.quality_score > best.quality_score
    }

    /// 判断是否为更差的性能
    fn is_worse_performance(&self, current: &SimulationMetrics, worst: &SimulationMetrics) -> bool {
        // 简化的性能比较
        current.average_latency_ms > worst.average_latency_ms ||
        current.packet_loss_rate > worst.packet_loss_rate ||
        current.quality_score < worst.quality_score
    }

    /// 更新按类型分类的统计
    fn update_type_statistics(&mut self, result: &SimulationResult) {
        let sim_type = result.simulation_type;
        let type_stats = self.statistics.statistics_by_type.entry(sim_type).or_insert_with(TypeStatistics::default);
        
        type_stats.simulation_count += 1;
        
        if result.success {
            type_stats.success_count += 1;
        }

        // 更新平均指标
        // 使用一个临时变量来检查是否更好，避免 borrow 冲突
        let should_update = if let Some(avg) = &type_stats.average_metrics {
            // 内联is_better_performance的逻辑以避免borrow冲突
            // 简化的性能比较：延迟越低、丢包率越低、质量分数越高越好
            result.metrics.average_latency_ms < avg.average_latency_ms ||
            result.metrics.packet_loss_rate < avg.packet_loss_rate ||
            result.metrics.quality_score > avg.quality_score
        } else {
            true
        };
            
        if should_update {
            type_stats.average_metrics = Some(result.metrics.clone());
        }
    }

    /// 加载预定义场景
    fn load_predefined_scenarios(&mut self) {
        // 加载预设的网络状况场景
        self.condition_simulator.add_predefined_scenario(SimulationScenario {
            id: "perfect_network".to_string(),
            name: "完美网络".to_string(),
            description: "理想的网络状况，低延迟、无丢包".to_string(),
            scenario_type: ScenarioType::Preset,
            parameters: SimulationParameters {
                latency: LatencyParameters {
                    base_latency_ms: 10.0,
                    latency_variation_ms: 2.0,
                    distribution_type: LatencyDistribution::Normal,
                    enable_burst_latency: false,
                    burst_probability: 0.0,
                    burst_multiplier: 1.0,
                },
                packet_loss: PacketLossParameters {
                    base_loss_rate: 0.0,
                    burst_probability: 0.0,
                    burst_loss_rate: 0.0,
                    burst_duration_ms: 0,
                    loss_pattern: PacketLossPattern::Random,
                },
                bandwidth: BandwidthParameters {
                    upload_bandwidth_kbps: 10000.0,
                    download_bandwidth_kbps: 10000.0,
                    bandwidth_fluctuation: 0.1,
                    fluctuation_period_s: 10,
                    enable_throttling: false,
                    throttling_strategy: ThrottlingStrategy::Fixed,
                },
                jitter: JitterParameters {
                    jitter_amplitude_ms: 1.0,
                    jitter_frequency_hz: 1.0,
                    jitter_distribution: JitterDistribution::Normal,
                    enable_random_jitter: false,
                    random_probability: 0.0,
                },
                congestion: CongestionParameters {
                    congestion_level: 0.0,
                    congestion_duration_ms: 0,
                    congestion_interval_ms: 0,
                    congestion_pattern: CongestionPattern::Constant,
                },
            },
            duration_s: 60,
            loop_scenario: false,
            phases: Vec::new(),
        });

        // 添加更多预设场景...
        self.condition_simulator.add_predefined_scenario(SimulationScenario {
            id: "poor_network".to_string(),
            name: "糟糕网络".to_string(),
            description: "网络状况较差，高延迟、高丢包率".to_string(),
            scenario_type: ScenarioType::Preset,
            parameters: SimulationParameters {
                latency: LatencyParameters {
                    base_latency_ms: 200.0,
                    latency_variation_ms: 50.0,
                    distribution_type: LatencyDistribution::Normal,
                    enable_burst_latency: true,
                    burst_probability: 0.1,
                    burst_multiplier: 3.0,
                },
                packet_loss: PacketLossParameters {
                    base_loss_rate: 0.1,
                    burst_probability: 0.05,
                    burst_loss_rate: 0.3,
                    burst_duration_ms: 5000,
                    loss_pattern: PacketLossPattern::Burst,
                },
                bandwidth: BandwidthParameters {
                    upload_bandwidth_kbps: 1000.0,
                    download_bandwidth_kbps: 1000.0,
                    bandwidth_fluctuation: 0.5,
                    fluctuation_period_s: 5,
                    enable_throttling: true,
                    throttling_strategy: ThrottlingStrategy::Dynamic,
                },
                jitter: JitterParameters {
                    jitter_amplitude_ms: 20.0,
                    jitter_frequency_hz: 5.0,
                    jitter_distribution: JitterDistribution::Normal,
                    enable_random_jitter: true,
                    random_probability: 0.2,
                },
                congestion: CongestionParameters {
                    congestion_level: 0.7,
                    congestion_duration_ms: 10000,
                    congestion_interval_ms: 30000,
                    congestion_pattern: CongestionPattern::Periodic,
                },
            },
            duration_s: 60,
            loop_scenario: false,
            phases: Vec::new(),
        });
    }

    /// 加载预定义故障类型
    fn load_predefined_failures(&mut self) {
        // 添加预定义的故障类型
        self.failure_simulator.add_predefined_failure(FailureScenario {
            id: "connection_interruption".to_string(),
            name: "连接中断".to_string(),
            description: "模拟网络连接中断".to_string(),
            failure_type: FailureType::ConnectionInterruption,
            parameters: FailureParameters {
                duration_ms: 5000,
                intensity: 1.0,
                frequency: 0.1,
                affected_components: vec!["network".to_string()],
                custom_parameters: HashMap::new(),
            },
            trigger_condition: None,
            recovery_condition: Some("auto_recovery".to_string()),
        });

        // 添加更多预定义故障类型...
    }

    /// 加载恢复策略
    fn load_recovery_strategies(&mut self) {
        // 添加预定义的恢复策略
        self.recovery_tester.add_predefined_strategy(RecoveryStrategy {
            id: "auto_recovery".to_string(),
            name: "自动恢复".to_string(),
            description: "自动检测并恢复网络连接".to_string(),
            recovery_steps: vec![
                RecoveryStep {
                    id: "detect_failure".to_string(),
                    name: "检测故障".to_string(),
                    description: "检测网络故障类型".to_string(),
                    step_type: RecoveryStepType::Custom,
                    parameters: HashMap::new(),
                    timeout_s: 5,
                    retry_count: 3,
                },
                RecoveryStep {
                    id: "retry_connection".to_string(),
                    name: "重试连接".to_string(),
                    description: "尝试重新建立连接".to_string(),
                    step_type: RecoveryStepType::RetryConnection,
                    parameters: HashMap::new(),
                    timeout_s: 10,
                    retry_count: 5,
                },
                RecoveryStep {
                    id: "reset_state".to_string(),
                    name: "重置状态".to_string(),
                    description: "重置网络状态".to_string(),
                    step_type: RecoveryStepType::ResetState,
                    parameters: HashMap::new(),
                    timeout_s: 5,
                    retry_count: 1,
                },
            ],
            estimated_recovery_time_s: 30,
            success_rate: 0.8,
        });

        // 添加更多预定义恢复策略...
    }
}

// 网络状况模拟器实现
impl NetworkConditionSimulator {
    fn new() -> Self {
        Self {
            state: SimulationState::default(),
            current_condition: NetworkCondition::default(),
            predefined_scenarios: HashMap::new(),
            custom_scenarios: HashMap::new(),
            scenario_history: VecDeque::with_capacity(100),
            max_history_length: 100,
        }
    }

    fn initialize(&mut self) {
        // 初始化网络状况
        self.current_condition = NetworkCondition::default();
    }

    fn add_predefined_scenario(&mut self, scenario: SimulationScenario) {
        self.predefined_scenarios.insert(scenario.id.clone(), scenario);
    }

    fn add_custom_scenario(&mut self, scenario: SimulationScenario) {
        self.custom_scenarios.insert(scenario.id.clone(), scenario);
    }

    fn start_scenario(&mut self, scenario: SimulationScenario) -> Result<(), String> {
        // 设置当前网络状况
        self.current_condition = NetworkCondition {
            current_latency_ms: scenario.parameters.latency.base_latency_ms,
            current_packet_loss_rate: scenario.parameters.packet_loss.base_loss_rate,
            current_bandwidth_utilization: 0.0,
            current_jitter_ms: scenario.parameters.jitter.jitter_amplitude_ms,
            congestion_level: scenario.parameters.congestion.congestion_level,
            quality_score: self.calculate_quality_score(&scenario.parameters),
            last_update: Instant::now(),
        };

        // 添加到历史
        self.scenario_history.push_back(ScenarioExecution {
            execution_id: rand::random(),
            scenario_id: scenario.id,
            start_time: Instant::now(),
            end_time: None,
            status: ExecutionStatus::Running,
            result: None,
        });

        Ok(())
    }

    fn stop_simulation(&mut self) {
        self.state.is_simulating = false;
        self.current_condition.last_update = Instant::now();
    }

    fn update(&mut self, _delta_time: Duration) {
        if !self.state.is_simulating {
            return;
        }

        // 更新网络状况
        self.update_network_condition();
    }

    fn get_current_condition(&self) -> Option<&NetworkCondition> {
        Some(&self.current_condition)
    }

    fn find_scenario(&self, scenario_id: &str) -> Option<SimulationScenario> {
        self.predefined_scenarios.get(scenario_id)
            .or_else(|| self.custom_scenarios.get(scenario_id))
            .cloned()
    }

    fn update_network_condition(&mut self) {
        // 根据当前参数更新网络状况
        // 这里可以实现更复杂的网络状况模拟逻辑
        
        // 简化实现：添加一些随机变化
        let random_factor = rand::random::<f32>() * 0.1 - 0.05; // -0.05 到 0.05
        self.current_condition.current_latency_ms *= 1.0 + random_factor;
        self.current_condition.current_packet_loss_rate = (self.current_condition.current_packet_loss_rate + random_factor).max(0.0).min(1.0);
        
        // 更新质量评分
        self.current_condition.quality_score = self.calculate_quality_score_from_condition(&self.current_condition);
        self.current_condition.last_update = Instant::now();
    }

    fn calculate_quality_score(&self, parameters: &SimulationParameters) -> f32 {
        // 简化的质量评分计算
        let latency_score = (100.0 - parameters.latency.base_latency_ms).max(0.0) / 100.0;
        let loss_score = (1.0 - parameters.packet_loss.base_loss_rate) * 100.0;
        let bandwidth_score = parameters.bandwidth.upload_bandwidth_kbps / 10000.0; // 假设10Gbps为满分
        let jitter_score = (100.0 - parameters.jitter.jitter_amplitude_ms).max(0.0) / 100.0;
        
        (latency_score + loss_score + bandwidth_score + jitter_score) / 4.0
    }

    fn calculate_quality_score_from_condition(&self, condition: &NetworkCondition) -> f32 {
        let latency_score = (100.0 - condition.current_latency_ms).max(0.0) / 100.0;
        let loss_score = (1.0 - condition.current_packet_loss_rate) * 100.0;
        let jitter_score = (100.0 - condition.current_jitter_ms).max(0.0) / 100.0;
        
        (latency_score + loss_score + jitter_score) / 3.0
    }
}

// 负载测试器实现
impl LoadTester {
    fn new() -> Self {
        Self {
            state: SimulationState::default(),
            current_test_config: LoadTestConfig::default(),
            test_history: VecDeque::with_capacity(100),
            max_history_length: 100,
            active_test_threads: Vec::new(),
            test_statistics: LoadTestStatistics::default(),
        }
    }

    fn initialize(&mut self) {
        // 初始化负载测试器
    }

    fn start_test(&mut self, test_config: LoadTestConfig) -> Result<u64, String> {
        let test_id = rand::random();
        
        // 设置测试配置
        self.current_test_config = test_config.clone();
        
        // 设置模拟状态
        self.state = SimulationState {
            is_simulating: true,
            simulation_time: Instant::now(),
            progress: 0.0,
            current_scenario: Some("load_test".to_string()),
            active_parameters: SimulationParameters::default(),
            start_time: Some(Instant::now()),
            estimated_end_time: Some(Instant::now() + Duration::from_secs(test_config.duration_s)),
        };

        // 模拟负载测试执行
        // 实际实现中应该在新线程中执行测试
        self.simulate_load_test(test_id);

        Ok(test_id)
    }

    fn stop_test(&mut self) {
        self.state.is_simulating = false;
        
        // 停止所有测试线程
        for thread in self.active_test_threads.drain(..) {
            // 在实际实现中，应该优雅地停止线程
            drop(thread);
        }
    }

    fn update(&mut self, _delta_time: Duration) {
        if !self.state.is_simulating {
            return;
        }

        // 更新测试进度
        self.update_test_progress();

        // 检查测试是否完成
        self.check_test_completion();
    }

    fn get_test_results(&self) -> Vec<LoadTestResult> {
        self.test_history.iter().cloned().collect()
    }

    fn simulate_load_test(&mut self, test_id: u64) {
        // 模拟负载测试执行
        let test_result = LoadTestResult {
            test_id,
            test_type: self.current_test_config.test_type,
            start_time: self.state.start_time.unwrap_or_else(Instant::now),
            end_time: self.state.estimated_end_time.unwrap_or_else(|| Instant::now() + Duration::from_secs(60)),
            status: TestStatus::Completed,
            metrics: LoadTestMetrics {
                total_requests: 1000,
                successful_requests: 950,
                failed_requests: 50,
                average_latency_ms: 50.0,
                min_latency_ms: 10.0,
                max_latency_ms: 200.0,
                latency_std_dev: 20.0,
                throughput_rps: 100.0,
                bandwidth_utilization_mbps: 50.0,
                error_rate: 0.05,
            },
            error_message: None,
        };

        self.test_history.push_back(test_result.clone());

        // 限制历史长度
        while self.test_history.len() > self.max_history_length {
            self.test_history.pop_front();
        }

        // 更新统计信息
        self.update_test_statistics(&test_result);
    }

    fn update_test_progress(&mut self) {
        if let (Some(start_time), Some(end_time)) = 
            (self.state.start_time, self.state.estimated_end_time) {
            let total_duration = end_time.duration_since(start_time);
            let elapsed = start_time.elapsed();
            
            if total_duration.as_secs() > 0 {
                self.state.progress = 
                    (elapsed.as_secs_f64() / total_duration.as_secs_f64()) as f32;
            }
        }
    }

    fn check_test_completion(&mut self) {
        if let Some(end_time) = self.state.estimated_end_time {
            if Instant::now() >= end_time {
                // 测试完成
                self.state.is_simulating = false;
            }
        }
    }

    fn update_test_statistics(&mut self, result: &LoadTestResult) {
        self.test_statistics.total_tests += 1;
        
        if result.status == TestStatus::Completed {
            self.test_statistics.successful_tests += 1;
        } else {
            self.test_statistics.failed_tests += 1;
        }

        // 更新平均测试时间
        let test_time = result.end_time.duration_since(result.start_time).as_secs_f64();
        let total_tests = self.test_statistics.total_tests as f64;
        self.test_statistics.average_test_time_s = 
            (self.test_statistics.average_test_time_s * (total_tests - 1.0) + test_time) / total_tests;

        // 更新最佳/最差性能
        self.update_performance_extremes(&result.metrics);
    }

    fn update_performance_extremes(&mut self, metrics: &LoadTestMetrics) {
        // 更新最佳性能
        if self.test_statistics.best_performance.is_none() || 
           self.is_better_load_performance(metrics, self.test_statistics.best_performance.as_ref().unwrap()) {
            self.test_statistics.best_performance = Some(metrics.clone());
        }

        // 更新最差性能
        if self.test_statistics.worst_performance.is_none() || 
           self.is_worse_load_performance(metrics, self.test_statistics.worst_performance.as_ref().unwrap()) {
            self.test_statistics.worst_performance = Some(metrics.clone());
        }
    }

    fn is_better_load_performance(&self, current: &LoadTestMetrics, best: &LoadTestMetrics) -> bool {
        // 简化的性能比较：延迟越低、吞吐量越高、错误率越低越好
        current.average_latency_ms < best.average_latency_ms ||
        current.throughput_rps > best.throughput_rps ||
        current.error_rate < best.error_rate
    }

    fn is_worse_load_performance(&self, current: &LoadTestMetrics, worst: &LoadTestMetrics) -> bool {
        // 简化的性能比较
        current.average_latency_ms > worst.average_latency_ms ||
        current.throughput_rps < worst.throughput_rps ||
        current.error_rate > worst.error_rate
    }
}

// 故障模拟器实现
impl FailureSimulator {
    fn new() -> Self {
        Self {
            state: SimulationState::default(),
            current_failure_scenario: None,
            predefined_failures: HashMap::new(),
            failure_history: VecDeque::with_capacity(100),
            max_history_length: 100,
            failure_injector: FailureInjector {
                injection_method: InjectionMethod::Simulation,
                injection_targets: Vec::new(),
                injection_state: HashMap::new(),
            },
        }
    }

    fn initialize(&mut self) {
        // 初始化故障模拟器
    }

    fn add_predefined_failure(&mut self, scenario: FailureScenario) {
        self.predefined_failures.insert(scenario.id.clone(), scenario.failure_type);
    }

    fn add_custom_scenario(&mut self, _scenario: FailureScenario) {
        // 添加自定义故障场景
    }

    fn start_failure(&mut self, scenario: FailureScenario) -> Result<u64, String> {
        let execution_id = rand::random();
        
        // 设置当前故障场景
        self.current_failure_scenario = Some(scenario.clone());
        
        // 设置模拟状态
        self.state = SimulationState {
            is_simulating: true,
            simulation_time: Instant::now(),
            progress: 0.0,
            current_scenario: Some(scenario.id.clone()),
            active_parameters: SimulationParameters::default(),
            start_time: Some(Instant::now()),
            estimated_end_time: Some(Instant::now() + Duration::from_millis(scenario.parameters.duration_ms)),
        };

        // 注入故障
        self.failure_injector.inject_failure(&scenario);

        // 添加到历史
        self.failure_history.push_back(FailureExecution {
            execution_id,
            failure_type: scenario.failure_type,
            start_time: Instant::now(),
            end_time: None,
            status: ExecutionStatus::Running,
            result: None,
        });

        Ok(execution_id)
    }

    fn stop_failure(&mut self) {
        self.state.is_simulating = false;
        self.current_failure_scenario = None;
        
        // 清除故障注入
        self.failure_injector.clear_injections();
    }

    fn update(&mut self, _delta_time: Duration) {
        if !self.state.is_simulating {
            return;
        }

        // 更新故障模拟进度
        self.update_failure_progress();

        // 检查故障是否完成
        self.check_failure_completion();
    }

    fn get_failure_results(&self) -> Vec<FailureExecution> {
        self.failure_history.iter().cloned().collect()
    }

    fn find_scenario(&self, scenario_id: &str) -> Option<FailureScenario> {
        self.predefined_failures.get(scenario_id).and_then(|&failure_type| {
            Some(FailureScenario {
                id: scenario_id.to_string(),
                name: format!("Scenario {}", scenario_id),
                description: format!("Test scenario {}", scenario_id),
                failure_type,
                parameters: FailureParameters {
                    duration_ms: 1000,
                    intensity: 0.5,
                    frequency: 0.0,
                    affected_components: Vec::new(),
                    custom_parameters: HashMap::new(),
                },
                trigger_condition: None,
                recovery_condition: None,
            })
        })
    }

    fn update_failure_progress(&mut self) {
        if let (Some(start_time), Some(end_time)) = 
            (self.state.start_time, self.state.estimated_end_time) {
            let total_duration = end_time.duration_since(start_time);
            let elapsed = start_time.elapsed();
            
            if total_duration.as_millis() > 0 {
                self.state.progress = 
                    (elapsed.as_millis() as f64 / total_duration.as_millis() as f64) as f32;
            }
        }
    }

    fn check_failure_completion(&mut self) {
        if let Some(end_time) = self.state.estimated_end_time {
            if Instant::now() >= end_time {
                // 故障模拟完成
                self.state.is_simulating = false;
                
                // 更新执行记录
                if let Some(ref mut execution) = self.failure_history.back_mut() {
                    execution.end_time = Some(Instant::now());
                    execution.status = ExecutionStatus::Completed;
                    execution.result = Some(ExecutionResult {
                        success: true,
                        execution_time_s: execution.end_time.unwrap().duration_since(execution.start_time).as_secs_f64(),
                        error_message: None,
                        performance_metrics: HashMap::new(),
                    });
                }
            }
        }
    }

    fn is_simulating(&self) -> bool {
        self.state.is_simulating
    }
}

// 故障注入器实现
impl FailureInjector {
    fn inject_failure(&mut self, scenario: &FailureScenario) {
        // 注入故障
        for target in &scenario.parameters.affected_components {
            self.injection_state.insert(target.clone(), InjectionState {
                injected: true,
                injection_time: Instant::now(),
                injection_parameters: HashMap::new(),
            });
        }
    }

    fn clear_injections(&mut self) {
        self.injection_state.clear();
    }
}

// 恢复测试器实现
impl RecoveryTester {
    fn new() -> Self {
        Self {
            state: SimulationState::default(),
            current_test_config: RecoveryTestConfig::default(),
            test_history: VecDeque::with_capacity(100),
            max_history_length: 100,
            recovery_strategies: HashMap::new(),
        }
    }

    fn initialize(&mut self) {
        // 初始化恢复测试器
    }

    fn add_predefined_strategy(&mut self, strategy: RecoveryStrategy) {
        self.recovery_strategies.insert(strategy.id.clone(), strategy);
    }

    fn add_custom_strategy(&mut self, strategy: RecoveryStrategy) {
        self.recovery_strategies.insert(strategy.id.clone(), strategy);
    }

    fn start_test(&mut self, test_config: RecoveryTestConfig) -> Result<u64, String> {
        let test_id = rand::random();
        
        // 设置测试配置
        self.current_test_config = test_config.clone();
        
        // 设置模拟状态
        self.state = SimulationState {
            is_simulating: true,
            simulation_time: Instant::now(),
            progress: 0.0,
            current_scenario: Some("recovery_test".to_string()),
            active_parameters: SimulationParameters::default(),
            start_time: Some(Instant::now()),
            estimated_end_time: Some(Instant::now() + Duration::from_secs(test_config.recovery_timeout_s)),
        };

        // 模拟恢复测试执行
        self.simulate_recovery_test(test_id);

        Ok(test_id)
    }

    fn stop_test(&mut self) {
        self.state.is_simulating = false;
    }

    fn update(&mut self, _delta_time: Duration) {
        if !self.state.is_simulating {
            return;
        }

        // 更新测试进度
        self.update_test_progress();

        // 检查测试是否完成
        self.check_test_completion();
    }

    fn get_test_results(&self) -> Vec<RecoveryTestResult> {
        self.test_history.iter().cloned().collect()
    }

    fn simulate_recovery_test(&mut self, test_id: u64) {
        // 模拟恢复测试执行
        let test_result = RecoveryTestResult {
            test_id,
            test_type: self.current_test_config.test_type,
            failure_type: self.current_test_config.failure_type,
            start_time: self.state.start_time.unwrap_or_else(Instant::now),
            end_time: self.state.estimated_end_time.unwrap_or_else(|| Instant::now() + Duration::from_secs(30)),
            status: TestStatus::Completed,
            recovery_metrics: RecoveryMetrics {
                recovery_time_s: 15.0,
                recovery_success_rate: 0.8,
                average_retry_count: 3.0,
                data_loss_amount: 100,
                service_downtime_s: 20.0,
            },
            error_message: None,
        };

        self.test_history.push_back(test_result);

        // 限制历史长度
        while self.test_history.len() > self.max_history_length {
            self.test_history.pop_front();
        }
    }

    fn update_test_progress(&mut self) {
        if let (Some(start_time), Some(end_time)) = 
            (self.state.start_time, self.state.estimated_end_time) {
            let total_duration = end_time.duration_since(start_time);
            let elapsed = start_time.elapsed();
            
            if total_duration.as_secs() > 0 {
                self.state.progress = 
                    (elapsed.as_secs_f64() / total_duration.as_secs_f64()) as f32;
            }
        }
    }

    fn check_test_completion(&mut self) {
        if let Some(end_time) = self.state.estimated_end_time {
            if Instant::now() >= end_time {
                // 测试完成
                self.state.is_simulating = false;
            }
        }
    }
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            default_parameters: SimulationParameters::default(),
            max_concurrent_simulations: 5,
            update_interval_ms: 100,
            enable_realtime_simulation: true,
            save_simulation_results: true,
            results_save_path: Some("./simulation_results".to_string()),
            enable_detailed_logging: false,
        }
    }
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            latency: LatencyParameters {
                base_latency_ms: 50.0,
                latency_variation_ms: 10.0,
                distribution_type: LatencyDistribution::Normal,
                enable_burst_latency: false,
                burst_probability: 0.05,
                burst_multiplier: 2.0,
            },
            packet_loss: PacketLossParameters {
                base_loss_rate: 0.01,
                burst_probability: 0.01,
                burst_loss_rate: 0.1,
                burst_duration_ms: 1000,
                loss_pattern: PacketLossPattern::Random,
            },
            bandwidth: BandwidthParameters {
                upload_bandwidth_kbps: 5000.0,
                download_bandwidth_kbps: 5000.0,
                bandwidth_fluctuation: 0.2,
                fluctuation_period_s: 10,
                enable_throttling: false,
                throttling_strategy: ThrottlingStrategy::Fixed,
            },
            jitter: JitterParameters {
                jitter_amplitude_ms: 5.0,
                jitter_frequency_hz: 1.0,
                jitter_distribution: JitterDistribution::Normal,
                enable_random_jitter: false,
                random_probability: 0.1,
            },
            congestion: CongestionParameters {
                congestion_level: 0.0,
                congestion_duration_ms: 0,
                congestion_interval_ms: 0,
                congestion_pattern: CongestionPattern::Constant,
            },
        }
    }
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            is_simulating: false,
            simulation_time: Instant::now(),
            progress: 0.0,
            current_scenario: None,
            active_parameters: SimulationParameters::default(),
            start_time: None,
            estimated_end_time: None,
        }
    }
}

impl Default for NetworkCondition {
    fn default() -> Self {
        Self {
            current_latency_ms: 50.0,
            current_packet_loss_rate: 0.01,
            current_bandwidth_utilization: 0.5,
            current_jitter_ms: 5.0,
            congestion_level: 0.0,
            quality_score: 80.0,
            last_update: Instant::now(),
        }
    }
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            test_type: LoadTestType::Comprehensive,
            target_address: "127.0.0.1:8080".to_string(),
            concurrent_connections: 10,
            duration_s: 60,
            request_interval_ms: 100,
            packet_size: 1024,
            test_mode: TestMode::Constant,
            enable_ramp_up: false,
            ramp_up_time_s: 10,
        }
    }
}

impl Default for RecoveryTestConfig {
    fn default() -> Self {
        Self {
            test_type: RecoveryTestType::Comprehensive,
            failure_type: FailureType::ConnectionInterruption,
            failure_parameters: FailureParameters {
                duration_ms: 5000,
                intensity: 1.0,
                frequency: 0.1,
                affected_components: vec!["network".to_string()],
                custom_parameters: HashMap::new(),
            },
            recovery_strategy: "auto_recovery".to_string(),
            recovery_timeout_s: 30,
            test_rounds: 3,
            enable_auto_recovery: true,
        }
    }
}

impl Default for SimulationMetrics {
    fn default() -> Self {
        Self {
            average_latency_ms: 0.0,
            latency_distribution: LatencyDistributionStats {
                min_latency_ms: 0.0,
                max_latency_ms: 0.0,
                average_latency_ms: 0.0,
                median_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
            },
            packet_loss_rate: 0.0,
            bandwidth_utilization: 0.0,
            throughput_rps: 0.0,
            error_rate: 0.0,
            quality_score: 100.0,
            custom_metrics: HashMap::new(),
        }
    }
}

impl Default for NetworkSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_simulator_creation() {
        let simulator = NetworkSimulator::new();
        assert!(simulator.is_enabled());
        assert!(!simulator.is_active());
    }

    #[test]
    fn test_condition_simulation() {
        let mut simulator = NetworkSimulator::new();
        simulator.initialize();
        
        // 开始网络状况模拟
        let result = simulator.start_condition_simulation("perfect_network");
        assert!(result.is_ok());
        assert!(simulator.is_active());
        
        // 停止模拟
        let _ = simulator.stop_simulation();
        assert!(!simulator.is_active());
    }

    #[test]
    fn test_load_test() {
        let mut simulator = NetworkSimulator::new();
        simulator.initialize();
        
        let test_config = LoadTestConfig {
            test_type: LoadTestType::Latency,
            target_address: "127.0.0.1:8080".to_string(),
            concurrent_connections: 5,
            duration_s: 10,
            request_interval_ms: 100,
            packet_size: 512,
            test_mode: TestMode::Constant,
            enable_ramp_up: false,
            ramp_up_time_s: 5,
        };
        
        // 开始负载测试
        let result = simulator.start_load_test(test_config);
        assert!(result.is_ok());
        assert!(simulator.is_active());
        
        // 停止测试
        let _ = simulator.stop_simulation();
        assert!(!simulator.is_active());
    }

    #[test]
    fn test_failure_simulation() {
        let mut simulator = NetworkSimulator::new();
        simulator.initialize();
        
        // 开始故障模拟
        let result = simulator.start_failure_simulation("connection_interruption");
        assert!(result.is_ok());
        assert!(simulator.is_active());
        
        // 停止模拟
        let _ = simulator.stop_simulation();
        assert!(!simulator.is_active());
    }

    #[test]
    fn test_recovery_test() {
        let mut simulator = NetworkSimulator::new();
        simulator.initialize();
        
        let test_config = RecoveryTestConfig {
            test_type: RecoveryTestType::ConnectionRecovery,
            failure_type: FailureType::ConnectionInterruption,
            failure_parameters: FailureParameters {
                duration_ms: 1000,
                intensity: 0.5,
                frequency: 0.1,
                affected_components: vec!["network".to_string()],
                custom_parameters: HashMap::new(),
            },
            recovery_strategy: "auto_recovery".to_string(),
            recovery_timeout_s: 15,
            test_rounds: 1,
            enable_auto_recovery: true,
        };
        
        // 开始恢复测试
        let result = simulator.start_recovery_test(test_config);
        assert!(result.is_ok());
        assert!(simulator.is_active());
        
        // 停止测试
        let _ = simulator.stop_simulation();
        assert!(!simulator.is_active());
    }

    #[test]
    fn test_simulation_statistics() {
        let mut simulator = NetworkSimulator::new();
        simulator.initialize();
        
        // 执行一些测试以生成统计数据
        let _ = simulator.start_condition_simulation("perfect_network");
        let _ = simulator.stop_simulation();
        
        let _ = simulator.start_load_test(LoadTestConfig::default());
        let _ = simulator.stop_simulation();
        
        let stats = simulator.get_statistics();
        assert_eq!(stats.total_simulations, 2);
        assert_eq!(stats.successful_simulations, 2);
    }
}