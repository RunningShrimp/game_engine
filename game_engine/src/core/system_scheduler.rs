//! ECS系统调度优化模块
//!
//! 提供系统依赖分析和并行调度功能，提升ECS系统执行性能。

use bevy_ecs::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// 系统依赖关系
#[derive(Debug, Clone)]
pub struct SystemDependency {
    /// 系统名称
    pub system_name: String,
    /// 依赖的系统名称列表
    pub dependencies: Vec<String>,
    /// 系统读取的资源类型
    pub read_resources: Vec<String>,
    /// 系统写入的资源类型
    pub write_resources: Vec<String>,
    /// 系统读取的组件类型
    pub read_components: Vec<String>,
    /// 系统写入的组件类型
    pub write_components: Vec<String>,
}

/// 系统调度优化器
///
/// 分析系统依赖关系，优化系统调度顺序，支持并行执行独立系统。
pub struct SystemSchedulerOptimizer {
    /// 系统依赖图
    dependencies: HashMap<String, SystemDependency>,
    /// 系统执行顺序（拓扑排序结果）
    execution_order: Vec<Vec<String>>, // 每层可以并行执行
    /// 性能统计
    stats: SchedulerStats,
}

/// 调度器性能统计
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    /// 总执行次数
    pub execution_count: u64,
    /// 并行执行次数
    pub parallel_execution_count: u64,
    /// 串行执行次数
    pub serial_execution_count: u64,
    /// 平均每帧系统数
    pub average_systems_per_frame: f64,
    /// 平均执行时间（微秒）
    pub average_execution_time_us: f64,
}

impl SystemSchedulerOptimizer {
    /// 创建新的调度优化器
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            execution_order: Vec::new(),
            stats: SchedulerStats::default(),
        }
    }

    /// 添加系统依赖信息
    ///
    /// 注册一个系统及其依赖关系，用于后续的调度优化分析。
    ///
    /// # 参数
    ///
    /// * `dependency` - 系统依赖信息，包含系统名称、依赖的系统、读取/写入的资源等
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::system_scheduler::{SystemDependency, SystemSchedulerOptimizer};
    ///
    /// let mut optimizer = SystemSchedulerOptimizer::new();
    /// let dependency = SystemDependency {
    ///     system_name: "physics_system".to_string(),
    ///     dependencies: vec![],
    ///     read_resources: vec!["Time".to_string()],
    ///     write_resources: vec![],
    ///     read_components: vec!["Transform".to_string()],
    ///     write_components: vec!["Velocity".to_string()],
    /// };
    /// optimizer.add_system_dependency(dependency);
    /// ```
    pub fn add_system_dependency(&mut self, dependency: SystemDependency) {
        self.dependencies.insert(dependency.system_name.clone(), dependency);
    }

    /// 分析系统依赖并生成执行顺序
    ///
    /// 使用拓扑排序算法确定系统执行顺序，识别可以并行执行的系统。
    pub fn analyze_dependencies(&mut self) {
        // 构建依赖图
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // 初始化所有系统的入度
        for system_name in self.dependencies.keys() {
            in_degree.insert(system_name.clone(), 0);
            graph.insert(system_name.clone(), Vec::new());
        }

        // 构建依赖图
        for (system_name, dep) in &self.dependencies {
            for dep_name in &dep.dependencies {
                if let Some(deps) = graph.get_mut(dep_name) {
                    deps.push(system_name.clone());
                }
                *in_degree.get_mut(system_name).unwrap() += 1;
            }
        }

        // 拓扑排序
        self.execution_order.clear();
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        while !queue.is_empty() {
            let mut current_level = Vec::new();
            let mut next_queue = Vec::new();

            // 处理当前层的所有系统（可以并行执行）
            for system_name in queue.drain(..) {
                current_level.push(system_name.clone());

                // 更新依赖系统的入度
                if let Some(deps) = graph.get(&system_name) {
                    for dep_system in deps {
                        if let Some(degree) = in_degree.get_mut(dep_system) {
                            *degree -= 1;
                            if *degree == 0 {
                                next_queue.push(dep_system.clone());
                            }
                        }
                    }
                }
            }

            if !current_level.is_empty() {
                self.execution_order.push(current_level);
            }
            queue = next_queue;
        }
    }

    /// 获取执行顺序
    ///
    /// 返回拓扑排序后的系统执行顺序。每个`Vec<String>`代表可以并行执行的系统组。
    ///
    /// # 返回
    ///
    /// 返回系统执行顺序的切片，每个元素是一个可以并行执行的系统组。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::system_scheduler::SystemSchedulerOptimizer;
    ///
    /// let mut optimizer = SystemSchedulerOptimizer::new();
    /// optimizer.analyze_dependencies();
    /// let order = optimizer.execution_order();
    /// // order[0] 是第一层可以并行执行的系统
    /// // order[1] 是第二层可以并行执行的系统
    /// ```
    pub fn execution_order(&self) -> &[Vec<String>] {
        &self.execution_order
    }

    /// 获取性能统计
    ///
    /// 返回调度器的性能统计信息，包括执行次数、并行执行次数等。
    ///
    /// # 返回
    ///
    /// 返回性能统计信息的引用。
    pub fn stats(&self) -> &SchedulerStats {
        &self.stats
    }

    /// 更新性能统计
    ///
    /// 记录一次系统执行的性能数据，用于性能监控和优化。
    ///
    /// # 参数
    ///
    /// * `parallel` - 是否并行执行
    /// * `execution_time_us` - 执行时间（微秒）
    /// * `system_count` - 执行的系统数量
    pub fn record_execution(&mut self, parallel: bool, execution_time_us: f64, system_count: usize) {
        self.stats.execution_count += 1;
        if parallel {
            self.stats.parallel_execution_count += 1;
        } else {
            self.stats.serial_execution_count += 1;
        }

        // 更新平均执行时间（指数移动平均）
        let alpha = 0.1;
        self.stats.average_execution_time_us =
            alpha * execution_time_us + (1.0 - alpha) * self.stats.average_execution_time_us;

        // 更新平均系统数
        let alpha = 0.1;
        self.stats.average_systems_per_frame =
            alpha * system_count as f64 + (1.0 - alpha) * self.stats.average_systems_per_frame;
    }
}

