# Week 3: Build a CRUD Microservice in an Unfamiliar Language
**Project:** Polyglot Microservice — EECS ULCS Crawler  
**Language:** Rust  
**Domain:** UMich EECS Upper-Level Course Search Engine

---

## PART 1: LANGUAGE SELECTION RATIONALE

**Chosen Language:** Rust  
**Project Domain:** A niche search engine for University of Michigan EECS upper-level courses (300–499), with BM25 text ranking fused with custom workload signals, served via a REST API and a Next.js frontend.

### Why This Language for This Problem
Rust is uniquely suited for a search engine because search is a computation-heavy workload where BM25 scoring runs across every document in the index on every query. Rust's zero-cost abstractions deliver this performance without a garbage collector, and its ownership model enforces thread safety at compile time — critical for the shared index state the API serves concurrently. No other language on the candidate list provides both performance guarantees and compile-time concurrency safety.

### Trade-offs Considered

| Factor | Rust | Alternative Considered (Go) |
|--------|------|-----------------------------|
| Problem fit | Excellent — zero-cost abstractions, no GC pauses during search | Good — fast, simple concurrency, but less expressive type system |
| Ecosystem/libraries | Strong for systems work (axum, sqlx, tokio); smaller web ecosystem than Go | Large, mature web ecosystem; excellent stdlib |
| Performance characteristics | Best-in-class; predictable latency, no GC | Near-Rust performance; GC but low-pause |
| Team maintainability | Steep learning curve; borrow checker adds friction | Simpler syntax; easier to onboard new engineers |
| Deployment/operations | Small binaries; no runtime dependency | Small binaries; no runtime dependency |

### Business Justification
Rust minimizes infrastructure cost for a search engine — a single Rust binary handles hundreds of concurrent queries on a free-tier Render instance without the memory overhead of a JVM or Python runtime. For a performance-sensitive internal tool serving a university department, this means reliable response times without paid infrastructure.

### What I'm Giving Up
Rust's borrow checker and ownership system significantly slow down initial development compared to Go or Python. Iteration speed suffers — what takes 10 minutes to prototype in Python took 45 minutes in Rust. For a time-constrained project, this was the most significant cost.

---

## PART 2: ARCHITECTURE DECISION RECORD

### API Design
- **Endpoints:**
  - `GET /health` — liveness check, returns `{"status": "ok"}`
  - `GET /search?q=<query>` — returns ranked course results with scores, workload badges, and metadata
- **Authentication:** None — this is an internal read-only service with no mutation endpoints and no sensitive data
- **Error format:**
  ```json
  { "error": "description of what went wrong" }
  ```
  With appropriate HTTP status codes (400 for bad input, 500 for server errors)

### Database Design
- **Database:** SQLite (planned), static JSON (current) — chosen because the dataset is small (20–50 courses), read-heavy, and requires no concurrent writes. SQLite is a single file, zero-ops, and sufficient for this scale.
- **Key tables:**
  - `documents` — id, title, description, credits, level, url, workload_score
  - `crawl_log` — last_crawled timestamp for cache invalidation
- **Key relationships:** No joins needed — documents are independent, flat records

### Project Structure
```
mini-google/
├── Cargo.toml              # workspace root
├── crates/
│   ├── common/             # shared types: Document, AppError — no logic
│   ├── indexer/            # tokenizer, inverted index, BM25, workload signals — pure logic, no I/O
│   ├── crawler/            # data ingestion — currently static JSON, designed for live crawl swap
│   └── api/                # axum REST server, AppState, handlers
├── data/
│   └── courses.json        # static course data (20 EECS ULCS courses)
└── frontend/               # Next.js app (outside Cargo workspace)
```

Each crate has exactly one responsibility. The `indexer` crate has no I/O — every function is a pure transformation on data. This makes the BM25 logic fully testable without network or filesystem mocks.

### Error Handling Strategy
- Library crates (`common`, `indexer`, `crawler`) define errors using `thiserror` — typed, composable
- Binary entrypoints (`api`) use `anyhow` for flexible error propagation
- No `.unwrap()` in crawler or API paths — all errors either propagate with `?` or fall back gracefully
- The client sees consistent JSON error responses with meaningful messages; 500s are logged server-side without leaking internals

