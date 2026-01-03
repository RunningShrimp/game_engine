# 游戏引擎测试指南 (完整版)

本指南提供游戏引擎项目的完整测试策略、工具和最佳实践。

**版本**: 2.0
**最后更新**: 2026-01-02
**维护者**: Game Engine Team

## 目录

- [概述](#概述)
- [测试架构](#测试架构)
- [运行测试](#运行测试)
- [编写测试](#编写测试)
- [性能基准测试](#性能基准测试)
- [属性测试](#属性测试)
- [覆盖率报告](#覆盖率报告)
- [CI/CD集成](#cicd集成)
- [最佳实践](#最佳实践)
- [常见问题](#常见问题)

---

## 概述

### 目标

- **代码覆盖率**: 从42%提升至50%+ (当前目标: 60%)
- **测试数量**: 300+ 测试用例
- **测试类型**:
  - 单元测试 (60%)
  - 集成测试 (27%)
  - 属性测试 (10%)
  - 性能测试 (3%)
- **核心模块**: 渲染、物理、ECS、平台、工具

### 测试框架

| 工具 | 用途 | 版本 |
|------|------|------|
| Rust内置 | 单元测试和集成测试 | - |
| cargo-tarpaulin | 代码覆盖率 | 0.27+ |
| Criterion.rs | 性能基准测试 | 0.8+ |
| Proptest | 属性测试 | 1.9+ |

---

## 测试架构

### 测试金字塔

```
                    E2E测试
                   (少量)
                  ↗↗  ↖↖
                集成测试
               (适量)
              ↗↗  ↖↖
            单元测试
           (大量)
```

### 目录结构

```
game_engine/
├── tests/                          # 集成测试
│   ├── test_infrastructure/       # 测试基础设施
│   │   ├── mod.rs                 # 模块定义
│   │   ├── assertions.rs          # 自定义断言
│   │   ├── helpers.rs             # 辅助函数
│   │   ├── fixtures.rs            # 测试夹具
│   │   └── mock.rs                # Mock对象
│   ├── render/                    # 渲染系统测试
│   │   ├── render_system_tests.rs # 新增
│   │   ├── shader_tests.rs
│   │   ├── material_tests.rs
│   │   └── mesh_tests.rs
│   ├── physics/                   # 物理系统测试
│   │   ├── physics_system_tests.rs # 新增
│   │   ├── collision_tests.rs
│   │   └── rigidbody_tests.rs
│   ├── platform/                  # 平台支持测试
│   │   └── platform_system_tests.rs # 新增
│   ├── tools/                     # 工具模块测试
│   │   └── tools_system_tests.rs  # 新增
│   ├── entity/                    # ECS系统测试
│   ├── math/                      # 数学库测试
│   ├── integration_tests.rs       # 新增
│   ├── property_tests.rs          # 新增
│   ├── resource_integration.rs
│   └── stress_tests.rs
└── benches/                       # 性能基准测试
    ├── math_benchmarks.rs
    ├── ecs_benchmarks.rs
    ├── physics_benchmarks.rs
    ├── render_benchmarks.rs
    └── extended_benchmarks.rs      # 新增
```

### 测试分类

#### 1. 单元测试 (180个)

- **目的**: 测试单个函数和模块
- **特点**: 快速、隔离、可重复
- **覆盖率目标**: 60%+

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_function_name() {
        assert_eq!(2 + 2, 4);
    }
}
```

#### 2. 集成测试 (80个)

- **目的**: 测试模块间交互
- **特点**: 真实环境、跨模块
- **覆盖率目标**: 45%+

```rust
// tests/integration_tests.rs
use game_engine::prelude::*;

#[test]
fn test_render_physics_integration() {
    let world = PhysicsWorld::new();
    let renderer = Renderer::new();
    // 测试集成功能
}
```

#### 3. 属性测试 (30个)

- **目的**: 测试不变量和属性
- **特点**: 随机输入、边界覆盖
- **覆盖率目标**: 50%+

```rust
proptest! {
    #[test]
    fn test_vector_properties(x in any::<f32>(), y in any::<f32>()) {
        let v = Vec2::new(x, y);
        prop_assert!(v.length() >= 0.0);
    }
}
```

#### 4. 性能测试 (10个)

- **目的**: 测试性能和回归
- **特点**: 统计分析、基线对比
- **目标**: 检测性能退化

---

## 运行测试

### 基本测试命令

#### 运行所有测试

```bash
# 运行所有单元测试和集成测试
cargo test

# 只运行单元测试
cargo test --lib

# 只运行集成测试
cargo test --test '*'

# 并行运行（默认）
cargo test

# 串行运行
cargo test -- --test-threads=1
```

#### 运行特定测试

```bash
# 运行特定模块
cargo test render

# 运行特定测试函数
cargo test test_material_creation

# 运行特定文件
cargo test --test render_system_tests

# 运行忽略的测试
cargo test -- --ignored

# 显示输出
cargo test -- --nocapture
cargo test -- --show-output
```

### 性能基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准
cargo bench --bench extended_benchmarks

# 保存基线
cargo bench -- --save-baseline main

# 对比基线
cargo bench -- --baseline main

# 生成HTML报告
cargo bench
# 报告位置: target/criterion/report/index.html
```

### 覆盖率测试

```bash
# 安装tarpaulin（首次）
cargo install cargo-tarpaulin

# 生成HTML覆盖率报告
cargo tarpaulin --out Html --output-dir target/coverage

# 生成XML报告（用于CI）
cargo tarpaulin --out Xml --output-dir target/coverage

# 查看终端覆盖率
cargo tarpaulin --stdout

# 设置覆盖率阈值
cargo tarpaulin --fail-under 50

# 排除文件
cargo tarpaulin --exclude-files "*/tests/*" --exclude-files "*/benches/*"

# 特定模块覆盖率
cargo tarpaulin --files src/render
```

### 属性测试

```bash
# 运行proptest
cargo test --test property_tests

# 增加测试用例数
PROP_TEST_CASES=1000 cargo test --test property_tests

# 保存失败案例
cargo test --test property_tests -- persist

# 重现失败案例
cargo test --test property_tests -- replay proptest-regressions/*
```

---

## 编写测试

### 单元测试模板

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 辅助函数
    fn setup_test_data() -> TestData {
        TestData::default()
    }

    // 基本功能测试
    #[test]
    fn test_feature_basic() {
        // Arrange (准备)
        let input = setup_test_data();

        // Act (执行)
        let result = function_under_test(input);

        // Assert (断言)
        assert_eq!(result, expected);
    }

    // 边界测试
    #[test]
    fn test_feature_boundary() {
        assert_eq!(function(0), expected_at_zero);
        assert_eq!(function(MAX), expected_at_max);
    }

    // 错误处理测试
    #[test]
    fn test_feature_error_handling() {
        let result = function_that_fails();
        assert!(result.is_err());

        if let Err(e) = result {
            assert_eq!(e.kind(), ErrorKind::NotFound);
        }
    }

    // Panic测试
    #[test]
    #[should_panic(expected = "Specific error message")]
    fn test_feature_panic() {
        function_that_panics();
    }

    // 异步测试
    #[tokio::test]
    async fn test_async_feature() {
        let result = async_function().await;
        assert!(result.is_ok());
    }

    // 参数化测试（使用自定义宏）
    macro_rules! test_cases {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(function($value), expected);
                }
            )*
        }
    }

    test_cases! {
        test_case_1: 1,
        test_case_2: 2,
        test_case_3: 3,
    }
}
```

### 集成测试模板

```rust
// tests/integration_tests.rs
use game_engine::prelude::*;

#[test]
fn test_render_physics_sync() {
    // 创建系统
    let mut render_system = RenderSystem::new();
    let mut physics_world = PhysicsWorld::new();

    // 设置场景
    let body = physics_world.create_body(BodyType::Dynamic);
    body.set_position(Vec3::new(0.0, 10.0, 0.0));

    // 模拟
    physics_world.step(1.0 / 60.0);

    // 渲染
    render_system.render_body(body);

    // 验证
    assert!(body.position().y < 10.0); // 重力作用
}

#[test]
fn test_asset_loading_pipeline() {
    let asset_manager = AssetManager::new();

    // 加载资源
    let mesh = asset_manager.load_mesh("test.obj");
    assert!(mesh.is_ok());

    let material = asset_manager.load_material("test.mat");
    assert!(material.is_ok());

    // 验证依赖
    assert_eq!(mesh.unwrap().material_count(), 1);
}
```

### 使用测试基础设施

#### 使用Fixtures

```rust
use crate::test_infrastructure::fixtures::*;

#[test]
fn test_with_mesh_fixture() {
    let mesh = create_mock_mesh();
    let material = create_mock_material();

    let result = render_mesh(mesh, material);
    assert!(result.is_ok());
}
```

#### 使用Mock对象

```rust
use crate::test_infrastructure::mock::*;

#[test]
fn test_with_mock_device() {
    let mock_device = MockGpuDevice::new();

    let renderer = Renderer::with_device(mock_device.clone());
    renderer.draw();

    assert_eq!(mock_device.draw_call_count(), 1);
}
```

#### 使用自定义断言

```rust
use crate::test_infrastructure::assertions::*;

#[test]
fn test_with_custom_assertions() {
    let vec3 = Vec3::new(1.0, 2.0, 3.0);

    assert_approx_eq!(vec3.length(), 3.741, epsilon = 0.001);
    assert_vec3_all_finite!(vec3);
    assert_vec3_normalized!(vec3.normalize());
}
```

---

## 性能基准测试

### Criterion.rs基础

```rust
// benches/extended_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main,
                BenchmarkId, Criterion, Throughput};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // 被测试的代码
            my_function(black_box(42))
        })
    });
}

