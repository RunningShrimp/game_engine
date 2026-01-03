//! 中间语言（IL）编译器
//!
//! 提供跨平台的中间字节码表示、编译器、虚拟机和优化系统。
//! 支持从多种源语言（Rust、Lua、TypeScript、C#等）编译到统一的IL。
//!
//! ## 架构
//!
//! ```text
//! 源代码 → 前端 → IR → 优化 → 后端 → 目标代码
//!           前端               后端
//!  ┌──────┴──────┐    ┌──────┴──────┐
//!  │   解析器    │    │  代码生成   │
//!  │   类型检查  │    │  寄存器分配 │
//!  │   语义分析  │    │  指令选择   │
//!  └─────────────┘    └─────────────┘
//! ```
//!
//! ## 功能特性
//!
//! - **多语言支持**: Rust、Lua、TypeScript、C# → IL
//! - **JIT编译**: 运行时即时编译为本地代码
//! - **AOT编译**: 提前编译为本地代码
//! - **优化通道**: 常量折叠、死代码消除、内联等
//! - **热重载**: 无需重启游戏即可重载代码
//! - **调试信息**: 丰富的调试信息支持

pub mod bytecode;
pub mod codegen;
pub mod debug;
pub mod hot_reload;
pub mod jit;
pub mod optimizer;
pub mod parser;
pub mod vm;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IL编译器版本
pub const IL_VERSION: &str = "1.0.0";

/// IL魔术字节（用于文件识别）
pub const IL_MAGIC: &[u8; 4] = b"IL\x00\x01";

/// IL字节码文件头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ILHeader {
    /// 魔术字节
    pub magic: [u8; 4],
    /// IL版本
    pub version: String,
    /// 源语言类型
    pub source_language: SourceLanguage,
    /// 编译时间戳
    pub compile_timestamp: u64,
    /// 入口函数索引
    pub entry_point: u32,
    /// 常量表偏移
    pub constants_offset: u32,
    /// 代码段偏移
    pub code_offset: u32,
    /// 调试信息偏移
    pub debug_info_offset: u32,
    /// 元数据偏移
    pub metadata_offset: u32,
}

impl Default for ILHeader {
    fn default() -> Self {
        Self {
            magic: *IL_MAGIC,
            version: IL_VERSION.to_string(),
            source_language: SourceLanguage::Unknown,
            compile_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            entry_point: 0,
            constants_offset: 0,
            code_offset: 0,
            debug_info_offset: 0,
            metadata_offset: 0,
        }
    }
}

/// 源语言类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceLanguage {
    /// Rust
    Rust,
    /// Lua
    Lua,
    /// TypeScript/JavaScript
    TypeScript,
    /// C#
    CSharp,
    /// Python
    Python,
    /// 其他
    Other,
    /// 未知
    Unknown,
}

impl std::fmt::Display for SourceLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceLanguage::Rust => write!(f, "Rust"),
            SourceLanguage::Lua => write!(f, "Lua"),
            SourceLanguage::TypeScript => write!(f, "TypeScript"),
            SourceLanguage::CSharp => write!(f, "C#"),
            SourceLanguage::Python => write!(f, "Python"),
            SourceLanguage::Other => write!(f, "Other"),
            SourceLanguage::Unknown => write!(f, "Unknown"),
        }
    }
}

/// IL编译器
pub struct ILCompiler {
    /// 源语言类型
    source_language: SourceLanguage,
    /// 编译选项
    options: CompileOptions,
    /// 符号表
    symbol_table: HashMap<String, Symbol>,
    /// 函数表
    functions: Vec<Function>,
    /// 常量表
    constants: Vec<Constant>,
    /// 全局变量
    globals: Vec<GlobalVariable>,
    /// 调试信息
    debug_info: DebugInfo,
}

