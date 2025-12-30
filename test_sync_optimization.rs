// 测试同步优化效果的脚本
use std::sync::Arc;
use std::time::Instant;

// 模拟 ServiceRegistry 的基本功能
#[derive(Debug)]
struct ServiceRegistry {
    services: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<String, ()>>>,
}

impl ServiceRegistry {
    fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // 原始异步版本
    async fn count_async(&self) -> usize {
        let services = self.services.read().await;
        services.len()
    }

    // 优化后的同步版本
    fn count_sync(&self) -> usize {
        let services = self.services.blocking_read();
        services.len()
    }

    // 写入操作（保持异步）
    async fn insert_async(&self, key: String) {
        let mut services = self.services.write().await;
        services.insert(key, ());
    }

    // 写入操作（保持异步）
    async fn insert_sync(&self, key: String) {
        let mut services = self.services.blocking_write();
        services.insert(key, ());
    }
}

#[tokio::main]
async fn main() {
    println!("=== 同步优化验证测试 ===\n");

    let registry = Arc::new(ServiceRegistry::new());

    // 测试1：读取性能对比
    println!("1. 读取性能对比测试");
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = registry.count_async().await;
    }
    let async_time = start.elapsed();
    println!("   异步读取耗时: {:?}", async_time);

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = registry.count_sync();
    }
    let sync_time = start.elapsed();
    println!("   同步读取耗时: {:?}", sync_time);
    println!("   性能提升: {:.2}x", async_time.as_nanos() as f64 / sync_time.as_nanos() as f64);

    // 测试2：写入操作性能
    println!("\n2. 写入操作性能测试");
    let start = Instant::now();
    for i in 0..1000 {
        registry.insert_async(format!("service_{}", i)).await;
    }
    let async_write_time = start.elapsed();
    println!("   异步写入耗时: {:?}", async_write_time);

    let start = Instant::now();
    for i in 0..1000 {
        registry.insert_sync(format!("service_{}", i)).await;
    }
    let sync_write_time = start.elapsed();
    println!("   同步写入耗时: {:?}", sync_write_time);

    println!("\n=== 测试完成 ===");
}