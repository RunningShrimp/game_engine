//! # 插件版本管理与沙箱
//!
//! 提供插件版本兼容性检查和WASI沙箱隔离。
//!
//! ## 功能特性
//!
//! - **语义化版本**: Semantic Versioning 2.0.0
//! - **版本约束**: 支持 ^, ~, >=, <, = 等约束
//! - **兼容性检查**: 自动检查插件版本兼容性
//! - **WASI沙箱**: 使用WASI隔离插件执行
//!
//! ## 使用场景
//!
//! - **插件版本管理**: 确保插件版本兼容
//! - **安全隔离**: 沙箱化执行不受信任的插件

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// =============================================================================
// 语义化版本 (Semantic Versioning)
// =============================================================================

/// 语义化版本号 (SemVer 2.0.0)
///
/// 格式: MAJOR.MINOR.PATCH
/// - MAJOR: 不兼容的API变更
/// - MINOR: 向后兼容的功能新增
/// - PATCH: 向后兼容的问题修复
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    /// 创建新版本
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// 从字符串解析
    ///
    /// # 示例
    ///
    /// ```
    /// # use game_engine::plugins::versioning::SemVer;
    /// let version = SemVer::parse("1.2.3").unwrap();
    /// assert_eq!(version.major, 1);
    /// assert_eq!(version.minor, 2);
    /// assert_eq!(version.patch, 3);
    /// ```
    pub fn parse(s: &str) -> Result<Self, SemVerError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(SemVerError::InvalidFormat(s.to_string()));
        }

        let major = parts[0].parse().map_err(|_| SemVerError::InvalidComponent {
            version: s.to_string(),
            component: "major".to_string(),
            value: parts[0].to_string(),
        })?;

        let minor = parts[1].parse().map_err(|_| SemVerError::InvalidComponent {
            version: s.to_string(),
            component: "minor".to_string(),
            value: parts[1].to_string(),
        })?;

        let patch = parts[2].parse().map_err(|_| SemVerError::InvalidComponent {
            version: s.to_string(),
            component: "patch".to_string(),
            value: parts[2].to_string(),
        })?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    /// 是否为预发布版本（0.x.x）
    pub fn is_pre_release(&self) -> bool {
        self.major == 0
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// SemVer 错误
#[derive(Debug, Error, Clone)]
pub enum SemVerError {
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    #[error("Invalid {component} in {version}: {value}")]
    InvalidComponent {
        version: String,
        component: String,
        value: String,
    },
}

// =============================================================================
// 版本约束 (Version Requirements)
// =============================================================================

/// 版本约束
///
/// 支持语义化版本约束语法。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionRequirement {
    /// 精确版本 (=1.2.3)
    Exact(SemVer),
    /// 大于等于 (>=1.2.3)
    GreaterOrEqual(SemVer),
    /// 小于 (<1.2.3)
    LessThan(SemVer),
    /// 范围 (>=1.2.3 <2.0.0)
    Range { min: SemVer, max: SemVer },
    /// 兼容版本 (^1.2.3)
    Compatible(SemVer),
    /// 近似版本 (~1.2.3)
    Approximate(SemVer),
    /// 任意版本 (*)
    Any,
    /// 或 (OR)
    Or(Box<VersionRequirement>, Box<VersionRequirement>),
    /// 且 (AND)
    And(Box<VersionRequirement>, Box<VersionRequirement>),
}

