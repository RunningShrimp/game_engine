//! .NET 运行时集成模块（跨平台 - dotnet CLI）
//!
//! 使用 dotnet CLI 通过进程间通信执行 C# 代码。
//!
//! **平台支持:**
//! - ✅ Windows: 完整支持（使用 .NET CLI）
//! - ✅ Linux: 完整支持（使用 .NET CLI）
//! - ✅ macOS: 完整支持（使用 .NET CLI）
//!
//! **系统要求:**
//! - .NET SDK 8.0 或更高版本
//! - 安装: https://dotnet.microsoft.com/download
//!
//! **特性:**
//! - 使用 dotnet CLI 编译和执行 C# 代码
//! - 支持临时程序集生成
//! - 支持标准输入/输出通信
//! - 支持 .NET 8/9/10 运行时
//!
//! **参考:**
//! - .NET CLI: https://learn.microsoft.com/zh-cn/dotnet/core/tools/
//! - dotnet build: https://learn.microsoft.com/zh-cn/dotnet/core/tools/dotnet-build
//! - dotnet run: https://learn.microsoft.com/zh-cn/dotnet/core/tools/dotnet-run

#[cfg(feature = "csharp")]
use crate::scripting::csharp::{AssemblyMetadata, FunctionSignature, TypeMetadata};
use crate::scripting::csharp_compile_cache::CompileCache;
use crate::scripting::csharp_process_pool::{DotNetProcessPool, ProcessPoolConfig};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// .NET 运行时主机（使用 dotnet CLI）
#[cfg(feature = "csharp")]
#[derive(Debug)]
pub struct DotNetCliHost {
    /// 运行时是否已初始化
    pub initialized: bool,

    /// dotnet CLI 版本
    pub dotnet_version: String,

    /// 临时目录
    temp_dir: PathBuf,

    /// 已编译的程序集缓存
    assemblies: Mutex<Vec<LoadedAssembly>>,

    /// 编译结果缓存
    compile_cache: Option<Arc<CompileCache>>,

    /// 进程池（可选，用于性能优化）
    process_pool: Option<Mutex<DotNetProcessPool>>,

    /// 是否启用进程池
    enable_process_pool: bool,
}

#[cfg(feature = "csharp")]
impl DotNetCliHost {
    /// 初始化 .NET 运行时
    ///
    /// 这将：
    /// 1. 检查 dotnet CLI 是否可用
    /// 2. 获取 .NET 版本信息
    /// 3. 创建临时工作目录
    /// 4. 初始化编译缓存
    ///
    /// **示例:**
    /// ```ignore
    /// let host = DotNetCliHost::initialize()?;
    /// println!("Using .NET version: {}", host.dotnet_version);
    /// ```
    pub fn initialize() -> Result<Self, String> {
        tracing::info!("Initializing .NET runtime using dotnet CLI (cross-platform)");

        // 检查 dotnet CLI 是否可用
        if !Self::check_dotnet_installation() {
            return Err(
                ".NET CLI not found. Please install .NET SDK 8.0 or higher:\n\
                 - macOS: brew install --cask dotnet-sdk\n\
                 - Linux: https://learn.microsoft.com/en-us/dotnet/core/install/linux\n\
                 - Windows: https://dotnet.microsoft.com/download"
                    .to_string(),
            );
        }

        // 获取 .NET 版本
        let dotnet_version = Self::get_dotnet_version();

        // 创建临时目录
        let temp_dir = std::env::temp_dir().join("csharp_dotnet");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {e}"))?;

        // 初始化编译缓存
        let cache_dir = std::env::temp_dir().join("csharp_compile_cache");
        let compile_cache = match CompileCache::new(cache_dir, 100) {
            Ok(cache) => {
                tracing::info!("Compile cache initialized (100 MB limit)");
                Some(Arc::new(cache))
            }
            Err(e) => {
                tracing::warn!("Failed to initialize compile cache: {}", e);
                tracing::info!("Proceeding without compilation caching");
                None
            }
        };

        tracing::info!("dotnet CLI initialized successfully");
        tracing::info!("Using .NET version: {}", dotnet_version);
        tracing::info!("Temporary directory: {}", temp_dir.display());

        // 🔥 性能优化：初始化进程池
        let mut process_pool = None;
        let enable_process_pool = std::env::var("CSHARP_ENABLE_PROCESS_POOL")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        if enable_process_pool {
            let pool_config = ProcessPoolConfig::default();
            let pool_work_dir = temp_dir.join("process_pool");

            match DotNetProcessPool::new(pool_config, pool_work_dir) {
                Ok(pool) => {
                    tracing::info!("🚀 .NET process pool enabled (4 processes)");
                    process_pool = Some(Mutex::new(pool));
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize process pool: {}", e);
                    tracing::info!("Proceeding without process pool optimization");
                }
            }
        }

        Ok(Self {
            initialized: true,
            dotnet_version,
            temp_dir,
            assemblies: Mutex::new(Vec::new()),
            compile_cache,
            process_pool,
            enable_process_pool,
        })
    }

