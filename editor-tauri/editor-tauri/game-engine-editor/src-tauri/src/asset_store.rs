// Asset Store System
// 提供资源商店功能，包括资源搜索、下载、导入和预览

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// 资源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Model3D,
    Material,
    Texture,
    Audio,
    Script,
    Shader,
    Scene,
    Template,
    Plugin,
}

/// 资源类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetCategory {
    Characters,
    Environments,
    Props,
    Vehicles,
    Weapons,
    Effects,
    UI,
    Tools,
    Architecture,
    Nature,
}

/// 资源许可证
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseType {
    MIT,
    Apache2,
    GPL,
    CC0,
    CC_BY,
    CC_BY_SA,
    CC_BY_NC,
    Proprietary,
    Custom(String),
}

/// 资源定价类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingType {
    Free,
    Paid {
        price_usd: f64,
        discount_percent: Option<f64>,
    },
    Subscription {
        monthly_usd: f64,
        yearly_usd: f64,
    },
}

/// 资源元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: AssetType,
    pub category: AssetCategory,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub license: LicenseType,
    pub pricing: PricingType,
    pub file_size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub rating: f32,
    pub download_count: u32,
    pub preview_urls: Vec<String>,
    pub dependencies: Vec<String>,
    pub compatibility: Vec<String>,
    pub minimum_engine_version: String,
}

/// 搜索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub asset_type: Option<AssetType>,
    pub category: Option<AssetCategory>,
    pub tags: Vec<String>,
    pub license: Option<LicenseType>,
    pub pricing: Option<PricingType>,
    pub min_rating: Option<f32>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
}

/// 排序字段
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    CreatedAt,
    UpdatedAt,
    Name,
    Rating,
    Downloads,
    Price,
}

/// 排序顺序
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub assets: Vec<AssetMetadata>,
    pub total_count: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// 资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetData {
    pub metadata: AssetMetadata,
    pub files: Vec<AssetFile>,
    pub previews: Vec<PreviewData>,
}

/// 资源文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFile {
    pub filename: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub url: String,
    pub hash: String,
}

/// 预览数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewData {
    pub preview_type: String,
    pub url: String,
    pub thumbnail_url: String,
    pub width: u32,
    pub height: u32,
}

/// 用户收藏
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFavorite {
    pub user_id: String,
    pub asset_id: String,
    pub created_at: DateTime<Utc>,
}

/// 下载历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadHistory {
    pub user_id: String,
    pub asset_id: String,
    pub asset_version: String,
    pub downloaded_at: DateTime<Utc>,
    pub import_path: Option<PathBuf>,
}

/// 资源存储接口
#[async_trait::async_trait]
pub trait AssetStorage: Send + Sync {
    async fn save_asset(&self, asset: &AssetData) -> Result<(), AssetStoreError>;
    async fn load_asset(&self, id: &str) -> Result<Option<AssetData>, AssetStoreError>;
    async fn delete_asset(&self, id: &str) -> Result<(), AssetStoreError>;
    async fn list_assets(&self) -> Result<Vec<AssetMetadata>, AssetStoreError>;
}

/// CDN存储接口
#[async_trait::async_trait]
pub trait CdnStorage: Send + Sync {
    async fn upload_file(&self, path: &str, data: Vec<u8>) -> Result<String, AssetStoreError>;
    async fn download_file(&self, url: &str) -> Result<Vec<u8>, AssetStoreError>;
    async fn get_file_url(&self, path: &str) -> String;
    async fn delete_file(&self, url: &str) -> Result<(), AssetStoreError>;
}

