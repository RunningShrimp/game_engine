//! Unity项目导入器

use super::{MigrationError, ProjectAnalysis};
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "serde_yaml")]
use serde_yaml::Value as Yaml;

/// 迁移报告
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// 总脚本数
    pub total_scripts: u32,
    /// 成功迁移的脚本数
    pub migrated_scripts: u32,
    /// 失败的脚本数
    pub failed_scripts: u32,
    /// 警告列表
    pub warnings: Vec<String>,
}

/// 资源转换报告
#[derive(Debug, Clone)]
pub struct AssetConversionReport {
    /// 转换的纹理数
    pub converted_textures: u32,
    /// 转换的网格数
    pub converted_meshes: u32,
    /// 转换的材质数
    pub converted_materials: u32,
    /// 警告列表
    pub warnings: Vec<String>,
}

/// Unity脚本
#[derive(Debug, Clone)]
pub struct UnityScript {
    /// 脚本名称
    pub name: String,
    /// 脚本路径
    pub path: PathBuf,
    /// 是否继承自MonoBehaviour
    pub is_mono_behaviour: bool,
}

/// Unity预制体
#[derive(Debug, Clone)]
pub struct UnityPrefab {
    /// 预制体名称
    pub name: String,
    /// 预制体路径
    pub path: PathBuf,
    /// 游戏对象
    pub game_object: UnityGameObject,
}

/// Unity资源
#[derive(Debug, Clone)]
pub enum UnityAsset {
    /// 纹理
    Texture { path: PathBuf },
    /// 网格
    Mesh { path: PathBuf },
    /// 材质
    Material { path: PathBuf },
    /// 音频
    Audio { path: PathBuf },
}

/// Unity项目分析结果
#[derive(Debug, Clone)]
pub struct UnityProjectAnalysis {
    /// 脚本列表
    pub scripts: Vec<UnityScript>,
    /// 预制体列表
    pub prefabs: Vec<UnityPrefab>,
    /// 场景列表
    pub scenes: Vec<UnityScene>,
    /// 资源列表
    pub assets: Vec<UnityAsset>,
}

/// Unity项目导入器
pub struct UnityProjectImporter {
    /// 项目路径
    project_path: PathBuf,
    /// C#脚本转换器
    #[cfg(feature = "regex")]
    script_converter: Option<super::script_converter::UnityScriptConverter>,
}

impl UnityProjectImporter {
    /// 创建新导入器
    pub fn new() -> Self {
        Self {
            project_path: PathBuf::new(),
            #[cfg(feature = "regex")]
            script_converter: None,
        }
    }

    /// 创建带脚本转换器的导入器
    #[cfg(feature = "regex")]
    pub fn with_script_converter(mut self) -> Self {
        self.script_converter = Some(super::script_converter::UnityScriptConverter::new());
        self
    }

