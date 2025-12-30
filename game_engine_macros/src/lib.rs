//! # Game Engine Macros
//!
//! 提供游戏引擎开发中常用的派生宏，减少代码重复。
//!
//! ## 可用的宏
//!
//! - [`Constructor`] - 自动生成构造函数
//! - [`ComponentWrapper`] - 自动生成ECS组件包装器
//! - [`Serializable`] - 自动生成序列化方法
//!
//! ## 示例
//!
//! ```rust
//! use game_engine_macros::{Constructor, ComponentWrapper, Serializable};
//! use bevy_ecs::component::Component;
//!
//! #[derive(Constructor, ComponentWrapper, Serializable)]
//! pub struct Velocity {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//! }
//!
//! // 自动生成:
//! // - new() 构造函数 (Constructor)
//! // - Component trait实现 (ComponentWrapper)
//! // - serialize() / deserialize() 方法 (Serializable)
//! ```

use proc_macro::TokenStream;

mod component_wrapper;
mod constructor;
mod serializable;

/// Constructor派生宏
///
/// 自动为结构体生成构造函数。
///
/// # 示例
///
/// ```
/// use game_engine_macros::Constructor;
///
/// #[derive(Constructor)]
/// pub struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// // 自动生成:
/// // impl Point {
/// //     pub fn new(x: f32, y: f32) -> Self {
/// //         Self { x, y }
/// //     }
/// // }
/// ```
#[proc_macro_derive(Constructor)]
pub fn derive_constructor(input: TokenStream) -> TokenStream {
    constructor::impl_constructor_macro(input)
}

/// ComponentWrapper派生宏
///
/// 为ECS组件包装器自动生成实现。
///
/// # 示例
///
/// ```
/// use game_engine_macros::ComponentWrapper;
///
/// #[derive(ComponentWrapper)]
/// pub struct Velocity(pub glam::Vec3);
///
/// // 自动生成:
/// // - impl Component for Velocity
/// // - impl Deref for Velocity
/// // - impl DerefMut for Velocity
/// // - impl From<glam::Vec3> for Velocity
/// ```
#[proc_macro_derive(ComponentWrapper)]
pub fn derive_component_wrapper(input: TokenStream) -> TokenStream {
    component_wrapper::impl_component_wrapper_macro(input)
}

/// Serializable派生宏
///
/// 自动生成序列化和反序列化方法。
///
/// # 示例
///
/// ```
/// use game_engine_macros::Serializable;
///
/// #[derive(Serializable, serde::Serialize, serde::Deserialize)]
/// pub struct GameState {
///     score: u32,
///     level: u32,
/// }
///
/// // 自动生成:
/// // impl GameState {
/// //     pub fn serialize(&self) -> Result<Vec<u8>, SerializationError>;
/// //     pub fn deserialize(data: &[u8]) -> Result<Self, SerializationError>;
/// // }
/// ```
#[proc_macro_derive(Serializable)]
pub fn derive_serializable(input: TokenStream) -> TokenStream {
    serializable::impl_serializable_macro(input)
}
