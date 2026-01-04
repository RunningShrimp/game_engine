//! # LSP测试模块
//!
//! 测试Language Server Protocol实现。

mod lsp_tests;
pub mod completion_test;
pub mod diagnostics_test;
pub mod text_sync_test;
pub mod hover_test;
pub mod symbols_test;
pub mod e2e_test;
pub mod performance_test;

// 重新导出测试辅助函数
pub use lsp_tests::{create_test_document, create_test_position};
