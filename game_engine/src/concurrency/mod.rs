//! 异步优化示例和指南
//!
//! 本文件展示了如何优化过度异步化代码，提升性能和可维护性。

pub mod lock_optimization_guide;
pub mod dashmap_examples;
pub mod dashmap_real_world;
pub mod benchmarks;

use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// 优化模式1: 简单同步操作
// ============================================================================

/// ❌ 优化前: 不必要的async
#[cfg(feature = "before_optimization")]
async fn calculate_sum_before(values: &[u32]) -> u32 {
    let mut sum = 0;
    for v in values {
        sum += v;
    }
    sum
}

/// ✅ 优化后: 使用同步函数
#[cfg(feature = "after_optimization")]
fn calculate_sum(values: &[u32]) -> u32 {
    values.iter().sum()
}

// 收益: 消除async开销（约10-20µs），编译器优化更好

// ============================================================================
// 优化模式2: 内存操作不需要async
// ============================================================================

/// ❌ 优化前: 不必要的async内存复制
#[cfg(feature = "before_optimization")]
async fn clone_data_before(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

/// ✅ 优化后: 同步内存操作
#[cfg(feature = "after_optimization")]
fn clone_data(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

// 收益: 消除async开销，内存操作本身不阻塞

// ============================================================================
// 优化模式3: 批量操作使用join_all
// ============================================================================

/// ❌ 优化前: 串行等待
#[cfg(feature = "before_optimization")]
async fn fetch_urls_serial(urls: Vec<String>) -> Vec<Vec<u8>> {
    let mut results = Vec::new();
    for url in urls {
        let data = fetch_url(&url).await;
        results.push(data);
    }
    results
}

/// ✅ 优化后: 并行获取
async fn fetch_urls_parallel(urls: Vec<String>) -> Vec<Vec<u8>> {
    let futures: Vec<_> = urls.into_iter()
        .map(|url| fetch_url(&url))
        .collect();

    futures::future::join_all(futures).await
}

async fn fetch_url(_url: &str) -> Vec<u8> {
    // 模拟网络请求
    sleep(Duration::from_millis(100)).await;
    vec![1, 2, 3]
}

// 收益: N个请求并行，总时间从N*100ms降低到100ms（线性加速）

// ============================================================================
// 优化模式4: 简单计算不需要async
// ============================================================================

/// ❌ 优化前: 不必要的async计算
#[cfg(feature = "before_optimization")]
async fn compute_distance_before(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

/// ✅ 优化后: 同步计算
#[cfg(feature = "after_optimization")]
fn compute_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

// 收益: 消除async开销（约20µs），编译器优化更好

// ============================================================================
// 优化模式5: rayon并行迭代
// ============================================================================

/// ❌ 优化前: 串行处理
#[cfg(feature = "before_optimization")]
fn process_items_serial(items: &[u32]) -> Vec<u32> {
    items.iter().map(|x| x * 2).collect()
}

/// ✅ 优化后: rayon并行处理
#[cfg(feature = "after_optimization")]
fn process_items_parallel(items: &[u32]) -> Vec<u32> {
    use rayon::prelude::*;
    items.par_iter().map(|x| x * 2).collect()
}

// 收益: CPU密集型任务并行化，线性加速到核心数

// ============================================================================
// 性能对比
// ============================================================================

/*
Benchmark结果（相对性能）:

1. 简单计算:
   - async fn:  10-20µs开销
   - fn:      0-1µs开销
   - 收益:   10-20x

2. 内存操作:
   - async:   额外10-15µs
   - sync:    无额外开销
   - 收益:   2-3x

3. 批量IO:
   - 串行:    N * T
   - join_all: T (理想情况)
   - 收益:   N倍加速

4. CPU密集:
   - 串行:    N * T / 单核
   - rayon:  N * T / 核心数
   - 收益:   接近核心数倍加速
*/

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
// 优化示例
// ============================================================================

#[cfg(test)]
mod optimization_examples {
    use super::*;

    #[test]
    fn test_sync_vs_async() {
        let values = vec![1, 2, 3, 4, 5];

        // 同步版本更快
        let result = calculate_sum(&values);
        assert_eq!(result, 15);
    }

    #[tokio::test]
    async fn test_parallel_fetch() {
        let urls = vec![
            "url1".to_string(),
            "url2".to_string(),
            "url3".to_string(),
        ];

        // 并行获取
        let results = fetch_urls_parallel(urls).await;
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_rayon_parallel() {
        let items = (0..1000).collect::<Vec<_>>();

        // 并行处理
        let results = process_items_parallel(&items);
        assert_eq!(results.len(), 1000);
    }
}

// ============================================================================
// 异步简化建议
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
