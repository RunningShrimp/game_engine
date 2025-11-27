//! WGPU 纹理管理
//!
//! 包含纹理加载、创建和管理功能。

/// 纹理管理器
/// 
/// 负责纹理的创建、加载和生命周期管理。
pub struct TextureManager {
    /// 纹理绑定组布局
    texture_bgl: wgpu::BindGroupLayout,
    /// 纹理绑定组列表
    texture_bind_groups: Vec<wgpu::BindGroup>,
    /// 纹理尺寸列表
    textures_size: Vec<[u32; 2]>,
}

impl TextureManager {
    /// 创建纹理管理器
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, texture_bgl: wgpu::BindGroupLayout) -> Self {
        // 创建默认棋盘纹理
        let (default_bg, default_size) = Self::create_checkerboard_texture(device, queue, &texture_bgl);
        
        Self {
            texture_bgl,
            texture_bind_groups: vec![default_bg],
            textures_size: vec![default_size],
        }
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
                let c = if ((x / 32) % 2) ^ ((y / 32) % 2) == 0 { 220 } else { 60 };
                data[idx] = c;
                data[idx + 1] = c;
                data[idx + 2] = c;
                data[idx + 3] = 255;
            }
        }
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Checkerboard"),
            size: wgpu::Extent3d { width: tex_size, height: tex_size, depth_or_array_layers: 1 },
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
            wgpu::Extent3d { width: tex_size, height: tex_size, depth_or_array_layers: 1 },
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Default Texture BG"),
            layout: texture_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
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
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
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
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Texture BG: {:?}", path)),
            layout: &self.texture_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        
        let idx = self.texture_bind_groups.len() as u32;
        self.texture_bind_groups.push(bind_group);
        self.textures_size.push([w, h]);
        
        Some(idx)
    }
    
    /// 从内存加载纹理
    pub fn load_from_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        linear: bool,
    ) -> Option<u32> {
        let img = image::load_from_memory(bytes).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        
        let format = if linear {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture from bytes"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
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
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture BG from bytes"),
            layout: &self.texture_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        
        let idx = self.texture_bind_groups.len() as u32;
        self.texture_bind_groups.push(bind_group);
        self.textures_size.push([w, h]);
        
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
    pub fn new(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        Self { texture, view, width, height }
    }
    
    /// 调整大小
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) {
        if width == self.width && height == self.height {
            return;
        }
        
        *self = Self::new(device, width, height, format);
    }
}

