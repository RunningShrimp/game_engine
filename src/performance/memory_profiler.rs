use std::collections::HashMap;
use std::time::Instant;

/// 内存分配记录
#[derive(Debug, Clone)]
pub struct AllocationRecord {
    pub size: usize,
    pub timestamp: Instant,
    pub tag: String,
}

/// 内存分析器
pub struct MemoryProfiler {
    /// 当前分配记录
    allocations: HashMap<usize, AllocationRecord>,
    /// 总分配大小
    total_allocated: usize,
    /// 总释放大小
    total_freed: usize,
    /// 峰值内存使用
    peak_memory: usize,
    /// 是否启用
    enabled: bool,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            total_allocated: 0,
            total_freed: 0,
            peak_memory: 0,
            enabled: true,
        }
    }
    
    /// 记录分配
    pub fn record_allocation(&mut self, ptr: usize, size: usize, tag: String) {
        if !self.enabled {
            return;
        }
        
        self.allocations.insert(ptr, AllocationRecord {
            size,
            timestamp: Instant::now(),
            tag,
        });
        
        self.total_allocated += size;
        
        let current_memory = self.get_current_memory_usage();
        if current_memory > self.peak_memory {
            self.peak_memory = current_memory;
        }
    }
    
    /// 记录释放
    pub fn record_deallocation(&mut self, ptr: usize) {
        if !self.enabled {
            return;
        }
        
        if let Some(record) = self.allocations.remove(&ptr) {
            self.total_freed += record.size;
        }
    }
    
    /// 获取当前内存使用量
    pub fn get_current_memory_usage(&self) -> usize {
        self.allocations.values().map(|r| r.size).sum()
    }
    
    /// 获取峰值内存使用
    pub fn get_peak_memory_usage(&self) -> usize {
        self.peak_memory
    }
    
    /// 获取分配统计
    pub fn get_allocation_stats(&self) -> HashMap<String, (usize, usize)> {
        let mut stats: HashMap<String, (usize, usize)> = HashMap::new();
        
        for record in self.allocations.values() {
            let entry = stats.entry(record.tag.clone()).or_insert((0, 0));
            entry.0 += 1; // 计数
            entry.1 += record.size; // 总大小
        }
        
        stats
    }
    
    /// 检测内存泄漏
    pub fn detect_leaks(&self, threshold_seconds: u64) -> Vec<(usize, AllocationRecord)> {
        let now = Instant::now();
        let mut leaks = Vec::new();
        
        for (ptr, record) in &self.allocations {
            let age = now.duration_since(record.timestamp).as_secs();
            if age > threshold_seconds {
                leaks.push((*ptr, record.clone()));
            }
        }
        
        leaks
    }
    
    /// 生成内存报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Memory Profiler Report ===\n\n");
        
        let current_memory = self.get_current_memory_usage();
        report.push_str(&format!("Current Memory Usage: {} bytes ({:.2} MB)\n", 
            current_memory, current_memory as f64 / 1024.0 / 1024.0));
        report.push_str(&format!("Peak Memory Usage: {} bytes ({:.2} MB)\n", 
            self.peak_memory, self.peak_memory as f64 / 1024.0 / 1024.0));
        report.push_str(&format!("Total Allocated: {} bytes ({:.2} MB)\n", 
            self.total_allocated, self.total_allocated as f64 / 1024.0 / 1024.0));
        report.push_str(&format!("Total Freed: {} bytes ({:.2} MB)\n\n", 
            self.total_freed, self.total_freed as f64 / 1024.0 / 1024.0));
        
        report.push_str("Allocation Statistics by Tag:\n");
        let stats = self.get_allocation_stats();
        let mut sorted_stats: Vec<_> = stats.iter().collect();
        sorted_stats.sort_by(|a, b| b.1.1.cmp(&a.1.1));
        
        for (tag, (count, size)) in sorted_stats {
            report.push_str(&format!("  {}: {} allocations, {} bytes ({:.2} MB)\n", 
                tag, count, size, *size as f64 / 1024.0 / 1024.0));
        }
        
        report.push('\n');
        
        let leaks = self.detect_leaks(60);
        if !leaks.is_empty() {
            report.push_str("Potential Memory Leaks (allocations older than 60s):\n");
            for (ptr, record) in leaks.iter().take(10) {
                report.push_str(&format!("  0x{:x}: {} bytes, tag: {}\n", 
                    ptr, record.size, record.tag));
            }
        } else {
            report.push_str("No potential memory leaks detected.\n");
        }
        
        report
    }
    
    /// 清空统计
    pub fn clear(&mut self) {
        self.allocations.clear();
        self.total_allocated = 0;
        self.total_freed = 0;
        self.peak_memory = 0;
    }
    
    /// 设置是否启用
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU性能分析器
pub struct GpuProfiler {
    /// GPU查询结果 (查询名称 -> 时间(ms))
    queries: HashMap<String, f32>,
    /// 是否启用
    enabled: bool,
}

impl GpuProfiler {
    pub fn new() -> Self {
        Self {
            queries: HashMap::new(),
            enabled: true,
        }
    }
    
    /// 记录GPU查询结果
    pub fn record_query(&mut self, name: String, time_ms: f32) {
        if !self.enabled {
            return;
        }
        
        self.queries.insert(name, time_ms);
    }
    
    /// 获取查询结果
    pub fn get_query(&self, name: &str) -> Option<f32> {
        self.queries.get(name).copied()
    }
    
    /// 获取所有查询
    pub fn get_all_queries(&self) -> &HashMap<String, f32> {
        &self.queries
    }
    
    /// 生成GPU报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== GPU Profiler Report ===\n\n");
        
        if self.queries.is_empty() {
            report.push_str("No GPU queries recorded.\n");
            return report;
        }
        
        let mut sorted_queries: Vec<_> = self.queries.iter().collect();
        sorted_queries.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        let total_time: f32 = self.queries.values().sum();
        report.push_str(&format!("Total GPU Time: {:.2}ms\n\n", total_time));
        
        report.push_str("Query Times:\n");
        for (name, time) in sorted_queries {
            let percentage = (time / total_time) * 100.0;
            report.push_str(&format!("  {}: {:.2}ms ({:.1}%)\n", name, time, percentage));
        }
        
        report
    }
    
    /// 清空查询
    pub fn clear(&mut self) {
        self.queries.clear();
    }
    
    /// 设置是否启用
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for GpuProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_profiler() {
        let mut profiler = MemoryProfiler::new();
        
        // 记录分配
        profiler.record_allocation(0x1000, 1024, "Texture".to_string());
        profiler.record_allocation(0x2000, 2048, "Mesh".to_string());
        
        assert_eq!(profiler.get_current_memory_usage(), 3072);
        assert_eq!(profiler.get_peak_memory_usage(), 3072);
        
        // 记录释放
        profiler.record_deallocation(0x1000);
        assert_eq!(profiler.get_current_memory_usage(), 2048);
    }
    
    #[test]
    fn test_gpu_profiler() {
        let mut profiler = GpuProfiler::new();
        
        profiler.record_query("Shadow Pass".to_string(), 2.5);
        profiler.record_query("Main Pass".to_string(), 10.0);
        
        assert_eq!(profiler.get_query("Shadow Pass"), Some(2.5));
        assert_eq!(profiler.get_query("Main Pass"), Some(10.0));
    }
}
