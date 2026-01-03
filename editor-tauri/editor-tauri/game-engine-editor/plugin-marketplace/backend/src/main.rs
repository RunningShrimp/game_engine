//! Plugin Marketplace Backend Server
//!
//! RESTful API server for the game engine plugin marketplace

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use sqlx::PgPool;
use std::env;

mod models;
mod routes;
mod services;
mod handlers;
mod middleware;

use routes::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // JWT secret
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    // AWS S3 configuration
    let s3_bucket = env::var("S3_BUCKET")
        .unwrap_or_else(|_| "plugin-marketplace".to_string());

    let bind_address = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    log::info!("Starting server at {}", bind_address);

    // App state
    let app_state = web::Data::new(AppState {
        db: pool,
        jwt_secret,
        s3_bucket,
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            // Plugin routes
            .service(routes::plugins::search_plugins)
            .service(routes::plugins::get_plugin)
            .service(routes::plugins::get_plugin_versions)
            .service(routes::plugins::get_plugin_download)
            .service(routes::plugins::create_plugin)
            .service(routes::plugins::update_plugin)
            .service(routes::plugins::delete_plugin)
            // Review routes
            .service(routes::reviews::get_reviews)
            .service(routes::reviews::create_review)
            .service(routes::reviews::update_review)
            .service(routes::reviews::delete_review)
            // User routes
            .service(routes::users::register)
            .service(routes::users::login)
            .service(routes::users::get_profile)
            .service(routes::users::update_profile)
            // Stats routes
            .service(routes::stats::get_marketplace_stats)
            .service(routes::stats::get_plugin_stats)
            // Analytics routes
            .service(routes::analytics::track_download)
            .service(routes::analytics::track_view)
            // Category routes
            .service(routes::categories::list_categories)
            // Health check
            .service(routes::health)
    })
    .bind(&bind_address)?
    .run()
    .await
}

/// Application state
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub s3_bucket: String,
}
