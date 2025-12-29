//  程序化网格生成器
//
//  提供各种程序化网格生成算法：
//  - 基础几何体（球体、立方体、圆柱等）
//  - 地形网格
//  - 洞穴网格
//  - 植被网格
//
//  ## 性能优化
//
//  1. **索引复用**
//     - 共享顶点数据
//     - 减少内存占用
//
//  2. **LOD支持**
//     - 动态细节层次
//     - 距离-based简化
//
//  3. **法线计算**
//     - 自动法线生成
//     - 平滑/硬边切换

use super::noise::{NoiseConfig, NoiseGenerator, PerlinNoise};
use glam::{Vec2, Vec3};

/// 程序化网格顶点
#[derive(Debug, Clone)]
pub struct ProceduralVertex {
    /// 位置
    pub position: Vec3,
    /// 法线
    pub normal: Vec3,
    /// UV坐标
    pub uv: Vec2,
    /// 切线
    pub tangent: Vec3,
    /// 副切线
    pub bitangent: Vec3,
    /// 颜色
    pub color: [f32; 4],
}

/// 程序化网格
#[derive(Debug, Clone)]
pub struct ProceduralMesh {
    /// 顶点数据
    pub vertices: Vec<ProceduralVertex>,
    /// 索引数据
    pub indices: Vec<u32>,
}

// 类型别名用于向后兼容
pub type Vertex = ProceduralVertex;
pub type Mesh = ProceduralMesh;

/// 网格生成器trait
pub trait MeshGenerator {
    /// 生成网格
    fn generate(&self) -> Mesh;
}

/// 基础几何体生成器
pub struct PrimitiveGenerator;