/// 编译选项
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// 是否启用优化
    pub optimize: bool,
    /// 优化级别（0-3）
    pub opt_level: u8,
    /// 是否生成调试信息
    pub debug_info: bool,
    /// 是否启用JIT编译
    pub enable_jit: bool,
    /// 是否启用AOT编译
    pub enable_aot: bool,
    /// 目标平台
    pub target_platform: TargetPlatform,
    /// 验证字节码
    pub verify_bytecode: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            optimize: true,
            opt_level: 2,
            debug_info: true,
            enable_jit: true,
            enable_aot: false,
            target_platform: TargetPlatform::Native,
            verify_bytecode: true,
        }
    }
}

/// 目标平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    /// 本地平台
    Native,
    /// WebAssembly
    WebAssembly,
    /// x86_64
    X86_64,
    /// ARM64
    ARM64,
    /// Nintendo Switch
    Switch,
    /// PlayStation 4/5
    PlayStation,
    /// Xbox One/Series
    Xbox,
}

/// 符号
#[derive(Debug, Clone)]
pub struct Symbol {
    /// 符号名称
    pub name: String,
    /// 符号类型
    pub symbol_type: SymbolType,
    /// 作用域层级
    pub scope_level: u32,
    /// 是否为常量
    pub is_const: bool,
    /// 源位置
    pub source_location: SourceLocation,
}

/// 符号类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    /// 函数
    Function {
        return_type: Box<Type>,
        parameters: Vec<(String, Type)>,
    },
    /// 变量
    Variable(Type),
    /// 常量
    Constant(Type),
    /// 类型
    Type(Type),
    /// 结构体
    Struct { fields: Vec<(String, Type)> },
    /// 枚举
    Enum { variants: Vec<String> },
}

/// 类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// void
    Void,
    /// 布尔值
    Bool,
    /// 整数（i8, i16, i32, i64）
    Integer(IntegerSize),
    /// 浮点数（f32, f64）
    Float(FloatSize),
    /// 字符串
    String,
    /// 数组
    Array(Box<Type>, usize),
    /// 切片
    Slice(Box<Type>),
    /// 结构体
    Struct(String),
    /// 枚举
    Enum(String),
    /// 函数指针
    FunctionPointer {
        return_type: Box<Type>,
        parameters: Vec<Type>,
    },
    /// 任意类型
    Any,
}

/// 整数大小
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerSize {
    I8,
    I16,
    I32,
    I64,
}

/// 浮点数大小
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSize {
    F32,
    F64,
}

/// 函数
#[derive(Debug, Clone)]
pub struct Function {
    /// 函数名称
    pub name: String,
    /// 函数索引
    pub index: u32,
    /// 返回类型
    pub return_type: Type,
    /// 参数列表
    pub parameters: Vec<Parameter>,
    /// 局部变量
    pub locals: Vec<LocalVariable>,
    /// 字节码
    pub bytecode: Vec<u8>,
    /// 异常处理表
    pub exception_handlers: Vec<ExceptionHandler>,
    /// 调试信息
    pub debug_info: FunctionDebugInfo,
}

/// 参数
#[derive(Debug, Clone)]
pub struct Parameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub param_type: Type,
    /// 参数索引
    pub index: u32,
}

/// 局部变量
#[derive(Debug, Clone)]
pub struct LocalVariable {
    /// 变量名称
    pub name: String,
    /// 变量类型
    pub var_type: Type,
    /// 变量索引（在栈帧中）
    pub index: u32,
    /// 源位置
    pub source_location: SourceLocation,
}

/// 异常处理器
#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    /// try块起始偏移
    pub try_start: u32,
    /// try块结束偏移
    pub try_end: u32,
    /// catch块起始偏移
    pub catch_start: u32,
    /// catch块结束偏移
    pub catch_end: u32,
    /// 异常类型
    pub exception_type: Option<Type>,
}

/// 全局变量
#[derive(Debug, Clone)]
pub struct GlobalVariable {
    /// 变量名称
    pub name: String,
    /// 变量类型
    pub var_type: Type,
    /// 初始值
    pub initial_value: Option<Constant>,
    /// 是否为常量
    pub is_const: bool,
}

/// 常量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constant {
    /// 空值
    Null,
    /// 布尔值
    Bool(bool),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 字节数组
    Bytes(Vec<u8>),
}

