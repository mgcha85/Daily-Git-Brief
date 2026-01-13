use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, sse::{Event, KeepAlive, Sse}},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tracing::{info, error};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use futures::stream::Stream;

use crate::db::Database;
use crate::config::Config;
use crate::models::{TrendingRepoResponse, LanguageTrend};
use crate::services::DataCollector;

pub struct AppState {
    pub db: Database,
    pub config: Config,
    pub progress_tx: tokio::sync::broadcast::Sender<crate::models::CollectionStatus>,
    pub is_collecting: Arc<AtomicBool>,
}

#[derive(Debug, Deserialize)]
pub struct DateQuery {
    pub date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CollectResponse {
    pub message: String,
    pub collected_count: usize,
}

// GET /api/trends
pub async fn get_trends(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DateQuery>,
) -> impl IntoResponse {
    let date = query.date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    match state.db.get_trending_repos(&date) {
        Ok(repos) => {
            let mut response_repos: Vec<TrendingRepoResponse> = Vec::new();
            
            for (rank, repo) in repos.into_iter().enumerate() {
                // Get languages for this repo
                let languages = state.db
                    .get_repo_languages(&date, repo.repo_id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|l| crate::models::LanguageInfo {
                        language: l.language,
                        percentage: l.percentage,
                    })
                    .collect();

                response_repos.push(TrendingRepoResponse {
                    rank: rank + 1,
                    repo_id: repo.repo_id,
                    repo_name: repo.repo_name.clone(),
                    github_url: format!("https://github.com/{}", repo.repo_name),
                    primary_language: repo.primary_language,
                    languages,
                    description: repo.description,
                    korean_summary: repo.korean_summary,
                    stars: repo.stars,
                    forks: repo.forks,
                    total_score: repo.total_score,
                });
            }

            Json(ApiResponse {
                success: true,
                data: Some(response_repos),
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to get trending repos: {}", e);
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

// GET /api/languages/daily
pub async fn get_daily_languages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DateQuery>,
) -> impl IntoResponse {
    let date = query.date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    match state.db.get_daily_language_trends(&date) {
        Ok(trends) => Json(ApiResponse {
            success: true,
            data: Some(trends),
            error: None,
        }),
        Err(e) => {
            error!("Failed to get daily language trends: {}", e);
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

// GET /api/languages/weekly
pub async fn get_weekly_languages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DateQuery>,
) -> impl IntoResponse {
    let date = query.date.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    match state.db.get_weekly_language_trends(&date) {
        Ok(trends) => Json(ApiResponse {
            success: true,
            data: Some(trends),
            error: None,
        }),
        Err(e) => {
            error!("Failed to get weekly language trends: {}", e);
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

// POST /api/collect
pub async fn trigger_collect(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check if already running
    if state.is_collecting.load(Ordering::SeqCst) {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Collection already in progress".to_string()),
            }),
        ).into_response();
    }

    info!("Manual data collection triggered (async)");
    state.is_collecting.store(true, Ordering::SeqCst);
    
    let is_collecting = state.is_collecting.clone();
    let tx = state.progress_tx.clone();

    // Spawn background task
    tokio::spawn(async move {
        let collector = DataCollector::new(&state.config, state.db.clone());
        match collector.collect(Some(tx)).await {
            Ok(count) => info!("Background collection complete: {} repos", count),
            Err(e) => error!("Background collection failed: {}", e),
        }
        // Reset flag
        is_collecting.store(false, Ordering::SeqCst);
    });

    // Return immediate response with 202 Accepted
    (
        StatusCode::ACCEPTED,
        Json(ApiResponse {
            success: true,
            data: Some(CollectResponse {
                message: "Data collection started in background. Connect to /api/collect/progress for updates.".to_string(),
                collected_count: 0,
            }),
            error: None,
        }),
    ).into_response()
}

// GET /api/collect/progress
pub async fn sse_progress(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.progress_tx.subscribe();
    
    let stream = BroadcastStream::new(rx).map(|msg| {
        match msg {
            Ok(status) => {
                let json = serde_json::to_string(&status).unwrap_or_default();
                Event::default().data(json)
            }
            // If channel lag or other error, we can send a comment or ignore
            Err(_) => Event::default().comment("keep-alive"),
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// GET /health
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
