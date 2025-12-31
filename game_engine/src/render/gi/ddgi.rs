//! # DDGI核心实现
//!
//! 实现动态漫反射全局光照的核心算法和渲染逻辑。

use crate::render::gi::irradiance::IrradianceTexture;
use crate::render::gi::probe::DDGIProbe;
use crate::render::gi::volume::DDGIConfig;
use glam::{UVec3, Vec3};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, Device, Queue, Texture, TextureUsages,
};

/// DDGI错误类型
#[derive(Debug, thiserror::Error)]
pub enum DDGIError {
    #[error("Invalid probe configuration: {0}")]
    InvalidConfig(String),

    #[error("Texture creation failed: {0}")]
    TextureError(String),

    #[error("Shader error: {0}")]
    ShaderError(String),

    #[error("Buffer error: {0}")]
    BufferError(String),

    #[error("Render error: {0}")]
    RenderError(String),
}

/// DDGI质量级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DDGIQuality {
    Low,
    Medium,
    High,
}

impl DDGIQuality {
    /// 获取默认配置
    pub fn default_config(&self) -> DDGIConfig {
        match self {
            DDGIQuality::Low => DDGIConfig {
                probe_spacing: 4.0,
                probe_counts: UVec3::new(5, 5, 5),
                irradiance_resolution: 8,
                depth_resolution: 8,
                max_depth: 50.0,
                normal_bias: 0.1,
                update_rate: 6, // 每6帧更新一次
                ..Default::default()
            },
            DDGIQuality::Medium => DDGIConfig {
                probe_spacing: 2.0,
                probe_counts: UVec3::new(10, 10, 10),
                irradiance_resolution: 16,
                depth_resolution: 16,
                max_depth: 50.0,
                normal_bias: 0.05,
                update_rate: 3, // 每3帧更新一次
                ..Default::default()
            },
            DDGIQuality::High => DDGIConfig {
                probe_spacing: 1.0,
                probe_counts: UVec3::new(20, 20, 20),
                irradiance_resolution: 32,
                depth_resolution: 32,
                max_depth: 100.0,
                normal_bias: 0.02,
                update_rate: 1, // 每帧更新
                ..Default::default()
            },
        }
    }
}

/// DDGI体积
pub struct DDGIVolume {
    /// 探针列表
    probes: Vec<DDGIProbe>,
    /// 探针间距
    probe_spacing: f32,
    /// 探针数量
    probe_counts: UVec3,
    /// 辐照度纹理
    irradiance_texture: IrradianceTexture,
    /// 深度纹理
    depth_texture: Texture,
    /// 偏移纹理（用于优化）
    offset_texture: Texture,
    /// 绑定组布局
    bind_group_layout: BindGroupLayout,
    /// 绑定组
    bind_group: Option<BindGroup>,
    /// 探针缓冲区
    probe_buffer: Option<Buffer>,
    /// 配置
    config: DDGIConfig,
    /// 当前帧计数
    frame_count: u32,
    /// 体积原点（世界空间）
    volume_origin: Vec3,
}

