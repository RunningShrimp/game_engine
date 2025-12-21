//  网络调试模块
// 
//  提供全面的网络调试和分析工具，包括性能监控、数据包分析、延迟可视化和网络模拟功能。
// 
//  ## 功能特性
// 
//  - 实时网络性能监控
//  - 数据包捕获和分析
//  - 延迟可视化
//  - 网络调试界面
//  - 网络状况模拟
// 
//  ## 架构设计
// 
//  ```text
//  ┌─────────────────────────────────────────┐
//  │           Network Debugging             │
//  ├─────────────────────────────────────────┤
//  │  ┌──────────┐  ┌──────────┐  ┌─────────┐│
//  │  │Performance│  │ Packet   │ │ Latency  ││
//  │  │ Monitor   │  │ Analyzer │ │ Visualizer││
//  │  └────┬─────┘  └────┬─────┘  └────┬────┘│
//  │       │             │             │     │
//  │       └─────────────┼─────────────┘     │
//  │                     │                   │
//  │              ┌──────▼──────┐            │
//  │              │   Debug    │            │
//  │              │   Interface│            │
//  │              └──────┬──────┘            │
//  │                     │                   │
//  │              ┌──────▼──────┐            │
//  │              │  Network   │            │
//  │              │ Simulator  │            │
//  │              └─────────────┘            │
//  └─────────────────────────────────────────┘
//  ```

pub mod performance_monitor;
pub mod packet_analyzer;
pub mod latency_visualizer;
pub mod debug_interface;
pub mod network_simulator;

use bevy_ecs::prelude::*;
use crate::ecs::Time;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 网络调试管理器
#[derive(Resource)]
pub struct NetworkDebugManager {
    /// 性能监控器
    pub performance_monitor: Arc<Mutex<performance_monitor::NetworkPerformanceMonitor>>,
    /// 数据包分析器
    pub packet_analyzer: Arc<Mutex<packet_analyzer::NetworkPacketAnalyzer>>,
    /// 延迟可视化器
    pub latency_visualizer: Arc<Mutex<latency_visualizer::LatencyVisualizer>>,
    /// 调试界面
    pub debug_interface: Arc<Mutex<debug_interface::NetworkDebugInterface>>,
    /// 网络模拟器
    pub network_simulator: Arc<Mutex<network_simulator::NetworkSimulator>>,
    /// 是否启用调试
    pub enabled: bool,
    /// 最后更新时间
    pub last_update: Instant,
}

impl NetworkDebugManager {
    /// 创建新的网络调试管理器
    pub fn new() -> Self {
        Self {
            performance_monitor: Arc::new(Mutex::new(performance_monitor::NetworkPerformanceMonitor::new())),
            packet_analyzer: Arc::new(Mutex::new(packet_analyzer::NetworkPacketAnalyzer::new())),
            latency_visualizer: Arc::new(Mutex::new(latency_visualizer::LatencyVisualizer::new())),
            debug_interface: Arc::new(Mutex::new(debug_interface::NetworkDebugInterface::new())),
            network_simulator: Arc::new(Mutex::new(network_simulator::NetworkSimulator::new())),
            enabled: true,
            last_update: Instant::now(),
        }
    }

    /// 更新所有调试组件
    pub fn update(&mut self, delta_time: Duration) {
        if !self.enabled {
            return;
        }

        let current_time = Instant::now();
        
        // 更新性能监控器
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.update(delta_time);
        }

        // 更新延迟可视化器
        if let Ok(mut visualizer) = self.latency_visualizer.lock() {
            visualizer.update(delta_time);
        }

