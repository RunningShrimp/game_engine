//! IL虚拟机
//!
//! 执行IL字节码的虚拟机实现。

use super::bytecode::{Opcode, Instruction};
use std::collections::HashMap;

/// 虚拟机
pub struct VirtualMachine {
    /// 栈
    stack: Vec<Value>,
    /// 调用栈
    call_stack: Vec<CallFrame>,
    /// 全局变量
    globals: HashMap<String, Value>,
    /// 当前函数索引
    current_function: usize,
    /// 程序计数器
    pc: usize,
}

/// 值
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

/// 调用帧
#[derive(Debug, Clone)]
struct CallFrame {
    /// 函数索引
    function_index: usize,
    /// 返回地址
    return_address: usize,
    /// 局部变量
    locals: Vec<Value>,
    /// 栈基址
    stack_base: usize,
}

/// 执行错误
#[derive(Debug, Clone)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    DivisionByZero,
    TypeError(String),
    UndefinedVariable(String),
    UndefinedFunction(String),
    Other(String),
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            call_stack: Vec::new(),
            globals: HashMap::new(),
            current_function: 0,
            pc: 0,
        }
    }

    pub fn execute(&mut self, bytecode: &[u8]) -> Result<Value, VMError> {
        // 简化的执行循环
        while self.pc < bytecode.len() {
            let opcode = bytecode[self.pc];
            let opcode_enum = match Self::decode_opcode(opcode) {
                Some(op) => op,
                None => {
                    return Err(VMError::Other(format!(
                        "Unknown opcode: 0x{:02X}",
                        opcode
                    )))
                }
            };

            self.execute_instruction(opcode_enum, bytecode)?;
        }

        Ok(Value::Null)
    }

    fn decode_opcode(byte: u8) -> Option<Opcode> {
        match byte {
            0x00 => Some(Opcode::Nop),
            0x10 => Some(Opcode::FunctionDef),
            0x11 => Some(Opcode::EndFunction),
            0x20 => Some(Opcode::VarDecl),
            0x40 => Some(Opcode::LoadConst),
            0x41 => Some(Opcode::LoadVar),
            0x42 => Some(Opcode::StoreVar),
            0x43 => Some(Opcode::Dup),
            0x50 => Some(Opcode::Add),
            0x51 => Some(Opcode::Sub),
            0x52 => Some(Opcode::Mul),
            0x53 => Some(Opcode::Div),
            0x70 => Some(Opcode::Eq),
            0x90 => Some(Opcode::Jump),
            0x91 => Some(Opcode::JumpIfFalse),
            0x93 => Some(Opcode::Call),
            0x94 => Some(Opcode::Return),
            _ => None,
        }
    }

    fn execute_instruction(&mut self, opcode: Opcode, bytecode: &[u8]) -> Result<(), VMError> {
        self.pc += 1;

        match opcode {
            Opcode::LoadConst => {
                let value = self.read_constant(bytecode)?;
                self.stack.push(value);
            }
            Opcode::Add => {
                let right = self.pop()?;
                let left = self.pop()?;
                let result = self.add(&left, &right)?;
                self.stack.push(result);
            }
            Opcode::Sub => {
                let right = self.pop()?;
                let left = self.pop()?;
                let result = self.sub(&left, &right)?;
                self.stack.push(result);
            }
            Opcode::Mul => {
                let right = self.pop()?;
                let left = self.pop()?;
                let result = self.mul(&left, &right)?;
                self.stack.push(result);
            }
            Opcode::Div => {
                let right = self.pop()?;
                let left = self.pop()?;
                let result = self.div(&left, &right)?;
                self.stack.push(result);
            }
            Opcode::Eq => {
                let right = self.pop()?;
                let left = self.pop()?;
                let result = Value::Bool(left == right);
                self.stack.push(result);
            }
            Opcode::Jump => {
                let offset = self.read_u32(bytecode)? as usize;
                self.pc = offset;
            }
            Opcode::JumpIfFalse => {
                let offset = self.read_u32(bytecode)? as usize;
                let condition = self.pop()?;
                match condition {
                    Value::Bool(false) | Value::Null => {
                        self.pc = offset;
                    }
                    _ => {}
                }
            }
            Opcode::Return => {
                return Ok(());
            }
            Opcode::Nop => {}
            _ => {
                // 未实现的操作码
            }
        }

        Ok(())
    }

    fn pop(&mut self) -> Result<Value, VMError> {
        self.stack.pop().ok_or(VMError::StackUnderflow)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn read_constant(&mut self, bytecode: &[u8]) -> Result<Value, VMError> {
        let type_byte = bytecode[self.pc];
        self.pc += 1;

        match type_byte {
            0x00 => Ok(Value::Null),
            0x01 => Ok(Value::Bool(true)),
            0x02 => Ok(Value::Bool(false)),
            0x03 => {
                let bytes = &bytecode[self.pc..self.pc + 8];
                self.pc += 8;
                let value = i64::from_be_bytes(bytes.try_into().unwrap());
                Ok(Value::Integer(value))
            }
            0x04 => {
                let bytes = &bytecode[self.pc..self.pc + 8];
                self.pc += 8;
                let value = f64::from_be_bytes(bytes.try_into().unwrap());
                Ok(Value::Float(value))
            }
            0x05 => {
                let len_bytes = &bytecode[self.pc..self.pc + 4];
                self.pc += 4;
                let len = u32::from_be_bytes(len_bytes.try_into().unwrap()) as usize;
                let bytes = &bytecode[self.pc..self.pc + len];
                self.pc += len;
                let string = String::from_utf8(bytes.to_vec()).unwrap();
                Ok(Value::String(string))
            }
            _ => Err(VMError::Other(format!("Unknown constant type: 0x{:02X}", type_byte))),
        }
    }

    fn read_u32(&mut self, bytecode: &[u8]) -> Result<u32, VMError> {
        let bytes = &bytecode[self.pc..self.pc + 4];
        self.pc += 4;
        Ok(u32::from_be_bytes(bytes.try_into().unwrap()))
    }

    fn add(&self, left: &Value, right: &Value) -> Result<Value, VMError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            _ => Err(VMError::TypeError(format!(
                "Cannot add {:?} and {:?}",
                left, right
            ))),
        }
    }

    fn sub(&self, left: &Value, right: &Value) -> Result<Value, VMError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            _ => Err(VMError::TypeError(format!(
                "Cannot subtract {:?} from {:?}",
                right, left
            ))),
        }
    }

    fn mul(&self, left: &Value, right: &Value) -> Result<Value, VMError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            _ => Err(VMError::TypeError(format!(
                "Cannot multiply {:?} by {:?}",
                left, right
            ))),
        }
    }

    fn div(&self, left: &Value, right: &Value) -> Result<Value, VMError> {
        match (left, right) {
            (Value::Integer(_), Value::Integer(0)) => Err(VMError::DivisionByZero),
            (Value::Float(_), Value::Float(0.0)) => Err(VMError::DivisionByZero),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
            _ => Err(VMError::TypeError(format!(
                "Cannot divide {:?} by {:?}",
                left, right
            ))),
        }
    }
}
