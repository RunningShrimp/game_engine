use wgpu::{Device, Queue, Texture, TextureFormat, TextureUsages, TextureView};

/// 离屏渲染目标
pub struct OffscreenTarget {
    /// 纹理
    pub texture: Texture,
    /// 纹理视图
    pub view: TextureView,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 格式
    pub format: TextureFormat,
}

impl OffscreenTarget {
    /// 创建新的离屏渲染目标
    pub fn new(device: &Device, width: u32, height: u32, format: TextureFormat) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Render Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
            format,
        }
    }

    /// 调整大小
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

        self.texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Render Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }
}

/// 特效类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    /// 模糊
    Blur,
    /// 发光
    Bloom,
    /// 色彩调整
    ColorAdjust,
    /// 扭曲
    Distortion,
}

/// 特效参数
#[derive(Debug, Clone)]
pub struct EffectParams {
    /// 特效类型
    pub effect_type: EffectType,
    /// 强度 (0.0 - 1.0)
    pub intensity: f32,
    /// 自定义参数
    pub custom_params: Vec<f32>,
}

impl EffectParams {
    /// 创建模糊特效
    pub fn blur(radius: f32) -> Self {
        Self {
            effect_type: EffectType::Blur,
            intensity: 1.0,
            custom_params: vec![radius],
        }
    }

    /// 创建发光特效
    pub fn bloom(threshold: f32, intensity: f32) -> Self {
        Self {
            effect_type: EffectType::Bloom,
            intensity,
            custom_params: vec![threshold],
        }
    }

    /// 创建色彩调整特效
    pub fn color_adjust(brightness: f32, contrast: f32, saturation: f32) -> Self {
        Self {
            effect_type: EffectType::ColorAdjust,
            intensity: 1.0,
            custom_params: vec![brightness, contrast, saturation],
        }
    }
}

/// 特效渲染器
pub struct EffectRenderer {
    /// 离屏渲染目标
    targets: Vec<OffscreenTarget>,
    /// 当前目标索引
    current_target: usize,
}

impl EffectRenderer {
    /// 创建新的特效渲染器
    pub fn new(device: &Device, width: u32, height: u32, format: TextureFormat) -> Self {
        // 创建两个离屏目标用于ping-pong渲染
        let targets = vec![
            OffscreenTarget::new(device, width, height, format),
            OffscreenTarget::new(device, width, height, format),
        ];

        Self {
            targets,
            current_target: 0,
        }
    }

    /// 获取当前渲染目标
    pub fn current_target(&self) -> &OffscreenTarget {
        &self.targets[self.current_target]
    }

    /// 获取下一个渲染目标
    pub fn next_target(&self) -> &OffscreenTarget {
        &self.targets[1 - self.current_target]
    }

    /// 交换渲染目标
    pub fn swap_targets(&mut self) {
        self.current_target = 1 - self.current_target;
    }

    /// 调整大小
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        for target in &mut self.targets {
            target.resize(device, width, height);
        }
    }

    /// 应用特效
    pub fn apply_effect(&mut self, _device: &Device, _queue: &Queue, _effect: &EffectParams) {
        // NOTE: 特效渲染逻辑待实现
        // 计划步骤：
        // 1. 从当前目标读取纹理
        // 2. 应用特效着色器
        // 3. 写入输出目标
        // 3. 渲染到下一个目标
        // 4. 交换目标
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_params() {
        let blur = EffectParams::blur(5.0);
        assert_eq!(blur.effect_type, EffectType::Blur);
        assert_eq!(blur.custom_params[0], 5.0);

        let bloom = EffectParams::bloom(0.8, 1.5);
        assert_eq!(bloom.effect_type, EffectType::Bloom);
        assert_eq!(bloom.intensity, 1.5);
        assert_eq!(bloom.custom_params[0], 0.8);
    }
}
