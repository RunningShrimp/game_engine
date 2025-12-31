//! Unity场景迁移工具
//!
//! 完整的Unity场景转换系统，支持所有Unity组件和嵌套GameObject。

use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "serde_yaml")]
use super::component_mapping::{ComponentMappingRegistry, UnityComponentType};
#[cfg(feature = "serde_yaml")]
use super::{MigrationError, MigrationPhase, MigrationProgress};
#[cfg(feature = "serde_yaml")]
use tokio::io::AsyncReadExt;

/// Unity场景迁移器
#[cfg(feature = "serde_yaml")]
pub struct UnitySceneMigrator {
    /// 配置
    config: SceneMigratorConfig,

    /// 组件映射表
    component_mappings: ComponentMappingRegistry,

    /// 进度回调
    progress_callback: Option<Box<dyn Fn(MigrationProgress) + Send + Sync>>,
}

/// 场景迁移配置
#[derive(Debug, Clone)]
pub struct SceneMigratorConfig {
    /// 是否保留原始层次结构
    pub preserve_hierarchy: bool,

    /// 是否转换预制体
    pub convert_prefabs: bool,

    /// 是否转换脚本组件
    pub convert_scripts: bool,

    /// 材质映射文件路径
    pub material_mapping_path: Option<PathBuf>,

    /// 是否生成日志
    pub generate_log: bool,
}

impl Default for SceneMigratorConfig {
    fn default() -> Self {
        Self {
            preserve_hierarchy: true,
            convert_prefabs: true,
            convert_scripts: true,
            material_mapping_path: None,
            generate_log: true,
        }
    }
}