impl VersionRequirement {
    /// 从字符串解析版本约束
    ///
    /// # 示例
    ///
    /// ```
    /// # use game_engine::plugins::versioning::VersionRequirement;
    /// let req = VersionRequirement::parse("^1.2.3").unwrap();
    /// ```
    pub fn parse(s: &str) -> Result<Self, SemVerError> {
        let s = s.trim();

        match s {
            "*" | "" => Ok(VersionRequirement::Any),
            _ if s.starts_with('^') => {
                let version = SemVer::parse(&s[1..])?;
                Ok(VersionRequirement::Compatible(version))
            }
            _ if s.starts_with('~') => {
                let version = SemVer::parse(&s[1..])?;
                Ok(VersionRequirement::Approximate(version))
            }
            _ if s.starts_with(">=") => {
                let version = SemVer::parse(&s[2..])?;
                Ok(VersionRequirement::GreaterOrEqual(version))
            }
            _ if s.starts_with('<') => {
                let version = SemVer::parse(&s[1..])?;
                Ok(VersionRequirement::LessThan(version))
            }
            _ if s.starts_with('=') => {
                let version = SemVer::parse(&s[1..])?;
                Ok(VersionRequirement::Exact(version))
            }
            _ if s.contains("||") => {
                // OR 约束
                let parts: Vec<&str> = s.split("||").collect();
                if parts.len() != 2 {
                    return Err(SemVerError::InvalidFormat(s.to_string()));
                }
                let left = Self::parse(parts[0])?;
                let right = Self::parse(parts[1])?;
                Ok(VersionRequirement::Or(Box::new(left), Box::new(right)))
            }
            _ if s.contains(',') => {
                // AND 约束 (逗号分隔)
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() != 2 {
                    return Err(SemVerError::InvalidFormat(s.to_string()));
                }
                let left = Self::parse(parts[0])?;
                let right = Self::parse(parts[1])?;
                Ok(VersionRequirement::And(Box::new(left), Box::new(right)))
            }
            _ => {
                // 纯版本号，视为精确版本
                let version = SemVer::parse(s)?;
                Ok(VersionRequirement::Exact(version))
            }
        }
    }

    /// 检查版本是否满足约束
    pub fn satisfies(&self, version: &SemVer) -> bool {
        match self {
            VersionRequirement::Exact(v) => version == v,
            VersionRequirement::GreaterOrEqual(v) => version >= v,
            VersionRequirement::LessThan(v) => version < v,
            VersionRequirement::Range { min, max } => version >= min && version < max,
            VersionRequirement::Compatible(v) => {
                // ^1.2.3 => >=1.2.3 <2.0.0
                if v.major == 0 {
                    // 0.x.y 为预发布，^0.2.3 => >=0.2.3 <0.3.0
                    version >= v && (version.major == 0 && version.minor == v.minor)
                } else {
                    version >= v && version.major == v.major
                }
            }
            VersionRequirement::Approximate(v) => {
                // ~1.2.3 => >=1.2.3 <1.3.0
                if v.major == 0 {
                    // ~0.2.3 => >=0.2.3 <0.3.0
                    version >= v && (version.major == 0 && version.minor == v.minor)
                } else {
                    version >= v && version.major == v.major && version.minor == v.minor
                }
            }
            VersionRequirement::Any => true,
            VersionRequirement::Or(left, right) => {
                left.satisfies(version) || right.satisfies(version)
            }
            VersionRequirement::And(left, right) => {
                left.satisfies(version) && right.satisfies(version)
            }
        }
    }
}

impl fmt::Display for VersionRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionRequirement::Exact(v) => write!(f, "={v}"),
            VersionRequirement::GreaterOrEqual(v) => write!(f, ">={v}"),
            VersionRequirement::LessThan(v) => write!(f, "<{v}"),
            VersionRequirement::Range { min, max } => write!(f, ">={min},<{max}"),
            VersionRequirement::Compatible(v) => write!(f, "^{v}"),
            VersionRequirement::Approximate(v) => write!(f, "~{v}"),
            VersionRequirement::Any => write!(f, "*"),
            VersionRequirement::Or(left, right) => write!(f, "{left} || {right}"),
            VersionRequirement::And(left, right) => write!(f, "{left}, {right}"),
        }
    }
}

// =============================================================================
// 版本管理器
// =============================================================================