### Testing Strategy
- Unit tests live in `indexer/src/index.rs` under `#[cfg(test)]`
- Tests cover: tokenizer correctness, BM25 ranking order, workload signal computation, duplicate document handling
- Integration testing via `curl` against the running API — no automated integration test harness yet
- The `indexer` crate's pure-function design means unit tests run with no setup or teardown

---

## PART 3: CONTEXT JOURNAL

### Pass 1: Structure
**What I told AI Code:** I provided the full architecture plan upfront — workspace structure, crate names and responsibilities, which external crates to use (axum, tokio, reqwest, scraper, sqlx, serde), and the Document struct definition. First prompt: "Create a Cargo workspace called mini-google with four member crates: common, indexer, crawler, and api. Use Rust 2021 edition."

**What worked well:** Providing the complete workspace structure in the first prompt produced clean, compilable scaffolding immediately. Specifying external crate names prevented Claude Code from choosing unfamiliar or unmaintained alternatives.

**What I had to refine:** The initial `common/Cargo.toml` was missing the `[package]` header entirely — it had only dependencies. Claude Code generated a malformed manifest that caused `cargo build` to fail with a cryptic error. I had to diagnose this myself before asking Claude Code to fix it.

**Key decisions I made:** Four-crate workspace split, `thiserror` for library errors vs. `anyhow` in binaries, `Arc<RwLock<Index>>` for shared API state, `include_str!` for embedding the JSON at compile time rather than reading it at runtime.

### Pass 2: Correctness
**What I told AI Code:** "Fix the tokenizer to use a stopword list instead of a length-based filter, because length-based filtering removes meaningful CS terms like 'os', 'ml', 'ai'." And: "Implement BM25 scoring with k1=1.2 and b=0.75. The final score should blend BM25 with workload signal: `final_score = 0.7 * bm25_score + 0.3 * (1.0 - doc.workload_score)`."

**Where AI Code surprised me:** The BM25 implementation was correct on the first attempt. I expected to iterate on the math but the formula implementation was clean and the unit tests passed immediately.

**What I had to push back on:** Claude Code's initial tokenizer used a length filter of `< 2` which filtered out one-character tokens but kept two-character tokens like "an". The test `tokenize_filters_short_tokens` failed because "an" was expected to be absent. I diagnosed that the fix was a stopword list, not a stricter length filter, and directed the fix accordingly.

### Pass 3: Quality
**What I told AI Code:** "Rewrite the crawler to use static JSON loaded with `include_str!` — remove all reqwest, scraper, fantoccini, and chromiumoxide dependencies. The crawler crate should have no network I/O." And: "Write CLAUDE.md files for the workspace root, indexer crate, and api crate documenting the architecture decisions, BM25 parameters, endpoint contracts, and known constraints."

**What I learned about Rust:** `include_str!` embeds file content at compile time, making the binary self-contained. The path is relative to the source file, not the workspace root — `../../data/courses.json` from `crawler/src/lib.rs` resolves correctly. I also learned that `Arc<T>` clone is cheap (just increments a reference count) and that `RwLock` allows concurrent reads with exclusive writes — the right primitive for a read-heavy search index.

**What a senior code review would flag:** The 70/30 BM25/workload blend ratio was chosen heuristically with no eval set. A senior engineer would want at least 10–20 labeled queries to validate the weights. The workload signal keywords are also hardcoded — a production system would want these configurable or learned from data.

---

## PART 4: ARCHITECTURE DECISIONS — FINAL

### What Changed from My Plan
The biggest deviation was abandoning live web crawling entirely. The original plan had the crawler scraping `bulletin.engin.umich.edu` for real-time course data. Three different approaches failed: plain `reqwest` got 403 from Cloudflare, `chromedriver`/`fantoccini` had a Chrome version mismatch, and `chromiumoxide` had a browser handler race condition. The LSA API required Okta SSO not available programmatically without interactive MFA. The decision to use static JSON was made after approximately 4 hours of crawling attempts — recognizing that the data ingestion problem was blocking progress on the search engine itself, which was the actual deliverable.

