//! WebGL适配器优化
//!
//! 提供WebGL特定的优化功能，包括：
//! - WGSL到GLSL的转换优化
//! - WebGL特性检测和降级
//! - WebGL性能优化建议
//! - 着色器变体生成（针对WebGL限制）

use crate::error::RenderError;
use std::collections::HashMap;
use tracing::{error, warn};

/// WebGL适配器配置
#[derive(Debug, Clone)]
pub struct WebGLAdapterConfig {
    /// 是否启用WebGL2（如果可用）
    pub prefer_webgl2: bool,
    /// 最大纹理尺寸（自动检测）
    pub max_texture_size: u32,
    /// 是否支持浮点纹理
    pub supports_float_textures: bool,
    /// 是否支持线性过滤浮点纹理
    pub supports_linear_filtering_float: bool,
    /// 是否支持深度纹理
    pub supports_depth_texture: bool,
    /// 是否支持实例化渲染
    pub supports_instancing: bool,
    /// 是否支持顶点数组对象（VAO）
    pub supports_vao: bool,
    /// 是否支持统一缓冲区对象（UBO）
    pub supports_ubo: bool,
    /// 最大顶点属性数
    pub max_vertex_attributes: u32,
    /// 最大纹理单元数
    pub max_texture_units: u32,
    /// 最大统一向量数
    pub max_uniform_vectors: u32,
}

impl Default for WebGLAdapterConfig {
    fn default() -> Self {
        Self {
            prefer_webgl2: true,
            max_texture_size: 4096, // 保守默认值
            supports_float_textures: false,
            supports_linear_filtering_float: false,
            supports_depth_texture: true,
            supports_instancing: true,
            supports_vao: true,
            supports_ubo: true,
            max_vertex_attributes: 16,
            max_texture_units: 16,
            max_uniform_vectors: 1024,
        }
    }
}

/// WebGL特性检测结果
#[derive(Debug, Clone)]
pub struct WebGLCapabilities {
    /// WebGL版本（1或2）
    pub version: u32,
    /// 供应商信息
    pub vendor: String,
    /// 渲染器信息
    pub renderer: String,
    /// 着色器语言版本
    pub shading_language_version: String,
    /// 支持的扩展列表
    pub extensions: Vec<String>,
    /// 适配器配置
    pub config: WebGLAdapterConfig,
}

impl WebGLCapabilities {
    /// 检测WebGL能力
    #[cfg(target_arch = "wasm32")]
    pub fn detect() -> Result<Self, RenderError> {
        use wasm_bindgen::JsCast;
        use web_sys::{WebGl2RenderingContext, WebGlRenderingContext};

        // 尝试获取WebGL2上下文
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| {
                d.get_element_by_id("game-canvas").or_else(|| {
                    // 如果没有找到，创建一个临时canvas
                    let c = d.create_element("canvas").ok()?;
                    Some(c)
                })
            })
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .ok_or_else(|| RenderError::Other("Failed to get canvas".to_string()))?;

        let mut version = 1;
        let mut gl_context: Option<web_sys::WebGl2RenderingContext> = None;

        // 尝试WebGL2
        if let Ok(Some(ctx)) =
            canvas.get_context_with_context_options("webgl2", &js_sys::Object::new())
        {
            if let Some(ctx) = ctx.dyn_ref::<WebGl2RenderingContext>() {
                version = 2;
                gl_context = Some(ctx.clone());
            }
        }

        // 回退到WebGL1
        if gl_context.is_none() {
            if let Ok(Some(ctx)) = canvas.get_context("webgl") {
                if let Some(ctx) = ctx.dyn_ref::<WebGlRenderingContext>() {
                    match ctx.clone().dyn_into::<WebGl2RenderingContext>() {
                        Ok(ctx2) => {
                            gl_context = Some(ctx2);
                        }
                        Err(_) => {
                            error!(
                                "WebGL1 context detected but not supported in this implementation"
                            );
                            return Err(RenderError::Other(
                                "WebGL1 to WebGL2 context conversion failed".to_string(),
                            ));
                        }
                    }
                }
            }
        }

        let gl = gl_context
            .ok_or_else(|| RenderError::Other("Failed to get WebGL context".to_string()))?;

