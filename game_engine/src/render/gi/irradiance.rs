//! # 辐照度纹理管理
//!
//! 管理DDGI的辐照度纹理，用于存储探针捕获的光照信息。

use crate::render::gi::volume::DDGIConfig;
use glam::UVec3;
use wgpu::{
    Device, Origin3d, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};

/// 辐照度纹理
pub struct IrradianceTexture {
    /// 纹理
    texture: Texture,
    /// 纹理视图
    view: TextureView,
    /// 探针数量
    probe_count: u32,
    /// 探针网格尺寸
    probe_counts: UVec3,
    /// 辐照度分辨率
    irradiance_resolution: u32,
}

impl IrradianceTexture {
    /// 创建新的辐照度纹理
    pub fn new(
        device: &Device,
        config: &DDGIConfig,
        probe_count: u32,
        probe_counts: UVec3,
    ) -> Result<Self, crate::render::gi::DDGIError> {
        // 使用2D数组纹理存储辐照度
        // 每个探针有6个面，每个面是 irradiance_resolution x irradiance_resolution
        let face_count = 6u32;
        let array_layers = probe_count * face_count;

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("DDGI Irradiance Texture"),
            size: wgpu::Extent3d {
                width: config.irradiance_resolution,
                height: config.irradiance_resolution,
                depth_or_array_layers: array_layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor {
            label: Some("DDGI Irradiance Texture View"),
            format: Some(TextureFormat::Rgba32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            usage: None, // 新版本需要这个字段
        });

        Ok(Self {
            texture,
            view,
            probe_count,
            probe_counts,
            irradiance_resolution: config.irradiance_resolution,
        })
    }

    /// 获取纹理
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// 获取纹理视图
    pub fn view(&self) -> &TextureView {
        &self.view
    }

    /// 获取探针数量
    pub fn probe_count(&self) -> u32 {
        self.probe_count
    }

    /// 获取探针网格尺寸
    pub fn probe_counts(&self) -> UVec3 {
        self.probe_counts
    }

    /// 获取辐照度分辨率
    pub fn irradiance_resolution(&self) -> u32 {
        self.irradiance_resolution
    }

    /// 计算探针的纹理索引
    pub fn probe_texture_index(&self, probe_index: u32, face: u32) -> u32 {
        assert!(face < 6, "Face must be in range [0, 6)");
        probe_index * 6 + face
    }

    /// 计算探针在3D网格中的位置
    pub fn probe_grid_position(&self, probe_index: u32) -> (u32, u32, u32) {
        let z = probe_index / (self.probe_counts.x * self.probe_counts.y);
        let temp = probe_index % (self.probe_counts.x * self.probe_counts.y);
        let y = temp / self.probe_counts.x;
        let x = temp % self.probe_counts.x;
        (x, y, z)
    }

    /// 清空纹理数据（简化实现）
    pub fn clear(&self, _queue: &wgpu::Queue) {
        // 简化实现：当前跳过
        // 实际应该使用queue.write_texture清空纹理
    }

    /// 更新探针辐照度数据（简化实现）
    pub fn update_probe(
        &self,
        _queue: &wgpu::Queue,
        _probe_index: u32,
        _face: u32,
        _data: &[glam::Vec4],
    ) {
        // 简化实现：当前跳过
        // 实际应该使用queue.write_texture更新纹理
    }
}

/// 球谐函数系数（用于辐照度存储）
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphericalHarmonics {
    /// L0系数（DC分量）
    pub l00: glam::Vec3,
    /// L1系数
    pub l1_1: glam::Vec3,
    pub l10: glam::Vec3,
    pub l11: glam::Vec3,
    /// L2系数（可选）
    pub l2_2: glam::Vec3,
    pub l2_1: glam::Vec3,
    pub l20: glam::Vec3,
    pub l21: glam::Vec3,
    pub l22: glam::Vec3,
}

impl Default for SphericalHarmonics {
    fn default() -> Self {
        Self {
            l00: glam::Vec3::ZERO,
            l1_1: glam::Vec3::ZERO,
            l10: glam::Vec3::ZERO,
            l11: glam::Vec3::ZERO,
            l2_2: glam::Vec3::ZERO,
            l2_1: glam::Vec3::ZERO,
            l20: glam::Vec3::ZERO,
            l21: glam::Vec3::ZERO,
            l22: glam::Vec3::ZERO,
        }
    }
}

impl SphericalHarmonics {
    /// 从环境光创建球谐函数
    pub fn from_environment(color: glam::Vec3) -> Self {
        Self {
            l00: color,
            l1_1: glam::Vec3::ZERO,
            l10: glam::Vec3::ZERO,
            l11: glam::Vec3::ZERO,
            l2_2: glam::Vec3::ZERO,
            l2_1: glam::Vec3::ZERO,
            l20: glam::Vec3::ZERO,
            l21: glam::Vec3::ZERO,
            l22: glam::Vec3::ZERO,
        }
    }

    /// 评估球谐函数
    pub fn evaluate(&self, direction: glam::Vec3) -> glam::Vec3 {
        // 简化实现：只使用L0和L1
        let c1 = 0.429043;
        let c2 = 0.511664;
        let c3 = 0.743125;
        let c4 = 0.886227;
        let c5 = 0.247708;

        self.l00 * c4
            + self.l1_1 * c2 * direction.y
            + self.l10 * c2 * direction.z
            + self.l11 * c2 * direction.x
            + self.l2_2 * c1 * direction.x * direction.y
            + self.l2_1 * c1 * direction.y * direction.z
            + self.l20 * c3 * direction.z * direction.z
            + self.l21 * c1 * direction.z * direction.x
            + self.l22 * c5 * (direction.x * direction.x - direction.y * direction.y)
    }

    /// 添加光照贡献
    pub fn add_light(&mut self, direction: glam::Vec3, color: glam::Vec3, intensity: f32) {
        // 简化实现：只更新L0
        self.l00 += color * intensity * 0.282095; // SH(0,0)系数
    }

    /// 转换为Vec4数组（用于存储到纹理）
    pub fn to_vec4_array(&self) -> [glam::Vec4; 3] {
        [
            glam::Vec4::new(self.l00.x, self.l00.y, self.l00.z, 0.0),
            glam::Vec4::new(self.l1_1.x, self.l1_1.y, self.l1_1.z, 0.0),
            glam::Vec4::new(self.l10.x, self.l10.y, self.l10.z, 0.0),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherical_harmonics_default() {
        let sh = SphericalHarmonics::default();
        assert_eq!(sh.l00, glam::Vec3::ZERO);
    }

    #[test]
    fn test_spherical_harmonics_from_environment() {
        let color = glam::Vec3::new(1.0, 0.5, 0.25);
        let sh = SphericalHarmonics::from_environment(color);
        assert_eq!(sh.l00, color);
    }

    #[test]
    fn test_spherical_harmonics_evaluate() {
        let mut sh = SphericalHarmonics::from_environment(glam::Vec3::splat(1.0));
        let result = sh.evaluate(glam::Vec3::Y);
        assert!(result.x > 0.0 && result.y > 0.0 && result.z > 0.0);
    }
}
