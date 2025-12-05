# game_engine_simd

高性能SIMD优化库，为游戏引擎提供跨平台的向量化数学运算和CPU特性检测。

## 特性

- **跨平台支持**: x86_64 (SSE2/SSE4.1/AVX/AVX2/AVX-512) 和 aarch64 (NEON/SVE)
- **自动检测**: 运行时检测CPU特性，选择最优SIMD后端
- **批量处理**: 优化的批量变换、蒙皮、粒子系统处理
- **零成本抽象**: 提供高级API，自动选择最优实现

## 快速开始

```rust
use game_engine_simd::{detect_cpu_features, Vec4Simd, SimdBackend};

// 检测CPU特性
let features = detect_cpu_features();
println!("AVX2支持: {}", features.avx2);

// 使用SIMD向量运算
let a = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
let b = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
let dot = a.dot(&b);

// 获取最优后端
let backend = SimdBackend::best_available();
println!("使用后端: {:?}", backend);
```

## 架构

- **cpu_detect**: CPU特性检测
- **math**: SIMD数学运算（Vec3/Vec4/Mat4/Quat）
- **batch**: 批量处理优化（变换、蒙皮、粒子）

## 性能

相比标量实现，典型性能提升：
- 向量运算: 2-4x
- 矩阵运算: 3-6x
- 批量变换: 4-8x

## 许可证

MIT OR Apache-2.0

