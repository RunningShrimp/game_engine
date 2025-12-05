//! 着色器缓存辅助函数
//!
//! 提供便捷的着色器创建函数，自动集成缓存和异步编译功能。

use crate::core::error::RenderError;
use crate::render::shader_async::{wait_for_compile, AsyncShaderCompiler, ShaderCompilePriority};
use crate::render::shader_cache::{ShaderCache, ShaderCacheKey};

/// 创建带缓存的着色器模块
///
/// 自动检查缓存，如果缓存命中则使用缓存的二进制（如果支持），
/// 否则编译源码并存储到缓存。
///
/// # 参数
/// - `device`: WGPU设备
/// - `cache`: 着色器缓存管理器
/// - `label`: 着色器标签（用于调试）
/// - `source`: WGSL源码
/// - `compile_options`: 编译选项（用于区分不同编译配置）
///
/// # 返回
/// WGPU着色器模块
pub fn create_cached_shader_module(
    device: &wgpu::Device,
    cache: &mut ShaderCache,
    label: Option<&str>,
    source: &str,
    compile_options: &str,
) -> Result<wgpu::ShaderModule, RenderError> {
    // 生成缓存键
    let key = ShaderCacheKey::from_source(source, compile_options);

    // 尝试从缓存获取
    // 注意：当前wgpu不支持直接加载SPIR-V二进制（需要naga集成）
    // 这里先检查缓存是否存在，如果存在则跳过编译（未来优化）
    let _cached = cache.get(&key)?;

    // 当前实现：总是编译（因为wgpu需要WGSL源码）
    // 未来优化：如果缓存命中且wgpu支持SPIR-V，可以直接加载
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label,
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });

    // 存储源码到缓存（用于验证，未来可以存储SPIR-V）
    if let Err(e) = cache.put_source(&key, source) {
        tracing::warn!(
            target: "render",
            "Failed to cache shader: {}",
            e
        );
    }

    Ok(shader)
}

/// 创建着色器模块（简化版本，使用默认编译选项）
pub fn create_shader_module_cached(
    device: &wgpu::Device,
    cache: &mut ShaderCache,
    label: Option<&str>,
    source: &str,
) -> Result<wgpu::ShaderModule, RenderError> {
    create_cached_shader_module(device, cache, label, source, "")
}

/// 异步创建着色器模块（使用异步编译器）
///
/// 在后台线程编译着色器，不阻塞主线程
pub async fn create_shader_module_async(
    device: &wgpu::Device,
    compiler: &AsyncShaderCompiler,
    label: Option<&str>,
    source: &str,
    priority: ShaderCompilePriority,
) -> Result<wgpu::ShaderModule, RenderError> {
    // 提交异步编译请求
    let rx = compiler
        .compile_async(label, source, "", priority)
        .map_err(|e| {
            RenderError::InvalidState(format!("Failed to submit compile request: {}", e))
        })?;

    // 等待编译完成
    let compiled = wait_for_compile(rx)
        .await
        .map_err(|e| RenderError::ShaderCompilation(e.to_string()))?;

    // 在主线程创建着色器模块（wgpu要求在主线程）
    // 注意：实际的wgpu编译仍然需要在主线程进行
    // 这里只是预处理和缓存
    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: label,
        source: wgpu::ShaderSource::Wgsl(compiled.source.into()),
    }))
}

/// 批量异步编译着色器
pub async fn compile_shaders_async(
    compiler: &AsyncShaderCompiler,
    requests: Vec<(Option<&str>, &str, ShaderCompilePriority)>,
) -> Result<Vec<Result<wgpu::ShaderModule, RenderError>>, RenderError> {
    // 提交所有编译请求
    let mut receivers = Vec::new();
    for (label, source, priority) in requests {
        let rx = compiler
            .compile_async(label, source, "", priority)
            .map_err(|e| {
                RenderError::InvalidState(format!("Failed to submit compile request: {}", e))
            })?;
        receivers.push((label, source, rx));
    }

    // 等待所有编译完成
    let mut results = Vec::new();
    for (label, source, rx) in receivers {
        match wait_for_compile(rx).await {
            Ok(compiled) => {
                // 注意：实际的wgpu::ShaderModule创建需要在主线程
                // 这里返回错误提示需要在主线程创建
                results.push(Err(RenderError::InvalidState(
                    "Shader module creation must be done on main thread".to_string(),
                )));
            }
            Err(e) => {
                results.push(Err(RenderError::ShaderCompilation(e.to_string())));
            }
        }
    }

    Ok(results)
}
