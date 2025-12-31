//! # SIMD优化模块
//!
//! 提供各种SIMD指令集的优化实现。

pub mod arm_neon;

pub use arm_neon::{
    NeonArrayOps, NeonBenchmark, NeonMatrixOps, NeonOptimizer, NeonVecOps, is_aarch64, is_arm_arch,
};
