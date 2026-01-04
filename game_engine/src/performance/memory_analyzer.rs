//! # Memory Bottleneck Analyzer
//!
//! 内存瓶颈详细分析工具 - 检测内存泄漏、碎片化和分配热点。
//!
//! ## 核心组件
//!
//! 1. **MemoryAnalyzer** - 内存分析器
//! 2. **LeakDetector** - 泄漏检测器
//! 3. **FragmentationAnalyzer** - 碎片化分析器
//! 4. **AllocationProfiler** - 分配热点分析器

use std::collections::HashMap;
use std::time::{Duration, Instant};

// Import Severity from profiler module
use super::profiler::Severity;

/// 内存瓶颈类型
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryBottleneckType {
    /// 内存泄漏
    Leak,
    /// 碎片化
    Fragmentation,
    /// 分配热点
    AllocationHotspot,
    /// 缓存未命中
    CacheMiss,
    /// 大对象分配
    LargeAllocation,
}

/// 内存分配记录
#[derive(Clone, Debug)]
pub struct AllocationRecord {
    pub address: usize,
    pub size: usize,
    pub allocation_type: String,
    pub timestamp: Instant,
    pub stack_trace: Vec<String>,
}

/// 内存统计
#[derive(Clone, Debug, Default)]
pub struct MemoryStats {
    /// 总分配量（字节）
    pub total_allocated: u64,
    /// 当前使用量（字节）
    pub current_usage: u64,
    /// 峰值使用量（字节）
    pub peak_usage: u64,
    /// 分配次数
    pub allocation_count: u64,
    /// 释放次数
    pub deallocation_count: u64,
    /// 活动分配数
    pub active_allocations: u64,
}

// ==================== 泄漏检测器 ====================

/// 泄漏检测器
pub struct LeakDetector {
    /// 分配记录
    allocations: HashMap<usize, AllocationRecord>,
    /// 类型统计
    type_stats: HashMap<String, TypeMemoryStats>,
    /// 检测阈值（分配次数）
    detection_threshold: u32,
    /// 时间窗口
    time_window: Duration,
}

/// 类型内存统计
#[derive(Clone, Debug)]
pub struct TypeMemoryStats {
    pub allocation_count: u64,
    pub total_allocated: u64,
    pub current_allocated: u64,
    pub average_size: f64,
}

impl LeakDetector {
    /// 创建新的检测器
    pub fn new(detection_threshold: u32, time_window: Duration) -> Self {
        Self {
            allocations: HashMap::new(),
            type_stats: HashMap::new(),
            detection_threshold,
            time_window,
        }
    }

    /// 记录分配
    pub fn record_allocation(
        &mut self,
        address: usize,
        size: usize,
        allocation_type: String,
        stack_trace: Vec<String>,
    ) {
        let record = AllocationRecord {
            address,
            size,
            allocation_type: allocation_type.clone(),
            timestamp: Instant::now(),
            stack_trace,
        };

        self.allocations.insert(address, record.clone());

        // 更新类型统计
        let stats = self.type_stats.entry(allocation_type).or_insert_with(|| TypeMemoryStats {
            allocation_count: 0,
            total_allocated: 0,
            current_allocated: 0,
            average_size: 0.0,
        });

        stats.allocation_count += 1;
        stats.total_allocated += size as u64;
        stats.current_allocated += size as u64;
        stats.average_size = stats.total_allocated as f64 / stats.allocation_count as f64;
    }

    /// 记录释放
    pub fn record_deallocation(&mut self, address: usize) {
        if let Some(record) = self.allocations.remove(&address) {
            // 更新类型统计
            if let Some(stats) = self.type_stats.get_mut(&record.allocation_type) {
                stats.current_allocated -= record.size as u64;
            }
        }
    }

    /// 检测泄漏
    pub fn detect_leaks(&self) -> Vec<MemoryLeak> {
        let mut leaks = Vec::new();
        let now = Instant::now();

        // 按类型分组
        let mut type_groups: HashMap<String, Vec<&AllocationRecord>> = HashMap::new();
        for record in self.allocations.values() {
            type_groups.entry(record.allocation_type.clone()).or_default().push(record);
        }

        // 分析每个类型
        for (alloc_type, records) in type_groups {
            // 检查长时间未释放的分配
            let long_lived: Vec<_> = records
                .iter()
                .filter(|r| now.duration_since(r.timestamp) > self.time_window)
                .collect();

            if long_lived.len() > self.detection_threshold as usize {
                let total_size: u64 = long_lived.iter().map(|r| r.size as u64).sum();
                let avg_size = total_size as f64 / long_lived.len() as f64;

                leaks.push(MemoryLeak {
                    leak_type: alloc_type,
                    leak_count: long_lived.len(),
                    total_size,
                    average_size: avg_size as f32,
                    severity: if total_size > 100_000_000 {
                        LeakSeverity::Critical // >100MB
                    } else if total_size > 10_000_000 {
                        LeakSeverity::High // >10MB
                    } else {
                        LeakSeverity::Moderate
                    },
                    sample_stack_traces: long_lived
                        .iter()
                        .take(5)
                        .map(|r| r.stack_trace.clone())
                        .collect(),
                });
            }
        }

        leaks
    }
}