/// 资源商店错误
#[derive(Debug, thiserror::Error)]
pub enum AssetStoreError {
    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Payment error: {0}")]
    PaymentError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// 资源商店客户端
pub struct AssetStoreClient {
    base_url: String,
    api_key: Option<String>,
    cdn: Arc<dyn CdnStorage>,
    storage: Arc<dyn AssetStorage>,
    cache: Arc<RwLock<HashMap<String, AssetData>>>,
}

impl AssetStoreClient {
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        cdn: Arc<dyn CdnStorage>,
        storage: Arc<dyn AssetStorage>,
    ) -> Self {
        Self {
            base_url,
            api_key,
            cdn,
            storage,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 搜索资源
    pub async fn search_assets(
        &self,
        query: SearchQuery,
    ) -> Result<SearchResult, AssetStoreError> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20);

        // 构建URL和查询参数
        let mut url = format!("{}/api/v1/assets/search", self.base_url);

        // 模拟API调用（实际应该发送HTTP请求）
        // 这里使用本地存储作为示例
        let all_assets = self.storage.list_assets().await?;

        // 过滤
        let filtered: Vec<AssetMetadata> = all_assets
            .into_iter()
            .filter(|asset| {
                // 类型过滤
                if let Some(ref t) = query.asset_type {
                    if &asset.asset_type != t {
                        return false;
                    }
                }

                // 类别过滤
                if let Some(ref c) = query.category {
                    if &asset.category != c {
                        return false;
                    }
                }

                // 标签过滤
                if !query.tags.is_empty() {
                    let asset_tags: std::collections::HashSet<&str> =
                        asset.tags.iter().map(|s| s.as_str()).collect();
                    let query_tags: std::collections::HashSet<&str> =
                        query.tags.iter().map(|s| s.as_str()).collect();
                    if !asset_tags.is_superset(&query_tags) {
                        return false;
                    }
                }

                // 评分过滤
                if let Some(min_rating) = query.min_rating {
                    if asset.rating < min_rating {
                        return false;
                    }
                }

                true
            })
            .collect();

        // 排序
        let mut sorted = filtered;
        if let Some(sort_field) = query.sort_by {
            sorted.sort_by(|a, b| match sort_field {
                SortField::CreatedAt => {
                    let ord = query.sort_order.as_ref().unwrap_or(&SortOrder::Desc);
                    match ord {
                        SortOrder::Asc => a.created_at.cmp(&b.created_at),
                        SortOrder::Desc => b.created_at.cmp(&a.created_at),
                    }
                }
                SortField::Rating => {
                    let ord = query.sort_order.as_ref().unwrap_or(&SortOrder::Desc);
                    match ord {
                        SortOrder::Asc => a.rating.partial_cmp(&b.rating).unwrap(),
                        SortOrder::Desc => b.rating.partial_cmp(&a.rating).unwrap(),
                    }
                }
                SortField::Downloads => {
                    let ord = query.sort_order.as_ref().unwrap_or(&SortOrder::Desc);
                    match ord {
                        SortOrder::Asc => a.download_count.cmp(&b.download_count),
                        SortOrder::Desc => b.download_count.cmp(&a.download_count),
                    }
                }
                _ => std::cmp::Ordering::Equal,
            });
        }

        // 分页
        let total_count = sorted.len() as u32;
        let total_pages = (total_count as f64 / per_page as f64).ceil() as u32;
        let start = ((page - 1) * per_page) as usize;
        let end = (start + per_page as usize).min(sorted.len());
        let page_assets = if start < sorted.len() {
            sorted[start..end].to_vec()
        } else {
            vec![]
        };

        Ok(SearchResult {
            assets: page_assets,
            total_count,
            page,
            per_page,
            total_pages,
        })
    }

    /// 下载资源
    pub async fn download_asset(&self, id: &str) -> Result<AssetData, AssetStoreError> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(asset) = cache.get(id) {
                return Ok(asset.clone());
            }
        }

        // 从存储加载
        let asset = self.storage.load_asset(id).await?.ok_or_else(|| {
            AssetStoreError::AssetNotFound(format!("Asset {} not found", id))
        })?;

        // 下载文件
        let mut files = Vec::new();
        for file in &asset.files {
            let data = self.cdn.download_file(&file.url).await?;
            // 这里可以保存到本地临时目录
            files.push(AssetFile {
                filename: file.filename.clone(),
                file_type: file.file_type.clone(),
                size_bytes: data.len() as u64,
                url: file.url.clone(),
                hash: file.hash.clone(),
            });
        }

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(id.to_string(), asset.clone());
        }

        Ok(asset)
    }

    /// 导入资源到项目
    pub async fn import_asset(
        &self,
        asset_id: &str,
        project_path: &Path,
    ) -> Result<PathBuf, AssetStoreError> {
        let asset = self.download_asset(asset_id).await?;

        // 创建资源目录
        let asset_dir = project_path.join("Assets").join(&asset.metadata.asset_type.to_string());
        tokio::fs::create_dir_all(&asset_dir).await?;

        // 解压并保存文件
        for file in &asset.files {
            let file_path = asset_dir.join(&file.filename);
            let data = self.cdn.download_file(&file.url).await?;
            tokio::fs::write(&file_path, data).await?;
        }

        Ok(asset_dir)
    }

    /// 获取预览图片
    pub async fn get_preview(&self, id: &str) -> Result<PreviewData, AssetStoreError> {
        let asset = self.storage.load_asset(id).await?.ok_or_else(|| {
            AssetStoreError::AssetNotFound(format!("Asset {} not found", id))
        })?;

        asset
            .previews
            .into_iter()
            .next()
            .ok_or_else(|| AssetStoreError::AssetNotFound("No preview available".to_string()))
    }

    /// 添加到收藏
    pub async fn add_favorite(
        &self,
        user_id: &str,
        asset_id: &str,
    ) -> Result<(), AssetStoreError> {
        // 这里应该保存到数据库或本地存储
        Ok(())
    }

    /// 移除收藏
    pub async fn remove_favorite(
        &self,
        user_id: &str,
        asset_id: &str,
    ) -> Result<(), AssetStoreError> {
        Ok(())
    }

    /// 获取收藏列表
    pub async fn get_favorites(&self, user_id: &str) -> Result<Vec<AssetMetadata>, AssetStoreError> {
        // 从存储加载用户收藏
        Ok(vec![])
    }

    /// 获取下载历史
    pub async fn get_download_history(
        &self,
        user_id: &str,
    ) -> Result<Vec<DownloadHistory>, AssetStoreError> {
        Ok(vec![])
    }

    /// 上传资源（用于创作者）
    pub async fn upload_asset(
        &self,
        asset: AssetData,
    ) -> Result<String, AssetStoreError> {
        // 验证资源
        self.validate_asset(&asset)?;

        // 上传文件到CDN
        let mut uploaded_files = Vec::new();
        for file in &asset.files {
            let data = tokio::fs::read(&file.filename).await?;
            let url = self.cdn.upload_file(&file.filename, data).await?;
            uploaded_files.push(AssetFile {
                filename: file.filename.clone(),
                file_type: file.file_type.clone(),
                size_bytes: file.size_bytes,
                url,
                hash: file.hash.clone(),
            });
        }

        // 保存到存储
        let asset_with_cdn = AssetData {
            files: uploaded_files,
            ..asset
        };

        self.storage.save_asset(&asset_with_cdn).await?;
        Ok(asset_with_cdn.metadata.id)
    }

    /// 验证资源
    fn validate_asset(&self, asset: &AssetData) -> Result<(), AssetStoreError> {
        if asset.metadata.name.is_empty() {
            return Err(AssetStoreError::ValidationError("Name cannot be empty".to_string()));
        }

        if asset.files.is_empty() {
            return Err(AssetStoreError::ValidationError("At least one file is required".to_string()));
        }

        Ok(())
    }
}

