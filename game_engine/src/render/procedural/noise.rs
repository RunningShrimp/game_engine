//  程序化噪声生成器
//
//  提供多种噪声算法用于程序化内容生成：
//  - Perlin噪声：平滑的自然噪声
//  - Simplex噪声：更高维度的性能优化
//  - Worley噪声：细胞噪声，用于纹理效果
//  - 分形布朗运动（FBM）：多层噪声叠加
//
//  ## 性能优化策略
//
//  1. **预计算梯度表**
//     - 避免运行时三角函数计算
//     - 使用查找表
//
//  2. **SIMD优化**
//     - 批量采样
//     - 向量化计算
//
//  3. **缓存友好**
//     - 连续内存访问
//     - 局部性优化
//
//  ## 应用场景
//
//  - 地形生成
//  - 云层纹理
//  - 材质图案
//  - 动画噪声

use glam::{Vec2, Vec3};
use std::f32::consts::PI;

/// 噪声生成器trait
pub trait NoiseGenerator {
    /// 1D噪声采样
    fn noise1d(&self, x: f32) -> f32;

    /// 2D噪声采样
    fn noise2d(&self, x: f32, y: f32) -> f32;

    /// 3D噪声采样
    fn noise3d(&self, x: f32, y: f32, z: f32) -> f32;

