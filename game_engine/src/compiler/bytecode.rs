//! IL字节码格式定义
//!
//! 定义中间语言字节码的指令集和格式。

use serde::{Deserialize, Serialize};

/// IL操作码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    // === 栈操作 ===
    /// 加载常量
    LoadConst = 0x40,
    /// 加载变量
    LoadVar = 0x41,
    /// 存储变量
    StoreVar = 0x42,
    /// 复制栈顶
    Dup = 0x43,
    /// 交换栈顶两个元素
    Swap = 0x44,
    /// 弹出栈顶
    Pop = 0x45,

    // === 算术运算 ===
    /// 加法
    Add = 0x50,
    /// 减法
    Sub = 0x51,
    /// 乘法
    Mul = 0x52,
    /// 除法
    Div = 0x53,
    /// 取模
    Mod = 0x54,
    /// 取负
    Neg = 0x55,

    // === 位运算 ===
    /// 按位与
    BitAnd = 0x60,
    /// 按位或
    BitOr = 0x61,
    /// 按位异或
    BitXor = 0x62,
    /// 按位取反
    BitNot = 0x63,
    /// 左移
    ShiftLeft = 0x64,
    /// 右移
    ShiftRight = 0x65,

    // === 比较运算 ===
    /// 相等
    Eq = 0x70,
    /// 不等
    Ne = 0x71,
    /// 小于
    Lt = 0x72,
    /// 小于等于
    Le = 0x73,
    /// 大于
    Gt = 0x74,
    /// 大于等于
    Ge = 0x75,

    // === 逻辑运算 ===
    /// 逻辑与
    LogicalAnd = 0x80,
    /// 逻辑或
    LogicalOr = 0x81,
    /// 逻辑非
    LogicalNot = 0x82,

    // === 控制流 ===
    /// 跳转
    Jump = 0x90,
    /// 条件跳转（为假时跳转）
    JumpIfFalse = 0x91,
    /// 条件跳转（为真时跳转）
    JumpIfTrue = 0x92,
    /// 调用函数
    Call = 0x93,
    /// 返回
    Return = 0x94,
    /// 返回值
    ReturnValue = 0x95,

    // === 数组/对象操作 ===
    /// 数组取值
    Index = 0xA0,
    /// 数组赋值
    SetIndex = 0xA1,
    /// 成员访问
    GetMember = 0xA2,
    /// 成员赋值
    SetMember = 0xA3,

    // === 函数操作 ===
    /// 定义函数
    FunctionDef = 0x10,
    /// 结束函数定义
    EndFunction = 0x11,
    /// 定义变量
    VarDecl = 0x20,
    /// 匿名函数
    Closure = 0xB0,

    // === 其他 ===
    /// 空操作
    Nop = 0x00,
}

/// 指令
#[derive(Debug, Clone)]
pub struct Instruction {
    /// 操作码
    pub opcode: Opcode,
    /// 操作数
    pub operands: Vec<Operand>,
    /// 源位置
    pub source_location: Option<SourceLocation>,
}

/// 操作数
#[derive(Debug, Clone)]
pub enum Operand {
    /// 无操作数
    None,
    /// 8位整数
    U8(u8),
    /// 16位整数
    U16(u16),
    /// 32位整数
    U32(u32),
    /// 64位整数
    U64(u64),
    /// 字符串
    String(String),
    /// 标签
    Label(String),
}

/// 源位置
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// 字节码流
#[derive(Debug, Clone)]
pub struct BytecodeStream {
    /// 字节码数据
    pub data: Vec<u8>,
    /// 标签表
    pub labels: std::collections::HashMap<String, usize>,
}

impl BytecodeStream {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            labels: std::collections::HashMap::new(),
        }
    }

    pub fn emit(&mut self, opcode: Opcode) {
        self.data.push(opcode as u8);
    }

    pub fn emit_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn emit_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn emit_u64(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn emit_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn define_label(&mut self, label: String) {
        self.labels.insert(label, self.data.len());
    }

    pub fn emit_label_ref(&mut self, label: &str) {
        if let Some(&pos) = self.labels.get(label) {
            self.emit_u32(pos as u32);
        }
    }
}
