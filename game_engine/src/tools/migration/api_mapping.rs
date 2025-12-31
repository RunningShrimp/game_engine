//! Unity API映射表
//!
//! 完整的Unity API到本引擎API的映射定义。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unity API类别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnityAPICategory {
    GameObject,
    Transform,
    Rigidbody,
    Camera,
    Input,
    Time,
    Physics,
    Audio,
    UI,
    Animation,
    Scene,
    Resources,
}

/// Unity API映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnityAPIMapping {
    /// Unity API
    pub unity_api: String,
    /// 目标引擎API
    pub engine_api: String,
    /// API类别
    pub category: UnityAPICategory,
    /// 映射类型
    pub mapping_type: APIMappingType,
}

/// API映射类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum APIMappingType {
    /// 直接映射（名称相同或直接替换）
    Direct,
    /// 属性访问器（需要get/set）
    PropertyAccessor,
    /// 方法调用（需要参数转换）
    MethodCall,
    /// 事件系统（需要重写）
    Event,
    /// 自定义转换（需要特殊处理）
    Custom,
}

/// API映射表
pub struct APIMappingTable {
    mappings: HashMap<String, UnityAPIMapping>,
}

impl APIMappingTable {
    /// 创建新的API映射表
    pub fn new() -> Self {
        let mut table = Self {
            mappings: HashMap::new(),
        };

        table.add_default_mappings();
        table
    }

