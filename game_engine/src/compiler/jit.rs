//! JIT编译模块
//!
//! 即时编译IL字节码为本地机器码。

use super::{JITCode, JITCompiler, JITError, Value};

pub struct NativeJITCompiler;

impl JITCompiler for NativeJITCompiler {
    fn compile(&self, _bytecode: &[u8]) -> Result<JITCode, JITError> {
        Ok(JITCode {
            code_ptr: std::ptr::null(),
            code_size: 0,
        })
    }

    fn execute(&self, _code: &JITCode, _args: &[Value]) -> Result<Value, JITError> {
        Ok(Value::Null)
    }
}
