use glam::{Mat4, Vec3, Vec4};

/// 阴影质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowQuality {
    /// 无阴影
    None,
    /// 低质量 - 硬阴影
    Low,
    /// 中等质量 - 3x3 PCF
    #[default]
    Medium,
    /// 高质量 - 5x5 PCF
    High,
    /// 极致质量 - 16 采样 Poisson Disk
    Ultra,
}

impl ShadowQuality {
    /// 获取 PCF 采样数
    pub fn sample_count(&self) -> u32 {
        match self {
            ShadowQuality::None => 0,
            ShadowQuality::Low => 1,
            ShadowQuality::Medium => 9,    // 3x3
            ShadowQuality::High => 25,     // 5x5
            ShadowQuality::Ultra => 16,    // Poisson disk
        }
    }
    
    /// 获取 PCF 半径
    pub fn pcf_radius(&self) -> f32 {
        match self {
            ShadowQuality::None => 0.0,
            ShadowQuality::Low => 0.0,
            ShadowQuality::Medium => 1.0,
            ShadowQuality::High => 1.5,
            ShadowQuality::Ultra => 2.5,
        }
    }
}

/// 级联阴影贴图配置
pub struct CsmConfig {
    /// 级联数量 (通常为3-4)
    pub cascade_count: u32,
    /// 阴影贴图分辨率
    pub shadow_map_size: u32,
    /// 级联分割距离 (相对于相机的远平面)
    pub cascade_splits: Vec<f32>,
    /// 阴影质量
    pub shadow_quality: ShadowQuality,
    /// 阴影偏移 (减少 shadow acne)
    pub shadow_bias: f32,
    /// 法线偏移 (减少 peter panning)
    pub normal_bias: f32,
    /// PCF 半径
    pub pcf_radius: f32,
    /// 级联混合范围 (级联过渡平滑度)
    pub cascade_blend_distance: f32,
    /// 是否启用级联混合
    pub enable_cascade_blending: bool,
    /// 对数分割因子 (0.0 = 均匀, 1.0 = 完全对数)
    pub split_lambda: f32,
}

impl Default for CsmConfig {
    fn default() -> Self {
        Self {
            cascade_count: 4,
            shadow_map_size: 2048,
            cascade_splits: vec![0.05, 0.15, 0.4, 1.0],
            shadow_quality: ShadowQuality::Medium,
            shadow_bias: 0.005,
            normal_bias: 0.02,
            pcf_radius: 1.0,
            cascade_blend_distance: 0.1,
            enable_cascade_blending: true,
            split_lambda: 0.75,
        }
    }
}

impl CsmConfig {
    /// 创建低质量配置 (适合移动端)
    pub fn low_quality() -> Self {
        Self {
            cascade_count: 2,
            shadow_map_size: 1024,
            cascade_splits: vec![0.2, 1.0],
            shadow_quality: ShadowQuality::Low,
            shadow_bias: 0.008,
            normal_bias: 0.03,
            pcf_radius: 0.0,
            cascade_blend_distance: 0.0,
            enable_cascade_blending: false,
            split_lambda: 0.5,
        }
    }
    
    /// 创建高质量配置 (适合高端PC)
    pub fn high_quality() -> Self {
        Self {
            cascade_count: 4,
            shadow_map_size: 4096,
            cascade_splits: vec![0.03, 0.1, 0.3, 1.0],
            shadow_quality: ShadowQuality::High,
            shadow_bias: 0.003,
            normal_bias: 0.015,
            pcf_radius: 1.5,
            cascade_blend_distance: 0.15,
            enable_cascade_blending: true,
            split_lambda: 0.8,
        }
    }
    
    /// 创建极致质量配置 (用于截图/过场动画)
    pub fn ultra_quality() -> Self {
        Self {
            cascade_count: 4,
            shadow_map_size: 8192,
            cascade_splits: vec![0.02, 0.08, 0.25, 1.0],
            shadow_quality: ShadowQuality::Ultra,
            shadow_bias: 0.002,
            normal_bias: 0.01,
            pcf_radius: 2.5,
            cascade_blend_distance: 0.2,
            enable_cascade_blending: true,
            split_lambda: 0.85,
        }
    }
    
