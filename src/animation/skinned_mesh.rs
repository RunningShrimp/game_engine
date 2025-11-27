//! 蒙皮网格组件
//!
//! 定义支持骨骼动画的蒙皮网格。

use bevy_ecs::prelude::*;
use std::sync::Arc;

use super::skeleton::Skeleton;
use crate::render::mesh::GpuMesh;

// ============================================================================
// 蒙皮顶点数据
// ============================================================================

/// 蒙皮顶点（包含骨骼权重）
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedVertex3D {
    /// 位置
    pub position: [f32; 3],
    /// 法线
    pub normal: [f32; 3],
    /// 纹理坐标
    pub uv: [f32; 2],
    /// 骨骼索引（最多 4 个）
    pub bone_indices: [u32; 4],
    /// 骨骼权重（最多 4 个，总和为 1.0）
    pub bone_weights: [f32; 4],
}

impl SkinnedVertex3D {
    /// 创建新的蒙皮顶点
    pub fn new(
        position: [f32; 3],
        normal: [f32; 3],
        uv: [f32; 2],
        bone_indices: [u32; 4],
        bone_weights: [f32; 4],
    ) -> Self {
        Self {
            position,
            normal,
            uv,
            bone_indices,
            bone_weights,
        }
    }

    /// 顶点缓冲区布局描述
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SkinnedVertex3D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Bone Indices
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32x4,
                },
                // Bone Weights
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    /// 归一化骨骼权重
    pub fn normalize_weights(&mut self) {
        let sum: f32 = self.bone_weights.iter().sum();
        if sum > 0.0001 {
            let inv_sum = 1.0 / sum;
            for w in &mut self.bone_weights {
                *w *= inv_sum;
            }
        }
    }
}

// ============================================================================
// 蒙皮网格组件
// ============================================================================

/// 蒙皮网格组件
#[derive(Component)]
pub struct SkinnedMesh {
    /// GPU 网格数据
    pub mesh: Arc<GpuMesh>,
    /// 关联的骨骼实体
    pub skeleton_entity: Entity,
    /// 是否启用 GPU 蒙皮
    pub gpu_skinning: bool,
    /// 蒙皮绑定组（包含骨骼矩阵缓冲区）
    pub skin_bind_group: Option<wgpu::BindGroup>,
}

impl SkinnedMesh {
    /// 创建新的蒙皮网格
    pub fn new(mesh: Arc<GpuMesh>, skeleton_entity: Entity) -> Self {
        Self {
            mesh,
            skeleton_entity,
            gpu_skinning: true,
            skin_bind_group: None,
        }
    }

    /// 创建蒙皮绑定组
    pub fn create_bind_group(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        skeleton: &Skeleton,
    ) {
        if let Some(matrix_buffer) = &skeleton.matrix_buffer {
            self.skin_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Skinning Bind Group"),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding(),
                }],
            }));
        }
    }
}

// ============================================================================
// 蒙皮网格 GPU 管线
// ============================================================================

/// 蒙皮渲染管线
pub struct SkinnedMeshPipeline {
    /// 渲染管线
    pub pipeline: wgpu::RenderPipeline,
    /// 蒙皮绑定组布局
    pub skin_bind_group_layout: wgpu::BindGroupLayout,
}

impl SkinnedMeshPipeline {
    /// 创建蒙皮渲染管线
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        material_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // 蒙皮绑定组布局（骨骼矩阵）
        let skin_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Skinning Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // 着色器模块
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Skinned PBR Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("skinned_pbr.wgsl").into()),
        });

        // 管线布局
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skinned Pipeline Layout"),
            bind_group_layouts: &[
                camera_bind_group_layout,      // @group(0) 相机
                material_bind_group_layout,    // @group(1) 材质
                &skin_bind_group_layout,       // @group(2) 骨骼矩阵
            ],
            push_constant_ranges: &[],
        });

        // 渲染管线
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skinned Mesh Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[SkinnedVertex3D::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            skin_bind_group_layout,
        }
    }
}

// ============================================================================
// ECS 系统
// ============================================================================

/// 蒙皮网格更新系统
pub fn skinned_mesh_update_system(
    skinned_query: Query<&SkinnedMesh>,
    skeleton_query: Query<&Skeleton>,
) {
    for skinned in skinned_query.iter() {
        // 获取关联的骨骼
        if let Ok(skeleton) = skeleton_query.get(skinned.skeleton_entity) {
            // 骨骼矩阵已在 skeleton_update_system 中更新
            // 这里可以做一些额外的处理
            let _ = skeleton;
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skinned_vertex_size() {
        // 确保顶点大小符合预期
        assert_eq!(std::mem::size_of::<SkinnedVertex3D>(), 64);
    }

    #[test]
    fn test_normalize_weights() {
        let mut vertex = SkinnedVertex3D::new(
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
            [0, 1, 0, 0],
            [0.5, 0.3, 0.0, 0.0],
        );
        
        vertex.normalize_weights();
        
        let sum: f32 = vertex.bone_weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }
}