    /// 添加默认映射
    fn add_default_mappings(&mut self) {
        // ========== GameObject API ==========
        self.add_mapping(
            "GameObject.Find",
            "find_entity",
            UnityAPICategory::GameObject,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "GameObject.Instantiate",
            "instantiate_entity",
            UnityAPICategory::GameObject,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "GameObject.Destroy",
            "destroy_entity",
            UnityAPICategory::GameObject,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "gameObject.activeSelf",
            "visible",
            UnityAPICategory::GameObject,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "gameObject.name",
            "name",
            UnityAPICategory::GameObject,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "gameObject.tag",
            "tag",
            UnityAPICategory::GameObject,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "gameObject.transform",
            "transform",
            UnityAPICategory::GameObject,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "gameObject.GetComponent",
            "get_component",
            UnityAPICategory::GameObject,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "gameObject.AddComponent",
            "add_component",
            UnityAPICategory::GameObject,
            APIMappingType::MethodCall,
        );

        // ========== Transform API ==========
        self.add_mapping(
            "transform.position",
            "translation",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.localPosition",
            "local_translation",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.rotation",
            "rotation",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.localRotation",
            "local_rotation",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.localScale",
            "scale",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.forward",
            "forward",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.right",
            "right",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.up",
            "up",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.parent",
            "parent",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.root",
            "root",
            UnityAPICategory::Transform,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "transform.Translate",
            "translate",
            UnityAPICategory::Transform,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "transform.Rotate",
            "rotate",
            UnityAPICategory::Transform,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "transform.localScale",
            "scale",
            UnityAPICategory::Transform,
            APIMappingType::MethodCall,
        );

        // ========== Rigidbody API ==========
        self.add_mapping(
            "rigidbody.mass",
            "mass",
            UnityAPICategory::Rigidbody,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "rigidbody.drag",
            "linear_damping",
            UnityAPICategory::Rigidbody,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "rigidbody.angularDrag",
            "angular_damping",
            UnityAPICategory::Rigidbody,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "rigidbody.velocity",
            "linear_velocity",
            UnityAPICategory::Rigidbody,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "rigidbody.angularVelocity",
            "angular_velocity",
            UnityAPICategory::Rigidbody,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "rigidbody.useGravity",
            "use_gravity",
            UnityAPICategory::Rigidbody,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "rigidbody.isKinematic",
            "is_kinematic",
            UnityAPICategory::Rigidbody,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "Rigidbody.AddForce",
            "add_force",
            UnityAPICategory::Physics,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Rigidbody.AddTorque",
            "add_torque",
            UnityAPICategory::Physics,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Rigidbody.MovePosition",
            "move_position",
            UnityAPICategory::Physics,
            APIMappingType::MethodCall,
        );

        // ========== Camera API ==========
        self.add_mapping(
            "Camera.main",
            "primary_camera",
            UnityAPICategory::Camera,
            APIMappingType::Direct,
        );
        self.add_mapping(
            "camera.fieldOfView",
            "fov",
            UnityAPICategory::Camera,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "camera.nearClipPlane",
            "near",
            UnityAPICategory::Camera,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "camera.farClipPlane",
            "far",
            UnityAPICategory::Camera,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "camera.backgroundColor",
            "background_color",
            UnityAPICategory::Camera,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "camera.depth",
            "depth",
            UnityAPICategory::Camera,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "Camera.ScreenPointToRay",
            "screen_to_ray",
            UnityAPICategory::Camera,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Camera.ScreenToWorldPoint",
            "screen_to_world",
            UnityAPICategory::Camera,
            APIMappingType::MethodCall,
        );

        // ========== Input API ==========
        self.add_mapping(
            "Input.GetKey",
            "is_key_pressed",
            UnityAPICategory::Input,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Input.GetKeyDown",
            "is_key_just_pressed",
            UnityAPICategory::Input,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Input.GetKeyUp",
            "is_key_just_released",
            UnityAPICategory::Input,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Input.GetMouseButton",
            "is_mouse_button_pressed",
            UnityAPICategory::Input,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Input.GetMouseButtonDown",
            "is_mouse_button_just_pressed",
            UnityAPICategory::Input,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Input.GetMouseButtonUp",
            "is_mouse_button_just_released",
            UnityAPICategory::Input,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Input.mousePosition",
            "mouse_position",
            UnityAPICategory::Input,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "Input.GetAxis",
            "get_axis",
            UnityAPICategory::Input,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Input.GetAxisRaw",
            "get_axis_raw",
            UnityAPICategory::Input,
            APIMappingType::MethodCall,
        );

        // ========== Time API ==========
        self.add_mapping(
            "Time.deltaTime",
            "delta_time",
            UnityAPICategory::Time,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "Time.timeScale",
            "time_scale",
            UnityAPICategory::Time,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "Time.fixedDeltaTime",
            "fixed_delta_time",
            UnityAPICategory::Time,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "Time.time",
            "elapsed_time",
            UnityAPICategory::Time,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "Time.frameCount",
            "frame_count",
            UnityAPICategory::Time,
            APIMappingType::PropertyAccessor,
        );

        // ========== Physics API ==========
        self.add_mapping(
            "Physics.Raycast",
            "raycast",
            UnityAPICategory::Physics,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Physics.Linecast",
            "linecast",
            UnityAPICategory::Physics,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Physics.OverlapSphere",
            "overlap_sphere",
            UnityAPICategory::Physics,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Physics.CheckSphere",
            "check_sphere",
            UnityAPICategory::Physics,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Physics.gravity",
            "gravity",
            UnityAPICategory::Physics,
            APIMappingType::PropertyAccessor,
        );

        // ========== Audio API ==========
        self.add_mapping(
            "AudioSource.Play",
            "play",
            UnityAPICategory::Audio,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "AudioSource.Stop",
            "stop",
            UnityAPICategory::Audio,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "AudioSource.Pause",
            "pause",
            UnityAPICategory::Audio,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "audioSource.volume",
            "volume",
            UnityAPICategory::Audio,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "audioSource.clip",
            "sound",
            UnityAPICategory::Audio,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "audioSource.loop",
            "looping",
            UnityAPICategory::Audio,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "audioSource.spatialBlend",
            "spatial",
            UnityAPICategory::Audio,
            APIMappingType::PropertyAccessor,
        );
        self.add_mapping(
            "Audio.PlayOneShot",
            "play_one_shot",
            UnityAPICategory::Audio,
            APIMappingType::MethodCall,
        );

        // ========== Animation API ==========
        self.add_mapping(
            "animator.Play",
            "play_animation",
            UnityAPICategory::Animation,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "animator.Stop",
            "stop_animation",
            UnityAPICategory::Animation,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "animator.SetBool",
            "set_bool_parameter",
            UnityAPICategory::Animation,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "animator.SetFloat",
            "set_float_parameter",
            UnityAPICategory::Animation,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "animator.GetBool",
            "get_bool_parameter",
            UnityAPICategory::Animation,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "animator.GetFloat",
            "get_float_parameter",
            UnityAPICategory::Animation,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "animator.GetCurrentAnimatorStateInfo",
            "get_current_state",
            UnityAPICategory::Animation,
            APIMappingType::MethodCall,
        );

        // ========== Scene API ==========
        self.add_mapping(
            "SceneManager.LoadScene",
            "load_scene",
            UnityAPICategory::Scene,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "SceneManager.UnloadScene",
            "unload_scene",
            UnityAPICategory::Scene,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "SceneManager.GetActiveScene",
            "get_active_scene",
            UnityAPICategory::Scene,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "SceneManager.GetSceneByName",
            "get_scene_by_name",
            UnityAPICategory::Scene,
            APIMappingType::MethodCall,
        );

        // ========== Resources API ==========
        self.add_mapping(
            "Resources.Load",
            "load_resource",
            UnityAPICategory::Resources,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Resources.LoadAsync",
            "load_resource_async",
            UnityAPICategory::Resources,
            APIMappingType::MethodCall,
        );
        self.add_mapping(
            "Resources.UnloadUnusedAssets",
            "unload_unused_assets",
            UnityAPICategory::Resources,
            APIMappingType::MethodCall,
        );
    }

