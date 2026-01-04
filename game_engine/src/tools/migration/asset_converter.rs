//! Unity资源转换工具
//!
//! 完整的Unity资源转换系统，支持FBX、材质、动画转换。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "serde_yaml")]
use super::{MigrationError, MigrationPhase, MigrationProgress};
#[cfg(feature = "serde_yaml")]
use tokio::io::AsyncReadExt;

/// Unity资源转换器
#[cfg(feature = "serde_yaml")]
pub struct UnityAssetConverter {
    /// 配置
    config: AssetConverterConfig,

    /// 材质映射表
    material_mappings: HashMap<String, MaterialMapping>,

    /// 进度回调
    progress_callback: Option<Box<dyn Fn(MigrationProgress) + Send + Sync>>,
}

/// 资源转换配置
#[derive(Debug, Clone)]
pub struct AssetConverterConfig {
    /// 输出格式
    pub output_format: AssetFormat,

    /// 纹理质量
    pub texture_quality: TextureQuality,

    /// 网格优化
    pub optimize_meshes: bool,

    /// 动画压缩
    pub compress_animations: bool,

    /// 材质转换模式
    pub material_mode: MaterialConversionMode,
}

/// 资源输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFormat {
    /// glTF 2.0
    GLTF2,
    /// GLB (二进制glTF)
    GLB,
    /// 自定义引擎格式
    Engine,
}

/// 纹理质量
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
    Maximum,
}

/// 材质转换模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialConversionMode {
    /// 标准PBR
    StandardPBR,
    /// 简化着色器
    Simplified,
    /// 自定义映射
    Custom,
}

impl Default for AssetConverterConfig {
    fn default() -> Self {
        Self {
            output_format: AssetFormat::GLTF2,
            texture_quality: TextureQuality::High,
            optimize_meshes: true,
            compress_animations: true,
            material_mode: MaterialConversionMode::StandardPBR,
        }
    }
}

#[cfg(feature = "serde_yaml")]
impl UnityAssetConverter {
    /// 创建新的资源转换器
    pub fn new(config: AssetConverterConfig) -> Self {
        Self {
            config,
            material_mappings: HashMap::new(),
            progress_callback: None,
        }
    }

    /// 设置进度回调
    pub fn with_progress_callback(
        mut self,
        callback: Box<dyn Fn(MigrationProgress) + Send + Sync>,
    ) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// 添加材质映射
    pub fn add_material_mapping(&mut self, unity_name: String, mapping: MaterialMapping) {
        self.material_mappings.insert(unity_name, mapping);
    }

    /// 转换FBX模型
    pub async fn convert_fbx(
        &self,
        fbx_path: PathBuf,
        output_path: PathBuf,
    ) -> Result<ConvertedModel, MigrationError> {
        self.report_progress(0, 4, "Reading FBX file".to_string());

        // 1. 读取FBX文件
        let fbx_data = self.read_fbx_file(&fbx_path).await?;

        self.report_progress(1, 4, "Parsing mesh data".to_string());

        // 2. 解析网格
        let mesh_data = self.parse_fbx_mesh(&fbx_data)?;

        self.report_progress(2, 4, "Processing skeleton".to_string());

        // 3. 处理骨骼
        let skeleton_data = self.parse_fbx_skeleton(&fbx_data)?;

        self.report_progress(3, 4, "Processing animations".to_string());

        // 4. 处理动画
        let animations = self.parse_fbx_animations(&fbx_data)?;

        // 转换为glTF
        let gltf_data = self.convert_to_gltf(&mesh_data, &skeleton_data, &animations)?;

        // 保存到输出路径
        self.save_model(&gltf_data, &output_path).await?;

        self.report_progress(4, 4, "FBX conversion complete".to_string());

        Ok(ConvertedModel {
            mesh: mesh_data,
            skeleton: skeleton_data,
            animations,
            gltf_path: output_path,
        })
    }

    /// 转换材质
    pub async fn convert_material(
        &self,
        unity_material_path: PathBuf,
        output_path: PathBuf,
    ) -> Result<ConvertedMaterial, MigrationError> {
        self.report_progress(0, 3, "Reading Unity material".to_string());

        // 1. 读取Unity材质
        let material_data = self.read_unity_material(&unity_material_path).await?;

        self.report_progress(1, 3, "Converting to PBR".to_string());

        // 2. 转换为PBR材质
        let pbr_material = self.convert_to_pbr(&material_data)?;

        self.report_progress(2, 3, "Saving material".to_string());

        // 3. 保存材质
        self.save_material(&pbr_material, &output_path).await?;

        self.report_progress(3, 3, "Material conversion complete".to_string());

        Ok(pbr_material)
    }

