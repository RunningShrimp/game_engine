//! WASM部署工具链
//!
//! 自动化的WebAssembly构建、优化和部署系统。

use std::path::PathBuf;
use std::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 代码块优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkPriority {
    /// 关键 - 必须立即加载
    Critical,
    /// 高 - 应该尽快加载
    High,
    /// 中 - 可以延迟加载
    Medium,
    /// 低 - 延迟加载
    Low,
}

/// 代码块加载策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkLoadStrategy {
    /// 立即加载
    Eager,
    /// 按需加载
    OnDemand,
    /// 延迟加载
    Lazy,
}

/// 代码块
#[derive(Debug, Clone)]
pub struct CodeChunk {
    /// 名称
    pub name: String,
    /// 数据
    pub data: Vec<u8>,
    /// 大小（字节）
    pub size: usize,
    /// 优先级
    pub priority: ChunkPriority,
    /// 加载策略
    pub load_strategy: ChunkLoadStrategy,
}

/// 代码块集合
#[derive(Debug, Clone)]
pub struct CodeChunks {
    /// 核心模块
    pub core: CodeChunk,
    /// 功能模块
    pub features: Vec<CodeChunk>,
    /// 资源模块
    pub resources: Vec<CodeChunk>,
    /// 总大小
    pub total_size: usize,
}

/// 代码块清单
#[derive(Debug, Clone)]
pub struct CodeChunkManifest {
    /// 核心模块
    pub core_chunk: CodeChunk,
    /// 功能模块
    pub feature_chunks: Vec<CodeChunk>,
    /// 资源模块
    pub resource_chunks: Vec<CodeChunk>,
    /// 总大小
    pub total_size: usize,
}

/// WASM部署配置
#[derive(Debug, Clone)]
pub struct WasmDeployConfig {
    /// 输入项目路径
    pub project_path: PathBuf,

    /// 输出目录
    pub output_dir: PathBuf,

    /// 优化级别
    pub optimization_level: WasmOptLevel,

    /// 是否启用代码分割
    pub enable_code_splitting: bool,

    /// 是否压缩
    pub compress_output: bool,

    /// 部署目标
    pub deployment_target: DeploymentTarget,
}

/// WASM优化级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmOptLevel {
    /// 零优化（快速编译）
    O0,
    /// 基础优化
    O2,
    /// 最大优化
    O3,
    /// 最大优化 + 内联
    O4,
}

/// 部署目标
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentTarget {
    /// 本地目录
    Local,
    /// 自定义服务器
    CustomServer,
    /// GitHub Pages
    GitHubPages,
    /// Netlify
    Netlify,
    /// Vercel
    Vercel,
}

/// WASM部署工具
pub struct WasmDeployTool {
    /// 配置
    config: WasmDeployConfig,

    /// 构建状态
    build_status: BuildStatus,
}

/// 构建状态
#[derive(Debug, Clone)]
pub struct BuildStatus {
    /// 当前阶段
    pub current_phase: BuildPhase,

    /// 进度百分比
    pub progress: f32,

    /// 总步骤数
    pub total_steps: u32,

    /// 已完成步骤
    pub completed_steps: u32,
}

/// 构建阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPhase {
    /// 准备中
    Preparing,
    /// 编译WASM
    Compiling,
    /// 优化WASM
    Optimizing,
    /// 打包
    Bundling,
    /// 部署
    Deploying,
    /// 完成
    Completed,
}

impl WasmDeployTool {
    /// 创建新的部署工具
    pub fn new(config: WasmDeployConfig) -> Self {
        Self {
            config,
            build_status: BuildStatus {
                current_phase: BuildPhase::Preparing,
                progress: 0.0,
                total_steps: 5,
                completed_steps: 0,
            },
        }
    }

