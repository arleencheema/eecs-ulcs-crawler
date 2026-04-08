# API Crate

Axum web server. Builds the search index on startup, then serves queries.

## Startup Sequence

1. Call crawler::crawl_all() to get Vec<Document>
2. Build indexer::Index from documents, computing workload scores
3. Wrap index in Arc<RwLock<Index>> inside AppState
4. Bind TcpListener on 0.0.0.0:3000
5. Serve — index is read-only after startup, no locking overhead on search

Future: replace step 1-2 with sqlx load from SQLite, re-crawl on a background
Tokio task every 24h.

## AppState

  #[derive(Clone)]
  struct AppState {
      index: Arc<RwLock<Index>>,
  }

Clone is cheap — Arc just increments a reference count.
RwLock allows concurrent reads (multiple simultaneous searches) with exclusive
writes (re-indexing). Use index.read() in search handler, index.write() only
during re-crawl.

## Endpoints

GET /health
  Returns: {"status": "ok"}
  Use to verify the server is up before running searches.

GET /search?q=<query>
  Returns: {
    "results": [
      {
        "title": "Web Systems",
        "description": "...",
        "url": "https://...",
        "credits": "4",
        "level": 400,
        "score": 3.847,
        "workload_score": 0.72
      }
    ],
    "total": 5,
    "query_ms": 2
  }
  Max results returned: 10
  Empty query returns empty results, not an error.

POST /reindex  (future — not yet implemented)
  Triggers a fresh crawl and rebuilds the index in the background.
  Returns 202 Accepted immediately.

## Example curl Commands

  curl http://localhost:3000/health

  curl "http://localhost:3000/search?q=operating+systems"

  curl "http://localhost:3000/search?q=easy+ULCS+low+workload"

  curl "http://localhost:3000/search?q=machine+learning" | jq .

## CORS

tower-http CorsLayer is configured to allow all origins so the Next.js
frontend on localhost:3001 can call the API during development.
Tighten this to specific origins before any public deployment.