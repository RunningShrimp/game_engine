//  后处理管线模块
//
//  提供完整的后处理效果管线，包括：
//  - Antialiasing（抗锯齿：FXAA/TAA）
//  - Bloom（辉光效果）
//  - SSAO（屏幕空间环境光遮蔽）
//  - Tonemap（HDR色调映射）
//
//  # 示例
//
//  ```ignore
//  let mut postprocess = PostProcessPipeline::new(&device, &config);
//  postprocess.set_antialiasing(AntialiasingMode::FXAA);
//  postprocess.set_bloom_enabled(true);
//  postprocess.set_bloom_intensity(0.8);
//  postprocess.render(&mut encoder, &scene_texture, &output_view);
//  ```

use crate::impl_default;

pub mod antialiasing;
pub mod bloom;
pub mod color_correction;
pub mod depth_of_field;
pub mod effect_manager;
pub mod motion_blur;
pub mod ssao;
pub mod ssr;
pub mod tonemap;
pub mod volumetric_lighting;
pub mod procedural_noise;

pub use antialiasing::{AntialiasingMode, FxaaPass, FxaaQuality, TaaPass};
pub use bloom::BloomPass;
pub use color_correction::{ColorCorrectionPass, ColorCorrectionUniforms};
pub use depth_of_field::DepthOfFieldPass;
pub use effect_manager::{
    EffectPerformanceStats, EffectPreset, PostProcessEffect, PostProcessEffectManager, QualityMode,
};
pub use motion_blur::MotionBlurPass;
pub use ssao::SsaoPass;
pub use ssr::{SsrConfig, SsrPass, SsrUniforms};
pub use tonemap::{TonemapOperator, TonemapPass};
pub use volumetric_lighting::{VolumetricLightingConfig, VolumetricLightingPass, VolumetricLightingUniforms};
pub use procedural_noise::{ProceduralNoiseConfig, ProceduralNoisePass, ProceduralNoiseUniforms};

use wgpu::TextureFormat;

/// 后处理配置
#[derive(Debug, Clone)]
pub struct PostProcessConfig {
    /// 抗锯齿模式
    pub antialiasing: AntialiasingMode,
    /// FXAA 质量等级
    pub fxaa_quality: FxaaQuality,

    /// 是否启用 Bloom
    pub bloom_enabled: bool,
    /// Bloom 强度 (0.0 - 2.0)
    pub bloom_intensity: f32,
    /// Bloom 阈值 - 亮度超过此值才会产生辉光
    pub bloom_threshold: f32,
    /// Bloom 模糊半径
    pub bloom_radius: f32,

    /// 是否启用 SSAO
    pub ssao_enabled: bool,
    /// SSAO 采样半径
    pub ssao_radius: f32,
    /// SSAO 强度
    pub ssao_intensity: f32,
    /// SSAO 偏移
    pub ssao_bias: f32,

    /// 是否启用色调映射
    pub tonemap_enabled: bool,
    /// 色调映射算法
    pub tonemap_operator: TonemapOperator,
    /// 曝光值
    pub exposure: f32,
    /// Gamma 校正值
    pub gamma: f32,

    /// 是否启用运动模糊
    pub motion_blur_enabled: bool,
    /// 运动模糊强度 (0.0 - 1.0)
    pub motion_blur_intensity: f32,
    /// 运动模糊最大采样数 (4 - 32)
    pub motion_blur_max_samples: u32,

    /// 是否启用景深
    pub depth_of_field_enabled: bool,
    /// 焦点距离 (0.0 - 100.0)
    pub focus_distance: f32,
    /// 光圈大小 (0.0 - 1.0)
    pub aperture: f32,
    /// 近景模糊强度 (0.0 - 10.0)
    pub near_blur: f32,
    /// 远景模糊强度 (0.0 - 10.0)
    pub far_blur: f32,
    /// 最大模糊半径 (0.0 - 20.0)
    pub max_blur_radius: f32,

    /// 是否启用色彩校正
    pub color_correction_enabled: bool,
    /// 亮度调整 (-1.0 到 1.0)
    pub brightness: f32,
    /// 对比度调整 (0.0 到 2.0)
    pub contrast: f32,
    /// 饱和度调整 (0.0 到 2.0)
    pub saturation: f32,
    /// 色调偏移 (-180.0 到 180.0 度)
    pub hue_shift: f32,
    /// 色差强度 (0.0 到 1.0)
    pub chromatic_aberration: f32,
    /// 暗角强度 (0.0 到 1.0)
    pub vignette_intensity: f32,
    /// 暗角圆度 (0.0 到 1.0)
    pub vignette_roundness: f32,
}