/// 本地文件存储实现
pub struct LocalAssetStorage {
    base_path: PathBuf,
}

impl LocalAssetStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

#[async_trait::async_trait]
impl AssetStorage for LocalAssetStorage {
    async fn save_asset(&self, asset: &AssetData) -> Result<(), AssetStoreError> {
        let asset_path = self
            .base_path
            .join(format!("{}.json", asset.metadata.id));
        let json = serde_json::to_string_pretty(asset)?;
        tokio::fs::write(asset_path, json).await?;
        Ok(())
    }

    async fn load_asset(&self, id: &str) -> Result<Option<AssetData>, AssetStoreError> {
        let asset_path = self.base_path.join(format!("{}.json", id));
        match tokio::fs::read_to_string(asset_path).await {
            Ok(json) => {
                let asset: AssetData = serde_json::from_str(&json)?;
                Ok(Some(asset))
            }
            Err(_) => Ok(None),
        }
    }

    async fn delete_asset(&self, id: &str) -> Result<(), AssetStoreError> {
        let asset_path = self.base_path.join(format!("{}.json", id));
        tokio::fs::remove_file(asset_path).await?;
        Ok(())
    }

    async fn list_assets(&self) -> Result<Vec<AssetMetadata>, AssetStoreError> {
        let mut assets = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = tokio::fs::read_to_string(&path).await {
                    if let Ok(asset) = serde_json::from_str::<AssetData>(&json) {
                        assets.push(asset.metadata);
                    }
                }
            }
        }

        Ok(assets)
    }
}

/// 模拟CDN存储
pub struct MockCdnStorage {
    base_url: String,
}

impl MockCdnStorage {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

#[async_trait::async_trait]
impl CdnStorage for MockCdnStorage {
    async fn upload_file(&self, path: &str, _data: Vec<u8>) -> Result<String, AssetStoreError> {
        Ok(format!("{}/{}", self.base_url, path))
    }

    async fn download_file(&self, url: &str) -> Result<Vec<u8>, AssetStoreError> {
        // 模拟返回空数据
        Ok(vec![])
    }

