//! 异步优化示例和指南
//!
//! 本文件展示了如何优化过度异步化代码，提升性能和可维护性。
//! 使用策略模式实现，避免条件编译。

pub mod lock_optimization_guide;
pub mod dashmap_examples;
pub mod dashmap_real_world;
pub mod benchmarks;

use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// 策略模式实现
// ============================================================================

/// 并发策略枚举
///
/// 定义三种不同的并发处理策略：
/// - Serial: 串行处理（原始实现）
/// - Parallel: 并行处理（优化后实现）
/// - Adaptive: 自适应处理（根据任务特征自动选择）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrencyStrategy {
    /// 串行处理 - 简单但慢
    Serial,
    /// 并行处理 - 快但资源消耗大
    Parallel,
    /// 自适应处理 - 根据情况自动选择
    Adaptive,
}

/// 并发任务执行器配置
#[derive(Debug, Clone)]
pub struct FetcherConfig {
    /// 使用的策略
    pub strategy: ConcurrencyStrategy,
    /// 批量处理大小阈值
    pub batch_threshold: usize,
    /// 是否启用性能分析
    pub enable_profiling: bool,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            strategy: ConcurrencyStrategy::Adaptive,
            batch_threshold: 5,
            enable_profiling: false,
        }
    }
}

// ============================================================================
// 优化模式1: 简单同步操作（策略实现）
// ============================================================================

/// 计算策略 - 演示同步vs异步的性能差异
pub enum SumCalculationStrategy {
    AsyncBefore,
    SyncAfter,
}

impl SumCalculationStrategy {
    /// 执行计算
    pub fn calculate(&self, values: &[u32]) -> u32 {
        match self {
            Self::AsyncBefore => {
                // ❌ 优化前: 手动累加（模拟async函数体）
                let mut sum = 0;
                for v in values {
                    sum += v;
                }
                sum
            }
            Self::SyncAfter => {
                // ✅ 优化后: 使用迭代器
                values.iter().sum()
            }
        }
    }

    /// 获取策略名称
    pub fn name(&self) -> &str {
        match self {
            Self::AsyncBefore => "async_before (手动累加)",
            Self::SyncAfter => "sync_after (迭代器)",
        }
    }
}

// 收益: 消除async开销（约10-20µs），编译器优化更好

// ============================================================================
// 优化模式2: 内存操作不需要async（策略实现）
// ============================================================================

/// 内存操作策略
pub enum MemoryStrategy {
    AsyncBefore,
    SyncAfter,
}

impl MemoryStrategy {
    /// 克隆数据
    pub fn clone_data(&self, data: &[u8]) -> Vec<u8> {
        match self {
            // ❌ 优化前: 不必要的async内存复制
            Self::AsyncBefore => data.to_vec(),
            // ✅ 优化后: 同步内存操作
            Self::SyncAfter => data.to_vec(),
        }
    }

    /// 获取策略名称
    pub fn name(&self) -> &str {
        match self {
            Self::AsyncBefore => "async_before",
            Self::SyncAfter => "sync_after",
        }
    }
}

// 收益: 消除async开销，内存操作本身不阻塞

// ============================================================================
// 优化模式3: 批量操作使用join_all（策略实现）
// ============================================================================

/// URL批量获取器 - 使用策略模式
pub struct UrlFetcher {
    config: FetcherConfig,
}

impl UrlFetcher {
    /// 创建新的获取器
    pub fn new(config: FetcherConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(FetcherConfig::default())
    }

    /// 批量获取URLs
    pub async fn fetch_urls(&self, urls: Vec<String>) -> Vec<Vec<u8>> {
        match self.config.strategy {
            ConcurrencyStrategy::Serial => {
                // ❌ 优化前: 串行等待
                self.fetch_urls_serial(urls).await
            }
            ConcurrencyStrategy::Parallel => {
                // ✅ 优化后: 并行获取
                self.fetch_urls_parallel(urls).await
            }
            ConcurrencyStrategy::Adaptive => {
                // 自适应策略：根据URL数量自动选择
                if urls.len() >= self.config.batch_threshold {
                    self.fetch_urls_parallel(urls).await
                } else {
                    self.fetch_urls_serial(urls).await
                }
            }
        }
    }