SQLite persistence (Step 6) was also deferred. The static JSON loads in milliseconds at startup and Supabase is wired in for future use, but the current deployment reads from the embedded JSON on every startup.

### Decisions I'm Confident About
**Four-crate workspace split.** The `indexer` crate's isolation from I/O made BM25 testing fast and reliable. Every scoring test runs in under 1ms with no setup. This paid off immediately when debugging the tokenizer — I could run `cargo test -p indexer` in isolation without starting the API or hitting the network.

**Axum for the API layer.** Axum's extractor pattern (`State`, `Query`) made handler signatures clean and testable. The tower middleware stack (CORS, tracing) composed cleanly with the application router. No regrets here.

**Static JSON with `include_str!`.** Embedding the data at compile time means the Render deployment is a single self-contained binary with no file system dependencies. The deployment just works.

### Decisions I'd Reconsider
**Not building an evaluation harness for the ranking.** The 70/30 BM25/workload split and the workload keyword weights were chosen without data. A 20-query labeled eval set would take 2 hours to build and would make the ranking decisions defensible. I'd do this first before tuning any weights in a future iteration.

**Attempting live crawling before validating the data source.** I should have verified the bulletin URL was scrapable (by checking for bot protection in a browser's DevTools Network tab) before writing any crawler code. The Cloudflare check would have taken 5 minutes and would have redirected me to the static JSON approach immediately.

### What Rust Does Well for This
Rust's type system made the `Document` struct the single source of truth across all four crates — if I added a field to `Document` in `common`, the compiler immediately flagged every place in `indexer`, `crawler`, and `api` that needed updating. This refactoring safety was genuinely valuable when adding `workload_score` mid-project. The `?` operator for error propagation also made the API handlers clean without hiding error paths.

### What Rust Made Harder
The borrow checker created friction when building the index. Passing `&mut Index` through async code required careful lifetime reasoning that Python or Go would have handled transparently. The `Arc<RwLock<Index>>` pattern is idiomatic but adds conceptual overhead that slowed initial implementation. Compile times were also significant — a full `cargo build` on the workspace took 45–60 seconds, which makes the tight iteration loop that AI Code benefits from feel slower.

---

## PART 5: REFLECTION

### On Language Identity
Building in Rust changed how I think about Python and JavaScript in a specific way: I now see their permissiveness as a design choice with real costs, not just a feature. When Rust's borrow checker rejected my first attempt at sharing the index across async tasks, I had to understand the actual concurrency model — not just write code that seemed right. In Python I would have written the same code, it would have run, and I would never have understood why a race condition was possible. The discomfort of Rust's compiler made me a more precise thinker about memory and ownership, and I'll carry that precision back into other languages.

### On AI Code as a Tool
AI Code excelled at boilerplate-heavy Rust work: struct definitions, trait implementations, Cargo.toml configuration, and axum handler signatures. These are high-syntax, low-decision tasks where knowing idiomatic Rust matters more than architectural judgment. AI Code fell short when the problem was environmental rather than logical — the Cloudflare blocking, the Chrome version mismatch, and the Render port binding issue all required me to read error output, form a hypothesis, and direct a specific fix. AI Code's suggestions in those moments were sometimes plausible-sounding but wrong. The context that made the biggest difference was the architecture plan provided upfront — prompts that included the ADR produced significantly better output than prompts that described only the immediate task.

### On the Three-Pass Method
The structure → correctness → quality progression worked well for this project. The most important boundary was between Pass 1 and Pass 2: building the happy path first without worrying about error handling meant I had a working foundation to reason about before adding complexity. The place where the method broke down slightly was the crawler — I iterated on the data ingestion approach so many times (reqwest → scraper → chromedriver → chromiumoxide → static JSON) that it blurred the pass boundaries. In retrospect, the crawler should have been validated as a standalone spike before being integrated into the workspace. I would use the three-pass method again, but add a "spike" phase before Pass 1 for any component with significant external dependencies.

### On Ownership
Yes — this feels like my project. The reason is that every major decision was mine: the four-crate workspace split, the BM25 + workload signal fusion, the decision to stop fighting Cloudflare and use static JSON, the Render + Vercel + Supabase deployment stack. AI Code implemented what I specified. When the implementation was wrong — the tokenizer length filter, the missing `[package]` header, the port binding on Render — I diagnosed the problem and directed the fix. The code I can't explain (some of the axum middleware configuration) is the code I'm least confident in, which confirms that ownership tracks understanding.

### On Senior Thinking
Two weeks ago I would have started coding immediately after getting the assignment. This week I wrote an architecture document before opening my terminal. That's the most concrete change. I also made a scope management decision that I wouldn't have made before: stopping the live crawling work after recognizing it was blocking the actual deliverable. A junior engineer treats every problem as something to solve completely. A senior engineer asks whether solving this problem is the best use of the time budget. Switching to static JSON was the senior call — it unblocked 6 hours of progress on the search engine in exchange for a known, documented limitation.

---

## DECISION LOG ENTRIES

### Entry 1 — Language Selection

**Decision:** Use Rust as the implementation language for the EECS ULCS search engine

**Context:** This was a Week 3 assignment requiring an unfamiliar language. I needed a language that could handle text indexing, BM25 scoring, and serve a REST API efficiently. As a CS student familiar with C++ and Python, Rust was the highest-value stretch.

**Reasoning:** Rust is uniquely suited for a search engine because BM25 scoring runs across every document in the index on every query — a computation-heavy workload where garbage collection pauses would cause latency spikes. Rust's ownership model also enforces thread safety at compile time, which is critical for the `Arc<RwLock<Index>>` shared state pattern the API uses. Go was the closest alternative but lacks Rust's type-level expressiveness for designing the indexer crate's pure-function interface.

**Alternatives:** Go (simpler concurrency, faster to learn), Python (familiar ecosystem, fast iteration), TypeScript/Node.js (full-stack, huge ecosystem)

**Trade-offs:** Gained memory safety without GC, excellent performance for search workloads, strict compiler that catches bugs at compile time. Gave up fast iteration speed — Rust's ownership system added significant learning overhead and the borrow checker caused many compile errors early on.

**AI Code note:** Providing the full architecture plan upfront dramatically improved output quality. When I told Claude Code "create a Cargo workspace with four member crates: common, indexer, crawler, api" I got clean compilable output immediately. Vague prompts produced unfocused results.

---

### Entry 2 — Cargo Workspace Architecture

**Decision:** Structure the project as a Cargo workspace with four separate crates instead of a single binary

**Context:** The search engine had four clearly distinct concerns: shared types, search logic, data ingestion, and HTTP serving. A monolithic structure would have made testing the BM25 logic difficult and coupled the crawler to the API.

**Reasoning:** The single-responsibility principle applied at the crate level. The `indexer` crate has no I/O — it's pure functions operating on data structures. This means every BM25 test runs in milliseconds with no network or filesystem setup. If the crawler and indexer were in the same crate, testing the scoring logic would require mocking HTTP calls. The `common` crate as the shared type layer prevents circular dependencies — every other crate imports from `common` but `common` imports nothing. This is the dependency inversion principle applied to a Rust workspace.

**Alternatives:** Single binary crate with modules, two crates (lib + binary), monorepo with separate Cargo.toml per crate without workspace

**Trade-offs:** Gained clear separation of concerns, independent compilation per crate, ability to unit test the indexer in isolation. Gave up simplicity — workspace configuration added overhead and cross-crate dependency management caused several compile errors early on.

**AI Code note:** Specifying the workspace structure in the very first prompt prevented a major refactor later. Claude Code defaulted to single-crate when not given explicit structure. The CLAUDE.md file created mid-project significantly improved subsequent prompt quality by giving Claude Code persistent architectural context.

---

### Entry 3 — Static JSON vs. Live Crawling

**Decision:** Use a static `data/courses.json` file instead of live web crawling for course data

**Context:** The original plan was to scrape the UMich Engineering Bulletin. Three different crawling approaches failed over approximately 4 hours: plain `reqwest` (Cloudflare 403), `chromedriver`/`fantoccini` (Chrome version mismatch), `chromiumoxide` (browser handler race condition). The LSA API requires Okta SSO with interactive MFA.

**Reasoning:** The search engine's value is in the ranking and retrieval logic, not in the data ingestion pipeline. Continuing to fight Cloudflare was delivering zero value toward the actual project goals. The static JSON approach is also architecturally honest — a production search engine would have a proper ETL pipeline with scheduled crawls, not a one-shot scraper running on startup. By isolating the data concern into `data/courses.json` and loading it with `include_str!`, I preserved the ability to swap in a real crawler later without touching the indexer or API crates.

**Alternatives:** Okta bearer token copied manually from DevTools (fragile, expires), headless Chrome via chromiumoxide (version mismatch issues), LSA API with student credentials (requires interactive MFA)

**Trade-offs:** Gained reliability, zero external dependencies at runtime, fast startup, clean Render deployment. Gave up real-time data freshness — course descriptions are static and won't reflect semester changes automatically.

**AI Code note:** This was the biggest place where I had to override AI Code's suggestions. Claude Code kept proposing more complex crawling solutions. The decision to use static data was mine — I recognized that the data ingestion problem was blocking progress on the actual search engine.

---

### Entry 4 — BM25 + Custom Workload Signal Fusion

**Decision:** Blend BM25 text relevance (70%) with a custom workload signal (30%) for final ranking score

**Context:** The core query use case was "easy ULCS with low workload" — a query that pure text matching handles poorly because "easy" and "low workload" don't appear in course descriptions. Pure BM25 would rank courses whose descriptions mention these words over courses that are genuinely lighter.

**Reasoning:** BM25 is optimal for "what document talks about this topic" queries but fails for "what document represents this property" queries. The workload signal computed from description keywords (exam frequency, project count, lab requirements) is a proxy for the real-world property the user cares about. Fusing it with BM25 rather than using it as a post-filter preserves ranking continuity — a course that is both highly relevant AND low workload scores higher than one that is only low workload. The 70/30 split prioritizes text relevance since most queries are topic-based, with workload as a differentiating signal.

**Alternatives:** Pure BM25 with no signal blending, TF-IDF instead of BM25, separate workload filter rather than signal fusion, Reddit sentiment analysis for workload data

**Trade-offs:** Gained meaningful ranking for workload-aware queries, differentiates the engine from basic keyword search. Gave up interpretability — the blended score is harder to explain, and the 70/30 split was chosen heuristically without a labeled eval set.

**AI Code note:** I specified the exact formula to Claude Code rather than asking it to design the ranking. When I asked it to "design a ranking system," output was overly complex. Specifying `final_score = 0.7 * bm25_score + 0.3 * (1.0 - doc.workload_score)` myself and asking Claude Code to implement it produced clean, correct code immediately.

---

### Entry 5 — Deployment Stack (Render + Vercel + Supabase)

**Decision:** Deploy the Rust API on Render, the Next.js frontend on Vercel, with Supabase as the database layer

**Context:** Needed a free deployment stack that could handle a Rust binary, a Next.js frontend, and a Postgres database. Railway was the initial choice for the API but requires a paid plan for sustained use.

**Reasoning:** The constraint was free tier plus Rust binary support plus no credit card required. Render was the only platform meeting all three without significant ops configuration. Vercel was the natural choice for Next.js since it's built by the same team and handles the build pipeline automatically. Separating API and frontend hosting is architecturally correct — the Rust API can be replaced, scaled, or migrated independently of the frontend. If the API later moves to a paid tier, the Vercel frontend only needs one environment variable updated.

**Alternatives:** Railway (paid after trial), Fly.io (requires CLI setup and credit card), self-hosted VPS (too much ops overhead), keep everything local (doesn't demonstrate deployment)

**Trade-offs:** Gained zero-cost deployment with a live shareable URL suitable for a portfolio. Gave up cold start performance — Render's free tier spins down after 15 minutes of inactivity, causing ~50 second delays on the first request after idle.

**AI Code note:** Deployment configuration was the area where AI Code was least helpful. The Render build command, start command binary path, and PORT env var binding all required iteration that Claude Code didn't anticipate. The most effective prompt pattern was pasting the exact error message and asking for the specific fix, rather than asking Claude Code to set up deployment from scratch.