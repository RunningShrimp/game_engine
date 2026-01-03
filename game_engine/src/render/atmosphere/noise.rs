//! # Noise Generation System
//!
//! This module provides various noise generation algorithms for atmospheric effects:
//! - Perlin noise
//! - Simplex noise
//! - Worley noise (cellular noise)
//! - Fractal Brownian Motion (FBM)
//!
//! These noise types are used for:
//! - Procedural cloud generation
//! - Fog density variation
//! - Atmospheric detail
//! - Weather simulation

use crate::error::RenderError;
use glam::{Vec3, Vec4};
use wgpu::util::DeviceExt;
use wgpu::{
    Buffer, Device, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureFormat,
};

/// Noise type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseType {
    /// Perlin noise
    Perlin,
    /// Simplex noise
    Simplex,
    /// Worley (cellular) noise
    Worley,
    /// Fractal Brownian Motion
    Fbm,
}

/// Noise quality settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseQuality {
    /// Low quality (faster generation)
    Low,
    /// Medium quality (balanced)
    Medium,
    /// High quality (slower, better results)
    High,
}

impl NoiseQuality {
    /// Get texture resolution for this quality
    pub fn resolution(&self) -> u32 {
        match self {
            Self::Low => 32,
            Self::Medium => 64,
            Self::High => 128,
        }
    }

    /// Get octave count for FBM
    pub fn octaves(&self) -> u32 {
        match self {
            Self::Low => 3,
            Self::Medium => 5,
            Self::High => 7,
        }
    }
}

/// Perlin noise generator
#[derive(Debug, Clone)]
pub struct PerlinNoise {
    /// Permutation table
    permutation: [u8; 512],
    /// Seed for reproducibility
    seed: u32,
}

impl PerlinNoise {
    /// Create a new Perlin noise generator
    pub fn new(seed: u32) -> Self {
        let mut permutation = [0u8; 512];
        let mut p: [u8; 256] = [0; 256];

        // Initialize permutation table with seed
        for (i, p) in p.iter_mut().enumerate() {
            *p = i as u8;
        }

        // Shuffle using seed
        let mut random = seed;
        for i in (1..256).rev() {
            random = random.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (random % (i as u32 + 1)) as usize;
            p.swap(i, j);
        }

        // Duplicate for easy wrapping
        permutation[0..256].copy_from_slice(&p);
        permutation[256..512].copy_from_slice(&p);

        Self { permutation, seed }
    }

    /// Sample 2D noise
    pub fn sample2d(&self, x: f32, y: f32) -> f32 {
        self.fade_coord(x, y)
    }

    /// Sample 3D noise
    pub fn sample3d(&self, x: f32, y: f32, z: f32) -> f32 {
        let xi = x.floor() as i32 & 255;
        let yi = y.floor() as i32 & 255;
        let zi = z.floor() as i32 & 255;

        let xf = x - x.floor();
        let yf = y - y.floor();
        let zf = z - z.floor();

        let u = Self::fade(xf);
        let v = Self::fade(yf);
        let w = Self::fade(zf);

        let aaa = self.perm(i32::from(self.perm(xi)) + yi + zi);
        let aba = self.perm(i32::from(self.perm(xi)) + yi + 1 + zi);
        let aab = self.perm(i32::from(self.perm(xi)) + yi + zi + 1);
        let abb = self.perm(i32::from(self.perm(xi)) + yi + 1 + zi + 1);
        let baa = self.perm(i32::from(self.perm(xi + 1)) + yi + zi);
        let bba = self.perm(i32::from(self.perm(xi + 1)) + yi + 1 + zi);
        let bab = self.perm(i32::from(self.perm(xi + 1)) + yi + zi + 1);
        let bbb = self.perm(i32::from(self.perm(xi + 1)) + yi + 1 + zi + 1);

        let x1 = Self::lerp(aaa as f32, baa as f32, u);
        let x2 = Self::lerp(aba as f32, bba as f32, u);
        let y1 = Self::lerp(x1, x2, v);

        let x1 = Self::lerp(aab as f32, bab as f32, u);
        let x2 = Self::lerp(abb as f32, bbb as f32, u);
        let y2 = Self::lerp(x1, x2, v);

        Self::lerp(y1, y2, w)
    }