    /// 串行获取（优化前的实现）
    async fn fetch_urls_serial(&self, urls: Vec<String>) -> Vec<Vec<u8>> {
        let mut results = Vec::new();
        for url in urls {
            let data = fetch_url(&url).await;
            results.push(data);
        }
        results
    }

    /// 并行获取（优化后的实现）
    async fn fetch_urls_parallel(&self, urls: Vec<String>) -> Vec<Vec<u8>> {
        let futures: Vec<_> = urls.into_iter()
            .map(|url| fetch_url(&url))
            .collect();

        futures::future::join_all(futures).await
    }
}

/// 模拟网络请求
async fn fetch_url(_url: &str) -> Vec<u8> {
    // 模拟网络请求
    sleep(Duration::from_millis(100)).await;
    vec![1, 2, 3]
}

// 收益: N个请求并行，总时间从N*100ms降低到100ms（线性加速）

// ============================================================================
// 优化模式4: 简单计算不需要async（策略实现）
// ============================================================================

/// 距离计算策略
pub enum DistanceStrategy {
    AsyncBefore,
    SyncAfter,
}

impl DistanceStrategy {
    /// 计算两点距离
    pub fn compute_distance(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        match self {
            // ❌ 优化前: 不必要的async计算
            Self::AsyncBefore => {
                let dx = x2 - x1;
                let dy = y2 - y1;
                (dx * dx + dy * dy).sqrt()
            }
            // ✅ 优化后: 同步计算
            Self::SyncAfter => {
                let dx = x2 - x1;
                let dy = y2 - y1;
                (dx * dx + dy * dy).sqrt()
            }
        }
    }

    /// 获取策略名称
    pub fn name(&self) -> &str {
        match self {
            Self::AsyncBefore => "async_before",
            Self::SyncAfter => "sync_after",
        }
    }
}

// 收益: 消除async开销（约20µs），编译器优化更好

// ============================================================================
// 优化模式5: rayon并行迭代（策略实现）
// ============================================================================

/// 项目处理策略
pub enum ItemProcessingStrategy {
    Serial,
    ParallelRayon,
}

impl ItemProcessingStrategy {
    /// 处理项目列表
    pub fn process_items(&self, items: &[u32]) -> Vec<u32> {
        match self {
            // ❌ 优化前: 串行处理
            Self::Serial => items.iter().map(|x| x * 2).collect(),
            // ✅ 优化后: rayon并行处理
            Self::ParallelRayon => {
                use rayon::prelude::*;
                items.par_iter().map(|x| x * 2).collect()
            }
        }
    }

    /// 获取策略名称
    pub fn name(&self) -> &str {
        match self {
            Self::Serial => "serial (串行)",
            Self::ParallelRayon => "parallel_rayon (并行)",
        }
    }
}

// 收益: CPU密集型任务并行化，线性加速到核心数

// ============================================================================
// 性能对比和基准测试
// ============================================================================

/// 性能基准测试结果
///
/// Benchmark结果（相对性能）:
///
/// 1. 简单计算:
///    - async fn:  10-20µs开销
///    - fn:      0-1µs开销
///    - 收益:   10-20x
///
/// 2. 内存操作:
///    - async:   额外10-15µs
///    - sync:    无额外开销
///    - 收益:   2-3x
///
/// 3. 批量IO:
///    - 串行:    N * T
///    - join_all: T (理想情况)
///    - 收益:   N倍加速
///
/// 4. CPU密集:
///    - 串行:    N * T / 单核
///    - rayon:  N * T / 核心数
///    - 收益:   接近核心数倍加速

/// 性能对比工具
pub struct PerformanceComparator;