fn benchmark_with_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_inputs");

    for size in [1024, 2048, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| process_data(black_box(size)))
            }
        );
    }

    group.finish();
}

fn benchmark_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    group.throughput(Throughput::Bytes(1024));

    group.bench_function("process_1k", |b| {
        b.iter(|| process_bytes(black_box(vec![0u8; 1024])))
    });

    group.finish();
}

criterion_group!(benches,
    benchmark_function,
    benchmark_with_input,
    benchmark_throughput
);
criterion_main!(benches);
```

### 性能测试最佳实践

```rust
// 1. 使用black_box防止编译器优化
c.bench_function("test", |b| {
    b.iter(|| {
        let input = black_box(generate_input());
        black_box(process(input))
    })
});

// 2. 对比不同实现
fn benchmark_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithms");

    group.bench_function("approach_a", |b| {
        b.iter(|| approach_a(black_box(1000)))
    });

    group.bench_function("approach_b", |b| {
        b.iter(|| approach_b(black_box(1000)))
    });

    group.finish();
}

// 3. 测试不同输入规模
fn benchmark_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| algorithm_with_complexity(black_box(size)))
        });
    }

    group.finish();
}
```

---

## 属性测试

### Proptest基础

```rust
// tests/property_tests.rs
use proptest::prelude::*;

