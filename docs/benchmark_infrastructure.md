# 游戏引擎性能基准测试基础设施

## 概述

本项目建立了一套完整的性能基准测试体系，用于持续监控和优化引擎性能，防止性能回归。基准测试覆盖了引擎的所有核心子系统，并提供详细的性能分析报告。

## 基准测试套件

### 1. ECS基准测试 (`ecs_benchmarks.rs`)

**测试内容:**
- 实体创建性能 (spawn_entities)
- 组件添加性能 (add_components)
- 查询迭代性能 (query_iteration)
- 多组件查询性能 (query_multiple_components)
- 系统调度性能 (schedule_execution)
- 自定义组件性能 (custom_components)

**测试规模:**
- 小规模: 100 个实体
- 中规模: 1,000 个实体
- 大规模: 10,000 个实体

**关键指标:**
- 实体创建时间
- 组件迭代吞吐量
- 查询延迟
- 系统调度开销

### 2. 物理基准测试 (`physics_benchmarks.rs`)

**测试内容:**
- 物理步进性能 (physics_step)
- 碰撞检测性能 (collision_detection)
- 空间查询性能 (spatial_query)
- 刚体创建性能 (rigid_body_creation)
- 物理ECS集成性能 (physics_ecs_integration)
- 连续碰撞检测CCD性能 (ccd)

**测试规模:**
- 小型场景: 10-50 个刚体
- 中型场景: 100 个刚体
- 大型场景: 500-1,000 个刚体

**关键指标:**
- 物理模拟帧时间
- 碰撞对检测速度
- 射线投射延迟
- 内存使用效率

### 3. 渲染基准测试 (`render_benchmarks.rs`)

**测试内容:**
- 视锥剔除性能 (frustum_culling)
- 变换计算性能 (transform_calculations)
- 渲染排序性能 (render_sorting)
- 批处理性能 (batching)
- MVP矩阵计算 (mvp_calculation)
- 骨骼动画计算 (skeletal_animation)

**测试规模:**
- 小场景: 100 个对象
- 中场景: 1,000 个对象
- 大场景: 10,000 个对象

**关键指标:**
- 剔除效率
- 批处理合并率
- 排序算法性能
- 矩阵计算吞吐量

### 4. 序列化基准测试 (`serialization_benchmarks.rs`)

**测试内容:**
- 网络消息序列化 (message_serialization)
- 网络消息反序列化 (message_deserialization)
- 场景序列化 (scene_serialization)
- 场景反序列化 (scene_deserialization)
- JSON序列化对比 (json_serialization)
- 存档保存/加载 (save_game/load_game)
- 压缩性能 (compression)

**测试规模:**
- 小消息: 64 字节
- 中消息: 256 字节 - 1 KB
- 大消息: 4 KB
- 场景数据: 10-1,000 个实体

**关键指标:**
- 序列化吞吐量
- 反序列化延迟
- 压缩率
- 内存分配次数

### 5. 内存基准测试 (`memory_benchmarks.rs`)

**测试内容:**
- 实体内存分配 (entity_memory_allocation)
- 组件内存分配 (component_memory_allocation)
- 实体池重用 (entity_pool_reuse)
- 组件布局效率 (component_layout)
- 查询内存访问 (query_memory_access)
- 批量操作内存 (batch_operations)
- 资源内存使用 (resource_memory)
- 内存碎片 (memory_fragmentation)

**测试规模:**
- 小规模: 100 个对象
- 中规模: 1,000 个对象
- 大规模: 10,000 个对象

**关键指标:**
- 分配次数
- 分配字节数
- 内存碎片率
- 缓存命中率

### 6. 数学基准测试 (`math_benchmarks.rs`)

**测试内容:**
- 向量运算
- 矩阵变换
- 四元数旋转
- SIMD加速对比

### 7. 网络基准测试 (`network_benchmarks.rs`)

**测试内容:**
- 消息编码/解码
- 连接管理
- 数据包处理

### 8. 密路查找基准测试 (`pathfinding_benchmarks.rs`)

**测试内容:**
- A*算法性能
- 导航网格查询
- 路径优化

### 9. 资源管理基准测试 (`resource_benchmarks.rs`)

**测试内容:**
- 缓存命中率
- 资源加载
- 内存管理

## 使用方法

### 运行所有基准测试

```bash
# 运行所有benchmark并生成HTML报告
cargo bench --workspace

# 保存baseline
cargo bench --workspace -- --save-baseline main

# 与baseline对比
cargo bench --workspace -- --baseline main
```

### 运行特定基准测试

```bash
# ECS benchmarks
cargo bench --bench ecs_benchmarks

# Physics benchmarks
cargo bench --bench physics_benchmarks

# Render benchmarks
cargo bench --bench render_benchmarks

# Serialization benchmarks
cargo bench --bench serialization_benchmarks

# Memory benchmarks
cargo bench --bench memory_benchmarks
```

### 查看基准测试结果

基准测试结果保存在 `game_engine/benches/results/` 目录:

```bash
# 打开HTML报告
open game_engine/benches/results/report/index.html

# 查看特定benchmark的结果
cat game_engine/benches/results/<benchmark-name>/new/estimates.json
```

## 配置文件

### Criterion配置 (`criterion.toml`)

```toml
[criterion]
output_folder = "benches/results"
baseline = "main"
measurement_time = 5.0
warm_up_time = 3.0
sample_size = 100

[output]
plaintext = true
html = true
json = true

[plots]
comparison = true
```

### Cargo配置 (`.cargo/config.toml`)