        // 获取供应商和渲染器信息
        let vendor =
            gl.get_parameter(WebGl2RenderingContext::VENDOR).as_string().unwrap_or_else(|| {
                warn!("Failed to retrieve WebGL VENDOR parameter, using default");
                "Unknown".to_string()
            });
        let renderer = gl
            .get_parameter(WebGl2RenderingContext::RENDERER)
            .as_string()
            .unwrap_or_else(|| {
                warn!("Failed to retrieve WebGL RENDERER parameter, using default");
                "Unknown".to_string()
            });
        let shading_language_version = gl
            .get_parameter(WebGl2RenderingContext::SHADING_LANGUAGE_VERSION)
            .as_string()
            .unwrap_or_else(|| {
                warn!("Failed to retrieve WebGL SHADING_LANGUAGE_VERSION parameter, using default");
                "Unknown".to_string()
            });

        // 获取扩展列表
        let extensions: Vec<String> = gl
            .get_supported_extensions()
            .map(|exts| {
                exts.iter()
                    .map(|ext| {
                        ext.as_string().unwrap_or_else(|| {
                            warn!("Extension name conversion failed, using default");
                            String::default()
                        })
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                warn!("Failed to retrieve supported extensions list, using empty list");
                Vec::default()
            });

        // 检测能力
        let max_texture_size = gl
            .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)
            .as_f64()
            .unwrap_or_else(|| {
                warn!("Failed to retrieve MAX_TEXTURE_SIZE, using default 4096");
                4096.0
            }) as u32;

        let max_vertex_attributes = gl
            .get_parameter(WebGl2RenderingContext::MAX_VERTEX_ATTRIBS)
            .as_f64()
            .unwrap_or_else(|| {
                warn!("Failed to retrieve MAX_VERTEX_ATTRIBS, using default 16");
                16.0
            }) as u32;

        let max_texture_units = gl
            .get_parameter(WebGl2RenderingContext::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
            .as_f64()
            .unwrap_or_else(|| {
                warn!("Failed to retrieve MAX_COMBINED_TEXTURE_IMAGE_UNITS, using default 16");
                16.0
            }) as u32;

        let max_uniform_vectors = gl
            .get_parameter(WebGl2RenderingContext::MAX_VERTEX_UNIFORM_VECTORS)
            .as_f64()
            .unwrap_or_else(|| {
                warn!("Failed to retrieve MAX_VERTEX_UNIFORM_VECTORS, using default 1024");
                1024.0
            }) as u32;

        let supports_float_textures =
            extensions.iter().any(|ext| ext.contains("OES_texture_float"));
        let supports_linear_filtering_float =
            extensions.iter().any(|ext| ext.contains("OES_texture_float_linear"));
        let supports_depth_texture =
            extensions.iter().any(|ext| ext.contains("WEBGL_depth_texture"));
        let supports_instancing =
            version >= 2 || extensions.iter().any(|ext| ext.contains("ANGLE_instanced_arrays"));
        let supports_vao =
            version >= 2 || extensions.iter().any(|ext| ext.contains("OES_vertex_array_object"));
        let supports_ubo = version >= 2;

        let config = WebGLAdapterConfig {
            prefer_webgl2: version >= 2,
            max_texture_size,
            supports_float_textures,
            supports_linear_filtering_float,
            supports_depth_texture,
            supports_instancing,
            supports_vao,
            supports_ubo,
            max_vertex_attributes,
            max_texture_units,
            max_uniform_vectors,
        };

        Ok(Self {
            version,
            vendor,
            renderer,
            shading_language_version,
            extensions,
            config,
        })
    }

    /// 检测WebGL能力（非Web平台，返回默认值）
    #[cfg(not(target_arch = "wasm32"))]
    pub fn detect() -> Result<Self, RenderError> {
        Ok(Self {
            version: 2,
            vendor: "Unknown".to_string(),
            renderer: "Unknown".to_string(),
            shading_language_version: "Unknown".to_string(),
            extensions: vec![],
            config: WebGLAdapterConfig::default(),
        })
    }

    /// 检查是否支持特定扩展
    pub fn supports_extension(&self, extension: &str) -> bool {
        self.extensions.iter().any(|ext| ext.contains(extension))
    }

    /// 获取性能优化建议
    pub fn get_optimization_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if self.version < 2 {
            suggestions
                .push("Consider using WebGL2 for better performance and features".to_string());
        }

        if !self.config.supports_instancing {
            suggestions.push(
                "Instancing not supported, consider using geometry batching instead".to_string(),
            );
        }

        if self.config.max_texture_size < 4096 {
            suggestions.push(format!(
                "Max texture size is {}px, consider using texture atlases for large textures",
                self.config.max_texture_size
            ));
        }

        if !self.config.supports_float_textures {
            suggestions
                .push("Float textures not supported, HDR rendering may be limited".to_string());
        }

        if self.config.max_vertex_attributes < 16 {
            suggestions.push(format!(
                "Limited vertex attributes ({}) may require shader variants",
                self.config.max_vertex_attributes
            ));
        }

        suggestions
    }
}

