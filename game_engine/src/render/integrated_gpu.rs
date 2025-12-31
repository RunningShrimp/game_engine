//! # 集成显卡优化
//!
//! 为集成显卡（Intel HD/UHD Graphics, AMD APU, Apple Silicon等）提供专门的渲染优化。
//!
//! ## 优化目标
//!
//! - **减少显存带宽**: 压缩纹理、优化buffer访问
//! - **降低填充率**: 减少overdraw、优化像素着色器
//! - **共享内存管理**: 优化系统RAM和GPU显存共享
//! - **计算着色器优化**: 减少GPU计算负载
//!
//! ## 功能特性
//!
//! - **自适应质量**: 根据GPU性能动态调整
//! - **带宽优化**: 纹理压缩、mipmap优化
//! - **渲染策略**: 降低分辨率、简化着色器
//! - **内存管理**: 优化buffer分配和使用
//!
//! ## 使用场景
//!
//! - **轻薄本**: 低功耗集成显卡
//! - **办公PC**: 无独立显卡的桌面系统
//! - **移动设备**: 集成GPU的手机/平板
//! - **Apple Silicon**: M1/M2/M3系列芯片

use std::sync::atomic::{AtomicUsize, Ordering};

/// 集成显卡性能级别
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegratedGpuTier {
    /// 低端集成显卡 (Intel HD 4000及以下)
    Low = 0,
    /// 中端集成显卡 (Intel UHD, AMD APU)
    Medium = 1,
    /// 高端集成显卡 (Intel Iris Xe, Apple M1/M2)
    High = 2,
}

/// 集成显卡配置
#[derive(Clone, Debug)]
pub struct IntegratedGpuConfig {
    /// 性能级别
    pub tier: IntegratedGpuTier,
    /// 共享内存限制（MB）
    pub shared_memory_mb: usize,
    /// 是否启用带宽优化
    pub enable_bandwidth_optimization: bool,
    /// 是否启用着色器简化
    pub enable_shader_simplification: bool,
    /// 渲染缩放比例 (0.5 - 1.0)
    pub render_scale: f32,
    /// 纹理质量 (0.0 - 1.0)
    pub texture_quality: f32,
    /// 阴影质量 (0.0 - 1.0)
    pub shadow_quality: f32,
    /// 最大动态灯光数量
    pub max_dynamic_lights: usize,
}

impl Default for IntegratedGpuConfig {
    fn default() -> Self {
        Self {
            tier: IntegratedGpuTier::Medium,
            shared_memory_mb: 512, // 默认512MB共享显存
            enable_bandwidth_optimization: true,
            enable_shader_simplification: true,
            render_scale: 0.75, // 默认75%分辨率渲染
            texture_quality: 0.75,
            shadow_quality: 0.5,
            max_dynamic_lights: 4,
        }
    }
}

impl IntegratedGpuConfig {
    /// 创建低端集成显卡配置
    pub fn low_end() -> Self {
        Self {
            tier: IntegratedGpuTier::Low,
            shared_memory_mb: 256,
            render_scale: 0.5,
            texture_quality: 0.5,
            shadow_quality: 0.25,
            max_dynamic_lights: 2,
            ..Default::default()
        }
    }

    /// 创建中端集成显卡配置
    pub fn mid_range() -> Self {
        Self::default()
    }

    /// 创建高端集成显卡配置
    pub fn high_end() -> Self {
        Self {
            tier: IntegratedGpuTier::High,
            shared_memory_mb: 1024,
            render_scale: 0.9,
            texture_quality: 0.9,
            shadow_quality: 0.75,
            max_dynamic_lights: 8,
            ..Default::default()
        }
    }

