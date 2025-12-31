/// 音频服务模块
pub mod audio;
/// Python脚本服务模块（需要pyo3 feature）
#[cfg(feature = "pyo3")]
pub mod python_scripting;
/// 渲染服务模块
pub mod render;
/// 脚本调试器模块
pub mod script_debugger;
/// 脚本热重载模块
pub mod script_hot_reload;
/// 脚本服务模块
pub mod scripting;

#[cfg(test)]
mod tests;