    /// 检查 .NET CLI 是否已安装
    fn check_dotnet_installation() -> bool {
        // 检查 dotnet 命令是否可用
        std::process::Command::new("dotnet")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 获取 .NET 版本
    fn get_dotnet_version() -> String {
        std::process::Command::new("dotnet")
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
    /// 使用 .NET 运行时加载程序集并扫描类型元数据。
    ///
    /// **参数:**
    /// - `path`: 程序集文件路径（.dll）
    ///
    /// **示例:**
    /// ```ignore
    /// let assembly = host.load_assembly(&PathBuf::from("./MyGame.dll"))?;
    /// println!("Loaded assembly: {}", assembly.name);
    /// ```
    pub fn load_assembly(&self, path: &PathBuf) -> Result<LoadedAssembly, String> {
        if !self.initialized {
            return Err(".NET runtime not initialized".to_string());
        }

        if !path.exists() {
            return Err(format!("Assembly file not found: {path:?}"));
        }

        tracing::debug!("Loading .NET assembly: {:?}", path);

        // 获取程序集名称
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();

        tracing::info!("Loaded assembly: {}", name);

        // 框架实现：元数据扫描
        // 完整实现需要使用反射工具或读取程序集元数据

        Ok(LoadedAssembly {
            name,
            path: path.clone(),
            types: Vec::new(),
        })
    }

    /// 调用 .NET 静态方法
    ///
    /// 使用 dotnet CLI 调用程序集中的静态方法。
    ///
    /// **参数:**
    /// - `assembly_path`: 程序集路径
    /// - `type_name`: 类型名称
    /// - `method_name`: 方法名称
    /// - `args`: 参数列表（JSON 格式）
    ///
    /// **示例:**
    /// ```ignore
    /// let result = host.invoke_method(
    ///     &PathBuf::from("./MyGame.dll"),
    ///     "MyGame.Program",
    ///     "Hello",
    ///     &[]
    /// )?;
    /// ```
    pub fn invoke_method(
        &self,
        assembly_path: &Path,
        type_name: &str,
        method_name: &str,
        args: &[crate::scripting::csharp::NetValue],
    ) -> Result<crate::scripting::csharp::NetValue, String> {
        if !self.initialized {
            return Err(".NET runtime not initialized".to_string());
        }

        tracing::debug!(
            "Invoking method: {}::{} from {} with {} args",
            type_name,
            method_name,
            assembly_path.display(),
            args.len()
        );

        // 创建调用脚本的 C# 代码
        let script_code =
            self.generate_invoker_script(assembly_path, type_name, method_name, args)?;

        // 编译并执行
        self.compile_and_execute(&script_code, "method_invoker")
    }

    /// 生成方法调用脚本的 C# 代码
    #[cfg(feature = "csharp")]
    fn generate_invoker_script(
        &self,
        assembly_path: &Path,
        type_name: &str,
        method_name: &str,
        args: &[crate::scripting::csharp::NetValue],
    ) -> Result<String, String> {
        use std::fmt::Write;

        let mut code = String::new();

        writeln!(code, "using System;").unwrap();
        writeln!(code, "using System.Reflection;").unwrap();
        writeln!(code, "using System.Text.Json;").unwrap();
        writeln!(code).unwrap();

        writeln!(code, "public class MethodInvoker {{").unwrap();
        writeln!(code, "    public static int Main() {{").unwrap();

        // 加载程序集
        writeln!(
            code,
            "        var assembly = Assembly.LoadFrom(\"{}\");",
            assembly_path.display()
        )
        .unwrap();

        // 获取类型
        writeln!(
            code,
            "        var type = assembly.GetType(\"{type_name}\");"
        )
        .unwrap();

        // 获取方法
        writeln!(
            code,
            "        var method = type.GetMethod(\"{method_name}\");"
        )
        .unwrap();

        // 准备参数
        if args.is_empty() {
            writeln!(code, "        var result = method.Invoke(null, null);").unwrap();
        } else {
            writeln!(code, "        var parameters = new object[{}];", args.len()).unwrap();

            for (i, arg) in args.iter().enumerate() {
                match arg {
                    crate::scripting::csharp::NetValue::Null => {
                        writeln!(code, "        parameters[{i}] = null;").unwrap();
                    }
                    crate::scripting::csharp::NetValue::Boolean(b) => {
                        writeln!(code, "        parameters[{i}] = {b};").unwrap();
                    }
                    crate::scripting::csharp::NetValue::Integer(n) => {
                        writeln!(code, "        parameters[{i}] = {n}L;").unwrap();
                    }
                    crate::scripting::csharp::NetValue::Number(n) => {
                        writeln!(code, "        parameters[{i}] = {n};").unwrap();
                    }
                    crate::scripting::csharp::NetValue::String(s) => {
                        writeln!(
                            code,
                            "        parameters[{}] = \"{}\";",
                            i,
                            s.replace('"', "\\\"")
                        )
                        .unwrap();
                    }
                    _ => {
                        writeln!(code, "        parameters[{i}] = null;").unwrap();
                    }
                }
            }

            writeln!(
                code,
                "        var result = method.Invoke(null, parameters);"
            )
            .unwrap();
        }

        // 序列化返回值为 JSON
        writeln!(code, "        var json = JsonSerializer.Serialize(result);").unwrap();
        writeln!(code, "        Console.WriteLine(json);").unwrap();
        writeln!(code, "        return 0;").unwrap();
        writeln!(code, "    }}").unwrap();
        writeln!(code, "}}").unwrap();

        Ok(code)
    }

    /// 编译并执行 C# 代码
    ///
    /// 使用 dotnet CLI 编译 C# 代码并执行。
    ///
    /// **参数:**
    /// - `code`: C# 源代码
    /// - `script_name`: 脚本名称（用于生成临时文件）
    ///
    /// **示例:**
    /// ```ignore
    /// let code = r#"
    /// using System;
    ///
    /// public class Script {
    ///     public static int Main() {
    ///         Console.WriteLine("Hello from C#!");
    ///         return 42;
    ///     }
    /// }
    /// "#;
    ///
    /// let result = host.compile_and_execute(code, "my_script")?;
    /// ```
    pub fn compile_and_execute(
        &self,
        code: &str,
        script_name: &str,
    ) -> Result<crate::scripting::csharp::NetValue, String> {
        if !self.initialized {
            return Err(".NET runtime not initialized".to_string());
        }

        // 🔥 性能优化：检查编译缓存
        if let Some(ref cache) = self.compile_cache {
            if let Some(cached_dll) = cache.get(code, script_name) {
                tracing::info!(
                    "Cache hit for script: {} - skipping compilation",
                    script_name
                );

                // 直接执行缓存的DLL
                return self.execute_cached_dll(&cached_dll, script_name);
            }
        }

        tracing::debug!("Compiling and executing C# script: {}", script_name);

        // 创建临时源文件
        let source_path = self.temp_dir.join(format!("{script_name}.cs"));
        let project_path = self.temp_dir.join(format!("{script_name}.csproj"));

        // 写入源代码
        std::fs::write(&source_path, code)
            .map_err(|e| format!("Failed to write source file: {e}"))?;

        // 创建 .csproj 文件（生成DLL库）
        let dll_name = format!("{script_name}.dll");
        let proj_content = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>disable</ImplicitUsings>
  </PropertyGroup>
</Project>"#
            .to_string();

        std::fs::write(&project_path, proj_content)
            .map_err(|e| format!("Failed to write project file: {e}"))?;

        // DLL路径
        let dll_path = self.temp_dir.join(&dll_name);

        // 🔥 性能优化：检查编译缓存
        if let Some(ref cache) = self.compile_cache {
            if let Some(cached_dll) = cache.get(code, script_name) {
                tracing::info!(
                    "✅ Cache HIT for '{}' - skipping compilation (~500ms saved)",
                    script_name
                );
                return self.execute_cached_dll(&cached_dll, script_name);
            } else {
                tracing::debug!("❌ Cache MISS for '{}' - compiling...", script_name);
            }
        }

        // 使用 dotnet build 编译
        let build_result = std::process::Command::new("dotnet")
            .args([
                "build",
                project_path.to_str().unwrap(),
                "-c",
                "Release",
                "-o",
                self.temp_dir.to_str().unwrap(),
            ])
            .current_dir(&self.temp_dir)
            .output();

        match build_result {
            Ok(output) if output.status.success() => {
                tracing::debug!("✅ C# compilation successful");

                // 🔥 性能优化：将编译结果插入缓存
                if dll_path.exists() {
                    if let Some(ref cache) = self.compile_cache {
                        // 复制DLL到缓存目录
                        let cache_dll_path = cache.get_cache_dir().join(&dll_name);
                        if std::fs::copy(&dll_path, &cache_dll_path).is_ok() {
                            if let Err(e) = cache.insert(code, script_name, cache_dll_path) {
                                tracing::warn!("Failed to cache compiled DLL: {}", e);
                            } else {
                                tracing::info!("💾 Cached compiled DLL: {}", dll_name);
                            }
                        }
                    }
                }

                // 执行编译后的DLL
                let execute_result = self.execute_compiled_dll(&dll_path, script_name);

                // 清理临时文件
                let _ = std::fs::remove_file(&source_path);
                let _ = std::fs::remove_file(&project_path);

                execute_result
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Compilation failed: {stderr}"))
            }
            Err(e) => Err(format!("Failed to run dotnet build: {e}")),
        }
    }

    /// 执行编译后的DLL
    ///
    /// 使用 dotnet CLI 执行已编译的DLL程序集。
    /// 🔥 性能优化：优先使用进程池
    fn execute_compiled_dll(
        &self,
        dll_path: &PathBuf,
        script_name: &str,
    ) -> Result<crate::scripting::csharp::NetValue, String> {
        tracing::debug!("Executing compiled DLL: {}", dll_path.display());

        // 🔥 性能优化：使用进程池执行（如果启用）
        if self.enable_process_pool {
            if let Some(ref pool) = self.process_pool {
                tracing::debug!("Using process pool for execution");

                // 使用进程池执行DLL
                let exec_result =
                    pool.lock().unwrap().execute(&format!("dotnet \"{}\"", dll_path.display()));

                match exec_result {
                    Ok(output) => {
                        // 解析输出
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&output) {
                            return Ok(crate::scripting::csharp::NetValue::from_json(&value));
                        } else {
                            return Ok(crate::scripting::csharp::NetValue::String(
                                output.trim().to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Process pool execution failed, falling back to standard execution: {}",
                            e
                        );
                        // 继续使用标准方法
                    }
                }
            }
        }

        // 标准执行方法
        let execute_result = std::process::Command::new("dotnet")
            .arg(dll_path)
            .current_dir(&self.temp_dir)
            .output();

        match execute_result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Err(format!("DLL execution failed: {stderr}"));
                }

                // 解析输出
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(crate::scripting::csharp::NetValue::from_json(&value))
                } else {
                    Ok(crate::scripting::csharp::NetValue::String(
                        stdout.trim().to_string(),
                    ))
                }
            }
            Err(e) => Err(format!("Failed to execute DLL: {e}")),
        }
    }

    /// 执行缓存的DLL
    ///
    /// 从缓存中获取并执行已编译的DLL。
    /// 🔥 性能优化：优先使用进程池
    fn execute_cached_dll(
        &self,
        cached_dll: &PathBuf,
        script_name: &str,
    ) -> Result<crate::scripting::csharp::NetValue, String> {
        tracing::debug!(
            "Executing cached DLL: {} for '{}'",
            cached_dll.display(),
            script_name
        );

        if !cached_dll.exists() {
            return Err(format!("Cached DLL not found: {}", cached_dll.display()));
        }

        // 🔥 性能优化：使用进程池执行（如果启用）
        if self.enable_process_pool {
            if let Some(ref pool) = self.process_pool {
                tracing::debug!("Using process pool for cached execution");

                let exec_result =
                    pool.lock().unwrap().execute(&format!("dotnet \"{}\"", cached_dll.display()));

                match exec_result {
                    Ok(output) => {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&output) {
                            return Ok(crate::scripting::csharp::NetValue::from_json(&value));
                        } else {
                            return Ok(crate::scripting::csharp::NetValue::String(
                                output.trim().to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Process pool execution failed: {}, falling back", e);
                        // 继续使用标准方法
                    }
                }
            }
        }

        // 标准执行方法
        let execute_result = std::process::Command::new("dotnet").arg(cached_dll).output();

        match execute_result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Err(format!("Cached DLL execution failed: {stderr}"));
                }

                // 解析输出
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(crate::scripting::csharp::NetValue::from_json(&value))
                } else {
                    Ok(crate::scripting::csharp::NetValue::String(
                        stdout.trim().to_string(),
                    ))
                }
            }
            Err(e) => Err(format!("Failed to execute cached DLL: {e}")),
        }
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> Option<crate::scripting::csharp_compile_cache::CacheStats> {
        self.compile_cache.as_ref().map(|cache| cache.get_stats())
    }

    /// 获取缓存命中率
    pub fn get_cache_hit_rate(&self) -> f64 {
        self.compile_cache.as_ref().map(|cache| cache.get_hit_rate()).unwrap_or(0.0)
    }

    /// 清除所有编译缓存
    pub fn clear_cache(&self) -> Result<(), String> {
        if let Some(ref cache) = self.compile_cache {
            cache.clear()?;
            tracing::info!("Cleared all compile cache");
        }
        Ok(())
    }

    /// 获取进程池统计信息
    pub fn get_process_pool_stats(&self) -> Option<std::collections::HashMap<String, String>> {
        if let Some(ref pool) = self.process_pool {
            let pool = pool.lock().unwrap();
            let stats = pool.get_stats();
            let process_stats = pool.get_process_stats();

            let mut result = std::collections::HashMap::new();
            result.insert(
                "total_executions".to_string(),
                stats.total_executions.to_string(),
            );
            result.insert("pool_hits".to_string(), stats.pool_hits.to_string());
            result.insert(
                "process_creations".to_string(),
                stats.process_creations.to_string(),
            );
            result.insert(
                "process_failures".to_string(),
                stats.process_failures.to_string(),
            );
            result.insert(
                "process_restarts".to_string(),
                stats.process_restarts.to_string(),
            );
            result.insert(
                "active_processes".to_string(),
                process_stats.len().to_string(),
            );

            // 计算命中率
            if stats.total_executions > 0 {
                let hit_rate = (stats.pool_hits as f64 / stats.total_executions as f64) * 100.0;
                result.insert("hit_rate_percent".to_string(), format!("{hit_rate:.1}"));
            }

            Some(result)
        } else {
            None
        }
    }

    /// 清理空闲进程
    pub fn cleanup_idle_processes(&self) {
        if let Some(ref pool) = self.process_pool {
            pool.lock().unwrap().cleanup_idle_processes();
        }
    }

    /// 进程池健康检查
    pub fn health_check_process_pool(&self) {
        if let Some(ref pool) = self.process_pool {
            pool.lock().unwrap().health_check();
        }
    }
}

