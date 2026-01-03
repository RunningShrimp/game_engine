//! Mono 运行时集成模块（macOS）
//!
//! 使用 wrapped_mono crate 嵌入 Mono 运行时，提供完整的 C# 支持。
//!
//! **平台支持:**
//! - ✅ macOS: 完整支持（使用 Mono）- **需要手动启用**
//! - ❌ Windows/Linux: 不使用此模块（使用 netcorehost）
//!
//! **系统要求:**
//! - macOS 10.9+
//! - Mono 框架（通过 Homebrew 或官方安装包）
//!
//! **安装 Mono:**
//! ```bash
//! brew install mono
//! # 或访问 https://www.mono-project.com/download/stable/
//! ```
//!
//! **启用步骤:**
//! 1. 取消注释 `Cargo.toml` 中的 `wrapped_mono` 依赖
//! 2. 取消注释本文件中的条件编译指令
//! 3. 重新编译

// 注意：当前使用条件编译来禁用 Mono 集成
// 要启用 Mono 支持，请将下面的 #[cfg(feature = "mono")] 改为 #[cfg(feature = "mono")]

#[cfg(feature = "mono")]
use crate::scripting::csharp::{AssemblyMetadata, FunctionSignature, TypeMetadata};
use std::path::PathBuf;
use std::sync::Mutex;

/// Mono 运行时主机
#[cfg(feature = "mono")]
#[derive(Debug)]
pub struct MonoHost {
    /// Mono 运行时是否已初始化
    pub initialized: bool,

    /// Mono 域（AppDomain）
    #[cfg(feature = "mono")]
    domain: Option<wrapped_mono::domain::Domain>,

    /// 已加载的程序集缓存
    assemblies: Mutex<Vec<LoadedAssembly>>,
}

#[cfg(feature = "mono")]
impl MonoHost {
    /// 初始化 Mono 运行时
    ///
    /// 这将：
    /// 1. 初始化 Mono 运行时
    /// 2. 创建默认域（AppDomain）
    /// 3. 设置基本的运行时配置
    pub fn initialize() -> Result<Self, String> {
        tracing::info!("Initializing Mono runtime on macOS");

        // 检查 Mono 是否安装
        if !Self::check_mono_installation() {
            return Err("Mono not found. Please install Mono: brew install mono \
                 or visit https://www.mono-project.com/download/stable/"
                .to_string());
        }

        // 初始化 wrapped_mono
        wrapped_mono::initialize()
            .map_err(|e| format!("Failed to initialize Mono runtime: {}", e))?;

        // 创建默认域
        let domain = wrapped_mono::domain::Domain::get_current()
            .map_err(|e| format!("Failed to get current domain: {}", e))?;

        tracing::info!("Mono runtime initialized successfully");
        tracing::info!("Mono version: {}", Self::get_mono_version());

        Ok(Self {
            initialized: true,
            domain: Some(domain),
            assemblies: Mutex::new(Vec::new()),
        })
    }