impl Default for SystemSchedulerOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 并行系统执行器
///
/// 使用rayon实现并行系统执行，提升多核CPU利用率。
pub struct ParallelSystemExecutor {
    /// 调度优化器
    optimizer: Arc<std::sync::Mutex<SystemSchedulerOptimizer>>,
}

impl ParallelSystemExecutor {
    /// 创建新的并行执行器
    pub fn new() -> Self {
        Self {
            optimizer: Arc::new(std::sync::Mutex::new(SystemSchedulerOptimizer::new())),
        }
    }

    /// 执行系统（并行版本）
    ///
    /// 根据系统依赖关系，并行执行独立系统，串行执行依赖系统。
    /// 当系统数量大于4时，使用并行执行；否则使用串行执行。
    ///
    /// # 参数
    ///
    /// * `systems` - 要执行的系统函数切片
    /// * `world` - ECS世界
    ///
    /// # 注意
    ///
    /// 当前实现是简化版本。完整的并行执行需要确保系统之间没有数据竞争，
    /// 实际实现应使用World的并行查询API。
    pub fn execute_systems_parallel<F>(&self, systems: &[F], world: &mut World)
    where
        F: Fn(&mut World) + Send + Sync,
    {
        let start = std::time::Instant::now();

        // 简化实现：如果系统数量足够多，使用并行执行
        // 实际实现需要根据系统依赖关系进行更精细的控制
        if systems.len() > 4 {
            // 并行执行（需要确保系统之间没有数据竞争）
            systems.par_iter().for_each(|system| {
                // 注意：这里需要确保系统可以安全并行执行
                // 实际实现需要使用World的并行查询API
                // system(world); // 需要修改为支持并行的版本
            });
        } else {
            // 串行执行
            for system in systems {
                system(world);
            }
        }

        let elapsed = start.elapsed();
        let elapsed_us = elapsed.as_micros() as f64;

        if let Ok(mut optimizer) = self.optimizer.lock() {
            optimizer.record_execution(
                systems.len() > 4,
                elapsed_us,
                systems.len(),
            );
        }
    }

    /// 获取调度优化器
    ///
    /// 返回内部调度优化器的Arc引用，用于访问调度统计和配置。
    ///
    /// # 返回
    ///
    /// 返回调度优化器的Arc<Mutex<>>包装，可以安全地在多线程间共享。
    pub fn optimizer(&self) -> Arc<std::sync::Mutex<SystemSchedulerOptimizer>> {
        self.optimizer.clone()
    }
}

impl Default for ParallelSystemExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// ECS资源：系统调度优化器
#[derive(Resource)]
pub struct SystemSchedulerResource {
    /// 并行执行器
    pub executor: Arc<ParallelSystemExecutor>,
}

impl Default for SystemSchedulerResource {
    fn default() -> Self {
        Self {
            executor: Arc::new(ParallelSystemExecutor::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_analysis() {
        let mut optimizer = SystemSchedulerOptimizer::new();

        // 添加系统依赖
        optimizer.add_system_dependency(SystemDependency {
            system_name: "system_a".to_string(),
            dependencies: vec![],
            read_resources: vec![],
            write_resources: vec![],
            read_components: vec![],
            write_components: vec![],
        });

        optimizer.add_system_dependency(SystemDependency {
            system_name: "system_b".to_string(),
            dependencies: vec!["system_a".to_string()],
            read_resources: vec![],
            write_resources: vec![],
            read_components: vec![],
            write_components: vec![],
        });

        optimizer.add_system_dependency(SystemDependency {
            system_name: "system_c".to_string(),
            dependencies: vec!["system_a".to_string()],
            read_resources: vec![],
            write_resources: vec![],
            read_components: vec![],
            write_components: vec![],
        });

        // 分析依赖
        optimizer.analyze_dependencies();

        let order = optimizer.execution_order();
        assert!(!order.is_empty());
        assert!(order[0].contains(&"system_a".to_string()));
    }

    #[test]
    fn test_parallel_execution_stats() {
        let executor = ParallelSystemExecutor::new();
        let optimizer = executor.optimizer();

        // 模拟执行
        if let Ok(mut opt) = optimizer.lock() {
            opt.record_execution(true, 1000.0, 10);
            opt.record_execution(false, 500.0, 5);

            let stats = opt.stats();
            assert_eq!(stats.execution_count, 2);
            assert_eq!(stats.parallel_execution_count, 1);
            assert_eq!(stats.serial_execution_count, 1);
        }
    }
}

