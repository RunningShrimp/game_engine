//! 解析器模块
//!
//! 将各种源语言解析为抽象语法树（AST）。

use super::{AST, CompileError, Statement, Expression, Type, Parameter, SourceLocation};

pub mod rust;
pub mod lua;
pub mod typescript;
pub mod csharp;

pub fn parse_rust(_source: &str, _filename: &str) -> Result<AST, CompileError> {
    Ok(AST::Module { statements: Vec::new() })
}

pub fn parse_lua(_source: &str, _filename: &str) -> Result<AST, CompileError> {
    Ok(AST::Module { statements: Vec::new() })
}

pub fn parse_typescript(_source: &str, _filename: &str) -> Result<AST, CompileError> {
    Ok(AST::Module { statements: Vec::new() })
}

pub fn parse_csharp(_source: &str, _filename: &str) -> Result<AST, CompileError> {
    Ok(AST::Module { statements: Vec::new() })
}