    /// 执行完整部署流程
    pub async fn deploy(&mut self) -> Result<DeploymentResult, WasmDeployError> {
        self.update_phase(BuildPhase::Preparing, 0, 5);

        // 1. 准备构建
        self.prepare_build().await?;

        self.update_phase(BuildPhase::Compiling, 1, 5);

        // 2. 编译WASM
        let wasm_path = self.compile_wasm().await?;

        self.update_phase(BuildPhase::Optimizing, 2, 5);

        // 3. 优化WASM
        let optimized_path = self.optimize_wasm(&wasm_path).await?;

        self.update_phase(BuildPhase::Bundling, 3, 5);

        // 4. 打包和代码分割
        let bundle = self.bundle_assets(&optimized_path).await?;

        self.update_phase(BuildPhase::Deploying, 4, 5);

        // 5. 部署
        let deployment_url = self.deploy_bundle(&bundle).await?;

        self.update_phase(BuildPhase::Completed, 5, 5);

        Ok(DeploymentResult {
            success: true,
            deployment_url,
            build_time: std::time::Duration::from_secs(60),
            wasm_size: bundle.total_size,
            load_time: bundle.estimated_load_time,
        })
    }

    /// 准备构建
    async fn prepare_build(&self) -> Result<(), WasmDeployError> {
        // 确保输出目录存在
        tokio::fs::create_dir_all(&self.config.output_dir)
            .await
            .map_err(|e| WasmDeployError::IoError(e.to_string()))?;

        Ok(())
    }