    /// 根据GPU名称自动创建配置
    pub fn from_gpu_name(gpu_name: &str) -> Self {
        let gpu_lower = gpu_name.to_lowercase();

        // Apple Silicon
        if gpu_lower.contains("m1") || gpu_lower.contains("m2") || gpu_lower.contains("m3") {
            return Self::high_end();
        }

        // Intel Iris Xe / Iris Plus
        if gpu_lower.contains("iris xe") || gpu_lower.contains("iris plus") {
            return Self::high_end();
        }

        // Intel UHD
        if gpu_lower.contains("uhd") {
            return Self::mid_range();
        }

        // Intel HD
        if gpu_lower.contains("hd graphics") {
            // HD 4000-6000: 中端
            if gpu_lower.contains("hd 4")
                || gpu_lower.contains("hd 5")
                || gpu_lower.contains("hd 6")
            {
                return Self::mid_range();
            }
            // HD 2000-3000: 低端
            return Self::low_end();
        }

        // AMD APU
        if gpu_lower.contains(" Radeon ") {
            // Radeon Vega/RDNA: 高端
            if gpu_lower.contains("vega") || gpu_lower.contains("rdna") {
                return Self::high_end();
            }
            return Self::mid_range();
        }

        // 默认中端配置
        Self::mid_range()
    }
}

/// 带宽优化策略
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BandwidthOptimization {
    /// 无优化
    None,
    /// 轻度优化（纹理压缩）
    Light,
    /// 中度优化（纹理压缩 + mipmap优化）
    Medium,
    /// 重度优化（纹理压缩 + mipmap + 降低分辨率）
    Heavy,
}

/// 着色器简化级别
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderSimplification {
    /// 完整着色器
    Full,
    /// 移除高级特性
    Simplified,
    /// 基础着色器
    Basic,
}

/// 集成显卡优化管理器
pub struct IntegratedGpuOptimizer {
    config: IntegratedGpuConfig,
    current_bandwidth_usage: AtomicUsize, // 字节
    peak_bandwidth_usage: AtomicUsize,    // 字节
}

impl IntegratedGpuOptimizer {
    /// 创建新的优化管理器
    pub fn new(config: IntegratedGpuConfig) -> Self {
        Self {
            config,
            current_bandwidth_usage: AtomicUsize::new(0),
            peak_bandwidth_usage: AtomicUsize::new(0),
        }
    }

    /// 从GPU名称自动检测并创建优化器
    pub fn from_gpu_detection(gpu_name: &str) -> Self {
        let config = IntegratedGpuConfig::from_gpu_name(gpu_name);
        Self::new(config)
    }

