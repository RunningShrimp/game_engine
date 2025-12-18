use crate::impl_default;
use glam::{Vec3, Vec4};

/// PBR材质参数
#[derive(Clone, Debug, PartialEq)]
pub struct PbrMaterial {
    /// 基础颜色 (RGB + Alpha)
    pub base_color: Vec4,
    /// 金属度 (0.0 = 非金属, 1.0 = 金属)
    pub metallic: f32,
    /// 粗糙度 (0.0 = 光滑, 1.0 = 粗糙)
    pub roughness: f32,
    /// 环境光遮蔽
    pub ambient_occlusion: f32,
    /// 自发光颜色
    pub emissive: Vec3,
    /// 法线贴图强度
    pub normal_scale: f32,
    /// UV 偏移 (KHR_texture_transform)
    pub uv_offset: [f32; 2],
    /// UV 缩放 (KHR_texture_transform)
    pub uv_scale: [f32; 2],
    /// UV 旋转 (弧度, KHR_texture_transform)
    pub uv_rotation: f32,
    /// 清漆强度
    pub clearcoat: f32,
    /// 清漆粗糙度
    pub clearcoat_roughness: f32,
    /// 各向异性强度
    pub anisotropy: f32,
    /// 各向异性方向
    pub anisotropy_direction: [f32; 2],
}

impl_default!(PbrMaterial {
    base_color: Vec4::ONE,
    metallic: 0.0,
    roughness: 0.5,
    ambient_occlusion: 1.0,
    emissive: Vec3::ZERO,
    normal_scale: 1.0,
    uv_offset: [0.0, 0.0],
    uv_scale: [1.0, 1.0],
    uv_rotation: 0.0,
    clearcoat: 0.0,
    clearcoat_roughness: 0.5,
    anisotropy: 0.0,
    anisotropy_direction: [1.0, 0.0],
});

/// PBR纹理集
#[derive(Clone, Debug, Default)]
pub struct PbrTextures {
    /// 基础颜色贴图
    pub base_color_texture: Option<u32>,
    /// 金属度/粗糙度贴图 (R通道=金属度, G通道=粗糙度)
    pub metallic_roughness_texture: Option<u32>,
    /// 法线贴图
    pub normal_texture: Option<u32>,
    /// 环境光遮蔽贴图
    pub ao_texture: Option<u32>,
    /// 自发光贴图
    pub emissive_texture: Option<u32>,
}

/// 完整的PBR材质,包含参数和纹理
#[derive(Clone, Debug, Default)]
pub struct PbrMaterialFull {
    pub material: PbrMaterial,
    pub textures: PbrTextures,
}

/// 点光源
#[derive(Clone, Debug)]
pub struct PointLight3D {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
}

impl_default!(PointLight3D {
    position: Vec3::ZERO,
    color: Vec3::ONE,
    intensity: 1.0,
    radius: 10.0,
});

/// 方向光
#[derive(Clone, Debug)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl_default!(DirectionalLight {
    direction: Vec3::new(0.0, -1.0, 0.0),
    color: Vec3::ONE,
    intensity: 1.0,
});

/// 聚光灯
#[derive(Clone, Debug)]
pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub inner_cutoff: f32, // 内圆锥角度 (弧度)
    pub outer_cutoff: f32, // 外圆锥角度 (弧度)
    pub radius: f32,
}

impl_default!(SpotLight {
    position: Vec3::ZERO,
    direction: Vec3::new(0.0, -1.0, 0.0),
    color: Vec3::ONE,
    intensity: 1.0,
    inner_cutoff: 0.5,
    outer_cutoff: 0.7,
    radius: 10.0,
});
