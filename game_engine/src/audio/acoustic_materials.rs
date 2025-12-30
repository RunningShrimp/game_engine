// 扩展声学材质库
//
// 提供更多真实的声学材质预设

use std::collections::HashMap;

/// 声学材质属性
#[derive(Debug, Clone, Copy)]
pub struct AcousticMaterialProperties {
    /// 传输系数 (0.0 = 完全反射, 1.0 = 完全传输)
    pub transmission_coefficient: f32,
    /// 吸收系数 (0.0 = 完全反射, 1.0 = 完全吸收)
    pub absorption_coefficient: f32,
    /// 扩散系数 (0.0 = 镜面反射, 1.0 = 完全扩散)
    pub diffusion_coefficient: f32,
    /// 频率依赖性 (低频/高频传输比)
    pub frequency_dependency: f32,
}

impl AcousticMaterialProperties {
    pub fn new(
        transmission: f32,
        absorption: f32,
        diffusion: f32,
        freq_dep: f32,
    ) -> Self {
        Self {
            transmission_coefficient: transmission,
            absorption_coefficient: absorption,
            diffusion_coefficient: diffusion,
            frequency_dependency: freq_dep,
        }
    }
}

/// 预设材质库
pub struct MaterialLibrary {
    materials: HashMap<String, AcousticMaterialProperties>,
}

impl MaterialLibrary {
    /// 创建材质库并加载所有预设
    pub fn new() -> Self {
        let mut materials = HashMap::new();

        // === 基础材质 ===

        materials.insert("air".to_string(), AcousticMaterialProperties::new(
            1.0,   // 完全传输
            0.0,   // 无吸收
            0.0,   // 无扩散
            1.0,   // 频率无关
        ));

        materials.insert("concrete".to_string(), AcousticMaterialProperties::new(
            0.01,  // 几乎不传输
            0.02,  // 很少吸收
            0.1,   // 少量扩散
            0.5,   // 低频传输更多
        ));

        materials.insert("wood".to_string(), AcousticMaterialProperties::new(
            0.1,   // 部分传输
            0.15,  // 中等吸收
            0.2,   // 少量扩散
            0.7,   // 中等频率依赖
        ));

        materials.insert("glass".to_string(), AcousticMaterialProperties::new(
            0.3,   // 较好传输
            0.05,  // 很少吸收
            0.05,  // 几乎镜面反射
            0.9,   // 高频传输更多
        ));

        materials.insert("metal".to_string(), AcousticMaterialProperties::new(
            0.001, // 几乎不传输
            0.01,  // 极少吸收
            0.02,  // 镜面反射
            0.2,   // 低频传输略多
        ));

        // === 织物和软材料 ===

        materials.insert("carpet".to_string(), AcousticMaterialProperties::new(
            0.05,  // 几乎不传输
            0.6,   // 高吸收
            0.3,   // 中等扩散
            0.6,   // 中高频吸收更多
        ));

        materials.insert("curtain_heavy".to_string(), AcousticMaterialProperties::new(
            0.02,
            0.7,   // 很高吸收
            0.4,   // 较高扩散
            0.7,   // 高频吸收更多
        ));

        materials.insert("curtain_light".to_string(), AcousticMaterialProperties::new(
            0.05,
            0.4,   // 中等吸收
            0.3,
            0.65,
        ));

        materials.insert("upholstery".to_string(), AcousticMaterialProperties::new(
            0.03,
            0.5,   // 高吸收
            0.35,
            0.65,
        ));

        materials.insert("foam".to_string(), AcousticMaterialProperties::new(
            0.01,
            0.8,   // 极高吸收
            0.5,   // 高扩散
            0.8,   // 高频吸收更多
        ));

        // === 建筑材料 ===

        materials.insert("brick".to_string(), AcousticMaterialProperties::new(
            0.02,
            0.03,  // 很少吸收
            0.15,  // 中等扩散
            0.4,
        ));

        materials.insert("drywall".to_string(), AcousticMaterialProperties::new(
            0.05,
            0.05,
            0.1,
            0.5,
        ));

        materials.insert("plaster".to_string(), AcousticMaterialProperties::new(
            0.03,
            0.02,
            0.08,
            0.45,
        ));

        materials.insert("tile_ceramic".to_string(), AcousticMaterialProperties::new(
            0.02,
            0.01,  // 极少吸收
            0.05,  // 镜面反射
            0.6,   // 高频反射更多
        ));

        materials.insert("tile_marble".to_string(), AcousticMaterialProperties::new(
            0.015,
            0.005, // 几乎无吸收
            0.03,
            0.55,
        ));

        materials.insert("wood_floor".to_string(), AcousticMaterialProperties::new(
            0.08,
            0.1,
            0.15,
            0.65,
        ));

        materials.insert("parquet".to_string(), AcousticMaterialProperties::new(
            0.06,
            0.08,
            0.12,
            0.6,
        ));

        // === 自然材料 ===

        materials.insert("grass".to_string(), AcousticMaterialProperties::new(
            0.1,
            0.3,   // 中等吸收
            0.4,   // 较高扩散
            0.7,
        ));

        materials.insert("snow".to_string(), AcousticMaterialProperties::new(
            0.05,
            0.4,   // 高吸收
            0.5,   // 高扩散
            0.75,
        ));

        materials.insert("sand".to_string(), AcousticMaterialProperties::new(
            0.08,
            0.2,
            0.35,
            0.65,
        ));

        materials.insert("gravel".to_string(), AcousticMaterialProperties::new(
            0.05,
            0.35,  // 高吸收
            0.45,  // 高扩散
            0.7,
        ));

        materials.insert("water".to_string(), AcousticMaterialProperties::new(
            0.0,   // 完全反射
            0.0,
            0.02,
            0.3,
        ));

        // === 特殊材料 ===

        materials.insert("fiberglass".to_string(), AcousticMaterialProperties::new(
            0.01,
            0.9,   // 极高吸收
            0.6,   // 高扩散
            0.85,  // 高频吸收更多
        ));

        materials.insert("mineral_wool".to_string(), AcousticMaterialProperties::new(
            0.01,
            0.85,
            0.55,
            0.82,
        ));

        materials.insert("acoustic_panel".to_string(), AcousticMaterialProperties::new(
            0.01,
            0.75,
            0.5,
            0.8,
        ));

        materials.insert("perforated_panel".to_string(), AcousticMaterialProperties::new(
            0.1,
            0.5,   // 中高吸收
            0.3,
            0.75,
        ));

        materials.insert("membrane_absorber".to_string(), AcousticMaterialProperties::new(
            0.02,
            0.4,   // 低频吸收
            0.15,
            0.3,   // 低频吸收更多
        ));

        materials.insert("helmholtz_resonator".to_string(), AcousticMaterialProperties::new(
            0.01,
            0.7,   // 特定频率高吸收
            0.2,
            0.25,  // 窄频带
        ));

        Self { materials }
    }

