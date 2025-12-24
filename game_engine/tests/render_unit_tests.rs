//! 渲染系统单元测试
//!
//! 测试渲染系统的核心功能，包括：
//! - 着色器编译和缓存
//! - 纹理加载和管理
//! - 渲染管线创建
//! - 着色器键生成
//! - 纹理格式检测

use game_engine::render::shader_cache::{
    CleanupStrategy, ShaderCache, ShaderCacheConfig, ShaderCacheKey,
};
use game_engine::render::texture_compression::CompressedTextureFormat;
use tempfile::TempDir;

// ============================================================================
// 着色器缓存测试
// ============================================================================

#[test]
fn test_shader_cache_key_generation() {
    let source1 = "fn main() { }";
    let source2 = "fn main() { }";
    let source3 = "fn main() { let x = 1; }";
    let options = "";

    let key1 = ShaderCacheKey::from_source(source1, options);
    let key2 = ShaderCacheKey::from_source(source2, options);
    let key3 = ShaderCacheKey::from_source(source3, options);

    // 相同源码应该生成相同的键
    assert_eq!(key1, key2);

    // 不同源码应该生成不同的键
    assert_ne!(key1, key3);
}

#[test]
fn test_shader_cache_key_with_options() {
    let source = "fn main() { }";
    let options1 = "optimize";
    let options2 = "optimize";
    let options3 = "debug";

    let key1 = ShaderCacheKey::from_source(source, options1);
    let key2 = ShaderCacheKey::from_source(source, options2);
    let key3 = ShaderCacheKey::from_source(source, options3);

    // 相同选项应该生成相同的键
    assert_eq!(key1, key2);

    // 不同选项应该生成不同的键
    assert_ne!(key1, key3);
}

#[test]
fn test_shader_cache_key_filename() {
    let source = "fn main() { }";
    let options = "";
    let key = ShaderCacheKey::from_source(source, options);

    let filename = key.cache_filename();
    let metadata_filename = key.metadata_filename();

    // 文件名应该包含哈希
    assert!(filename.contains(".spv") || filename.contains(".bin"));
    assert!(metadata_filename.contains(".meta") || metadata_filename.contains(".json"));
}

#[test]
fn test_shader_cache_creation() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let config = ShaderCacheConfig {
        cache_dir: Some(cache_dir.clone()),
        max_cache_size_bytes: 100 * 1024 * 1024, // 100MB
        ..Default::default()
    };

    let cache = ShaderCache::new(config);
    assert!(cache.is_ok());
}

#[test]
fn test_shader_cache_default_config() {
    let cache = ShaderCache::with_default_config();
    assert!(cache.is_ok());
}

#[test]
fn test_shader_cache_stats() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let config = ShaderCacheConfig {
        cache_dir: Some(cache_dir),
        max_cache_size_bytes: 100 * 1024 * 1024,
        ..Default::default()
    };

    let mut cache = ShaderCache::new(config).unwrap();
    let stats = cache.stats();

    // 初始统计应该为零
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.total_requests, 0);
}

#[test]
fn test_shader_cache_put_and_get() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let config = ShaderCacheConfig {
        cache_dir: Some(cache_dir),
        max_cache_size_bytes: 100 * 1024 * 1024,
        ..Default::default()
    };

    let mut cache = ShaderCache::new(config).unwrap();

    let source = "fn main() { }";
    let options = "";
    let key = ShaderCacheKey::from_source(source, options);

    // 存储着色器
    let binary_data = b"fake_spirv_binary_data".to_vec();
    let result = cache.put_binary(&key, &binary_data);
    assert!(result.is_ok());

    // 获取着色器
    let retrieved = cache.get(&key).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), binary_data);

    // 验证统计
    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.total_requests, 1);
}

#[test]
fn test_shader_cache_miss() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let config = ShaderCacheConfig {
        cache_dir: Some(cache_dir),
        max_cache_size_bytes: 100 * 1024 * 1024,
        ..Default::default()
    };

    let mut cache = ShaderCache::new(config).unwrap();

    let source = "fn main() { }";
    let options = "";
    let key = ShaderCacheKey::from_source(source, options);

    // 获取不存在的着色器
    let retrieved = cache.get(&key).unwrap();
    assert!(retrieved.is_none());

    // 验证统计
    let stats = cache.stats();
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.total_requests, 1);
}