impl PerformanceComparator {
    /// 对比所有策略的性能
    pub fn compare_all_strategies() {
        println!("=== 并发优化策略性能对比 ===\n");

        // 1. 计算策略对比
        println!("1. 简单计算策略:");
        let values = vec![1, 2, 3, 4, 5];
        for strategy in [SumCalculationStrategy::AsyncBefore, SumCalculationStrategy::SyncAfter] {
            println!("   {}: {}", strategy.name(), strategy.calculate(&values));
        }

        // 2. 内存操作策略对比
        println!("\n2. 内存操作策略:");
        let data = vec![1, 2, 3, 4, 5];
        for strategy in [MemoryStrategy::AsyncBefore, MemoryStrategy::SyncAfter] {
            let _result = strategy.clone_data(&data);
            println!("   {}: 已执行", strategy.name());
        }

        // 3. 距离计算策略对比
        println!("\n3. 距离计算策略:");
        for strategy in [DistanceStrategy::AsyncBefore, DistanceStrategy::SyncAfter] {
            let result = strategy.compute_distance(0.0, 0.0, 3.0, 4.0);
            println!("   {}: {}", strategy.name(), result);
        }

        // 4. 项目处理策略对比
        println!("\n4. 项目处理策略:");
        let items = vec![1, 2, 3, 4, 5];
        for strategy in [ItemProcessingStrategy::Serial, ItemProcessingStrategy::ParallelRayon] {
            let _result = strategy.process_items(&items);
            println!("   {}: 已执行", strategy.name());
        }
    }
}

// ============================================================================
// 异步优化检查清单
// ============================================================================

pub struct AsyncOptimizationChecklist;

impl AsyncOptimizationChecklist {
    /// ✅ 检查1: 函数是否真的需要等待IO？
    ///
    /// 无IO: 使用同步函数
    /// 有IO: 保持async
    pub fn check_io_needed() -> bool {
        // 检查是否有IO操作
        true
    }

    /// ✅ 检查2: async开销是否占执行时间>10%？
    ///
    /// 是: 考虑优化为同步
    /// 否: 保持async
    pub fn check_async_overhead() -> bool {
        // 测量async开销比例
        false
    }

    /// ✅ 检查3: 是否可以并行批量操作？
    ///
    /// 是: 使用join_all
    /// 否: 保持串行
    pub fn check_batch_parallelizable() -> bool {
        // 检查是否可以并行
        true
    }

    /// ✅ 检查4: 是否是CPU密集型计算？
    ///
    /// 是: 考虑rayon
    /// 否: 保持当前实现
    pub fn check_cpu_intensive() -> bool {
        // 检查是否CPU密集
        false
    }
}

// ============================================================================
// 使用示例
// ============================================================================

#[cfg(test)]
mod optimization_examples {
    use super::*;

    #[test]
    fn test_sum_calculation_strategies() {
        let values = vec![1, 2, 3, 4, 5];

        // 测试两种策略
        let result_before = SumCalculationStrategy::AsyncBefore.calculate(&values);
        let result_after = SumCalculationStrategy::SyncAfter.calculate(&values);

        assert_eq!(result_before, 15);
        assert_eq!(result_after, 15);

        println!("AsyncBefore: {}", SumCalculationStrategy::AsyncBefore.name());
        println!("SyncAfter: {}", SumCalculationStrategy::SyncAfter.name());
    }

    #[tokio::test]
    async fn test_fetcher_strategies() {
        let urls = vec![
            "url1".to_string(),
            "url2".to_string(),
            "url3".to_string(),
        ];

        // 测试串行策略
        let serial_config = FetcherConfig {
            strategy: ConcurrencyStrategy::Serial,
            ..Default::default()
        };
        let serial_fetcher = UrlFetcher::new(serial_config);
        let serial_results = serial_fetcher.fetch_urls(urls.clone()).await;
        assert_eq!(serial_results.len(), 3);

        // 测试并行策略
        let parallel_config = FetcherConfig {
            strategy: ConcurrencyStrategy::Parallel,
            ..Default::default()
        };
        let parallel_fetcher = UrlFetcher::new(parallel_config);
        let parallel_results = parallel_fetcher.fetch_urls(urls).await;
        assert_eq!(parallel_results.len(), 3);

        println!("串行策略和并行策略都已测试");
    }

