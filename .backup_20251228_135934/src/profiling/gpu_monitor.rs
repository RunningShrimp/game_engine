//! GPU性能监控
//!
//! 提供深度的GPU性能监控和分析功能。
//! 支持GPU使用率监控、着色器编译时间追踪、绘制调用统计和GPU负载分析。

use std::collections::VecDeque;
use std::time::Duration;

/// 着色器编译指标
#[derive(Debug, Clone)]
pub struct ShaderCompileMetric {
    /// 着色器名称
    pub shader_name: String,
    /// 编译时间
    pub compile_time: Duration,
    /// 是否缓存命中
    pub cache_hit: bool,
    /// 着色器大小（字节）
    pub shader_size: usize,
    /// 编译时间戳
    pub timestamp: std::time::SystemTime,
}

impl ShaderCompileMetric {
    /// 创建新的着色器编译指标
    pub fn new(
        shader_name: impl Into<String>,
        compile_time: Duration,
        cache_hit: bool,
        shader_size: usize,
    ) -> Self {
        Self {
            shader_name: shader_name.into(),
            compile_time,
            cache_hit,
            shader_size,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// 绘制调用统计
#[derive(Debug, Clone, Default)]
pub struct DrawCallStatistics {
    /// 总绘制调用数
    pub total_draws: u64,
    /// 实例化绘制调用数
    pub instanced_draws: u64,
    /// 间接绘制调用数
    pub indirect_draws: u64,
    /// 每个绘制调用的平均三角形数
    pub avg_triangles_per_draw: f64,
    /// 每个绘制调用的平均顶点数
    pub avg_vertices_per_draw: f64,
    /// 总三角形数
    pub total_triangles: u64,
    /// 总顶点数
    pub total_vertices: u64,
}

impl DrawCallStatistics {
    /// 记录一次绘制调用
    pub fn record_draw(&mut self, triangles: u64, vertices: u64, instanced: bool, indirect: bool) {
        self.total_draws += 1;
        if instanced {
            self.instanced_draws += 1;
        }
        if indirect {
            self.indirect_draws += 1;
        }
        self.total_triangles += triangles;
        self.total_vertices += vertices;

        if self.total_draws > 0 {
            self.avg_triangles_per_draw = self.total_triangles as f64 / self.total_draws as f64;
            self.avg_vertices_per_draw = self.total_vertices as f64 / self.total_draws as f64;
        }
    }

    /// 重置统计
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 获取绘制调用效率（实例化+间接绘制的比例）
    pub fn efficiency(&self) -> f64 {
        if self.total_draws == 0 {
            return 0.0;
        }
        (self.instanced_draws + self.indirect_draws) as f64 / self.total_draws as f64
    }
}

/// 移动平均计算器
struct MovingAverage {
    /// 样本队列
    samples: VecDeque<f64>,
    /// 最大样本数
    max_samples: usize,
    /// 当前平均值
    current_avg: f64,
}

impl MovingAverage {
    /// 创建新的移动平均计算器
    fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            current_avg: 0.0,
        }
    }

    /// 添加样本
    fn add_sample(&mut self, value: f64) {
        if self.samples.len() >= self.max_samples {
            let removed = self.samples.pop_front().unwrap_or(0.0);
            self.current_avg = (self.current_avg * self.samples.len() as f64 - removed + value)
                / self.samples.len() as f64;
        } else {
            let n = self.samples.len() as f64;
            self.current_avg = (self.current_avg * n + value) / (n + 1.0);
        }
        self.samples.push_back(value);
    }

    /// 获取当前平均值
    fn average(&self) -> f64 {
        self.current_avg
    }

    /// 重置
    fn reset(&mut self) {
        self.samples.clear();
        self.current_avg = 0.0;
    }
}

/// GPU负载分析
#[derive(Debug, Clone)]
pub struct GpuLoadAnalysis {
    /// 顶点处理负载（0.0-1.0）
    pub vertex_processing_load: f64,
    /// 片段处理负载（0.0-1.0）
    pub fragment_processing_load: f64,
    /// 计算着色器负载（0.0-1.0）
    pub compute_load: f64,
    /// 内存带宽利用率（0.0-1.0）
    pub memory_bandwidth_utilization: f64,
    /// 推荐优化建议
    pub recommended_optimizations: Vec<String>,
}

impl GpuLoadAnalysis {
    /// 创建新的GPU负载分析
    pub fn new(
        vertex_load: f64,
        fragment_load: f64,
        compute_load: f64,
        memory_bandwidth: f64,
    ) -> Self {
        let mut optimizations = Vec::new();

        // 根据负载情况生成优化建议
        if vertex_load > 0.8 {
            optimizations.push("顶点处理负载高，考虑：减少顶点数、使用LOD、优化几何体".to_string());
        }
        if fragment_load > 0.8 {
            optimizations
                .push("片段处理负载高，考虑：减少过度绘制、优化着色器、降低分辨率".to_string());
        }
        if compute_load > 0.8 {
            optimizations.push("计算着色器负载高，考虑：优化计算逻辑、减少计算量".to_string());
        }
        if memory_bandwidth > 0.8 {
            optimizations.push(
                "内存带宽利用率高，考虑：减少纹理大小、使用压缩纹理、优化内存访问模式".to_string(),
            );
        }

        Self {
            vertex_processing_load: vertex_load.clamp(0.0, 1.0),
            fragment_processing_load: fragment_load.clamp(0.0, 1.0),
            compute_load: compute_load.clamp(0.0, 1.0),
            memory_bandwidth_utilization: memory_bandwidth.clamp(0.0, 1.0),
            recommended_optimizations: optimizations,
        }
    }