/// 内存泄漏
#[derive(Clone, Debug)]
pub struct MemoryLeak {
    pub leak_type: String,
    pub leak_count: usize,
    pub total_size: u64,
    pub average_size: f32,
    pub severity: LeakSeverity,
    pub sample_stack_traces: Vec<Vec<String>>,
}

/// 泄漏严重程度
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeakSeverity {
    Moderate,
    High,
    Critical,
}

// ==================== 碎片化分析器 ====================

/// 碎片化分析器
pub struct FragmentationAnalyzer {
    /// 历史快照
    snapshots: Vec<MemorySnapshot>,
    /// 最大快照数
    max_snapshots: usize,
}

/// 内存快照
#[derive(Clone, Debug)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub total_heap: u64,
    pub used_heap: u64,
    pub free_heap: u64,
    pub largest_free_block: u64,
    pub fragmentation_ratio: f32,
}

impl FragmentationAnalyzer {
    /// 创建新的分析器
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Vec::with_capacity(max_snapshots),
            max_snapshots,
        }
    }

    /// 记录快照
    pub fn record_snapshot(&mut self, snapshot: MemorySnapshot) {
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot);
    }

    /// 分析碎片化
    pub fn analyze(&self) -> FragmentationReport {
        if self.snapshots.is_empty() {
            return FragmentationReport {
                current_fragmentation: 0.0,
                average_fragmentation: 0.0,
                trend: FragmentationTrend::Stable,
                severity: FragmentationSeverity::None,
                recommendations: Vec::new(),
            };
        }

        let latest = self.snapshots.last().unwrap();
        let avg_fragmentation = self.snapshots.iter().map(|s| s.fragmentation_ratio).sum::<f32>()
            / self.snapshots.len() as f32;

        // 判断趋势
        let recent_avg: f32 =
            self.snapshots.iter().rev().take(5).map(|s| s.fragmentation_ratio).sum::<f32>()
                / self.snapshots.len().min(5) as f32;

        let older_avg: f32 = self
            .snapshots
            .iter()
            .take(self.snapshots.len().saturating_sub(5))
            .map(|s| s.fragmentation_ratio)
            .sum::<f32>()
            / self.snapshots.len().saturating_sub(5).max(1) as f32;

        let trend = if recent_avg > older_avg * 1.1 {
            FragmentationTrend::Worsening
        } else if recent_avg < older_avg * 0.9 {
            FragmentationTrend::Improving
        } else {
            FragmentationTrend::Stable
        };

        // 评估严重程度
        let severity = if latest.fragmentation_ratio > 0.5 {
            FragmentationSeverity::Severe
        } else if latest.fragmentation_ratio > 0.3 {
            FragmentationSeverity::Moderate
        } else {
            FragmentationSeverity::None
        };

        let mut recommendations = Vec::new();

        if severity != FragmentationSeverity::None {
            recommendations.push("Consider defragmentation or memory compaction".to_string());
            recommendations.push("Use memory pools to reduce fragmentation".to_string());
            recommendations.push("Allocate similar-sized objects together".to_string());
        }

        if trend == FragmentationTrend::Worsening {
            recommendations.push("Memory fragmentation is increasing over time".to_string());
            recommendations.push("Review allocation patterns".to_string());
        }

        FragmentationReport {
            current_fragmentation: latest.fragmentation_ratio,
            average_fragmentation: avg_fragmentation,
            trend,
            severity,
            recommendations,
        }
    }
}

/// 碎片化趋势
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentationTrend {
    Improving,
    Stable,
    Worsening,
}

/// 碎片化严重程度
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentationSeverity {
    None,
    Moderate,
    Severe,
}

/// 碎片化报告
#[derive(Clone, Debug)]
pub struct FragmentationReport {
    pub current_fragmentation: f32,
    pub average_fragmentation: f32,
    pub trend: FragmentationTrend,
    pub severity: FragmentationSeverity,
    pub recommendations: Vec<String>,
}

// ==================== 分配热点分析器 ====================

/// 分配热点分析器
pub struct AllocationProfiler {
    /// 分配统计
    allocation_stats: HashMap<String, AllocationStats>,
    /// 调用栈统计
    stack_stats: HashMap<String, StackAllocationStats>,
}

