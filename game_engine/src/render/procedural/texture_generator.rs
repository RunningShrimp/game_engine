//  程序化纹理生成器
//
//  使用噪声算法生成各种纹理：
//  - 云层纹理
//  - 大理石纹理
//  - 木纹纹理
//  - 噪声纹理
//
//  ## 应用场景
//
//  - 材质生成
//  - 环境纹理
//  - 特效纹理
//  - UI纹理

use image::{ImageBuffer, Rgba, RgbaImage};
use super::noise::{NoiseGenerator, PerlinNoise, SimplexNoise, WorleyNoise, NoiseConfig};

/// 纹理生成器trait
pub trait TextureGenerator {
    /// 生成纹理
    fn generate(&self, width: u32, height: u32) -> RgbaImage;
}

/// 云层纹理生成器
pub struct CloudTextureGenerator {
    /// 噪声配置
    pub noise_config: NoiseConfig,
    /// 云层密度
    pub density: f32,
    /// 云层覆盖度
    pub coverage: f32,
}

impl CloudTextureGenerator {
    /// 创建新的云层生成器
    pub fn new(noise_config: NoiseConfig, density: f32, coverage: f32) -> Self {
        Self {
            noise_config,
            density,
            coverage,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(
            NoiseConfig {
                seed: 42,
                scale: 3.0,
                octaves: 6,
                persistence: 0.5,
                lacunarity: 2.0,
            },
            0.8,
            0.5,
        )
    }
}

impl TextureGenerator for CloudTextureGenerator {
    fn generate(&self, width: u32, height: u32) -> RgbaImage {
        let perlin = PerlinNoise::new(self.noise_config.seed);
        let mut img = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 / width as f32 * self.noise_config.scale;
                let ny = y as f32 / height as f32 * self.noise_config.scale;

                let noise = perlin.fbm(
                    nx,
                    ny,
                    self.noise_config.octaves,
                    self.noise_config.persistence,
                    self.noise_config.lacunarity,
                );

                // 映射到[0, 1]
                let value = (noise + 1.0) / 2.0;

                // 应用覆盖度和密度
                let alpha = if value > self.coverage {
                    (value - self.coverage) * self.density
                } else {
                    0.0
                };
                let alpha = alpha.clamp(0.0, 1.0);

                // 白色云层
                let color = Rgba([
                    255,
                    255,
                    255,
                    (alpha * 255.0) as u8,
                ]);

                img.put_pixel(x, y, color);
            }
        }

        img
    }
}

/// 大理石纹理生成器
pub struct MarbleTextureGenerator {
    /// 噪声配置
    pub noise_config: NoiseConfig,
    /// 纹理颜色
    pub color1: [u8; 3],
    pub color2: [u8; 3],
    /// 条纹数量
    pub stripes: f32,
    /// 扭曲度
    pub turbulence: f32,
}

impl MarbleTextureGenerator {
    /// 创建新的大理石生成器
    pub fn new(
        noise_config: NoiseConfig,
        color1: [u8; 3],
        color2: [u8; 3],
        stripes: f32,
        turbulence: f32,
    ) -> Self {
        Self {
            noise_config,
            color1,
            color2,
            stripes,
            turbulence,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(
            NoiseConfig::default(),
            [255, 255, 255],
            [50, 50, 50],
            10.0,
            1.0,
        )
    }
}

impl TextureGenerator for MarbleTextureGenerator {
    fn generate(&self, width: u32, height: u32) -> RgbaImage {
        let perlin = PerlinNoise::new(self.noise_config.seed);
        let mut img = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 / width as f32 * self.noise_config.scale;
                let ny = y as f32 / height as f32 * self.noise_config.scale;

                // 基础条纹
                let stripe = (nx * self.stripes).sin();

                // 添加噪声扭曲
                let noise = perlin.fbm(
                    nx,
                    ny,
                    self.noise_config.octaves,
                    self.noise_config.persistence,
                    self.noise_config.lacunarity,
                ) * self.turbulence;

                let value = stripe + noise;
                let t = ((value + 1.0) / 2.0).clamp(0.0, 1.0);

                // 混合颜色
                let r = (self.color1[0] as f32 * (1.0 - t) + self.color2[0] as f32 * t) as u8;
                let g = (self.color1[1] as f32 * (1.0 - t) + self.color2[1] as f32 * t) as u8;
                let b = (self.color1[2] as f32 * (1.0 - t) + self.color2[2] as f32 * t) as u8;

                img.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }

        img
    }
}

/// 木纹纹理生成器
pub struct WoodTextureGenerator {
    /// 噪声配置
    pub noise_config: NoiseConfig,
    /// 基础颜色
    pub base_color: [u8; 3],
    /// 纹理颜色
    pub grain_color: [u8; 3],
    /// 环数量
    pub rings: f32,
    /// 噪声强度
    pub noise_strength: f32,
}