    /// Sample 4D noise
    pub fn sample4d(&self, x: f32, y: f32, z: f32, w: f32) -> f32 {
        // Simplified 4D noise
        let n3d = self.sample3d(x, y, z);
        let n3d_w = self.sample3d(x + w, y + w, z + w);
        Self::lerp(n3d, n3d_w, Self::fade(w))
    }

    /// Fade function for smooth interpolation
    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    /// Linear interpolation
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    /// Grad function for gradient computation
    fn grad(&self, hash: u8, x: f32, y: f32, z: f32) -> f32 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 {
            y
        } else if h == 12 || h == 14 {
            x
        } else {
            z
        };

        ((if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })) as f32
    }

    /// Permutation lookup
    fn perm(&self, i: i32) -> u8 {
        self.permutation[(i & 255) as usize]
    }

    /// Fade coordinates for 2D
    fn fade_coord(&self, x: f32, y: f32) -> f32 {
        let xi = x.floor() as i32 & 255;
        let yi = y.floor() as i32 & 255;

        let xf = x - x.floor();
        let yf = y - y.floor();

        let u = Self::fade(xf);
        let v = Self::fade(yf);

        let aa = self.perm(i32::from(self.perm(xi)) + yi);
        let ab = self.perm(i32::from(self.perm(xi)) + yi + 1);
        let ba = self.perm(i32::from(self.perm(xi + 1)) + yi);
        let bb = self.perm(i32::from(self.perm(xi + 1)) + yi + 1);

        let x1 = Self::lerp(aa as f32, ba as f32, u);
        let x2 = Self::lerp(ab as f32, bb as f32, u);
        Self::lerp(x1, x2, v)
    }
}

impl Default for PerlinNoise {
    fn default() -> Self {
        Self::new(42)
    }
}

/// Simplex noise generator
#[derive(Debug, Clone)]
pub struct SimplexNoise {
    /// Permutation table
    permutation: [u8; 512],
    /// Skew factors for 3D
    f3: f32,
    g3: f32,
}

impl SimplexNoise {
    /// Create a new Simplex noise generator
    pub fn new(seed: u32) -> Self {
        let mut permutation = [0u8; 512];
        let mut p: [u8; 256] = [0; 256];

        for (i, p) in p.iter_mut().enumerate() {
            *p = i as u8;
        }

        // Shuffle
        let mut random = seed;
        for i in (1..256).rev() {
            random = random.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (random % (i as u32 + 1)) as usize;
            p.swap(i, j);
        }

        permutation[0..256].copy_from_slice(&p);
        permutation[256..512].copy_from_slice(&p);

        // Skew factors for 3D simplex
        let f3 = 1.0 / 3.0;
        let g3 = 1.0 / 6.0;

        Self {
            permutation,
            f3,
            g3,
        }
    }

