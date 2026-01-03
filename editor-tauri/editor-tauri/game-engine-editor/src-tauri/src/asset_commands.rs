// 资源管理Tauri命令实现
// 为AssetBrowser组件提供简化的资源管理功能

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

// 资源元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub tags: HashSet<String>,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub created_at: String,
    pub modified_at: String,
}

// 资源过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFilter {
    pub asset_type: Option<String>,
    pub tags: Vec<String>,
    pub search_query: Option<String>,
}

// 资源管理器（内存存储）
pub struct AssetManager {
    assets: Mutex<Vec<AssetMetadata>>,
    project_path: Mutex<Option<PathBuf>>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            assets: Mutex::new(Vec::new()),
            project_path: Mutex::new(None),
        }
    }

    /// 添加资源元数据
    pub fn add_asset(&self, asset: AssetMetadata) -> Result<(), String> {
        let mut assets = self.assets.lock().unwrap();

        // 检查是否已存在
        if assets.iter().any(|a| a.id == asset.id) {
            return Err(format!("资源 '{}' 已存在", asset.id));
        }

        assets.push(asset);
        Ok(())
    }

    /// 根据标签过滤资源
    pub fn filter_by_tags(&self, tags: &[String]) -> Vec<AssetMetadata> {
        let assets = self.assets.lock().unwrap();

        if tags.is_empty() {
            return assets.clone();
        }

        assets
            .iter()
            .filter(|asset| {
                tags.iter().all(|tag| asset.tags.contains(tag))
            })
            .cloned()
            .collect()
    }

    /// 搜索资源
    pub fn search_assets(&self, query: &str) -> Vec<AssetMetadata> {
        let assets = self.assets.lock().unwrap();
        let query_lower = query.to_lowercase();

        assets
            .iter()
            .filter(|asset| {
                asset.name.to_lowercase().contains(&query_lower)
                    || asset.asset_type.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// 按类型过滤
    pub fn filter_by_type(&self, asset_type: &str) -> Vec<AssetMetadata> {
        let assets = self.assets.lock().unwrap();

        assets
            .iter()
            .filter(|asset| asset.asset_type == asset_type)
            .cloned()
            .collect()
    }

    /// 获取所有资源
    pub fn get_all_assets(&self) -> Vec<AssetMetadata> {
        let assets = self.assets.lock().unwrap();
        assets.clone()
    }

    /// 删除资源
    pub fn delete_asset(&self, asset_id: &str) -> Result<(), String> {
        let mut assets = self.assets.lock().unwrap();
        let original_len = assets.len();

        assets.retain(|asset| asset.id != asset_id);

        if assets.len() < original_len {
            Ok(())
        } else {
            Err(format!("资源 '{}' 不存在", asset_id))
        }
    }

    /// 设置项目路径
    pub fn set_project_path(&self, path: PathBuf) {
        *self.project_path.lock().unwrap() = Some(path);
    }
}

// 全局资源管理器实例
lazy_static::lazy_static! {
    pub static ref ASSET_MANAGER: AssetManager = AssetManager::new();
}

// 公开管理器实例给其他模块使用
impl AssetManager {
    pub fn global() -> &'static AssetManager {
        &ASSET_MANAGER
    }
}

// Tauri命令实现
#[tauri::command]
pub async fn get_assets_by_tags(tags: Vec<String>) -> Result<Vec<AssetMetadata>, String> {
    Ok(ASSET_MANAGER.filter_by_tags(&tags))
}

#[tauri::command]
pub async fn search_assets(query: String) -> Result<Vec<AssetMetadata>, String> {
    Ok(ASSET_MANAGER.search_assets(&query))
}

#[tauri::command]
pub async fn get_assets_by_type(asset_type: String) -> Result<Vec<AssetMetadata>, String> {
    Ok(ASSET_MANAGER.filter_by_type(&asset_type))
}

#[tauri::command]
pub async fn get_all_assets() -> Result<Vec<AssetMetadata>, String> {
    Ok(ASSET_MANAGER.get_all_assets())
}

#[tauri::command]
pub async fn delete_asset(asset_id: String) -> Result<(), String> {
    ASSET_MANAGER.delete_asset(&asset_id)
}

#[tauri::command]
pub async fn update_asset_tags(asset_id: String, tags: Vec<String>) -> Result<(), String> {
    let mut assets = ASSET_MANAGER.assets.lock().unwrap();

    let asset = assets
        .iter_mut()
        .find(|a| a.id == asset_id)
        .ok_or_else(|| format!("资源 '{}' 不存在", asset_id))?;

    asset.tags = tags.into_iter().collect();
    Ok(())
}