/// 插件版本管理器
///
/// 管理插件版本依赖和兼容性。
pub struct PluginVersionManager {
    /// 已注册的插件版本
    versions: std::collections::HashMap<String, SemVer>,
}

impl PluginVersionManager {
    /// 创建新的版本管理器
    pub fn new() -> Self {
        Self {
            versions: std::collections::HashMap::new(),
        }
    }

    /// 注册插件版本
    pub fn register(&mut self, name: impl Into<String>, version: SemVer) {
        self.versions.insert(name.into(), version);
    }

    /// 检查版本兼容性
    ///
    /// # 参数
    ///
    /// - `name`: 插件名称
    /// - `requirement`: 版本约束
    ///
    /// # 返回
    ///
    /// 如果版本兼容返回 Ok(())，否则返回 Err
    pub fn check_compatibility(
        &self,
        name: &str,
        requirement: &VersionRequirement,
    ) -> Result<(), VersionConflictError> {
        if let Some(&version) = self.versions.get(name) {
            if requirement.satisfies(&version) {
                Ok(())
            } else {
                Err(VersionConflictError::IncompatibleVersion {
                    plugin: name.to_string(),
                    required: requirement.to_string(),
                    found: version.to_string(),
                })
            }
        } else {
            Err(VersionConflictError::PluginNotFound(name.to_string()))
        }
    }

    /// 获取插件版本
    pub fn get_version(&self, name: &str) -> Option<SemVer> {
        self.versions.get(name).copied()
    }

    /// 解析依赖并检查兼容性
    pub fn check_dependencies(
        &self,
        dependencies: &[(String, VersionRequirement)],
    ) -> Result<(), VersionConflictError> {
        for (name, requirement) in dependencies {
            self.check_compatibility(name, requirement)?;
        }
        Ok(())
    }
}

impl Default for PluginVersionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 版本冲突错误
#[derive(Debug, Error, Clone)]
pub enum VersionConflictError {
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Incompatible version for '{plugin}': required {required}, found {found}")]
    IncompatibleVersion {
        plugin: String,
        required: String,
        found: String,
    },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
}

// =============================================================================
// WASI 沙箱
// =============================================================================

/// WASI 沙箱配置
#[derive(Debug, Clone)]
pub struct WasiSandboxConfig {
    /// 允许的目录
    pub allowed_dirs: Vec<String>,
    /// 允许的环境变量
    pub allowed_env: Vec<String>,
    /// 是否允许网络访问
    pub allow_network: bool,
    /// 最大内存 (MB)
    pub max_memory_mb: Option<usize>,
    /// 最大执行时间 (秒)
    pub max_execution_time_secs: Option<u64>,
}

impl Default for WasiSandboxConfig {
    fn default() -> Self {
        Self {
            allowed_dirs: vec![".".to_string()], // 只允许当前目录
            allowed_env: vec![],
            allow_network: false,
            max_memory_mb: Some(512),
            max_execution_time_secs: Some(30),
        }
    }
}

/// WASI 沙箱
///
/// 提供安全的插件执行环境。
pub struct WasiSandbox {
    config: WasiSandboxConfig,
}