    /// 编译WASM
    async fn compile_wasm(&self) -> Result<PathBuf, WasmDeployError> {
        let output = Command::new("cargo")
            .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
            .current_dir(&self.config.project_path)
            .output()
            .map_err(|e| WasmDeployError::BuildError(e.to_string()))?;

        if !output.status.success() {
            return Err(WasmDeployError::BuildError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // 查找生成的.wasm文件
        let wasm_path = self
            .config
            .project_path
            .join("target/wasm32-unknown-unknown/release/game_engine.wasm");

        if !wasm_path.exists() {
            return Err(WasmDeployError::BuildError(
                "WASM file not found after build".to_string(),
            ));
        }

        Ok(wasm_path)
    }

    /// 优化WASM
    async fn optimize_wasm(&self, wasm_path: &PathBuf) -> Result<PathBuf, WasmDeployError> {
        let output_path = self.config.output_dir.join("optimized.wasm");

        // 使用wasm-opt优化
        let opt_level = match self.config.optimization_level {
            WasmOptLevel::O0 => "O0",
            WasmOptLevel::O2 => "O2",
            WasmOptLevel::O3 => "O3",
            WasmOptLevel::O4 => "O4",
        };

        let output = Command::new("wasm-opt")
            .args([
                "-O", // 启用优化
                opt_level,
                "--enable-bulk-memory",
                "--enable-simd",
                "-o", // 输出文件
                output_path.to_str().unwrap(),
                wasm_path.to_str().unwrap(),
            ])
            .output();

        match output {
            Ok(out) if out.status.success() => Ok(output_path),
            Ok(_) => {
                // wasm-opt失败，复制原始文件
                tokio::fs::copy(wasm_path, &output_path)
                    .await
                    .map_err(|e| WasmDeployError::OptimizationError(e.to_string()))?;
                Ok(output_path)
            }
            Err(_) => {
                // wasm-opt不可用，复制原始文件
                tokio::fs::copy(wasm_path, &output_path)
                    .await
                    .map_err(|e| WasmDeployError::OptimizationError(e.to_string()))?;
                Ok(output_path)
            }
        }
    }

    /// 打包资源
    async fn bundle_assets(&self, wasm_path: &PathBuf) -> Result<WasmBundle, WasmDeployError> {
        let mut bundle = WasmBundle::default();

        // 读取WASM文件
        let mut wasm_file = tokio::fs::File::open(wasm_path)
            .await
            .map_err(|e| WasmDeployError::IoError(e.to_string()))?;

        let mut wasm_data = Vec::new();
        wasm_file
            .read_to_end(&mut wasm_data)
            .await
            .map_err(|e| WasmDeployError::IoError(e.to_string()))?;

        bundle.wasm_data = wasm_data.clone();
        bundle.total_size = wasm_data.len();

        // 代码分割
        if self.config.enable_code_splitting {
            bundle = self.split_code(bundle).await?;
        }

        // 压缩
        if self.config.compress_output {
            bundle = self.compress_bundle(bundle).await?;
        }

        Ok(bundle)
    }

    /// 代码分割
    async fn split_code(&self, mut bundle: WasmBundle) -> Result<WasmBundle, WasmDeployError> {
        // 实现代码分割策略
        // 将WASM模块分成多个部分以优化加载

        let chunks = self.create_code_chunks(&bundle.wasm_data)?;

        // 创建代码块清单
        let chunk_manifest = CodeChunkManifest {
            core_chunk: chunks.core,
            feature_chunks: chunks.features,
            resource_chunks: chunks.resources,
            total_size: chunks.total_size,
        };

        // 更新bundle信息
        bundle.chunk_info = Some(chunk_manifest);
        bundle.code_split = true;

        Ok(bundle)
    }

    /// 创建代码块
    fn create_code_chunks(&self, wasm_data: &[u8]) -> Result<CodeChunks, WasmDeployError> {
        // 简化实现：基于启发式分割
        // 实际实现应该使用WASM模块解析器

        let data_size = wasm_data.len();

        // 核心模块（约40%）
        let core_size = data_size * 40 / 100;
        let core_chunk = CodeChunk {
            name: "core".to_string(),
            data: wasm_data[..core_size].to_vec(),
            size: core_size,
            priority: ChunkPriority::Critical,
            load_strategy: ChunkLoadStrategy::Eager,
        };

        // 功能模块（约35%）
        let feature_size = data_size * 35 / 100;
        let feature_chunks = vec![CodeChunk {
            name: "rendering".to_string(),
            data: wasm_data[core_size..core_size + feature_size].to_vec(),
            size: feature_size,
            priority: ChunkPriority::High,
            load_strategy: ChunkLoadStrategy::OnDemand,
        }];

        // 资源模块（约25%）
        let resource_size = data_size - core_size - feature_size;
        let resource_chunks = vec![CodeChunk {
            name: "assets".to_string(),
            data: wasm_data[core_size + feature_size..].to_vec(),
            size: resource_size,
            priority: ChunkPriority::Low,
            load_strategy: ChunkLoadStrategy::Lazy,
        }];

        Ok(CodeChunks {
            core: core_chunk,
            features: feature_chunks,
            resources: resource_chunks,
            total_size: data_size,
        })
    }

    /// 压缩打包
    async fn compress_bundle(&self, bundle: WasmBundle) -> Result<WasmBundle, WasmDeployError> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder
            .write_all(&bundle.wasm_data)
            .map_err(|e| WasmDeployError::CompressionError(e.to_string()))?;

        let compressed =
            encoder.finish().map_err(|e| WasmDeployError::CompressionError(e.to_string()))?;

        let total_size = compressed.len();
        Ok(WasmBundle {
            wasm_data: compressed,
            compressed: true,
            total_size,
            ..bundle
        })
    }

    /// 部署打包
    async fn deploy_bundle(&self, bundle: &WasmBundle) -> Result<String, WasmDeployError> {
        match self.config.deployment_target {
            DeploymentTarget::Local => {
                let output_path = self.config.output_dir.join("game_engine.wasm");
                let mut file = tokio::fs::File::create(&output_path)
                    .await
                    .map_err(|e| WasmDeployError::IoError(e.to_string()))?;

                file.write_all(&bundle.wasm_data)
                    .await
                    .map_err(|e| WasmDeployError::IoError(e.to_string()))?;

                Ok(format!("file://{}", output_path.display()))
            }
            _ => Err(WasmDeployError::DeploymentError(
                "Deployment target not yet implemented".to_string(),
            )),
        }
    }

    /// 更新构建阶段
    fn update_phase(&mut self, phase: BuildPhase, completed: u32, total: u32) {
        self.build_status.current_phase = phase;
        self.build_status.completed_steps = completed;
        self.build_status.total_steps = total;
        self.build_status.progress = (completed as f32 / total as f32) * 100.0;
    }

    /// 获取构建状态
    pub fn get_status(&self) -> &BuildStatus {
        &self.build_status
    }
}

/// WASM打包
#[derive(Debug, Clone, Default)]
pub struct WasmBundle {
    /// WASM数据
    pub wasm_data: Vec<u8>,