    /// 计算最优级联分割点 (使用对数-均匀混合)
    pub fn calculate_optimal_splits(&self, near: f32, far: f32) -> Vec<f32> {
        let mut splits = Vec::with_capacity(self.cascade_count as usize);
        
        for i in 1..=self.cascade_count {
            let p = i as f32 / self.cascade_count as f32;
            
            // 对数分割 - 近处更多细节
            let log_split = near * (far / near).powf(p);
            
            // 均匀分割
            let uniform_split = near + (far - near) * p;
            
            // 混合
            let split = log_split * self.split_lambda + uniform_split * (1.0 - self.split_lambda);
            
            // 归一化到 [0, 1]
            splits.push((split - near) / (far - near));
        }
        
        splits
    }
}

/// CSM Uniform 数据 (传递到着色器)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CsmUniforms {
    /// 级联视图投影矩阵 (最多4个)
    pub cascade_view_proj: [[f32; 16]; 4],
    /// 级联分割距离
    pub cascade_splits: [f32; 4],
    /// 光源方向
    pub light_direction: [f32; 3],
    /// 级联数量
    pub cascade_count: u32,
    /// 阴影偏移
    pub shadow_bias: f32,
    /// 法线偏移
    pub normal_bias: f32,
    /// PCF 半径
    pub pcf_radius: f32,
    /// 级联混合距离
    pub cascade_blend_distance: f32,
    /// 是否启用级联混合 (0 或 1)
    pub enable_blending: u32,
    /// 阴影质量等级 (0-4)
    pub quality_level: u32,
    /// 填充
    pub _pad: [u32; 2],
}

impl CsmUniforms {
    /// 从配置创建 Uniform 数据
    pub fn from_config(config: &CsmConfig, light_direction: Vec3) -> Self {
        let quality_level = match config.shadow_quality {
            ShadowQuality::None => 0,
            ShadowQuality::Low => 1,
            ShadowQuality::Medium => 2,
            ShadowQuality::High => 3,
            ShadowQuality::Ultra => 4,
        };
        
        let mut cascade_splits = [0.0f32; 4];
        for (i, &split) in config.cascade_splits.iter().enumerate().take(4) {
            cascade_splits[i] = split;
        }
        
        Self {
            cascade_view_proj: [[0.0; 16]; 4],
            cascade_splits,
            light_direction: [light_direction.x, light_direction.y, light_direction.z],
            cascade_count: config.cascade_count,
            shadow_bias: config.shadow_bias,
            normal_bias: config.normal_bias,
            pcf_radius: config.pcf_radius,
            cascade_blend_distance: config.cascade_blend_distance,
            enable_blending: if config.enable_cascade_blending { 1 } else { 0 },
            quality_level,
            _pad: [0; 2],
        }
    }
    
    /// 更新级联视图投影矩阵
    pub fn update_cascade_matrices(&mut self, matrices: &[Mat4]) {
        for (i, mat) in matrices.iter().enumerate().take(4) {
            self.cascade_view_proj[i] = mat.to_cols_array();
        }
    }
}

/// 级联阴影贴图
pub struct CascadedShadowMap {
    pub config: CsmConfig,
    /// 阴影贴图纹理数组
    pub shadow_maps: Vec<wgpu::Texture>,
    pub shadow_views: Vec<wgpu::TextureView>,
    /// 每个级联的光源视图投影矩阵
    pub light_view_proj_matrices: Vec<Mat4>,
    /// 级联分割距离 (在视图空间中)
    pub cascade_distances: Vec<f32>,
    /// 阴影贴图绑定组
    pub bind_group: wgpu::BindGroup,
}