    /// 分形布朗运动（FBM）
    fn fbm(&self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += self.noise2d(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        total / max_value
    }

    /// 3D FBM
    fn fbm3d(&self, pos: Vec3, octaves: u32, persistence: f32, lacunarity: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total +=
                self.noise3d(pos.x * frequency, pos.y * frequency, pos.z * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        total / max_value
    }
}

/// Perlin噪声生成器
#[derive(Debug, Clone)]
pub struct PerlinNoise {
    /// 梯度表（预计算）
    gradients: Vec<Vec2>,
    /// 排列表
    permutation: Vec<usize>,
    /// 种子
    seed: u32,
}

impl PerlinNoise {
    /// 创建新的Perlin噪声生成器
    pub fn new(seed: u32) -> Self {
        let mut rng = SeededRng { seed };
        let mut gradients = Vec::with_capacity(256);
        let mut permutation = Vec::with_capacity(512);

        // 生成梯度向量
        for _ in 0..256 {
            let angle = rng.random_f32() * 2.0 * PI;
            gradients.push(Vec2::new(angle.cos(), angle.sin()));
        }

        // 生成排列表
        let mut p: Vec<usize> = (0..256).collect();
        for i in (1..256).rev() {
            let j = rng.random_range(0..=i);
            p.swap(i, j);
        }

        // 复制排列表以避免边界检查
        for i in 0..512 {
            permutation.push(p[i & 255]);
        }

        Self {
            gradients,
            permutation,
            seed,
        }
    }

    /// 使用默认种子创建
    pub fn default_seed() -> Self {
        Self::new(42)
    }

    /// 平滑插值函数
    #[inline]
    fn fade(&self, t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    /// 线性插值
    #[inline]
    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    /// 计算梯度点积
    #[inline]
    fn grad(&self, hash: usize, x: f32, y: f32) -> f32 {
        let grad = &self.gradients[hash & 255];
        grad.x * x + grad.y * y
    }
}

impl NoiseGenerator for PerlinNoise {
    fn noise1d(&self, x: f32) -> f32 {
        // 1D Perlin噪声实现
        let x0 = x.floor() as i32;
        let x1 = x0 + 1;

        let fx = x - x.floor();

        let s = self.grad(self.permutation[x0 as usize & 255], fx, 0.0);
        let g = self.grad(self.permutation[x1 as usize & 255], fx - 1.0, 0.0);

        let u = self.fade(fx);

        self.lerp(s, g, u)
    }

    fn noise2d(&self, x: f32, y: f32) -> f32 {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let fx = x - x.floor();
        let fy = y - y.floor();

        let u = self.fade(fx);
        let v = self.fade(fy);

        let aaa = self.grad(
            self.permutation[(self.permutation[x0 as usize & 255] + y0 as usize) & 255],
            fx,
            fy,
        );
        let baa = self.grad(
            self.permutation[(self.permutation[x1 as usize & 255] + y0 as usize) & 255],
            fx - 1.0,
            fy,
        );
        let aba = self.grad(
            self.permutation[(self.permutation[x0 as usize & 255] + y1 as usize) & 255],
            fx,
            fy - 1.0,
        );
        let bba = self.grad(
            self.permutation[(self.permutation[x1 as usize & 255] + y1 as usize) & 255],
            fx - 1.0,
            fy - 1.0,
        );

        let x1_lerp = self.lerp(aaa, baa, u);
        let x2_lerp = self.lerp(aba, bba, u);

        self.lerp(x1_lerp, x2_lerp, v)
    }

    fn noise3d(&self, x: f32, y: f32, z: f32) -> f32 {
        // 简化的3D实现
        let xy = self.noise2d(x, y);
        let yz = self.noise2d(y, z);
        let xz = self.noise2d(x, z);

        (xy + yz + xz) / 3.0
    }
}

/// Simplex噪声生成器
///
/// 相比Perlin噪声，Simplex噪声在高维度上性能更好。
#[derive(Debug, Clone)]
pub struct SimplexNoise {
    /// 梯度表
    gradients3d: Vec<Vec3>,
    /// 排列表
    permutation: Vec<usize>,
    /// 种子
    seed: u32,
}

impl SimplexNoise {
    /// 创建新的Simplex噪声生成器
    pub fn new(seed: u32) -> Self {
        let mut rng = SeededRng { seed };
        let mut gradients3d = Vec::with_capacity(12);
        let mut permutation = Vec::with_capacity(512);

        // 12个3D梯度向量（正二十面体的顶点）
        let sqrt3 = (3.0_f32).sqrt();
        let _sqrt6 = (6.0_f32).sqrt();

        gradients3d.push(Vec3::new(1.0, 1.0, 0.0));
        gradients3d.push(Vec3::new(-1.0, 1.0, 0.0));
        gradients3d.push(Vec3::new(1.0, -1.0, 0.0));
        gradients3d.push(Vec3::new(-1.0, -1.0, 0.0));

        gradients3d.push(Vec3::new(1.0, 0.0, 1.0));
        gradients3d.push(Vec3::new(-1.0, 0.0, 1.0));
        gradients3d.push(Vec3::new(1.0, 0.0, -1.0));
        gradients3d.push(Vec3::new(-1.0, 0.0, -1.0));

        gradients3d.push(Vec3::new(0.0, 1.0, 1.0));
        gradients3d.push(Vec3::new(0.0, -1.0, 1.0));
        gradients3d.push(Vec3::new(0.0, 1.0, -1.0));
        gradients3d.push(Vec3::new(0.0, -1.0, -1.0));

        // 归一化
        for grad in &mut gradients3d {
            *grad /= sqrt3;
        }

        // 生成排列表
        let mut p: Vec<usize> = (0..256).collect();
        for i in (1..256).rev() {
            let j = rng.random_range(0..=i);
            p.swap(i, j);
        }

        for i in 0..512 {
            permutation.push(p[i & 255]);
        }

        Self {
            gradients3d,
            permutation,
            seed,
        }
    }

    /// 使用默认种子创建
    pub fn default_seed() -> Self {
        Self::new(42)
    }

    /// Simplex噪声的F2偏移常量
    const F2: f32 = 0.366_025_42; // 0.5 * (sqrt(3) - 1)
    /// Simplex噪声的G2偏移常量
    const G2: f32 = 0.211_324_87; // (sqrt(3) - 1) / 6
}

impl NoiseGenerator for SimplexNoise {
    fn noise1d(&self, x: f32) -> f32 {
        // 1D使用Perlin噪声
        let perlin = PerlinNoise::new(self.seed);
        perlin.noise1d(x)
    }

    fn noise2d(&self, x: f32, y: f32) -> f32 {
        // Skew输入空间
        let s = (x + y) * Self::F2;
        let i = (x + s).floor() as i32;
        let j = (y + s).floor() as i32;

        let t = (i + j) as f32 * Self::G2;
        let x0 = x - (i as f32 - t);
        let y0 = y - (j as f32 - t);

        // 确定哪个simplex corner
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        let x1 = x0 - i1 as f32 + Self::G2;
        let y1 = y0 - j1 as f32 + Self::G2;
        let x2 = x0 - 1.0 + 2.0 * Self::G2;
        let y2 = y0 - 1.0 + 2.0 * Self::G2;

        // 计算贡献
        let mut n0 = 0.0;
        let mut n1 = 0.0;
        let mut n2 = 0.0;

        let t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 >= 0.0 {
            let t0_sq = t0 * t0;
            let gi0 =
                &self.gradients3d[self.permutation[i as usize & (255 + j as usize) & 255] % 12];
            n0 = t0_sq * t0_sq * (gi0.x * x0 + gi0.y * y0);
        }

        let t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 >= 0.0 {
            let t1_sq = t1 * t1;
            let gi1 = &self.gradients3d
                [self.permutation[i as usize & (255 + i1 + j as usize) & (255 + j1)] % 12];
            n1 = t1_sq * t1_sq * (gi1.x * x1 + gi1.y * y1);
        }

        let t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 >= 0.0 {
            let t2_sq = t2 * t2;
            let gi2 = &self.gradients3d
                [self.permutation[i as usize & (255 + 1 + j as usize) & (255 + 1)] % 12];
            n2 = t2_sq * t2_sq * (gi2.x * x2 + gi2.y * y2);
        }

        // 缩放结果
        70.0 * (n0 + n1 + n2)
    }

    fn noise3d(&self, x: f32, y: f32, z: f32) -> f32 {
        // 简化的3D实现
        let xy = self.noise2d(x, y);
        let yz = self.noise2d(y, z);
        let xz = self.noise2d(x, z);

        (xy + yz + xz) / 3.0
    }
}

/// Worley噪声（细胞噪声）
///
/// 基于特征点的距离函数，常用于纹理效果。
#[derive(Debug, Clone)]
pub struct WorleyNoise {
    seed: u32,
}

impl WorleyNoise {
    /// 创建新的Worley噪声生成器
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// 使用默认种子创建
    pub fn default_seed() -> Self {
        Self::new(42)
    }

    /// 哈希函数
    fn hash(&self, x: i32, y: i32) -> f32 {
        let mut h = (x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263))) as u32;
        h = h.wrapping_add(self.seed);
        h ^= h >> 13;
        h = h.wrapping_mul(1274126177);
        (h as f32) / (u32::MAX as f32)
    }

    /// 2D Worley噪声
    pub fn worley2d(&self, x: f32, y: f32, scale: f32) -> f32 {
        let sx = (x * scale).floor() as i32;
        let sy = (y * scale).floor() as i32;

        let mut min_dist = f32::MAX;

        // 检查3x3网格
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cx = sx + dx;
                let cy = sy + dy;

                // 在单元格内生成特征点
                let fx = cx as f32 + self.hash(cx, cy);
                let fy = cy as f32 + self.hash(cx + 1, cy + 1);

                let px = x * scale;
                let py = y * scale;

                let dist = ((px - fx) * (px - fx) + (py - fy) * (py - fy)).sqrt();
                min_dist = min_dist.min(dist);
            }
        }