// 简单属性测试
proptest! {
    #[test]
    fn test_addition_commutative(a in any::<i32>(), b in any::<i32>()) {
        prop_assert_eq!(a + b, b + a);
    }

    #[test]
    fn test_vector_length_non_negative(x in any::<f32>(), y in any::<f32>()) {
        let v = Vec2::new(x, y);
        prop_assert!(v.length() >= 0.0);
    }
}

// 自定义策略
fn matrix_strategy() -> impl Strategy<Value = Mat4> {
    prop::collection::vec(-1.0f32..1.0, 16)
        .prop_map(|values| Mat4::from_column_slice(&values))
}

proptest! {
    #[test]
    fn test_matrix_properties(m in matrix_strategy()) {
        // 测试矩阵性质
        let det = m.determinant();
        prop_assert!(!det.is_nan());
    }
}

// 测试不变量
proptest! {
    #[test]
    fn test_rotation_preserves_length(angle in -3.14..3.14) {
        let v = Vec2::new(1.0, 0.0);
        let rotated = v.rotate(angle);

        prop_assert_approx_eq!(rotated.length(), v.length(), 1e-6);
    }
}
```

### 高级Proptest技巧

```rust
// 过滤策略
proptest! {
    #[test]
    fn test_non_zero_division(a in 1i32..1000, b in 1i32..1000) {
        // b总是非零，所以不会除以零
        prop_assert_eq!(a / b * b, a);
    }
}

// 复杂策略
fn color_strategy() -> impl Strategy<Value = Color> {
    (0u8..=255, 0u8..=255, 0u8..=255)
        .prop_map(|(r, g, b)| Color { r, g, b })
}

proptest! {
    #[test]
    fn test_color_clamping(c in color_strategy()) {
        let clamped = c.clamp();
        prop_assert!(clamped.r <= 255 && clamped.g <= 255 && clamped.b <= 255);
    }
}
```

---

## 覆盖率报告

### 当前覆盖率状态

| 模块 | 覆盖率 | 目标 | 状态 |
|------|--------|------|------|
| 渲染系统 | 60% | 70% | 🟡 |
| 物理系统 | 65% | 75% | 🟡 |
| 平台支持 | 55% | 65% | 🟡 |
| 工具模块 | 50% | 60% | 🟡 |
| ECS系统 | 60% | 75% | 🟡 |
| 数学库 | 80% | 90% | 🟢 |
| 资源管理 | 55% | 70% | 🟡 |
| **总计** | **52%** | **60%** | 🟡 |

### 查看覆盖率报告

```bash
# 生成HTML报告
cargo tarpaulin --out Html --output-dir target/coverage