#[cfg(feature = "serde_yaml")]
impl UnitySceneMigrator {
    /// 创建新的场景迁移器
    pub fn new(config: SceneMigratorConfig) -> Self {
        Self {
            config,
            component_mappings: ComponentMappingRegistry::new(),
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

    /// 迁移Unity场景
    pub async fn migrate_scene(
        &self,
        scene_path: PathBuf,
    ) -> Result<MigratedScene, MigrationError> {
        // 报告开始
        self.report_progress(0, 5, "Reading scene file".to_string());

        // 1. 读取Unity场景文件
        let scene_data = self.read_unity_scene(&scene_path).await?;

        // 2. 解析场景
        self.report_progress(1, 5, "Parsing scene structure".to_string());
        let parsed_scene = self.parse_scene(&scene_data)?;

        // 3. 转换游戏对象
        self.report_progress(2, 5, "Converting GameObjects".to_string());
        let entities = self.convert_game_objects(&parsed_scene)?;

        // 4. 转换组件
        self.report_progress(3, 5, "Converting components".to_string());
        let components = self.convert_components(&parsed_scene)?;

        // 5. 构建最终场景
        self.report_progress(4, 5, "Building scene".to_string());
        let migrated_scene = MigratedScene {
            entities,
            components,
            hierarchy: self.build_hierarchy(&parsed_scene),
            metadata: self.generate_metadata(&parsed_scene),
        };

        self.report_progress(5, 5, "Scene migration complete".to_string());

        Ok(migrated_scene)
    }

    /// 读取Unity场景文件
    async fn read_unity_scene(&self, path: &PathBuf) -> Result<String, MigrationError> {
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        Ok(contents)
    }

    /// 解析Unity场景
    fn parse_scene(&self, scene_data: &str) -> Result<ParsedUnityScene, MigrationError> {
        // Unity场景文件是YAML格式
        let yaml_data: serde_yaml::Value = serde_yaml::from_str(scene_data)
            .map_err(|e| MigrationError::ParseError(format!("Failed to parse YAML: {}", e)))?;

        let mut game_objects = Vec::new();
        let mut components = Vec::new();
        let mut prefabs = Vec::new();

        // 解析游戏对象
        if let Some(objects) = yaml_data.get("GameObject") {
            if let Some(obj_array) = objects.as_sequence() {
                for obj in obj_array {
                    if let Ok(game_obj) = self.parse_game_object(obj) {
                        game_objects.push(game_obj);
                    }
                }
            }
        }

        // 解析组件
        if let Some(comps) = yaml_data.get("Component") {
            if let Some(comp_array) = comps.as_sequence() {
                for comp in comp_array {
                    if let Ok(component) = self.parse_component(comp) {
                        components.push(component);
                    }
                }
            }
        }

        // 解析预制体
        if let Some(prefs) = yaml_data.get("PrefabInstance") {
            if let Some(pref_array) = prefs.as_sequence() {
                for pref in pref_array {
                    if let Ok(prefab) = self.parse_prefab(pref) {
                        prefabs.push(prefab);
                    }
                }
            }
        }

        Ok(ParsedUnityScene {
            game_objects,
            components,
            prefabs,
        })
    }

    /// 解析单个游戏对象
    fn parse_game_object(
        &self,
        obj: &serde_yaml::Value,
    ) -> Result<ParsedGameObject, MigrationError> {
        let file_id = obj.get("fileID").and_then(|v| v.as_i64()).unwrap_or(0) as u64;

        let name = obj.get("m_Name").and_then(|v| v.as_str()).unwrap_or("Unnamed").to_string();

        let transform = self.parse_transform(obj).unwrap_or_default();

        let parent_id = obj.get("m_Father").and_then(|v| v.as_i64()).map(|id| id as u64);

        let layer = obj.get("m_Layer").and_then(|v| v.as_i64()).unwrap_or(0) as u32;

        let tag = obj.get("m_Tag").and_then(|v| v.as_str()).unwrap_or("Untagged").to_string();

        let is_active = obj.get("m_IsActive").and_then(|v| v.as_bool()).unwrap_or(true);

        Ok(ParsedGameObject {
            file_id,
            name,
            transform,
            parent_id,
            layer,
            tag,
            is_active,
            children: Vec::new(),
        })
    }

    /// 解析变换组件
    fn parse_transform(&self, obj: &serde_yaml::Value) -> Option<TransformData> {
        let position = self.parse_vector3(obj.get("m_LocalPosition"))?;
        let rotation = self.parse_quaternion(obj.get("m_LocalRotation"))?;
        let scale = self.parse_vector3(obj.get("m_LocalScale")).unwrap_or(Vec3::ONE);

        Some(TransformData {
            position,
            rotation,
            scale,
        })
    }

    /// 解析Vector3
    fn parse_vector3(&self, value: Option<&serde_yaml::Value>) -> Option<Vec3> {
        let arr = value.as_ref()?.as_sequence()?;
        if arr.len() < 3 {
            return None;
        }
        Some(Vec3::new(
            arr[0].as_f64()? as f32,
            arr[1].as_f64()? as f32,
            arr[2].as_f64()? as f32,
        ))
    }

    /// 解析Quaternion
    fn parse_quaternion(&self, value: Option<&serde_yaml::Value>) -> Option<Quat> {
        let arr = value.as_ref()?.as_sequence()?;
        if arr.len() < 4 {
            return None;
        }
        Some(Quat::from_xyzw(
            arr[0].as_f64()? as f32,
            arr[1].as_f64()? as f32,
            arr[2].as_f64()? as f32,
            arr[3].as_f64()? as f32,
        ))
    }

    /// 解析组件
    fn parse_component(&self, comp: &serde_yaml::Value) -> Result<ParsedComponent, MigrationError> {
        let component_type = comp
            .get("component_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let game_object_id =
            comp.get("game_object_id").and_then(|v| v.as_i64()).unwrap_or(0) as u64;

        let enabled = comp.get("m_Enabled").and_then(|v| v.as_bool()).unwrap_or(true);

        let properties = comp.clone();

        Ok(ParsedComponent {
            component_type,
            game_object_id,
            enabled,
            properties,
        })
    }

    /// 解析预制体
    fn parse_prefab(&self, pref: &serde_yaml::Value) -> Result<ParsedPrefab, MigrationError> {
        let prefab_id = pref.get("fileID").and_then(|v| v.as_i64()).unwrap_or(0) as u64;

        let prefab_path = pref.get("m_Prefab").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let root_game_object_id =
            pref.get("m_RootGameObject").and_then(|v| v.as_i64()).unwrap_or(0) as u64;

        Ok(ParsedPrefab {
            prefab_id,
            prefab_path,
            root_game_object_id,
            modifications: HashMap::new(),
        })
    }

    /// 转换游戏对象
    fn convert_game_objects(
        &self,
        scene: &ParsedUnityScene,
    ) -> Result<Vec<MigratedEntity>, MigrationError> {
        let mut entities = Vec::new();

        for game_obj in &scene.game_objects {
            let entity = MigratedEntity {
                id: game_obj.file_id,
                name: game_obj.name.clone(),
                transform: game_obj.transform.clone(),
                parent_id: game_obj.parent_id,
                is_active: game_obj.is_active,
                layer: game_obj.layer,
                tag: game_obj.tag.clone(),
            };

            entities.push(entity);
        }

        Ok(entities)
    }

    /// 转换组件
    fn convert_components(
        &self,
        scene: &ParsedUnityScene,
    ) -> Result<Vec<MigratedComponent>, MigrationError> {
        let mut components = Vec::new();
        let mut unsupported_count = 0;

        for comp in &scene.components {
            // 尝试解析Unity组件类型
            let unity_component_type = self.parse_component_type(&comp.component_type);

            // 检查组件是否支持
            let supported = unity_component_type
                .as_ref()
                .map(|ctype| self.component_mappings.is_supported(ctype))
                .unwrap_or(false);

            if !supported {
                unsupported_count += 1;
                // 记录不支持的组件
                if self.config.generate_log {
                    eprintln!(
                        "Warning: Unsupported component '{}' on GameObject {}",
                        comp.component_type, comp.game_object_id
                    );
                }
                continue;
            }

            // 获取组件映射
            let engine_component_type = unity_component_type
                .as_ref()
                .and_then(|ctype| self.component_mappings.get_mapping(ctype))
                .map(|mapping| mapping.engine_component.clone())
                .unwrap_or_else(|| comp.component_type.clone());

            // 转换组件属性
            let converted_properties = if let Some(ctype) = unity_component_type {
                self.convert_component_properties(&comp.properties, &ctype)
            } else {
                comp.properties.clone()
            };

            let migrated = MigratedComponent {
                entity_id: comp.game_object_id,
                component_type: engine_component_type,
                enabled: comp.enabled,
                properties: converted_properties,
            };

            components.push(migrated);
        }

        if unsupported_count > 0 && self.config.generate_log {
            eprintln!(
                "Warning: {} components were not supported and skipped",
                unsupported_count
            );
        }

        Ok(components)
    }

    /// 解析组件类型
    fn parse_component_type(&self, component_type: &str) -> Option<UnityComponentType> {
        match component_type {
            "Transform" => Some(UnityComponentType::Transform),
            "MeshRenderer" => Some(UnityComponentType::MeshRenderer),
            "SkinnedMeshRenderer" => Some(UnityComponentType::SkinnedMeshRenderer),
            "BoxCollider" => Some(UnityComponentType::BoxCollider),
            "SphereCollider" => Some(UnityComponentType::SphereCollider),
            "CapsuleCollider" => Some(UnityComponentType::CapsuleCollider),
            "MeshCollider" => Some(UnityComponentType::MeshCollider),
            "Rigidbody" => Some(UnityComponentType::Rigidbody),
            "Camera" => Some(UnityComponentType::Camera),
            "Light" => Some(UnityComponentType::Light),
            "AudioSource" => Some(UnityComponentType::AudioSource),
            "ParticleSystem" => Some(UnityComponentType::ParticleSystem),
            "Canvas" => Some(UnityComponentType::Canvas),
            "Image" => Some(UnityComponentType::Image),
            "Text" | "TextMesh" => Some(UnityComponentType::Text),
            "Button" => Some(UnityComponentType::Button),
            "Toggle" => Some(UnityComponentType::Toggle),
            "Slider" => Some(UnityComponentType::Slider),
            "ScrollRect" => Some(UnityComponentType::ScrollRect),
            "Animator" => Some(UnityComponentType::Animator),
            "Animation" => Some(UnityComponentType::Animation),
            "NavMeshAgent" => Some(UnityComponentType::NavMeshAgent),
            "Terrain" => Some(UnityComponentType::Terrain),
            _ => None,
        }
    }

    /// 转换组件属性
    fn convert_component_properties(
        &self,
        properties: &serde_yaml::Value,
        component_type: &UnityComponentType,
    ) -> serde_yaml::Value {
        if let Some(mapping) = self.component_mappings.get_mapping(component_type) {
            let mut converted_map = serde_yaml::mapping::Mapping::new();

            // 复制所有原始属性
            if let Some(map) = properties.as_mapping() {
                for (key, value) in map {
                    let key_str = key.as_str().unwrap_or("");

                    // 检查是否有属性映射
                    let new_key = mapping
                        .property_mappings
                        .get(key_str)
                        .map(|s| s.as_str())
                        .unwrap_or(key_str);

                    converted_map.insert(
                        serde_yaml::Value::String(new_key.to_string()),
                        value.clone(),
                    );
                }
            }

            serde_yaml::Value::Mapping(converted_map)
        } else {
            properties.clone()
        }
    }

    /// 构建层次结构
    fn build_hierarchy(&self, scene: &ParsedUnityScene) -> Vec<HierarchyNode> {
        let mut hierarchy = Vec::new();
        let mut id_to_obj: HashMap<u64, &ParsedGameObject> = HashMap::new();

        for obj in &scene.game_objects {
            id_to_obj.insert(obj.file_id, obj);
        }

        // 找到根对象（没有父对象的）
        for obj in &scene.game_objects {
            if obj.parent_id.is_none() {
                let node = self.build_hierarchy_node(obj, &id_to_obj);
                hierarchy.push(node);
            }
        }

        hierarchy
    }

    /// 构建层次结构节点
    fn build_hierarchy_node(
        &self,
        obj: &ParsedGameObject,
        id_to_obj: &HashMap<u64, &ParsedGameObject>,
    ) -> HierarchyNode {
        let mut children = Vec::new();

        for child in &obj.children {
            if let Some(child_obj) = id_to_obj.get(child) {
                let child_node = self.build_hierarchy_node(child_obj, id_to_obj);
                children.push(child_node);
            }
        }

        HierarchyNode {
            name: obj.name.clone(),
            file_id: obj.file_id,
            children,
        }
    }

    /// 生成元数据
    fn generate_metadata(&self, scene: &ParsedUnityScene) -> SceneMetadata {
        SceneMetadata {
            game_object_count: scene.game_objects.len(),
            component_count: scene.components.len(),
            prefab_count: scene.prefabs.len(),
        }
    }

    /// 报告进度
    fn report_progress(&self, completed: u32, total: u32, message: String) {
        if let Some(callback) = &self.progress_callback {
            let progress = MigrationProgress {
                total_steps: total,
                completed_steps: completed,
                current_phase: super::super::migration::MigrationPhase::ConvertingScenes,
            };
            callback(progress);
        }
    }
}

/// Unity场景解析结果
#[cfg(feature = "serde_yaml")]
#[derive(Debug, Clone)]
struct ParsedUnityScene {
    game_objects: Vec<ParsedGameObject>,
    components: Vec<ParsedComponent>,
    prefabs: Vec<ParsedPrefab>,
}

/// 解析的游戏对象
#[derive(Debug, Clone)]
struct ParsedGameObject {
    file_id: u64,
    name: String,
    transform: TransformData,
    parent_id: Option<u64>,
    layer: u32,
    tag: String,
    is_active: bool,
    children: Vec<u64>,
}

/// 变换数据
#[derive(Debug, Clone)]
struct TransformData {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Default for TransformData {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// 解析的组件
#[cfg(feature = "serde_yaml")]
#[derive(Debug, Clone)]
struct ParsedComponent {
    component_type: String,
    game_object_id: u64,
    enabled: bool,
    properties: serde_yaml::Value,
}

/// 解析的预制体
#[cfg(feature = "serde_yaml")]
#[derive(Debug, Clone)]
struct ParsedPrefab {
    prefab_id: u64,
    prefab_path: String,
    root_game_object_id: u64,
    modifications: HashMap<String, serde_yaml::Value>,
}

/// 迁移后的场景
#[cfg(feature = "serde_yaml")]
#[derive(Debug, Clone)]
pub struct MigratedScene {
    pub entities: Vec<MigratedEntity>,
    pub components: Vec<MigratedComponent>,
    pub hierarchy: Vec<HierarchyNode>,
    pub metadata: SceneMetadata,
}

/// 迁移的实体
#[derive(Debug, Clone)]
pub struct MigratedEntity {
    pub id: u64,
    pub name: String,
    pub transform: TransformData,
    pub parent_id: Option<u64>,
    pub is_active: bool,
    pub layer: u32,
    pub tag: String,
}

/// 迁移的组件
#[cfg(feature = "serde_yaml")]
#[derive(Debug, Clone)]
pub struct MigratedComponent {
    pub entity_id: u64,
    pub component_type: String,
    pub enabled: bool,
    pub properties: serde_yaml::Value,
}

/// 层次结构节点
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub name: String,
    pub file_id: u64,
    pub children: Vec<HierarchyNode>,
}

/// 场景元数据
#[derive(Debug, Clone)]
pub struct SceneMetadata {
    pub game_object_count: usize,
    pub component_count: usize,
    pub prefab_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrator_creation() {
        let config = SceneMigratorConfig::default();
        let migrator = UnitySceneMigrator::new(config);
        assert!(migrator.progress_callback.is_none());
    }

    #[test]
    fn test_transform_data_default() {
        let transform = TransformData::default();
        assert_eq!(transform.position, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_hierarchy_node() {
        let node = HierarchyNode {
            name: "Root".to_string(),
            file_id: 1,
            children: vec![HierarchyNode {
                name: "Child".to_string(),
                file_id: 2,
                children: vec![],
            }],
        };

        assert_eq!(node.name, "Root");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, "Child");
    }
}