impl DDGIVolume {
    /// 创建新的DDGI体积
    pub fn new(device: &Device, config: &DDGIConfig) -> Result<Self, DDGIError> {
        // 验证配置
        if config.probe_counts.x == 0 || config.probe_counts.y == 0 || config.probe_counts.z == 0 {
            return Err(DDGIError::InvalidConfig(
                "Probe counts must be non-zero".to_string(),
            ));
        }

        if config.probe_spacing <= 0.0 {
            return Err(DDGIError::InvalidConfig(
                "Probe spacing must be positive".to_string(),
            ));
        }

        // 创建探针网格
        let probe_counts = config.probe_counts;
        let probe_spacing = config.probe_spacing;

        let mut probes = Vec::new();
        let total_probes = probe_counts.x * probe_counts.y * probe_counts.z;

        for z in 0..probe_counts.z {
            for y in 0..probe_counts.y {
                for x in 0..probe_counts.x {
                    let pos = Vec3::new(
                        x as f32 * probe_spacing,
                        y as f32 * probe_spacing,
                        z as f32 * probe_spacing,
                    );

                    probes.push(DDGIProbe::new(pos));
                }
            }
        }

        // 计算体积原点（使体积居中）
        let volume_size = Vec3::new(
            (probe_counts.x - 1) as f32 * probe_spacing,
            (probe_counts.y - 1) as f32 * probe_spacing,
            (probe_counts.z - 1) as f32 * probe_spacing,
        );
        let volume_origin = -volume_size / 2.0;

        // 创建辐照度纹理（2D数组纹理，每个探针6个面）
        let irradiance_texture =
            IrradianceTexture::new(device, config, total_probes, probe_counts)?;

        // 创建深度纹理
        let depth_texture = Self::create_depth_texture(device, config, total_probes)?;

        // 创建偏移纹理
        let offset_texture = Self::create_offset_texture(device, config, total_probes)?;

        // 创建绑定组布局
        let bind_group_layout = Self::create_bind_group_layout(device);

        // 创建探针缓冲区
        let probe_buffer = Self::create_probe_buffer(device, &probes);

        Ok(Self {
            probes,
            probe_spacing,
            probe_counts,
            irradiance_texture,
            depth_texture,
            offset_texture,
            bind_group_layout,
            bind_group: None,
            probe_buffer: Some(probe_buffer),
            config: config.clone(),
            frame_count: 0,
            volume_origin,
        })
    }

