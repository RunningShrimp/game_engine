//! WGPU 纹理管理
//!
//! 包含纹理加载、创建和管理功能。
//! 支持压缩纹理格式（ASTC、BC等）的加载和解码。

use crate::render::texture_compression::{
    CompressedTextureFormat, CompressedTextureInfo, TextureFormatDetector,
};

/// 纹理管理器
///
/// 负责纹理的创建、加载和生命周期管理。
/// 支持压缩纹理格式（ASTC、BC等）的自动检测和加载。
pub struct TextureManager {
    /// 纹理绑定组布局
    texture_bgl: wgpu::BindGroupLayout,
    /// 纹理绑定组列表
    texture_bind_groups: Vec<wgpu::BindGroup>,
    /// 纹理尺寸列表
    textures_size: Vec<[u32; 2]>,
    /// 纹理压缩配置（可选）
    compression_config: Option<crate::config::graphics::TextureCompressionConfig>,
}

impl TextureManager {
    /// 创建纹理管理器
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bgl: wgpu::BindGroupLayout,
    ) -> Self {
        Self::with_compression_config(device, queue, texture_bgl, None)
    }

    /// 创建纹理管理器（带压缩配置）
    pub fn with_compression_config(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bgl: wgpu::BindGroupLayout,
        compression_config: Option<crate::config::graphics::TextureCompressionConfig>,
    ) -> Self {
        // 创建默认棋盘纹理
        let (default_bg, default_size) =
            Self::create_checkerboard_texture(device, queue, &texture_bgl);

        Self {
            texture_bgl,
            texture_bind_groups: vec![default_bg],
            textures_size: vec![default_size],
            compression_config,
        }
    }

    /// 设置压缩配置
    pub fn set_compression_config(
        &mut self,
        config: Option<crate::config::graphics::TextureCompressionConfig>,
    ) {
        self.compression_config = config;
    }

    /// 获取压缩配置
    pub fn compression_config(&self) -> Option<&crate::config::graphics::TextureCompressionConfig> {
        self.compression_config.as_ref()
    }

    /// 创建棋盘纹理
    fn create_checkerboard_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bgl: &wgpu::BindGroupLayout,
    ) -> (wgpu::BindGroup, [u32; 2]) {
        let tex_size = 256u32;
        let mut data = vec![0u8; (tex_size * tex_size * 4) as usize];

        for y in 0..tex_size {
            for x in 0..tex_size {
                let idx = ((y * tex_size + x) * 4) as usize;
                let c = if ((x / 32) % 2) ^ ((y / 32) % 2) == 0 {
                    220
                } else {
                    60
                };
                data[idx] = c;
                data[idx + 1] = c;
                data[idx + 2] = c;
                data[idx + 3] = 255;
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Checkerboard"),
            size: wgpu::Extent3d {
                width: tex_size,
                height: tex_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * tex_size),
                rows_per_image: Some(tex_size),
            },
            wgpu::Extent3d {
                width: tex_size,
                height: tex_size,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Default Texture BG"),
            layout: texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        (bind_group, [tex_size, tex_size])
    }

    /// 从文件加载纹理
    pub fn load_from_file(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &std::path::Path,
        srgb: bool,
    ) -> Option<u32> {
        let img = image::open(path).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        let format = if srgb {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Texture: {:?}", path)),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba.as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * w),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Texture BG: {:?}", path)),
            layout: &self.texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let idx = self.texture_bind_groups.len() as u32;
        self.texture_bind_groups.push(bind_group);
        self.textures_size.push([w, h]);

        Some(idx)
    }

    /// 从内存加载纹理（支持压缩格式）
    ///
    /// 自动检测压缩格式（ASTC、BC等），如果不支持则回退到未压缩格式
    /// 根据压缩配置决定是否启用压缩纹理加载
    pub fn load_from_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        linear: bool,
    ) -> Option<u32> {
        // 检查压缩配置是否启用
        let compression_enabled = self
            .compression_config
            .as_ref()
            .map(|config| config.enabled)
            .unwrap_or(true); // 默认启用

        if compression_enabled {
            // 首先尝试检测压缩格式
            if let Some(compressed_info) = TextureFormatDetector::detect_and_parse(bytes) {
                // 检查格式是否在首选列表中
                let format_allowed = self
                    .compression_config
                    .as_ref()
                    .map(|config| {
                        use crate::config::graphics::TextureCompressionFormat as ConfigFormat;
                        let detected_format: ConfigFormat = compressed_info.format.into();
                        config.preferred_formats.contains(&detected_format)
                    })
                    .unwrap_or(true); // 如果没有配置，允许所有格式

                if format_allowed {
                    // 尝试加载压缩纹理
                    if let Some(texture_idx) = self.load_compressed_texture(
                        device,
                        queue,
                        bytes,
                        compressed_info.format,
                        compressed_info.width,
                        compressed_info.height,
                        linear,
                    ) {
                        return Some(texture_idx);
                    }
                    // 如果压缩纹理加载失败，回退到未压缩格式
                    tracing::debug!(
                        target: "render",
                        "Compressed texture loading failed, falling back to uncompressed format. Format: {}",
                        compressed_info.format.name()
                    );
                } else {
                    tracing::debug!(
                        target: "render",
                        "Compressed texture format {} not in preferred list, using uncompressed format",
                        compressed_info.format.name()
                    );
                }
            }
        }

        // 加载未压缩纹理
        let img = image::load_from_memory(bytes).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        // 检查是否需要压缩（如果启用运行时压缩）
        if let Some(config) = &self.compression_config {
            if config.runtime_compression {
                let max_size = config.max_uncompressed_size;
                if w > max_size || h > max_size {
                    tracing::debug!(
                        target: "render",
                        "Texture {}x{} exceeds max uncompressed size {}, but runtime compression not yet implemented",
                        w, h, max_size
                    );
                    // 注意：运行时压缩功能待实现
                    // 未来计划：实现纹理运行时压缩，自动压缩超大纹理以节省内存
                }
            }
        }

        let format = if linear {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        self.create_texture_from_rgba(device, queue, rgba, format, w, h)
    }

    /// 加载压缩纹理
    ///
    /// 当前实现：所有压缩格式都需要CPU解码
    /// 未来可以优化为GPU原生支持（如果设备支持）
    fn load_compressed_texture(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _data: &[u8],
        format: CompressedTextureFormat,
        width: Option<u32>,
        height: Option<u32>,
        _linear: bool,
    ) -> Option<u32> {
        // 当前实现：所有压缩格式都需要CPU解码
        // 未来可以优化为GPU原生支持（如果设备支持）

        if !format.requires_cpu_decode() {
            // GPU原生支持（未来实现）
            return None;
        }

        // CPU解码
        // 注意：压缩纹理CPU解码功能需要外部库支持
        // 当前实现返回None，表示压缩纹理加载需要实际解码库支持
        //
        // 未来实现步骤：
        // 1. 添加astc-rs或类似库到Cargo.toml
        // 2. 实现ASTC文件头解析（获取宽度、高度、块大小）
        // 3. 调用解码库进行CPU解码
        // 4. 使用解码后的RGBA数据创建纹理
        // 相关任务：评估解码库选项，集成到项目中

        let _block_size = format.block_size();

        // 检查是否有尺寸信息
        let (width, height) = match (width, height) {
            (Some(w), Some(h)) => (w, h),
            _ => {
                tracing::warn!(
                    target: "render",
                    "Compressed texture dimensions not available, cannot decode. Format: {}",
                    format.name()
                );
                return None;
            }
        };

        match format {
            CompressedTextureFormat::Astc4x4
            | CompressedTextureFormat::Astc6x6
            | CompressedTextureFormat::Astc8x8 => {
                // ASTC解码需要知道纹理尺寸
                // 实际实现中需要从ASTC文件头解析尺寸
                // 注意：当前实现未解析ASTC文件头，需要外部库支持
                // 未来计划：实现ASTC文件头解析，调用AstcDecoder::decode进行解码
                let block_size = format.block_size();
                tracing::warn!(
                    target: "render",
                    "ASTC texture loading requires ASTC decoder library (not yet integrated). Format: {}, Size: {}x{}, Block: {:?}",
                    format.name(), width, height, block_size
                );
            }
            CompressedTextureFormat::BC1 => {
                // 注意：BC1解码功能需要外部库支持
                // 未来计划：实现BC1解码，调用BcDecoder::decode_bc1进行解码
                // BC格式通常包含在DDS文件中，尺寸已从DDS头解析
                tracing::warn!(
                    target: "render",
                    "BC1 texture loading requires BC decoder library (not yet integrated). Size: {}x{}",
                    width, height
                );
            }
            CompressedTextureFormat::BC3 => {
                // 注意：BC3解码功能需要外部库支持
                // 未来计划：实现BC3解码
                tracing::warn!(
                    target: "render",
                    "BC3 texture loading requires BC decoder library (not yet integrated). Size: {}x{}",
                    width, height
                );
            }
            CompressedTextureFormat::BC7 => {
                // 注意：BC7解码功能需要外部库支持
                // 未来计划：实现BC7解码
                tracing::warn!(
                    target: "render",
                    "BC7 texture loading requires BC decoder library (not yet integrated). Size: {}x{}",
                    width, height
                );
            }
            CompressedTextureFormat::ETC2 => {
                // 注意：ETC2解码功能需要外部库支持
                // 未来计划：实现ETC2解码
                tracing::warn!(
                    target: "render",
                    "ETC2 texture loading requires ETC2 decoder library (not yet integrated). Size: {}x{}",
                    width, height
                );
            }
        }

        // 当前返回None，表示需要实际解码库支持
        None
    }

    /// 从RGBA图像创建纹理（内部辅助方法）
    fn create_texture_from_rgba(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: image::RgbaImage,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Option<u32> {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture from RGBA"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba.as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture BG from RGBA"),
            layout: &self.texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let idx = self.texture_bind_groups.len() as u32;
        self.texture_bind_groups.push(bind_group);
        self.textures_size.push([width, height]);

        Some(idx)
    }

    /// 获取纹理绑定组
    pub fn get_bind_group(&self, index: usize) -> Option<&wgpu::BindGroup> {
        self.texture_bind_groups.get(index)
    }

    /// 获取纹理尺寸
    pub fn get_size(&self, index: usize) -> Option<[u32; 2]> {
        self.textures_size.get(index).copied()
    }

    /// 获取纹理数量
    pub fn count(&self) -> usize {
        self.texture_bind_groups.len()
    }

    /// 获取绑定组布局
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bgl
    }
}

/// 深度纹理辅助函数
pub fn create_depth_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// 离屏渲染目标
pub struct OffscreenTarget {
    /// 纹理
    pub texture: wgpu::Texture,
    /// 视图
    pub view: wgpu::TextureView,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
}

impl OffscreenTarget {
    /// 创建离屏渲染目标
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
        }
    }

    /// 调整大小
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        if width == self.width && height == self.height {
            return;
        }

        *self = Self::new(device, width, height, format);
    }
}
