//! XR 渲染适配器
//!
//! 实现立体渲染、异步时间扭曲（ATW）和注视点渲染

use super::*;
use std::sync::Arc;
use wgpu::*;

/// XR 渲染器
pub struct XrRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    /// ATW 计算着色器
    atw_compute_pipeline: Option<ComputePipeline>,
    /// 注视点渲染计算着色器
    foveated_compute_pipeline: Option<ComputePipeline>,
    /// ATW 参数缓冲区
    atw_params_buffer: Option<Buffer>,
    /// 注视点参数缓冲区
    foveated_params_buffer: Option<Buffer>,
    /// 是否启用 ATW
    atw_enabled: bool,
    /// 是否启用注视点渲染
    foveated_enabled: bool,
    /// 注视点配置
    foveated_config: foveated::FoveatedConfig,
}

impl XrRenderer {
    /// 创建新的 XR 渲染器
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self {
            device,
            queue,
            atw_compute_pipeline: None,
            foveated_compute_pipeline: None,
            atw_params_buffer: None,
            foveated_params_buffer: None,
            atw_enabled: true,
            foveated_enabled: false,
            foveated_config: foveated::FoveatedConfig::default(),
        }
    }

    /// 初始化渲染管线
    pub fn initialize(&mut self) -> Result<(), String> {
        // 初始化 ATW 计算着色器
        if self.atw_enabled {
            self.init_atw_pipeline()?;
        }

        // 初始化注视点渲染计算着色器
        if self.foveated_enabled {
            self.init_foveated_pipeline()?;
        }

        Ok(())
    }

    /// 初始化 ATW 管线
    fn init_atw_pipeline(&mut self) -> Result<(), String> {
        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("ATW Compute Shader"),
            source: ShaderSource::Wgsl(atw::ATW_SHADER.into()),
        });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("ATW Pipeline Layout"),
                bind_group_layouts: &[&self.create_atw_bind_group_layout()],
                push_constant_ranges: &[],
            });

        self.atw_compute_pipeline = Some(self.device.create_compute_pipeline(
            &ComputePipelineDescriptor {
                label: Some("ATW Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
        ));

        Ok(())
    }

    /// 创建 ATW 绑定组布局
    fn create_atw_bind_group_layout(&self) -> BindGroupLayout {
        self.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("ATW Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Depth,
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
    }

    /// 初始化注视点渲染管线
    fn init_foveated_pipeline(&mut self) -> Result<(), String> {
        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Foveated Compute Shader"),
            source: ShaderSource::Wgsl(foveated::FOVEATED_RECONSTRUCT_SHADER.into()),
        });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Foveated Pipeline Layout"),
                bind_group_layouts: &[&self.create_foveated_bind_group_layout()],
                push_constant_ranges: &[],
            });

        self.foveated_compute_pipeline = Some(self.device.create_compute_pipeline(
            &ComputePipelineDescriptor {
                label: Some("Foveated Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
        ));

        Ok(())
    }

    /// 创建注视点渲染绑定组布局
    fn create_foveated_bind_group_layout(&self) -> BindGroupLayout {
        self.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Foveated Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
    }

    /// 渲染立体视图
    pub fn render_stereo(
        &mut self,
        encoder: &mut CommandEncoder,
        views: &[XrView],
        render_targets: &[Arc<TextureView>],
        depth_targets: &[Arc<TextureView>],
        render_callback: impl Fn(&XrView, &TextureView, &TextureView),
    ) -> Result<(), XrError> {
        if views.len() != render_targets.len() || views.len() != depth_targets.len() {
            return Err(XrError::RuntimeFailure("View count mismatch".to_string()));
        }

        // 为每个视图渲染
        for (i, view) in views.iter().enumerate() {
            let render_target = render_targets
                .get(i)
                .ok_or_else(|| XrError::RuntimeFailure("Missing render target".to_string()))?;
            let depth_target = depth_targets
                .get(i)
                .ok_or_else(|| XrError::RuntimeFailure("Missing depth target".to_string()))?;

            // 调用渲染回调
            render_callback(view, render_target, depth_target);
        }

        Ok(())
    }

    /// 应用异步时间扭曲（ATW）
    pub fn apply_atw(
        &mut self,
        encoder: &mut CommandEncoder,
        rendered_texture: &TextureView,
        depth_texture: &TextureView,
        output_texture: &TextureView,
        rendered_pose: &Pose,
        current_pose: &Pose,
        projection: &Mat4,
        inv_projection: &Mat4,
        resolution: (u32, u32),
    ) -> Result<(), XrError> {
        if !self.atw_enabled {
            return Ok(());
        }

        // 计算旋转差异
        let delta_rotation = atw::compute_delta_rotation(rendered_pose, current_pose);

        // 更新参数缓冲区（在借用pipeline之前）
        self.update_atw_params(delta_rotation, *projection, *inv_projection, resolution)?;

        let pipeline = self
            .atw_compute_pipeline
            .as_ref()
            .ok_or_else(|| XrError::RuntimeFailure("ATW pipeline not initialized".to_string()))?;

        // 创建采样器
        let sampler = self.device.create_sampler(&SamplerDescriptor {
            label: Some("ATW Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        // 创建绑定组
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("ATW Bind Group"),
            layout: &self.create_atw_bind_group_layout(),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(rendered_texture),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(output_texture),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(depth_texture),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self
                        .atw_params_buffer
                        .as_ref()
                        .ok_or_else(|| {
                            XrError::RuntimeFailure("ATW params buffer not initialized".to_string())
                        })?
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        // 执行计算着色器
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("ATW Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_size = 8;
            let workgroup_count_x = (resolution.0 + workgroup_size - 1) / workgroup_size;
            let workgroup_count_y = (resolution.1 + workgroup_size - 1) / workgroup_size;

            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        Ok(())
    }

    /// 更新 ATW 参数
    fn update_atw_params(
        &mut self,
        delta_rotation: Mat4,
        projection: Mat4,
        inv_projection: Mat4,
        resolution: (u32, u32),
    ) -> Result<(), XrError> {
        #[repr(C)]
        #[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
        struct AtwParams {
            delta_rotation: [[f32; 4]; 4],
            inv_projection: [[f32; 4]; 4],
            projection: [[f32; 4]; 4],
            resolution: [f32; 2],
        }

        let params = AtwParams {
            delta_rotation: delta_rotation.to_cols_array_2d(),
            inv_projection: inv_projection.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            resolution: [resolution.0 as f32, resolution.1 as f32],
        };

        if self.atw_params_buffer.is_none() {
            self.atw_params_buffer = Some(self.device.create_buffer(&BufferDescriptor {
                label: Some("ATW Params Buffer"),
                size: std::mem::size_of::<AtwParams>() as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        self.queue.write_buffer(
            self.atw_params_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&params),
        );

        Ok(())
    }

    /// 应用注视点渲染
    pub fn apply_foveated_rendering(
        &mut self,
        encoder: &mut CommandEncoder,
        inner_texture: &TextureView,
        middle_texture: &TextureView,
        outer_texture: &TextureView,
        output_texture: &TextureView,
        resolution: (u32, u32),
    ) -> Result<(), XrError> {
        if !self.foveated_enabled {
            return Ok(());
        }

        // 更新参数缓冲区（在借用pipeline之前）
        self.update_foveated_params(resolution)?;

        let pipeline = self.foveated_compute_pipeline.as_ref().ok_or_else(|| {
            XrError::RuntimeFailure("Foveated pipeline not initialized".to_string())
        })?;

        // 创建采样器
        let sampler = self.device.create_sampler(&SamplerDescriptor {
            label: Some("Foveated Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        // 创建绑定组
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Foveated Bind Group"),
            layout: &self.create_foveated_bind_group_layout(),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(inner_texture),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(middle_texture),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(outer_texture),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(output_texture),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: self
                        .foveated_params_buffer
                        .as_ref()
                        .ok_or_else(|| {
                            XrError::RuntimeFailure(
                                "Foveated params buffer not initialized".to_string(),
                            )
                        })?
                        .as_entire_binding(),
                },
            ],
        });

        // 执行计算着色器
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Foveated Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_size = 8;
            let workgroup_count_x = (resolution.0 + workgroup_size - 1) / workgroup_size;
            let workgroup_count_y = (resolution.1 + workgroup_size - 1) / workgroup_size;

            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        Ok(())
    }

    /// 更新注视点渲染参数
    fn update_foveated_params(&mut self, resolution: (u32, u32)) -> Result<(), XrError> {
        #[repr(C)]
        #[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
        struct FoveatedParams {
            gaze_point: [f32; 2],
            inner_radius: f32,
            middle_radius: f32,
            outer_radius: f32,
            resolution: [f32; 2],
        }

        let params = FoveatedParams {
            gaze_point: self.foveated_config.gaze_point,
            inner_radius: self.foveated_config.inner_radius,
            middle_radius: self.foveated_config.middle_radius,
            outer_radius: self.foveated_config.outer_radius,
            resolution: [resolution.0 as f32, resolution.1 as f32],
        };

        if self.foveated_params_buffer.is_none() {
            self.foveated_params_buffer = Some(self.device.create_buffer(&BufferDescriptor {
                label: Some("Foveated Params Buffer"),
                size: std::mem::size_of::<FoveatedParams>() as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        self.queue.write_buffer(
            self.foveated_params_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&params),
        );

        Ok(())
    }

    /// 设置 ATW 启用状态
    pub fn set_atw_enabled(&mut self, enabled: bool) {
        self.atw_enabled = enabled;
    }

    /// 设置注视点渲染启用状态
    pub fn set_foveated_enabled(&mut self, enabled: bool) {
        self.foveated_enabled = enabled;
        if enabled && self.foveated_compute_pipeline.is_none() {
            let _ = self.init_foveated_pipeline();
        }
    }

    /// 设置注视点配置
    pub fn set_foveated_config(&mut self, config: foveated::FoveatedConfig) {
        self.foveated_config = config;
    }
}
