//! # LSP测试模块
//!
//! 测试Language Server Protocol实现。

mod lsp_tests;

// 重新导出测试辅助函数
pub use lsp_tests::{create_test_document, create_test_position};
