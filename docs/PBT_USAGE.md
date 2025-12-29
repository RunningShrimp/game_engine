# Property-Based Testing (PBT) 使用指南

## 概述

Property-Based Testing (PBT) 是一种自动生成大量随机测试用例来验证通用属性的测试方法。本项目使用 `proptest` 库为关键模块添加了属性测试。

## 什么是Property-Based Testing？

### 与传统测试的对比

**传统测试（Example-Based Testing）**:
```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);  // 手工编写的具体测试用例
}
```

**属性测试（Property-Based Testing）**:
```rust
proptest! {
    #[test]
    fn test_addition_commutative(a in 0i32..1000i32, b in 0i32..1000i32) {
        // 自动生成数百个测试用例
        prop_assert_eq!(a + b, b + a);  // 验证交换律
    }
}
```

### PBT的优势

1. **自动发现边界情况**: 随机生成能发现手工测试遗漏的边界值
2. **快速失败**: 发现bug时自动缩小到最小失败案例
3. **回归测试**: 失败案例保存到文件，持续验证
4. **文档作用**: 属性就是函数的规范文档

## 项目结构

```
game_engine/
├── proptest.toml                    # PBT配置文件
├── game_engine/
│   └── tests/
│       ├── property_tests.rs        # PBT主入口和策略定义
│       ├── ecs_properties.rs        # ECS模块属性测试
│       ├── physics_properties.rs    # Physics模块属性测试
│       ├── network_properties.rs    # Network模块属性测试
│       ├── resources_properties.rs  # Resources模块属性测试
│       ├── math_properties.rs       # Math模块属性测试
│       └── pbt_simple_test.rs       # 简单示例
```

## 配置文件

### `proptest.toml`

```toml
# 失败案例保存位置
casemap = "proptest-regressions"

# 测试用例数量（默认256个）
test = 256

# 失败后重试次数
max_shrink_iters = 1000

# 在子进程中运行测试（提高稳定性）
fork = true

# 单个测试超时时间（秒）
timeout = 60
```

## 快速开始

### 1. 运行所有属性测试

```bash
# 运行所有属性测试
cargo test --test property_tests

# 运行特定模块的测试
cargo test --test ecs_properties

# 运行单个测试
cargo test --test ecs_properties test_entity_id_stability

# 显示输出
cargo test --test property_tests -- --nocapture
```

### 2. 编写第一个属性测试

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_vector_addition_commutative(
        x in 0.0f32..100.0f32,
        y in 0.0f32..100.0f32,
        z in 0.0f32..100.0f32
    ) {
        use glam::Vec3;

        let vec1 = Vec3::new(x, y, z);
        let vec2 = Vec3::new(1.0, 2.0, 3.0);

        // 验证交换律
        let sum1 = vec1 + vec2;
        let sum2 = vec2 + vec1;

        prop_assert!(vec3_approx_eq(sum1, sum2, 0.001));
    }
}
```

## 测试策略（Strategies）

### 基础策略

```rust
// 坐标范围
fn coord() -> impl Strategy<Value = f32> {
    -1000.0..=1000.0f32
}

// 3D向量
fn vec3() -> impl Strategy<Value = Vec3> {
    prop::array::uniform3(coord())
}

// 颜色（RGBA）
fn color() -> impl Strategy<Value = [f32; 4]> {
    prop::array::uniform4(0.0..=1.0f32)
}

// 正整数
fn usize_small() -> impl Strategy<Value = usize> {
    0usize..1000
}
```

### 组合策略

```rust
// 字符串策略
fn non_empty_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,50}"
}

// 集合策略
fn vec_of_vectors() -> impl Strategy<Value = Vec<Vec3>> {
    prop::collection::vec(vec3(), 10..100)
}

// 枚举策略
fn body_type() -> impl Strategy<Value = RigidBodyType> {
    prop_oneof![
        Just(RigidBodyType::Dynamic),
        Just(RigidBodyType::Static),
        Just(RigidBodyType::Kinematic),
    ]
}
```

### 过滤和映射

```rust
// 单位向量（过滤零向量）
fn vec3_normalized() -> impl Strategy<Value = Vec3> {
    vec3().prop_filter("vector too close to zero", |v| {
        v.length() > 0.001
    }).prop_map(|v| v.normalize())
}

