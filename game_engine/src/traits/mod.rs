//! 公共trait模块
//!
//! 提供引擎范围内使用的公共trait，减少代码重复。

pub mod common;

// 重新导出常用trait
pub use common::{Builder, CloneExt, ComponentExt, Serializable, Service};