```toml
[profile.bench]
inherits = "release"
opt-level = 3
lto = true
codegen-units = 1  # 确保稳定的结果
```

## CI/CD集成

### GitHub Actions工作流

项目包含了完整的CI集成 (`.github/workflows/benchmark.yml`):

**工作流程:**
1. **运行基准测试**: 在每次push和PR时运行
2. **建立baseline**: main分支自动保存为baseline
3. **性能回归检测**: PR分支与main分支对比
4. **结果报告**: 自动评论PR，显示性能变化
5. **性能阈值**: 超过150%性能下降会失败

**触发条件:**
- Push到main/master分支
- 创建Pull Request
- 手动触发 (workflow_dispatch)

## 性能基准线

### 当前性能基准线

运行以下命令建立baseline:

```bash
# 在main分支上
git checkout main
cargo bench --workspace -- --save-baseline main
```

### 性能回归检测

当检测到性能回归时:

1. CI会自动失败
2. PR会收到评论，显示性能下降的详细信息
3. 需要分析原因并修复后才能合并

**允许的性能变化范围:**
- 正常波动: ±5%
- 警告: 5-15%
- 严重回归: >15% (CI失败)
- 极端回归: >50% (立即修复)

## 基准测试最佳实践

### 编写新基准测试

1. **使用criterion框架**:
   ```rust
   use criterion::{black_box, BenchmarkId, Criterion, criterion_group, criterion_main};

   fn bench_my_feature(c: &mut Criterion) {
       let mut group = c.benchmark_group("my_feature");

       for size in [100, 1000, 10000].iter() {
           group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
               b.iter(|| {
                   // 使用black_box防止编译器优化
                   black_box(my_function(size))
               });
           });
       }

       group.finish();
   }

   criterion_group!(benches, bench_my_feature);
   criterion_main!(benches);
   ```

2. **使用black_box**: 确保编译器不会优化掉测试代码
3. **设置合理的sample_size**: 确保结果稳定
4. **预热测试**: 允许JIT编译器和缓存预热
5. **测试多个规模**: 不同输入规模下的性能表现

### 基准测试命名规范

- 使用描述性名称: `bench_physics_step` 而不是 `bench_test1`
- 使用snake_case: 符合Rust命名规范
- 包含测试内容: `bench_collision_detection_100_bodies`
- 保持一致性: 所有相关benchmark使用相同前缀

### 性能分析

1. **查看HTML报告**: 最直观的性能对比
2. **分析火焰图**: 识别性能瓶颈
3. **使用perf/macOS Instruments**: 深入分析
4. **检查内存分配**: 使用内存profiler

## 性能优化建议

基于基准测试结果的常见优化方向:

### 1. 减少内存分配
- 使用对象池
- 预分配容量
- 重用缓冲区
- 使用栈分配的小数组

### 2. SIMD优化
- 使用glam的SIMD类型
- 批量处理数据
- 对齐内存访问
- 避免分支预测失败

### 3. 缓存友好
- 优化数据布局 (SoA vs AoS)
- 提高空间局部性
- 减少指针追踪
- 使用紧凑数据结构

### 4. 并行化
- 使用rayon并行迭代
- 分离读写阶段
- 避免false sharing
- 使用work-stealing

### 5. 算法优化
- 选择合适的复杂度
- 使用空间数据结构
- 延迟计算
- 批处理操作

## 故障排除

### 基准测试编译失败

**问题**: API变更导致benchmark无法编译

**解决方案**:
```bash
# 检查编译错误
cargo build --benches --verbose

# 更新benchmark以匹配新API
# 参考现有代码示例
```

### 性能结果不稳定

**问题**: 每次运行结果差异很大

**解决方案**:
1. 增加measurement_time
2. 提高sample_size
3. 关闭后台应用
4. 确保系统负载稳定
5. 使用专用测试机器

### 内存不足

**问题**: 大规模benchmark导致OOM

**解决方案**:
1. 减少测试数据集大小
2. 分批处理数据
3. 使用更小的迭代次数
4. 优化benchmark代码的内存使用

### CI超时

**问题**: Benchmark在CI中运行超时

**解决方案**:
```yaml
# 在.github/workflows/benchmark.yml中增加timeout
timeout-minutes: 60

# 或减少benchmark规模
for size in [100, 1000].iter() {  # 移除10000
```

## 扩展阅读

- [Criterion.rs用户指南](https://bheisler.github.io/criterion.rs/book/index.html)
- [Rust性能优化指南](https://nnethercote.github.io/perf-book/)
- [游戏引擎性能优化](https://www.youtube.com/watch?v=3v74bG5q6xI)
- [SIMD编程指南](https://www.agner.org/optimize/optimizing_cpp.pdf)

## 维护指南

### 定期任务

1. **每周**: 检查CI中的benchmark结果
2. **每月**: 更新baseline（性能提升后）
3. **每季度**: 审查和优化benchmark套件
4. **每年**: 评估是否需要添加新的benchmark

### 添加新benchmark的清单

- [ ] 编写benchmark代码
- [ ] 在Cargo.toml中注册
- [ ] 运行并验证结果
- [ ] 更新文档
- [ ] 更新CI配置（如需要）
- [ ] 通知团队成员

## 贡献指南

提交PR时:

1. 确保所有benchmark通过
2. 检查性能回归（>5%需要说明）
3. 更新相关文档
4. 添加新benchmark时遵循命名规范

## 许可证

与主项目相同 (MIT OR Apache-2.0)