impl CascadedShadowMap {
    pub fn new(device: &wgpu::Device, config: CsmConfig, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let mut shadow_maps = Vec::new();
        let mut shadow_views = Vec::new();
        
        // 创建级联阴影贴图
        for i in 0..config.cascade_count {
            let shadow_map = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("CSM Shadow Map {}", i)),
                size: wgpu::Extent3d {
                    width: config.shadow_map_size,
                    height: config.shadow_map_size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            
            let shadow_view = shadow_map.create_view(&wgpu::TextureViewDescriptor::default());
            
            shadow_maps.push(shadow_map);
            shadow_views.push(shadow_view);
        }
        
        // 创建采样器
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("CSM Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        
        // 创建绑定组 (最多支持4个级联)
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&shadow_sampler),
            },
        ];
        
        for (i, view) in shadow_views.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: (i + 1) as u32,
                resource: wgpu::BindingResource::TextureView(view),
            });
        }
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CSM Bind Group"),
            layout: bind_group_layout,
            entries: &entries,
        });
        
        Self {
            config,
            shadow_maps,
            shadow_views,
            light_view_proj_matrices: vec![Mat4::IDENTITY; 4],
            cascade_distances: vec![0.0; 4],
            bind_group,
        }
    }
    
    /// 更新级联阴影贴图的视图投影矩阵
    pub fn update_cascades(
        &mut self,
        camera_view: Mat4,
        camera_proj: Mat4,
        light_direction: Vec3,
        near: f32,
        far: f32,
    ) {
        let inv_camera_view_proj = (camera_proj * camera_view).inverse();
        
        let mut last_split_dist = 0.0;
        
        for i in 0..self.config.cascade_count as usize {
            let split_dist = self.config.cascade_splits[i];
            
            // 计算当前级联的近平面和远平面
            let cascade_near = near + last_split_dist * (far - near);
            let cascade_far = near + split_dist * (far - near);
            
            self.cascade_distances[i] = cascade_far;
            
            // 计算视锥体的8个角点 (在NDC空间中)
            let frustum_corners = [
                Vec4::new(-1.0, -1.0, -1.0, 1.0),
                Vec4::new(1.0, -1.0, -1.0, 1.0),
                Vec4::new(1.0, 1.0, -1.0, 1.0),
                Vec4::new(-1.0, 1.0, -1.0, 1.0),
                Vec4::new(-1.0, -1.0, 1.0, 1.0),
                Vec4::new(1.0, -1.0, 1.0, 1.0),
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(-1.0, 1.0, 1.0, 1.0),
            ];
            
            // 将角点转换到世界空间
            let mut world_corners = Vec::new();
            for corner in &frustum_corners {
                let world_corner = inv_camera_view_proj * *corner;
                let w = world_corner.w;
                world_corners.push(Vec3::new(world_corner.x / w, world_corner.y / w, world_corner.z / w));
            }
            
            // 计算视锥体中心
            let mut center = Vec3::ZERO;
            for corner in &world_corners {
                center += *corner;
            }
            center /= 8.0;
            
            // 计算光源视图矩阵
            let light_view = Mat4::look_at_rh(
                center - light_direction * 50.0,
                center,
                Vec3::Y,
            );
            
            // 计算包围盒
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;
            
            for corner in &world_corners {
                let light_space_corner = light_view.transform_point3(*corner);
                min_x = min_x.min(light_space_corner.x);
                max_x = max_x.max(light_space_corner.x);
                min_y = min_y.min(light_space_corner.y);
                max_y = max_y.max(light_space_corner.y);
                min_z = min_z.min(light_space_corner.z);
                max_z = max_z.max(light_space_corner.z);
            }
            
            // 扩展Z范围以包含场景中的阴影投射物
            let z_mult = 10.0;
            if min_z < 0.0 {
                min_z *= z_mult;
            } else {
                min_z /= z_mult;
            }
            if max_z < 0.0 {
                max_z /= z_mult;
            } else {
                max_z *= z_mult;
            }
            
            // 计算光源正交投影矩阵
            let light_proj = Mat4::orthographic_rh(
                min_x, max_x,
                min_y, max_y,
                min_z, max_z,
            );
            
            self.light_view_proj_matrices[i] = light_proj * light_view;
            
            last_split_dist = split_dist;
        }
    }
}

/// CSM渲染器
pub struct CsmRenderer {
    pub csm: CascadedShadowMap,
    pub shadow_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl CsmRenderer {
    pub fn new(device: &wgpu::Device, config: CsmConfig) -> Self {
        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("CSM BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        
        // 创建CSM
        let csm = CascadedShadowMap::new(device, config, &bind_group_layout);
        
        // 创建阴影渲染管线
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("CSM Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_csm_shadow.wgsl").into()),
        });
        
        let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("CSM Shadow Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("CSM Shadow Pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<crate::render::mesh::Vertex3D>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None, // 深度渲染不需要片段着色器
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
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        Self {
            csm,
            shadow_pipeline,
            bind_group_layout,
        }
    }
}