impl WasiSandbox {
    /// 创建新的 WASI 沙箱
    pub fn new(config: WasiSandboxConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建沙箱
    pub fn with_default_config() -> Self {
        Self::new(WasiSandboxConfig::default())
    }

    /// 执行 WASM 插件
    ///
    /// # 参数
    ///
    /// - `wasm_bytes`: WASM 模块字节码
    /// - `function`: 要调用的函数名
    /// - `args`: 函数参数
    ///
    /// # 返回
    ///
    /// 返回函数执行结果
    #[cfg(feature = "wasm")]
    pub async fn execute_wasm(
        &self,
        _wasm_bytes: &[u8],
        _function: &str,
        _args: &[Vec<u8>],
    ) -> Result<Vec<u8>, SandboxError> {
        // 注：实际的 WASI 实现需要 wasmtime 或 wasmer 库
        // 这里提供框架代码

        #[cfg(feature = "wasmtime")]
        {
            use wasmtime::*;

            // 配置引擎
            let mut config = Config::new();
            config.wasm_simd(true);
            config.async_support(true);

            // 设置内存限制
            if let Some(max_memory) = self.config.max_memory_mb {
                config.max_wasm_memory(max_memory_mb * 1024 * 1024);
            }

            let engine =
                Engine::new(&config).map_err(|e| SandboxError::Initialization(e.to_string()))?;

            // 配置 WASI
            let mut linker = Linker::new(&engine);
            wasi_common::add_to_linker(&mut linker, &self.create_wasi_ctx())?;

            // 编译模块
            let module = Module::from_binary(&engine, _wasm_bytes)
                .map_err(|e| SandboxError::Compilation(e.to_string()))?;

            // 创建实例
            let mut store = Store::new(&engine, WasiState::new());
            let instance = linker
                .instantiate(&mut store, &module)
                .map_err(|e| SandboxError::Instantiation(e.to_string()))?;

            // 调用函数
            let func = instance
                .get_typed_func::<(i32, i32), i32>(&store, _function)
                .map_err(|e| SandboxError::FunctionNotFound(e.to_string()))?;

            // 执行（这里简化，实际需要处理参数）
            func.call(&mut store, 0, 0)
                .map_err(|e| SandboxError::Execution(e.to_string()))?;

            Ok(Vec::new())
        }

        #[cfg(not(feature = "wasmtime"))]
        {
            Err(SandboxError::NotSupported(
                "WASM execution requires 'wasmtime' feature".to_string(),
            ))
        }
    }

    /// 执行 WASM 插件（非异步版本）
    #[cfg(feature = "wasm")]
    pub fn execute_wasm_sync(
        &self,
        wasm_bytes: &[u8],
        function: &str,
        args: &[Vec<u8>],
    ) -> Result<Vec<u8>, SandboxError> {
        // 简化版本，委托到异步实现
        use std::sync::Mutex;

        static RT: once_cell::sync::Lazy<Mutex<tokio::runtime::Runtime>> =
            once_cell::sync::Lazy::new(|| Mutex::new(tokio::runtime::Runtime::new().unwrap()));

        let rt = RT.lock().unwrap();
        rt.block_on(self.execute_wasm(wasm_bytes, function, args))
    }

    /// 创建 WASI 上下文
    #[cfg(feature = "wasm")]
    fn create_wasi_ctx(&self) -> wasi_common::WasiCtx {
        use wasi_common::WasiCtx;

        let mut builder = WasiCtx::new_builder();

        // 添加允许的目录
        for dir in &self.config.allowed_dirs {
            builder.preopen_dir(dir, dir).ok();
        }

        // 添加允许的环境变量
        for env in &self.config.allowed_env {
            if let Some((key, value)) = env.split_once('=') {
                builder.env(key, value);
            }
        }

        builder.build()
    }

    /// 验证 WASM 模块
    pub fn validate_wasm(&self, wasm_bytes: &[u8]) -> Result<ValidationResult, SandboxError> {
        // 基本 WASM 魔数检查
        if wasm_bytes.len() < 8 {
            return Err(SandboxError::InvalidWasm("File too small".to_string()));
        }

        if &wasm_bytes[0..4] != b"\0asm" {
            return Err(SandboxError::InvalidWasm(
                "Invalid WASM magic number".to_string(),
            ));
        }

        // 检查版本
        let version = &wasm_bytes[4..8];
        if version != b"\x01\x00\x00\x00" {
            return Err(SandboxError::InvalidWasm(format!(
                "Unsupported WASM version: {version:?}"
            )));
        }

        Ok(ValidationResult {
            is_valid: true,
            size_bytes: wasm_bytes.len(),
        })
    }
}

/// WASM 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub size_bytes: usize,
}

