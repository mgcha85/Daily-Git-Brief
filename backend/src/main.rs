mod api;
mod config;
mod db;
mod models;
mod services;

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{AppState, get_trends, get_daily_languages, get_weekly_languages, trigger_collect, health_check};
use crate::config::Config;
use crate::db::Database;
use crate::services::DataCollector;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Daily-Git-Brief backend");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded");
    info!("  OSS Insight URL: {}", config.oss_insight_base_url);
    info!("  DeepSeek URL: {}", config.deepseek_base_url);
    info!("  Language threshold: {}%", config.language_threshold * 100.0);
    info!("  Database path: {}", config.database_path);

    // Initialize database
    let db = Database::new(&config.database_path)?;
    info!("Database initialized");

    // Setup scheduler for daily collection at UTC 00:00
    let scheduler = JobScheduler::new().await?;
    
    let collector_config = config.clone();
    let collector_db = db.clone();
    
    scheduler.add(
        Job::new_async("0 0 0 * * *", move |_uuid, _l| {
            let config = collector_config.clone();
            let db = collector_db.clone();
            Box::pin(async move {
                info!("Scheduled data collection starting");
                let collector = DataCollector::new(&config, db);
                match collector.collect().await {
                    Ok(count) => info!("Scheduled collection complete: {} repos", count),
                    Err(e) => error!("Scheduled collection failed: {}", e),
                }
            })
        })?
    ).await?;
    
    scheduler.start().await?;
    info!("Scheduler started (daily at UTC 00:00)");

    // Create app state
    let state = Arc::new(AppState { db, config: config.clone() });

    // Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/trends", get(get_trends))
        .route("/api/languages/daily", get(get_daily_languages))
        .route("/api/languages/weekly", get(get_weekly_languages))
        .route("/api/collect", post(trigger_collect))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    info!("Server starting at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