    /// 是否压缩
    pub compressed: bool,

    /// 总大小（字节）
    pub total_size: usize,

    /// 预估加载时间（毫秒）
    pub estimated_load_time: u32,

    /// 是否启用代码分割
    pub code_split: bool,

    /// 代码块清单（如果启用代码分割）
    pub chunk_info: Option<CodeChunkManifest>,
}

/// 部署结果
#[derive(Debug, Clone)]
pub struct DeploymentResult {
    /// 是否成功
    pub success: bool,

    /// 部署URL
    pub deployment_url: String,

    /// 构建时间
    pub build_time: std::time::Duration,

    /// WASM大小
    pub wasm_size: usize,

    /// 预估加载时间
    pub load_time: u32,
}

/// WASM部署错误
#[derive(thiserror::Error, Debug)]
pub enum WasmDeployError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Optimization error: {0}")]
    OptimizationError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Deployment error: {0}")]
    DeploymentError(String),
}

/// CI/CD集成
pub struct CIPipelineIntegration {
    /// 配置
    config: CIPipelineConfig,
}

/// CI/CD配置
#[derive(Debug, Clone)]
pub struct CIPipelineConfig {
    /// CI平台
    pub platform: CIPlatform,

    /// 自动部署
    pub auto_deploy: bool,

    /// 优化级别
    pub optimization_level: WasmOptLevel,
}

/// CI平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CIPlatform {
    GitHubActions,
    GitLabCI,
    Jenkins,
    CircleCI,
}

impl CIPipelineIntegration {
    /// 生成CI配置文件
    pub fn generate_config(&self) -> Result<String, WasmDeployError> {
        match self.config.platform {
            CIPlatform::GitHubActions => self.generate_github_actions(),
            _ => Err(WasmDeployError::DeploymentError(
                "Platform not yet implemented".to_string(),
            )),
        }
    }

    /// 生成GitHub Actions配置
    fn generate_github_actions(&self) -> Result<String, WasmDeployError> {
        Ok(r#"
name: Build and Deploy WASM

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        target: wasm32-unknown-unknown

    - name: Build WASM
      run: cargo build --target wasm32-unknown-unknown --release

    - name: Install wasm-opt
      run: |
        curl -L https://github.com/WebAssembly/binaryen/releases/download/version_111/binaryen-version_111-x86_64-linux.tar.gz | tar xz

    - name: Optimize WASM
      run: |
        ./bin/wasm-opt -O3 --enable-bulk-memory --enable-simd \
          -o game_engine.opt.wasm \
          target/wasm32-unknown-unknown/release/game_engine.wasm

    - name: Deploy
      run: echo "Deploying to CDN..."
"#.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_deploy_tool_creation() {
        let config = WasmDeployConfig {
            project_path: PathBuf::from("."),
            output_dir: PathBuf::from("./dist"),
            optimization_level: WasmOptLevel::O3,
            enable_code_splitting: true,
            compress_output: true,
            deployment_target: DeploymentTarget::Local,
        };

        let tool = WasmDeployTool::new(config);
        assert_eq!(tool.build_status.total_steps, 5);
    }

    #[test]
    fn test_wasm_bundle_default() {
        let bundle = WasmBundle::default();
        assert_eq!(bundle.total_size, 0);
        assert!(!bundle.compressed);
    }

    #[test]
    fn test_ci_pipeline_github_actions() {
        let ci = CIPipelineIntegration {
            config: CIPipelineConfig {
                platform: CIPlatform::GitHubActions,
                auto_deploy: true,
                optimization_level: WasmOptLevel::O3,
            },
        };

        let config = ci.generate_config();
        assert!(config.is_ok());
    }
}

/// WASM缓存策略
#[derive(Debug, Clone)]
pub struct WasmCacheStrategy {
    /// 缓存控制头（Cache-Control）
    pub cache_control: String,

