//! # Platform Mock Simulation System
//!
//! Mock implementations of console platforms for testing and development.
//! Allows testing console-specific code without actual hardware.

pub mod base_mock;
pub mod ps4_mock;
pub mod ps5_mock;
pub mod switch_mock;
pub mod xbox_mock;

pub use base_mock::*;
pub use ps4_mock::*;
pub use ps5_mock::*;
pub use switch_mock::*;
pub use xbox_mock::*;