    /// Sample 2D simplex noise
    pub fn sample2d(&self, x: f32, y: f32) -> f32 {
        const F2: f32 = 0.5 * (std::f32::consts::SQRT_2 - 1.0);
        const G2: f32 = (3.0 - std::f32::consts::SQRT_2) / 6.0;

        let s = (x + y) * F2;
        let i = (x + s).floor() as i32;
        let j = (y + s).floor() as i32;

        let t = (i + j) as f32 * G2;
        let x0 = x - (i as f32 - t);
        let y0 = y - (j as f32 - t);

        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        let x1 = x0 - i1 as f32 + G2;
        let y1 = y0 - j1 as f32 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        let ii = i & 255;
        let jj = j & 255;

        let mut n0 = 0.0;
        let mut t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 >= 0.0 {
            let gi0 = self.perm(i32::from(self.perm(ii + jj)) & 7);
            t0 *= t0;
            n0 = t0 * t0 * self.dot2(gi0, x0, y0);
        }

        let mut n1 = 0.0;
        let mut t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 >= 0.0 {
            let gi1 = self.perm(i32::from(self.perm(ii + i1 + jj + j1)) & 7);
            t1 *= t1;
            n1 = t1 * t1 * self.dot2(gi1, x1, y1);
        }

        let mut n2 = 0.0;
        let mut t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 >= 0.0 {
            let gi2 = self.perm(i32::from(self.perm(ii + 1 + jj + 1)) & 7);
            t2 *= t2;
            n2 = t2 * t2 * self.dot2(gi2, x2, y2);
        }

        70.0 * (n0 + n1 + n2)
    }