impl WoodTextureGenerator {
    /// 创建新的木纹生成器
    pub fn new(
        noise_config: NoiseConfig,
        base_color: [u8; 3],
        grain_color: [u8; 3],
        rings: f32,
        noise_strength: f32,
    ) -> Self {
        Self {
            noise_config,
            base_color,
            grain_color,
            rings,
            noise_strength,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(
            NoiseConfig::default(),
            [139, 90, 43],  // 棕色
            [80, 50, 20],    // 深棕色
            15.0,
            0.3,
        )
    }
}

impl TextureGenerator for WoodTextureGenerator {
    fn generate(&self, width: u32, height: u32) -> RgbaImage {
        let perlin = PerlinNoise::new(self.noise_config.seed);
        let mut img = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 / width as f32;
                let ny = y as f32 / height as f32;

                // 距离中心的距离
                let dx = nx - 0.5;
                let dy = ny - 0.5;
                let dist = (dx * dx + dy * dy).sqrt();

                // 年环
                let ring = (dist * self.rings * 2.0 * std::f32::consts::PI).sin();

                // 添加噪声
                let noise = perlin.noise2d(
                    nx * self.noise_config.scale,
                    ny * self.noise_config.scale,
                ) * self.noise_strength;

                let value = ring + noise;
                let t = ((value + 1.0) / 2.0).clamp(0.0, 1.0);

                // 混合颜色
                let r = (self.base_color[0] as f32 * (1.0 - t) + self.grain_color[0] as f32 * t) as u8;
                let g = (self.base_color[1] as f32 * (1.0 - t) + self.grain_color[1] as f32 * t) as u8;
                let b = (self.base_color[2] as f32 * (1.0 - t) + self.grain_color[2] as f32 * t) as u8;

                img.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }

        img
    }
}

/// 噪声纹理生成器
pub struct NoiseTextureGenerator {
    /// 噪声类型
    pub noise_type: NoiseType,
    /// 噪声配置
    pub noise_config: NoiseConfig,
    /// 颜色模式
    pub color_mode: ColorMode,
}

/// 噪声类型
#[derive(Debug, Clone, Copy)]
pub enum NoiseType {
    Perlin,
    Simplex,
    Worley,
}

/// 颜色模式
#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    Grayscale,
    Rgb,
    Heatmap,
}

impl NoiseTextureGenerator {
    /// 创建新的噪声纹理生成器
    pub fn new(noise_type: NoiseType, noise_config: NoiseConfig, color_mode: ColorMode) -> Self {
        Self {
            noise_type,
            noise_config,
            color_mode,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(
            NoiseType::Perlin,
            NoiseConfig::default(),
            ColorMode::Grayscale,
        )
    }

    /// 将噪声值转换为颜色
    fn value_to_color(&self, value: f32) -> Rgba<u8> {
        let t = ((value + 1.0) / 2.0).clamp(0.0, 1.0);

        match self.color_mode {
            ColorMode::Grayscale => {
                let v = (t * 255.0) as u8;
                Rgba([v, v, v, 255])
            }
            ColorMode::Rgb => {
                Rgba([
                    (t * 255.0) as u8,
                    ((1.0 - t) * 255.0) as u8,
                    128,
                    255,
                ])
            }
            ColorMode::Heatmap => {
                // 热力图：蓝->绿->黄->红
                let (r, g, b) = if t < 0.25 {
                    let s = t / 0.25;
                    (0, (s * 255.0) as u8, 255)
                } else if t < 0.5 {
                    let s = (t - 0.25) / 0.25;
                    (0, 255, ((1.0 - s) * 255.0) as u8)
                } else if t < 0.75 {
                    let s = (t - 0.5) / 0.25;
                    ((s * 255.0) as u8, 255, 0)
                } else {
                    let s = (t - 0.75) / 0.25;
                    (255, ((1.0 - s) * 255.0) as u8, 0)
                };
                Rgba([r, g, b, 255])
            }
        }
    }
}

impl TextureGenerator for NoiseTextureGenerator {
    fn generate(&self, width: u32, height: u32) -> RgbaImage {
        let perlin = PerlinNoise::new(self.noise_config.seed);
        let simplex = SimplexNoise::new(self.noise_config.seed);
        let worley = WorleyNoise::new(self.noise_config.seed);

        let mut img = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 / width as f32 * self.noise_config.scale;
                let ny = y as f32 / height as f32 * self.noise_config.scale;

                let value = match self.noise_type {
                    NoiseType::Perlin => perlin.fbm(
                        nx,
                        ny,
                        self.noise_config.octaves,
                        self.noise_config.persistence,
                        self.noise_config.lacunarity,
                    ),
                    NoiseType::Simplex => {
                        let s = SimplexNoise::new(self.noise_config.seed);
                        s.noise2d(nx, ny)
                    }
                    NoiseType::Worley => {
                        let w = worley.worley2d(nx, ny, 5.0);
                        (1.0 - w) * 2.0 - 1.0 // 转换到[-1, 1]
                    }
                };

                let color = self.value_to_color(value);
                img.put_pixel(x, y, color);
            }
        }

        img
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_texture_generation() {
        let generator = CloudTextureGenerator::default_config();
        let img = generator.generate(64, 64);

        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
    }

    #[test]
    fn test_marble_texture_generation() {
        let generator = MarbleTextureGenerator::default_config();
        let img = generator.generate(64, 64);

        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
    }

    #[test]
    fn test_wood_texture_generation() {
        let generator = WoodTextureGenerator::default_config();
        let img = generator.generate(64, 64);

        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
    }

    #[test]
    fn test_noise_texture_generation() {
        let generator = NoiseTextureGenerator::default_config();
        let img = generator.generate(64, 64);

        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
    }
}