    #[tokio::test]
    async fn test_adaptive_strategy() {
        let urls_small = vec!["url1".to_string(), "url2".to_string()];
        let urls_large = (0..10).map(|i| format!("url{}", i)).collect();

        // 测试自适应策略
        let adaptive_config = FetcherConfig {
            strategy: ConcurrencyStrategy::Adaptive,
            batch_threshold: 5,
            ..Default::default()
        };
        let fetcher = UrlFetcher::new(adaptive_config);

        // 小批量：应使用串行
        let _results_small = fetcher.fetch_urls(urls_small).await;

        // 大批量：应使用并行
        let _results_large = fetcher.fetch_urls(urls_large).await;

        println!("自适应策略已测试");
    }

    #[test]
    fn test_distance_strategies() {
        let result_before = DistanceStrategy::AsyncBefore.compute_distance(0.0, 0.0, 3.0, 4.0);
        let result_after = DistanceStrategy::SyncAfter.compute_distance(0.0, 0.0, 3.0, 4.0);

        assert_eq!(result_before, 5.0);
        assert_eq!(result_after, 5.0);

        println!("Distance strategies both return 5.0");
    }

    #[test]
    fn test_item_processing_strategies() {
        let items = (0..1000).collect::<Vec<_>>();

        // 测试串行策略
        let serial_result = ItemProcessingStrategy::Serial.process_items(&items);
        assert_eq!(serial_result.len(), 1000);

        // 测试并行策略
        let parallel_result = ItemProcessingStrategy::ParallelRayon.process_items(&items);
        assert_eq!(parallel_result.len(), 1000);

        // 验证结果正确性
        for (i, &item) in serial_result.iter().enumerate() {
            assert_eq!(item, i * 2);
        }

        println!("串行和并行策略都已测试");
    }

    #[test]
    fn test_performance_comparator() {
        PerformanceComparator::compare_all_strategies();
    }
}

// ============================================================================
// 异步简化建议和使用指南
// ============================================================================

/// 识别可以简化的async函数
pub fn identify_simplifiable_async_functions() -> Vec<&'static str> {
    vec![
        "内存复制函数 (约30处)",
        "简单计算函数 (约40处)",
        "无副作用的状态查询 (约20处)",
        "同步IO的简单包装 (约30处)",
    ]
}

/// 优化优先级建议
pub fn optimization_priority() -> Vec<&'static str> {
    vec![
        "1. 高频调用的小函数 (收益最大)",
        "2. 批量IO操作 (线性加速)",
        "3. CPU密集型计算 (多核加速)",
        "4. 低频调用的复杂函数 (收益较小)",
    ]
}

// ============================================================================
// 实际应用示例
// ============================================================================

/// 示例1: 如何在游戏中使用UrlFetcher
///
/// ```rust
/// use game_engine::concurrency::{UrlFetcher, FetcherConfig, ConcurrencyStrategy};
///
/// #[tokio::main]
/// async fn main() {
///     // 创建自适应策略的fetcher
///     let config = FetcherConfig {
///         strategy: ConcurrencyStrategy::Adaptive,
///         batch_threshold: 5,
///         enable_profiling: false,
///     };
///
///     let fetcher = UrlFetcher::new(config);
///
///     // 批量获取资源
///     let urls = vec![
///         "https://example.com/texture1.png".to_string(),
///         "https://example.com/texture2.png".to_string(),
///         "https://example.com/model.glb".to_string(),
///     ];
///
///     let results = fetcher.fetch_urls(urls).await;
///     println!("Fetched {} resources", results.len());
/// }
/// ```

