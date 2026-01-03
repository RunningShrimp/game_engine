//! Database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Plugin model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Plugin {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub author_id: Uuid,
    pub version: String,
    pub latest_version: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub screenshots: Vec<String>,
    pub videos: Vec<serde_json::Value>,
    pub rating_average: f32,
    pub rating_count: i32,
    pub downloads: i64,
    pub pricing_type: String,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub manifest: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Plugin version model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PluginVersion {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub version: String,
    pub changelog: String,
    pub download_url: String,
    pub file_size: i64,
    pub sha256: String,
    pub status: String,
    pub published_at: DateTime<Utc>,
}

/// User model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub bio: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Review model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub title: String,
    pub content: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Category model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
    pub plugin_count: i32,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

/// Download analytics model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DownloadEvent {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub version: String,
    pub platform: String,
    pub engine_version: Option<String>,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// View analytics model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ViewEvent {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub referrer: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
            message: None,
        }
    }
}

/// Pagination params
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl PaginationParams {
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(20).min(100)
    }

    pub fn offset(&self) -> u32 {
        (self.page() - 1) * self.limit()
    }
}