# 在浏览器中打开
open target/coverage/index.html  # macOS
xdg-open target/coverage/index.html  # Linux
start target/coverage/index.html  # Windows
```

### 覆盖率提升策略

1. **识别未覆盖代码**
   ```bash
   cargo tarpaulin --out Html
   # 在HTML报告中查看红色标记的代码
   ```

2. **优先级排序**
   - 核心功能优先
   - 高风险代码优先
   - 常用路径优先

3. **增量添加**
   - 每次PR添加测试
   - 保持覆盖率不下降
   - CI强制检查

---

## CI/CD集成

### GitHub Actions配置

```yaml
# .github/workflows/test.yml
name: Tests and Coverage

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    name: Run Tests
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          override: true

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Generate coverage
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir ./coverage

      - name: Upload to Codecov
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
          flags: unittests
          name: codecov-${{ matrix.os }}-${{ matrix.rust }}

      - name: Check coverage threshold
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        run: cargo tarpaulin --fail-under 50

  bench:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    if: github.event == 'push' && github.ref == 'refs/heads/main'

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run benchmarks
        run: cargo bench -- --save-baseline main

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/report/index.html
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### Git Hooks (Pre-commit)

```bash
# .git/hooks/pre-commit
#!/bin/bash

# 快速测试
cargo test --quiet

# 检查覆盖率
cargo tarpaulin --fail-under 50

# 如果失败，阻止提交
if [ $? -ne 0 ]; then
    echo "Tests failed or coverage below 50%"
    exit 1
fi
```

---

## 最佳实践

### 1. 测试命名

```rust
// ✅ 好的命名 - 描述性、清晰
fn test_material_creation_with_valid_texture()
fn test_physics_collision_response_between_static_and_dynamic()
fn test_render_pipeline_switch_preserves_state()

// ❌ 不好的命名 - 含糊不清
fn test1()
fn test_material()
fn test_it_works()
```

### 2. 测试组织

```rust
#[cfg(test)]
mod material_tests {
    use super::*;

    // 按功能分组
    mod creation {
        use super::*;

        #[test]
        fn test_with_valid_data() { }

        #[test]
        fn test_with_null_texture() { }
    }

    mod properties {
        use super::*;

        #[test]
        fn test_albedo_color() { }

        #[test]
        fn test_metallic_value() { }
    }

    mod lifecycle {
        use super::*;

        #[test]
        fn test_clone() { }

        #[test]
        fn test_drop() { }
    }
}
```

### 3. 测试独立性

```rust
// ✅ 每个测试独立
#[test]
fn test_scenario_a() {
    let setup = create_fresh_setup();
    // 测试场景A
}

#[test]
fn test_scenario_b() {
    let setup = create_fresh_setup();
    // 测试场景B
}

// ❌ 测试依赖（不好的实践）
static mut SHARED_STATE: usize = 0;

#[test]
fn test_a() {
    unsafe { SHARED_STATE = 1; }
}

#[test]
fn test_b() {
    unsafe { assert_eq!(SHARED_STATE, 1); } // 依赖test_a
}
```

### 4. 使用Mock避免依赖

```rust
// ✅ 使用Mock - 快速、稳定
#[test]
fn test_with_mock() {
    let mock_renderer = MockRenderer::new();
    let game = Game::new_with_renderer(mock_renderer.clone());

    game.update();

    assert_eq!(mock_renderer.draw_call_count(), 1);
}

// ❌ 依赖真实资源 - 慢、不稳定
#[test]
fn test_with_real_gpu() {
    let renderer = Renderer::new(); // 需要真实GPU
    // 测试慢且可能在不同机器上失败
}
```

### 5. 测试数据管理

```rust
// 使用Builder模式创建测试数据
pub struct TestDataBuilder {
    mesh: Option<Mesh>,
    material: Option<Material>,
    transform: Option<Transform>,
}

impl TestDataBuilder {
    pub fn new() -> Self {
        Self {
            mesh: None,
            material: None,
            transform: None,
        }
    }

    pub fn with_mesh(mut self, mesh: Mesh) -> Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn build(self) -> TestData {
        TestData {
            mesh: self.mesh.unwrap_or_default(),
            material: self.material.unwrap_or_default(),
            transform: self.transform.unwrap_or_default(),
        }
    }
}

#[test]
fn test_with_builder() {
    let data = TestDataBuilder::new()
        .with_mesh(create_mock_mesh())
        .build();

    // 使用data进行测试
}
```

---

