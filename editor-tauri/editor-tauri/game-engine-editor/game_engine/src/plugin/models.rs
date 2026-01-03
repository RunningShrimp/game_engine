//! Data models for the plugin marketplace

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: PluginAuthor,
    pub version: String,
    pub latest_version: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub screenshots: Vec<String>,
    pub videos: Vec<VideoInfo>,
    pub rating: RatingInfo,
    pub downloads: u64,
    pub created_at: String,
    pub updated_at: String,
    pub dependencies: Vec<PluginDependency>,
    pub compatibility: CompatibilityInfo,
    pub pricing: PricingInfo,
    pub manifest: PluginManifest,
}

/// Plugin author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuthor {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub website: Option<String>,
}

/// Rating and review information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingInfo {
    pub average: f32,
    pub count: u32,
    pub distribution: HashMap<u32, u32>, // 5 stars -> count, 4 stars -> count, etc.
}

/// Video information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub url: String,
    pub thumbnail: String,
    pub title: String,
    pub duration: Option<u32>,
}

/// Plugin dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub version_requirement: String, // e.g., ">=1.0.0", "2.x"
    pub optional: bool,
}

/// Compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub engine_version_min: String,
    pub engine_version_max: Option<String>,
    pub platforms: Vec<String>, // "windows", "macos", "linux", "ios", "android"
    pub features: Vec<String>,  // Required engine features
}

/// Pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    pub pricing_type: PricingType,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub trial_available: bool,
    pub subscription: Option<SubscriptionInfo>,
}

/// Pricing type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PricingType {
    Free,
    Paid,
    Freemium,
    Subscription,
}

/// Subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub monthly: Option<f64>,
    pub yearly: Option<f64>,
    pub currency: String,
}

/// Plugin manifest (metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub entry_point: String,
    pub permissions: Vec<String>,
    pub resources: Vec<ResourceInfo>,
    pub commands: Vec<CommandInfo>,
    pub settings: Vec<SettingInfo>,
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub path: String,
    pub resource_type: String,
    pub description: String,
}

/// Command information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub id: String,
    pub title: String,
    pub category: String,
    pub icon: Option<String>,
    pub keybinding: Option<String>,
}

/// Setting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingInfo {
    pub key: String,
    pub title: String,
    pub description: String,
    pub setting_type: String,
    pub default_value: serde_json::Value,
    pub options: Option<Vec<SettingOption>>,
}

/// Setting option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingOption {
    pub label: String,
    pub value: serde_json::Value,
}

/// Plugin version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    pub version: String,
    pub changelog: String,
    pub download_url: String,
    pub file_size: u64,
    pub sha256: String,
    pub published_at: String,
    pub compatibility: CompatibilityInfo,
}

/// Search filters
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub pricing_type: Option<PricingType>,
    pub min_rating: Option<f32>,
    pub sort_by: Option<SortBy>,
    pub platforms: Vec<String>,
}

/// Sort options
#[derive(Debug, Clone, Copy)]
pub enum SortBy {
    Relevance,
    Downloads,
    Rating,
    Updated,
    Name,
}

/// Plugin package for installation
#[derive(Debug, Clone)]
pub struct PluginPackage {
    pub plugin_id: String,
    pub version: String,
    pub manifest: PluginManifest,
    pub files: Vec<PluginFile>,
    pub install_path: std::path::PathBuf,
}

/// File in a plugin package
#[derive(Debug, Clone)]
pub struct PluginFile {
    pub path: String,
    pub content: Vec<u8>,
    pub executable: bool,
}

/// Review for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub id: String,
    pub plugin_id: String,
    pub user: ReviewUser,
    pub rating: u32,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub helpful_count: u32,
}

/// Review user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewUser {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
}

/// Marketplace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_plugins: u32,
    pub total_downloads: u64,
    pub active_developers: u32,
    pub categories: HashMap<String, u32>,
}
