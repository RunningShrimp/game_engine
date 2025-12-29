pub mod baseline_updater;
pub mod benchmark;
pub mod benchmark_baselines;
pub mod benchmark_runner;
pub mod critical_path_benchmarks;
pub mod gpu_comparative_benchmark;
pub mod performance_regression_tests;
pub mod regression_testing;

pub use baseline_updater::{
    BaselineUpdater, BenchmarkBaseline as BaselineUpdaterBaseline, PerformanceBaselines, SystemInfo,
};
pub use benchmark::{
    Benchmark, BenchmarkResult as BenchResult, MemoryBenchmark, PerformanceRegression,
    ThroughputTest,
};
pub use benchmark_baselines::{
    BenchmarkBaseline, CriticalPathBenchmarks, RegressionDetector, RegressionReport,
};
pub use benchmark_runner::{
    BenchmarkResult as RunnerBenchResult, BenchmarkRunner, BenchmarkStatistics, BenchmarkSuite,
};
pub use game_engine_common::benchmarking::{
    CpuGpuComparison, OptimizationGoal, OptimizationResult, PerformanceValidationSuite,
    ValidationSummary,
};
pub use gpu_comparative_benchmark::{
    CPUBenchmarkResult, GPUComparativeBenchmarkSuite, GPUSimulationResult,
    PerformanceAnalysis as GPUPerformanceAnalysis, PerformanceBenchmark,
};
pub use performance_regression_tests::{
    PerformanceRegressionSuite, PerformanceThresholds,
    RegressionTestResult as PerformanceRegressionTestResult,
};
pub use regression_testing::{
    BaselineType, PerformanceBaseline, RegressionSummary, RegressionTestResult, RegressionTestSuite,
};