// 对称矩阵
fn symmetric_matrix() -> impl Strategy<Value = [[f32; 3]; 3]> {
    // 生成并验证对称性
}
```

## 常见测试模式

### 1. 往返一致性（Roundtrip）

```rust
proptest! {
    #[test]
    fn test_serialization_roundtrip(vec in vec3()) {
        // 序列化
        let serialized = bincode::serialize(&vec).unwrap();

        // 反序列化
        let deserialized: Vec3 = bincode::deserialize(&serialized).unwrap();

        // 验证一致性
        prop_assert_eq!(vec, deserialized);
    }
}
```

### 2. 幂等性（Idempotence）

```rust
proptest! {
    #[test]
    fn test_normalize_idempotent(vec in vec3()) {
        if vec.length() < 0.001 { return Ok(()); }

        let n1 = vec.normalize();
        let n2 = n1.normalize();

        prop_assert!(vec3_approx_eq(n1, n2, 0.001));
    }
}
```

### 3. 不变量（Invariants）

```rust
proptest! {
    #[test]
    fn test_quaternion_normalized(unit_vec in vec3_normalized()) {
        // 归一化的四元数长度始终为1
        let quat = Quat::from_axis_angle(unit_vec, 1.0);
        let length = quat.length();

        prop_assert!(approx_eq(length, 1.0, 0.001));
    }
}
```

### 4. 对称性（Symmetry）

```rust
proptest! {
    #[test]
    fn test_distance_symmetric(a in vec3(), b in vec3()) {
        let dist_ab = a.distance(b);
        let dist_ba = b.distance(a);

        prop_assert_eq!(dist_ab, dist_ba);
    }
}
```

### 5. 结合律（Associativity）

```rust
proptest! {
    #[test]
    fn test_matrix_associativity(m1 in matrix(), m2 in matrix(), m3 in matrix()) {
        let left = (m1 * m2) * m3;
        let right = m1 * (m2 * m3);

        prop_assert!(left.approx_eq(&right, 0.001));
    }
}
```

## 各模块测试属性

### ECS模块 (`ecs_properties.rs`)

**测试的属性**:
- Entity ID唯一性和稳定性
- Transform数据保持性
- Query计数准确性
- Entity生命周期
- TileEntityPool容量限制

**示例**:
```bash
cargo test --test ecs_properties
```

### Physics模块 (`physics_properties.rs`)

**测试的属性**:
- RigidBody质量为正
- 位置更新遵循物理定律
- 碰撞检测对称性
- 空间分区查询一致性
- 物理同步往返

**示例**:
```bash
cargo test --test physics_properties test_rigid_body_mass_positive
```

### Network模块 (`network_properties.rs`)

**测试的属性**:
- Delta编码往返一致性
- 压缩/解压缩数据完整性
- 消息序列化往返
- 插值单调性
- 状态更新幂等性

**示例**:
```bash
cargo test --test network_properties test_delta_encode_decode_roundtrip
```

### Resources模块 (`resources_properties.rs`)

**测试的属性**:
- LRU缓存容量限制
- 内存池分配回收
- 环形Buffer读写一致性
- 缓存命中率边界

**示例**:
```bash
cargo test --test resources_properties test_lru_capacity_limit
```

### Math模块 (`math_properties.rs`)

**测试的属性**:
- 向量运算的线性代数性质
- 四元数乘法结合律
- 矩阵运算性质
- 插值边界条件
- AABB几何性质

**示例**:
```bash
cargo test --test math_properties test_vec3_addition_commutative
```

## 调试技巧

### 1. 查看生成的测试用例

```bash
# 启用详细输出
cargo test --test property_tests -- --nocapture

