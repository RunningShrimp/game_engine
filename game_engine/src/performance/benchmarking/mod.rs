pub mod benchmark;
pub mod benchmark_runner;
pub mod benchmark_baselines;
pub mod critical_path_benchmarks;
pub mod gpu_comparative_benchmark;
pub mod performance_regression_tests;
pub mod regression_testing;

pub use benchmark::{
    Benchmark, BenchmarkResult as BenchResult, MemoryBenchmark, PerformanceRegression,
    ThroughputTest,
};
pub use benchmark_runner::{
    BenchmarkResult as RunnerBenchResult, BenchmarkRunner, BenchmarkStatistics, BenchmarkSuite,
};
pub use benchmark_baselines::{
    BenchmarkBaseline, CriticalPathBenchmarks, RegressionDetector, RegressionReport,
};
pub use gpu_comparative_benchmark::{
    CPUBenchmarkResult, GPUComparativeBenchmarkSuite, GPUSimulationResult,
    PerformanceAnalysis as GPUPerformanceAnalysis, PerformanceBenchmark,
};
pub use regression_testing::{
    BaselineType, PerformanceBaseline, RegressionSummary, RegressionTestResult, RegressionTestSuite,
};
pub use game_engine_common::benchmarking::{
    CpuGpuComparison, OptimizationGoal, OptimizationResult, PerformanceValidationSuite,
    ValidationSummary,
};
pub use performance_regression_tests::{
    PerformanceRegressionSuite, PerformanceThresholds, RegressionTestResult,
};