/// 分配统计
#[derive(Clone, Debug)]
pub struct AllocationStats {
    pub allocation_count: u64,
    pub total_size: u64,
    pub average_size: f64,
    pub peak_size: u64,
}

/// 调用栈统计
#[derive(Clone, Debug)]
pub struct StackAllocationStats {
    pub location: String,
    pub allocation_count: u64,
    pub total_size: u64,
    pub average_size: f32,
}

impl Default for AllocationProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl AllocationProfiler {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            allocation_stats: HashMap::new(),
            stack_stats: HashMap::new(),
        }
    }

    /// 记录分配
    pub fn record_allocation(
        &mut self,
        allocation_type: String,
        size: usize,
        stack_location: String,
    ) {
        // 更新类型统计
        let stats = self.allocation_stats.entry(allocation_type.clone()).or_insert_with(|| {
            AllocationStats {
                allocation_count: 0,
                total_size: 0,
                average_size: 0.0,
                peak_size: 0,
            }
        });

        stats.allocation_count += 1;
        stats.total_size += size as u64;
        stats.average_size = stats.total_size as f64 / stats.allocation_count as f64;
        stats.peak_size = stats.peak_size.max(size as u64);

        // 更新调用栈统计
        let stack_stats = self.stack_stats.entry(stack_location.clone()).or_insert_with(|| {
            StackAllocationStats {
                location: stack_location.clone(),
                allocation_count: 0,
                total_size: 0,
                average_size: 0.0,
            }
        });

        stack_stats.allocation_count += 1;
        stack_stats.total_size += size as u64;
        stack_stats.average_size =
            stack_stats.total_size as f32 / stack_stats.allocation_count as f32;
    }

    /// 分析热点
    pub fn analyze_hotspots(&self, top_n: usize) -> AllocationHotspotReport {
        // 找出最频繁分配的类型
        let mut type_vec: Vec<_> = self.allocation_stats.iter().collect();
        type_vec.sort_by(|a, b| b.1.allocation_count.cmp(&a.1.allocation_count));

        let top_types: Vec<(String, AllocationStats)> = type_vec
            .into_iter()
            .take(top_n)
            .map(|(name, stats)| ((*name).clone(), (*stats).clone()))
            .collect();

        // 找出最频繁的分配位置
        let mut stack_vec: Vec<_> = self.stack_stats.iter().collect();
        stack_vec.sort_by(|a, b| b.1.allocation_count.cmp(&a.1.allocation_count));

        let top_stacks: Vec<(String, StackAllocationStats)> = stack_vec
            .into_iter()
            .take(top_n)
            .map(|(name, stats)| ((*name).clone(), (*stats).clone()))
            .collect();

        AllocationHotspotReport {
            top_allocation_types: top_types,
            top_allocation_stacks: top_stacks,
        }
    }
}

/// 分配热点报告
#[derive(Clone, Debug)]
pub struct AllocationHotspotReport {
    pub top_allocation_types: Vec<(String, AllocationStats)>,
    pub top_allocation_stacks: Vec<(String, StackAllocationStats)>,
}

// ==================== 内存分析器 ====================

/// 内存分析器（主入口）
pub struct MemoryAnalyzer {
    leak_detector: LeakDetector,
    fragmentation_analyzer: FragmentationAnalyzer,
    allocation_profiler: AllocationProfiler,
}