# 设置环境变量显示生成的值
RUST_LOG=proptest=debug cargo test --test property_tests
```

### 2. 重现失败案例

proptest会自动将失败案例保存到 `proptest-regressions/` 目录：

```toml
# proptest-regressions/ecs_properties/test_entity_id_stability.txt
# LCS 2025-12-28: cases for test_entity_id_stability
hash: 1234567890
casemap: [
    EntityId(42),
    EntityId(43),
    # ... 更多失败案例
]
```

### 3. 限制测试用例数量

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,  // 只运行100个案例
        .. ProptestConfig::default()
    })]

    #[test]
    fn test_something(x in 0usize..1000usize) {
        // ...
    }
}
```

### 4. 处理浮点数误差

```rust
// 不要这样
prop_assert_eq!(a, b);

// 应该这样
prop_assert!(approx_eq(a, b, 0.001));
prop_assert!(vec3_approx_eq(v1, v2, 0.001));
```

## 最佳实践

### 1. 选择有意义的属性

✅ **好的属性**:
- 交换律、结合律等数学性质
- 往返一致性（序列化/反序列化）
- 不变量（归一化后长度为1）
- 边界条件（插值在0-1之间）

❌ **避免**:
- 测试具体值（用传统测试）
- 测试实现细节
- 太复杂的属性

### 2. 合理设置范围

```rust
// 太宽泛 - 生成太多无效案例
fn coord() -> impl Strategy<Value = f32> {
    f32::MIN..f32::MAX
}

// 合理的范围
fn coord() -> impl Strategy<Value = f32> {
    -1000.0..=1000.0f32
}
```

### 3. 过滤无效输入

```rust
proptest! {
    #[test]
    fn test_normalize(vec in vec3()) {
        // 提前返回
        if vec.length() < 0.001 {
            return Ok(());
        }

        let normalized = vec.normalize();
        prop_assert!(approx_eq(normalized.length(), 1.0, 0.001));
    }
}
```

### 4. 使用策略组合

```rust
// 创建可复用的策略
mod strategies {
    use super::*;

    pub fn vec3() -> impl Strategy<Value = Vec3> {
        prop::array::uniform3(coord())
    }

    pub fn transform() -> impl Strategy<Value = Transform> {
        (vec3(), quat(), vec3_small()).prop_map(|(pos, rot, scale)| {
            Transform { pos, rot, scale }
        })
    }
}
```

## 性能考虑

### 优化策略

```rust
// 缓存复杂的策略
lazy_static! {
    static ref COMPLEX_STRATEGY: Arc<dyn Strategy<Value = ComplexType>> = {
        Arc::new(complex_strategy())
    };
}

// 使用prop_sample!减少运行次数
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 50,  // 减少案例数
        .. ProptestConfig::default()
    })]

    #[test]
    fn test_expensive_operation(x in complex_strategy()) {
        // ...
    }
}
```

## 故障排除

### 问题1: 编译错误

```
error[E0433]: failed to resolve: use of undeclared crate or module `proptest`
```

**解决**: 确保 `proptest` 在 `dev-dependencies` 中：

```toml
[dev-dependencies]
proptest = "1.9"
```

### 问题2: 测试超时

**解决**: 调整 `proptest.toml` 中的超时设置：

```toml
timeout = 120  # 增加到120秒
```

### 问题3: 生成太多无效案例

**解决**: 使用 `prop_filter` 或调整策略范围：

```rust
fn non_zero() -> impl Strategy<Value = f32> {
    0.001f32..1000.0f32  // 排除0
}
```

## 扩展阅读

- [proptest官方文档](https://altsysrq.github.io/proptest-book/proptest/getting-started.html)
- [Property-Based Testing in Rust](https://rust-lang.github.io/rust-clippy/master/index.html)
- [The Rise of Property-Based Testing](https://www.youtube.com/watch?v=hN9xc9jvJX0)

## 贡献指南

添加新的属性测试时：

1. 在对应模块的 `*_properties.rs` 中添加测试
2. 使用 `property_tests.rs` 中定义的策略
3. 确保测试有清晰的文档注释
4. 运行 `cargo test` 验证
5. 提交时附带失败的案例分析（如果有）

## 统计数据

- **总代码行数**: 2722行
- **属性测试文件**: 6个
- **proptest测试块**: 33个
- **测试函数总数**: 106个
- **覆盖模块**: ECS, Physics, Network, Resources, Math

## 许可证

本测试框架遵循项目主许可证。