    /// 获取配置
    pub fn config(&self) -> &IntegratedGpuConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: IntegratedGpuConfig) {
        self.config = config;
    }

    /// 记录带宽使用
    pub fn record_bandwidth_usage(&self, bytes: usize) {
        self.current_bandwidth_usage.store(bytes, Ordering::Relaxed);
        let peak = self.peak_bandwidth_usage.load(Ordering::Relaxed);
        if bytes > peak {
            self.peak_bandwidth_usage.store(bytes, Ordering::Relaxed);
        }
    }

    /// 获取当前带宽使用
    pub fn current_bandwidth_usage(&self) -> usize {
        self.current_bandwidth_usage.load(Ordering::Relaxed)
    }

    /// 获取峰值带宽使用
    pub fn peak_bandwidth_usage(&self) -> usize {
        self.peak_bandwidth_usage.load(Ordering::Relaxed)
    }

    /// 获取推荐带宽优化级别
    pub fn recommended_bandwidth_optimization(&self) -> BandwidthOptimization {
        if !self.config.enable_bandwidth_optimization {
            return BandwidthOptimization::None;
        }

        match self.config.tier {
            IntegratedGpuTier::Low => BandwidthOptimization::Heavy,
            IntegratedGpuTier::Medium => BandwidthOptimization::Medium,
            IntegratedGpuTier::High => BandwidthOptimization::Light,
        }
    }

    /// 获取推荐着色器简化级别
    pub fn recommended_shader_simplification(&self) -> ShaderSimplification {
        if !self.config.enable_shader_simplification {
            return ShaderSimplification::Full;
        }

        match self.config.tier {
            IntegratedGpuTier::Low => ShaderSimplification::Basic,
            IntegratedGpuTier::Medium => ShaderSimplification::Simplified,
            IntegratedGpuTier::High => ShaderSimplification::Full,
        }
    }

    /// 获取推荐的纹理压缩格式
    pub fn recommended_texture_compression(&self) -> TextureCompressionFormat {
        match self.config.tier {
            IntegratedGpuTier::Low => TextureCompressionFormat::Bc3,
            IntegratedGpuTier::Medium => TextureCompressionFormat::Bc3,
            IntegratedGpuTier::High => TextureCompressionFormat::Bc7,
        }
    }

    /// 获取推荐的最大纹理尺寸
    pub fn recommended_max_texture_size(&self) -> u32 {
        match self.config.tier {
            IntegratedGpuTier::Low => 1024,
            IntegratedGpuTier::Medium => 2048,
            IntegratedGpuTier::High => 4096,
        }
    }

    /// 是否启用渲染缩放
    pub fn should_use_render_scale(&self) -> bool {
        self.config.render_scale < 1.0
    }

    /// 获取渲染缩放比例
    pub fn render_scale(&self) -> f32 {
        self.config.render_scale
    }

    /// 是否应该降低阴影贴图分辨率
    pub fn should_reduce_shadow_resolution(&self) -> bool {
        self.config.shadow_quality < 1.0
    }

    /// 获取阴影贴图缩放比例
    pub fn shadow_map_scale(&self) -> f32 {
        self.config.shadow_quality
    }

    /// 是否应该限制动态灯光数量
    pub fn should_limit_dynamic_lights(&self) -> bool {
        self.config.max_dynamic_lights < 16
    }

    /// 获取最大动态灯光数量
    pub fn max_dynamic_lights(&self) -> usize {
        self.config.max_dynamic_lights
    }

    /// 计算推荐的渲染分辨率
    pub fn recommended_render_resolution(
        &self,
        display_width: u32,
        display_height: u32,
    ) -> (u32, u32) {
        if self.config.render_scale >= 1.0 {
            return (display_width, display_height);
        }

        let scaled_width = (display_width as f32 * self.config.render_scale) as u32;
        let scaled_height = (display_height as f32 * self.config.render_scale) as u32;

        // 确保至少是320x240
        (scaled_width.max(320), scaled_height.max(240))
    }
}

/// 纹理压缩格式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureCompressionFormat {
    /// BC1 (DXT1) - 4:1压缩比，无alpha
    Bc1,
    /// BC2 (DXT3) - 4:1压缩比，明确alpha
    Bc2,
    /// BC3 (DXT5) - 4:1压缩比，插值alpha
    Bc3,
    /// BC4 - 2:1压缩比，单通道
    Bc4,
    /// BC5 - 2:1压缩比，双通道
    Bc5,
    /// BC6H - 3:1压缩比，HDR RGB
    Bc6h,
    /// BC7 - 3:1压缩比，高质量RGBA
    Bc7,
    /// ASTC 4x4 - 高质量可变压缩
    Astc4x4,
    /// ETC2 - 移动端压缩
    Etc2,
}

impl TextureCompressionFormat {
    /// 获取压缩比
    pub fn compression_ratio(&self) -> f32 {
        match self {
            TextureCompressionFormat::Bc1
            | TextureCompressionFormat::Bc2
            | TextureCompressionFormat::Bc3 => 4.0,
            TextureCompressionFormat::Bc4 | TextureCompressionFormat::Bc5 => 2.0,
            TextureCompressionFormat::Bc6h | TextureCompressionFormat::Bc7 => 3.0,
            TextureCompressionFormat::Astc4x4 => 4.0,
            TextureCompressionFormat::Etc2 => 4.0,
        }
    }

    /// 是否支持alpha通道
    pub fn has_alpha(&self) -> bool {
        matches!(
            self,
            TextureCompressionFormat::Bc2
                | TextureCompressionFormat::Bc3
                | TextureCompressionFormat::Bc5
                | TextureCompressionFormat::Bc7
                | TextureCompressionFormat::Astc4x4
        )
    }
}

/// 带宽监控器
pub struct BandwidthMonitor {
    texture_bandwidth: AtomicUsize,
    vertex_bandwidth: AtomicUsize,
    index_bandwidth: AtomicUsize,
    uniform_bandwidth: AtomicUsize,
}

