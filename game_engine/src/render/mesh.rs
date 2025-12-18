use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex3D {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex3D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct GpuMesh {
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub index_count: u32,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub vertex_layout: wgpu::VertexBufferLayout<'static>,
}

impl GpuMesh {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex3D], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for v in vertices {
            if v.pos[0] < min[0] {
                min[0] = v.pos[0];
            }
            if v.pos[1] < min[1] {
                min[1] = v.pos[1];
            }
            if v.pos[2] < min[2] {
                min[2] = v.pos[2];
            }
            if v.pos[0] > max[0] {
                max[0] = v.pos[0];
            }
            if v.pos[1] > max[1] {
                max[1] = v.pos[1];
            }
            if v.pos[2] > max[2] {
                max[2] = v.pos[2];
            }
        }

        Self {
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
            index_count: indices.len() as u32,
            aabb_min: min,
            aabb_max: max,
            vertex_layout: Vertex3D::desc(),
        }
    }

    /// 创建一个简单的立方体网格用于测试
    pub fn create_test_cube(device: &wgpu::Device) -> Self {
        let vertices = [
            // Front face
            Vertex3D { pos: [-1.0, -1.0,  1.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [ 1.0, -1.0,  1.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [ 1.0,  1.0,  1.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [-1.0,  1.0,  1.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            // Back face
            Vertex3D { pos: [-1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0], tangent: [-1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [ 1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0], tangent: [-1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [ 1.0,  1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0], tangent: [-1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [-1.0,  1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0], tangent: [-1.0, 0.0, 0.0, 1.0] },
        ];

        let indices: [u32; 36] = [
            // Front
            0, 1, 2, 2, 3, 0,
            // Back
            4, 6, 5, 6, 4, 7,
            // Top
            3, 2, 6, 6, 7, 3,
            // Bottom
            0, 5, 1, 5, 0, 4,
            // Right
            1, 5, 6, 6, 2, 1,
            // Left
            0, 3, 7, 7, 4, 0,
        ];

        Self::new(device, &vertices, &indices)
    }

    /// 创建一个简单的平面网格用于测试
    pub fn create_test_plane(device: &wgpu::Device) -> Self {
        let vertices = [
            Vertex3D { pos: [-1.0, 0.0, -1.0], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [ 1.0, 0.0, -1.0], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [ 1.0, 0.0,  1.0], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex3D { pos: [-1.0, 0.0,  1.0], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        ];

        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        Self::new(device, &vertices, &indices)
    }

    /// 获取顶点数量（估算）
    pub fn vertex_count(&self) -> u32 {
        // 根据缓冲区大小估算
        (self.vertex_buffer.size() / std::mem::size_of::<Vertex3D>() as u64) as u32
    }

    /// 获取 AABB 中心点
    pub fn aabb_center(&self) -> [f32; 3] {
        [
            (self.aabb_min[0] + self.aabb_max[0]) / 2.0,
            (self.aabb_min[1] + self.aabb_max[1]) / 2.0,
            (self.aabb_min[2] + self.aabb_max[2]) / 2.0,
        ]
    }

    /// 获取 AABB 尺寸
    pub fn aabb_size(&self) -> [f32; 3] {
        [
            self.aabb_max[0] - self.aabb_min[0],
            self.aabb_max[1] - self.aabb_min[1],
            self.aabb_max[2] - self.aabb_min[2],
        ]
    }
}

/// 用于测试的 Mock GpuMesh 工厂
/// 
/// 提供创建测试用网格的辅助函数，无需手动构建顶点数据
#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// 测试辅助结构，用于在没有 wgpu 设备的情况下进行测试
    pub struct MockMeshData {
        pub vertex_count: u32,
        pub index_count: u32,
        pub aabb_min: [f32; 3],
        pub aabb_max: [f32; 3],
    }

    impl MockMeshData {
        /// 创建模拟的立方体数据
        pub fn cube() -> Self {
            Self {
                vertex_count: 8,
                index_count: 36,
                aabb_min: [-1.0, -1.0, -1.0],
                aabb_max: [1.0, 1.0, 1.0],
            }
        }

        /// 创建模拟的平面数据
        pub fn plane() -> Self {
            Self {
                vertex_count: 4,
                index_count: 6,
                aabb_min: [-1.0, 0.0, -1.0],
                aabb_max: [1.0, 0.0, 1.0],
            }
        }

        /// 创建自定义尺寸的模拟数据
        pub fn custom(vertex_count: u32, index_count: u32) -> Self {
            Self {
                vertex_count,
                index_count,
                aabb_min: [-1.0, -1.0, -1.0],
                aabb_max: [1.0, 1.0, 1.0],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_helpers::*;

    #[test]
    fn test_mock_mesh_data_cube() {
        let cube = MockMeshData::cube();
        assert_eq!(cube.vertex_count, 8);
        assert_eq!(cube.index_count, 36);
        assert_eq!(cube.aabb_min, [-1.0, -1.0, -1.0]);
        assert_eq!(cube.aabb_max, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_mock_mesh_data_plane() {
        let plane = MockMeshData::plane();
        assert_eq!(plane.vertex_count, 4);
        assert_eq!(plane.index_count, 6);
    }

    #[test]
    fn test_mock_mesh_data_custom() {
        let custom = MockMeshData::custom(100, 300);
        assert_eq!(custom.vertex_count, 100);
        assert_eq!(custom.index_count, 300);
    }

    #[test]
    fn test_vertex3d_layout() {
        let layout = Vertex3D::desc();
        assert_eq!(layout.array_stride, std::mem::size_of::<Vertex3D>() as u64);
        assert_eq!(layout.attributes.len(), 4);
    }
}
