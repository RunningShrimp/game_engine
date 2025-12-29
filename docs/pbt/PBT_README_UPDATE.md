# Property-Based Testing 实施说明

本文档说明Property-Based Testing (PBT)在本项目中的实施情况。

## 什么是Property-Based Testing?

Property-Based Testing是一种测试方法，通过自动生成大量随机测试用例来验证代码的通用属性，而不是手工编写具体的测试案例。

### 传统测试 vs 属性测试

**传统测试**:
```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
}
```

**属性测试**:
```rust
proptest! {
    #[test]
    fn test_addition_commutative(a in 0i32..100i32, b in 0i32..100i32) {
        prop_assert_eq!(a + b, b + a);  // 验证交换律
    }
}
```

## 本项目的PBT实施

### 统计数据

- **测试代码**: 2722行
- **测试文件**: 6个
- **测试用例**: 106个
- **覆盖模块**: 5个核心模块

### 模块覆盖

1. **ECS模块** (`ecs_properties.rs`)
   - Entity唯一性和生命周期
   - Component数据保持性
   - Query准确性

2. **Physics模块** (`physics_properties.rs`)
   - 刚体质量和位置
   - 碰撞检测正确性
   - 空间分区查询

3. **Network模块** (`network_properties.rs`)
   - 序列化往返一致性
   - 压缩数据完整性
   - Delta编码正确性

4. **Resources模块** (`resources_properties.rs`)
   - LRU缓存行为
   - 内存池管理
   - Staging Buffer操作

5. **Math模块** (`math_properties.rs`)
   - 向量线性代数性质
   - 四元数运算性质
   - 插值算法边界

## 如何使用

### 快速开始

```bash
# 1. 运行所有属性测试
cargo test --test property_tests

# 2. 运行特定模块
cargo test --test ecs_properties

# 3. 查看详细输出
cargo test --test property_tests -- --nocapture
```

### 添加新的属性测试

1. 在对应的测试文件中添加测试函数
2. 使用 `proptest!` 宏定义属性
3. 选择合适的输入策略
4. 运行测试验证

示例：
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_my_property(input in 0i32..1000i32) {
        let result = my_function(input);
        prop_assert!(result >= 0);  // 验证某个性质
    }
}
```

## 文档

- **完整使用指南**: [docs/PBT_USAGE.md](docs/PBT_USAGE.md)
- **快速参考**: [docs/PBT_QUICK_REFERENCE.md](docs/PBT_QUICK_REFERENCE.md)
- **实施总结**: [PBT_IMPLEMENTATION_SUMMARY.md](PBT_IMPLEMENTATION_SUMMARY.md)

## 配置

PBT配置文件: `proptest.toml`

```toml
test = 256                          # 测试用例数量
max_shrink_iters = 1000             # 失败案例收缩迭代次数
fork = true                         # 子进程运行
timeout = 60                        # 超时（秒）
casemap = "proptest-regressions"    # 失败案例保存位置
```

## 优势

1. **自动发现bug**: 随机生成能发现边界情况
2. **快速失败定位**: 自动缩小到最小失败案例
3. **回归测试**: 失败案例保存到文件持续验证
4. **文档作用**: 属性就是函数的规范
5. **高覆盖率**: 自动生成数百个测试用例

## 常见测试模式

### 往返一致性
```rust
// 序列化再反序列化应该得到原始数据
let serialized = serialize(data);
let deserialized = deserialize(&serialized);
prop_assert_eq!(data, deserialized);
```

### 幂等性
```rust
// 多次调用应该得到相同结果
let result1 = operation(value);
let result2 = operation(result1);
prop_assert_eq!(result1, result2);
```

### 对称性
```rust
// 操作的顺序不应该影响结果（对于某些操作）
let result1 = operation(a, b);
let result2 = operation(b, a);
prop_assert_eq!(result1, result2);
```

## 故障排除

### 测试失败时

proptest会自动：
1. 显示失败的输入
2. 缩小到最小失败案例
3. 保存到 `proptest-regressions/` 目录
4. 提供重放命令

### 常见问题

1. **编译错误**: 确保在 `dev-dependencies` 中添加了 `proptest`
2. **测试超时**: 调整 `proptest.toml` 中的 `timeout` 值
3. **浮点数误差**: 使用 `approx_eq` 而不是 `assert_eq`

## 贡献

添加新的属性测试时：

1. 选择有意义的属性（数学性质、不变量、对称性等）
2. 使用合适的输入范围
3. 处理边界情况
4. 添加清晰的文档注释
5. 运行测试验证

## 后续工作

- [ ] 运行所有测试并修复发现的bug
- [ ] 添加CI集成
- [ ] 扩展到更多模块
- [ ] 优化测试性能
- [ ] 添加更多文档和示例

## 致谢

本项目使用 [proptest](https://github.com/AltSysrq/proptest) 库实现Property-Based Testing。