    /// 服务端工作器（Service Worker）支持
    pub enable_service_worker: bool,

    /// 预缓存资源列表
    pub precache_assets: Vec<String>,

    /// 缓存优先级
    pub cache_priority: CachePriority,
}

/// 缓存优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePriority {
    /// 高优先级（核心WASM）
    High,
    /// 中优先级（资源）
    Medium,
    /// 低优先级（可选内容）
    Low,
}

impl Default for WasmCacheStrategy {
    fn default() -> Self {
        Self {
            cache_control: "public, max-age=31536000, immutable".to_string(),
            enable_service_worker: true,
            precache_assets: vec![
                "game_engine.wasm".to_string(),
                "game_engine.data".to_string(),
            ],
            cache_priority: CachePriority::High,
        }
    }
}

impl WasmCacheStrategy {
    /// 生成缓存头
    pub fn generate_cache_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            ("Cache-Control".to_string(), self.cache_control.clone()),
            ("ETag".to_string(), format!("\"{}\"", uuid::Uuid::new_v4())),
        ];

        if self.enable_service_worker {
            headers.push(("Service-Worker-Allowed".to_string(), "true".to_string()));
        }

        headers
    }

    /// 生成Service Worker脚本
    pub fn generate_service_worker(&self, wasm_url: &str) -> String {
        format!(
            r#"
// Service Worker for WASM Game Engine
const CACHE_NAME = 'game-engine-v1';
const PRECACHE_ASSETS = [
    '{}',
    '{}.data',
];

self.addEventListener('install', (event) => {{
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {{
            return cache.addAll(PRECACHE_ASSETS);
        }})
    );
}});

self.addEventListener('activate', (event) => {{
    event.waitUntil(
        caches.keys().then((cacheNames) => {{
            return Promise.all(
                cacheNames
                    .filter((cacheName) => cacheName !== CACHE_NAME)
                    .map((cacheName) => caches.delete(cacheName))
            );
        }})
    );
}});

self.addEventListener('fetch', (event) => {{
    event.respondWith(
        caches.match(event.request).then((response) => {{
            return response || fetch(event.request);
        }})
    );
}});
"#,
            wasm_url,
            wasm_url.trim_end_matches(".wasm")
        )
    }

    /// 生成资源提示（Resource Hints）
    pub fn generate_resource_hints(&self) -> String {
        let mut hints = String::new();

        // 预加载核心WASM文件
        hints.push_str(&format!(
            "<link rel=\"preload\" href=\"game_engine.wasm\" as=\"fetch\" crossorigin>\n"
        ));

        // 预连接到CDN（如果配置了）
        hints.push_str("<link rel=\"preconnect\" href=\"https://cdn.example.com\">\n");
        hints.push_str("<link rel=\"dns-prefetch\" href=\"https://cdn.example.com\">\n");

        // 预加载关键资源
        for asset in &self.precache_assets {
            hints.push_str(&format!(
                "<link rel=\"preload\" href=\"{}\" as=\"fetch\">\n",
                asset
            ));
        }

        hints
    }
}

/// CDN配置
#[derive(Debug, Clone)]
pub struct CdnConfig {
    /// CDN提供商
    pub provider: CdnProvider,

    /// CDN域名
    pub cdn_domain: String,

    /// 是否启用HTTPS
    pub enable_https: bool,

    /// 自定义域名（CNAME）
    pub custom_domain: Option<String>,

    /// 地理分布
    pub geo_distribution: Vec<String>,
}

