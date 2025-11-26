use wgpu::util::DeviceExt;

/// G-Buffer纹理
pub struct GBuffer {
    /// 位置 + 深度 (RGB = 世界坐标, A = 深度)
    pub position_texture: wgpu::Texture,
    pub position_view: wgpu::TextureView,
    
    /// 法线 + 粗糙度 (RGB = 法线, A = 粗糙度)
    pub normal_texture: wgpu::Texture,
    pub normal_view: wgpu::TextureView,
    
    /// 反照率 + 金属度 (RGB = 反照率, A = 金属度)
    pub albedo_texture: wgpu::Texture,
    pub albedo_view: wgpu::TextureView,
    
    /// 深度缓冲
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    
    /// G-Buffer绑定组
    pub bind_group: wgpu::BindGroup,
}

impl GBuffer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        // 位置纹理 (RGBA32Float)
        let position_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G-Buffer Position"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let position_view = position_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // 法线纹理 (RGBA16Float)
        let normal_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G-Buffer Normal"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let normal_view = normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // 反照率纹理 (RGBA8UnormSrgb)
        let albedo_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G-Buffer Albedo"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let albedo_view = albedo_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // 深度纹理
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G-Buffer Depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("G-Buffer Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("G-Buffer Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&position_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        
        Self {
            position_texture,
            position_view,
            normal_texture,
            normal_view,
            albedo_texture,
            albedo_view,
            depth_texture,
            depth_view,
            bind_group,
        }
    }
    
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, bind_group_layout: &wgpu::BindGroupLayout) {
        *self = Self::new(device, width, height, bind_group_layout);
    }
}

/// 延迟渲染器
pub struct DeferredRenderer {
    pub gbuffer: GBuffer,
    pub geometry_pipeline: wgpu::RenderPipeline,
    pub lighting_pipeline: wgpu::RenderPipeline,
    pub gbuffer_bind_group_layout: wgpu::BindGroupLayout,
    pub fullscreen_vertex_buffer: wgpu::Buffer,
}

impl DeferredRenderer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, surface_format: wgpu::TextureFormat) -> Self {
        // 创建G-Buffer绑定组布局
        let gbuffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("G-Buffer BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });
        
        // 创建G-Buffer
        let gbuffer = GBuffer::new(device, width, height, &gbuffer_bind_group_layout);
        
        // 创建几何阶段着色器
        let geometry_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred Geometry Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_deferred_geometry.wgsl").into()),
        });
        
        // 创建光照阶段着色器
        let lighting_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred Lighting Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_deferred_lighting.wgsl").into()),
        });
        
        // 创建几何阶段管线 (写入G-Buffer)
        let geometry_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Deferred Geometry Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        let geometry_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Deferred Geometry Pipeline"),
            layout: Some(&geometry_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &geometry_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<crate::render::mesh::Vertex3D>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &geometry_shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float, // Position
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float, // Normal
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8UnormSrgb, // Albedo
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        // 创建光照阶段管线 (读取G-Buffer,输出到屏幕)
        let lighting_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Deferred Lighting Pipeline Layout"),
            bind_group_layouts: &[&gbuffer_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let lighting_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Deferred Lighting Pipeline"),
            layout: Some(&lighting_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &lighting_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &lighting_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        // 创建全屏四边形顶点缓冲
        let fullscreen_vertices: &[[f32; 2]] = &[
            [-1.0, -1.0],
            [1.0, -1.0],
            [1.0, 1.0],
            [-1.0, -1.0],
            [1.0, 1.0],
            [-1.0, 1.0],
        ];
        
        let fullscreen_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fullscreen Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(fullscreen_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        Self {
            gbuffer,
            geometry_pipeline,
            lighting_pipeline,
            gbuffer_bind_group_layout,
            fullscreen_vertex_buffer,
        }
    }
    
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.gbuffer.resize(device, width, height, &self.gbuffer_bind_group_layout);
    }
}
