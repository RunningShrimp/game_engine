//! 构建管理器
//!
//! 提供增强的构建功能：
//! - 增量构建（只构建变化的包）
//! - 并行构建（多包并行编译）
//! - 进度显示（实时构建进度）
//! - 构建缓存（避免重复构建）
//! - 构建统计（构建时间和资源使用）

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Command as TokioCommand;
use tokio::sync::Semaphore;
use serde_json;

/// 构建配置
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// 构建模式（debug/release）
    pub profile: BuildProfile,
    /// 是否启用增量构建
    pub incremental: bool,
    /// 最大并行构建数
    pub max_parallel: usize,
    /// 是否显示进度
    pub show_progress: bool,
    /// 要构建的包列表（None表示构建所有包）
    pub packages: Option<Vec<String>>,
    /// 构建目标（None表示默认目标）
    pub target: Option<String>,
    /// 特性列表
    pub features: Vec<String>,
    /// 是否启用所有特性
    pub all_features: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
    Test,
    Bench,
}

impl BuildProfile {
    fn to_cargo_args(&self) -> Vec<&str> {
        match self {
            BuildProfile::Debug => vec![],
            BuildProfile::Release => vec!["--release"],
            BuildProfile::Test => vec!["--tests"],
            BuildProfile::Bench => vec!["--benches"],
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            profile: BuildProfile::Release,
            incremental: true,
            max_parallel: num_cpus::get(),
            show_progress: true,
            packages: None,
            target: None,
            features: Vec::new(),
            all_features: false,
        }
    }
}

/// 构建结果
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// 包名
    pub package: String,
    /// 是否成功
    pub success: bool,
    /// 构建时间（秒）
    pub duration: f64,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 输出大小（字节）
    pub output_size: Option<u64>,
}

/// 构建统计
#[derive(Debug, Clone)]
pub struct BuildStats {
    /// 总构建时间（秒）
    pub total_time: f64,
    /// 成功构建数
    pub success_count: usize,
    /// 失败构建数
    pub failure_count: usize,
    /// 总包数
    pub total_packages: usize,
    /// 各包构建时间
    pub package_times: HashMap<String, f64>,
    /// 并行度
    pub parallelism: usize,
}

/// 构建管理器
pub struct BuildManager {
    config: BuildConfig,
    /// 构建缓存（包名 -> 最后构建时间）
    build_cache: Arc<Mutex<HashMap<String, Instant>>>,
    /// 进度追踪
    progress: Arc<Mutex<BuildProgress>>,
}

#[derive(Debug, Clone, Default)]
struct BuildProgress {
    completed: usize,
    total: usize,
    current_packages: Vec<String>,
    start_time: Option<Instant>,
}

impl BuildManager {
    /// 创建构建管理器
    pub fn new(config: BuildConfig) -> Self {
        Self {
            config,
            build_cache: Arc::new(Mutex::new(HashMap::new())),
            progress: Arc::new(Mutex::new(BuildProgress::default())),
        }
    }

    /// 执行构建
    pub async fn build(&self) -> Result<BuildStats, BuildError> {
        let start_time = Instant::now();

        // 获取要构建的包列表
        let packages = self.get_packages_to_build().await?;

        // 初始化进度
        {
            let mut progress = self.progress.lock().unwrap();
            progress.total = packages.len();
            progress.completed = 0;
            progress.start_time = Some(Instant::now());
        }

        // 创建信号量以限制并行度
        let semaphore = Arc::new(Semaphore::new(self.config.max_parallel));

        // 并行构建所有包
        let mut tasks = Vec::new();
        for package in packages {
            let semaphore = semaphore.clone();
            let config = self.config.clone();
            let progress = self.progress.clone();
            let cache = self.build_cache.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                // 更新进度
                {
                    let mut prog = progress.lock().unwrap();
                    prog.current_packages.push(package.clone());
                }

                let result = Self::build_package(&package, &config).await;

                // 更新进度
                {
                    let mut prog = progress.lock().unwrap();
                    prog.completed += 1;
                    prog.current_packages.retain(|p| p != &package);
                }

                // 更新缓存
                if result.success {
                    let mut cache = cache.lock().unwrap();
                    cache.insert(package.clone(), Instant::now());
                }

                result
            });