/// 源位置
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// 文件名
    pub file: String,
    /// 行号
    pub line: u32,
    /// 列号
    pub column: u32,
}

/// 调试信息
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// 源文件列表
    pub source_files: Vec<String>,
    /// 行号表
    pub line_table: LineTable,
}

/// 行号表
#[derive(Debug, Clone)]
pub struct LineTable {
    /// 地址到行号的映射
    pub address_to_line: Vec<(u32, u32)>,
}

/// 函数调试信息
#[derive(Debug, Clone)]
pub struct FunctionDebugInfo {
    /// 函数名称
    pub name: String,
    /// 源位置
    pub source_location: SourceLocation,
    /// 局部变量位置
    pub local_variables: Vec<(String, u32)>,
    /// 行号表
    pub line_table: LineTable,
}

/// 编译结果
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// IL字节码
    pub bytecode: Vec<u8>,
    /// 函数列表
    pub functions: Vec<Function>,
    /// 常量列表
    pub constants: Vec<Constant>,
    /// 全局变量列表
    pub globals: Vec<GlobalVariable>,
    /// 编译统计
    pub stats: CompileStats,
    /// 错误列表
    pub errors: Vec<CompileError>,
    /// 警告列表
    pub warnings: Vec<CompileWarning>,
}

/// 编译统计
#[derive(Debug, Clone)]
pub struct CompileStats {
    /// 源代码行数
    pub source_lines: usize,
    /// 生成的字节数
    pub bytecode_bytes: usize,
    /// 函数数量
    pub function_count: usize,
    /// 常量数量
    pub constant_count: usize,
    /// 全局变量数量
    pub global_count: usize,
    /// 编译时间（毫秒）
    pub compile_time_ms: u64,
}

/// 编译错误
#[derive(Debug, Clone)]
pub struct CompileError {
    /// 错误类型
    pub error_type: CompileErrorType,
    /// 错误消息
    pub message: String,
    /// 源位置
    pub location: SourceLocation,
}

/// 编译错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileErrorType {
    /// 语法错误
    SyntaxError,
    /// 类型错误
    TypeError,
    /// 未定义符号
    UndefinedSymbol,
    /// 重复定义
    DuplicateDefinition,
    /// 其他错误
    Other(String),
}

/// 编译警告
#[derive(Debug, Clone)]
pub struct CompileWarning {
    /// 警告类型
    pub warning_type: CompileWarningType,
    /// 警告消息
    pub message: String,
    /// 源位置
    pub location: SourceLocation,
}

/// 编译警告类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileWarningType {
    /// 未使用的变量
    UnusedVariable,
    /// 未使用的函数
    UnusedFunction,
    /// 隐式类型转换
    ImplicitTypeConversion,
    /// 其他警告
    Other(String),
}

impl ILCompiler {
    /// 创建新的IL编译器
    pub fn new(source_language: SourceLanguage, options: CompileOptions) -> Self {
        Self {
            source_language,
            options,
            symbol_table: HashMap::new(),
            functions: Vec::new(),
            constants: Vec::new(),
            globals: Vec::new(),
            debug_info: DebugInfo {
                source_files: Vec::new(),
                line_table: LineTable {
                    address_to_line: Vec::new(),
                },
            },
        }
    }