    /// 创建深度纹理
    fn create_depth_texture(device: &Device, config: &DDGIConfig, probe_count: u32) -> Texture {
        let face_count = 6u32;
        let size = config.depth_resolution;

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("DDGI Depth Texture"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: probe_count * face_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    /// 创建偏移纹理
    fn create_offset_texture(device: &Device, config: &DDGIConfig, probe_count: u32) -> Texture {
        let size = config.depth_resolution;

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("DDGI Offset Texture"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: probe_count * 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg32Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        })
    }

    /// 创建绑定组布局（简化实现）
    fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
        // 简化实现：创建一个最小化的绑定组布局
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("DDGI Bind Group Layout"),
            entries: &[
                // 辐照度纹理
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        })
    }

    /// 创建探针缓冲区
    fn create_probe_buffer(device: &Device, probes: &[DDGIProbe]) -> Buffer {
        let probe_data: Vec<f32> = probes
            .iter()
            .flat_map(|p| {
                [
                    p.position.x,
                    p.position.y,
                    p.position.z,
                    0.0, // padding
                    p.irradiance.x,
                    p.irradiance.y,
                    p.irradiance.z,
                    0.0, // padding
                    p.depth,
                    0.0,
                    0.0,
                    0.0,
                    p.offset.x,
                    p.offset.y,
                    0.0,
                    0.0,
                ]
            })
            .collect();

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DDGI Probe Buffer"),
            contents: bytemuck::cast_slice(&probe_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    /// 更新DDGI体积
    pub fn update(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
    ) -> Result<(), DDGIError> {
        self.frame_count += 1;

        // 检查是否需要更新
        if self.frame_count % self.config.update_rate != 0 {
            return Ok(());
        }

        // 1. 更新探针位置（如果场景动态变化）
        self.update_probe_positions();

        // 2. 渲染探针（由外部渲染器调用）
        // 这里只提供接口，实际渲染在渲染器中完成

        // 3. 更新辐照度
        self.update_irradiance(device, queue, encoder)?;

        // 4. 传播光照
        self.propagate_lighting(device, queue, encoder)?;

        // 更新探针缓冲区
        if let Some(buffer) = &self.probe_buffer {
            let probe_data: Vec<f32> = self
                .probes
                .iter()
                .flat_map(|p| {
                    [
                        p.position.x,
                        p.position.y,
                        p.position.z,
                        0.0,
                        p.irradiance.x,
                        p.irradiance.y,
                        p.irradiance.z,
                        0.0,
                        p.depth,
                        0.0,
                        0.0,
                        0.0,
                        p.offset.x,
                        p.offset.y,
                        0.0,
                        0.0,
                    ]
                })
                .collect();

            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&probe_data));
        }

        Ok(())
    }

    /// 更新探针位置
    fn update_probe_positions(&mut self) {
        // 如果场景有动态变化，可以在这里更新探针位置
        // 当前实现保持探针静止
    }

    /// 更新辐照度
    fn update_irradiance(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
    ) -> Result<(), DDGIError> {
        // 这里应该调用计算着色器来更新辐照度
        // 着色器会从深度纹理和法线纹理计算辐照度

        // 简化实现：直接更新CPU端数据
        for probe in &mut self.probes {
            // 在真实实现中，这里会从GPU读取数据
            // 当前使用简化版本
            probe.irradiance = Vec3::new(0.5, 0.5, 0.5); // 默认灰色
        }

        Ok(())
    }

    /// 传播光照
    fn propagate_lighting(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
    ) -> Result<(), DDGIError> {
        // 实现探针间的光照传播
        // 可以使用迭代扩散或直接求解

        // 简化实现：当前跳过
        Ok(())
    }

    /// 渲染探针（用于调试）
    pub fn render_probes(
        &self,
        _encoder: &mut CommandEncoder,
        _scene: &crate::render::domain_objects::RenderScene,
    ) -> Result<(), DDGIError> {
        // 渲染每个探针的6个面
        // 由外部渲染器实现

        Ok(())
    }

    /// 获取探针数量
    pub fn probe_count(&self) -> usize {
        self.probes.len()
    }

    /// 获取探针
    pub fn get_probe(&self, index: usize) -> Option<&DDGIProbe> {
        self.probes.get(index)
    }

    /// 获取探针（可变）
    pub fn get_probe_mut(&mut self, index: usize) -> Option<&mut DDGIProbe> {
        self.probes.get_mut(index)
    }

    /// 获取配置
    pub fn config(&self) -> &DDGIConfig {
        &self.config
    }

    /// 获取体积原点
    pub fn volume_origin(&self) -> Vec3 {
        self.volume_origin
    }

    /// 获取辐照度纹理
    pub fn irradiance_texture(&self) -> &IrradianceTexture {
        &self.irradiance_texture
    }

    /// 获取深度纹理视图
    pub fn depth_texture_view(&self) -> wgpu::TextureView {
        self.depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("DDGI Depth Texture View"),
            format: Some(wgpu::TextureFormat::R32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            usage: None, // 新版本需要这个字段
        })
    }

    /// 获取绑定组布局
    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
}

/// DDGI Uniform 数据
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DDGIUniforms {
    pub volume_origin: [f32; 3],
    pub volume_size: [f32; 3],
    pub probe_counts: [u32; 3],
    pub probe_spacing: f32,
    pub max_depth: f32,
    pub normal_bias: f32,
    pub padding: f32,
}

impl DDGIUniforms {
    pub fn new(volume: &DDGIVolume) -> Self {
        Self {
            volume_origin: [
                volume.volume_origin.x,
                volume.volume_origin.y,
                volume.volume_origin.z,
            ],
            volume_size: [
                (volume.probe_counts.x - 1) as f32 * volume.probe_spacing,
                (volume.probe_counts.y - 1) as f32 * volume.probe_spacing,
                (volume.probe_counts.z - 1) as f32 * volume.probe_spacing,
            ],
            probe_counts: [
                volume.probe_counts.x,
                volume.probe_counts.y,
                volume.probe_counts.z,
            ],
            probe_spacing: volume.probe_spacing,
            max_depth: volume.config.max_depth,
            normal_bias: volume.config.normal_bias,
            padding: 0.0,
        }
    }
}