    /// Sample 3D simplex noise
    pub fn sample3d(&self, xin: f32, yin: f32, zin: f32) -> f32 {
        let n0: f32;
        let n1: f32;
        let n2: f32;
        let n3: f32;

        let s = (xin + yin + zin) * self.f3;
        let i = (xin + s).floor() as i32;
        let j = (yin + s).floor() as i32;
        let k = (zin + s).floor() as i32;

        let t = (i + j + k) as f32 * self.g3;
        let x0 = xin - (i as f32 - t);
        let y0 = yin - (j as f32 - t);
        let z0 = zin - (k as f32 - t);

        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 {
                (1, 0, 0, 1, 1, 0)
            } else if x0 >= z0 {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else {
            // y0 < x0
            if y0 < z0 {
                (0, 0, 1, 0, 1, 1)
            } else if x0 < z0 {
                (0, 1, 0, 0, 1, 1)
            } else {
                (0, 1, 0, 1, 1, 0)
            }
        };

        let x1 = x0 - i1 as f32 + self.g3;
        let y1 = y0 - j1 as f32 + self.g3;
        let z1 = z0 - k1 as f32 + self.g3;
        let x2 = x0 - i2 as f32 + 2.0 * self.g3;
        let y2 = y0 - j2 as f32 + 2.0 * self.g3;
        let z2 = z0 - k2 as f32 + 2.0 * self.g3;
        let x3 = x0 - 1.0 + 3.0 * self.g3;
        let y3 = y0 - 1.0 + 3.0 * self.g3;
        let z3 = z0 - 1.0 + 3.0 * self.g3;

        let ii = i & 255;
        let jj = j & 255;
        let kk = k & 255;

        let gi0 = self.perm(i32::from(self.perm(i32::from(self.perm(ii)) + jj)) + kk) % 12;
        let t0 = 0.6 - x0 * x0 - y0 * y0 - z0 * z0;
        n0 = if t0 < 0.0 {
            0.0
        } else {
            let t0 = t0 * t0;
            t0 * t0 * self.dot3(gi0, x0, y0, z0)
        };

        let gi1 =
            self.perm(i32::from(self.perm(i32::from(self.perm(ii + i1)) + jj + j1)) + kk + k1) % 12;
        let t1 = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
        n1 = if t1 < 0.0 {
            0.0
        } else {
            let t1 = t1 * t1;
            t1 * t1 * self.dot3(gi1, x1, y1, z1)
        };

        let gi2 =
            self.perm(i32::from(self.perm(i32::from(self.perm(ii + i2)) + jj + j2)) + kk + k2) % 12;
        let t2 = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
        n2 = if t2 < 0.0 {
            0.0
        } else {
            let t2 = t2 * t2;
            t2 * t2 * self.dot3(gi2, x2, y2, z2)
        };

        let gi3 =
            self.perm(i32::from(self.perm(i32::from(self.perm(ii + 1)) + jj + 1)) + kk + 1) % 12;
        let t3 = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
        n3 = if t3 < 0.0 {
            0.0
        } else {
            let t3 = t3 * t3;
            t3 * t3 * self.dot3(gi3, x3, y3, z3)
        };

        32.0 * (n0 + n1 + n2 + n3)
    }

    /// Permutation lookup
    fn perm(&self, i: i32) -> u8 {
        self.permutation[(i & 255) as usize]
    }

    /// Dot product for 2D gradient
    fn dot2(&self, g: u8, x: f32, y: f32) -> f32 {
        match g {
            0 => x + y,
            1 => -x + y,
            2 => x - y,
            3 => -x - y,
            4 => y,
            5 => -y,
            6 => x,
            7 => -x,
            _ => 0.0,
        }
    }

    /// Dot product for 3D gradient
    fn dot3(&self, g: u8, x: f32, y: f32, z: f32) -> f32 {
        match g {
            0 => x + y,
            1 => -x + y,
            2 => x - y,
            3 => -x - y,
            4 => x + z,
            5 => -x + z,
            6 => x - z,
            7 => -x - z,
            8 => y + z,
            9 => -y + z,
            10 => y - z,
            11 => -y - z,
            _ => 0.0,
        }
    }
}

impl Default for SimplexNoise {
    fn default() -> Self {
        Self::new(42)
    }
}

/// Worley (cellular) noise generator
#[derive(Debug, Clone)]
pub struct WorleyNoise {
    perlin: PerlinNoise,
}

impl WorleyNoise {
    /// Create new Worley noise generator
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: PerlinNoise::new(seed),
        }
    }

    /// Sample 2D Worley noise
    pub fn sample2d(&self, x: f32, y: f32) -> f32 {
        self.cellular2d(x, y).0
    }

    /// Sample 3D Worley noise
    pub fn sample3d(&self, x: f32, y: f32, z: f32) -> f32 {
        self.cellular3d(x, y, z).0
    }

    /// Get cellular features (distance to closest feature point)
    pub fn cellular2d(&self, x: f32, y: f32) -> (f32, f32, f32) {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;

        let mut min_dist = f32::MAX;
        let mut second_min = f32::MAX;
        let mut third_min = f32::MAX;

        // Search neighboring cells
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cell_x = xi + dx;
                let cell_y = yi + dy;

                // Use hash to generate feature point in this cell
                let hash = self.hash2d(cell_x, cell_y);
                let feature_x = cell_x as f32 + (hash & 255) as f32 / 256.0;
                let feature_y = cell_y as f32 + ((hash >> 8) & 255) as f32 / 256.0;

                let dist = ((x - feature_x).powi(2) + (y - feature_y).powi(2)).sqrt();

                if dist < min_dist {
                    third_min = second_min;
                    second_min = min_dist;
                    min_dist = dist;
                } else if dist < second_min {
                    third_min = second_min;
                    second_min = dist;
                } else if dist < third_min {
                    third_min = dist;
                }
            }
        }

        (min_dist, second_min, third_min)
    }

    /// Get cellular features in 3D
    pub fn cellular3d(&self, x: f32, y: f32, z: f32) -> (f32, f32) {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let zi = z.floor() as i32;

        let mut min_dist = f32::MAX;
        let mut second_min = f32::MAX;

        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let cell_x = xi + dx;
                    let cell_y = yi + dy;
                    let cell_z = zi + dz;

                    let hash = self.hash3d(cell_x, cell_y, cell_z);
                    let feature_x = cell_x as f32 + (hash & 255) as f32 / 256.0;
                    let feature_y = cell_y as f32 + ((hash >> 8) & 255) as f32 / 256.0;
                    let feature_z = cell_z as f32 + ((hash >> 16) & 255) as f32 / 256.0;

                    let dist = ((x - feature_x).powi(2)
                        + (y - feature_y).powi(2)
                        + (z - feature_z).powi(2))
                    .sqrt();

                    if dist < min_dist {
                        second_min = min_dist;
                        min_dist = dist;
                    } else if dist < second_min {
                        second_min = dist;
                    }
                }
            }
        }

        (min_dist, second_min)
    }

    /// Hash 2D coordinates
    fn hash2d(&self, x: i32, y: i32) -> u32 {
        let mut hash = x as u32 * 374761393 + y as u32 * 668265263;
        hash = hash.wrapping_mul(1013904223).wrapping_add(1664525);
        hash
    }

    /// Hash 3D coordinates
    fn hash3d(&self, x: i32, y: i32, z: i32) -> u32 {
        let mut hash = x as u32 * 73856093 + y as u32 * 19349663 + z as u32 * 83492791;
        hash = hash.wrapping_mul(1013904223).wrapping_add(1664525);
        hash
    }
}

