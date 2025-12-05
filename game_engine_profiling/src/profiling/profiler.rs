use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 性能分析器 - 测量和记录性能指标
#[derive(Default)]
pub struct Profiler {
    scopes: HashMap<String, ScopeStats>,
    current_scope: Option<(String, Instant)>,
}

/// 作用域统计信息
#[derive(Debug, Clone)]
pub struct ScopeStats {
    pub name: String,
    pub total_time: Duration,
    pub call_count: u64,
    pub min_time: Duration,
    pub max_time: Duration,
}

impl ScopeStats {
    fn new(name: String) -> Self {
        Self {
            name,
            total_time: Duration::ZERO,
            call_count: 0,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.total_time += duration;
        self.call_count += 1;
        self.min_time = self.min_time.min(duration);
        self.max_time = self.max_time.max(duration);
    }

    pub fn average_time(&self) -> Duration {
        if self.call_count > 0 {
            self.total_time / self.call_count as u32
        } else {
            Duration::ZERO
        }
    }
}

impl Profiler {
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始一个性能测量作用域
    pub fn begin_scope(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.current_scope = Some((name, Instant::now()));
    }

    /// 结束当前作用域
    pub fn end_scope(&mut self) {
        if let Some((name, start)) = self.current_scope.take() {
            let duration = start.elapsed();
            let stats = self
                .scopes
                .entry(name.clone())
                .or_insert_with(|| ScopeStats::new(name));
            stats.record(duration);
        }
    }

    /// 获取作用域统计信息
    pub fn get_stats(&self, name: &str) -> Option<&ScopeStats> {
        self.scopes.get(name)
    }

    /// 获取所有统计信息
    pub fn all_stats(&self) -> Vec<&ScopeStats> {
        self.scopes.values().collect()
    }

    /// 清空所有统计信息
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.current_scope = None;
    }

    /// 打印性能报告
    pub fn print_report(&self) {
        println!("\n=== Performance Report ===");
        let mut stats: Vec<_> = self.scopes.values().collect();
        stats.sort_by(|a, b| b.total_time.cmp(&a.total_time));

        for stat in stats {
            println!(
                "{}: {} calls, total: {:?}, avg: {:?}, min: {:?}, max: {:?}",
                stat.name,
                stat.call_count,
                stat.total_time,
                stat.average_time(),
                stat.min_time,
                stat.max_time
            );
        }
        println!("========================\n");
    }
}

/// 性能测量作用域守卫 - 使用RAII自动测量
pub struct ProfileScope<'a> {
    profiler: &'a mut Profiler,
}

impl<'a> ProfileScope<'a> {
    pub fn new(profiler: &'a mut Profiler, name: impl Into<String>) -> Self {
        profiler.begin_scope(name);
        Self { profiler }
    }
}

impl<'a> Drop for ProfileScope<'a> {
    fn drop(&mut self) {
        self.profiler.end_scope();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler() {
        let mut profiler = Profiler::new();

        // 测量一个作用域
        profiler.begin_scope("test_scope");
        thread::sleep(Duration::from_millis(10));
        profiler.end_scope();

        // 再次测量同一个作用域
        profiler.begin_scope("test_scope");
        thread::sleep(Duration::from_millis(20));
        profiler.end_scope();

        // 验证统计信息
        let stats = profiler.get_stats("test_scope").unwrap();
        assert_eq!(stats.call_count, 2);
        assert!(stats.total_time >= Duration::from_millis(30));
        assert!(stats.min_time >= Duration::from_millis(10));
        assert!(stats.max_time >= Duration::from_millis(20));
    }

    #[test]
    fn test_profile_scope() {
        let mut profiler = Profiler::new();

        {
            let _scope = ProfileScope::new(&mut profiler, "auto_scope");
            thread::sleep(Duration::from_millis(10));
        } // 作用域结束时自动调用end_scope

        let stats = profiler.get_stats("auto_scope").unwrap();
        assert_eq!(stats.call_count, 1);
    }
}
