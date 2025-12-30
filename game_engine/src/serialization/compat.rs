// 序列化兼容性层
//
// 提供统一的序列化API，处理不同版本的bincode和ron
//
// 版本历史：
// - v1.0: 初始版本
// - v1.5: 文档更新，说明使用bincode 1.3

use serde::{Deserialize, Serialize};

/// Bincode序列化辅助函数
pub mod bincode_compat {
    use super::*;

    /// 序列化为Vec<u8> (无Send+Sync约束，用于单线程上下文)
    pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        bincode::serialize(value).map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })
    }

    /// 序列化为Vec<u8> (带Send+Sync约束，用于异步/多线程上下文)
    pub fn serialize_send<T: Serialize>(
        value: &T,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        bincode::serialize(value)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
    }

    /// 从切片反序列化 (无Send+Sync约束，用于单线程上下文)
    pub fn deserialize<'a, T: Deserialize<'a>>(
        data: &'a [u8],
    ) -> Result<T, Box<dyn std::error::Error>> {
        bincode::deserialize(data).map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })
    }

    /// 从切片反序列化 (带Send+Sync约束，用于异步/多线程上下文)
    pub fn deserialize_send<'a, T: Deserialize<'a>>(
        data: &'a [u8],
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        bincode::deserialize(data)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
    }
}

/// RON序列化辅助函数
pub mod ron_compat {
    use super::*;

    /// 序列化为RON字符串（ron 0.12兼容）
    pub fn to_string_pretty<T: Serialize>(
        value: &T,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::new())
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
    }

    /// 从RON字符串反序列化
    pub fn from_str<T: for<'de> Deserialize<'de>>(
        s: &str,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        ron::from_str(s).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_bincode_serialize() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let serialized = bincode_compat::serialize(&data).unwrap();
        let deserialized: TestData = bincode_compat::deserialize(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_ron_serialize() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let ron_str = ron_compat::to_string_pretty(&data).unwrap();
        let deserialized: TestData = ron_compat::from_str(&ron_str).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_bincode_send_sync() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let serialized = bincode_compat::serialize_send(&data).unwrap();
        let deserialized: TestData = bincode_compat::deserialize_send(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_bincode_roundtrip_multiple_values() {
        let values = vec![
            TestData {
                name: "first".to_string(),
                value: 1,
            },
            TestData {
                name: "second".to_string(),
                value: 2,
            },
            TestData {
                name: "third".to_string(),
                value: 3,
            },
        ];

        for data in values {
            let serialized = bincode_compat::serialize(&data).unwrap();
            let deserialized: TestData = bincode_compat::deserialize(&serialized).unwrap();
            assert_eq!(data, deserialized);
        }
    }
}