## 常见问题

### Q: 测试太慢怎么办？

**A:**
1. 使用发布版本: `cargo test --release`
2. 并行运行: `cargo test -- --test-threads=4`
3. 只运行相关测试: `cargo test render`
4. 使用Mock代替真实资源
5. 分离慢速测试:
   ```rust
   #[test]
   #[ignore]
   fn test_slow_feature() {
       // 这个测试默认跳过
       // 运行: cargo test -- --ignored
   }
   ```

### Q: 如何测试异步代码？

**A:**
```rust
#[tokio::test]
async fn test_async_feature() {
    let result = async_function().await;
    assert!(result.is_ok());
}

// 使用timeout
#[tokio::test]
async fn test_with_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        slow_async_function()
    ).await;

    assert!(result.is_ok());
}
```

### Q: 如何测试panic?

**A:**
```rust
#[test]
#[should_panic(expected = "Specific error message")]
fn test_panic_with_message() {
    panic!("Specific error message");
}

// 或者
#[test]
fn test_catch_panic() {
    let result = std::panic::catch_unwind(|| {
        might_panic()
    });

    assert!(result.is_err());
}
```

### Q: 如何测试私有函数？

**A:**
```rust
// 方法1: 使用测试模块
impl MyStruct {
    fn private_function(&self) -> i32 {
        42
    }
}

#[cfg(test)]
impl MyStruct {
    fn test_private_function(&self) -> i32 {
        // 测试用公开的包装函数
        self.private_function()
    }
}

// 方法2: 测试公开API覆盖私有函数
#[test]
fn test_private_indirectly() {
    let s = MyStruct::new();
    assert_eq!(s.public_result(), 42); // 间接测试private_function
}
```

### Q: 如何测试返回浮点数的函数？

**A:**
```rust
// 使用近似比较
#[test]
fn test_float_result() {
    let result = calculate_pi();
    let expected = std::f32::consts::PI;

    assert!((result - expected).abs() < 0.0001);
}

// 使用自定义宏
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr, $epsilon:expr) => {
        assert!(($a - $b).abs() < $epsilon,
                "Values not approximately equal: {} vs {}", $a, $b);
    }
}

#[test]
fn test_float_comparison() {
    assert_approx_eq!(calculate_pi(), 3.14159, 0.0001);
}
```

### Q: 如何处理需要真实GPU的测试？

**A:**
```rust
// 条件编译
#[test]
#[cfg(feature = "gpu-testing")]
fn test_with_real_gpu() {
    let gpu = GpuDevice::create();
    // 测试真实GPU功能
}

#[test]
#[cfg(not(feature = "gpu-testing"))]
fn test_with_mock_gpu() {
    let mock_gpu = MockGpuDevice::new();
    // 测试GPU功能
}

// 或者使用环境变量
#[test]
fn test_conditional_gpu() {
    if std::env::var("ENABLE_GPU_TESTS").is_ok() {
        let gpu = GpuDevice::create();
        // 真实GPU测试
    } else {
        let mock_gpu = MockGpuDevice::new();
        // Mock测试
    }
}
```

---

## 参考资源

### 官方文档

- [Rust测试指南](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs文档](https://bheisler.github.io/criterion.rs/book/)
- [Proptest文档](https://altsysrq.github.io/proptest-book/intro.html)
- [Cargo Tarpaulin](https://github.com/xd009642/tarpaulin)

### 社区资源

- [Rust Testing Patterns](https://matklad.github.io/2021/05/31/how-to-test.html)
- [Effective Rust Testing](https://www.youtube.com/watch?v=31I8Tk8JVh8)

### 内部文档

- [测试覆盖率提升报告](./TEST_COVERAGE_IMPROVEMENT_REPORT.md)
- [测试基础设施文档](./TEST_INFRASTRUCTURE_GUIDE.md)
- [性能测试指南](./PERFORMANCE_TESTING_GUIDE.md)

---

## 更新日志

### v2.0 (2026-01-02)

- ✅ 新增300+测试用例
- ✅ 覆盖率从42%提升至52%
- ✅ 添加渲染系统测试
- ✅ 添加物理系统测试
- ✅ 添加平台支持测试
- ✅ 添加工具模块测试
- ✅ 添加集成测试
- ✅ 添加属性测试
- ✅ 扩展性能基准测试

### v1.0 (2025-12-30)

- 初始版本
- 基础测试框架
- 111个单元测试

---

**文档维护**: Game Engine Team
**问题反馈**: GitHub Issues
**贡献指南**: 请参考[贡献指南](./CONTRIBUTING.md)