    /// 编译源代码
    pub fn compile(
        &mut self,
        source_code: &str,
        filename: &str,
    ) -> Result<CompileResult, CompileError> {
        let start_time = std::time::Instant::now();

        // 1. 解析源代码
        let ast = self.parse(source_code, filename)?;

        // 2. 语义分析
        self.analyze(&ast)?;

        // 3. 生成字节码
        let mut bytecode = self.generate_bytecode(&ast)?;

        // 4. 优化（如果启用）
        if self.options.optimize {
            bytecode = self.optimize(bytecode)?;
        }

        // 5. 验证字节码（如果启用）
        if self.options.verify_bytecode {
            self.verify_bytecode(&bytecode)?;
        }

        let compile_time = start_time.elapsed().as_millis() as u64;

        // 生成编译结果
        let result = CompileResult {
            bytecode: bytecode.clone(),
            functions: self.functions.clone(),
            constants: self.constants.clone(),
            globals: self.globals.clone(),
            stats: CompileStats {
                source_lines: source_code.lines().count(),
                bytecode_bytes: bytecode.len(),
                function_count: self.functions.len(),
                constant_count: self.constants.len(),
                global_count: self.globals.len(),
                compile_time_ms: compile_time,
            },
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        Ok(result)
    }

    /// 解析源代码
    fn parse(&mut self, source_code: &str, filename: &str) -> Result<AST, CompileError> {
        // 根据源语言选择解析器
        match self.source_language {
            SourceLanguage::Lua => parser::lua::parse_lua(source_code, filename),
            SourceLanguage::TypeScript => {
                parser::typescript::parse_typescript(source_code, filename)
            }
            SourceLanguage::CSharp => parser::csharp::parse_csharp(source_code, filename),
            SourceLanguage::Rust => {
                // Rust解析器（简化版）
                parser::rust::parse_rust(source_code, filename)
            }
            _ => Err(CompileError {
                error_type: CompileErrorType::Other("Unsupported language".to_string()),
                message: format!(
                    "Source language {:?} is not supported",
                    self.source_language
                ),
                location: SourceLocation {
                    file: filename.to_string(),
                    line: 0,
                    column: 0,
                },
            }),
        }
    }

    /// 语义分析
    fn analyze(&mut self, ast: &AST) -> Result<(), CompileError> {
        // 构建符号表
        self.build_symbol_table(ast)?;

        // 类型检查
        self.type_check(ast)?;

        Ok(())
    }

    /// 构建符号表
    fn build_symbol_table(&mut self, ast: &AST) -> Result<(), CompileError> {
        match ast {
            AST::Module { statements } => {
                for stmt in statements {
                    self.declare_symbols(stmt)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 声明符号
    fn declare_symbols(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::FunctionDecl {
                name,
                params,
                return_type,
                ..
            } => {
                let param_types =
                    params.iter().map(|p| (p.name.clone(), p.param_type.clone())).collect();

                let symbol = Symbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Function {
                        return_type: Box::new(return_type.clone()),
                        parameters: param_types,
                    },
                    scope_level: 0,
                    is_const: false,
                    source_location: SourceLocation {
                        file: "".to_string(),
                        line: 0,
                        column: 0,
                    },
                };

                self.symbol_table.insert(name.clone(), symbol);
            }
            Statement::VarDecl { name, var_type, .. } => {
                let symbol = Symbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Variable(var_type.clone()),
                    scope_level: 0,
                    is_const: false,
                    source_location: SourceLocation {
                        file: "".to_string(),
                        line: 0,
                        column: 0,
                    },
                };

                self.symbol_table.insert(name.clone(), symbol);
            }
            _ => {}
        }
        Ok(())
    }

    /// 类型检查
    fn type_check(&mut self, ast: &AST) -> Result<(), CompileError> {
        match ast {
            AST::Module { statements } => {
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
            }
        }
        Ok(())
    }

    /// 检查语句
    fn check_statement(&self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::FunctionDecl { body, .. } => {
                for stmt in body {
                    self.check_statement(stmt)?;
                }
            }
            Statement::VarDecl {
                initial_value,
                var_type,
                ..
            } => {
                if let Some(expr) = initial_value {
                    let expr_type = self.infer_expr_type(expr)?;
                    if &expr_type != var_type && var_type != &Type::Any {
                        return Err(CompileError {
                            error_type: CompileErrorType::TypeError,
                            message: format!(
                                "Type mismatch: expected {:?}, got {:?}",
                                var_type, expr_type
                            ),
                            location: SourceLocation {
                                file: "".to_string(),
                                line: 0,
                                column: 0,
                            },
                        });
                    }
                }
            }
            Statement::Return { value } => {
                if let Some(expr) = value {
                    self.infer_expr_type(expr)?;
                }
            }
            Statement::Expression { expr } => {
                self.infer_expr_type(expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// 推断表达式类型
    fn infer_expr_type(&self, expr: &Expression) -> Result<Type, CompileError> {
        match expr {
            Expression::Literal { literal } => match literal {
                Literal::Null => Ok(Type::Any),
                Literal::Bool(_) => Ok(Type::Bool),
                Literal::Integer(_) => Ok(Type::Integer(IntegerSize::I32)),
                Literal::Float(_) => Ok(Type::Float(FloatSize::F64)),
                Literal::String(_) => Ok(Type::String),
            },
            Expression::Identifier { name } => {
                if let Some(symbol) = self.symbol_table.get(name) {
                    match &symbol.symbol_type {
                        SymbolType::Variable(t) => Ok(t.clone()),
                        SymbolType::Constant(t) => Ok(t.clone()),
                        _ => Ok(Type::Any),
                    }
                } else {
                    Ok(Type::Any)
                }
            }
            Expression::Binary { left, op, right } => {
                let left_type = self.infer_expr_type(left)?;
                let right_type = self.infer_expr_type(right)?;
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        if left_type == right_type {
                            match left_type {
                                Type::Integer(_) | Type::Float(_) => Ok(left_type),
                                _ => Ok(Type::Any),
                            }
                        } else {
                            Ok(Type::Any)
                        }
                    }
                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge => Ok(Type::Bool),
                    _ => Ok(Type::Any),
                }
            }
            Expression::Call { callee, args } => {
                if let Expression::Identifier { name } = callee.as_ref() {
                    if let Some(symbol) = self.symbol_table.get(name) {
                        if let SymbolType::Function { return_type, .. } = &symbol.symbol_type {
                            return Ok(*return_type.clone());
                        }
                    }
                }
                Ok(Type::Any)
            }
            _ => Ok(Type::Any),
        }
    }

    /// 生成字节码
    fn generate_bytecode(&mut self, ast: &AST) -> Result<Vec<u8>, CompileError> {
        let mut bytecode = Vec::new();

        // 写入文件头
        let header = ILHeader::default();
        self.write_header(&header, &mut bytecode);

        // 写入常量表
        let constants_offset = bytecode.len() as u32;
        self.write_constants(&mut bytecode);

        // 写入代码
        let code_offset = bytecode.len() as u32;
        self.write_code(ast, &mut bytecode);

        // 写入调试信息
        let debug_info_offset = bytecode.len() as u32;
        if self.options.debug_info {
            self.write_debug_info(&mut bytecode);
        }

        Ok(bytecode)
    }

    /// 写入文件头
    fn write_header(&self, header: &ILHeader, bytecode: &mut Vec<u8>) {
        bytecode.extend_from_slice(&header.magic);
        bytecode.extend_from_slice(header.version.as_bytes());
        bytecode.push(header.source_language as u8);
        bytecode.extend_from_slice(&header.compile_timestamp.to_be_bytes());
        bytecode.extend_from_slice(&header.entry_point.to_be_bytes());
        bytecode.extend_from_slice(&header.constants_offset.to_be_bytes());
        bytecode.extend_from_slice(&header.code_offset.to_be_bytes());
        bytecode.extend_from_slice(&header.debug_info_offset.to_be_bytes());
        bytecode.extend_from_slice(&header.metadata_offset.to_be_bytes());
    }

    /// 写入常量表
    fn write_constants(&self, bytecode: &mut Vec<u8>) {
        bytecode.extend_from_slice(&(self.constants.len() as u32).to_be_bytes());
        for constant in &self.constants {
            match constant {
                Constant::Null => bytecode.push(0x00),
                Constant::Bool(b) => bytecode.push(if *b { 0x01 } else { 0x02 }),
                Constant::Integer(i) => {
                    bytecode.push(0x03);
                    bytecode.extend_from_slice(&i.to_be_bytes());
                }
                Constant::Float(f) => {
                    bytecode.push(0x04);
                    bytecode.extend_from_slice(&f.to_be_bytes());
                }
                Constant::String(s) => {
                    bytecode.push(0x05);
                    bytecode.extend_from_slice(&(s.len() as u32).to_be_bytes());
                    bytecode.extend_from_slice(s.as_bytes());
                }
                Constant::Bytes(b) => {
                    bytecode.push(0x06);
                    bytecode.extend_from_slice(&(b.len() as u32).to_be_bytes());
                    bytecode.extend_from_slice(b);
                }
            }
        }
    }

    /// 写入代码
    fn write_code(&mut self, ast: &AST, bytecode: &mut Vec<u8>) {
        match ast {
            AST::Module { statements } => {
                for stmt in statements {
                    self.write_statement(stmt, bytecode);
                }
            }
        }
    }

    /// 写入语句
    fn write_statement(&mut self, stmt: &Statement, bytecode: &mut Vec<u8>) {
        match stmt {
            Statement::FunctionDecl {
                name, params, body, ..
            } => {
                // 函数声明（简化）
                bytecode.push(0x10); // FUNCTION_DEF
                bytecode.extend_from_slice(&(name.len() as u8).to_be_bytes());
                bytecode.extend_from_slice(name.as_bytes());
                bytecode.extend_from_slice(&(params.len() as u8).to_be_bytes());
                for stmt in body {
                    self.write_statement(stmt, bytecode);
                }
                bytecode.push(0x11); // END_FUNCTION
            }
            Statement::VarDecl {
                name,
                initial_value,
                ..
            } => {
                bytecode.push(0x20); // VAR_DECL
                bytecode.extend_from_slice(&(name.len() as u8).to_be_bytes());
                bytecode.extend_from_slice(name.as_bytes());
                if let Some(expr) = initial_value {
                    self.write_expression(expr, bytecode);
                }
            }
            Statement::Return { value } => {
                bytecode.push(0x30); // RETURN
                if let Some(expr) = value {
                    self.write_expression(expr, bytecode);
                }
            }
            Statement::Expression { expr } => {
                self.write_expression(expr, bytecode);
            }
            _ => {}
        }
    }

    /// 写入表达式
    fn write_expression(&mut self, expr: &Expression, bytecode: &mut Vec<u8>) {
        match expr {
            Expression::Literal { literal } => {
                bytecode.push(0x40); // PUSH_CONST
                match literal {
                    Literal::Null => bytecode.push(0x00),
                    Literal::Bool(b) => bytecode.push(if *b { 0x01 } else { 0x02 }),
                    Literal::Integer(i) => {
                        bytecode.extend_from_slice(&i.to_be_bytes());
                    }
                    Literal::Float(f) => {
                        bytecode.extend_from_slice(&f.to_be_bytes());
                    }
                    Literal::String(s) => {
                        bytecode.extend_from_slice(&(s.len() as u32).to_be_bytes());
                        bytecode.extend_from_slice(s.as_bytes());
                    }
                }
            }
            Expression::Identifier { name } => {
                bytecode.push(0x41); // LOAD_VAR
                bytecode.extend_from_slice(&(name.len() as u8).to_be_bytes());
                bytecode.extend_from_slice(name.as_bytes());
            }
            Expression::Binary { left, op, right } => {
                self.write_expression(left, bytecode);
                self.write_expression(right, bytecode);
                bytecode.push(0x42); // BINARY_OP
                bytecode.push(*op as u8);
            }
            Expression::Call { callee, args } => {
                for arg in args {
                    self.write_expression(arg, bytecode);
                }
                bytecode.push(0x43); // CALL
                bytecode.extend_from_slice(&(args.len() as u8).to_be_bytes());
                self.write_expression(callee, bytecode);
            }
            _ => {}
        }
    }

    /// 写入调试信息
    fn write_debug_info(&self, bytecode: &mut Vec<u8>) {
        // 写入行号表
        bytecode.extend_from_slice(&self.debug_info.line_table.address_to_line.len().to_be_bytes());
        for (addr, line) in &self.debug_info.line_table.address_to_line {
            bytecode.extend_from_slice(&addr.to_be_bytes());
            bytecode.extend_from_slice(&line.to_be_bytes());
        }
    }

    /// 优化字节码
    fn optimize(&mut self, bytecode: Vec<u8>) -> Result<Vec<u8>, CompileError> {
        // 简化的优化：常量折叠
        let mut optimizer = optimizer::ConstantFolder::new();
        optimizer.optimize(bytecode)
    }

    /// 验证字节码
    fn verify_bytecode(&self, bytecode: &[u8]) -> Result<(), CompileError> {
        // 简化验证：检查魔术字节
        if bytecode.len() < 4 {
            return Err(CompileError {
                error_type: CompileErrorType::Other("Invalid bytecode".to_string()),
                message: "Bytecode too short".to_string(),
                location: SourceLocation {
                    file: "".to_string(),
                    line: 0,
                    column: 0,
                },
            });
        }

        if &bytecode[0..4] != IL_MAGIC {
            return Err(CompileError {
                error_type: CompileErrorType::Other("Invalid magic".to_string()),
                message: "Invalid IL magic bytes".to_string(),
                location: SourceLocation {
                    file: "".to_string(),
                    line: 0,
                    column: 0,
                },
            });
        }

        Ok(())
    }

    /// 获取符号
    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.get(name)
    }

    /// 获取函数
    pub fn get_function(&self, index: u32) -> Option<&Function> {
        self.functions.get(index as usize)
    }
}

/// 抽象语法树
#[derive(Debug, Clone)]
pub enum AST {
    /// 模块
    Module { statements: Vec<Statement> },
}

/// 语句
#[derive(Debug, Clone)]
pub enum Statement {
    /// 函数声明
    FunctionDecl {
        name: String,
        params: Vec<Parameter>,
        return_type: Type,
        body: Vec<Statement>,
    },
    /// 变量声明
    VarDecl {
        name: String,
        var_type: Type,
        initial_value: Option<Expression>,
    },
    /// 返回语句
    Return { value: Option<Expression> },
    /// 表达式语句
    Expression { expr: Expression },
    /// 块语句
    Block { statements: Vec<Statement> },
    /// If语句
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    /// While语句
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    /// For语句
    For {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Vec<Statement>,
    },
}

/// 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    /// 字面量
    Literal { literal: Literal },
    /// 标识符
    Identifier { name: String },
    /// 二元运算
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    /// 一元运算
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    /// 函数调用
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    /// 数组索引
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    /// 成员访问
    Member {
        object: Box<Expression>,
        property: String,
    },
}

/// 字面量
#[derive(Debug, Clone)]
pub enum Literal {
    /// null
    Null,
    /// 布尔值
    Bool(bool),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
}

/// 二元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// 一元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// JIT编译器接口
pub trait JITCompiler {
    /// 编译IL字节码为本地代码
    fn compile(&self, bytecode: &[u8]) -> Result<JITCode, JITError>;
    /// 执行编译后的代码
    fn execute(&self, code: &JITCode, args: &[Value]) -> Result<Value, JITError>;
}

/// JIT编译后的代码
pub struct JITCode {
    /// 代码指针
    pub code_ptr: *const u8,
    /// 代码大小
    pub code_size: usize,
}

/// JIT错误
#[derive(Debug, Clone)]
pub enum JITError {
    CompilationError(String),
    ExecutionError(String),
}

/// 运行时值
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

// 模块声明（在实际文件中，这些会独立成文件）
pub mod bytecode;
pub mod codegen;
pub mod debug;
pub mod hot_reload;
pub mod jit;
pub mod optimizer;
pub mod parser;
pub mod vm;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_default() {
        let header = ILHeader::default();
        assert_eq!(&header.magic, IL_MAGIC);
        assert_eq!(header.version, IL_VERSION);
    }

    #[test]
    fn test_compiler_creation() {
        let options = CompileOptions::default();
        let compiler = ILCompiler::new(SourceLanguage::Lua, options);
        assert_eq!(compiler.source_language, SourceLanguage::Lua);
    }
}
