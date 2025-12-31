//! Unity组件映射和转换系统
//!
//! 完整的Unity组件到本引擎组件的映射定义。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unity组件类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnityComponentType {
    // 基础组件
    Transform,
    MeshRenderer,
    SkinnedMeshRenderer,
    BoxCollider,
    SphereCollider,
    CapsuleCollider,
    MeshCollider,
    Rigidbody,
    Camera,
    Light,
    AudioSource,
    ParticleSystem,

    // UI组件
    Canvas,
    Image,
    Text,
    Button,
    Toggle,
    Slider,
    ScrollRect,

    // 动画组件
    Animator,
    Animation,

    // 导航组件
    NavMeshAgent,
    OffMeshLink,

    // 其他
    Terrain,
    WindZone,
    FlareLayer,
}

impl UnityComponentType {
    /// 获取组件名称
    pub fn name(&self) -> &str {
        match self {
            UnityComponentType::Transform => "Transform",
            UnityComponentType::MeshRenderer => "MeshRenderer",
            UnityComponentType::SkinnedMeshRenderer => "SkinnedMeshRenderer",
            UnityComponentType::BoxCollider => "BoxCollider",
            UnityComponentType::SphereCollider => "SphereCollider",
            UnityComponentType::CapsuleCollider => "CapsuleCollider",
            UnityComponentType::MeshCollider => "MeshCollider",
            UnityComponentType::Rigidbody => "Rigidbody",
            UnityComponentType::Camera => "Camera",
            UnityComponentType::Light => "Light",
            UnityComponentType::AudioSource => "AudioSource",
            UnityComponentType::ParticleSystem => "ParticleSystem",
            UnityComponentType::Canvas => "Canvas",
            UnityComponentType::Image => "Image",
            UnityComponentType::Text => "Text",
            UnityComponentType::Button => "Button",
            UnityComponentType::Toggle => "Toggle",
            UnityComponentType::Slider => "Slider",
            UnityComponentType::ScrollRect => "ScrollRect",
            UnityComponentType::Animator => "Animator",
            UnityComponentType::Animation => "Animation",
            UnityComponentType::NavMeshAgent => "NavMeshAgent",
            UnityComponentType::OffMeshLink => "OffMeshLink",
            UnityComponentType::Terrain => "Terrain",
            UnityComponentType::WindZone => "WindZone",
            UnityComponentType::FlareLayer => "FlareLayer",
        }
    }
}

/// Unity组件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnityComponent {
    /// 组件类型
    pub component_type: UnityComponentType,
    /// 是否启用
    pub enabled: bool,
    /// 属性数据
    pub properties: HashMap<String, ComponentProperty>,
}

/// 组件属性值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentProperty {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    Color([f32; 4]),
    Quaternion([f32; 4]),
    Array(Vec<ComponentProperty>),
    ObjectReference(u64), // GameObject instance ID
}

impl ComponentProperty {
    /// 获取float值
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ComponentProperty::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// 获取int值
    pub fn as_int(&self) -> Option<i32> {
        match self {
            ComponentProperty::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// 获取string值
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ComponentProperty::String(v) => Some(v),
            _ => None,
        }
    }
}

/// 组件映射配置
#[derive(Debug, Clone)]
pub struct ComponentMapping {
    /// Unity组件类型
    pub unity_component: UnityComponentType,
    /// 目标引擎组件类型
    pub engine_component: String,
    /// 属性映射
    pub property_mappings: HashMap<String, String>,
    /// 是否支持
    pub supported: bool,
    /// 转换函数名
    pub converter_function: Option<String>,
}

impl ComponentMapping {
    /// 创建新的组件映射
    pub fn new(
        unity_component: UnityComponentType,
        engine_component: String,
        supported: bool,
    ) -> Self {
        Self {
            unity_component,
            engine_component,
            property_mappings: HashMap::new(),
            supported,
            converter_function: None,
        }
    }

    /// 添加属性映射
    pub fn add_property_mapping(mut self, unity_prop: String, engine_prop: String) -> Self {
        self.property_mappings.insert(unity_prop, engine_prop);
        self
    }
}