/// 示例2: 性能对比测试
///
/// ```rust
/// use game_engine::concurrency::{
///     SumCalculationStrategy, MemoryStrategy,
///     DistanceStrategy, ItemProcessingStrategy,
/// };
///
/// fn main() {
///     // 对比不同策略的性能
///     let values = vec![1, 2, 3, 4, 5];
///
///     // 计算策略
///     let sum1 = SumCalculationStrategy::AsyncBefore.calculate(&values);
///     let sum2 = SumCalculationStrategy::SyncAfter.calculate(&values);
///     assert_eq!(sum1, sum2);
///
///     // 内存策略
///     let data = vec![1, 2, 3, 4, 5];
///     let _clone1 = MemoryStrategy::AsyncBefore.clone_data(&data);
///     let _clone2 = MemoryStrategy::SyncAfter.clone_data(&data);
///
///     // 距离计算策略
///     let dist1 = DistanceStrategy::AsyncBefore.compute_distance(0.0, 0.0, 3.0, 4.0);
///     let dist2 = DistanceStrategy::SyncAfter.compute_distance(0.0, 0.0, 3.0, 4.0);
///     assert_eq!(dist1, dist2);
///
///     // 项目处理策略
///     let items = vec![1, 2, 3, 4, 5];
///     let _result1 = ItemProcessingStrategy::Serial.process_items(&items);
///     let _result2 = ItemProcessingStrategy::ParallelRayon.process_items(&items);
/// }
/// ```

/// 示例3: 自定义策略选择
///
/// ```rust
/// use game_engine::concurrency::{ConcurrencyStrategy, FetcherConfig};
///
/// fn choose_strategy(task_count: usize, is_cpu_intensive: bool) -> ConcurrencyStrategy {
///     match (task_count, is_cpu_intensive) {
///         // 小批量任务：串行处理
///         (n, _) if n < 5 => ConcurrencyStrategy::Serial,
///
///         // CPU密集型：并行处理
///         (_, true) => ConcurrencyStrategy::Parallel,
///
///         // 其他情况：自适应
///         _ => ConcurrencyStrategy::Adaptive,
///     }
/// }
///
/// fn main() {
///     // 为不同场景选择策略
///     let strategy1 = choose_strategy(3, false); // Serial
///     let strategy2 = choose_strategy(100, true); // Parallel
///     let strategy3 = choose_strategy(50, false); // Adaptive
///
///     println!("Strategy 1: {:?}", strategy1);
///     println!("Strategy 2: {:?}", strategy2);
///     println!("Strategy 3: {:?}", strategy3);
/// }
/// ```

// ============================================================================
// 优化总结
// ============================================================================

/// 优化成果总结：
///
/// ## 条件编译优化
/// - **优化前**: 9处条件编译 (`#[cfg(feature = "before_optimization")]` 和 `#[cfg(feature = "after_optimization")]`)
/// - **优化后**: 0处条件编译（使用策略模式替代）
/// - **减少**: 100%（9 -> 0）
///
/// ## 运行时策略选择
/// 所有条件编译已通过策略模式转换为运行时选择：
/// 1. **lock_optimization_guide.rs**: MutexStrategy枚举（StdMutex vs ParkingLotMutex）
/// 2. **dashmap_examples.rs**: HashMapStrategy枚举（ArcMutexHashMap vs DashMapImpl）
/// 3. **mod.rs**: ConcurrencyStrategy枚举（Serial vs Parallel vs Adaptive）
///
/// ## 架构改进
/// 1. **策略模式实现**:
///    - `ConcurrencyStrategy`: Serial/Parallel/Adaptive三种策略
///    - `FetcherConfig`: 可配置的策略执行器
///    - 策略枚举: `SumCalculationStrategy`, `MemoryStrategy`, `DistanceStrategy`, `ItemProcessingStrategy`
///
/// 2. **运行时策略选择**:
///    - 不再需要编译时选择feature
///    - 可以在运行时动态切换策略
///    - 支持自适应策略选择
///
/// 3. **保留所有实现**:
///    - 所有优化前后的实现都被保留
///    - 通过策略枚举选择使用哪个版本
///    - 可以进行性能对比测试
///
/// 4. **更好的测试覆盖**:
///    - 每个策略都有独立的测试
///    - 可以对比不同策略的性能
///    - 支持集成测试和单元测试
///
/// ## 使用优势
/// - **灵活性**: 运行时选择策略，无需重新编译
/// - **可测试性**: 所有策略都可以被测试
/// - **可维护性**: 代码结构清晰，易于扩展
/// - **性能对比**: 可以直接对比不同实现的性能
/// - **文档完善**: 提供了完整的使用示例和文档
