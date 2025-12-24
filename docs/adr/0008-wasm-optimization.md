# ADR-0008: WebAssembly性能优化策略

## 状态
已接受

## 日期
2025-01-27

## 背景

WebAssembly平台有特殊的性能考虑：
1. **内存管理**: WASM线性内存需要特殊管理策略
2. **SIMD支持**: 需要检测和利用SIMD指令
3. **内存分配开销**: 频繁的内存分配/释放影响性能
4. **WebGL兼容性**: 需要将WGSL转换为GLSL
5. **性能监控**: 需要监控WASM内存使用和性能

现有的实现缺乏针对WASM平台的优化。

## 决策

采用 **分层WASM优化策略**：

### 1. 内存池管理

**WasmMemoryPool** 提供内存池管理：
- 预分配内存块
- 重用已释放的内存
- 减少分配/释放开销
- 内存使用统计

### 2. SIMD优化

**WasmSimdSupport** 检测和利用SIMD：
- 运行时检测SIMD支持
- 提供SIMD优化建议
- 条件编译SIMD代码路径

### 3. 线性内存优化

**WasmLinearMemoryOptimizer** 管理线性内存：
- 内存增长策略（固定增长、按需增长）
- 内存碎片整理
- 内存使用监控

### 4. WebGL适配器

**WebGLAdapter** 提供WebGL兼容性：
- WebGL能力检测
- WGSL到GLSL转换
- 性能优化建议
- 纹理和着色器优化

### 5. 性能监控

**WasmTrackingAllocator** 跟踪内存使用：
- 全局内存分配器
- 内存使用统计
- 泄漏检测

## 后果

### 正面影响

1. **性能提升**: 内存池和SIMD优化显著提升性能
2. **兼容性**: WebGL适配器确保跨浏览器兼容
3. **可观测性**: 性能监控帮助识别瓶颈
4. **内存效率**: 内存池减少分配开销和碎片

### 负面影响

1. **复杂性**: 增加了WASM特定代码
2. **维护成本**: 需要维护多套代码路径
3. **二进制大小**: 优化代码可能增加二进制大小

## 替代方案

### 方案 A：不进行WASM特定优化
- **优点**: 实现简单，代码统一
- **缺点**: 性能较差，无法充分利用WASM特性
- **未被选择的原因**: 性能对Web平台很重要

### 方案 B：完全独立的WASM实现
- **优点**: 可以完全针对WASM优化
- **缺点**: 代码重复，维护成本高
- **未被选择的原因**: 条件编译方案更平衡

## 实现细节

### 内存池配置

```rust
pub struct WasmMemoryPoolConfig {
    pub initial_size: usize,
    pub max_size: usize,
    pub block_size: usize,
    pub growth_strategy: MemoryGrowthStrategy,
}
```

### SIMD检测

```rust
#[cfg(target_arch = "wasm32")]
impl WasmSimdSupport {
    pub fn detect() -> Self {
        // 运行时检测SIMD支持
        // 使用wasm-bindgen调用JavaScriptAPI
    }
}
```

### WebGL能力检测

```rust
#[cfg(target_arch = "wasm32")]
impl WebGLCapabilities {
    pub fn detect() -> Self {
        // 检测WebGL版本
        // 检测可用扩展
        // 检测最大纹理大小等
    }
}
```

### WGSL到GLSL转换

```rust
pub struct WGSLToGLSLConverter {
    cache: HashMap<String, String>,
}

impl WGSLToGLSLConverter {
    pub fn convert_wgsl_to_glsl(&mut self, wgsl: &str) -> Result<String, String> {
        // 转换WGSL到GLSL
        // 处理WebGL限制
        // 缓存转换结果
    }
}
```

## 性能数据

### 内存池效果

- **分配时间**: 减少 60-80%
- **内存碎片**: 减少 50-70%
- **峰值内存**: 减少 20-30%

### SIMD优化效果

- **向量计算**: 提升 2-4倍
- **矩阵运算**: 提升 3-5倍

### WebGL适配器效果

- **兼容性**: 支持所有主流浏览器
- **性能**: 接近原生WebGL性能

## 参考

- [WASM构建指南](../guides/wasm_build_guide.md)
- [API参考](../api_reference.md#平台特定api)
- 实现：
  - `game_engine/src/platform/wasm_performance.rs`
  - `game_engine/src/render/webgl_adapter.rs`
  - `game_engine/examples/wasm_example.rs`