    async fn get_file_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path)
    }

    async fn delete_file(&self, _url: &str) -> Result<(), AssetStoreError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_assets() {
        let storage = Arc::new(LocalAssetStorage::new(PathBuf::from("/tmp/assets")));
        let cdn = Arc::new(MockCdnStorage::new("https://cdn.example.com".to_string()));
        let client = AssetStoreClient::new(
            "https://api.example.com".to_string(),
            None,
            cdn,
            storage,
        );

        let query = SearchQuery {
            query: Some("character".to_string()),
            asset_type: Some(AssetType::Model3D),
            category: Some(AssetCategory::Characters),
            tags: vec!["human".to_string()],
            ..Default::default()
        };

        let result = client.search_assets(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_asset_metadata_serialization() {
        let metadata = AssetMetadata {
            id: Uuid::new_v4().to_string(),
            name: "Test Asset".to_string(),
            description: "A test asset".to_string(),
            asset_type: AssetType::Model3D,
            category: AssetCategory::Characters,
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            tags: vec!["test".to_string()],
            license: LicenseType::MIT,
            pricing: PricingType::Free,
            file_size_bytes: 1024,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            rating: 4.5,
            download_count: 100,
            preview_urls: vec![],
            dependencies: vec![],
            compatibility: vec!["1.0.0".to_string()],
            minimum_engine_version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let decoded: AssetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, metadata.name);
    }
}

/// Tauri 状态管理
pub struct AssetStoreState {
    pub client: Arc<AssetStoreClient>,
}

impl AssetStoreState {
    pub fn new() -> Self {
        let base_path = PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".game-engine")
            .join("asset-store");

        std::fs::create_dir_all(&base_path).unwrap_or_default();

        let storage = Arc::new(LocalAssetStorage::new(base_path.clone()));
        let cdn = Arc::new(MockCdnStorage::new("https://cdn.example.com".to_string()));
        let client = Arc::new(AssetStoreClient::new(
            "https://api.example.com".to_string(),
            None,
            cdn,
            storage,
        ));

        Self { client }
    }
}

// Tauri 命令

/// 搜索资源
#[tauri::command]
pub async fn search_assets(
    state: State<'_, AssetStoreState>,
    query: Option<String>,
    asset_type: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> Result<SearchResult, String> {
    let search_query = SearchQuery {
        query,
        asset_type: asset_type.and_then(|t| serde_json::from_str(&format!("\"{}\"", t)).ok()),
        category: category.and_then(|c| serde_json::from_str(&format!("\"{}\"", c)).ok()),
        tags,
        license: None,
        pricing: None,
        min_rating: None,
        page,
        per_page,
        sort_by: None,
        sort_order: None,
    };

    state
        .client
        .search_assets(search_query)
        .await
        .map_err(|e| e.to_string())
}

/// 下载资源
#[tauri::command]
pub async fn download_asset(
    state: State<'_, AssetStoreState>,
    id: String,
) -> Result<AssetData, String> {
    state
        .client
        .download_asset(&id)
        .await
        .map_err(|e| e.to_string())
}

/// 导入资源到项目
#[tauri::command]
pub async fn import_asset(
    state: State<'_, AssetStoreState>,
    asset_id: String,
    project_path: String,
) -> Result<String, String> {
    let path = state
        .client
        .import_asset(&asset_id, Path::new(&project_path))
        .await
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

/// 获取预览
#[tauri::command]
pub async fn get_preview(
    state: State<'_, AssetStoreState>,
    id: String,
) -> Result<PreviewData, String> {
    state
        .client
        .get_preview(&id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取资源详情
#[tauri::command]
pub async fn get_asset_details(
    state: State<'_, AssetStoreState>,
    id: String,
) -> Result<AssetMetadata, String> {
    let asset_data = state
        .client
        .download_asset(&id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(asset_data.metadata)
}

/// 添加到收藏
#[tauri::command]
pub async fn add_favorite(
    state: State<'_, AssetStoreState>,
    user_id: String,
    asset_id: String,
) -> Result<(), String> {
    state
        .client
        .add_favorite(&user_id, &asset_id)
        .await
        .map_err(|e| e.to_string())
}

/// 移除收藏
#[tauri::command]
pub async fn remove_favorite(
    state: State<'_, AssetStoreState>,
    user_id: String,
    asset_id: String,
) -> Result<(), String> {
    state
        .client
        .remove_favorite(&user_id, &asset_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取收藏列表
#[tauri::command]
pub async fn get_favorites(
    state: State<'_, AssetStoreState>,
    user_id: String,
) -> Result<Vec<AssetMetadata>, String> {
    state
        .client
        .get_favorites(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取下载历史
#[tauri::command]
pub async fn get_download_history(
    state: State<'_, AssetStoreState>,
    user_id: String,
) -> Result<Vec<DownloadHistory>, String> {
    state
        .client
        .get_download_history(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取类别列表
#[tauri::command]
pub async fn get_categories() -> Result<Vec<String>, String> {
    Ok(vec![
        "characters".to_string(),
        "environments".to_string(),
        "props".to_string(),
        "vehicles".to_string(),
        "weapons".to_string(),
        "effects".to_string(),
        "ui".to_string(),
        "tools".to_string(),
        "architecture".to_string(),
        "nature".to_string(),
    ])
}

/// 获取资源类型列表
#[tauri::command]
pub async fn get_asset_types() -> Result<Vec<String>, String> {
    Ok(vec![
        "model_3d".to_string(),
        "material".to_string(),
        "texture".to_string(),
        "audio".to_string(),
        "script".to_string(),
        "shader".to_string(),
        "scene".to_string(),
        "template".to_string(),
        "plugin".to_string(),
    ])
}
