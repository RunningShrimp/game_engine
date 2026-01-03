//! Marketplace client for interacting with the plugin marketplace API

use super::models::*;
use super::{PluginError, SearchFilters, SortBy};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use std::collections::HashMap;

/// Marketplace client configuration
#[derive(Debug, Clone)]
pub struct MarketplaceConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub timeout: Duration,
}

/// Marketplace client
pub struct Marketplace {
    client: Client,
    config: MarketplaceConfig,
}

impl Marketplace {
    /// Create a new marketplace client
    pub fn new(config: MarketplaceConfig) -> Result<Self, PluginError> {
        let mut client_builder = Client::builder()
            .timeout(config.timeout);

        let client = client_builder
            .build()
            .map_err(|e| PluginError::MarketplaceError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Search for plugins
    pub async fn search(&self, query: &str, filters: SearchFilters) -> Result<Vec<PluginInfo>, PluginError> {
        let mut params = HashMap::new();
        params.insert("q", query);

        if !filters.categories.is_empty() {
            params.insert("category", &filters.categories.join(","));
        }
        if !filters.tags.is_empty() {
            params.insert("tags", &filters.tags.join(","));
        }
        if let Some(sort) = filters.sort_by {
            let sort_str = match sort {
                SortBy::Relevance => "relevance",
                SortBy::Downloads => "downloads",
                SortBy::Rating => "rating",
                SortBy::Updated => "updated",
                SortBy::Name => "name",
            };
            params.insert("sort", sort_str);
        }

        let response = self
            .get("/api/v1/plugins/search", &params)
            .await?;

        let plugins: Vec<PluginInfo> = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse search results: {}", e)))?;

        Ok(plugins)
    }

    /// Get plugin information by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<PluginInfo, PluginError> {
        let response = self
            .get(&format!("/api/v1/plugins/{}", plugin_id), &HashMap::new())
            .await?;

        let plugin: PluginInfo = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse plugin info: {}", e)))?;

        Ok(plugin)
    }

    /// Get all versions of a plugin
    pub async fn get_plugin_versions(&self, plugin_id: &str) -> Result<Vec<PluginVersion>, PluginError> {
        let response = self
            .get(&format!("/api/v1/plugins/{}/versions", plugin_id), &HashMap::new())
            .await?;

        let versions: Vec<PluginVersion> = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse versions: {}", e)))?;

        Ok(versions)
    }

    /// Download a plugin package
    pub async fn download_plugin(&self, plugin_id: &str, version: &str) -> Result<super::PluginPackage, PluginError> {
        let mut params = HashMap::new();
        params.insert("version", version);

        let download_info: Value = self
            .get(&format!("/api/v1/plugins/{}/download", plugin_id), &params)
            .await?;

        let download_url = download_info["url"]
            .as_str()
            .ok_or_else(|| PluginError::MarketplaceError("Missing download URL".to_string()))?;

        let sha256 = download_info["sha256"]
            .as_str()
            .ok_or_else(|| PluginError::MarketplaceError("Missing SHA256".to_string()))?;

        // Download the plugin package
        let response = self.client
            .get(download_url)
            .send()
            .await
            .map_err(|e| PluginError::Network(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(PluginError::Network(format!("Download failed with status: {}", response.status())));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| PluginError::Network(format!("Failed to read response: {}", e)))?;

        // Verify SHA256
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let calculated_hash = format!("{:x}", hasher.finalize());

        if calculated_hash != sha256 {
            return Err(PluginError::InstallerError(format!(
                "SHA256 mismatch: expected {}, got {}",
                sha256, calculated_hash
            )));
        }

        // Extract the package
        let plugin_package = self.extract_package(&bytes)?;

        Ok(plugin_package)
    }

    /// Get plugin reviews
    pub async fn get_reviews(&self, plugin_id: &str, page: u32, limit: u32) -> Result<Vec<PluginReview>, PluginError> {
        let mut params = HashMap::new();
        params.insert("page", &page.to_string());
        params.insert("limit", &limit.to_string());

        let response = self
            .get(&format!("/api/v1/plugins/{}/reviews", plugin_id), &params)
            .await?;

        let reviews: Vec<PluginReview> = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse reviews: {}", e)))?;

        Ok(reviews)
    }

    /// Submit a review
    pub async fn submit_review(&self, plugin_id: &str, review: ReviewSubmission) -> Result<PluginReview, PluginError> {
        let response = self
            .post(&format!("/api/v1/plugins/{}/reviews", plugin_id), &review)
            .await?;

        let created_review: PluginReview = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse review response: {}", e)))?;

        Ok(created_review)
    }

    /// Get marketplace statistics
    pub async fn get_stats(&self) -> Result<MarketplaceStats, PluginError> {
        let response = self
            .get("/api/v1/stats", &HashMap::new())
            .await?;

        let stats: MarketplaceStats = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse stats: {}", e)))?;

        Ok(stats)
    }

    /// Publish a plugin
    pub async fn publish_plugin(&self, package: &PublishPackage) -> Result<PublishResult, PluginError> {
        let response = self
            .post("/api/v1/plugins/publish", package)
            .await?;

        let result: PublishResult = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse publish result: {}", e)))?;

        Ok(result)
    }

    /// Update a plugin
    pub async fn update_plugin(&self, plugin_id: &str, package: &PublishPackage) -> Result<(), PluginError> {
        self.put(&format!("/api/v1/plugins/{}", plugin_id), package)
            .await?;
        Ok(())
    }

    /// Get user's published plugins
    pub async fn get_my_plugins(&self) -> Result<Vec<PluginInfo>, PluginError> {
        let response = self
            .get("/api/v1/me/plugins", &HashMap::new())
            .await?;

        let plugins: Vec<PluginInfo> = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse plugins: {}", e)))?;

        Ok(plugins)
    }

    /// Get download statistics for a plugin
    pub async fn get_download_stats(&self, plugin_id: &str, days: u32) -> Result<DownloadStats, PluginError> {
        let mut params = HashMap::new();
        params.insert("days", &days.to_string());

        let response = self
            .get(&format!("/api/v1/plugins/{}/stats/downloads", plugin_id), &params)
            .await?;

        let stats: DownloadStats = serde_json::from_value(response)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse download stats: {}", e)))?;

        Ok(stats)
    }

    // Helper methods

    async fn get(&self, path: &str, params: &HashMap<&str, &str>) -> Result<Value, PluginError> {
        let url = format!("{}{}", self.config.api_url, path);

        let mut request = self.client.get(&url);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let request = request.query(params);

        let response = request
            .send()
            .await
            .map_err(|e| PluginError::Network(format!("GET request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(PluginError::MarketplaceError(format!("API error: {} - {}", status, error_text)));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| PluginError::Serialization(format!("Failed to parse JSON: {}", e)))?;

        Ok(json)
    }

    async fn post(&self, path: &str, body: &impl serde::Serialize) -> Result<Value, PluginError> {
        let url = format!("{}{}", self.config.api_url, path);

        let mut request = self.client.post(&url).json(body);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| PluginError::Network(format!("POST request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(PluginError::MarketplaceError(format!("API error: {} - {}", status, error_text)));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| PluginError::Serialization(format!("Failed to parse JSON: {}", e)))?;

        Ok(json)
    }

    async fn put(&self, path: &str, body: &impl serde::Serialize) -> Result<Value, PluginError> {
        let url = format!("{}{}", self.config.api_url, path);

        let mut request = self.client.put(&url).json(body);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| PluginError::Network(format!("PUT request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(PluginError::MarketplaceError(format!("API error: {} - {}", status, error_text)));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| PluginError::Serialization(format!("Failed to parse JSON: {}", e)))?;

        Ok(json)
    }

    fn extract_package(&self, bytes: &[u8]) -> Result<super::PluginPackage, PluginError> {
        use std::io::Cursor;

        // Decompress gzip/tarball
        let cursor = Cursor::new(bytes);
        let decoder = flate2::read::GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(decoder);

        let temp_dir = std::env::temp_dir();
        let extract_path = temp_dir.join(format!("plugin_{}", uuid::Uuid::new_v4()));

        archive
            .unpack(&extract_path)
            .map_err(|e| PluginError::InstallerError(format!("Failed to extract package: {}", e)))?;

        // Read manifest
        let manifest_path = extract_path.join("plugin.json");
        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::InstallerError(format!("Failed to read manifest: {}", e)))?;

        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse manifest: {}", e)))?;

        // Collect all files
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(&extract_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path() != &extract_path)
        {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(&extract_path)
                .map_err(|e| PluginError::InstallerError(format!("Failed to get relative path: {}", e)))?;

            let content = std::fs::read(path)
                .map_err(|e| PluginError::InstallerError(format!("Failed to read file: {}", e)))?;

            files.push(super::PluginFile {
                path: relative_path.to_string_lossy().to_string(),
                content,
                executable: is_executable(path),
            });
        }

        Ok(super::PluginPackage {
            plugin_id: manifest.name.clone(),
            version: manifest.version.clone(),
            manifest,
            files,
            install_path: extract_path,
        })
    }
}

fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }
    false
}

/// Review submission
#[derive(Debug, serde::Serialize)]
pub struct ReviewSubmission {
    pub rating: u32,
    pub title: String,
    pub content: String,
}

/// Package for publishing a plugin
#[derive(Debug, serde::Serialize)]
pub struct PublishPackage {
    pub manifest: PluginManifest,
    pub files: Vec<PublishFile>,
    pub changelog: String,
    pub draft: bool,
}

/// File for publishing
#[derive(Debug, serde::Serialize)]
pub struct PublishFile {
    pub path: String,
    pub content: String, // Base64 encoded
    pub executable: bool,
}

/// Result of publishing
#[derive(Debug, serde::Deserialize)]
pub struct PublishResult {
    pub plugin_id: String,
    pub version: String,
    pub status: String,
    pub url: Option<String>,
}

/// Download statistics
#[derive(Debug, serde::Deserialize)]
pub struct DownloadStats {
    pub total_downloads: u64,
    pub daily_downloads: Vec<DailyDownloads>,
    pub by_version: HashMap<String, u64>,
    pub by_platform: HashMap<String, u64>,
}

/// Daily downloads
#[derive(Debug, serde::Deserialize)]
pub struct DailyDownloads {
    pub date: String,
    pub downloads: u64,
}