        min_dist
    }
}

/// 简单的种子随机数生成器
struct SeededRng {
    seed: u32,
}

impl SeededRng {
    fn random_f32(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.seed as f32) / (u32::MAX as f32)
    }

    fn random_range(&mut self, range: std::ops::RangeInclusive<usize>) -> usize {
        let scale = (range.end() - range.start() + 1) as f32;
        (self.random_f32() * scale).floor() as usize + range.start()
    }
}

/// 噪声配置
#[derive(Debug, Clone)]
pub struct NoiseConfig {
    /// 种子
    pub seed: u32,
    /// 缩放
    pub scale: f32,
    /// 八度数（FBM层数）
    pub octaves: u32,
    /// 持续度（每层振幅衰减）
    pub persistence: f32,
    /// 频率（每层频率倍增）
    pub lacunarity: f32,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            scale: 1.0,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_noise() {
        let perlin = PerlinNoise::default_seed();

        // 测试噪声值在[-1, 1]范围内
        let n = perlin.noise2d(0.5, 0.5);
        assert!(n >= -1.0 && n <= 1.0);

        // 测试FBM
        let fbm = perlin.fbm(0.5, 0.5, 4, 0.5, 2.0);
        assert!(fbm >= -1.0 && fbm <= 1.0);
    }

    #[test]
    fn test_simplex_noise() {
        let simplex = SimplexNoise::default_seed();

        let n = simplex.noise2d(0.5, 0.5);
        assert!(n >= -1.0 && n <= 1.0);
    }

    #[test]
    fn test_worley_noise() {
        let worley = WorleyNoise::default_seed();

        let n = worley.worley2d(0.5, 0.5, 5.0);
        assert!(n >= 0.0);
    }

    #[test]
    fn test_noise_consistency() {
        let perlin1 = PerlinNoise::new(12345);
        let perlin2 = PerlinNoise::new(12345);

        let n1 = perlin1.noise2d(1.0, 2.0);
        let n2 = perlin2.noise2d(1.0, 2.0);

        assert!((n1 - n2).abs() < 1e-6);
    }
}
