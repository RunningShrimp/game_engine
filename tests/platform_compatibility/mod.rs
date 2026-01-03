//! Platform Compatibility Test Suite
//!
//! Comprehensive tests for platform compatibility across all console platforms.

mod switch_tests;
mod ps5_tests;
mod ps4_tests;
mod xbox_tests;
mod cross_platform_tests;

pub use switch_tests::*;
pub use ps5_tests::*;
pub use ps4_tests::*;
pub use xbox_tests::*;
pub use cross_platform_tests::*;