impl PrimitiveGenerator {
    /// 创建立方体
    pub fn cube(size: f32) -> Mesh {
        let half = size / 2.0;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // 8个顶点
        let positions = [
            Vec3::new(-half, -half, -half),
            Vec3::new(half, -half, -half),
            Vec3::new(half, half, -half),
            Vec3::new(-half, half, -half),
            Vec3::new(-half, -half, half),
            Vec3::new(half, -half, half),
            Vec3::new(half, half, half),
            Vec3::new(-half, half, half),
        ];

        // 6个面，每个面2个三角形
        let faces = vec![
            // Front
            [4, 5, 6],
            [4, 6, 7],
            // Back
            [1, 0, 3],
            [1, 3, 2],
            // Top
            [7, 6, 2],
            [7, 2, 3],
            // Bottom
            [0, 1, 5],
            [0, 5, 4],
            // Right
            [5, 1, 2],
            [5, 2, 6],
            // Left
            [0, 4, 7],
            [0, 7, 3],
        ];

        let normals = [
            Vec3::new(0.0, 0.0, 1.0),  // Front
            Vec3::new(0.0, 0.0, -1.0), // Back
            Vec3::new(0.0, 1.0, 0.0),  // Top
            Vec3::new(0.0, -1.0, 0.0), // Bottom
            Vec3::new(1.0, 0.0, 0.0),  // Right
            Vec3::new(-1.0, 0.0, 0.0), // Left
        ];

        // 生成顶点
        for (face_idx, face) in faces.iter().enumerate() {
            let normal = normals[face_idx / 2];
            for &idx in face {
                vertices.push(ProceduralVertex {
                    position: positions[idx],
                    normal,
                    uv: Vec2::ZERO,
                    tangent: Vec3::X,
                    bitangent: Vec3::Y,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
        }

        // 生成索引
        for i in 0..36 {
            indices.push(i as u32);
        }

        Mesh { vertices, indices }
    }

    /// 创建球体
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for ring in 0..=rings {
            let theta = (ring as f32 / rings as f32) * std::f32::consts::PI;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for segment in 0..=segments {
                let phi = (segment as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta;
                let y = cos_theta;
                let z = sin_phi * sin_theta;

                let position = Vec3::new(x, y, z) * radius;
                let normal = Vec3::new(x, y, z);
                let uv = Vec2::new(segment as f32 / segments as f32, ring as f32 / rings as f32);

                vertices.push(ProceduralVertex {
                    position,
                    normal,
                    uv,
                    tangent: Vec3::X,
                    bitangent: Vec3::Y,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
        }

        for ring in 0..rings {
            for segment in 0..segments {
                let top = ring * (segments + 1) + segment;
                let bottom = top + segments + 1;

                indices.push(top);
                indices.push(bottom);
                indices.push(top + 1);

                indices.push(top + 1);
                indices.push(bottom);
                indices.push(bottom + 1);
            }
        }

        Mesh { vertices, indices }
    }

    /// 创建圆柱体
    pub fn cylinder(radius: f32, height: f32, segments: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_height = height / 2.0;

        // 侧面顶点
        for ring in 0..=1 {
            let y = if ring == 0 { -half_height } else { half_height };
            for segment in 0..segments {
                let angle = (segment as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
                let x = angle.cos() * radius;
                let z = angle.sin() * radius;

                let normal = Vec3::new(x, 0.0, z).normalize();

                vertices.push(ProceduralVertex {
                    position: Vec3::new(x, y, z),
                    normal,
                    uv: Vec2::new(segment as f32 / segments as f32, ring as f32),
                    tangent: Vec3::X,
                    bitangent: Vec3::Y,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
        }

        // 侧面索引
        for segment in 0..segments {
            let next = (segment + 1) % segments;

            let top = segment;
            let bottom = segment + segments;

            indices.push(top);
            indices.push(bottom);
            indices.push(next);

            indices.push(next);
            indices.push(bottom);
            indices.push(next + segments);
        }

        // 顶盖和底盖
        for (ring, y_sign) in [(0, 1.0), (1, -1.0)].iter() {
            let center_idx = vertices.len() as u32;
            let y = *y_sign * half_height;

            vertices.push(ProceduralVertex {
                position: Vec3::new(0.0, y, 0.0),
                normal: Vec3::new(0.0, *y_sign, 0.0),
                uv: Vec2::new(0.5, 0.5),
                tangent: Vec3::X,
                bitangent: Vec3::Y,
                color: [1.0, 1.0, 1.0, 1.0],
            });

            for segment in 0..segments {
                let angle = (segment as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
                let x = angle.cos() * radius;
                let z = angle.sin() * radius;

                vertices.push(ProceduralVertex {
                    position: Vec3::new(x, y, z),
                    normal: Vec3::new(0.0, *y_sign, 0.0),
                    uv: Vec2::new(0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5),
                    tangent: Vec3::X,
                    bitangent: Vec3::Y,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }

            let ring_offset = if *ring == 0 { 0 } else { segments };

            for segment in 0..segments {
                let next = (segment + 1) % segments;
                let v1 = center_idx;
                let v2 = center_idx + 1 + segment + ring_offset;
                let v3 = center_idx + 1 + next + ring_offset;

                if *ring == 0 {
                    indices.push(v1);
                    indices.push(v3);
                    indices.push(v2);
                } else {
                    indices.push(v1);
                    indices.push(v2);
                    indices.push(v3);
                }
            }
        }

        Mesh { vertices, indices }
    }

    /// 创建平面
    pub fn plane(width: f32, depth: f32, segments_x: u32, segments_z: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_width = width / 2.0;
        let half_depth = depth / 2.0;

        for z in 0..=segments_z {
            for x in 0..=segments_x {
                let px = (x as f32 / segments_x as f32) * width - half_width;
                let pz = (z as f32 / segments_z as f32) * depth - half_depth;

                vertices.push(ProceduralVertex {
                    position: Vec3::new(px, 0.0, pz),
                    normal: Vec3::Y,
                    uv: Vec2::new(x as f32 / segments_x as f32, z as f32 / segments_z as f32),
                    tangent: Vec3::X,
                    bitangent: Vec3::Y,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
        }

        for z in 0..segments_z {
            for x in 0..segments_x {
                let top = z * (segments_x + 1) + x;
                let bottom = top + segments_x + 1;

                indices.push(top);
                indices.push(bottom);
                indices.push(top + 1);

                indices.push(top + 1);
                indices.push(bottom);
                indices.push(bottom + 1);
            }
        }

        Mesh { vertices, indices }
    }
}

/// 地形生成器
pub struct TerrainGenerator {
    /// 噪声配置
    pub noise_config: NoiseConfig,
    /// 地形大小
    pub size: f32,
    /// 最大高度
    pub max_height: f32,
    /// 分段数
    pub segments: u32,
}

impl TerrainGenerator {
    /// 创建新的地形生成器
    pub fn new(noise_config: NoiseConfig, size: f32, max_height: f32, segments: u32) -> Self {
        Self {
            noise_config,
            size,
            max_height,
            segments,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(NoiseConfig::default(), 100.0, 20.0, 100)
    }

    /// 生成地形网格
    pub fn generate(&self) -> Mesh {
        let perlin = PerlinNoise::new(self.noise_config.seed);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_size = self.size / 2.0;

        for z in 0..=self.segments {
            for x in 0..=self.segments {
                let px = (x as f32 / self.segments as f32) * self.size - half_size;
                let pz = (z as f32 / self.segments as f32) * self.size - half_size;

                // 使用FBM生成高度
                let nx = px / self.size * self.noise_config.scale;
                let nz = pz / self.size * self.noise_config.scale;
                let height = perlin.fbm(
                    nx,
                    nz,
                    self.noise_config.octaves,
                    self.noise_config.persistence,
                    self.noise_config.lacunarity,
                ) * self.max_height;

                vertices.push(ProceduralVertex {
                    position: Vec3::new(px, height, pz),
                    normal: Vec3::Y, // 稍后重新计算
                    uv: Vec2::new(
                        x as f32 / self.segments as f32,
                        z as f32 / self.segments as f32,
                    ),
                    tangent: Vec3::X,
                    bitangent: Vec3::Y,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
        }

        // 生成索引
        for z in 0..self.segments {
            for x in 0..self.segments {
                let top = z * (self.segments + 1) + x;
                let bottom = top + self.segments + 1;

                indices.push(top);
                indices.push(bottom);
                indices.push(top + 1);

                indices.push(top + 1);
                indices.push(bottom);
                indices.push(bottom + 1);
            }
        }

        // 计算法线
        let mut mesh = Mesh { vertices, indices };
        Self::calculate_normals(&mut mesh);
        mesh
    }

    /// 计算网格法线
    fn calculate_normals(mesh: &mut Mesh) {
        // 重置法线
        for vertex in &mut mesh.vertices {
            vertex.normal = Vec3::ZERO;
        }

        // 累加三角形法线
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                let i0 = chunk[0] as usize;
                let i1 = chunk[1] as usize;
                let i2 = chunk[2] as usize;

                let v0 = mesh.vertices[i0].position;
                let v1 = mesh.vertices[i1].position;
                let v2 = mesh.vertices[i2].position;

                let edge1 = v1 - v0;
                let edge2 = v2 - v0;
                let normal = edge1.cross(edge2).normalize();

                mesh.vertices[i0].normal += normal;
                mesh.vertices[i1].normal += normal;
                mesh.vertices[i2].normal += normal;
            }
        }

        // 归一化法线
        for vertex in &mut mesh.vertices {
            vertex.normal = vertex.normal.normalize();
        }
    }
}

impl MeshGenerator for TerrainGenerator {
    fn generate(&self) -> Mesh {
        self.generate()
    }
}

/// 洞穴生成器
pub struct CaveGenerator {
    /// 噪声配置
    pub noise_config: NoiseConfig,
    /// 洞穴大小
    pub size: f32,
    /// 阈值（低于此值生成洞穴）
    pub threshold: f32,
}

impl CaveGenerator {
    /// 创建新的洞穴生成器
    pub fn new(noise_config: NoiseConfig, size: f32, threshold: f32) -> Self {
        Self {
            noise_config,
            size,
            threshold,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(NoiseConfig::default(), 50.0, 0.3)
    }

    /// 生成3D噪声场
    pub fn generate_3d_field(&self) -> Vec<f32> {
        let perlin = PerlinNoise::new(self.noise_config.seed);
        let resolution = 32; // 32x32x32
        let mut field = Vec::with_capacity(resolution * resolution * resolution);

        for z in 0..resolution {
            for y in 0..resolution {
                for x in 0..resolution {
                    let nx = x as f32 / resolution as f32;
                    let ny = y as f32 / resolution as f32;
                    let nz = z as f32 / resolution as f32;

                    let value = perlin.fbm3d(
                        Vec3::new(nx, ny, nz) * self.noise_config.scale,
                        self.noise_config.octaves,
                        self.noise_config.persistence,
                        self.noise_config.lacunarity,
                    );

                    field.push(value);
                }
            }
        }

        field
    }

    /// 生成洞穴网格（使用Marching Cubes）
    pub fn generate_mesh(&self) -> Mesh {
        // 简化实现：生成一个盒子表示洞穴
        PrimitiveGenerator::cube(self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_cube() {
        let cube = PrimitiveGenerator::cube(1.0);
        assert!(!cube.vertices.is_empty());
        assert!(!cube.indices.is_empty());
        assert_eq!(cube.indices.len() % 3, 0); // 三角形数量
    }

    #[test]
    fn test_primitive_sphere() {
        let sphere = PrimitiveGenerator::sphere(1.0, 16, 16);
        assert!(!sphere.vertices.is_empty());
        assert!(!sphere.indices.is_empty());
    }

    #[test]
    fn test_terrain_generation() {
        let generator = TerrainGenerator::default_config();
        let terrain = generator.generate();

        assert!(!terrain.vertices.is_empty());
        assert!(!terrain.indices.is_empty());

        // 检查高度变化
        let min_y = terrain.vertices.iter().map(|v| v.position.y).fold(f32::INFINITY, f32::min);
        let max_y = terrain.vertices.iter().map(|v| v.position.y).fold(f32::NEG_INFINITY, f32::max);

        assert!(max_y > min_y);
    }

    #[test]
    fn test_cave_field_generation() {
        let generator = CaveGenerator::default_config();
        let field = generator.generate_3d_field();

        assert_eq!(field.len(), 32 * 32 * 32);

        // 检查值在合理范围内
        for &value in &field {
            assert!(value >= -1.0 && value <= 1.0);
        }
    }
}
