use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use indexer::Index;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    index: Arc<RwLock<Index>>,
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

#[derive(Serialize)]
struct SearchEntry {
    title:          String,
    description:    String,
    url:            String,
    credits:        String,
    level:          u32,
    score:          f32,
    workload_score: f32,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Value>, StatusCode> {
    let q = match params.q {
        Some(q) if !q.trim().is_empty() => q,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let start = Instant::now();
    let results: Vec<SearchEntry> = {
        let idx = state.index.read().await;
        idx.search(&q)
            .into_iter()
            .map(|(doc, score)| SearchEntry {
                title:          doc.title,
                description:    doc.description,
                url:            doc.url,
                credits:        doc.credits,
                level:          doc.level,
                workload_score: doc.workload_score,
                score,
            })
            .collect()
    };
    let query_ms = start.elapsed().as_millis();
    let total = results.len();

    Ok(Json(json!({
        "query_ms": query_ms,
        "total":    total,
        "results":  results,
    })))
}

async fn stats(State(state): State<AppState>) -> Json<Value> {
    let s = state.index.read().await.stats();
    let levels: serde_json::Map<String, Value> = s
        .levels
        .into_iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect();
    Json(json!({
        "total_courses":       s.total_courses,
        "avg_workload_score":  (s.avg_workload_score * 100.0).round() / 100.0,
        "high_workload_count": s.high_workload_count,
        "low_workload_count":  s.low_workload_count,
        "levels":              levels,
    }))
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    println!("Crawling EECS courses from static data...");
    let docs = crawler::crawl_all().await;

    let doc_count = docs.len();
    println!("Indexing {doc_count} courses...");
    let mut idx = Index::new();
    for doc in docs {
        if let Err(e) = idx.add_document(doc) {
            eprintln!("  index warn: {e}");
        }
    }
    println!("Indexed {doc_count} courses.");

    let state = AppState {
        index: Arc::new(RwLock::new(idx)),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/search", get(search))
        .route("/stats", get(stats))
        .with_state(state)
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("Binding to {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Successfully bound to {addr}");
    axum::serve(listener, app).await.unwrap();
}
