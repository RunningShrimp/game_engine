//! 公共trait模块
//!
//! 提供引擎范围内使用的公共trait，减少代码重复。

/// Serializable trait - 统一的序列化接口
///
/// 为类型提供序列化和反序列化的统一接口。
///
/// # 示例
///
/// ```rust
/// use game_engine::traits::Serializable;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct GameState {
///     score: u32,
/// }
///
/// impl Serializable for GameState {
///     fn serialize(&self) -> Result<Vec<u8>, crate::error::SerializationError> {
///         // 实现序列化
///     }
///
///     fn deserialize(data: &[u8]) -> Result<Self, crate::error::SerializationError>
///     where
///         Self: Sized {
///         // 实现反序列化
///     }
/// }
/// ```
pub trait Serializable: Sized {
    /// 序列化到二进制格式
    fn serialize(&self) -> Result<Vec<u8>, crate::error::SerializationError>;

    /// 从二进制格式反序列化
    fn deserialize(data: &[u8]) -> Result<Self, crate::error::SerializationError>;
}

/// Service trait - 服务基础trait
///
/// 为引擎中的服务提供统一接口。
///
/// # 示例
///
/// ```rust
/// use game_engine::traits::Service;
///
/// struct MyService {
///     // fields
/// }
///
/// impl Service for MyService {
///     type Error = MyError;
///
///     fn initialize(&mut self) -> Result<(), Self::Error> {
///         // 初始化服务
///         Ok(())
///     }
///
///     fn shutdown(&mut self) -> Result<(), Self::Error> {
///         // 关闭服务
///         Ok(())
///     }
/// }
/// ```
pub trait Service {
    /// 服务相关的错误类型
    type Error: std::error::Error + Send + Sync + 'static;

    /// 初始化服务
    fn initialize(&mut self) -> Result<(), Self::Error>;

    /// 关闭服务
    fn shutdown(&mut self) -> Result<(), Self::Error>;

    /// 检查服务是否已初始化
    fn is_initialized(&self) -> bool {
        true
    }
}

/// ComponentExt trait - ECS组件扩展trait
///
/// 为ECS组件提供额外的通用方法。
///
/// # 示例
///
/// ```rust
/// use game_engine::traits::ComponentExt;
/// use bevy_ecs::component::Component;
///
/// #[derive(Component)]
/// struct MyComponent {
///     value: f32,
/// }
///
/// impl ComponentExt for MyComponent {
///     fn as_bytes(&self) -> Vec<u8> {
///         // 将组件转换为字节
///         vec![]
///     }
/// }
/// ```
pub trait ComponentExt: bevy_ecs::component::Component {
    /// 将组件转换为字节数组
    fn as_bytes(&self) -> Vec<u8>;

    /// 获取组件的内存大小
    fn size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

/// Builder trait - 构建器模式trait
///
/// 为复杂类型提供流式构建接口。
///
/// # 示例
///
/// ```rust
/// use game_engine::traits::Builder;
///
/// struct MyConfig {
///     value: u32,
///     name: String,
/// }
///
/// impl Builder for MyConfig {
///     type Output = MyConfig;
///
///     fn build(self) -> Self::Output {
///         self
///     }
/// }
/// ```
pub trait Builder {
    /// 构建的输出类型
    type Output;

    /// 构建最终对象
    fn build(self) -> Self::Output;
}

/// CloneExt trait - 克隆扩展trait
///
/// 为克隆操作提供额外的实用方法。
pub trait CloneExt {
    /// 深度克隆
    fn deep_clone(&self) -> Self
    where
        Self: Sized;

    /// 克隆到Box
    fn cloned_box(&self) -> Box<Self>
    where
        Self: Sized;
}

impl<T: Clone> CloneExt for T {
    fn deep_clone(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }

    fn cloned_box(&self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_ext() {
        let value = 42;
        let cloned = value.deep_clone();
        assert_eq!(value, cloned);

        let boxed = value.cloned_box();
        assert_eq!(*boxed, 42);
    }
}
