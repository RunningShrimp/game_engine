//! API routes

pub mod plugins;
pub mod reviews;
pub mod users;
pub mod stats;
pub mod analytics;
pub mod categories;

use actix_web::{web, HttpResponse};
use crate::models::ApiResponse;

/// Health check endpoint
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}