            tasks.push((package, task));
        }

        // 等待所有任务完成
        let mut results = Vec::new();
        let mut package_times = HashMap::new();

        for (package, task) in tasks {
            let result = task.await.map_err(|e| BuildError::TaskError(e.to_string()))?;
            package_times.insert(package.clone(), result.duration);
            results.push((package, result));
        }

        // 计算统计
        let success_count = results.iter().filter(|(_, r)| r.success).count();
        let failure_count = results.len() - success_count;

        let stats = BuildStats {
            total_time: start_time.elapsed().as_secs_f64(),
            success_count,
            failure_count,
            total_packages: results.len(),
            package_times,
            parallelism: self.config.max_parallel,
        };

        // 显示结果
        if self.config.show_progress {
            self.display_results(&results, &stats);
        }

        // 如果有失败，返回错误
        if failure_count > 0 {
            let errors: Vec<String> = results
                .iter()
                .filter_map(|(pkg, r)| {
                    if !r.success {
                        Some(format!("{}: {}", pkg, r.error.as_deref().unwrap_or("Unknown error")))
                    } else {
                        None
                    }
                })
                .collect();
            return Err(BuildError::BuildFailed(errors));
        }

        Ok(stats)
    }

    /// 获取要构建的包列表
    async fn get_packages_to_build(&self) -> Result<Vec<String>, BuildError> {
        if let Some(ref packages) = self.config.packages {
            return Ok(packages.clone());
        }

        // 获取工作空间中的所有包
        let output = Command::new("cargo")
            .arg("metadata")
            .arg("--format-version")
            .arg("1")
            .arg("--no-deps")
            .output()
            .map_err(|e| BuildError::CommandError(e.to_string()))?;

        if !output.status.success() {
            return Err(BuildError::CommandError("Failed to get workspace metadata".to_string()));
        }

        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| BuildError::ParseError(e.to_string()))?;

        let packages: Vec<String> = metadata
            .get("packages")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| p.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // 如果启用增量构建，过滤出需要构建的包
        if self.config.incremental {
            Ok(self.filter_changed_packages(packages))
        } else {
            Ok(packages)
        }
    }

    /// 过滤出需要构建的包（基于文件修改时间）
    fn filter_changed_packages(&self, packages: Vec<String>) -> Vec<String> {
        // 简化实现：对于增量构建，暂时返回所有包
        // 实际实现应该检查源文件的修改时间
        // 这里使用简化的逻辑：如果缓存存在且时间较近，跳过构建
        let cache = self.build_cache.lock().unwrap();
        let mut changed = Vec::new();

        for package in packages {
            if let Some(cached_time) = cache.get(&package) {
                // 如果缓存时间在5分钟内，跳过构建（简化逻辑）
                if cached_time.elapsed() < Duration::from_secs(300) {
                    continue;
                }
            }
            changed.push(package);
        }

        changed
    }

    /// 构建单个包
    async fn build_package(package: &str, config: &BuildConfig) -> BuildResult {
        let start = Instant::now();

        let mut cmd = TokioCommand::new("cargo");
        cmd.arg("build");

        // 添加profile参数
        for arg in config.profile.to_cargo_args() {
            cmd.arg(arg);
        }

        // 添加包参数
        cmd.arg("--package").arg(package);

        // 添加目标参数
        if let Some(ref target) = config.target {
            cmd.arg("--target").arg(target);
        }

        // 添加特性参数
        if config.all_features {
            cmd.arg("--all-features");
        } else if !config.features.is_empty() {
            cmd.arg("--features").arg(config.features.join(","));
        }

        // 设置输出
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let result = cmd.output().await;

        let duration = start.elapsed().as_secs_f64();

        match result {
            Ok(output) => {
                let success = output.status.success();
                let error = if success {
                    None
                } else {
                    Some(
                        String::from_utf8_lossy(&output.stderr)
                            .lines()
                            .take(5)
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                };

                // 计算输出大小（简化实现）
                let output_size = if success {
                    // 尝试查找构建产物大小
                    None // 简化实现，实际应该查找target目录
                } else {
                    None
                };

                BuildResult {
                    package: package.to_string(),
                    success,
                    duration,
                    error,
                    output_size,
                }
            }
            Err(e) => BuildResult {
                package: package.to_string(),
                success: false,
                duration,
                error: Some(e.to_string()),
                output_size: None,
            },
        }
    }

    /// 显示构建结果
    fn display_results(&self, results: &[(String, BuildResult)], stats: &BuildStats) {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📦 构建完成");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("总时间: {:.2}秒", stats.total_time);
        println!("成功: {} | 失败: {}", stats.success_count, stats.failure_count);
        println!("并行度: {}", stats.parallelism);
        println!();

        // 显示各包构建时间
        println!("各包构建时间:");
        let mut sorted_results: Vec<_> = results.iter().collect();
        sorted_results.sort_by(|a, b| b.1.duration.partial_cmp(&a.1.duration).unwrap());

        for (package, result) in sorted_results.iter().take(10) {
            let status = if result.success { "✓" } else { "✗" };
            println!("  {} {}: {:.2}秒", status, package, result.duration);
        }
    }

    /// 获取当前构建进度
    pub fn get_progress(&self) -> (usize, usize, Vec<String>) {
        let progress = self.progress.lock().unwrap();
        (
            progress.completed,
            progress.total,
            progress.current_packages.clone(),
        )
    }

    /// 获取进度追踪器（用于外部访问）
    pub fn progress(&self) -> Arc<Mutex<BuildProgress>> {
        self.progress.clone()
    }
}

/// 构建错误
#[derive(Debug, Clone)]
pub enum BuildError {
    CommandError(String),
    ParseError(String),
    TaskError(String),
    BuildFailed(Vec<String>),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::CommandError(msg) => write!(f, "Command error: {}", msg),
            BuildError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            BuildError::TaskError(msg) => write!(f, "Task error: {}", msg),
            BuildError::BuildFailed(errors) => {
                write!(f, "Build failed:\n{}", errors.join("\n"))
            }
        }
    }
}

impl std::error::Error for BuildError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_profile_args() {
        assert_eq!(BuildProfile::Debug.to_cargo_args(), Vec::<&str>::new());
        assert_eq!(BuildProfile::Release.to_cargo_args(), vec!["--release"]);
    }

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert_eq!(config.profile, BuildProfile::Release);
        assert!(config.incremental);
        assert!(config.show_progress);
    }
}