impl Default for WorleyNoise {
    fn default() -> Self {
        Self::new(42)
    }
}

/// Fractal Brownian Motion (FBM) configuration
#[derive(Debug, Clone)]
pub struct FbmConfig {
    /// Number of octaves
    pub octaves: u32,
    /// Lacunarity (frequency multiplier per octave)
    pub lacunarity: f32,
    /// Persistence (amplitude multiplier per octave)
    pub persistence: f32,
    /// Base frequency
    pub frequency: f32,
    /// Base amplitude
    pub amplitude: f32,
}

impl Default for FbmConfig {
    fn default() -> Self {
        Self {
            octaves: 5,
            lacunarity: 2.0,
            persistence: 0.5,
            frequency: 1.0,
            amplitude: 1.0,
        }
    }
}

/// Noise generator combining all noise types
#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    perlin: PerlinNoise,
    simplex: SimplexNoise,
    worley: WorleyNoise,
    fbm_config: FbmConfig,
}

impl NoiseGenerator {
    /// Create new noise generator
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: PerlinNoise::new(seed),
            simplex: SimplexNoise::new(seed + 1),
            worley: WorleyNoise::new(seed + 2),
            fbm_config: FbmConfig::default(),
        }
    }

    /// Set FBM configuration
    pub fn set_fbm_config(&mut self, config: FbmConfig) {
        self.fbm_config = config;
    }

    /// Sample Perlin noise
    pub fn perlin3d(&self, x: f32, y: f32, z: f32) -> f32 {
        self.perlin.sample3d(x, y, z)
    }

    /// Sample Simplex noise
    pub fn simplex3d(&self, x: f32, y: f32, z: f32) -> f32 {
        self.simplex.sample3d(x, y, z)
    }

    /// Sample Worley noise
    pub fn worley3d(&self, x: f32, y: f32, z: f32) -> f32 {
        self.worley.sample3d(x, y, z)
    }

    /// Sample FBM noise with Perlin
    pub fn fbm_perlin3d(&self, mut x: f32, mut y: f32, mut z: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = self.fbm_config.frequency;
        let mut amplitude = self.fbm_config.amplitude;
        let mut max_value = 0.0;

        for _ in 0..self.fbm_config.octaves {
            total += self.perlin.sample3d(x * frequency, y * frequency, z * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= self.fbm_config.persistence;
            frequency *= self.fbm_config.lacunarity;
        }

        total / max_value
    }

    /// Sample FBM noise with Simplex
    pub fn fbm_simplex3d(&self, mut x: f32, mut y: f32, mut z: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = self.fbm_config.frequency;
        let mut amplitude = self.fbm_config.amplitude;
        let mut max_value = 0.0;

        for _ in 0..self.fbm_config.octaves {
            total += self.simplex.sample3d(x * frequency, y * frequency, z * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= self.fbm_config.persistence;
            frequency *= self.fbm_config.lacunarity;
        }

        total / max_value
    }

    /// Generate 3D noise texture
    pub fn generate_texture_3d(
        &self,
        device: &Device,
        queue: &Queue,
        size: u32,
        noise_type: NoiseType,
    ) -> Result<Texture, RenderError> {
        let format = TextureFormat::R8Unorm;
        let bytes_per_pixel = 1;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("3D Noise Texture ({:?})", noise_type)),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: size,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mut data = Vec::with_capacity((size * size * size) as usize);

        let scale = 1.0 / size as f32;

        for z in 0..size {
            for y in 0..size {
                for x in 0..size {
                    let nx = x as f32 * scale;
                    let ny = y as f32 * scale;
                    let nz = z as f32 * scale;

                    let noise = match noise_type {
                        NoiseType::Perlin => {
                            (self.perlin3d(nx * 4.0, ny * 4.0, nz * 4.0) * 0.5 + 0.5)
                        }
                        NoiseType::Simplex => {
                            (self.simplex3d(nx * 4.0, ny * 4.0, nz * 4.0) * 0.5 + 0.5)
                        }
                        NoiseType::Worley => (1.0 - self.worley3d(nx * 4.0, ny * 4.0, nz * 4.0)),
                        NoiseType::Fbm => {
                            (self.fbm_perlin3d(nx * 4.0, ny * 4.0, nz * 4.0) * 0.5 + 0.5)
                        }
                    };

                    let value = (noise.clamp(0.0, 1.0) * 255.0) as u8;
                    data.push(value);
                }
            }
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size * bytes_per_pixel),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: size,
            },
        );

        Ok(texture)
    }

    /// Generate 2D noise texture
    pub fn generate_texture_2d(
        &self,
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        noise_type: NoiseType,
    ) -> Result<Texture, RenderError> {
        let format = TextureFormat::R8Unorm;
        let bytes_per_pixel = 1;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("2D Noise Texture ({:?})", noise_type)),
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

        let mut data = Vec::with_capacity((width * height) as usize);

        let scale_x = 1.0 / width as f32;
        let scale_y = 1.0 / height as f32;

        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 * scale_x;
                let ny = y as f32 * scale_y;

                let noise = match noise_type {
                    NoiseType::Perlin => (self.perlin.sample2d(nx * 4.0, ny * 4.0) * 0.5 + 0.5),
                    NoiseType::Simplex => (self.simplex.sample2d(nx * 4.0, ny * 4.0) * 0.5 + 0.5),
                    NoiseType::Worley => (1.0 - self.worley.sample2d(nx * 4.0, ny * 4.0)),
                    NoiseType::Fbm => (self.fbm_perlin3d(nx * 4.0, ny * 4.0, 0.0) * 0.5 + 0.5),
                };

                let value = (noise.clamp(0.0, 1.0) * 255.0) as u8;
                data.push(value);
            }
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * bytes_per_pixel),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Ok(texture)
    }
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self::new(42)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_noise_range() {
        let perlin = PerlinNoise::new(42);
        let value = perlin.sample3d(0.5, 0.5, 0.5);
        assert!(value >= -1.0 && value <= 1.0);
    }

    #[test]
    fn test_simplex_noise_range() {
        let simplex = SimplexNoise::new(42);
        let value = simplex.sample3d(0.5, 0.5, 0.5);
        assert!(value >= -1.0 && value <= 1.0);
    }

    #[test]
    fn test_worley_noise() {
        let worley = WorleyNoise::new(42);
        let (d1, d2) = worley.cellular3d(0.5, 0.5, 0.5);
        assert!(d1 > 0.0);
        assert!(d2 >= d1);
    }

    #[test]
    fn test_fbm() {
        let mut noise_gen = NoiseGenerator::new(42);
        noise_gen.set_fbm_config(FbmConfig {
            octaves: 3,
            ..Default::default()
        });
        let value = noise_gen.fbm_perlin3d(0.5, 0.5, 0.5);
        assert!(value >= -1.0 && value <= 1.0);
    }
}
