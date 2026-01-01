//! 代码生成模块
//!
//! 从AST生成目标代码。

pub trait CodeGenerator {
    fn generate(&mut self) -> Result<Vec<u8>, String>;
}