impl_default!(PostProcessConfig {
    antialiasing: AntialiasingMode::FXAA,
    fxaa_quality: FxaaQuality::Medium,
    bloom_enabled: true,
    bloom_intensity: 0.5,
    bloom_threshold: 1.0,
    bloom_radius: 5.0,
    ssao_enabled: false,
    ssao_radius: 0.5,
    ssao_intensity: 1.0,
    ssao_bias: 0.025,
    tonemap_enabled: true,
    tonemap_operator: TonemapOperator::ACES,
    exposure: 1.0,
    gamma: 2.2,
    motion_blur_enabled: false,
    motion_blur_intensity: 0.3,
    motion_blur_max_samples: 16,
    depth_of_field_enabled: false,
    focus_distance: 10.0,
    aperture: 0.5,
    near_blur: 1.0,
    far_blur: 2.0,
    max_blur_radius: 10.0,
    color_correction_enabled: false,
    brightness: 0.0,
    contrast: 1.0,
    saturation: 1.0,
    hue_shift: 0.0,
    chromatic_aberration: 0.0,
    vignette_intensity: 0.0,
    vignette_roundness: 0.5,
});

/// 后处理 Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostProcessUniforms {
    /// 屏幕尺寸 (width, height)
    pub screen_size: [f32; 2],
    /// Bloom 强度
    pub bloom_intensity: f32,
    /// Bloom 阈值
    pub bloom_threshold: f32,
    /// 曝光值
    pub exposure: f32,
    /// Gamma 值
    pub gamma: f32,
    /// 色调映射算法 (0=None, 1=Reinhard, 2=ACES, 3=Filmic)
    pub tonemap_mode: u32,
    /// 填充对齐
    pub _pad: u32,
}

/// 后处理管线
///
/// 管理所有后处理效果的渲染通道
pub struct PostProcessPipeline {
    /// 配置
    pub config: PostProcessConfig,

    /// Bloom 通道
    bloom_pass: BloomPass,

    /// SSAO 通道
    ssao_pass: SsaoPass,

    /// Tonemap 通道
    tonemap_pass: TonemapPass,

    /// 运动模糊通道
    motion_blur_pass: MotionBlurPass,

    /// 景深通道
    depth_of_field_pass: DepthOfFieldPass,

    /// 色彩校正通道
    color_correction_pass: ColorCorrectionPass,

    /// Uniform 缓冲区
    uniform_buffer: wgpu::Buffer,

    /// Uniform 绑定组
    #[allow(dead_code)]
    uniform_bind_group: wgpu::BindGroup,

    /// 中间纹理（HDR场景）
    hdr_texture: wgpu::Texture,
    hdr_view: wgpu::TextureView,

    /// 屏幕尺寸
    width: u32,
    height: u32,

    /// 输出格式
    #[allow(dead_code)]
    output_format: TextureFormat,
}

impl PostProcessPipeline {
    /// 创建后处理管线
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let width = config.width;
        let height = config.height;
        let output_format = config.format;

        // 创建 HDR 中间纹理
        let hdr_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PostProcess HDR Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let hdr_view = hdr_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 创建 Uniform 缓冲区
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PostProcess Uniform Buffer"),
            size: std::mem::size_of::<PostProcessUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建绑定组布局
        let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PostProcess Uniform BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PostProcess Uniform BG"),
            layout: &uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 创建各个后处理通道
        let bloom_pass = BloomPass::new(device, width, height);
        let ssao_pass = SsaoPass::new(device, width, height);
        let tonemap_pass = TonemapPass::new(device, output_format);
        let motion_blur_pass = MotionBlurPass::new(device, output_format);
        let depth_of_field_pass = DepthOfFieldPass::new(device, output_format);
        let color_correction_pass = ColorCorrectionPass::new(device, output_format);

