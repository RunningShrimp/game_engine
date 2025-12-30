// ============================================================================
// Property-Based Testing 主入口
// ============================================================================
//
// 本文件是Property-Based Testing的主入口，整合所有模块的属性测试。
//
// ## Property-Based Testing简介
//
// PBT与传统测试的区别：
// - **传统测试**: 手工编写具体测试用例
// - **PBT**: 自动生成大量随机测试用例，验证通用属性
//
// ## 使用方法
//
// 运行所有属性测试：
// ```bash
// cargo test --test property_tests
// ```
//
// 运行特定模块的属性测试：
// ```bash
// cargo test --test property_tests test_entity_id_roundtrip
// ```
//
// ## 添加新的属性测试
//
// 1. 在对应的模块文件中添加proptest!宏
// 2. 在本文件中添加mod声明
// 3. 运行测试验证
//
// ## 相关文档
//
// - [proptest文档](https://altsysrq.github.io/proptest-book/proptest/getting-started.html)
// - 项目文档: docs/PBT_USAGE.md

// 模块化属性测试
// 注意：这些模块可能需要根据实际编译错误进行调整
// mod ecs_properties;
// mod physics_properties;
// mod network_properties;
// mod resources_properties;
// mod math_properties;

// ============================================================================
// 通用策略定义
// ============================================================================
//
// 这些策略在所有属性测试中共享使用，提供一致的测试数据生成。

pub mod strategies {
    use glam::Vec3;
    use proptest::prelude::*;

    /// 坐标策略：生成合理的浮点数坐标
    pub fn coord() -> impl Strategy<Value = f32> {
        -1000.0..=1000.0f32
    }

    /// 小坐标策略：生成小范围的坐标（适合局部测试）
    pub fn coord_small() -> impl Strategy<Value = f32> {
        -100.0..=100.0f32
    }

    /// 向量策略：生成3D向量
    pub fn vec3() -> impl Strategy<Value = Vec3> {
        prop::array::uniform3(coord()).prop_map(|arr| Vec3::from_array(arr))
    }

    /// 小向量策略：生成小范围的3D向量
    pub fn vec3_small() -> impl Strategy<Value = Vec3> {
        prop::array::uniform3(coord_small()).prop_map(|arr| Vec3::from_array(arr))
    }

    /// 单位向量策略：生成归一化的3D向量
    pub fn vec3_normalized() -> impl Strategy<Value = Vec3> {
        vec3()
            .prop_filter("vector too close to zero", |v| v.length() > 0.001)
            .prop_map(|v| v.normalize())
    }

    /// 颜色策略：生成RGBA颜色值（0.0-1.0）
    pub fn color() -> impl Strategy<Value = [f32; 4]> {
        prop::array::uniform4(0.0..=1.0f32)
    }

    /// RGB颜色策略：生成RGB颜色值（0.0-1.0）
    pub fn color_rgb() -> impl Strategy<Value = [f32; 3]> {
        prop::array::uniform3(0.0..=1.0f32)
    }

    /// 正整数策略：生成小范围的正整数
    pub fn usize_small() -> impl Strategy<Value = usize> {
        0usize..1000
    }

    /// 中等范围的正整数策略
    pub fn usize_medium() -> impl Strategy<Value = usize> {
        0usize..10000
    }

    /// 实体索引策略：生成合理的实体索引
    pub fn entity_index() -> impl Strategy<Value = u32> {
        0u32..1000000u32
    }

    /// 时间步长策略：生成合理的物理时间步长
    pub fn time_step() -> impl Strategy<Value = f32> {
        0.0001f32..0.1f32
    }

    /// 质量策略：生成合理的物体质量
    pub fn mass() -> impl Strategy<Value = f32> {
        0.001f32..1000.0f32
    }

    /// 字符串策略：生成非空字符串
    pub fn non_empty_string() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9]{1,50}"
    }

    /// 尺寸策略：生成合理的物体尺寸
    pub fn size() -> impl Strategy<Value = f32> {
        0.1f32..100.0f32
    }

    /// 半径策略：生成合理的物体半径
    pub fn radius() -> impl Strategy<Value = f32> {
        0.1f32..10.0f32
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 检查两个浮点数是否近似相等
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// 检查两个向量是否近似相等
pub fn vec3_approx_eq(a: glam::Vec3, b: glam::Vec3, epsilon: f32) -> bool {
    (a - b).length() < epsilon
}

/// 检查四元数是否近似相等
pub fn quat_approx_eq(a: glam::Quat, b: glam::Quat, epsilon: f32) -> bool {
    // 四元数 q 和 -q 表示相同的旋转
    let dot = a.dot(b);
    dot.abs() > 1.0 - epsilon
}

// ============================================================================
// 测试运行器
// ============================================================================

#[test]
#[ignore] // TODO: Fix compilation errors
fn run_all_property_tests() {
    // proptest会自动运行所有proptest!宏定义的测试
    // 这个函数用于确保所有测试模块被包含
}

// ============================================================================
// 常见属性验证函数
// ============================================================================

/// 验证实体的idempotence（幂等性）
/// 多次调用相同操作应该得到相同结果
pub fn check_idempotence<T>(value: T, f: impl Fn(&T) -> T) -> bool
where
    T: PartialEq + std::fmt::Debug,
{
    let result1 = f(&value);
    let result2 = f(&result1);
    result1 == result2
}

/// 验证操作的可逆性
/// 应用操作后再应用逆操作应该得到原始值
pub fn check_reversibility<T, F1, F2>(value: T, f: F1, g: F2) -> bool
where
    T: PartialEq + std::fmt::Debug,
    F1: Fn(&T) -> T,
    F2: Fn(&T) -> T,
{
    let transformed = f(&value);
    let reversed = g(&transformed);
    reversed == value
}

/// 验证对称性
/// 操作(a, b) 应该等于 reverse_operation(b, a)
pub fn check_symmetry<T, O>(a: T, b: T, op: O) -> bool
where
    T: PartialEq + std::fmt::Debug,
    O: Fn(&T, &T) -> bool,
{
    op(&a, &b) == op(&b, &a)
}

/// 验证结合律
/// op(a, op(b, c)) == op(op(a, b), c)
pub fn check_associativity<T, O>(a: T, b: T, c: T, op: O) -> bool
where
    T: PartialEq + std::fmt::Debug,
    O: Fn(&T, &T) -> T,
{
    let left = op(&a, &op(&b, &c));
    let right = op(&op(&a, &b), &c);
    left == right
}

// ============================================================================
// 基准测试属性
// ============================================================================

#[cfg(test)]
mod benchmarks {
    use proptest::prelude::*;
    use proptest::test_runner::TestRunner;

    /// 测试策略生成的性能
    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_strategy_generation_performance() {
        use super::strategies::*;
        use std::time::Instant;

        let start = Instant::now();
        let mut runner = TestRunner::default();

        for _ in 0..1000 {
            let _ = runner.run(&vec3(), |_| Ok(()));
        }

        let duration = start.elapsed();
        println!("Generated 1000 vec3 samples in {:?}", duration);
        assert!(duration.as_millis() < 1000, "Strategy generation too slow");
    }
}