        // 更新网络模拟器
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.update(delta_time);
        }

        self.last_update = current_time;
    }

    /// 启用/禁用调试
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        
        // 同时更新所有子组件
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.set_enabled(enabled);
        }
        
        if let Ok(mut analyzer) = self.packet_analyzer.lock() {
            analyzer.set_enabled(enabled);
        }
        
        if let Ok(mut visualizer) = self.latency_visualizer.lock() {
            visualizer.set_enabled(enabled);
        }
        
        if let Ok(mut interface) = self.debug_interface.lock() {
            interface.set_enabled(enabled);
        }
        
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.set_enabled(enabled);
        }
    }

    /// 获取调试状态摘要
    pub fn get_status_summary(&self) -> DebugStatusSummary {
        DebugStatusSummary {
            enabled: self.enabled,
            last_update: self.last_update,
            performance_monitoring: self.performance_monitor.lock().map(|m| m.is_active()).unwrap_or(false),
            packet_analysis: self.packet_analyzer.lock().map(|a| a.is_active()).unwrap_or(false),
            latency_visualization: self.latency_visualizer.lock().map(|v| v.is_active()).unwrap_or(false),
            network_simulation: self.network_simulator.lock().map(|s| s.is_active()).unwrap_or(false),
        }
    }

    /// 获取网络模拟器的可变引用
    pub fn get_network_simulator_mut(&mut self) -> Arc<Mutex<network_simulator::NetworkSimulator>> {
        self.network_simulator.clone()
    }

    /// 开始网络状况模拟
    pub fn start_condition_simulation(&mut self, scenario_id: &str) -> Result<(), String> {
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.start_condition_simulation(scenario_id)
        } else {
            Err("无法访问网络模拟器".to_string())
        }
    }

    /// 开始负载测试
    pub fn start_load_test(&mut self, test_config: network_simulator::LoadTestConfig) -> Result<u64, String> {
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.start_load_test(test_config)
        } else {
            Err("无法访问网络模拟器".to_string())
        }
    }

    /// 开始故障模拟
    pub fn start_failure_simulation(&mut self, scenario_id: &str) -> Result<u64, String> {
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.start_failure_simulation(scenario_id)
        } else {
            Err("无法访问网络模拟器".to_string())
        }
    }

    /// 开始恢复测试
    pub fn start_recovery_test(&mut self, test_config: network_simulator::RecoveryTestConfig) -> Result<u64, String> {
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.start_recovery_test(test_config)
        } else {
            Err("无法访问网络模拟器".to_string())
        }
    }

    /// 停止网络模拟
    pub fn stop_simulation(&mut self) -> Result<(), String> {
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.stop_simulation()
        } else {
            Err("无法访问网络模拟器".to_string())
        }
    }

    /// 获取模拟结果
    pub fn get_simulation_results(&self) -> Vec<network_simulator::SimulationResult> {
        if let Ok(simulator) = self.network_simulator.lock() {
            simulator.get_simulation_results()
        } else {
            Vec::new()
        }
    }

    /// 获取模拟统计
    pub fn get_simulation_statistics(&self) -> network_simulator::SimulationStatistics {
        if let Ok(simulator) = self.network_simulator.lock() {
            simulator.get_statistics().clone()
        } else {
            network_simulator::SimulationStatistics::default()
        }
    }

    /// 获取当前网络状况
    pub fn get_current_network_condition(&self) -> Option<network_simulator::NetworkCondition> {
        if let Ok(simulator) = self.network_simulator.lock() {
            simulator.get_current_condition().cloned()
        } else {
            None
        }
    }

    /// 添加自定义场景
    pub fn add_custom_scenario(&mut self, scenario: network_simulator::SimulationScenario) {
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.add_custom_scenario(scenario);
        }
    }

    /// 添加自定义故障场景
    pub fn add_custom_failure_scenario(&mut self, scenario: network_simulator::FailureScenario) {
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.add_custom_failure_scenario(scenario);
        }
    }

    /// 添加自定义恢复策略
    pub fn add_custom_recovery_strategy(&mut self, strategy: network_simulator::RecoveryStrategy) {
        if let Ok(mut simulator) = self.network_simulator.lock() {
            simulator.add_custom_recovery_strategy(strategy);
        }
    }
}

/// 调试状态摘要
#[derive(Debug, Clone)]
pub struct DebugStatusSummary {
    /// 是否启用
    pub enabled: bool,
    /// 最后更新时间
    pub last_update: Instant,
    /// 性能监控状态
    pub performance_monitoring: bool,
    /// 数据包分析状态
    pub packet_analysis: bool,
    /// 延迟可视化状态
    pub latency_visualization: bool,
    /// 网络模拟状态
    pub network_simulation: bool,
}

impl Default for NetworkDebugManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 网络调试系统
pub fn network_debug_system(
    time: Res<Time>,
    mut debug_manager: ResMut<NetworkDebugManager>,
) {
    debug_manager.update(Duration::from_secs_f64(time.delta_seconds_f64()));
}