/// 组件映射表
pub struct ComponentMappingRegistry {
    mappings: HashMap<UnityComponentType, ComponentMapping>,
}

impl ComponentMappingRegistry {
    /// 创建新的映射表
    pub fn new() -> Self {
        let mut registry = Self {
            mappings: HashMap::new(),
        };

        // 添加默认映射
        registry.add_default_mappings();

        registry
    }

    /// 添加默认映射
    fn add_default_mappings(&mut self) {
        // Transform -> Transform
        self.mappings.insert(
            UnityComponentType::Transform,
            ComponentMapping::new(UnityComponentType::Transform, "Transform".to_string(), true)
                .add_property_mapping("position".to_string(), "translation".to_string())
                .add_property_mapping("rotation".to_string(), "rotation".to_string())
                .add_property_mapping("scale".to_string(), "scale".to_string()),
        );

        // MeshRenderer -> MeshRenderer
        self.mappings.insert(
            UnityComponentType::MeshRenderer,
            ComponentMapping::new(
                UnityComponentType::MeshRenderer,
                "MeshRenderer".to_string(),
                true,
            )
            .add_property_mapping("materials".to_string(), "materials".to_string())
            .add_property_mapping("castShadows".to_string(), "cast_shadows".to_string())
            .add_property_mapping("receiveShadows".to_string(), "receive_shadows".to_string()),
        );

        // SkinnedMeshRenderer -> SkinnedMeshRenderer
        self.mappings.insert(
            UnityComponentType::SkinnedMeshRenderer,
            ComponentMapping::new(
                UnityComponentType::SkinnedMeshRenderer,
                "SkinnedMeshRenderer".to_string(),
                true,
            )
            .add_property_mapping("bones".to_string(), "bones".to_string())
            .add_property_mapping("rootBone".to_string(), "root_bone".to_string()),
        );

        // BoxCollider -> BoxCollider
        self.mappings.insert(
            UnityComponentType::BoxCollider,
            ComponentMapping::new(
                UnityComponentType::BoxCollider,
                "BoxCollider".to_string(),
                true,
            )
            .add_property_mapping("center".to_string(), "center".to_string())
            .add_property_mapping("size".to_string(), "size".to_string())
            .add_property_mapping("isTrigger".to_string(), "is_trigger".to_string()),
        );

        // Rigidbody -> RigidBody
        self.mappings.insert(
            UnityComponentType::Rigidbody,
            ComponentMapping::new(UnityComponentType::Rigidbody, "RigidBody".to_string(), true)
                .add_property_mapping("mass".to_string(), "mass".to_string())
                .add_property_mapping("drag".to_string(), "linear_damping".to_string())
                .add_property_mapping("angularDrag".to_string(), "angular_damping".to_string())
                .add_property_mapping("useGravity".to_string(), "use_gravity".to_string())
                .add_property_mapping("isKinematic".to_string(), "is_kinematic".to_string()),
        );

        // Camera -> Camera
        self.mappings.insert(
            UnityComponentType::Camera,
            ComponentMapping::new(UnityComponentType::Camera, "Camera".to_string(), true)
                .add_property_mapping("fieldOfView".to_string(), "fov".to_string())
                .add_property_mapping("nearClipPlane".to_string(), "near".to_string())
                .add_property_mapping("farClipPlane".to_string(), "far".to_string())
                .add_property_mapping("clearFlags".to_string(), "clear_mode".to_string()),
        );

        // Light -> Light
        self.mappings.insert(
            UnityComponentType::Light,
            ComponentMapping::new(UnityComponentType::Light, "Light".to_string(), true)
                .add_property_mapping("type".to_string(), "light_type".to_string())
                .add_property_mapping("color".to_string(), "color".to_string())
                .add_property_mapping("intensity".to_string(), "intensity".to_string())
                .add_property_mapping("range".to_string(), "range".to_string())
                .add_property_mapping("spotAngle".to_string(), "spot_angle".to_string()),
        );

        // Animator -> Animator
        self.mappings.insert(
            UnityComponentType::Animator,
            ComponentMapping::new(
                UnityComponentType::Animator,
                "AnimationStateMachine".to_string(),
                true,
            )
            .add_property_mapping("avatar".to_string(), "skeleton".to_string())
            .add_property_mapping(
                "runtimeAnimatorController".to_string(),
                "state_machine".to_string(),
            ),
        );

        // AudioSource -> AudioSource
        self.mappings.insert(
            UnityComponentType::AudioSource,
            ComponentMapping::new(
                UnityComponentType::AudioSource,
                "AudioSource".to_string(),
                true,
            )
            .add_property_mapping("clip".to_string(), "sound".to_string())
            .add_property_mapping("volume".to_string(), "volume".to_string())
            .add_property_mapping("loop".to_string(), "looping".to_string())
            .add_property_mapping("spatialBlend".to_string(), "spatial".to_string()),
        );

        // ParticleSystem -> ParticleSystem
        self.mappings.insert(
            UnityComponentType::ParticleSystem,
            ComponentMapping::new(
                UnityComponentType::ParticleSystem,
                "ParticleSystem".to_string(),
                true,
            ),
        );

        // UI组件映射
        self.mappings.insert(
            UnityComponentType::Canvas,
            ComponentMapping::new(UnityComponentType::Canvas, "UICanvas".to_string(), true),
        );

        self.mappings.insert(
            UnityComponentType::Image,
            ComponentMapping::new(UnityComponentType::Image, "UIImage".to_string(), true),
        );

        self.mappings.insert(
            UnityComponentType::Text,
            ComponentMapping::new(UnityComponentType::Text, "UILabel".to_string(), true),
        );

        self.mappings.insert(
            UnityComponentType::Button,
            ComponentMapping::new(UnityComponentType::Button, "UIButton".to_string(), true),
        );

        // NavMeshAgent -> NavMeshAgent
        self.mappings.insert(
            UnityComponentType::NavMeshAgent,
            ComponentMapping::new(
                UnityComponentType::NavMeshAgent,
                "NavMeshAgent".to_string(),
                true,
            ),
        );

        // Terrain -> Terrain
        self.mappings.insert(
            UnityComponentType::Terrain,
            ComponentMapping::new(
                UnityComponentType::Terrain,
                "Terrain".to_string(),
                false, // 需要自定义转换
            ),
        );
    }