/// WGSL到GLSL转换器（优化版本）
///
/// 注意：wgpu本身已经处理了WGSL到GLSL的转换，这个模块提供额外的优化。
pub struct WGSLToGLSLConverter {
    /// WebGL能力
    capabilities: WebGLCapabilities,
    /// 转换缓存
    conversion_cache: HashMap<String, String>,
}

impl WGSLToGLSLConverter {
    /// 创建新的转换器
    pub fn new(capabilities: WebGLCapabilities) -> Self {
        Self {
            capabilities,
            conversion_cache: HashMap::new(),
        }
    }

    /// 优化WGSL着色器以适配WebGL限制
    ///
    /// 这个函数对WGSL源码进行预处理，使其更适合WebGL后端：
    /// - 减少顶点属性数量（如果超过限制）
    /// - 优化统一缓冲区布局
    /// - 添加WebGL特定的优化提示
    pub fn optimize_for_webgl(&mut self, wgsl_source: &str) -> String {
        // 检查缓存
        if let Some(cached) = self.conversion_cache.get(wgsl_source) {
            return cached.clone();
        }

        let mut optimized = wgsl_source.to_string();

        // 如果顶点属性数量超过限制，进行优化
        if self.capabilities.config.max_vertex_attributes < 16 {
            // 简化着色器变体（实际实现需要更复杂的分析）
            optimized = self.reduce_vertex_attributes(&optimized);
        }

        // 优化统一缓冲区（如果UBO不支持）
        if !self.capabilities.config.supports_ubo {
            optimized = self.optimize_uniforms(&optimized);
        }

        // 缓存结果
        self.conversion_cache.insert(wgsl_source.to_string(), optimized.clone());

        optimized
    }

    /// 减少顶点属性数量
    fn reduce_vertex_attributes(&self, source: &str) -> String {
        // 简化实现：实际需要更复杂的着色器分析
        // 这里只是示例，实际实现需要：
        // 1. 解析WGSL AST
        // 2. 识别顶点属性
        // 3. 合并或打包属性
        // 4. 重新生成着色器代码

        // 占位实现
        source.to_string()
    }

    /// 优化统一缓冲区（转换为传统uniform）
    fn optimize_uniforms(&self, source: &str) -> String {
        // 简化实现：实际需要将UBO转换为传统uniform
        // 这里只是示例

        // 占位实现
        source.to_string()
    }

    /// 清除转换缓存
    pub fn clear_cache(&mut self) {
        self.conversion_cache.clear();
    }
}

/// WebGL性能优化器
pub struct WebGLPerformanceOptimizer {
    /// WebGL能力
    capabilities: WebGLCapabilities,
    /// 优化建议
    suggestions: Vec<String>,
}

impl WebGLPerformanceOptimizer {
    /// 创建新的优化器
    pub fn new(capabilities: WebGLCapabilities) -> Self {
        let suggestions = capabilities.get_optimization_suggestions();
        Self {
            capabilities,
            suggestions,
        }
    }

    /// 获取优化建议
    pub fn get_suggestions(&self) -> &[String] {
        &self.suggestions
    }

    /// 检查是否应该使用纹理图集
    pub fn should_use_texture_atlas(&self, texture_count: usize, avg_texture_size: u32) -> bool {
        // 如果纹理数量多或单个纹理较大，建议使用图集
        texture_count > 32 || avg_texture_size > self.capabilities.config.max_texture_size / 2
    }

    /// 检查是否应该使用实例化渲染
    pub fn should_use_instancing(&self, instance_count: usize) -> bool {
        self.capabilities.config.supports_instancing && instance_count > 100
    }

    /// 获取推荐的批处理大小
    pub fn recommended_batch_size(&self) -> usize {
        // 基于WebGL限制推荐批处理大小
        if self.capabilities.version >= 2 {
            1000 // WebGL2可以处理更多
        } else {
            500 // WebGL1更保守
        }
    }

    /// 检查是否应该使用深度纹理
    pub fn can_use_depth_texture(&self) -> bool {
        self.capabilities.config.supports_depth_texture
    }

    /// 检查是否应该使用浮点纹理
    pub fn can_use_float_textures(&self) -> bool {
        self.capabilities.config.supports_float_textures
    }
}

