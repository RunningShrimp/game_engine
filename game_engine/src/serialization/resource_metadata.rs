// 资源元数据序列化
//
// 提供资源元数据的序列化和反序列化功能。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 资源类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// 纹理
    Texture,
    /// 模型
    Model,
    /// 音频
    Audio,
    /// 着色器
    Shader,
    /// 材质
    Material,
    /// 场景
    Scene,
    /// 脚本
    Script,
    /// 字体
    Font,
    /// 自定义
    Custom(String),
}

/// 资源元数据
///
/// 包含资源的描述性信息，但不包含实际的资源数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    /// 资源ID
    pub id: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 资源路径
    pub path: PathBuf,
    /// 资源名称
    pub name: String,
    /// 资源描述
    #[serde(default)]
    pub description: String,
    /// 资源标签
    #[serde(default)]
    pub tags: Vec<String>,
    /// 资源大小（字节）
    #[serde(default)]
    pub size_bytes: u64,
    /// 资源依赖项
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// 自定义属性
    #[serde(default)]
    pub custom_properties: HashMap<String, String>,
    /// 加载状态
    #[serde(default)]
    pub load_state: ResourceLoadState,
    /// 缓存策略
    #[serde(default)]
    pub cache_policy: CachePolicy,
    /// 资源版本
    #[serde(default)]
    pub version: u32,
}

/// 资源加载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceLoadState {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 加载失败
    Failed,
}

impl Default for ResourceLoadState {
    fn default() -> Self {
        Self::Unloaded
    }
}

/// 缓存策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachePolicy {
    /// 不缓存
    Never,
    /// 总是缓存
    Always,
    /// 基于LRU缓存
    Lru,
    /// 仅在内存充足时缓存
    IfMemoryAvailable,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self::Lru
    }
}

impl ResourceMetadata {
    /// 创建新的资源元数据
    pub fn new(
        id: impl Into<String>,
        resource_type: ResourceType,
        path: impl AsRef<Path>,
    ) -> Self {
        let path = path.as_ref();

        Self {
            id: id.into(),
            resource_type,
            path: path.to_path_buf(),
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            description: String::new(),
            tags: Vec::new(),
            size_bytes: 0,
            dependencies: Vec::new(),
            custom_properties: HashMap::new(),
            load_state: ResourceLoadState::Unloaded,
            cache_policy: CachePolicy::Lru,
            version: 1,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// 设置大小
    pub fn with_size(mut self, size_bytes: u64) -> Self {
        self.size_bytes = size_bytes;
        self
    }

    /// 添加依赖项
    pub fn add_dependency(&mut self, dependency_id: impl Into<String>) {
        self.dependencies.push(dependency_id.into());
    }

    /// 设置自定义属性
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.custom_properties.insert(key.into(), value.into());
    }

    /// 获取自定义属性
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.custom_properties.get(key)
    }

    /// 设置缓存策略
    pub fn with_cache_policy(mut self, policy: CachePolicy) -> Self {
        self.cache_policy = policy;
        self
    }

    /// 检查是否已加载
    pub fn is_loaded(&self) -> bool {
        self.load_state == ResourceLoadState::Loaded
    }

    /// 检查是否有特定标签
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// 获取文件扩展名
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }

    /// 获取文件名（不含扩展名）
    pub fn file_stem(&self) -> Option<&str> {
        self.path.file_stem().and_then(|s| s.to_str())
    }
}

/// 资源包元数据
///
/// 描述一组相关资源的集合。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackMetadata {
    /// 资源包ID
    pub id: String,
    /// 资源包名称
    pub name: String,
    /// 资源包描述
    #[serde(default)]
    pub description: String,
    /// 资源包版本
    pub version: String,
    /// 包含的资源列表
    pub resources: Vec<ResourceMetadata>,
    /// 资源包标签
    #[serde(default)]
    pub tags: Vec<String>,
    /// 作者
    #[serde(default)]
    pub author: String,
    /// 许可证
    #[serde(default)]
    pub license: String,
    /// 创建时间（Unix timestamp）
    #[serde(default)]
    pub created_at: u64,
    /// 修改时间（Unix timestamp）
    #[serde(default)]
    pub modified_at: u64,
}

impl ResourcePackMetadata {
    /// 创建新的资源包元数据
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            version: "1.0.0".to_string(),
            resources: Vec::new(),
            tags: Vec::new(),
            author: String::new(),
            license: String::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// 添加资源
    pub fn add_resource(&mut self, resource: ResourceMetadata) {
        self.resources.push(resource);
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// 按类型查找资源
    pub fn find_resources_by_type(&self, resource_type: &ResourceType) -> Vec<&ResourceMetadata> {
        self.resources
            .iter()
            .filter(|r| &r.resource_type == resource_type)
            .collect()
    }

    /// 按标签查找资源
    pub fn find_resources_by_tag(&self, tag: &str) -> Vec<&ResourceMetadata> {
        self.resources
            .iter()
            .filter(|r| r.has_tag(tag))
            .collect()
    }

    /// 获取资源总数
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// 计算总大小
    pub fn total_size(&self) -> u64 {
        self.resources.iter().map(|r| r.size_bytes).sum()
    }
}

/// 资源索引
///
/// 用于快速查找资源的索引结构。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceIndex {
    /// 资源ID到元数据的映射
    #[serde(default)]
    id_index: HashMap<String, ResourceMetadata>,
    /// 路径到资源ID的映射
    #[serde(default)]
    path_index: HashMap<String, String>,
    /// 标签到资源ID列表的映射
    #[serde(default)]
    tag_index: HashMap<String, Vec<String>>,
    /// 类型到资源ID列表的映射
    #[serde(default)]
    type_index: HashMap<String, Vec<String>>,
}