    /// 分析Unity项目
    pub async fn analyze(&self, path: &PathBuf) -> Result<ProjectAnalysis, MigrationError> {
        // 验证项目路径
        let assets_path = path.join("Assets");
        if !assets_path.exists() {
            return Err(MigrationError::InvalidProjectPath);
        }

        // 统计资产
        let mut texture_count = 0;
        let mut mesh_count = 0;
        let mut material_count = 0;
        let mut scene_count = 0;
        let mut script_count = 0;

        // 递归扫描Assets目录
        if let Ok(entries) = fs::read_dir(&assets_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        self.analyze_directory_recursive(
                            entry.path(),
                            &mut texture_count,
                            &mut mesh_count,
                            &mut material_count,
                            &mut scene_count,
                            &mut script_count,
                        )?;
                    } else if file_type.is_file() {
                        self.analyze_file(
                            entry.path(),
                            &mut texture_count,
                            &mut mesh_count,
                            &mut material_count,
                            &mut scene_count,
                            &mut script_count,
                        )?;
                    }
                }
            }
        }

        let total_assets = texture_count + mesh_count + material_count + scene_count + script_count;

        Ok(ProjectAnalysis {
            total_assets,
            texture_count,
            mesh_count,
            material_count,
            scene_count,
            script_count,
        })
    }

    /// 递归分析目录
    fn analyze_directory_recursive(
        &self,
        dir_path: PathBuf,
        texture_count: &mut u32,
        mesh_count: &mut u32,
        material_count: &mut u32,
        scene_count: &mut u32,
        script_count: &mut u32,
    ) -> Result<(), MigrationError> {
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        self.analyze_directory_recursive(
                            entry.path(),
                            texture_count,
                            mesh_count,
                            material_count,
                            scene_count,
                            script_count,
                        )?;
                    } else if file_type.is_file() {
                        self.analyze_file(
                            entry.path(),
                            texture_count,
                            mesh_count,
                            material_count,
                            scene_count,
                            script_count,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 分析单个文件
    fn analyze_file(
        &self,
        file_path: PathBuf,
        texture_count: &mut u32,
        mesh_count: &mut u32,
        material_count: &mut u32,
        scene_count: &mut u32,
        script_count: &mut u32,
    ) -> Result<(), MigrationError> {
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "png" | "jpg" | "jpeg" | "tga" | "psd" | "gif" | "bmp" | "exr" => {
                *texture_count += 1;
            }
            "fbx" | "obj" | "gltf" | "glb" | "blend" => {
                *mesh_count += 1;
            }
            "mat" | "asset" => {
                // 简单检查文件内容判断是否是材质
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if content.contains("Shader") || content.contains("Material") {
                        *material_count += 1;
                    }
                }
            }
            "unity" => {
                *scene_count += 1;
            }
            "cs" => {
                *script_count += 1;
            }
            _ => {}
        }
        Ok(())
    }

    /// 导入场景
    #[cfg(feature = "serde_yaml")]
    pub async fn import_scene(&self, scene_path: &PathBuf) -> Result<UnityScene, MigrationError> {
        // 读取.unity场景文件（YAML格式）
        let content = fs::read_to_string(scene_path).map_err(|e| {
            MigrationError::FileReadError(format!("Failed to read scene file: {}", e))
        })?;

        // 解析YAML
        let yaml: Yaml = serde_yaml::from_str(&content)
            .map_err(|e| MigrationError::ParseError(format!("Failed to parse YAML: {}", e)))?;

        // 提取场景名称
        let scene_name = scene_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("UnknownScene")
            .to_string();

        // 解析游戏对象
        let mut game_objects = Vec::new();

        // Unity场景文件中，游戏对象通常在"GameObject"键下
        if let Some(obj_value) = yaml.get("GameObject") {
            if let Some(obj_array) = obj_value.as_sequence() {
                for obj_yaml in obj_array {
                    if let Ok(game_obj) = self.parse_game_object_yaml(obj_yaml) {
                        game_objects.push(game_obj);
                    }
                }
            }
        }

        // 也可能在根级别有多个GameObject
        if let Some(mapping) = yaml.as_mapping() {
            for (key, value) in mapping {
                if let Some(key_str) = key.as_str() {
                    if key_str.contains("GameObject") || key_str.contains("m_Name") {
                        // 尝试解析为游戏对象
                        if let Ok(game_obj) = self.parse_game_object_yaml(value) {
                            if !game_objects.iter().any(|go| go.name == game_obj.name) {
                                game_objects.push(game_obj);
                            }
                        }
                    }
                }
            }
        }

        Ok(UnityScene {
            name: scene_name,
            game_objects,
        })
    }

    /// 迁移C#脚本到Lua/Rust
    #[cfg(feature = "regex")]
    pub async fn migrate_scripts(
        &self,
        output_path: &PathBuf,
    ) -> Result<MigrationReport, MigrationError> {
        use super::script_converter::{ScriptTarget, UnityScriptConverter};

        let converter = self
            .script_converter
            .as_ref()
            .cloned()
            .unwrap_or_else(UnityScriptConverter::new);

        let scripts_path = self.project_path.join("Assets");
        let mut migrated_count = 0;
        let mut failed_count = 0;
        let mut warnings = Vec::new();

        // 查找所有C#脚本
        let entries = fs::read_dir(&scripts_path)
            .map_err(|e| MigrationError::FileReadError(format!("Failed to read scripts: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("cs") {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("script");

                // 读取C#代码
                let csharp_code = fs::read_to_string(&path).map_err(|e| {
                    MigrationError::FileReadError(format!("Failed to read script: {}", e))
                })?;

                // 转换到Lua
                match converter.convert_csharp_to_lua(&csharp_code, file_name) {
                    Ok(converted) => {
                        // 写入Lua文件
                        let output_file =
                            output_path.join("scripts").join(format!("{}.lua", file_name));
                        if let Some(parent) = output_file.parent() {
                            fs::create_dir_all(parent).map_err(|e| {
                                MigrationError::ConversionError(format!(
                                    "Failed to create output dir: {}",
                                    e
                                ))
                            })?;
                        }

                        fs::write(&output_file, &converted.code).map_err(|e| {
                            MigrationError::ConversionError(format!(
                                "Failed to write script: {}",
                                e
                            ))
                        })?;

                        migrated_count += 1;
                        tracing::info!(
                            "Migrated script: {} -> {}",
                            file_name,
                            output_file.display()
                        );
                    }
                    Err(e) => {
                        failed_count += 1;
                        warnings.push(format!("Failed to migrate {}: {}", file_name, e));
                    }
                }
            }
        }

        Ok(MigrationReport {
            total_scripts: migrated_count + failed_count,
            migrated_scripts: migrated_count,
            failed_scripts: failed_count,
            warnings,
        })
    }

    /// 转换Unity资源格式
    pub async fn convert_assets(
        &self,
        output_path: &PathBuf,
    ) -> Result<AssetConversionReport, MigrationError> {
        let assets_path = self.project_path.join("Assets");
        let mut converted_textures = 0;
        let mut converted_meshes = 0;
        let mut converted_materials = 0;
        let mut warnings = Vec::new();

        // 递归扫描并转换资源
        if let Ok(entries) = fs::read_dir(&assets_path) {
            for entry in entries.flatten() {
                self.convert_asset_recursive(
                    entry.path(),
                    output_path,
                    &mut converted_textures,
                    &mut converted_meshes,
                    &mut converted_materials,
                    &mut warnings,
                )?;
            }
        }

        Ok(AssetConversionReport {
            converted_textures,
            converted_meshes,
            converted_materials,
            warnings,
        })
    }

    /// 递归转换资源
    fn convert_asset_recursive(
        &self,
        asset_path: PathBuf,
        output_path: &PathBuf,
        converted_textures: &mut u32,
        converted_meshes: &mut u32,
        converted_materials: &mut u32,
        warnings: &mut Vec<String>,
    ) -> Result<(), MigrationError> {
        if asset_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&asset_path) {
                for entry in entries.flatten() {
                    self.convert_asset_recursive(
                        entry.path(),
                        output_path,
                        converted_textures,
                        converted_meshes,
                        converted_materials,
                        warnings,
                    )?;
                }
            }
        } else {
            let extension = asset_path.extension().and_then(|e| e.to_str()).unwrap_or("");

            match extension {
                "png" | "jpg" | "jpeg" | "tga" => {
                    self.convert_single_asset(&asset_path, output_path, "texture")?;
                    *converted_textures += 1;
                }
                "fbx" | "obj" | "gltf" => {
                    self.convert_single_asset(&asset_path, output_path, "mesh")?;
                    *converted_meshes += 1;
                }
                "mat" => {
                    self.convert_single_asset(&asset_path, output_path, "material")?;
                    *converted_materials += 1;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 转换单个资源
    fn convert_single_asset(
        &self,
        asset_path: &PathBuf,
        output_path: &PathBuf,
        asset_type: &str,
    ) -> Result<(), MigrationError> {
        let relative_path = asset_path.strip_prefix(&self.project_path).unwrap_or(asset_path);
        let output_file = output_path.join(relative_path);

        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                MigrationError::ConversionError(format!("Failed to create output dir: {}", e))
            })?;
        }

        // 复制资源文件 (实际项目中需要格式转换)
        fs::copy(asset_path, &output_file).map_err(|e| {
            MigrationError::ConversionError(format!("Failed to copy {}: {}", asset_type, e))
        })?;

        tracing::debug!(
            "Converted {} asset: {:?}",
            asset_type,
            asset_path.file_name()
        );
        Ok(())
    }

    /// 生成迁移报告
    pub fn generate_report(&self) -> MigrationReport {
        MigrationReport {
            total_scripts: 0,
            migrated_scripts: 0,
            failed_scripts: 0,
            warnings: vec![],
        }
    }

    /// 解析单个GameObject的YAML
    #[cfg(feature = "serde_yaml")]
    fn parse_game_object_yaml(&self, yaml: &Yaml) -> Result<UnityGameObject, MigrationError> {
        let name = yaml.get("m_Name").and_then(|v| v.as_str()).unwrap_or("Unnamed").to_string();

        // 解析Transform组件
        let transform = self.parse_transform_yaml(yaml)?;

        // 解析组件列表
        let mut components = Vec::new();

        // 检查常见组件
        if yaml.get("m_Component").is_some() {
            components.push("Transform".to_string());
        }

        // 检查MeshRenderer
        if yaml.get("m_MeshRenderer").is_some() || yaml.get("MeshRenderer").is_some() {
            components.push("MeshRenderer".to_string());
        }

        // 检查MeshFilter
        if yaml.get("m_MeshFilter").is_some() || yaml.get("MeshFilter").is_some() {
            components.push("MeshFilter".to_string());
        }

        // 检查Collider
        if yaml.get("m_Collider").is_some() {
            components.push("Collider".to_string());
        }

        // 检查Rigidbody
        if yaml.get("m_Rigidbody").is_some() {
            components.push("Rigidbody".to_string());
        }

        // 检查Light
        if yaml.get("m_Light").is_some() || yaml.get("Light").is_some() {
            components.push("Light".to_string());
        }

        // 检查Camera
        if yaml.get("m_Camera").is_some() || yaml.get("Camera").is_some() {
            components.push("Camera".to_string());
        }

        // 检查Animator
        if yaml.get("m_Animator").is_some() || yaml.get("Animator").is_some() {
            components.push("Animator".to_string());
        }

        // 检查AudioSource
        if yaml.get("m_AudioSource").is_some() || yaml.get("AudioSource").is_some() {
            components.push("AudioSource".to_string());
        }

        // 检查ParticleSystem
        if yaml.get("m_ParticleSystem").is_some() {
            components.push("ParticleSystem".to_string());
        }

        Ok(UnityGameObject {
            name,
            transform,
            components,
        })
    }

    /// 解析Transform组件
    #[cfg(feature = "serde_yaml")]
    fn parse_transform_yaml(&self, yaml: &Yaml) -> Result<UnityTransform, MigrationError> {
        // 默认值
        let mut position = (0.0, 0.0, 0.0);
        let mut rotation = (0.0, 0.0, 0.0, 1.0); // Quaternion (x, y, z, w)
        let mut scale = (1.0, 1.0, 1.0);

        // 查找Transform数据
        if let Some(transform) = yaml.get("m_LocalPosition").or_else(|| yaml.get("position")) {
            if let Some(pos_array) = transform.as_sequence() {
                let pos_vec: Vec<f32> =
                    pos_array.iter().filter_map(|v| v.as_f64()).map(|v| v as f32).collect();
                if pos_vec.len() >= 3 {
                    position = (pos_vec[0], pos_vec[1], pos_vec[2]);
                }
            }
        }

        if let Some(rot) = yaml.get("m_LocalRotation").or_else(|| yaml.get("rotation")) {
            if let Some(rot_array) = rot.as_sequence() {
                let rot_vec: Vec<f32> =
                    rot_array.iter().filter_map(|v| v.as_f64()).map(|v| v as f32).collect();
                if rot_vec.len() >= 4 {
                    rotation = (rot_vec[0], rot_vec[1], rot_vec[2], rot_vec[3]);
                }
            }
        }

        if let Some(scl) = yaml.get("m_LocalScale").or_else(|| yaml.get("scale")) {
            if let Some(scl_array) = scl.as_sequence() {
                let scl_vec: Vec<f32> =
                    scl_array.iter().filter_map(|v| v.as_f64()).map(|v| v as f32).collect();
                if scl_vec.len() >= 3 {
                    scale = (scl_vec[0], scl_vec[1], scl_vec[2]);
                }
            }
        }

        Ok(UnityTransform {
            position,
            rotation,
            scale,
        })
    }
}

/// Unity场景
#[derive(Debug, Clone)]
pub struct UnityScene {
    /// 场景名称
    pub name: String,
    /// 游戏对象列表
    pub game_objects: Vec<UnityGameObject>,
}

/// Unity游戏对象
#[derive(Debug, Clone)]
pub struct UnityGameObject {
    /// 名称
    pub name: String,
    /// 变换
    pub transform: UnityTransform,
    /// 组件列表
    pub components: Vec<String>,
}

/// Unity变换
#[derive(Debug, Clone)]
pub struct UnityTransform {
    /// 位置
    pub position: (f32, f32, f32),
    /// 旋转
    pub rotation: (f32, f32, f32, f32),
    /// 缩放
    pub scale: (f32, f32, f32),
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_importer_creation() {
        let importer = UnityProjectImporter::new();
        let analysis = importer.analyze(&PathBuf::from("/fake/path")).await;

        assert!(analysis.is_ok());
    }
}
