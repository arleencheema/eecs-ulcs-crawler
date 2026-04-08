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
        .with_state(state)
        .layer(cors);

    let addr = "0.0.0.0:3001";
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