#[test]
fn test_shader_cache_clear() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let config = ShaderCacheConfig {
        cache_dir: Some(cache_dir),
        max_cache_size_bytes: 100 * 1024 * 1024,
        ..Default::default()
    };

    let mut cache = ShaderCache::new(config).unwrap();

    // 存储一些数据
    let source = "fn main() { }";
    let options = "";
    let key = ShaderCacheKey::from_source(source, options);
    let binary_data = b"fake_spirv_binary_data".to_vec();
    let _ = cache.put_binary(&key, &binary_data);

    // 清空缓存
    let result = cache.clear();
    assert!(result.is_ok());

    // 验证缓存已清空
    let retrieved = cache.get(&key).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_shader_cache_hit_rate() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let config = ShaderCacheConfig {
        cache_dir: Some(cache_dir),
        max_cache_size_bytes: 100 * 1024 * 1024,
        ..Default::default()
    };

    let mut cache = ShaderCache::new(config).unwrap();

    let source = "fn main() { }";
    let options = "";
    let key = ShaderCacheKey::from_source(source, options);
    let binary_data = b"fake_spirv_binary_data".to_vec();

    // 存储并获取
    let _ = cache.put_binary(&key, &binary_data);
    let _ = cache.get(&key);
    let _ = cache.get(&key);

    // 验证命中率
    let stats = cache.stats();
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.total_requests, 2);
    assert!((stats.hit_rate() - 1.0).abs() < 0.01); // 100%命中率
}

// ============================================================================
// 纹理格式测试
// ============================================================================

#[test]
fn test_compressed_texture_format_detection() {
    use game_engine::render::texture_compression::TextureFormatDetector;

    // 测试BC格式检测
    let bc_data = vec![0u8; 16]; // 简化的BC格式头
    // 注意：实际实现需要根据格式规范检测
    // 这里只是测试接口存在

    // 测试ASTC格式检测
    let astc_data = vec![0u8; 16]; // 简化的ASTC格式头
    // 注意：实际实现需要根据格式规范检测
}

#[test]
fn test_compressed_texture_format_enum() {
    // 测试格式枚举值
    let format = CompressedTextureFormat::Bc1RgbUnorm;
    // 验证格式可以序列化/反序列化
    let serialized = serde_json::to_string(&format);
    assert!(serialized.is_ok());
}

// ============================================================================
// 渲染管线测试
// ============================================================================

#[test]
fn test_pipeline_builder_exists() {
    // 验证PipelineBuilder存在
    use game_engine::render::wgpu_modules::pipeline::PipelineBuilder;

    // PipelineBuilder是零大小的结构体，主要用于组织方法
    let _builder = PipelineBuilder;
    assert!(std::mem::size_of::<PipelineBuilder>() == 0);
}

// ============================================================================
// 着色器异步编译测试
// ============================================================================

#[tokio::test]
async fn test_async_shader_compiler_config() {
    use game_engine::render::shader_async::AsyncShaderCompiler;

    let compiler = AsyncShaderCompiler::with_default_config();
    assert!(compiler.is_ok());
}

