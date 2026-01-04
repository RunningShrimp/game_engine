//! # Lighting Calculation Trait
//!
//! 统一的光照计算接口 - 为不同光照模型提供公共trait。
//!
//! ## 核心功能
//!
//! 1. **LightSource** - 光源trait
//! 2. **LightingModel** - 光照模型trait
//! 3. **LightCalculator** - 光照计算器trait

use glam::{Vec3, Vec4};

/// 光源类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightType {
    /// 方向光（如太阳光）
    Directional,
    /// 点光源（如灯泡）
    Point,
    /// 聚光灯（如手电筒）
    Spot,
    /// 环境光
    Ambient,
    /// 区域光（如发光面板）
    Area,
    /// 体积光（如体积雾）
    Volumetric,
}

/// 光照表面属性
#[derive(Clone, Debug)]
pub struct SurfaceProperties {
    /// 世界空间位置
    pub position: Vec3,
    /// 世界空间法线
    pub normal: Vec3,
    /// 视线方向（从表面到相机）
    pub view_dir: Vec3,
    /// 基础颜色
    pub base_color: Vec4,
    /// 金属度
    pub metallic: f32,
    /// 粗糙度
    pub roughness: f32,
    /// 环境光遮蔽
    pub ambient_occlusion: f32,
    /// 自发光
    pub emissive: Vec3,
}

impl Default for SurfaceProperties {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::Y,
            view_dir: Vec3::Z,
            base_color: Vec4::ONE,
            metallic: 0.0,
            roughness: 0.5,
            ambient_occlusion: 1.0,
            emissive: Vec3::ZERO,
        }
    }
}

/// 光照计算结果
#[derive(Clone, Debug)]
pub struct LightingResult {
    /// 漫反射贡献
    pub diffuse: Vec3,
    /// 镜面反射贡献
    pub specular: Vec3,
    /// 总光照贡献
    pub total: Vec3,
}

impl Default for LightingResult {
    fn default() -> Self {
        Self {
            diffuse: Vec3::ZERO,
            specular: Vec3::ZERO,
            total: Vec3::ZERO,
        }
    }
}

/// 光源trait - 所有光源都应实现此trait
pub trait LightSource {
    /// 获取光源类型
    fn light_type(&self) -> LightType;

    /// 获取光源颜色
    fn color(&self) -> Vec3;

    /// 获取光源强度
    fn intensity(&self) -> f32;

    /// 计算到表面的光照方向
    fn light_direction(&self, surface: &SurfaceProperties) -> Vec3;

    /// 计算到表面的距离
    fn distance_to_surface(&self, surface: &SurfaceProperties) -> f32;

    /// 计算光照衰减
    fn attenuation(&self, distance: f32) -> f32;

    /// 判断光源是否影响表面
    fn affects_surface(&self, surface: &SurfaceProperties) -> bool {
        let distance_to_surface = self.distance_to_surface(surface);
        distance_to_surface > 0.0 && self.attenuation(distance_to_surface) > 0.0
    }
}

/// 光照模型trait - 定义如何计算光照
pub trait LightingModel {
    /// 计算漫反射贡献
    fn compute_diffuse(
        &self,
        surface: &SurfaceProperties,
        light_dir: Vec3,
        light_color: Vec3,
        light_intensity: f32,
    ) -> Vec3;

    /// 计算镜面反射贡献（使用BRDF）
    fn compute_specular(
        &self,
        surface: &SurfaceProperties,
        light_dir: Vec3,
        light_color: Vec3,
        light_intensity: f32,
    ) -> Vec3;

    /// 计算完整光照（漫反射 + 镜面反射）
    fn compute_lighting(
        &self,
        surface: &SurfaceProperties,
        light: &dyn LightSource,
    ) -> LightingResult {
        let light_dir = light.light_direction(surface);
        let distance = light.distance_to_surface(surface);
        let attenuation = light.attenuation(distance);

        let light_color = light.color();
        let light_intensity = light.intensity() * attenuation;

        let diffuse = self.compute_diffuse(surface, light_dir, light_color, light_intensity);

        let specular = self.compute_specular(surface, light_dir, light_color, light_intensity);

        let total = diffuse + specular;

        LightingResult {
            diffuse,
            specular,
            total,
        }
    }
}