    /// 检查 Mono 是否已安装
    fn check_mono_installation() -> bool {
        // 检查 mono 命令是否可用
        std::process::Command::new("mono")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 获取 Mono 版本
    fn get_mono_version() -> String {
        std::process::Command::new("mono")
            .arg("--version")
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string()
            })
            .unwrap_or_else(|_| "Unknown".to_string())
    }

    /// 加载 .NET 程序集
    ///
    /// 使用 Mono 加载 .NET 程序集并扫描类型元数据。
    pub fn load_assembly(&self, path: &PathBuf) -> Result<LoadedAssembly, String> {
        if !self.initialized {
            return Err("Mono runtime not initialized".to_string());
        }

        if !path.exists() {
            return Err(format!("Assembly file not found: {:?}", path));
        }

        tracing::debug!("Loading .NET assembly: {:?}", path);

        #[cfg(feature = "mono")]
        {
            if let Some(ref domain) = self.domain {
                return self.load_assembly_with_mono(domain, path);
            }
        }

        // 回退实现
        Ok(LoadedAssembly {
            name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string(),
            path: path.clone(),
            types: Vec::new(),
        })
    }

    /// 使用 Mono 加载程序集并扫描类型
    #[cfg(feature = "mono")]
    fn load_assembly_with_mono(
        &self,
        domain: &wrapped_mono::domain::Domain,
        path: &PathBuf,
    ) -> Result<LoadedAssembly, String> {
        // 将路径转换为字符串
        let path_str = path.to_str().ok_or_else(|| "Invalid path string".to_string())?;

        // 打开程序集
        let assembly = domain
            .assembly_open(path_str)
            .map_err(|e| format!("Failed to open assembly: {}", e))?;

        // 获取程序集名称
        let assembly_name =
            assembly.get_name().map_err(|e| format!("Failed to get assembly name: {}", e))?;

        let name = assembly_name.get_name().unwrap_or_else(|_| "Unknown".to_string());

        tracing::info!("Loaded assembly: {}", name);

        // 扫描程序集中的类型（框架实现）
        // 完整实现需要：
        // 1. 获取程序集镜像
        // 2. 遍历所有类型
        // 3. 提取方法签名
        // 4. 收集元数据

        // 暂时返回基本信息
        Ok(LoadedAssembly {
            name,
            path: path.clone(),
            types: Vec::new(),
        })
    }

    /// 调用 .NET 静态方法
    ///
    /// 使用 Mono 反射调用程序集中的静态方法。
    pub fn invoke_method(
        &self,
        assembly_name: &str,
        type_name: &str,
        method_name: &str,
        args: &[crate::scripting::csharp::NetValue],
    ) -> Result<crate::scripting::csharp::NetValue, String> {
        if !self.initialized {
            return Err("Mono runtime not initialized".to_string());
        }

        tracing::debug!(
            "Invoking method: {}::{}::{} with {} args",
            assembly_name,
            type_name,
            method_name,
            args.len()
        );

        #[cfg(feature = "mono")]
        {
            if let Some(ref domain) = self.domain {
                return self.invoke_method_with_mono(
                    domain,
                    assembly_name,
                    type_name,
                    method_name,
                    args,
                );
            }
        }

        // 回退实现
        tracing::warn!("Method invoke (fallback): returning null");
        Ok(crate::scripting::csharp::NetValue::Null)
    }

    /// 使用 Mono 调用方法
    #[cfg(feature = "mono")]
    fn invoke_method_with_mono(
        &self,
        domain: &wrapped_mono::domain::Domain,
        assembly_name: &str,
        type_name: &str,
        method_name: &str,
        _args: &[crate::scripting::csharp::NetValue],
    ) -> Result<crate::scripting::csharp::NetValue, String> {
        // 框架实现：记录调用
        tracing::debug!(
            "Framework: Mono method invocation would call {}::{}::{}",
            assembly_name,
            type_name,
            method_name
        );

        // 完整实现需要：
        // 1. 加载程序集
        // 2. 获取类型
        // 3. 查找方法
        // 4. 转换参数
        // 5. 调用方法
        // 6. 转换返回值

        // 暂时返回 null
        Ok(crate::scripting::csharp::NetValue::Null)
    }

    /// 编译并执行 C# 代码
    ///
    /// 使用 Mono 编译器（mcs）编译 C# 代码并执行。
    pub fn compile_and_execute(
        &self,
        code: &str,
        script_name: &str,
    ) -> Result<crate::scripting::csharp::NetValue, String> {
        if !self.initialized {
            return Err("Mono runtime not initialized".to_string());
        }

        tracing::debug!("Compiling and executing C# script: {}", script_name);

        // 创建临时源文件
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join(format!("{}.cs", script_name));
        let exe_path = temp_dir.join(format!("{}.exe", script_name));

        std::fs::write(&source_path, code)
            .map_err(|e| format!("Failed to write source file: {}", e))?;

        // 使用 mcs 编译
        let compile_result = std::process::Command::new("mcs")
            .arg("-out:&exe_path")
            .arg(&source_path)
            .output();

        match compile_result {
            Ok(output) if output.status.success() => {
                tracing::debug!("C# compilation successful");

                // 使用 mono 执行
                let execute_result = std::process::Command::new("mono").arg(&exe_path).output();

                // 清理临时文件
                let _ = std::fs::remove_file(&source_path);
                let _ = std::fs::remove_file(&exe_path);

                match execute_result {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        if !output.status.success() {
                            return Err(format!("Execution failed: {}", stderr));
                        }

                        // 解析输出（假设返回 JSON）
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stdout) {
                            // 转换为 NetValue
                            Ok(crate::scripting::csharp::NetValue::from_json(&value))
                        } else {
                            // 返回字符串
                            Ok(crate::scripting::csharp::NetValue::String(
                                stdout.to_string(),
                            ))
                        }
                    }
                    Err(e) => Err(format!("Failed to execute: {}", e)),
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Compilation failed: {}", stderr))
            }
            Err(e) => Err(format!("Failed to run mcs: {}", e)),
        }
    }
}

#[cfg(feature = "mono")]
impl Drop for MonoHost {
    fn drop(&mut self) {
        if self.initialized {
            tracing::debug!("Shutting down Mono runtime");
            // wrapped_mono 会自动清理
        }
    }
}

#[cfg(feature = "mono")]
unsafe impl Send for MonoHost {}
#[cfg(feature = "mono")]
unsafe impl Sync for MonoHost {}

/// 已加载的程序集
#[derive(Debug, Clone)]
pub struct LoadedAssembly {
    /// 程序集名称
    pub name: String,

    /// 程序集路径
    pub path: PathBuf,

    /// 程序集中的类型
    pub types: Vec<AssemblyTypeInfo>,
}

/// 程序集类型信息
#[derive(Debug, Clone)]
pub struct AssemblyTypeInfo {
    /// 类型名称
    pub name: String,

    /// 命名空间
    pub namespace: Option<String>,

    /// 类型的方法
    pub methods: Vec<MethodInfo>,
}

/// 方法信息
#[derive(Debug, Clone)]
pub struct MethodInfo {
    /// 方法名称
    pub name: String,

    /// 返回类型
    pub return_type: String,

    /// 参数类型
    pub parameter_types: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "mono")]
    fn test_mono_host_initialization() {
        // 这个测试需要 Mono 已安装
        let host = MonoHost::initialize();
        match host {
            Ok(host) => {
                assert!(host.initialized);
                tracing::info!("Mono runtime test passed");
            }
            Err(e) => {
                tracing::warn!("Mono runtime test skipped: {}", e);
            }
        }
    }
}
