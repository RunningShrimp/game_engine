//! Plugin routes

use actix_web::{web, HttpResponse, Scope};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{ApiResponse, Plugin, PaginationParams};
use crate::handlers::PluginHandler;
use validator::Validate;

pub fn routes() -> Scope {
    web::scope("/api/v1/plugins")
        .route("/search", web::get().to(search_plugins))
        .route("/{plugin_id}", web::get().to(get_plugin))
        .route("/{plugin_id}/versions", web::get().to(get_plugin_versions))
        .route("/{plugin_id}/download", web::get().to(get_plugin_download))
        .route("", web::post().to(create_plugin))
        .route("/{plugin_id}", web::put().to(update_plugin))
        .route("/{plugin_id}", web::delete().to(delete_plugin))
}

/// Search plugins
pub async fn search_plugins(
    pool: web::Data<PgPool>,
    query: web::Query<SearchQuery>,
    pagination: web::Query<PaginationParams>,
) -> HttpResponse {
    let handler = PluginHandler::new(pool.get_ref().clone());

    match handler.search(
        query.q.clone(),
        query.category.clone(),
        query.tags.clone(),
        query.sort_by.clone(),
        pagination.page(),
        pagination.limit(),
    ).await {
        Ok(plugins) => HttpResponse::Ok().json(ApiResponse::success(plugins)),
        Err(e) => {
            log::error!("Search error: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
    category: Option<String>,
    tags: Option<String>,
    sort_by: Option<String>,
}

/// Get plugin by ID
pub async fn get_plugin(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let plugin_id = path.into_inner();

    match Uuid::parse_str(&plugin_id) {
        Ok(uuid) => {
            let handler = PluginHandler::new(pool.get_ref().clone());
            match handler.get_plugin(uuid).await {
                Ok(plugin) => HttpResponse::Ok().json(ApiResponse::success(plugin)),
                Err(e) => HttpResponse::NotFound().json(ApiResponse::<()>::error(e.to_string()))
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid plugin ID".to_string()))
    }
}

/// Get plugin versions
pub async fn get_plugin_versions(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let plugin_id = path.into_inner();

    match Uuid::parse_str(&plugin_id) {
        Ok(uuid) => {
            let handler = PluginHandler::new(pool.get_ref().clone());
            match handler.get_versions(uuid).await {
                Ok(versions) => HttpResponse::Ok().json(ApiResponse::success(versions)),
                Err(e) => HttpResponse::NotFound().json(ApiResponse::<()>::error(e.to_string()))
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid plugin ID".to_string()))
    }
}

/// Get plugin download URL
pub async fn get_plugin_download(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    query: web::Query<DownloadQuery>,
) -> HttpResponse {
    let plugin_id = path.into_inner();

    match Uuid::parse_str(&plugin_id) {
        Ok(uuid) => {
            let handler = PluginHandler::new(pool.get_ref().clone());
            match handler.get_download_url(uuid, query.version.clone()).await {
                Ok(response) => {
                    // Track download asynchronously
                    let _ = handler.track_download(
                        uuid,
                        query.version.clone().unwrap_or_default(),
                        query.platform.clone().unwrap_or_else(|| "unknown".to_string()),
                        query.engine_version.clone(),
                    ).await;

                    HttpResponse::Ok().json(ApiResponse::success(response))
                }
                Err(e) => HttpResponse::NotFound().json(ApiResponse::<()>::error(e.to_string()))
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid plugin ID".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct DownloadQuery {
    version: Option<String>,
    platform: Option<String>,
    engine_version: Option<String>,
}

/// Create plugin
pub async fn create_plugin(
    pool: web::Data<PgPool>,
    payload: web::Json<CreatePluginRequest>,
) -> HttpResponse {
    match payload.validate() {
        Ok(_) => {
            let handler = PluginHandler::new(pool.get_ref().clone());
            match handler.create_plugin(payload.into_inner()).await {
                Ok(plugin) => HttpResponse::Created().json(ApiResponse::success(plugin)),
                Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string()))
            }
        }
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string()))
    }
}

#[derive(Debug, Deserialize, Validate)]
struct CreatePluginRequest {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[validate(length(max = 500))]
    description: String,
    categories: Vec<String>,
    tags: Vec<String>,
    license: String,
    homepage: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
    manifest: serde_json::Value,
}

/// Update plugin
pub async fn update_plugin(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdatePluginRequest>,
) -> HttpResponse {
    let plugin_id = path.into_inner();

    match Uuid::parse_str(&plugin_id) {
        Ok(uuid) => {
            match payload.validate() {
                Ok(_) => {
                    let handler = PluginHandler::new(pool.get_ref().clone());
                    match handler.update_plugin(uuid, payload.into_inner()).await {
                        Ok(plugin) => HttpResponse::Ok().json(ApiResponse::success(plugin)),
                        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string()))
                    }
                }
                Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string()))
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid plugin ID".to_string()))
    }
}

#[derive(Debug, Deserialize, Validate)]
struct UpdatePluginRequest {
    #[validate(length(min = 1, max = 100))]
    name: Option<String>,
    #[validate(length(max = 500))]
    description: Option<String>,
    categories: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    license: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
    manifest: Option<serde_json::Value>,
}

/// Delete plugin
pub async fn delete_plugin(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let plugin_id = path.into_inner();

    match Uuid::parse_str(&plugin_id) {
        Ok(uuid) => {
            let handler = PluginHandler::new(pool.get_ref().clone());
            match handler.delete_plugin(uuid).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => HttpResponse::NotFound().json(ApiResponse::<()>::error(e.to_string()))
            }
        }
        Err(_) => HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid plugin ID".to_string()))
    }
}
