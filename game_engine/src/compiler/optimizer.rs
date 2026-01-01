//! IL字节码优化器
//!
//! 提供各种优化通道以改进生成的IL字节码性能。

use super::bytecode::{Opcode, Instruction, BytecodeStream};
use super::CompileError;

/// 优化器trait
pub trait Optimizer {
    fn optimize(&mut self, bytecode: Vec<u8>) -> Result<Vec<u8>, CompileError>;
}

/// 常量折叠优化
pub struct ConstantFolder {
    passes: u32,
}

impl ConstantFolder {
    pub fn new() -> Self {
        Self { passes: 1 }
    }

    pub fn with_passes(mut self, passes: u32) -> Self {
        self.passes = passes;
        self
    }
}

impl Default for ConstantFolder {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer for ConstantFolder {
    fn optimize(&mut self, bytecode: Vec<u8>) -> Result<Vec<u8>, CompileError> {
        // 简化的常量折叠实现
        let mut optimized = bytecode.clone();
        
        for _ in 0..self.passes {
            optimized = self.fold_constants(optimized)?;
        }

        Ok(optimized)
    }
}

impl ConstantFolder {
    fn fold_constants(&self, bytecode: Vec<u8>) -> Result<Vec<u8>, CompileError> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            let opcode = bytecode[i];

            // 检查是否为算术运算指令
            if matches!(opcode, 0x50 | 0x51 | 0x52 | 0x53) {
                // 检查前两个指令是否为常量加载
                if i >= 9 {
                    // 简化：假设前面是两个常量
                    let result_opcode = match opcode {
                        0x50 => Some((0x50, self.add_consts(
                            &bytecode[i - 8..i],
                            &bytecode[i - 16..i - 8],
                        ))),
                        0x51 => Some((0x51, self.sub_consts(
                            &bytecode[i - 8..i],
                            &bytecode[i - 16..i - 8],
                        ))),
                        0x52 => Some((0x52, self.mul_consts(
                            &bytecode[i - 8..i],
                            &bytecode[i - 16..i - 8],
                        ))),
                        0x53 => Some((0x53, self.div_consts(
                            &bytecode[i - 8..i],
                            &bytecode[i - 16..i - 8],
                        ))),
                        _ => None,
                    };

                    if let Some((op, folded)) = result_opcode {
                        // 回退两个常量加载指令
                        for _ in 0..17 {
                            result.pop();
                        }
                        // 添加折叠后的常量
                        result.extend_from_slice(&folded);
                        result.push(op);
                        i += 1;
                        continue;
                    }
                }
            }

            result.push(opcode);
            i += 1;
        }

        Ok(result)
    }

    fn add_consts(&self, left: &[u8], right: &[u8]) -> Vec<u8> {
        // 简化：假设两个f64常量
        if left[0] == 0x04 && right[0] == 0x04 {
            let a = f64::from_be_bytes(left[1..9].try_into().unwrap());
            let b = f64::from_be_bytes(right[1..9].try_into().unwrap());
            let result = a + b;
            let mut bytes = vec![0x04];
            bytes.extend_from_slice(&result.to_be_bytes());
            bytes
        } else {
            // 返回原字节
            let mut result = Vec::new();
            result.extend_from_slice(left);
            result.extend_from_slice(right);
            result
        }
    }

    fn sub_consts(&self, left: &[u8], right: &[u8]) -> Vec<u8> {
        if left[0] == 0x04 && right[0] == 0x04 {
            let a = f64::from_be_bytes(left[1..9].try_into().unwrap());
            let b = f64::from_be_bytes(right[1..9].try_into().unwrap());
            let result = a - b;
            let mut bytes = vec![0x04];
            bytes.extend_from_slice(&result.to_be_bytes());
            bytes
        } else {
            let mut result = Vec::new();
            result.extend_from_slice(left);
            result.extend_from_slice(right);
            result
        }
    }

    fn mul_consts(&self, left: &[u8], right: &[u8]) -> Vec<u8> {
        if left[0] == 0x04 && right[0] == 0x04 {
            let a = f64::from_be_bytes(left[1..9].try_into().unwrap());
            let b = f64::from_be_bytes(right[1..9].try_into().unwrap());
            let result = a * b;
            let mut bytes = vec![0x04];
            bytes.extend_from_slice(&result.to_be_bytes());
            bytes
        } else {
            let mut result = Vec::new();
            result.extend_from_slice(left);
            result.extend_from_slice(right);
            result
        }
    }

    fn div_consts(&self, left: &[u8], right: &[u8]) -> Vec<u8> {
        if left[0] == 0x04 && right[0] == 0x04 {
            let a = f64::from_be_bytes(left[1..9].try_into().unwrap());
            let b = f64::from_be_bytes(right[1..9].try_into().unwrap());
            if b != 0.0 {
                let result = a / b;
                let mut bytes = vec![0x04];
                bytes.extend_from_slice(&result.to_be_bytes());
                bytes
            } else {
                let mut result = Vec::new();
                result.extend_from_slice(left);
                result.extend_from_slice(right);
                result
            }
        } else {
            let mut result = Vec::new();
            result.extend_from_slice(left);
            result.extend_from_slice(right);
            result
        }
    }
}

