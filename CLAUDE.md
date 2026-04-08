# Mini Google — UMich Course Search Engine

A niche search engine for University of Michigan EECS upper-level courses (300–499),
built in Rust. Planned second domain: GitHub repo search. The goal is a working
BM25 + custom signal search engine with a Next.js frontend.

## Architecture

Cargo workspace with four crates, each with a single responsibility:

- `common` — shared types only: Document, AppError. No logic.
- `indexer` — tokenizer, inverted index, BM25 scoring, workload signals. Pure logic, no I/O.
- `crawler` — data ingestion. Currently loads from data/courses.json. Live crawling
  via chromedriver/fantoccini targets bulletin.engin.umich.edu but is blocked by
  Cloudflare. LSA API (api.lsa.umich.edu) requires UMich Okta authentication.
- `api` — Axum REST API. Builds index on startup, serves search over HTTP.

Frontend is a Next.js app in frontend/ outside the Cargo workspace.

## Build and Run Commands

### Prerequisites
chromedriver must be running before using the live crawler:
  brew install chromedriver
  xattr -d com.apple.quarantine $(which chromedriver)
  chromedriver --port=9515   # keep this running in a separate terminal

### Backend
  cargo build                  # build all crates
  cargo run -p crawler         # test crawler in isolation
  cargo run -p api             # start API server on http://localhost:3000
  cargo test -p indexer        # run indexer unit tests (most coverage lives here)

### Frontend
  cd frontend
  npm install
  npm run dev                  # starts on http://localhost:3001

### Example search
  curl "http://localhost:3000/search?q=easy+operating+systems+low+workload"
  curl "http://localhost:3000/health"

## Known Issues and Constraints

- bulletin.engin.umich.edu returns a Cloudflare JS challenge to plain HTTP clients.
  Even with a spoofed User-Agent the parse returns 0 courses. Live crawling requires
  a real Chrome session via chromedriver.
- chromedriver is deprecated in Homebrew (removal date 2026-09-01). The quarantine
  flag must be removed with xattr before it will run on macOS.
- LSA API requires Okta SSO — no public client_id/secret available. Bearer tokens
  can be copied manually from DevTools after logging into atlas.ai.umich.edu.
- data/courses.json is the reliable fallback — 20 EECS ULCS courses with real
  descriptions. Use this when chromedriver is unavailable.

## Conventions

- Use `thiserror` for error types in library crates (common, indexer, crawler).
- Use `anyhow` for error handling in binary entrypoints.
- No `.unwrap()` in crawler or api code paths — use `?` or explicit fallbacks.
- All Document fields that can be absent must have `#[serde(default)]`.
- Workload score is always 0.0–1.0. Lower = lighter workload.
