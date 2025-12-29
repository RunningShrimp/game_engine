use crate::impl_default;
use bevy_ecs::prelude::Component;
use glam::{Vec3, Vec4};

/// PBR材质参数
#[derive(Clone, Debug, PartialEq, Component)]
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
#[derive(Clone, Debug, Default, Component)]
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
#[derive(Clone, Debug, Default, Component)]
pub struct PbrMaterialFull {
    pub material: PbrMaterial,
    pub textures: PbrTextures,
}

/// 点光源
#[derive(Clone, Debug, Component)]
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
#[derive(Clone, Debug, Component)]
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
#[derive(Clone, Debug, Component)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbr_material_default() {
        let material = PbrMaterial::default();
        assert_eq!(material.base_color, Vec4::ONE);
        assert_eq!(material.metallic, 0.0);
        assert_eq!(material.roughness, 0.5);
        assert_eq!(material.ambient_occlusion, 1.0);
        assert_eq!(material.emissive, Vec3::ZERO);
        assert_eq!(material.normal_scale, 1.0);
    }

    #[test]
    fn test_pbr_material_clone() {
        let material = PbrMaterial {
            base_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            metallic: 0.8,
            roughness: 0.2,
            ..Default::default()
        };
        let cloned = material.clone();
        assert_eq!(cloned.base_color, material.base_color);
        assert_eq!(cloned.metallic, material.metallic);
        assert_eq!(cloned.roughness, material.roughness);
    }

    #[test]
    fn test_pbr_textures_default() {
        let textures = PbrTextures::default();
        assert!(textures.base_color_texture.is_none());
        assert!(textures.metallic_roughness_texture.is_none());
        assert!(textures.normal_texture.is_none());
    }

    #[test]
    fn test_pbr_textures_with_textures() {
        let textures = PbrTextures {
            base_color_texture: Some(1),
            metallic_roughness_texture: Some(2),
            normal_texture: Some(3),
            ao_texture: Some(4),
            emissive_texture: Some(5),
        };
        assert_eq!(textures.base_color_texture, Some(1));
        assert_eq!(textures.metallic_roughness_texture, Some(2));
    }

    #[test]
    fn test_pbr_material_full_default() {
        let full = PbrMaterialFull::default();
        assert_eq!(full.material.base_color, Vec4::ONE);
        assert!(full.textures.base_color_texture.is_none());
    }

    #[test]
    fn test_point_light_default() {
        let light = PointLight3D::default();
        assert_eq!(light.position, Vec3::ZERO);
        assert_eq!(light.color, Vec3::ONE);
        assert_eq!(light.intensity, 1.0);
        assert_eq!(light.radius, 10.0);
    }

    #[test]
    fn test_point_light_custom() {
        let light = PointLight3D {
            position: Vec3::new(1.0, 2.0, 3.0),
            color: Vec3::new(1.0, 0.0, 0.0),
            intensity: 5.0,
            radius: 20.0,
        };
        assert_eq!(light.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(light.intensity, 5.0);
    }

    #[test]
    fn test_directional_light_default() {
        let light = DirectionalLight::default();
        assert_eq!(light.direction, Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(light.color, Vec3::ONE);
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_spot_light_default() {
        let light = SpotLight::default();
        assert_eq!(light.position, Vec3::ZERO);
        assert_eq!(light.direction, Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(light.inner_cutoff, 0.5);
        assert_eq!(light.outer_cutoff, 0.7);
    }

    #[test]
    fn test_uv_transform_defaults() {
        let material = PbrMaterial::default();
        assert_eq!(material.uv_offset, [0.0, 0.0]);
        assert_eq!(material.uv_scale, [1.0, 1.0]);
        assert_eq!(material.uv_rotation, 0.0);
    }

    #[test]
    fn test_clearcoat_defaults() {
        let material = PbrMaterial::default();
        assert_eq!(material.clearcoat, 0.0);
        assert_eq!(material.clearcoat_roughness, 0.5);
    }

    #[test]
    fn test_anisotropy_defaults() {
        let material = PbrMaterial::default();
        assert_eq!(material.anisotropy, 0.0);
        assert_eq!(material.anisotropy_direction, [1.0, 0.0]);
    }
}