/// 死代码消除
pub struct DeadCodeEliminator;

impl DeadCodeEliminator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeadCodeEliminator {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer for DeadCodeEliminator {
    fn optimize(&mut self, bytecode: Vec<u8>) -> Result<Vec<u8>, CompileError> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            let opcode = bytecode[i];

            // 移除NOP指令
            if opcode == 0x00 {
                i += 1;
                continue;
            }

            // 移除无用的POP后跟PUSH
            if opcode == 0x45 && i + 1 < bytecode.len() && bytecode[i + 1] == 0x40 {
                i += 2;
                continue;
            }

            result.push(opcode);
            i += 1;
        }

        Ok(result)
    }
}

/// 内联优化
pub struct Inliner {
    /// 最大内联大小（字节码指令数）
    max_inline_size: usize,
}

impl Inliner {
    pub fn new() -> Self {
        Self {
            max_inline_size: 10,
        }
    }

    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_inline_size = size;
        self
    }
}

impl Default for Inliner {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer for Inliner {
    fn optimize(&mut self, bytecode: Vec<u8>) -> Result<Vec<u8>, CompileError> {
        // 简化的内联实现（仅作示例）
        Ok(bytecode)
    }
}

/// 优化管理器
pub struct OptimizationManager {
    optimizers: Vec<Box<dyn Optimizer>>,
}

impl OptimizationManager {
    pub fn new() -> Self {
        Self {
            optimizers: Vec::new(),
        }
    }

    pub fn add_optimizer(&mut self, optimizer: Box<dyn Optimizer>) -> &mut Self {
        self.optimizers.push(optimizer);
        self
    }

    pub fn run_all(&mut self, mut bytecode: Vec<u8>) -> Result<Vec<u8>, CompileError> {
        for optimizer in &mut self.optimizers {
            bytecode = optimizer.optimize(bytecode)?;
        }
        Ok(bytecode)
    }

    pub fn with_standard_optimizations(level: u8) -> Self {
        let mut manager = Self::new();

        match level {
            0 => {
                // 无优化
            }
            1 => {
                // 基础优化
                manager.add_optimizer(Box::new(ConstantFolder::new()));
                manager.add_optimizer(Box::new(DeadCodeEliminator));
            }
            2 => {
                // 中等优化（默认）
                manager.add_optimizer(Box::new(ConstantFolder::with_passes(2)));
                manager.add_optimizer(Box::new(DeadCodeEliminator));
                manager.add_optimizer(Box::new(Inliner::new()));
            }
            3 => {
                // 激进优化
                manager.add_optimizer(Box::new(ConstantFolder::with_passes(3)));
                manager.add_optimizer(Box::new(DeadCodeEliminator));
                manager.add_optimizer(Box::new(Inliner::with_max_size(20)));
            }
            _ => {
                // 默认为中等优化
                manager.add_optimizer(Box::new(ConstantFolder::with_passes(2)));
                manager.add_optimizer(Box::new(DeadCodeEliminator));
            }
        }

        manager
    }
}

impl Default for OptimizationManager {
    fn default() -> Self {
        Self::with_standard_optimizations(2)
    }
}