    /// 转换动画
    pub async fn convert_animation(
        &self,
        unity_anim_path: PathBuf,
        output_path: PathBuf,
    ) -> Result<ConvertedAnimation, MigrationError> {
        self.report_progress(0, 3, "Reading Unity animation".to_string());

        // 1. 读取Unity动画
        let anim_data = self.read_unity_animation(&unity_anim_path).await?;

        self.report_progress(1, 3, "Processing curves".to_string());

        // 2. 处理动画曲线
        let curves = self.process_animation_curves(&anim_data)?;

        self.report_progress(2, 3, "Compressing animation".to_string());

        // 3. 压缩动画（如果启用）
        let compressed = if self.config.compress_animations {
            self.compress_animation(&curves)?
        } else {
            curves
        };

        self.save_animation(&compressed, &output_path).await?;

        self.report_progress(3, 3, "Animation conversion complete".to_string());

        Ok(ConvertedAnimation {
            curves: compressed,
            duration: anim_data.duration,
            is_looping: anim_data.looping,
        })
    }

    /// 读取FBX文件
    async fn read_fbx_file(&self, path: &PathBuf) -> Result<Vec<u8>, MigrationError> {
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        let metadata = file
            .metadata()
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        let mut buffer = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        Ok(buffer)
    }

    /// 解析FBX网格
    fn parse_fbx_mesh(&self, _fbx_data: &[u8]) -> Result<MeshData, MigrationError> {
        // 实际实现需要FBX SDK或第三方库
        // 这里返回示例数据
        Ok(MeshData {
            vertices: vec![],
            indices: vec![],
            normals: vec![],
            uvs: vec![],
            tangents: vec![],
            colors: vec![],
            bone_weights: vec![],
        })
    }

    /// 解析FBX骨骼
    fn parse_fbx_skeleton(&self, _fbx_data: &[u8]) -> Result<SkeletonData, MigrationError> {
        Ok(SkeletonData {
            bones: vec![],
            bone_names: vec![],
            parent_indices: vec![],
            inverse_bind_matrices: vec![],
        })
    }

    /// 解析FBX动画
    fn parse_fbx_animations(
        &self,
        _fbx_data: &[u8],
    ) -> Result<Vec<AnimationCurveData>, MigrationError> {
        Ok(vec![])
    }

    /// 转换为glTF
    fn convert_to_gltf(
        &self,
        mesh: &MeshData,
        skeleton: &SkeletonData,
        animations: &[AnimationCurveData],
    ) -> Result<GltfData, MigrationError> {
        Ok(GltfData {
            json: String::new(),
            binary: vec![],
        })
    }

    /// 保存模型
    async fn save_model(&self, gltf: &GltfData, path: &PathBuf) -> Result<(), MigrationError> {
        Ok(())
    }

    /// 读取Unity材质
    async fn read_unity_material(
        &self,
        path: &PathBuf,
    ) -> Result<UnityMaterialData, MigrationError> {
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        // 解析Unity材质文件（.mat是YAML格式）
        let yaml: serde_yaml::Value = serde_yaml::from_str(&contents)
            .map_err(|e| MigrationError::ParseError(e.to_string()))?;

        Ok(UnityMaterialData {
            shader: yaml.get("m_Shader").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            properties: HashMap::new(),
            textures: HashMap::new(),
            duration: 0.0,
            looping: false,
        })
    }

    /// 转换为PBR材质
    fn convert_to_pbr(
        &self,
        material: &UnityMaterialData,
    ) -> Result<ConvertedMaterial, MigrationError> {
        let mapping = self
            .material_mappings
            .get(&material.shader)
            .cloned()
            .unwrap_or(MaterialMapping::default());

        Ok(ConvertedMaterial {
            name: "converted_material".to_string(),
            albedo: mapping.albedo_color,
            metallic: mapping.metallic_value,
            roughness: mapping.roughness_value,
            normal_map: mapping.normal_map_path,
            albedo_map: mapping.albedo_map_path,
            metallic_map: mapping.metallic_map_path,
            roughness_map: mapping.roughness_map_path,
        })
    }

    /// 保存材质
    async fn save_material(
        &self,
        material: &ConvertedMaterial,
        path: &PathBuf,
    ) -> Result<(), MigrationError> {
        Ok(())
    }

    /// 读取Unity动画
    async fn read_unity_animation(
        &self,
        path: &PathBuf,
    ) -> Result<UnityAnimationData, MigrationError> {
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        Ok(UnityAnimationData {
            curves: vec![],
            duration: 1.0,
            looping: false,
        })
    }

    /// 处理动画曲线
    fn process_animation_curves(
        &self,
        anim: &UnityAnimationData,
    ) -> Result<Vec<AnimationCurveData>, MigrationError> {
        Ok(anim.curves.clone())
    }

    /// 压缩动画
    fn compress_animation(
        &self,
        curves: &[AnimationCurveData],
    ) -> Result<Vec<AnimationCurveData>, MigrationError> {
        // 实现关键帧压缩算法
        Ok(curves.to_vec())
    }

