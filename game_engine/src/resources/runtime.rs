//  全局异步运行时
//
//  提供统一的Tokio运行时，避免每个模块创建独立运行时

use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// 全局Tokio运行时
static GLOBAL_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// 获取全局Tokio运行时
///
/// 首次调用时会创建一个多线程运行时，后续调用返回同一实例
///
/// # Panics
/// 如果运行时创建失败会panic
pub fn global_runtime() -> &'static Runtime {
    GLOBAL_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("asset-io")
            .enable_all()
            .build()
            .unwrap_or_else(|e| {
                panic!("Failed to create global tokio runtime: {e}");
            })
    })
}

/// 在全局运行时中执行异步任务
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    global_runtime().spawn(future)
}

use crate::platform::run_sync;

/// 阻塞执行异步任务（仅在无法避免阻塞时使用）
pub fn block_on<F: std::future::Future + Send + 'static>(future: F) -> F::Output
where
    F::Output: Send,
{
    run_sync(future)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_runtime() {
        let rt1 = global_runtime();
        let rt2 = global_runtime();
        // 确保返回同一个运行时
        assert!(std::ptr::eq(rt1, rt2));
    }

    #[test]
    fn test_spawn_task() {
        let handle = spawn(async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            42
        });

        let result = run_sync(handle).expect("Test: operation should succeed");
        assert_eq!(result, 42);
    }
}