impl BandwidthMonitor {
    pub fn new() -> Self {
        Self {
            texture_bandwidth: AtomicUsize::new(0),
            vertex_bandwidth: AtomicUsize::new(0),
            index_bandwidth: AtomicUsize::new(0),
            uniform_bandwidth: AtomicUsize::new(0),
        }
    }

    /// 记录纹理带宽
    pub fn record_texture_bandwidth(&self, bytes: usize) {
        self.texture_bandwidth.fetch_add(bytes, Ordering::Relaxed);
    }

    /// 记录顶点带宽
    pub fn record_vertex_bandwidth(&self, bytes: usize) {
        self.vertex_bandwidth.fetch_add(bytes, Ordering::Relaxed);
    }

    /// 记录索引带宽
    pub fn record_index_bandwidth(&self, bytes: usize) {
        self.index_bandwidth.fetch_add(bytes, Ordering::Relaxed);
    }

    /// 记录Uniform带宽
    pub fn record_uniform_bandwidth(&self, bytes: usize) {
        self.uniform_bandwidth.fetch_add(bytes, Ordering::Relaxed);
    }

    /// 获取总带宽
    pub fn total_bandwidth(&self) -> usize {
        self.texture_bandwidth.load(Ordering::Relaxed)
            + self.vertex_bandwidth.load(Ordering::Relaxed)
            + self.index_bandwidth.load(Ordering::Relaxed)
            + self.uniform_bandwidth.load(Ordering::Relaxed)
    }

    /// 重置计数器
    pub fn reset(&self) {
        self.texture_bandwidth.store(0, Ordering::Relaxed);
        self.vertex_bandwidth.store(0, Ordering::Relaxed);
        self.index_bandwidth.store(0, Ordering::Relaxed);
        self.uniform_bandwidth.store(0, Ordering::Relaxed);
    }

    /// 获取带宽分布
    pub fn bandwidth_distribution(&self) -> BandwidthDistribution {
        let total = self.total_bandwidth();
        if total == 0 {
            return BandwidthDistribution {
                texture_percent: 0.0,
                vertex_percent: 0.0,
                index_percent: 0.0,
                uniform_percent: 0.0,
            };
        }

        BandwidthDistribution {
            texture_percent: (self.texture_bandwidth.load(Ordering::Relaxed) as f32 / total as f32)
                * 100.0,
            vertex_percent: (self.vertex_bandwidth.load(Ordering::Relaxed) as f32 / total as f32)
                * 100.0,
            index_percent: (self.index_bandwidth.load(Ordering::Relaxed) as f32 / total as f32)
                * 100.0,
            uniform_percent: (self.uniform_bandwidth.load(Ordering::Relaxed) as f32 / total as f32)
                * 100.0,
        }
    }
}

impl Default for BandwidthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 带宽分布统计
#[derive(Clone, Copy, Debug)]
pub struct BandwidthDistribution {
    pub texture_percent: f32,
    pub vertex_percent: f32,
    pub index_percent: f32,
    pub uniform_percent: f32,
}

/// 渲染分辨率适配器
pub struct ResolutionScaler {
    base_width: u32,
    base_height: u32,
    current_scale: f32,
}

impl ResolutionScaler {
    /// 创建分辨率缩放器
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            base_width: width,
            base_height: height,
            current_scale: 1.0,
        }
    }

    /// 设置缩放比例
    pub fn set_scale(&mut self, scale: f32) {
        self.current_scale = scale.clamp(0.25, 1.0);
    }

    /// 获取当前缩放分辨率
    pub fn scaled_resolution(&self) -> (u32, u32) {
        let width = (self.base_width as f32 * self.current_scale) as u32;
        let height = (self.base_height as f32 * self.current_scale) as u32;
        (width.max(320), height.max(240))
    }

    /// 获取缩放比例
    pub fn scale(&self) -> f32 {
        self.current_scale
    }

    /// 根据性能自动调整
    pub fn auto_adjust_from_fps(&mut self, fps: f32) {
        const TARGET_FPS: f32 = 60.0;
        const MIN_FPS: f32 = 30.0;

        if fps < MIN_FPS {
            // 性能不足，降低分辨率
            self.current_scale = (self.current_scale - 0.05).max(0.5);
        } else if fps > TARGET_FPS + 10.0 && self.current_scale < 1.0 {
            // 性能富余，提升分辨率
            self.current_scale = (self.current_scale + 0.05).min(1.0);
        }
    }
}

