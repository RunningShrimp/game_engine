// Benchmark Configuration
//
// Centralized configuration for all benchmarks

use std::time::Duration;

/// Benchmark configuration settings
pub struct BenchmarkConfig {
    /// Sample size for each benchmark
    pub sample_size: usize,

    /// Warm-up time
    pub warm_up_time: Duration,

    /// Measurement time
    pub measurement_time: Duration,

    /// Number of iterations to skip (for JIT compilation)
    pub skip_iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            sample_size: 100,
            warm_up_time: Duration::from_secs(3),
            measurement_time: Duration::from_secs(5),
            skip_iterations: 0,
        }
    }
}

/// Scene sizes for scalability testing
pub const SCENE_SIZES: &[usize] = &[100, 500, 1_000, 5_000, 10_000, 50_000];

/// Memory sizes for memory benchmarks (in bytes)
pub const MEMORY_SIZES: &[usize] = &[
    1024,           // 1 KB
    10_240,         // 10 KB
    102_400,        // 100 KB
    1_048_576,      // 1 MB
    10_485_760,     // 10 MB
    104_857_600,    // 100 MB
];

/// Performance targets for validation
pub struct PerformanceTargets {
    /// Maximum time for entity CRUD operations (microseconds)
    pub max_entity_crud_us: u128,

    /// Maximum time for undo/redo operations (milliseconds)
    pub max_undo_redo_ms: u128,

    /// Minimum GPU culling speedup factor
    pub min_gpu_culling_speedup: f64,

    /// Minimum VRAM savings percentage
    pub min_vram_savings_pct: f64,

    /// Minimum draw call reduction percentage
    pub min_draw_call_reduction_pct: f64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            max_entity_crud_us: 100,
            max_undo_redo_ms: 1,
            min_gpu_culling_speedup: 2.0,
            min_vram_savings_pct: 40.0,
            min_draw_call_reduction_pct: 60.0,
        }
    }
}

/// Helper function to format bytes
pub fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Helper function to format duration
pub fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();

    if nanos < 1_000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.2}μs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(512), "512.00 B");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_nanos(100)), "100ns");
        assert_eq!(format_duration(Duration::from_micros(100)), "100.00μs");
        assert_eq!(format_duration(Duration::from_millis(100)), "100.00ms");
    }
}
