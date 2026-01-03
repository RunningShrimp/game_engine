// Integration Tests Module
// 本模块包含所有集成测试

mod platform_certification_tests;
mod controller_extended_tests;
mod gpu_manager_tests;
mod code_tools_tests;
mod editor_integration_tests;
mod e2e_scenario_tests;

pub use platform_certification_tests::*;
pub use controller_extended_tests::*;
pub use gpu_manager_tests::*;
pub use code_tools_tests::*;
pub use editor_integration_tests::*;
pub use e2e_scenario_tests::*;