/// 沙箱错误
#[derive(Debug, Error, Clone)]
pub enum SandboxError {
    #[error("Sandbox initialization failed: {0}")]
    Initialization(String),

    #[error("WASM compilation failed: {0}")]
    Compilation(String),

    #[error("WASM instantiation failed: {0}")]
    Instantiation(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Execution failed: {0}")]
    Execution(String),

    #[error("Invalid WASM module: {0}")]
    InvalidWasm(String),

    #[error("Feature not supported: {0}")]
    NotSupported(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),
}

// =============================================================================
// 插件沙箱管理器
// =============================================================================

/// 插件沙箱管理器
///
/// 管理多个插件的沙箱环境。
pub struct PluginSandboxManager {
    sandboxes: std::collections::HashMap<String, WasiSandbox>,
}

impl PluginSandboxManager {
    /// 创建新的沙箱管理器
    pub fn new() -> Self {
        Self {
            sandboxes: std::collections::HashMap::new(),
        }
    }

    /// 为插件创建沙箱
    pub fn create_sandbox(&mut self, plugin_name: impl Into<String>, config: WasiSandboxConfig) {
        let sandbox = WasiSandbox::new(config);
        self.sandboxes.insert(plugin_name.into(), sandbox);
    }

    /// 获取插件沙箱
    pub fn get_sandbox(&self, plugin_name: &str) -> Option<&WasiSandbox> {
        self.sandboxes.get(plugin_name)
    }

    /// 移除插件沙箱
    pub fn remove_sandbox(&mut self, plugin_name: &str) -> bool {
        self.sandboxes.remove(plugin_name).is_some()
    }
}

impl Default for PluginSandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let version = SemVer::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_semver_comparison() {
        let v1 = SemVer::new(1, 2, 3);
        let v2 = SemVer::new(1, 2, 4);
        assert!(v1 < v2);
    }

    #[test]
    fn test_version_requirement_exact() {
        let req = VersionRequirement::parse("=1.2.3").unwrap();
        let v1 = SemVer::new(1, 2, 3);
        let v2 = SemVer::new(1, 2, 4);
        assert!(req.satisfies(&v1));
        assert!(!req.satisfies(&v2));
    }

    #[test]
    fn test_version_requirement_compatible() {
        let req = VersionRequirement::parse("^1.2.3").unwrap();
        assert!(req.satisfies(&SemVer::new(1, 2, 3)));
        assert!(req.satisfies(&SemVer::new(1, 5, 0)));
        assert!(!req.satisfies(&SemVer::new(2, 0, 0)));
    }

    #[test]
    fn test_version_requirement_approximate() {
        let req = VersionRequirement::parse("~1.2.3").unwrap();
        assert!(req.satisfies(&SemVer::new(1, 2, 3)));
        assert!(req.satisfies(&SemVer::new(1, 2, 5)));
        assert!(!req.satisfies(&SemVer::new(1, 3, 0)));
    }

    #[test]
    fn test_version_manager() {
        let mut manager = PluginVersionManager::new();
        manager.register("test-plugin", SemVer::new(1, 2, 3));

        let req = VersionRequirement::parse("^1.0.0").unwrap();
        assert!(manager.check_compatibility("test-plugin", &req).is_ok());

        let req = VersionRequirement::parse("^2.0.0").unwrap();
        assert!(manager.check_compatibility("test-plugin", &req).is_err());
    }

    #[test]
    fn test_wasm_validation() {
        let sandbox = WasiSandbox::with_default_config();

        // 无效的 WASM
        let invalid = b"not wasm";
        assert!(sandbox.validate_wasm(invalid).is_err());

        // 有效魔数但不完整的 WASM
        let mut valid_magic = [0u8; 8];
        valid_magic[0..4].copy_from_slice(b"\0asm");
        valid_magic[4..8].copy_from_slice(b"\x01\x00\x00\x00");

        let result = sandbox.validate_wasm(&valid_magic).unwrap();
        assert!(result.is_valid);
    }
}