/// Lambertian漫反射模型
#[derive(Clone, Debug, Default)]
pub struct LambertianDiffuse;

impl LightingModel for LambertianDiffuse {
    fn compute_diffuse(
        &self,
        surface: &SurfaceProperties,
        light_dir: Vec3,
        light_color: Vec3,
        light_intensity: f32,
    ) -> Vec3 {
        // N·L (法线·光线方向)
        let n_dot_l = surface.normal.dot(light_dir).max(0.0);

        // 漫反射 = 基础颜色 * 光照颜色 * 强度 * N·L
        let base_rgb = surface.base_color.truncate();
        base_rgb * light_color * light_intensity * n_dot_l
    }

    fn compute_specular(
        &self,
        _surface: &SurfaceProperties,
        _light_dir: Vec3,
        _light_color: Vec3,
        _light_intensity: f32,
    ) -> Vec3 {
        // Lambertian没有镜面反射
        Vec3::ZERO
    }
}

/// Blinn-Phong光照模型
#[derive(Clone, Debug, Default)]
pub struct BlinnPhong;

impl LightingModel for BlinnPhong {
    fn compute_diffuse(
        &self,
        surface: &SurfaceProperties,
        light_dir: Vec3,
        light_color: Vec3,
        light_intensity: f32,
    ) -> Vec3 {
        // Lambertian漫反射
        let n_dot_l = surface.normal.dot(light_dir).max(0.0);
        let base_rgb = surface.base_color.truncate();
        base_rgb * light_color * light_intensity * n_dot_l
    }

    fn compute_specular(
        &self,
        surface: &SurfaceProperties,
        light_dir: Vec3,
        light_color: Vec3,
        light_intensity: f32,
    ) -> Vec3 {
        // Blinn-Phong镜面反射
        let half_vector = (light_dir + surface.view_dir).normalize();
        let n_dot_h = surface.normal.dot(half_vector).max(0.0);

        // 使用粗糙度作为反光度
        let shininess = (2.0 / (surface.roughness + 0.001)).floor();
        let spec_factor = n_dot_h.powf(shininess);

        // 镜面反射强度基于金属度
        let specular_strength = if surface.metallic > 0.5 { 1.0 } else { 0.5 };

        light_color * light_intensity * spec_factor * specular_strength
    }
}

/// PBR光照模型（基于物理的渲染）
#[derive(Clone, Debug, Default)]
pub struct PbrLightingModel;

impl LightingModel for PbrLightingModel {
    fn compute_diffuse(
        &self,
        surface: &SurfaceProperties,
        light_dir: Vec3,
        light_color: Vec3,
        light_intensity: f32,
    ) -> Vec3 {
        // 使用Disney漫反射模型
        let n_dot_l = surface.normal.dot(light_dir).max(0.0);
        let base_rgb = surface.base_color.truncate();

        // 简化的Disney diffuse
        let diffuse_color = base_rgb * (1.0 - surface.metallic);
        diffuse_color * light_color * light_intensity * n_dot_l
    }

    fn compute_specular(
        &self,
        surface: &SurfaceProperties,
        light_dir: Vec3,
        light_color: Vec3,
        light_intensity: f32,
    ) -> Vec3 {
        // 使用简化的Cook-Torrance BRDF
        let half_vector = (light_dir + surface.view_dir).normalize();
        let n_dot_l = surface.normal.dot(light_dir).max(0.0);
        let n_dot_v = surface.normal.dot(surface.view_dir).max(0.001);
        let n_dot_h = surface.normal.dot(half_vector).max(0.0);
        let v_dot_h = surface.view_dir.dot(half_vector).max(0.001);

        // 法线分布函数 (GGX/Trowbridge-Reitz)
        let alpha = surface.roughness * surface.roughness;
        let alpha2 = alpha * alpha;
        let denom = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
        let d = alpha2 / (std::f32::consts::PI * denom * denom);

        // 几何函数 (Schlick-GGX)
        let k = alpha / 2.0;
        let g1 = n_dot_l / (n_dot_l * (1.0 - k) + k);
        let g2 = n_dot_v / (n_dot_v * (1.0 - k) + k);
        let g = g1 * g2;

        // 菲涅尔方程 (Schlick近似)
        let f0 = if surface.metallic > 0.5 {
            Vec3::splat(1.0)
        } else {
            surface.base_color.truncate() * 0.04
        };
        let f = f0 + (Vec3::ONE - f0) * (1.0 - v_dot_h).powf(5.0);

        // 镜面反射BRDF
        let numerator = d * g * f;
        let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
        let spec_brdf = numerator / denominator;

        // 最终镜面反射
        spec_brdf * light_color * light_intensity * n_dot_l
    }
}

