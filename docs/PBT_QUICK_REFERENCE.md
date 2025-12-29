# Property-Based Testing 快速参考卡

## 常用命令

```bash
# 运行所有属性测试
cargo test --test property_tests

# 运行特定模块
cargo test --test ecs_properties
cargo test --test physics_properties
cargo test --test network_properties
cargo test --test resources_properties
cargo test --test math_properties

# 运行单个测试
cargo test --test ecs_properties test_entity_id_stability

# 详细输出
cargo test --test property_tests -- --nocapture

# 统计测试数量
grep -r "proptest!" game_engine/tests/*properties*.rs | wc -l
```

## 基本模板

### 简单属性测试
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_something(a in 0i32..100i32, b in 0i32..100i32) {
        prop_assert!(a + b >= a);
        prop_assert_eq!(a + b, b + a);
    }
}
```

### 自定义策略
```rust
fn my_strategy() -> impl Strategy<Value = MyType> {
    prop::collection::vec(0u8..255u8, 10..100)
        .prop_map(|vec| MyType::new(vec))
}

proptest! {
    #[test]
    fn test_with_custom_strategy(input in my_strategy()) {
        prop_assert!(input.is_valid());
    }
}
```

## 常见测试模式

### 往返一致性
```rust
proptest! {
    #[test]
    fn test_roundtrip(data in any::<Vec<u8>>()) {
        let serialized = bincode::serialize(&data).unwrap();
        let deserialized: Vec<u8> = bincode::deserialize(&serialized).unwrap();
        prop_assert_eq!(data, deserialized);
    }
}
```

### 幂等性
```rust
proptest! {
    #[test]
    fn test_idempotent(value in 0.0f32..100.0f32) {
        let result1 = operation(value);
        let result2 = operation(result1);
        prop_assert!(approx_eq(result1, result2, 0.001));
    }
}
```

### 不变量
```rust
proptest! {
    #[test]
    fn test_invariant(vec in vec3()) {
        let normalized = vec.normalize();
        prop_assert!(approx_eq(normalized.length(), 1.0, 0.001));
    }
}
```

## 内置策略

```rust
// 基础类型
0i32..100i32                          // 整数范围
0.0f32..100.0f32                      // 浮点数范围
"[a-zA-Z]{1,50}"                      // 正则字符串

// 集合
prop::collection::vec(0i32..100i32, 10..100)           // Vec
prop::collection::hash_set("[a-z]{1,10}", 5..20)       // HashSet

// 数组
prop::array::uniform3(0.0f32..1.0f32)                  // [f32; 3]
prop::array::uniform4(0u8..255u8)                      // [u8; 4]

// 组合
(0i32..100i32, 0.0f32..1.0f32)                         // 元组

// 枚举
prop_oneof![
    Just(MyEnum::A),
    Just(MyEnum::B),
]

// 选项
prop::option::bool(0.0f32..1.0f32)                     // Option<f32>
prop::option::weighted(0.9, 0i32..100i32, 0.1, any::<()>())  // 90% Some
```

## 策略组合

### prop_map - 转换
```rust
fn vec3() -> impl Strategy<Value = Vec3> {
    prop::array::uniform3(0.0f32..100.0f32)
        .prop_map(|arr| Vec3::new(arr[0], arr[1], arr[2]))
}
```

### prop_filter - 过滤
```rust
fn non_zero() -> impl Strategy<Value = f32> {
    (0.0f32..100.0f32)
        .prop_filter("non-zero", |&x| x.abs() > 0.001)
}
```

### prop_flat_map - 展平映射
```rust
fn list_of_pairs() -> impl Strategy<Value<Vec<(i32, i32)>>> {
    prop::collection::vec(0i32..10i32, 1..10)
        .prop_flat_map(|vec| {
            prop::collection::vec((Just(vec), 0i32..10i32), vec.len()..vec.len())
        })
}
```

## 配置选项

### 在测试中配置
```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,                          // 测试用例数
        max_shrink_iters: 1000,               // 收缩迭代次数
        timeout: 30,                          // 超时（秒）
        .. ProptestConfig::default()
    })]

    #[test]
    fn test_configured(x in 0i32..100i32) {
        // ...
    }
}
```

### 全局配置 (proptest.toml)
```toml
test = 256
max_shrink_iters = 1000
fork = true
timeout = 60
```

## 调试技巧

### 查看生成的值
```rust
proptest! {
    #[test]
    fn test_debug(x in 0i32..100i32) {
        println!("Generated: {}", x);  // 查看生成值
        prop_assert!(x > 0);
    }
}
```

### 重放失败案例
```bash
# proptest会打印失败案例的命令
# 跟随该命令重放
cargo test test_function_name -- --exact
```

### 限制案例数
```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,  // 只运行10个案例用于调试
        .. ProptestConfig::default()
    })]
    // ...
}
```

## 浮点数比较

```rust
// 错误方式
prop_assert_eq!(a, b);

// 正确方式
fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

prop_assert!(approx_eq(a, b, 0.001));
```

## 常见错误

### 生成太多无效值
```rust
// 不好
fn non_zero() -> impl Strategy<Value = f32> {
    (-1000.0..1000.0).prop_filter("non-zero", |x| x.abs() > 0.001)
}

// 好
fn non_zero() -> impl Strategy<Value = f32> {
    0.001..1000.0  // 直接排除0
}
```

### 测试太慢
```rust
// 使用较少的案例
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,  // 减少到100
        .. ProptestConfig::default()
    })]
    // ...
}
```

### 忘记处理错误
```rust
// 错误
let serialized = bincode::serialize(&data).unwrap();

// 正确
let serialized = bincode::serialize(&data)?;
```

## 模块特定策略

### ECS模块
```rust
use game_engine::ecs::*;

// Entity ID
entity_index()

// Transform
(vec3(), quat(), vec3_small())

// Velocity
(vec3(), vec3())
```

### Physics模块
```rust
use game_engine::physics::*;

// 刚体
(mass(), vec3(), quat())

// 碰撞体
radius()  // 球体
vec3_small()  // 立方体半尺寸
```

### Network模块
```rust
// 数据包
prop::collection::vec(0u8..255u8, 100..10000)

// 时间戳
time_step()
```

### Resources模块
```rust
// 资源路径
"[a-zA-Z0-9_/]{1,100}"

// 缓存容量
10usize..1000usize
```

### Math模块
```rust
use glam::*;

// 向量
vec3()
vec3_normalized()

// 四元数
(-1.0..1.0, -1.0..1.0, -1.0..1.0, -1.0..1.0)

// 矩阵
(Mat4::IDENTITY, Mat4::IDENTITY)
```

## 参考资源

- [proptest文档](https://altsysrq.github.io/proptest-book/proptest/getting-started.html)
- [项目使用指南](./PBT_USAGE.md)
- [实施总结](../PBT_IMPLEMENTATION_SUMMARY.md)

---

**快速提示**: 遇到问题时，先用简单的测试验证框架，再逐步增加复杂性！