    /// 获取组件映射
    pub fn get_mapping(&self, component_type: &UnityComponentType) -> Option<&ComponentMapping> {
        self.mappings.get(component_type)
    }

    /// 是否支持该组件
    pub fn is_supported(&self, component_type: &UnityComponentType) -> bool {
        self.mappings.get(component_type).map(|m| m.supported).unwrap_or(false)
    }

    /// 获取所有支持的组件
    pub fn get_supported_components(&self) -> Vec<UnityComponentType> {
        self.mappings
            .iter()
            .filter(|(_, m)| m.supported)
            .map(|(t, _)| t.clone())
            .collect()
    }
}

impl Default for ComponentMappingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_type_names() {
        assert_eq!(UnityComponentType::Transform.name(), "Transform");
        assert_eq!(UnityComponentType::Camera.name(), "Camera");
        assert_eq!(UnityComponentType::Light.name(), "Light");
    }

    #[test]
    fn test_mapping_registry() {
        let registry = ComponentMappingRegistry::new();

        // 测试Transform映射
        let transform_mapping = registry.get_mapping(&UnityComponentType::Transform);
        assert!(transform_mapping.is_some());
        assert_eq!(transform_mapping.unwrap().engine_component, "Transform");

        // 测试是否支持
        assert!(registry.is_supported(&UnityComponentType::Transform));
        assert!(registry.is_supported(&UnityComponentType::Camera));
        assert!(!registry.is_supported(&UnityComponentType::Terrain)); // 不支持
    }

    #[test]
    fn test_supported_components() {
        let registry = ComponentMappingRegistry::new();
        let supported = registry.get_supported_components();

        assert!(supported.contains(&UnityComponentType::Transform));
        assert!(supported.contains(&UnityComponentType::Camera));
        assert!(supported.len() > 20); // 应该有20+支持的组件
    }
}
