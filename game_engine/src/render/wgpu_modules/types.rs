//! WGPU 公共类型定义
//!
//! 包含渲染所需的各种数据结构定义。

use crate::impl_default;

/// 渲染实例数据
///
/// 用于实例化渲染的每个实例的数据。
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    /// 位置 (x, y)
    pub pos: [f32; 2],
    /// 缩放 (sx, sy)
    pub scale: [f32; 2],
    /// 旋转角度（弧度）
    pub rot: f32,
    /// 目标 ID
    pub target: u32,
    /// 块 ID（用于批量更新）
    pub chunk: u32,
    /// 颜色 (r, g, b, a)
    pub color: [f32; 4],
    /// UV 偏移
    pub uv_offset: [f32; 2],
    /// UV 缩放
    pub uv_scale: [f32; 2],
    /// 图层深度
    pub layer: f32,
    /// 纹理索引
    pub tex_index: u32,
    /// 法线纹理索引
    pub normal_tex_index: u32,
    /// MSDF 标志
    pub msdf: f32,
    /// 像素范围
    pub px_range: f32,
}

impl Instance {
    /// 比较两个实例是否相等（用于脏检测）
    #[inline]
    pub fn equals(&self, other: &Instance) -> bool {
        self.pos == other.pos
            && self.scale == other.scale
            && self.rot == other.rot
            && self.target == other.target
            && self.chunk == other.chunk
            && self.color == other.color
            && self.uv_offset == other.uv_offset
            && self.uv_scale == other.uv_scale
            && self.layer == other.layer
            && self.tex_index == other.tex_index
            && self.normal_tex_index == other.normal_tex_index
            && self.msdf == other.msdf
            && self.px_range == other.px_range
    }

    /// 获取实例的顶点缓冲区布局
    pub fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 44,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 52,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 60,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 68,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 72,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 76,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 13,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

impl_default!(Instance {
    pos: [0.0, 0.0],
    scale: [1.0, 1.0],
    rot: 0.0,
    target: 0,
    chunk: 0,
    color: [1.0, 1.0, 1.0, 1.0],
    uv_offset: [0.0, 0.0],
    uv_scale: [1.0, 1.0],
    layer: 0.0,
    tex_index: 0,
    normal_tex_index: 0,
    msdf: 0.0,
    px_range: 0.0,
});

/// UI 实例数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiInstance {
    /// 位置 (x, y)
    pub pos: [f32; 2],
    /// 尺寸 (width, height)
    pub size: [f32; 2],
    /// 圆角半径
    pub radius: f32,
    /// 描边宽度
    pub stroke_width: f32,
    /// 填充颜色 (r, g, b, a)
    pub color: [f32; 4],
    /// 描边颜色 (r, g, b, a)
    pub stroke_color: [f32; 4],
    /// 旋转角度
    pub rotation: f32,
}

impl_default!(UiInstance {
    pos: [0.0, 0.0],
    size: [100.0, 100.0],
    radius: 0.0,
    stroke_width: 0.0,
    color: [1.0, 1.0, 1.0, 1.0],
    stroke_color: [0.0, 0.0, 0.0, 1.0],
    rotation: 0.0,
});

/// 基础顶点数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// 位置 (x, y)
    pub pos: [f32; 2],
}

impl Vertex {
    /// 获取顶点缓冲区布局
    pub fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }

    /// 创建四边形顶点数据
    pub fn quad() -> [Vertex; 6] {
        [
            Vertex { pos: [-0.5, -0.5] },
            Vertex { pos: [0.5, -0.5] },
            Vertex { pos: [0.5, 0.5] },
            Vertex { pos: [-0.5, -0.5] },
            Vertex { pos: [0.5, 0.5] },
            Vertex { pos: [-0.5, 0.5] },
        ]
    }
}

/// 屏幕 Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniform {
    /// 屏幕尺寸 (width, height)
    pub screen_size: [f32; 2],
    /// 缩放因子
    pub scale_factor: f32,
    /// 填充对齐
    pub _pad: f32,
}

/// 3D Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms3D {
    /// 视图投影矩阵
    pub view_proj: [[f32; 4]; 4],
}

/// 模型 Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    /// 模型矩阵
    pub model: [[f32; 4]; 4],
    /// 颜色
    pub color: [f32; 4],
    /// 填充 1
    pub _pad1: [f32; 32],
    /// 填充 2
    pub _pad2: [f32; 12],
}

/// GPU 点光源数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuPointLight {
    /// 位置 (x, y)
    pub pos: [f32; 2],
    /// 颜色 (r, g, b)
    pub color: [f32; 3],
    /// 半径
    pub radius: f32,
    /// 强度
    pub intensity: f32,
    /// 衰减
    pub falloff: f32,
    /// 填充对齐
    pub _pad: [f32; 2],
}

impl_default!(GpuPointLight {
    pos: [0.0, 0.0],
    color: [1.0, 1.0, 1.0],
    radius: 100.0,
    intensity: 1.0,
    falloff: 1.0,
    _pad: [0.0, 0.0],
});

/// 绘制组
pub struct DrawGroup {
    /// 起始索引
    pub start: u32,
    /// 结束索引
    pub end: u32,
    /// 纹理索引
    pub tex_idx: usize,
    /// 图层
    pub layer: f32,
    /// 裁剪区域
    pub scissor: Option<[u32; 4]>,
}

impl DrawGroup {
    /// 创建新的绘制组
    pub fn new(start: u32, end: u32, tex_idx: usize, layer: f32) -> Self {
        Self {
            start,
            end,
            tex_idx,
            layer,
            scissor: None,
        }
    }

    /// 设置裁剪区域
    pub fn with_scissor(mut self, scissor: Option<[u32; 4]>) -> Self {
        self.scissor = scissor;
        self
    }
}