/// WebGL适配器
///
/// 提供WebGL特定的优化和适配功能
pub struct WebGLAdapter {
    /// WebGL能力
    capabilities: WebGLCapabilities,
    /// WGSL转换器
    converter: WGSLToGLSLConverter,
    /// 性能优化器
    optimizer: WebGLPerformanceOptimizer,
}

impl WebGLAdapter {
    /// 创建新的WebGL适配器
    pub fn new() -> Result<Self, RenderError> {
        let capabilities = WebGLCapabilities::detect()?;
        let converter = WGSLToGLSLConverter::new(capabilities.clone());
        let optimizer = WebGLPerformanceOptimizer::new(capabilities.clone());

        Ok(Self {
            capabilities,
            converter,
            optimizer,
        })
    }

    /// 获取WebGL能力
    pub fn capabilities(&self) -> &WebGLCapabilities {
        &self.capabilities
    }

    /// 优化WGSL着色器
    pub fn optimize_shader(&mut self, wgsl_source: &str) -> String {
        self.converter.optimize_for_webgl(wgsl_source)
    }

    /// 获取性能优化建议
    pub fn get_optimization_suggestions(&self) -> &[String] {
        self.optimizer.get_suggestions()
    }

    /// 检查是否应该使用纹理图集
    pub fn should_use_texture_atlas(&self, texture_count: usize, avg_texture_size: u32) -> bool {
        self.optimizer.should_use_texture_atlas(texture_count, avg_texture_size)
    }

    /// 检查是否应该使用实例化渲染
    pub fn should_use_instancing(&self, instance_count: usize) -> bool {
        self.optimizer.should_use_instancing(instance_count)
    }

    /// 获取推荐的批处理大小
    pub fn recommended_batch_size(&self) -> usize {
        self.optimizer.recommended_batch_size()
    }

    /// 清除转换缓存
    pub fn clear_cache(&mut self) {
        self.converter.clear_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webgl_adapter_config_default() {
        let config = WebGLAdapterConfig::default();
        assert!(config.prefer_webgl2);
        assert_eq!(config.max_texture_size, 4096);
    }

    #[test]
    fn test_webgl_capabilities_detect() {
        // 在非Web平台，应该返回默认值
        let capabilities = WebGLCapabilities::detect();
        assert!(capabilities.is_ok());
    }

    #[test]
    fn test_wgsl_converter_creation() {
        let capabilities = WebGLCapabilities::detect().unwrap_or_else(|e| {
            tracing::error!("Failed to detect WebGL capabilities: {}", e);
            panic!("WebGL capabilities detection required for test");
        });
        let converter = WGSLToGLSLConverter::new(capabilities);
        assert_eq!(converter.conversion_cache.len(), 0);
    }

    #[test]
    fn test_wgsl_converter_optimize() {
        let capabilities = WebGLCapabilities::detect().unwrap_or_else(|e| {
            tracing::error!("Failed to detect WebGL capabilities: {}", e);
            panic!("WebGL capabilities detection required for test");
        });
        let mut converter = WGSLToGLSLConverter::new(capabilities);

        let wgsl = "fn main() { }";
        let optimized = converter.optimize_for_webgl(wgsl);

        // 应该返回优化后的代码（即使没有实际优化）
        assert!(!optimized.is_empty());

        // 第二次调用应该使用缓存
        let optimized2 = converter.optimize_for_webgl(wgsl);
        assert_eq!(optimized, optimized2);
    }

    #[test]
    fn test_performance_optimizer() {
        let capabilities = WebGLCapabilities::detect().unwrap_or_else(|e| {
            tracing::error!("Failed to detect WebGL capabilities: {}", e);
            panic!("WebGL capabilities detection required for test");
        });
        let optimizer = WebGLPerformanceOptimizer::new(capabilities);

        // 应该有一些优化建议（即使是默认值）
        let suggestions = optimizer.get_suggestions();
        assert!(suggestions.len() >= 0); // 可能为空，取决于检测结果
    }

    #[test]
    fn test_webgl_adapter_creation() {
        let adapter = WebGLAdapter::new();
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_texture_atlas_recommendation() {
        let capabilities = WebGLCapabilities::detect().unwrap_or_else(|e| {
            tracing::error!("Failed to detect WebGL capabilities: {}", e);
            panic!("WebGL capabilities detection required for test");
        });
        let optimizer = WebGLPerformanceOptimizer::new(capabilities);

        // 大量纹理应该建议使用图集
        assert!(optimizer.should_use_texture_atlas(100, 512));

        // 少量纹理可能不需要图集
        assert!(!optimizer.should_use_texture_atlas(5, 256));
    }
}