/// 初始化网络调试系统
pub fn initialize_network_debug(mut commands: Commands) {
    commands.insert_resource(NetworkDebugManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_manager_creation() {
        let manager = NetworkDebugManager::new();
        assert!(manager.enabled);
        assert!(manager.performance_monitor.lock().is_ok());
        assert!(manager.packet_analyzer.lock().is_ok());
        assert!(manager.latency_visualizer.lock().is_ok());
        assert!(manager.debug_interface.lock().is_ok());
        assert!(manager.network_simulator.lock().is_ok());
    }

    #[test]
    fn test_debug_status_summary() {
        let manager = NetworkDebugManager::new();
        let summary = manager.get_status_summary();
        assert!(summary.enabled);
        assert!(summary.performance_monitoring);
        assert!(summary.packet_analysis);
        assert!(summary.latency_visualization);
        assert!(summary.network_simulation);
    }

    #[test]
    fn test_enable_disable() {
        let mut manager = NetworkDebugManager::new();
        
        // 禁用
        manager.set_enabled(false);
        assert!(!manager.enabled);
        
        // 启用
        manager.set_enabled(true);
        assert!(manager.enabled);
    }

    #[test]
    fn test_network_simulation() {
        let mut manager = NetworkDebugManager::new();
        
        // 测试网络状况模拟
        let result = manager.start_condition_simulation("perfect_network");
        assert!(result.is_ok());
        
        // 停止模拟
        let _ = manager.stop_simulation();
        
        // 测试负载测试
        let test_config = network_simulator::LoadTestConfig::default();
        let test_id = manager.start_load_test(test_config);
        assert!(test_id.is_ok());
        
        // 停止模拟
        let _ = manager.stop_simulation();
        
        // 测试故障模拟
        let result = manager.start_failure_simulation("connection_interruption");
        assert!(result.is_ok());
        
        // 停止模拟
        let _ = manager.stop_simulation();
        
        // 测试恢复测试
        let test_config = network_simulator::RecoveryTestConfig::default();
        let test_id = manager.start_recovery_test(test_config);
        assert!(test_id.is_ok());
        
        // 停止模拟
        let _ = manager.stop_simulation();
    }

    #[test]
    fn test_custom_scenarios() {
        let mut manager = NetworkDebugManager::new();
        
        // 添加自定义场景
        let scenario = network_simulator::SimulationScenario {
            id: "test_scenario".to_string(),
            name: "测试场景".to_string(),
            description: "测试用的自定义场景".to_string(),
            scenario_type: network_simulator::ScenarioType::Custom,
            parameters: network_simulator::SimulationParameters::default(),
            duration_s: 60,
            loop_scenario: false,
            phases: Vec::new(),
        };
        manager.add_custom_scenario(scenario);
        
        // 添加自定义故障场景
        let failure_scenario = network_simulator::FailureScenario {
            id: "test_failure".to_string(),
            name: "测试故障".to_string(),
            description: "测试用的自定义故障".to_string(),
            failure_type: network_simulator::FailureType::Custom,
            parameters: network_simulator::FailureParameters {
                duration_ms: 5000,
                intensity: 0.5,
                frequency: 0.1,
                affected_components: vec!["test".to_string()],
                custom_parameters: HashMap::new(),
            },
            trigger_condition: None,
            recovery_condition: None,
        };
        manager.add_custom_failure_scenario(failure_scenario);
        
        // 添加自定义恢复策略
        let recovery_strategy = network_simulator::RecoveryStrategy {
            id: "test_recovery".to_string(),
            name: "测试恢复".to_string(),
            description: "测试用的自定义恢复策略".to_string(),
            recovery_steps: Vec::new(),
            estimated_recovery_time_s: 30,
            success_rate: 0.8,
        };
        manager.add_custom_recovery_strategy(recovery_strategy);
    }

    #[test]
    fn test_simulation_data_access() {
        let manager = NetworkDebugManager::new();
        
        // 获取模拟结果
        let results = manager.get_simulation_results();
        assert!(results.is_empty()); // 初始状态下应该为空
        
        // 获取模拟统计
        let stats = manager.get_simulation_statistics();
        assert_eq!(stats.total_simulations, 0); // 初始状态下应该为0
        
        // 获取当前网络状况
        let condition = manager.get_current_network_condition();
        assert!(condition.is_none()); // 初始状态下应该为None
    }
}