// =============================================================================
// 辅助函数
// =============================================================================

/// 检测是否为集成显卡
pub fn is_integrated_gpu(gpu_name: &str) -> bool {
    let gpu_lower = gpu_name.to_lowercase();

    // Intel集成显卡
    if gpu_lower.contains("hd graphics") || gpu_lower.contains("uhd") || gpu_lower.contains("iris")
    {
        return true;
    }

    // AMD APU
    if gpu_lower.contains("apu") {
        return true;
    }

    // Apple Silicon
    if gpu_lower.contains("m1") || gpu_lower.contains("m2") || gpu_lower.contains("m3") {
        return true;
    }

    false
}

/// 获取集成显卡性能级别
pub fn get_integrated_gpu_tier(gpu_name: &str) -> IntegratedGpuTier {
    IntegratedGpuConfig::from_gpu_name(gpu_name).tier
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_detection() {
        assert!(is_integrated_gpu("Intel HD Graphics 630"));
        assert!(is_integrated_gpu("Intel UHD Graphics 620"));
        assert!(is_integrated_gpu("Apple M1"));
        assert!(is_integrated_gpu("AMD Radeon APU"));
        assert!(!is_integrated_gpu("NVIDIA GeForce RTX 3080"));
    }

    #[test]
    fn test_config_from_gpu_name() {
        let config = IntegratedGpuConfig::from_gpu_name("Intel HD Graphics 4000");
        assert_eq!(config.tier, IntegratedGpuTier::Low);

        let config = IntegratedGpuConfig::from_gpu_name("Intel UHD Graphics 620");
        assert_eq!(config.tier, IntegratedGpuTier::Medium);

        let config = IntegratedGpuConfig::from_gpu_name("Apple M1");
        assert_eq!(config.tier, IntegratedGpuTier::High);
    }

    #[test]
    fn test_render_resolution() {
        let optimizer = IntegratedGpuOptimizer::new(IntegratedGpuConfig {
            render_scale: 0.5,
            ..Default::default()
        });

        let (width, height) = optimizer.recommended_render_resolution(1920, 1080);
        assert_eq!(width, 960);
        assert_eq!(height, 540);
    }

    #[test]
    fn test_texture_compression() {
        assert_eq!(TextureCompressionFormat::Bc3.compression_ratio(), 4.0);
        assert!(TextureCompressionFormat::Bc7.has_alpha());
        assert!(!TextureCompressionFormat::Bc1.has_alpha());
    }

    #[test]
    fn test_resolution_scaler() {
        let mut scaler = ResolutionScaler::new(1920, 1080);
        scaler.set_scale(0.5);
        assert_eq!(scaler.scaled_resolution(), (960, 540));

        scaler.set_scale(0.75);
        assert_eq!(scaler.scaled_resolution(), (1440, 810));
    }

    #[test]
    fn test_bandwidth_monitor() {
        let monitor = BandwidthMonitor::new();
        monitor.record_texture_bandwidth(1000);
        monitor.record_vertex_bandwidth(500);

        assert_eq!(monitor.total_bandwidth(), 1500);

        let dist = monitor.bandwidth_distribution();
        assert!((dist.texture_percent - 66.67).abs() < 0.1);
        assert!((dist.vertex_percent - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_optimization_recommendations() {
        let optimizer = IntegratedGpuOptimizer::new(IntegratedGpuConfig::low_end());

        assert_eq!(
            optimizer.recommended_bandwidth_optimization(),
            BandwidthOptimization::Heavy
        );
        assert_eq!(
            optimizer.recommended_shader_simplification(),
            ShaderSimplification::Basic
        );
        assert_eq!(optimizer.recommended_max_texture_size(), 1024);
    }
}