        Self {
            config: PostProcessConfig::default(),
            bloom_pass,
            ssao_pass,
            tonemap_pass,
            motion_blur_pass,
            depth_of_field_pass,
            color_correction_pass,
            uniform_buffer,
            uniform_bind_group,
            hdr_texture,
            hdr_view,
            width,
            height,
            output_format,
        }
    }

    /// 调整后处理管线大小
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == self.width && height == self.height {
            return;
        }

        self.width = width;
        self.height = height;

        // 重新创建 HDR 纹理
        self.hdr_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PostProcess HDR Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.hdr_view = self.hdr_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 调整各通道大小
        self.bloom_pass.resize(device, width, height);
        self.ssao_pass.resize(device, width, height);
    }

    /// 更新 Uniform 数据
    fn update_uniforms(&self, queue: &wgpu::Queue) {
        let uniforms = PostProcessUniforms {
            screen_size: [self.width as f32, self.height as f32],
            bloom_intensity: self.config.bloom_intensity,
            bloom_threshold: self.config.bloom_threshold,
            exposure: self.config.exposure,
            gamma: self.config.gamma,
            tonemap_mode: self.config.tonemap_operator as u32,
            _pad: 0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// 执行后处理渲染
    ///
    /// # 参数
    /// - `encoder`: 命令编码器
    /// - `device`: GPU 设备
    /// - `queue`: 命令队列
    /// - `scene_view`: 场景纹理视图（输入）
    /// - `depth_view`: 深度纹理视图（用于 SSAO 和景深）
    /// - `motion_vector_view`: 运动向量纹理视图（用于运动模糊）
    /// - `output_view`: 输出纹理视图
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_view: &wgpu::TextureView,
        depth_view: Option<&wgpu::TextureView>,
        motion_vector_view: Option<&wgpu::TextureView>,
        output_view: &wgpu::TextureView,
    ) {
        // 更新 uniforms
        self.update_uniforms(queue);

        let mut current_input = scene_view;

        // 1. SSAO 通道
        if self.config.ssao_enabled
            && let Some(depth) = depth_view {
                self.ssao_pass.render(
                    encoder,
                    device,
                    queue,
                    current_input,
                    depth,
                    self.config.ssao_radius,
                    self.config.ssao_intensity,
                    self.config.ssao_bias,
                );
                current_input = self.ssao_pass.output_view();
            }

        // 2. Bloom 通道
        if self.config.bloom_enabled {
            self.bloom_pass.render(
                encoder,
                device,
                queue,
                current_input,
                self.config.bloom_threshold,
                self.config.bloom_intensity,
                self.config.bloom_radius,
            );
            current_input = self.bloom_pass.output_view();
        }

        // 3. 运动模糊通道
        if self.config.motion_blur_enabled
            && let Some(motion_vectors) = motion_vector_view {
                self.motion_blur_pass.render(
                    encoder,
                    device,
                    queue,
                    current_input,
                    motion_vectors,
                    current_input,
                    self.width,
                    self.height,
                    self.config.motion_blur_intensity,
                    self.config.motion_blur_max_samples,
                );
            }

        // 4. 景深通道
        if self.config.depth_of_field_enabled
            && let Some(depth) = depth_view {
                self.depth_of_field_pass.render(
                    encoder,
                    device,
                    queue,
                    current_input,
                    depth,
                    current_input,
                    self.width,
                    self.height,
                    self.config.focus_distance,
                    self.config.aperture,
                    self.config.near_blur,
                    self.config.far_blur,
                    self.config.max_blur_radius,
                );
            }

        // 5. 色彩校正通道
        if self.config.color_correction_enabled {
            self.color_correction_pass.render(
                encoder,
                device,
                queue,
                current_input,
                output_view,
                self.config.brightness,
                self.config.contrast,
                self.config.saturation,
                self.config.hue_shift,
                self.config.chromatic_aberration,
                self.config.vignette_intensity,
                self.config.vignette_roundness,
            );
        } else {
            // 6. Tonemap 通道（最终输出）
            self.tonemap_pass.render(
                encoder,
                device,
                queue,
                current_input,
                output_view,
                self.config.exposure,
                self.config.gamma,
                self.config.tonemap_operator,
            );
        }
    }

    /// 获取 HDR 纹理视图（用于渲染场景）
    pub fn hdr_view(&self) -> &wgpu::TextureView {
        &self.hdr_view
    }

    /// 设置 Bloom 启用状态
    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.config.bloom_enabled = enabled;
    }

    /// 设置 Bloom 强度
    pub fn set_bloom_intensity(&mut self, intensity: f32) {
        self.config.bloom_intensity = intensity.clamp(0.0, 2.0);
    }

    /// 设置 Bloom 阈值
    pub fn set_bloom_threshold(&mut self, threshold: f32) {
        self.config.bloom_threshold = threshold.max(0.0);
    }

    /// 设置 SSAO 启用状态
    pub fn set_ssao_enabled(&mut self, enabled: bool) {
        self.config.ssao_enabled = enabled;
    }

    /// 设置 SSAO 参数
    pub fn set_ssao_params(&mut self, radius: f32, intensity: f32, bias: f32) {
        self.config.ssao_radius = radius.max(0.01);
        self.config.ssao_intensity = intensity.clamp(0.0, 5.0);
        self.config.ssao_bias = bias.clamp(0.0, 0.1);
    }

    /// 设置色调映射算法
    pub fn set_tonemap_operator(&mut self, operator: TonemapOperator) {
        self.config.tonemap_operator = operator;
    }

    /// 设置曝光值
    pub fn set_exposure(&mut self, exposure: f32) {
        self.config.exposure = exposure.clamp(0.1, 10.0);
    }

    /// 设置 Gamma 值
    pub fn set_gamma(&mut self, gamma: f32) {
        self.config.gamma = gamma.clamp(1.0, 3.0);
    }

    /// 设置运动模糊启用状态
    pub fn set_motion_blur_enabled(&mut self, enabled: bool) {
        self.config.motion_blur_enabled = enabled;
    }

    /// 设置运动模糊强度
    pub fn set_motion_blur_intensity(&mut self, intensity: f32) {
        self.config.motion_blur_intensity = intensity.clamp(0.0, 1.0);
    }

    /// 设置运动模糊最大采样数
    pub fn set_motion_blur_max_samples(&mut self, max_samples: u32) {
        self.config.motion_blur_max_samples = max_samples.clamp(4, 32);
    }

    /// 设置景深启用状态
    pub fn set_depth_of_field_enabled(&mut self, enabled: bool) {
        self.config.depth_of_field_enabled = enabled;
    }

    /// 设置景深焦点距离
    pub fn set_focus_distance(&mut self, distance: f32) {
        self.config.focus_distance = distance.max(0.0);
    }

    /// 设置景深光圈大小
    pub fn set_aperture(&mut self, aperture: f32) {
        self.config.aperture = aperture.clamp(0.0, 1.0);
    }

    /// 设置景深模糊强度
    pub fn set_depth_of_field_blur(&mut self, near_blur: f32, far_blur: f32) {
        self.config.near_blur = near_blur.max(0.0);
        self.config.far_blur = far_blur.max(0.0);
    }

    /// 设置最大模糊半径
    pub fn set_max_blur_radius(&mut self, radius: f32) {
        self.config.max_blur_radius = radius.clamp(0.0, 20.0);
    }

    /// 设置色彩校正启用状态
    pub fn set_color_correction_enabled(&mut self, enabled: bool) {
        self.config.color_correction_enabled = enabled;
    }

    /// 设置亮度
    pub fn set_brightness(&mut self, brightness: f32) {
        self.config.brightness = brightness.clamp(-1.0, 1.0);
    }

    /// 设置对比度
    pub fn set_contrast(&mut self, contrast: f32) {
        self.config.contrast = contrast.clamp(0.0, 2.0);
    }

    /// 设置饱和度
    pub fn set_saturation(&mut self, saturation: f32) {
        self.config.saturation = saturation.clamp(0.0, 2.0);
    }

    /// 设置色调偏移
    pub fn set_hue_shift(&mut self, hue_shift: f32) {
        self.config.hue_shift = hue_shift.clamp(-180.0, 180.0);
    }

    /// 设置色差强度
    pub fn set_chromatic_aberration(&mut self, intensity: f32) {
        self.config.chromatic_aberration = intensity.clamp(0.0, 1.0);
    }

    /// 设置暗角强度
    pub fn set_vignette_intensity(&mut self, intensity: f32) {
        self.config.vignette_intensity = intensity.clamp(0.0, 1.0);
    }

    /// 设置暗角圆度
    pub fn set_vignette_roundness(&mut self, roundness: f32) {
        self.config.vignette_roundness = roundness.clamp(0.0, 1.0);
    }
}