/// CDN提供商
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdnProvider {
    /// Cloudflare
    Cloudflare,

    /// AWS CloudFront
    AWSCloudFront,

    /// Fastly
    Fastly,

    /// Azure CDN
    AzureCDN,

    /// Google Cloud CDN
    GoogleCloudCDN,

    /// 自定义
    Custom(String),
}

impl Default for CdnConfig {
    fn default() -> Self {
        Self {
            provider: CdnProvider::Cloudflare,
            cdn_domain: "cdn.example.com".to_string(),
            enable_https: true,
            custom_domain: None,
            geo_distribution: vec![
                "us-east".to_string(),
                "eu-west".to_string(),
                "asia-east".to_string(),
            ],
        }
    }
}

impl CdnConfig {
    /// 生成CDN URL
    pub fn generate_cdn_url(&self, asset_path: &str) -> String {
        let protocol = if self.enable_https { "https" } else { "http" };
        let domain = self.custom_domain.as_ref().unwrap_or(&self.cdn_domain);
        format!("{}://{}/{}", protocol, domain, asset_path)
    }

    /// 生成CDN缓存配置
    pub fn generate_cdn_cache_config(&self) -> String {
        match self.provider {
            CdnProvider::Cloudflare => self.generate_cloudflare_config(),
            CdnProvider::AWSCloudFront => self.generate_cloudfront_config(),
            _ => "// Custom CDN configuration\n".to_string(),
        }
    }

    fn generate_cloudflare_config(&self) -> String {
        format!(
            r#"
# Cloudflare CDN Cache Configuration
# TTL Configuration
_ttl: 2y
cache_ttl: 31536000

# Browser Cache
browser_ttl: 604800

# Edge Cache
edge_cache_ttl: 604800

# Cache Key (ignore query strings for WASM files)
cache_key: {{
    main: {{
        path: {{ ignore: true }}
    }}
}}

# Security
https: true
security_level: high
ssl_mode: flexible

# Performance
minify: true
rocket_loader: false
brotli: true
"#
        )
    }

    fn generate_cloudfront_config(&self) -> String {
        format!(
            r#"
# AWS CloudFront Distribution Configuration
CacheBehavior:
  TargetOriginId: wasm-origin
  ViewerProtocolPolicy: redirect-to-https
  MinTTL: 31536000
  MaxTTL: 31536000
  DefaultTTL: 86400
  Compress: true
  LambdaFunctionAssociations:
    - EventType: origin-response
      LambdaFunctionARN: arn:aws:lambda:...

Origin:
  Id: wasm-origin
  DomainName: {}
  CustomHeaders:
    - Name: Cache-Control
      Value: public, max-age=31536000, immutable
    - Name: Service-Worker-Allowed
      Value: true
"#,
            self.cdn_domain
        )
    }
}

/// WASM性能监控
#[derive(Debug, Clone)]
pub struct WasmPerformanceMonitor {
    /// 监控数据
    pub metrics: PerformanceMetrics,

    /// 是否启用监控
    pub enabled: bool,

    /// 监控端点
    pub monitoring_endpoint: Option<String>,
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 加载时间（毫秒）
    pub load_time: u32,

    /// 首次内容绘制（FCP）
    pub first_contentful_paint: u32,

    /// 最大内容绘制（LCP）
    pub largest_contentful_paint: u32,

    /// 首次输入延迟（FID）
    pub first_input_delay: u32,

    /// 累积布局偏移（CLS）
    pub cumulative_layout_shift: f32,

    /// WASM编译时间
    pub wasm_compilation_time: u32,

    /// 内存使用
    pub memory_usage: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            load_time: 0,
            first_contentful_paint: 0,
            largest_contentful_paint: 0,
            first_input_delay: 0,
            cumulative_layout_shift: 0.0,
            wasm_compilation_time: 0,
            memory_usage: 0,
        }
    }
}

impl Default for WasmPerformanceMonitor {
    fn default() -> Self {
        Self {
            metrics: PerformanceMetrics::default(),
            enabled: true,
            monitoring_endpoint: None,
        }
    }
}

