# 测试基础设施使用指南

**文档版本**: 1.0
**更新时间**: 2025-12-28

---

## 📋 目录

1. [概述](#概述)
2. [快速开始](#快速开始)
3. [测试辅助工具](#测试辅助工具)
4. [测试断言](#测试断言)
5. [测试Fixture](#测试fixture)
6. [最佳实践](#最佳实践)

---

## 概述

测试基础设施模块提供统一的测试工具、辅助函数和测试模式，旨在：
- 简化测试编写
- 提高测试一致性
- 减少重复代码
- 增强测试可读性

### 模块结构

```
tests/test_infrastructure/
├── mod.rs              # 主模块导出
├── helpers.rs          # 测试辅助函数
├── assertions.rs       # 自定义断言
└── fixtures.rs         # 测试fixture
```

---

## 快速开始

### 基本导入

```rust
use game_engine::test_infrastructure::*;
```

### 简单示例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use game_engine::test_infrastructure::*;

    #[test]
    fn test_example() {
        // 使用测试工具
        let tools = TestTools::new("my_test");

        // 执行测试逻辑
        let result = 2 + 2;
        assert_eq!(result, 4);

        // 记录完成
        tools.complete();
    }
}
```

---

## 测试辅助工具

### wait_for - 等待条件满足

等待条件成立或超时。

```rust
use std::time::Duration;

#[test]
fn test_async_behavior() {
    let mut value = 0;

    // 在另一个线程中修改value
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(50));
        value = 42;
    });

    // 等待条件满足
    let result = wait_for(
        || value == 42,
        Duration::from_secs(5),
        Duration::from_millis(10),
    );

    assert!(result.is_ok(), "Timeout waiting for value");
}
```

### retry_until_success - 重试直到成功

```rust
#[test]
fn test_retry_mechanism() {
    let mut attempts = 0;

    let result = retry_until_success(
        || {
            attempts += 1;
            if attempts < 3 {
                Err("not yet")
            } else {
                Ok(())
            }
        },
        Duration::from_secs(5),
        Duration::from_millis(100),
    );

    assert!(result.is_ok());
}
```

### measure_time - 测量执行时间

```rust
#[test]
fn test_performance() {
    let duration = measure_time(|| {
        expensive_operation();
    });

    println!("Operation took: {:?}", duration);
    assert!(duration < Duration::from_secs(1), "Too slow!");
}
```

### Benchmark - 性能基准测试

```rust
#[test]
fn test_benchmark() {
    let bench = Benchmark::new("expensive_operation", 1000);
    let result = bench.run(|| {
        expensive_operation();
    });

    println!("{}", result);
    // 输出: Benchmark 'expensive_operation': 1000 iterations in 1.234s (avg: 1.234ms)
}
```

---

## 测试断言

### assert_approx_eq - 浮点数比较

```rust
#[test]
fn test_float_comparison() {
    let a = 1.0;
    let b = 1.001;

    // 允许0.01的误差
    assert_approx_eq(a, b, 0.01);
}
```

### assert_vec_approx_eq - 向量比较

```rust
#[test]
fn test_vector_comparison() {
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![1.001, 2.001, 3.001];

    assert_vec_approx_eq(&a, &b, 0.01);
}
```

### assert_completed_within - 超时断言

```rust
#[test]
fn test_timeout() {
    assert_completed_within(
        Duration::from_millis(100),
        || {
            fast_operation();
        }
    );
}
```

### assert_contains / assert_not_contains

```rust
#[test]
fn test_contains() {
    let vec = vec![1, 2, 3, 4, 5];

    assert_contains(&vec, &3);
    assert_not_contains(&vec, &6);
}
```

### assert_panics / assert_not_panics

```rust
#[test]
fn test_panic_behavior() {
    assert_panics(|| {
        panic!("Expected panic");
    });
}

#[test]
fn test_no_panic() {
    assert_not_panics(|| {
        let result = safe_computation();
        result
    });
}
```

---

## 测试Fixture

### WorldFixture - ECS World测试

```rust
#[test]
fn test_ecs_system() {
    let mut fixture = WorldFixture::new();

    // 添加测试实体
    let entity = fixture.spawn_test_entity();

    // 验证
    assert!(fixture.world.get::<Name>(entity).is_some());
}
```

### SceneFixture - 场景测试

```rust
#[test]
fn test_scene_operations() {
    let mut fixture = SceneFixture::new().unwrap();

    // 激活场景
    fixture.scene.activate().unwrap();
    assert_eq!(fixture.scene.state(), SceneState::Active);
}
```

### ConfigFixture - 配置测试

```rust
#[test]
fn test_with_config() {
    let fixture = ConfigFixture::new();
    let config = fixture.config;

    assert_eq!(config.graphics.resolution.width, 800);
}
```

### ResourceLoaderFixture - 资源加载测试

```rust
#[test]
fn test_resource_loading() {
    let mut fixture = ResourceLoaderFixture::new();
    let mut loader = fixture.loader.lock().unwrap();

    // 加载资源
    let result = loader.load("test_path");
    assert!(result.is_ok());
}
```

### PerformanceFixture - 性能测试

```rust
#[test]
fn test_performance_tracking() {
    let mut fixture = PerformanceFixture::new();

    // 测量多个操作
    fixture.measure("operation1", || {
        operation1();
    });

    fixture.measure("operation2", || {
        operation2();
    });

    // 打印报告
    fixture.print_report();
}
```

---

## 最佳实践

### 1. 使用fixture减少重复

**❌ Before**:
```rust
#[test]
fn test_scene() {
    let scene = Scene::new(SceneId::new(1), "test")
        .expect("Failed to create scene");

    scene.add_entity(entity).expect("Failed");
    // ...
}
```

**✅ After**:
```rust
#[test]
fn test_scene() {
    let mut fixture = SceneFixture::new().unwrap();
    fixture.scene.add_entity(entity).unwrap();
    // ...
}
```

### 2. 使用辅助函数简化异步测试

**❌ Before**:
```rust
#[test]
fn test_async() {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if condition() { break; }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(condition(), "Timeout");
}
```

**✅ After**:
```rust
#[test]
fn test_async() {
    let result = wait_for(
        || condition(),
        Duration::from_secs(5),
        Duration::from_millis(100),
    );
    assert!(result.is_ok(), "Timeout");
}
```

### 3. 使用性能测试fixture

**✅ Good**:
```rust
#[test]
fn test_performance() {
    let mut fixture = PerformanceFixture::new();

    fixture.measure("expensive_operation", || {
        expensive_operation();
    });

    fixture.print_report();
}
```

### 4. 组合使用工具

```rust
#[test]
fn test_complex_scenario() {
    let mut world = WorldFixture::new();
    let mut perf = PerformanceFixture::new();
    let tools = TestTools::new("complex_test");

    // 测量设置阶段
    perf.measure("setup", || {
        let entity = world.spawn_test_entity();
        // ...
    });

    // 测量执行阶段
    perf.measure("execute", || {
        execute_system(&mut world.world);
    });

    // 验证结果
    assert!(world.world.entities().len() > 0);

    // 完成
    tools.complete();
    perf.print_report();
}
```

---

## 测试模板

### 单元测试模板

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use game_engine::test_infrastructure::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = create_test_input();
        let expected = create_expected_output();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### 集成测试模板

```rust
#[test]
fn test_integration() {
    let mut world = WorldFixture::new();
    let tools = TestTools::new("integration_test");

    // Setup
    setup_system(&mut world.world);

    // Execute
    execute_workflow(&mut world.world);

    // Verify
    verify_results(&world.world);

    tools.complete();
}
```

### 性能测试模板

```rust
#[test]
fn test_performance() {
    let bench = Benchmark::new("operation", 1000);
    let result = bench.run(|| {
        operation_to_benchmark();
    });

    // 验证性能
    assert!(result.avg_duration < Duration::from_millis(10));
    println!("{}", result);
}
```

---

## 集成到项目

### Cargo.toml配置

确保`Cargo.toml`包含必要的开发依赖：

```toml
[dev-dependencies]
tempfile = "3"
criterion = "0.5"
```

### 测试目录结构

```
tests/
├── test_infrastructure/
│   ├── mod.rs
│   ├── helpers.rs
│   ├── assertions.rs
│   └── fixtures.rs
└── integration_tests/
    └── ...
```

---

## 总结

测试基础设施提供：
- ✅ 统一的测试工具
- ✅ 减少重复代码
- ✅ 提高测试一致性
- ✅ 增强测试可读性

使用这些工具可以编写更清晰、更易维护的测试代码。

---

**文档版本**: 1.0
**更新时间**: 2025-12-28
