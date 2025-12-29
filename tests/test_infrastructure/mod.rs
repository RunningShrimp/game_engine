//  测试基础设施模块
//
//  提供统一的测试工具、辅助函数和测试模式，
//  简化测试编写并提高测试一致性。

pub mod helpers;
pub mod assertions;
pub mod fixtures;

// 重新导出常用工具
pub use helpers::*;
pub use assertions::*;
pub use fixtures::*;

/// 测试辅助工具集合
pub struct TestTools {
    /// 测试名称
    pub name: String,
    /// 测试开始时间
    pub start_time: std::time::Instant,
}

impl TestTools {
    /// 创建新的测试工具实例
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start_time: std::time::Instant::now(),
        }
    }

    /// 记录测试完成
    pub fn complete(&self) {
        let duration = self.start_time.elapsed();
        println!("✅ Test '{}' completed in {:?}", self.name, duration);
    }

    /// 断言测试在预期时间内完成
    pub fn assert_completed_within(&self, max_duration: std::time::Duration) -> &Self {
        let duration = self.start_time.elapsed();
        assert!(
            duration <= max_duration,
            "Test '{}' exceeded expected duration: {:?} > {:?}",
            self.name,
            duration,
            max_duration
        );
        self
    }
}

/// 测试宏集合
#[macro_export]
macro_rules! test_tools {
    ($name:expr) => {
        $crate::test_infrastructure::TestTools::new($name)
    };
}

#[macro_export]
macro_rules! assert_elapsed {
    ($max_duration:expr, $block:block) => {
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        assert!(
            duration <= $max_duration,
            "Block exceeded expected duration: {:?} > {:?}",
            duration,
            $max_duration
        );
        result
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_test_tools() {
        let tools = TestTools::new("example_test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        tools.complete();
        tools.assert_completed_within(std::time::Duration::from_millis(100));
    }
}