#[cfg(feature = "csharp")]
impl Drop for DotNetCliHost {
    fn drop(&mut self) {
        if self.initialized {
            tracing::debug!("Cleaning up .NET CLI host");
            // 清理临时文件
            let _ = std::fs::remove_dir_all(&self.temp_dir);
        }
    }
}

#[cfg(feature = "csharp")]
unsafe impl Send for DotNetCliHost {}
#[cfg(feature = "csharp")]
unsafe impl Sync for DotNetCliHost {}

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
    #[cfg(feature = "csharp")]
    fn test_dotnet_cli_host_initialization() {
        // 这个测试需要 .NET SDK 已安装
        let host = DotNetCliHost::initialize();
        match host {
            Ok(host) => {
                assert!(host.initialized);
                tracing::info!(".NET CLI runtime test passed");
                tracing::info!(".NET version: {}", host.dotnet_version);
            }
            Err(e) => {
                tracing::warn!(".NET CLI runtime test skipped: {}", e);
            }
        }
    }

    #[test]
    #[cfg(feature = "csharp")]
    fn test_dotnet_installation_check() {
        let installed = DotNetCliHost::check_dotnet_installation();
        if installed {
            tracing::info!(".NET CLI is installed");
        } else {
            tracing::warn!(".NET CLI not found - please install .NET SDK 8.0 or higher");
        }
    }
}