impl ResourceIndex {
    /// 创建新的资源索引
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加资源到索引
    pub fn add(&mut self, metadata: ResourceMetadata) {
        let id = metadata.id.clone();
        let path_str = metadata.path.to_string_lossy().to_string();
        let type_str = format!("{:?}", metadata.resource_type);

        // 添加到ID索引
        self.id_index.insert(id.clone(), metadata.clone());

        // 添加到路径索引
        self.path_index.insert(path_str, id.clone());

        // 添加到标签索引
        for tag in &metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // 添加到类型索引
        self.type_index
            .entry(type_str)
            .or_insert_with(Vec::new)
            .push(id);
    }

    /// 按ID查找资源
    pub fn find_by_id(&self, id: &str) -> Option<&ResourceMetadata> {
        self.id_index.get(id)
    }

    /// 按路径查找资源
    pub fn find_by_path(&self, path: &Path) -> Option<&ResourceMetadata> {
        let path_str = path.to_string_lossy();
        let id = self.path_index.get(path_str.as_ref())?;
        self.id_index.get(id)
    }

    /// 按标签查找资源
    pub fn find_by_tag(&self, tag: &str) -> Vec<&ResourceMetadata> {
        self.tag_index
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.id_index.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 按类型查找资源
    pub fn find_by_type(&self, resource_type: &ResourceType) -> Vec<&ResourceMetadata> {
        let type_str = format!("{:?}", resource_type);
        self.type_index
            .get(&type_str)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.id_index.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取所有资源ID
    pub fn all_ids(&self) -> Vec<String> {
        self.id_index.keys().cloned().collect()
    }

    /// 获取资源总数
    pub fn count(&self) -> usize {
        self.id_index.len()
    }

    /// 移除资源
    pub fn remove(&mut self, id: &str) -> Option<ResourceMetadata> {
        if let Some(metadata) = self.id_index.remove(id) {
            // 从路径索引移除
            let path_str = metadata.path.to_string_lossy().to_string();
            self.path_index.remove(&path_str);

            // 从标签索引移除
            for tag in &metadata.tags {
                if let Some(ids) = self.tag_index.get_mut(tag) {
                    ids.retain(|x| x != id);
                }
            }

            // 从类型索引移除
            let type_str = format!("{:?}", metadata.resource_type);
            if let Some(ids) = self.type_index.get_mut(&type_str) {
                ids.retain(|x| x != id);
            }

            Some(metadata)
        } else {
            None
        }
    }

    /// 清空索引
    pub fn clear(&mut self) {
        self.id_index.clear();
        self.path_index.clear();
        self.tag_index.clear();
        self.type_index.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_metadata() {
        let mut metadata = ResourceMetadata::new(
            "texture_001",
            ResourceType::Texture,
            "/assets/textures/player.png",
        );

        metadata
            .add_tag("player")
            .add_tag("character")
            .add_dependency("shader_default");
        metadata.set_property("compression", "png");
        metadata.set_property("mipmaps", "true");

        assert_eq!(metadata.id, "texture_001");
        assert!(metadata.has_tag("player"));
        assert!(metadata.has_tag("character"));
        assert!(!metadata.has_tag("environment"));
        assert_eq!(
            metadata.get_property("compression"),
            Some(&"png".to_string())
        );
        assert_eq!(metadata.dependencies.len(), 1);
    }

    #[test]
    fn test_resource_pack_metadata() {
        let mut pack = ResourcePackMetadata::new("pack_001", "Main Assets");
        pack.description = "Main game assets pack".to_string();
        pack.author = "Game Studio".to_string();

        let texture1 = ResourceMetadata::new("tex1", ResourceType::Texture, "/assets/t1.png");
        let texture2 = ResourceMetadata::new("tex2", ResourceType::Texture, "/assets/t2.png");
        let audio = ResourceMetadata::new("audio1", ResourceType::Audio, "/assets/music.mp3");

        pack.add_resource(texture1);
        pack.add_resource(texture2);
        pack.add_resource(audio);

        assert_eq!(pack.resource_count(), 3);

        let textures = pack.find_resources_by_type(&ResourceType::Texture);
        assert_eq!(textures.len(), 2);

        let audios = pack.find_resources_by_type(&ResourceType::Audio);
        assert_eq!(audios.len(), 1);
    }

    #[test]
    fn test_resource_index() {
        let mut index = ResourceIndex::new();

        let mut tex1 =
            ResourceMetadata::new("tex1", ResourceType::Texture, "/assets/player.png");
        tex1.add_tag("player");

        let mut tex2 =
            ResourceMetadata::new("tex2", ResourceType::Texture, "/assets/enemy.png");
        tex2.add_tag("enemy");

        let mut audio1 =
            ResourceMetadata::new("audio1", ResourceType::Audio, "/assets/bgm.mp3");
        audio1.add_tag("player"); // 相同标签

        index.add(tex1);
        index.add(tex2);
        index.add(audio1);

        // 测试ID查找
        assert!(index.find_by_id("tex1").is_some());
        assert!(index.find_by_id("nonexistent").is_none());

        // 测试路径查找
        assert!(index
            .find_by_path(Path::new("/assets/player.png"))
            .is_some());

        // 测试标签查找
        let player_tagged = index.find_by_tag("player");
        assert_eq!(player_tagged.len(), 2);

        // 测试类型查找
        let textures = index.find_by_type(&ResourceType::Texture);
        assert_eq!(textures.len(), 2);

        let audios = index.find_by_type(&ResourceType::Audio);
        assert_eq!(audios.len(), 1);

        // 测试移除
        assert!(index.remove("tex1").is_some());
        assert_eq!(index.count(), 2);
        assert!(index.find_by_id("tex1").is_none());

        // 测试清空
        index.clear();
        assert_eq!(index.count(), 0);
    }
}