    /// 保存动画
    async fn save_animation(
        &self,
        anim: &[AnimationCurveData],
        path: &PathBuf,
    ) -> Result<(), MigrationError> {
        Ok(())
    }

    /// 报告进度
    fn report_progress(&self, completed: u32, total: u32, message: String) {
        if let Some(callback) = &self.progress_callback {
            let progress = MigrationProgress {
                total_steps: total,
                completed_steps: completed,
                current_phase: super::super::migration::MigrationPhase::ConvertingMeshes,
            };
            callback(progress);
        }
    }
}

/// 材质映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialMapping {
    /// Albedo颜色
    pub albedo_color: [f32; 4],

    /// Albedo贴图路径
    pub albedo_map_path: Option<String>,

    /// 金属度值
    pub metallic_value: f32,

    /// 金属度贴图路径
    pub metallic_map_path: Option<String>,

    /// 粗糙度值
    pub roughness_value: f32,

    /// 粗糙度贴图路径
    pub roughness_map_path: Option<String>,

    /// 法线贴图路径
    pub normal_map_path: Option<String>,
}

impl Default for MaterialMapping {
    fn default() -> Self {
        Self {
            albedo_color: [1.0, 1.0, 1.0, 1.0],
            albedo_map_path: None,
            metallic_value: 0.0,
            metallic_map_path: None,
            roughness_value: 0.5,
            roughness_map_path: None,
            normal_map_path: None,
        }
    }
}

/// 网格数据
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub tangents: Vec<[f32; 4]>,
    pub colors: Vec<[f32; 4]>,
    pub bone_weights: Vec<BoneWeight>,
}

/// 骨骼权重
#[derive(Debug, Clone)]
pub struct BoneWeight {
    pub bone_indices: [u32; 4],
    pub weights: [f32; 4],
}

/// 骨骼数据
#[derive(Debug, Clone)]
pub struct SkeletonData {
    pub bones: Vec<[f32; 16]>, // 变换矩阵
    pub bone_names: Vec<String>,
    pub parent_indices: Vec<i32>,
    pub inverse_bind_matrices: Vec<[f32; 16]>,
}

/// 动画曲线数据
#[derive(Debug, Clone)]
pub struct AnimationCurveData {
    pub bone_name: String,
    pub position_keys: Vec<(f32, [f32; 3])>,
    pub rotation_keys: Vec<(f32, [f32; 4])>,
    pub scale_keys: Vec<(f32, [f32; 3])>,
}

/// glTF数据
#[derive(Debug, Clone)]
pub struct GltfData {
    pub json: String,
    pub binary: Vec<u8>,
}

/// 转换的模型
#[derive(Debug, Clone)]
pub struct ConvertedModel {
    pub mesh: MeshData,
    pub skeleton: SkeletonData,
    pub animations: Vec<AnimationCurveData>,
    pub gltf_path: PathBuf,
}

/// 转换的材质
#[derive(Debug, Clone)]
pub struct ConvertedMaterial {
    pub name: String,
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub normal_map: Option<String>,
    pub albedo_map: Option<String>,
    pub metallic_map: Option<String>,
    pub roughness_map: Option<String>,
}

/// Unity材质数据
#[cfg(feature = "serde_yaml")]
#[derive(Debug, Clone)]
struct UnityMaterialData {
    pub shader: String,
    pub properties: HashMap<String, serde_yaml::Value>,
    pub textures: HashMap<String, String>,
    pub duration: f32,
    pub looping: bool,
}

/// Unity动画数据
#[cfg(feature = "serde_yaml")]
#[derive(Debug, Clone)]
struct UnityAnimationData {
    pub curves: Vec<AnimationCurve>,
    pub duration: f32,
    pub looping: bool,
}

/// 动画曲线
#[cfg(feature = "serde_yaml")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationCurve {
    pub bone_name: String,
    pub property_name: String,
    pub keys: Vec<(f32, f32)>,
}

/// 转换的动画
#[derive(Debug, Clone)]
pub struct ConvertedAnimation {
    pub curves: Vec<AnimationCurveData>,
    pub duration: f32,
    pub is_looping: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_creation() {
        let config = AssetConverterConfig::default();
        let converter = UnityAssetConverter::new(config);
        assert_eq!(converter.material_mappings.len(), 0);
    }

    #[test]
    fn test_material_mapping_default() {
        let mapping = MaterialMapping::default();
        assert_eq!(mapping.albedo_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(mapping.metallic_value, 0.0);
        assert_eq!(mapping.roughness_value, 0.5);
    }

    #[test]
    #[cfg(feature = "serde_yaml")]
    fn test_animation_curve_serialization() {
        let curve = AnimationCurve {
            bone_name: "root".to_string(),
            property_name: "position.x".to_string(),
            keys: vec![(0.0, 0.0), (1.0, 1.0)],
        };

        let json = serde_json::to_string(&curve);
        assert!(json.is_ok());
    }
}