#[tokio::test]
async fn test_async_shader_compiler_compile() {
    use game_engine::render::shader_async::{AsyncShaderCompiler, ShaderCompilePriority};

    let compiler = AsyncShaderCompiler::with_default_config().unwrap();

    let source = "fn main() { }";
    let result = compiler.compile(None, source);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_shader_compiler_priority() {
    use game_engine::render::shader_async::{AsyncShaderCompiler, ShaderCompilePriority};

    let compiler = AsyncShaderCompiler::with_default_config().unwrap();

    // 测试不同优先级
    let _rx_low = compiler.compile_async(None, "low", "", ShaderCompilePriority::Low);
    let _rx_high = compiler.compile_async(None, "high", "", ShaderCompilePriority::High);
    let _rx_critical =
        compiler.compile_async(None, "critical", "", ShaderCompilePriority::Critical);
}

// ============================================================================
// 纹理管理器测试
// ============================================================================

#[test]
fn test_texture_manager_compression_config() {
    // 测试纹理管理器的压缩配置接口
    // 注意：实际创建需要wgpu设备，这里只测试接口存在
    use game_engine::render::wgpu_modules::texture::TextureManager;

    // 验证类型存在
    assert!(std::mem::size_of::<TextureManager>() > 0);
}

// ============================================================================
// 着色器缓存配置测试
// ============================================================================

#[test]
fn test_shader_cache_config_default() {
    let config = ShaderCacheConfig::default();

    // 验证默认配置
    assert!(config.max_cache_size_bytes > 0);
    assert!(config.enable_compression);
}

#[test]
fn test_shader_cache_config_custom() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let config = ShaderCacheConfig {
        cache_dir: Some(cache_dir),
        max_cache_size_bytes: 50 * 1024 * 1024, // 50MB
        enable_compression: false,
        cleanup_strategy: CleanupStrategy::LRU,
    };

    assert_eq!(config.max_cache_size_bytes, 50 * 1024 * 1024);
    assert!(!config.enable_compression);
}

// ============================================================================
// 集成测试：着色器缓存完整流程
// ============================================================================

#[test]
fn test_shader_cache_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let config = ShaderCacheConfig {
        cache_dir: Some(cache_dir),
        max_cache_size_bytes: 100 * 1024 * 1024,
        ..Default::default()
    };

    let mut cache = ShaderCache::new(config).unwrap();

    // 1. 创建着色器键
    let source1 = "fn main() { let x = 1; }";
    let source2 = "fn main() { let y = 2; }";
    let options = "";

    let key1 = ShaderCacheKey::from_source(source1, options);
    let key2 = ShaderCacheKey::from_source(source2, options);

    // 2. 存储多个着色器
    let binary1 = b"shader1_binary".to_vec();
    let binary2 = b"shader2_binary".to_vec();

    assert!(cache.put_binary(&key1, &binary1).is_ok());
    assert!(cache.put_binary(&key2, &binary2).is_ok());

    // 3. 检索着色器
    let retrieved1 = cache.get(&key1).unwrap();
    let retrieved2 = cache.get(&key2).unwrap();

    assert_eq!(retrieved1, Some(binary1));
    assert_eq!(retrieved2, Some(binary2));

    // 4. 验证统计
    let stats = cache.stats();
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.total_requests, 2);

    // 5. 测试缓存失效（通过不同的键）
    let key3 = ShaderCacheKey::from_source("fn main() { let z = 3; }", options);
    let retrieved3 = cache.get(&key3).unwrap();
    assert!(retrieved3.is_none());

    // 6. 验证最终统计
    let final_stats = cache.stats();
    assert_eq!(final_stats.hits, 2);
    assert_eq!(final_stats.misses, 1);
    assert_eq!(final_stats.total_requests, 3);
}

// ============================================================================
// 边界情况测试
// ============================================================================

#[test]
fn test_shader_cache_empty_source() {
    let source = "";
    let options = "";
    let key = ShaderCacheKey::from_source(source, options);

    // 空源码应该也能生成有效的键
    assert!(!key.cache_filename().is_empty());
}

#[test]
fn test_shader_cache_very_long_source() {
    // 测试非常长的源码
    let source = "fn main() { ".repeat(10000) + " }";
    let options = "";
    let key = ShaderCacheKey::from_source(&source, options);

    // 应该能正常生成键
    assert!(!key.cache_filename().is_empty());
}

#[test]
fn test_shader_cache_special_characters() {
    // 测试特殊字符
    let source = "fn main() { let x = \"特殊字符\"; }";
    let options = "";
    let key = ShaderCacheKey::from_source(source, options);

    // 应该能正常处理
    assert!(!key.cache_filename().is_empty());
}

#[test]
fn test_shader_cache_unicode() {
    // 测试Unicode字符
    let source = "fn main() { let x = \"🚀\"; }";
    let options = "";
    let key = ShaderCacheKey::from_source(source, options);

    // 应该能正常处理
    assert!(!key.cache_filename().is_empty());
}