/// 光照计算器trait - 提供高级光照计算功能
pub trait LightCalculator {
    /// 计算多光源的光照
    fn compute_multiple_lights(
        &self,
        surface: &SurfaceProperties,
        lights: &[&dyn LightSource],
    ) -> LightingResult {
        let mut total_diffuse = Vec3::ZERO;
        let mut total_specular = Vec3::ZERO;

        for light in lights {
            if light.affects_surface(surface) {
                // 使用PBR模型计算光照
                let model = PbrLightingModel;
                let result = model.compute_lighting(surface, *light);
                total_diffuse += result.diffuse;
                total_specular += result.specular;
            }
        }

        // 应用环境光遮蔽
        total_diffuse *= surface.ambient_occlusion;

        LightingResult {
            diffuse: total_diffuse,
            specular: total_specular,
            total: total_diffuse + total_specular + surface.emissive,
        }
    }

    /// 计算图像基础光照 (IBL)
    fn compute_ibl(
        &self,
        _surface: &SurfaceProperties,
        _irradiance_map: u32,
        _prefilter_map: u32,
        _brdf_lut: u32,
    ) -> Vec3 {
        // 简化版IBL计算
        Vec3::ZERO
    }

    /// 计算全局光照贡献
    fn compute_global_illumination(&self, surface: &SurfaceProperties, gi_intensity: f32) -> Vec3 {
        let base_rgb = surface.base_color.truncate();
        // 简化的GI计算
        base_rgb * gi_intensity * surface.ambient_occlusion
    }
}

/// 默认光照计算器实现
#[derive(Clone, Debug, Default)]
pub struct DefaultLightCalculator;

impl LightCalculator for DefaultLightCalculator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_properties_default() {
        let surface = SurfaceProperties::default();
        assert_eq!(surface.position, Vec3::ZERO);
        assert_eq!(surface.normal, Vec3::Y);
    }

    #[test]
    fn test_lighting_result_default() {
        let result = LightingResult::default();
        assert_eq!(result.diffuse, Vec3::ZERO);
        assert_eq!(result.specular, Vec3::ZERO);
    }

    #[test]
    fn test_lambertian_diffuse() {
        let model = LambertianDiffuse;
        let surface = SurfaceProperties {
            normal: Vec3::Y,
            base_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        let light_dir = Vec3::Y;
        let light_color = Vec3::ONE;
        let light_intensity = 1.0;

        let diffuse = model.compute_diffuse(&surface, light_dir, light_color, light_intensity);
        // 漫反射应该是红色
        assert!(diffuse.x > 0.9);
        assert!(diffuse.y < 0.1);
        assert!(diffuse.z < 0.1);
    }

    #[test]
    fn test_blinn_phong_specular() {
        let model = BlinnPhong;
        let surface = SurfaceProperties {
            normal: Vec3::Y,
            view_dir: Vec3::Z,
            roughness: 0.1, // 光滑表面 = 强镜面反射
            metallic: 1.0,  // 金属表面
            ..Default::default()
        };

        let light_dir = Vec3::new(0.0, 1.0, 0.707);
        let light_color = Vec3::ONE;
        let light_intensity = 1.0;

        let specular = model.compute_specular(&surface, light_dir, light_color, light_intensity);
        // 光滑金属表面应该有强镜面反射
        assert!(specular.length() > 0.0);
    }
}
