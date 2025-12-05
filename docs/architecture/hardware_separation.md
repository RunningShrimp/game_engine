# 硬件代码分离架构设计

## 概述

将性能模块中的硬件特定代码（`src/performance/hardware/`）分离为独立的crate `game_engine_hardware`，以提升编译效率和模块化程度。

## 目标

1. **编译时间优化**: 硬件代码变更不影响主crate编译
2. **模块化**: 硬件检测和优化逻辑独立，易于维护
3. **可扩展性**: 便于添加新的硬件平台支持
4. **依赖管理**: 清晰的依赖边界

## 架构设计

### 目录结构

```
game_engine_hardware/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── gpu/
    │   ├── mod.rs
    │   ├── detect.rs
    │   ├── optimization.rs
    │   └── vendor_optimization.rs
    ├── npu/
    │   ├── mod.rs
    │   ├── detect.rs
    │   ├── acceleration.rs
    │   ├── upscaling.rs
    │   └── sdk/
    │       ├── mod.rs
    │       ├── openvino.rs
    │       ├── rocm.rs
    │       ├── ascend.rs
    │       ├── snpe.rs
    │       └── neuropilot.rs
    ├── soc/
    │   ├── mod.rs
    │   ├── detect.rs
    │   └── power.rs
    ├── capability/
    │   ├── mod.rs
    │   └── evaluation.rs
    ├── config/
    │   ├── mod.rs
    │   └── auto_config.rs
    ├── upscaling/
    │   ├── mod.rs
    │   └── sdk.rs
    ├── adaptive/
    │   ├── mod.rs
    │   └── performance.rs
    ├── utils/
    │   ├── mod.rs
    │   ├── cache.rs
    │   ├── ring_buffer.rs
    │   └── metrics.rs
    └── error.rs
```

### 公共API设计

#### 核心类型

```rust
// game_engine_hardware/src/lib.rs

/// 硬件信息
pub struct HardwareInfo {
    pub gpu: GpuInfo,
    pub npu: Option<NpuInfo>,
    pub soc: Option<SocInfo>,
    pub capability: HardwareCapability,
    pub recommended_config: AutoConfig,
}

/// GPU信息
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub name: String,
    pub tier: GpuTier,
    pub vram_mb: u64,
    pub driver_version: String,
}

/// NPU信息
pub struct NpuInfo {
    pub vendor: NpuVendor,
    pub name: String,
    pub tops: f32,
}

/// SoC信息
pub struct SocInfo {
    pub vendor: SocVendor,
    pub name: String,
    pub cpu_cores: u32,
    pub gpu_cores: u32,
}

/// 硬件能力
pub struct HardwareCapability {
    pub tier: PerformanceTier,
    pub gpu_tier: GpuTier,
    pub supports_raytracing: bool,
    pub has_npu: bool,
    // ...
}

/// 自动配置
pub struct AutoConfig {
    pub quality_preset: QualityPreset,
    pub resolution_scale: f32,
    // ...
}
```

#### 公共函数

```rust
/// 检测硬件信息
pub fn detect_hardware() -> HardwareInfo;

/// 获取缓存的硬件信息
pub fn get_hardware_info() -> &'static HardwareInfo;

/// 打印硬件信息
pub fn print_hardware_info();

/// 评估硬件能力
pub fn evaluate_capability(info: &HardwareInfo) -> HardwareCapability;

/// 生成自动配置
pub fn generate_auto_config(capability: &HardwareCapability) -> AutoConfig;
```

### 依赖关系

```
game_engine_hardware
├── serde (序列化)
├── thiserror (错误处理)
├── ort (ONNX Runtime，可选)
└── 平台特定SDK（可选）
    ├── openvino
    ├── rocm
    ├── ascend
    ├── snpe
    └── neuropilot

game_engine (主crate)
└── game_engine_hardware (依赖)
```

### 迁移计划

#### 阶段1: 创建新crate
1. 创建 `game_engine_hardware/` 目录
2. 创建 `Cargo.toml` 和基础结构
3. 定义公共API接口

#### 阶段2: 迁移代码
1. 迁移GPU检测和优化模块
2. 迁移NPU检测和SDK模块
3. 迁移SoC检测和功耗管理模块
4. 迁移能力评估和自动配置模块
5. 迁移工具类（cache, ring_buffer等）

#### 阶段3: 更新依赖
1. 更新主crate的 `Cargo.toml`
2. 更新 `src/performance/mod.rs` 中的导入
3. 更新所有使用硬件模块的代码
4. 修复编译错误

#### 阶段4: 测试和验证
1. 运行单元测试
2. 运行集成测试
3. 验证功能正常
4. 性能基准测试

### 接口兼容性

为了保持向后兼容，主crate中保留重新导出：

```rust
// src/performance/mod.rs
pub use game_engine_hardware::{
    HardwareInfo, GpuInfo, NpuInfo, SocInfo,
    HardwareCapability, AutoConfig,
    get_hardware_info, print_hardware_info,
    // ...
};
```

### 优势

1. **编译时间**: 硬件代码变更时，主crate无需重新编译
2. **模块化**: 硬件相关代码集中管理
3. **可测试性**: 独立crate便于单元测试
4. **可扩展性**: 新增硬件平台支持不影响主crate
5. **依赖清晰**: 硬件SDK依赖隔离在独立crate中

### 风险评估

1. **破坏性变更**: 通过重新导出保持API兼容
2. **编译错误**: 逐步迁移，充分测试
3. **性能影响**: 无性能影响，仅代码组织变更

### 实施时间表

- **设计阶段**: 1天
- **创建crate**: 1天
- **代码迁移**: 3-4天
- **依赖更新**: 1-2天
- **测试验证**: 1-2天

**总计**: 7-10天