impl MemoryAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            leak_detector: LeakDetector::new(1000, Duration::from_secs(60)),
            fragmentation_analyzer: FragmentationAnalyzer::new(100),
            allocation_profiler: AllocationProfiler::new(),
        }
    }

    /// 分析内存瓶颈
    pub fn analyze(&self, stats: &MemoryStats) -> MemoryBottleneckReport {
        // 检测泄漏
        let leaks = self.leak_detector.detect_leaks();

        // 分析碎片化
        let fragmentation_report = self.fragmentation_analyzer.analyze();

        // 分析热点
        let hotspot_report = self.allocation_profiler.analyze_hotspots(10);

        // 综合瓶颈
        let bottlenecks =
            self.detect_memory_bottlenecks(stats, &leaks, &fragmentation_report, &hotspot_report);

        // 生成建议
        let mut recommendations = Vec::new();

        recommendations.extend(fragmentation_report.recommendations.clone());

        if !leaks.is_empty() {
            recommendations.push("Potential memory leaks detected".to_string());
            recommendations.push("Review object lifetimes".to_string());
        }

        if !hotspot_report.top_allocation_types.is_empty() {
            let top_type = &hotspot_report.top_allocation_types[0];
            recommendations.push(format!(
                "Hot allocation type: {} ({} allocations)",
                top_type.0, top_type.1.allocation_count
            ));
        }

        MemoryBottleneckReport {
            leaks,
            fragmentation_report,
            hotspot_report,
            bottlenecks,
            recommendations,
        }
    }

    fn detect_memory_bottlenecks(
        &self,
        stats: &MemoryStats,
        leaks: &[MemoryLeak],
        fragmentation: &FragmentationReport,
        hotspots: &AllocationHotspotReport,
    ) -> Vec<MemoryBottleneck> {
        let mut bottlenecks = Vec::new();

        // 泄漏瓶颈
        for leak in leaks {
            bottlenecks.push(MemoryBottleneck {
                bottleneck_type: MemoryBottleneckType::Leak,
                severity: match leak.severity {
                    LeakSeverity::Moderate => Severity::Medium,
                    LeakSeverity::High => Severity::High,
                    LeakSeverity::Critical => Severity::Critical,
                },
                description: format!(
                    "Memory leak in {}: {} objects, {:.2} MB",
                    leak.leak_type,
                    leak.leak_count,
                    leak.total_size as f64 / 1_000_000.0
                ),
                impact: "Memory usage keeps growing".to_string(),
            });
        }

        // 碎片化瓶颈
        if fragmentation.severity != FragmentationSeverity::None {
            bottlenecks.push(MemoryBottleneck {
                bottleneck_type: MemoryBottleneckType::Fragmentation,
                severity: match fragmentation.severity {
                    FragmentationSeverity::Moderate => Severity::Medium,
                    FragmentationSeverity::Severe => Severity::High,
                    FragmentationSeverity::None => Severity::Low,
                },
                description: format!(
                    "Memory fragmentation: {:.1}%",
                    fragmentation.current_fragmentation * 100.0
                ),
                impact: "Reduces effective memory bandwidth".to_string(),
            });
        }

        // 分配热点
        if !hotspots.top_allocation_types.is_empty() {
            let top = &hotspots.top_allocation_types[0];
            if top.1.allocation_count > 10000 {
                bottlenecks.push(MemoryBottleneck {
                    bottleneck_type: MemoryBottleneckType::AllocationHotspot,
                    severity: Severity::Low,
                    description: format!(
                        "Frequent allocations: {} ({} times, {:.2} MB)",
                        top.0,
                        top.1.allocation_count,
                        top.1.total_size as f64 / 1_000_000.0
                    ),
                    impact: "Consider using object pools".to_string(),
                });
            }
        }

        bottlenecks
    }

    /// 记录分配
    pub fn record_allocation(
        &mut self,
        address: usize,
        size: usize,
        alloc_type: String,
        stack: Vec<String>,
    ) {
        self.leak_detector
            .record_allocation(address, size, alloc_type.clone(), stack.clone());
        self.allocation_profiler.record_allocation(alloc_type, size, stack.join(":"));
    }

    /// 记录释放
    pub fn record_deallocation(&mut self, address: usize) {
        self.leak_detector.record_deallocation(address);
    }

    /// 记录快照
    pub fn record_snapshot(&mut self, snapshot: MemorySnapshot) {
        self.fragmentation_analyzer.record_snapshot(snapshot);
    }
}

impl Default for MemoryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存瓶颈
#[derive(Clone, Debug)]
pub struct MemoryBottleneck {
    pub bottleneck_type: MemoryBottleneckType,
    pub severity: Severity,
    pub description: String,
    pub impact: String,
}

/// 内存瓶颈报告
#[derive(Clone, Debug)]
pub struct MemoryBottleneckReport {
    pub leaks: Vec<MemoryLeak>,
    pub fragmentation_report: FragmentationReport,
    pub hotspot_report: AllocationHotspotReport,
    pub bottlenecks: Vec<MemoryBottleneck>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leak_detector() {
        let mut detector = LeakDetector::new(10, Duration::from_secs(60));

        // 模拟分配
        for i in 0..100 {
            detector.record_allocation(i, 1024, "TestType".to_string(), Vec::new());
        }

        // 部分释放
        for i in 0..50 {
            detector.record_deallocation(i);
        }

        let leaks = detector.detect_leaks();
        assert!(!leaks.is_empty());
    }

    #[test]
    fn test_fragmentation_analyzer() {
        let mut analyzer = FragmentationAnalyzer::new(10);

        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            total_heap: 1_000_000_000,
            used_heap: 600_000_000,
            free_heap: 400_000_000,
            largest_free_block: 200_000_000,
            fragmentation_ratio: 0.4,
        };

        analyzer.record_snapshot(snapshot);
        let report = analyzer.analyze();

        assert_eq!(report.current_fragmentation, 0.4);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_allocation_profiler() {
        let mut profiler = AllocationProfiler::new();

        profiler.record_allocation("TestType".to_string(), 1024, "test_function".to_string());
        profiler.record_allocation("TestType".to_string(), 2048, "test_function".to_string());

        let report = profiler.analyze_hotspots(10);
        assert!(!report.top_allocation_types.is_empty());
    }
}