    /// 添加API映射
    fn add_mapping(
        &mut self,
        unity_api: &str,
        engine_api: &str,
        category: UnityAPICategory,
        mapping_type: APIMappingType,
    ) {
        self.mappings.insert(
            unity_api.to_string(),
            UnityAPIMapping {
                unity_api: unity_api.to_string(),
                engine_api: engine_api.to_string(),
                category,
                mapping_type,
            },
        );
    }

    /// 获取API映射
    pub fn get_mapping(&self, unity_api: &str) -> Option<&UnityAPIMapping> {
        self.mappings.get(unity_api)
    }

    /// 转换Unity API到引擎API
    pub fn convert_api(&self, unity_api: &str) -> Option<String> {
        self.mappings.get(unity_api).map(|mapping| match mapping.mapping_type {
            APIMappingType::Direct => mapping.engine_api.clone(),
            APIMappingType::PropertyAccessor => {
                format!("get_{}()", mapping.engine_api)
            }
            APIMappingType::MethodCall => mapping.engine_api.clone(),
            APIMappingType::Event => format!("event_{}", mapping.engine_api),
            APIMappingType::Custom => format!("custom_{}", mapping.engine_api),
        })
    }
}

impl Default for APIMappingTable {
    fn default() -> Self {
        Self::new()
    }
}

/// 脚本转换类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptTarget {
    Lua,
    TypeScript,
    JavaScript,
}

/// 脚本语言
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptLanguage {
    Lua,
    TypeScript,
    JavaScript,
}

/// 转换后的脚本
#[derive(Debug, Clone)]
pub struct ConvertedScript {
    /// 脚本代码
    pub code: String,
    /// 目标语言
    pub language: ScriptLanguage,
    /// 依赖列表
    pub dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_mapping_table() {
        let table = APIMappingTable::new();

        // 测试GameObject.Find映射
        let find_mapping = table.get_mapping("GameObject.Find");
        assert!(find_mapping.is_some());
        assert_eq!(find_mapping.unwrap().engine_api, "find_entity");

        // 测试API转换
        let converted = table.convert_api("GameObject.Find");
        assert_eq!(converted, Some("find_entity".to_string()));

        // 测试transform.position映射
        let pos_mapping = table.get_mapping("transform.position");
        assert!(pos_mapping.is_some());
        assert_eq!(pos_mapping.unwrap().engine_api, "translation");

        // 测试Time.deltaTime映射
        let delta_mapping = table.get_mapping("Time.deltaTime");
        assert!(delta_mapping.is_some());
        assert_eq!(delta_mapping.unwrap().engine_api, "delta_time");
    }

    #[test]
    fn test_api_categories() {
        let table = APIMappingTable::new();

        // 测试不同类别的API
        assert!(table.get_mapping("Input.GetKey").is_some());
        assert!(table.get_mapping("Camera.main").is_some());
        assert!(table.get_mapping("AudioSource.Play").is_some());
        assert!(table.get_mapping("Rigidbody.AddForce").is_some());
    }
}