impl WasmPerformanceMonitor {
    /// 创建性能监控器
    pub fn new() -> Self {
        Self::default()
    }

    /// 生成性能监控脚本
    pub fn generate_monitoring_script(&self) -> String {
        format!(
            r#"
// WASM Performance Monitoring
(function() {{
    const perfData = {{
        loadTime: 0,
        fcp: 0,
        lcp: 0,
        fid: 0,
        cls: 0,
        wasmCompilationTime: 0,
        memoryUsage: 0
    }};

    // Measure page load time
    window.addEventListener('load', () => {{
        const perfData = performance.getEntriesByType('navigation')[0];
        if (perfData) {{
            perfData.loadTime = perfData.loadEventEnd - perfData.fetchStart;
        }}
    }});

    // Measure FCP
    new PerformanceObserver((list) => {{
        const entries = list.getEntries();
        const fcpEntry = entries.find(entry => entry.name === 'first-contentful-paint');
        if (fcpEntry) {{
            perfData.fcp = Math.round(fcpEntry.startTime);
        }}
    }}).observe({{ type: 'paint', buffered: true }});

    // Measure LCP
    new PerformanceObserver((list) => {{
        const entries = list.getEntries();
        const lastEntry = entries[entries.length - 1];
        perfData.lcp = Math.round(lastEntry.startTime);
    }}).observe({{ type: 'largest-contentful-paint', buffered: true }});

    // Measure CLS
    let clsValue = 0;
    new PerformanceObserver((list) => {{
        for (const entry of list.getEntries()) {{
            if (!entry.hadRecentInput) {{
                clsValue += entry.value;
            }}
        }}
        perfData.cls = clsValue.toFixed(3);
    }}).observe({{ type: 'layout-shift', buffered: true }});

    // Measure WASM compilation time
    const wasmStartTime = performance.now();
    WebAssembly.instantiateStreaming(fetch('game_engine.wasm')).then(results => {{
        perfData.wasmCompilationTime = Math.round(performance.now() - wasmStartTime);
        return results;
    }});

    // Measure memory usage
    setInterval(() => {{
        if (performance.memory) {{
            perfData.memoryUsage = performance.memory.usedJSHeapSize;
        }}
    }}, 1000);

    // Send metrics to endpoint
    function sendMetrics() {{
        fetch('/api/metrics', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify(perfData)
        }}).catch(console.error);
    }}

    // Send metrics on page unload
    window.addEventListener('beforeunload', sendMetrics);

    // Expose metrics globally for debugging
    window.wasmPerformanceData = perfData;
}})();
"#
        )
    }

    /// 生成Web Vitals报告
    pub fn generate_web_vitals_report(&self) -> String {
        format!(
            r#"
<!-- Web Vitals Report -->
<script>
function sendWebVitals() {{
    // Use web-vitals library to measure Core Web Vitals
    import('https://unpkg.com/web-vitals').then(({{ getCLS, getFID, getLCP }}) => {{
        getCLS((metric) => {{
            console.log('CLS:', metric.value);
            // Send to analytics
        }});

        getFID((metric) => {{
            console.log('FID:', metric.value);
            // Send to analytics
        }});

        getLCP((metric) => {{
            console.log('LCP:', metric.value);
            // Send to analytics
        }});
    }});
}}

sendWebVitals();
</script>
"#
        )
    }

    /// 记录性能指标
    pub fn record_metric(&mut self, metric_name: &str, value: f64) {
        match metric_name {
            "load_time" => self.metrics.load_time = value as u32,
            "fcp" => self.metrics.first_contentful_paint = value as u32,
            "lcp" => self.metrics.largest_contentful_paint = value as u32,
            "fid" => self.metrics.first_input_delay = value as u32,
            "cls" => self.metrics.cumulative_layout_shift = value as f32,
            "wasm_compilation_time" => self.metrics.wasm_compilation_time = value as u32,
            "memory_usage" => self.metrics.memory_usage = value as usize,
            _ => tracing::warn!("Unknown metric: {}", metric_name),
        }
    }
}