    /// 获取材质
    pub fn get(&self, name: &str) -> Option<&AcousticMaterialProperties> {
        self.materials.get(name)
    }

    /// 获取所有材质名称
    pub fn list_materials(&self) -> Vec<&str> {
        self.materials.keys().map(|s| s.as_str()).collect()
    }

    /// 按类别获取材质
    pub fn get_by_category(&self, category: MaterialCategory) -> Vec<&str> {
        let category_names = match category {
            MaterialCategory::Basic => &["air", "concrete", "wood", "glass", "metal"][..],
            MaterialCategory::Fabric => &["carpet", "curtain_heavy", "curtain_light", "upholstery", "foam"][..],
            MaterialCategory::Building => &["brick", "drywall", "plaster", "tile_ceramic", "tile_marble", "wood_floor", "parquet"][..],
            MaterialCategory::Natural => &["grass", "snow", "sand", "gravel", "water"][..],
            MaterialCategory::Special => &["fiberglass", "mineral_wool", "acoustic_panel", "perforated_panel", "membrane_absorber", "helmholtz_resonator"][..],
        };

        category_names.to_vec()
    }
}

/// 材质类别
#[derive(Debug, Clone, Copy)]
pub enum MaterialCategory {
    Basic,
    Fabric,
    Building,
    Natural,
    Special,
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_library_creation() {
        let library = MaterialLibrary::new();
        assert!(library.get("concrete").is_some());
        assert!(library.get("wood").is_some());
        assert!(library.get("carpet").is_some());
    }

    #[test]
    fn test_material_properties() {
        let library = MaterialLibrary::new();
        let concrete = library.get("concrete").unwrap();

        assert_eq!(concrete.transmission_coefficient, 0.01);
        assert_eq!(concrete.absorption_coefficient, 0.02);
    }

    #[test]
    fn test_category_filtering() {
        let library = MaterialLibrary::new();
        let fabric_materials = library.get_by_category(MaterialCategory::Fabric);

        assert!(fabric_materials.contains(&"carpet"));
        assert!(fabric_materials.contains(&"foam"));
        assert!(!fabric_materials.contains(&"concrete"));
    }

    #[test]
    fn test_all_materials_accessible() {
        let library = MaterialLibrary::new();
        let materials = library.list_materials();

        assert!(materials.len() > 20); // 至少有20种材质

        // 验证关键材质存在
        assert!(materials.contains(&"air"));
        assert!(materials.contains(&"water"));
        assert!(materials.contains(&"fiberglass"));
    }
}