    /// 获取总负载
    pub fn total_load(&self) -> f64 {
        (self.vertex_processing_load
            + self.fragment_processing_load
            + self.compute_load
            + self.memory_bandwidth_utilization)
            / 4.0
    }
}

/// GPU性能监控器
pub struct GpuPerformanceMonitor {
    /// GPU使用率（移动平均）
    gpu_usage: MovingAverage,
    /// 显存使用量（MB）
    vram_usage_mb: u64,
    /// 显存总量（MB）
    vram_total_mb: u64,
    /// 着色器编译时间记录
    shader_compile_times: VecDeque<ShaderCompileMetric>,
    /// 最大编译记录数
    max_compile_records: usize,
    /// 绘制调用统计
    draw_call_stats: DrawCallStatistics,
    /// 顶点处理时间（移动平均，毫秒）
    vertex_time: MovingAverage,
    /// 片段处理时间（移动平均，毫秒）
    fragment_time: MovingAverage,
    /// 计算着色器时间（移动平均，毫秒）
    compute_time: MovingAverage,
    /// 内存传输时间（移动平均，毫秒）
    memory_transfer_time: MovingAverage,
}

impl GpuPerformanceMonitor {
    /// 创建新的GPU性能监控器
    ///
    /// # 参数
    /// - `max_samples`: 移动平均的最大样本数
    /// - `max_compile_records`: 最大着色器编译记录数
    pub fn new(max_samples: usize, max_compile_records: usize) -> Self {
        Self {
            gpu_usage: MovingAverage::new(max_samples),
            vram_usage_mb: 0,
            vram_total_mb: 0,
            shader_compile_times: VecDeque::with_capacity(max_compile_records),
            max_compile_records,
            draw_call_stats: DrawCallStatistics::default(),
            vertex_time: MovingAverage::new(max_samples),
            fragment_time: MovingAverage::new(max_samples),
            compute_time: MovingAverage::new(max_samples),
            memory_transfer_time: MovingAverage::new(max_samples),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(60, 100) // 60个样本（约1秒@60fps），100条编译记录
    }

    /// 更新GPU使用率
    pub fn update_gpu_usage(&mut self, usage: f64) {
        self.gpu_usage.add_sample(usage.clamp(0.0, 1.0));
    }

    /// 更新显存使用量
    pub fn update_vram_usage(&mut self, used_mb: u64, total_mb: u64) {
        self.vram_usage_mb = used_mb;
        self.vram_total_mb = total_mb;
    }

    /// 记录着色器编译
    pub fn record_shader_compile(&mut self, metric: ShaderCompileMetric) {
        if self.shader_compile_times.len() >= self.max_compile_records {
            self.shader_compile_times.pop_front();
        }
        self.shader_compile_times.push_back(metric);
    }

    /// 记录绘制调用
    pub fn record_draw_call(
        &mut self,
        triangles: u64,
        vertices: u64,
        instanced: bool,
        indirect: bool,
    ) {
        self.draw_call_stats.record_draw(triangles, vertices, instanced, indirect);
    }

    /// 记录顶点处理时间
    pub fn record_vertex_time(&mut self, time_ms: f64) {
        self.vertex_time.add_sample(time_ms);
    }

    /// 记录片段处理时间
    pub fn record_fragment_time(&mut self, time_ms: f64) {
        self.fragment_time.add_sample(time_ms);
    }

    /// 记录计算着色器时间
    pub fn record_compute_time(&mut self, time_ms: f64) {
        self.compute_time.add_sample(time_ms);
    }

    /// 记录内存传输时间
    pub fn record_memory_transfer_time(&mut self, time_ms: f64) {
        self.memory_transfer_time.add_sample(time_ms);
    }

    /// 获取GPU使用率
    pub fn gpu_usage(&self) -> f64 {
        self.gpu_usage.average()
    }

    /// 获取显存使用率（0.0-1.0）
    pub fn vram_usage_percent(&self) -> f64 {
        if self.vram_total_mb == 0 {
            return 0.0;
        }
        (self.vram_usage_mb as f64 / self.vram_total_mb as f64).clamp(0.0, 1.0)
    }

    /// 获取显存使用量（MB）
    pub fn vram_usage_mb(&self) -> u64 {
        self.vram_usage_mb
    }

    /// 获取显存总量（MB）
    pub fn vram_total_mb(&self) -> u64 {
        self.vram_total_mb
    }

    /// 获取着色器编译记录（迭代器）
    pub fn shader_compile_metrics_iter(&self) -> impl Iterator<Item = &ShaderCompileMetric> {
        self.shader_compile_times.iter()
    }

    /// 获取着色器编译统计
    pub fn shader_compile_stats(&self) -> ShaderCompileStats {
        let total_compiles = self.shader_compile_times.len();
        let cache_hits = self.shader_compile_times.iter().filter(|m| m.cache_hit).count();
        let total_time: Duration = self.shader_compile_times.iter().map(|m| m.compile_time).sum();
        let avg_time = if total_compiles > 0 {
            Duration::from_nanos(total_time.as_nanos() as u64 / total_compiles as u64)
        } else {
            Duration::ZERO
        };

        ShaderCompileStats {
            total_compiles,
            cache_hits,
            cache_misses: total_compiles - cache_hits,
            cache_hit_rate: if total_compiles > 0 {
                cache_hits as f64 / total_compiles as f64
            } else {
                0.0
            },
            avg_compile_time: avg_time,
            total_compile_time: total_time,
        }
    }

    /// 获取绘制调用统计
    pub fn draw_call_statistics(&self) -> &DrawCallStatistics {
        &self.draw_call_stats
    }

    /// 获取GPU负载分析
    pub fn get_load_analysis(&self) -> GpuLoadAnalysis {
        // 基于时间估算负载（简化实现）
        // 实际实现中应该使用GPU查询或性能计数器
        let frame_time_ms = 16.67; // 假设60fps
        let vertex_load = (self.vertex_time.average() / frame_time_ms).min(1.0);
        let fragment_load = (self.fragment_time.average() / frame_time_ms).min(1.0);
        let compute_load = (self.compute_time.average() / frame_time_ms).min(1.0);

        // 基于显存使用率估算内存带宽利用率
        let memory_bandwidth = self.vram_usage_percent() * 0.8; // 简化估算

        GpuLoadAnalysis::new(vertex_load, fragment_load, compute_load, memory_bandwidth)
    }

    /// 重置所有统计
    pub fn reset(&mut self) {
        self.gpu_usage.reset();
        self.draw_call_stats.reset();
        self.vertex_time.reset();
        self.fragment_time.reset();
        self.compute_time.reset();
        self.memory_transfer_time.reset();
        self.shader_compile_times.clear();
    }

    /// 获取着色器编译记录（克隆）
    pub fn get_shader_compile_metrics(&self) -> Vec<ShaderCompileMetric> {
        self.shader_compile_times.iter().cloned().collect()
    }
}

/// 着色器编译统计
#[derive(Debug, Clone)]
pub struct ShaderCompileStats {
    /// 总编译次数
    pub total_compiles: usize,
    /// 缓存命中次数
    pub cache_hits: usize,
    /// 缓存未命中次数
    pub cache_misses: usize,
    /// 缓存命中率（0.0-1.0）
    pub cache_hit_rate: f64,
    /// 平均编译时间
    pub avg_compile_time: Duration,
    /// 总编译时间
    pub total_compile_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_call_statistics() {
        let mut stats = DrawCallStatistics::default();
        stats.record_draw(100, 200, false, false);
        stats.record_draw(200, 400, true, false);
        stats.record_draw(150, 300, false, true);

        assert_eq!(stats.total_draws, 3);
        assert_eq!(stats.instanced_draws, 1);
        assert_eq!(stats.indirect_draws, 1);
        assert_eq!(stats.total_triangles, 450);
        assert!((stats.avg_triangles_per_draw - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_gpu_monitor() {
        let mut monitor = GpuPerformanceMonitor::with_default_config();
        monitor.update_gpu_usage(0.5);
        monitor.update_vram_usage(1024, 2048);

        assert!((monitor.gpu_usage() - 0.5).abs() < 0.01);
        assert_eq!(monitor.vram_usage_percent(), 0.5);
    }

    #[test]
    fn test_load_analysis() {
        let analysis = GpuLoadAnalysis::new(0.9, 0.7, 0.5, 0.6);
        assert!(!analysis.recommended_optimizations.is_empty());
        assert!(analysis.total_load() > 0.0);
    }
}
