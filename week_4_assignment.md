# Week 4: Building Context Infrastructure
**Project:** Polyglot Microservice — EECS ULCS Crawler  
**Language:** Rust  
**Tier:** Standard

---

## PART 1: CLAUDE.md — PROJECT BRAIN

### What I Built

Three CLAUDE.md files serving different scopes:

- **Workspace root `CLAUDE.md`** — project overview, architecture, 
  build commands, known constraints, deployment stack
- **`indexer/CLAUDE.md`** — BM25 parameters, workload signal formula, 
  tokenizer design decisions, test coverage
- **`api/CLAUDE.md`** — AppState shape, all endpoints with curl examples, 
  CORS configuration, startup sequence

### Testing Methodology

**Test task given to fresh Claude Code session:**  
"Add a `/stats` endpoint that returns total courses indexed, average 
workload score, high/low workload counts, and a breakdown by level."

**Fresh session test results:**

What AI got RIGHT with CLAUDE.md active:
- Used `Arc<RwLock<Index>>` pattern correctly without being told — 
  matched the AppState shape documented in `api/CLAUDE.md`
- Followed the existing handler signature pattern: 
  `State(state): State<AppState>` extractor
- Returned JSON using `serde_json::json!()` macro consistently with 
  the health and search handlers
- Added the `stats()` method to the `Index` struct in the indexer 
  crate rather than computing in the handler — correct separation 
  of concerns per the architecture documented in `indexer/CLAUDE.md`
- Used `f32` for workload scores consistently with the Document struct

What AI got WRONG or needed correction:
- Initially put business logic in the handler instead of the indexer — 
  corrected after pointing to the "indexer is pure logic" constraint
- Used `.unwrap()` on the RwLock read — had to remind it of the 
  no-unwrap rule in crawler/api paths
- Named the method `get_stats()` instead of `stats()` — minor 
  inconsistency with the existing naming pattern

**Iterations made after testing:**

1. Added explicit constraint to `api/CLAUDE.md`:  
   "Never use `.unwrap()` in handler code — use `?` or explicit error returns"
2. Added to `indexer/CLAUDE.md`:  
   "Computation methods go in the Index struct, not in API handlers. 
   The api crate calls index methods; it does not implement logic."
3. Added naming convention to workspace `CLAUDE.md`:  
   "Method names: verb only for actions (search, add), noun for 
   accessors (stats, health) — no get_ prefix"

### Final CLAUDE.md Content

**Workspace root:**
```markdown
# Mini Google — UMich EECS Course Search Engine

## Business Context
A niche search engine for University of Michigan EECS upper-level 
courses (300–499). Built by a CS student to solve a real problem: 
finding courses with low workload that match a topic area. Target 
users are UMich CS students planning schedules. The search engine 
ranks by BM25 text relevance fused with a custom workload signal 
so queries like "easy ULCS low workload" return meaningfully 
ranked results.

## Architecture Overview

### Tech Stack
- Language: Rust (2021 edition)
- Framework: Axum 0.7 with Tokio async runtime
- Database: Static JSON (data/courses.json) via include_str!
- Key Libraries: serde/serde_json, tower-http (CORS), sqlx (future)
- Frontend: Next.js 16 with Tailwind v4

### Project Structure
Cargo workspace with four crates, each with one responsibility:
- common/ — shared types only: Document, AppError. No logic.
- indexer/ — tokenizer, inverted index, BM25, workload signals. 
  Pure logic, no I/O. All business logic lives here.
- crawler/ — data ingestion. Loads data/courses.json via include_str!
- api/ — Axum REST server. Calls indexer methods. No business logic.
- frontend/ — Next.js app. Outside Cargo workspace.

### Data Model
Document { id: u32, title: String, description: String, 
credits: String, level: u32, url: String, workload_score: f32 }
workload_score is always 0.0–1.0. Lower = lighter workload.

## Conventions & Standards

### Naming
- Crates: lowercase, single word
- Structs: PascalCase
- Functions: snake_case, verb-only for actions (search, add), 
  noun for accessors (stats, health) — no get_ prefix
- Files: snake_case

### Error Handling
- Library crates: thiserror with typed AppError enum
- Binary entrypoints: anyhow for flexible propagation
- NEVER use .unwrap() in crawler or api code paths
- Use ? operator or explicit fallbacks
- Return Result<Json<Value>, StatusCode> from handlers

### Testing
- Unit tests in indexer/src/index.rs under #[cfg(test)]
- Run with: cargo test -p indexer
- Test both happy path and error cases
- Integration testing via curl against running API

### Code Style
- No .unwrap() in production paths
- Explicit error messages that help debugging
- #[serde(default)] on optional Document fields

### Logging
- println! for startup sequence (crawl, index, bind)
- eprintln! for warnings and non-fatal errors
- No structured logging yet — plain text

## Build & Run

### Development
cargo run -p api          # start API on $PORT (default 10000)
cargo run -p crawler      # test crawler in isolation
cargo test -p indexer     # run all unit tests

### Common Tasks
cargo build --release -p api    # production build for Render
cd frontend && npm run dev      # start Next.js on port 3000

### Deployment
- API: Render (https://eecs-ulcs-crawler.onrender.com)
  Build: cargo build --release -p api
  Start: /opt/render/project/src/target/release/api
- Frontend: Vercel (https://eecs-ulcs-crawler.vercel.app)
  Root directory: frontend
  Framework: Next.js

## Known Issues & Constraints
- bulletin.engin.umich.edu returns Cloudflare 403 to bots
- LSA API requires Okta SSO — not available programmatically
- data/courses.json is static — does not update automatically
- Render free tier spins down after 15 min inactivity (~50s cold start)
- DO NOT add PORT as an env var on Render — it injects automatically
- chromedriver deprecated in Homebrew (removal 2026-09-01)

## Important Decisions & Rationale
- Four-crate workspace: separation enables isolated unit testing 
  of BM25 logic with no I/O setup
- Static JSON with include_str!: self-contained binary, no file 
  system deps on Render
- Axum over Actix: better tower/tokio integration, cleaner 
  extractor pattern
- Tailwind v4: required for Next.js 16 Turbopack compatibility
```

---

## PART 2: FEATURE BUILD — THE PROOF

### Feature Built: Stats Endpoint + Search Pagination + Level Filter

Three additions to the search API that exercise the project's 
conventions end-to-end.

### Before (without CLAUDE.md active)

Tested by temporarily renaming CLAUDE.md and running a fresh session 
with the prompt: "Add a stats endpoint to the Rust API."

AI output WITHOUT CLAUDE.md:
- Created a new `stats.rs` file instead of adding to existing 
  structure — ignored the single-file-per-handler pattern
- Put computation logic directly in the handler instead of the 
  Index struct — violated separation of concerns
- Used `.unwrap()` on the RwLock: `state.index.read().unwrap()`
- Named the method `get_statistics()` — inconsistent with naming 
  convention
- Returned a custom Stats struct instead of `serde_json::json!()` 
  — added unnecessary type definition
- Did not add the endpoint to the router — output was incomplete

### After (with CLAUDE.md active)

AI output WITH CLAUDE.md:
- Added `stats()` method directly to `Index` struct in indexer crate 
  — correct location per architecture
- Handler used `.read().await` with no unwrap — followed error 
  handling convention
- Used `serde_json::json!()` for response — consistent with existing 
  handlers
- Added route to router in correct location — complete implementation
- Method named `stats()` — correct naming convention

### Specific Differences Traced to Infrastructure

| Difference | Infrastructure Element |
|------------|----------------------|
| Logic in Index not handler | indexer/CLAUDE.md: "Computation methods go in Index struct" |
| No .unwrap() usage | api/CLAUDE.md: "Never use .unwrap() in handler code" |
| json!() macro for response | api/CLAUDE.md: endpoint examples showing json!() pattern |
| Correct router placement | api/CLAUDE.md: startup sequence showing Router::new() pattern |
| stats() not get_statistics() | CLAUDE.md: "no get_ prefix" naming convention |

### Feature Code

**indexer/src/index.rs — added stats method:**
```rust
pub fn stats(&self) -> IndexStats {
    let total = self.documents.len();
    let avg_workload = if total == 0 { 0.0 } else {
        self.documents.values()
            .map(|d| d.workload_score)
            .sum::<f32>() / total as f32
    };
    let high_workload = self.documents.values()
        .filter(|d| d.workload_score > 0.6)
        .count();
    let low_workload = self.documents.values()
        .filter(|d| d.workload_score < 0.3)
        .count();
    let level_300 = self.documents.values()
        .filter(|d| d.level == 300)
        .count();
    let level_400 = self.documents.values()
        .filter(|d| d.level == 400)
        .count();
    IndexStats { total, avg_workload, high_workload, 
                 low_workload, level_300, level_400 }
}
```

**api/src/main.rs — added handler and route:**
```rust
async fn stats(State(state): State<AppState>) -> Json<Value> {
    let idx = state.index.read().await;
    let s = idx.stats();
    Json(json!({
        "total_courses": s.total,
        "avg_workload_score": s.avg_workload,
        "high_workload_count": s.high_workload,
        "low_workload_count": s.low_workload,
        "levels": { "300": s.level_300, "400": s.level_400 }
    }))
}
// Added to router: .route("/stats", get(stats))
```

**Pagination added to SearchParams:**
```rust
#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
    page: Option<usize>,
    limit: Option<usize>,
    level: Option<u32>,
}
```

### Test the Feature
```bash
curl https://eecs-ulcs-crawler.onrender.com/stats
curl "https://eecs-ulcs-crawler.onrender.com/search?q=machine+learning&level=400"
curl "https://eecs-ulcs-crawler.onrender.com/search?q=systems&page=1&limit=5"
```

---

## PART 3: DECISION LOG ENTRIES

### Entry 1 — What Context I Included in CLAUDE.md and Why

**Decision:** Structure CLAUDE.md across three files (workspace, 
indexer, api) rather than one monolithic file

**Context:** The project has three distinct concern layers — 
architecture (workspace), business logic (indexer), and API contract 
(api). A single file would either be too long or too shallow on each.

**Reasoning:** Claude Code reads the CLAUDE.md in the current working 
directory. When working in the indexer crate, it reads 
`indexer/CLAUDE.md` first — so BM25 parameters and workload signal 
details are immediately available without noise from API routing 
details. When working in the api crate, the endpoint contracts and 
AppState shape are front and center. The workspace CLAUDE.md provides 
the cross-cutting concerns (naming, error handling, build commands) 
that apply everywhere. This mirrors how a real engineering team would 
structure documentation — per-service READMEs plus a monorepo root doc.

**What AI would get wrong without this:**
Without the indexer CLAUDE.md, AI would put BM25 computation in the 
API handler. Without the api CLAUDE.md, AI would invent handler 
patterns inconsistent with the existing codebase. Without the 
workspace CLAUDE.md, AI would use .unwrap() freely and choose 
inconsistent naming.

**Alternatives:** Single root CLAUDE.md (simpler but noisier), 
inline code comments only (not read by Claude Code automatically)

**Trade-offs:** Three files to maintain vs. one, but each is focused 
and under 100 lines — well within the signal-to-noise threshold.

**AI Code note:** The most impactful section was the Known Issues 
constraint "DO NOT add PORT as env var on Render." Without this, 
Claude Code suggested adding PORT=3001 to the Render environment 
during the deployment debugging session, which would have broken 
port binding again.

---

### Entry 2 — What I Learned from Testing

**Decision:** Add explicit "never use .unwrap()" and "no get_ prefix" 
rules after the first test revealed both gaps

**Context:** First test task was "add a stats endpoint." Fresh session 
with CLAUDE.md active produced mostly correct output but used 
.unwrap() on the RwLock read and named the method get_statistics().

**Reasoning:** The original CLAUDE.md said "use ? operator or explicit 
fallbacks" — but this was in the context of Result types. Claude Code 
didn't generalize it to RwLock reads, which don't return Result. 
The rule needed to be more explicit: ".unwrap() is never acceptable 
in handler or crawler code, including on lock reads." Similarly, 
"snake_case for functions" is not specific enough to prevent 
get_statistics() — the naming convention needed to state the no-get_ 
prefix rule explicitly.

**What I iterated:**
- Before: "Use ? operator or explicit fallbacks"
- After: "NEVER use .unwrap() in handler code — use ? or explicit 
  error returns. This includes RwLock reads — use .read().await, 
  not .read().unwrap()"

- Before: "Functions: snake_case"
- After: "Functions: snake_case, verb-only for actions (search, add), 
  noun for accessors (stats, health) — no get_ prefix"

**AI Code note:** Testing revealed that CLAUDE.md content needs to 
anticipate the specific mistakes AI makes, not just state positive 
conventions. "Do X" is weaker than "Do X, not Y." The most effective 
lines in the final CLAUDE.md are the explicit prohibitions.

---

### Entry 3 — What I Iterated On

**Decision:** Add business context section as the first section, 
not the last

**Context:** First draft had business context buried after tech stack. 
Testing showed AI's responses focused on technical correctness but 
missed the user-facing framing — it described the stats endpoint 
as "for monitoring" rather than "for students to understand the 
index coverage."

**Reasoning:** Business context at the top sets the frame for 
everything that follows. When AI knows "target users are UMich CS 
students planning schedules," it makes different word choices in 
error messages, different decisions about what to include in responses, 
and different assumptions about what matters. Moving business context 
to the top of CLAUDE.md was the single highest-leverage edit — it 
changed the tone and focus of AI output more than any technical 
specification.

**What changed:** Moved business context from section 5 to section 1. 
Added "Target users are UMich CS students planning schedules" — 
previously absent. Added the specific query example 
("easy ULCS low workload") to give AI concrete context for what 
the system needs to handle well.

**AI Code note:** Before this change, AI described API responses in 
technical terms ("returns a JSON object with course metadata"). After, 
it described them in user terms ("returns courses ranked so students 
can quickly identify low-workload options"). Business context changes 
AI's mental model of the system, not just its syntax choices.

---

### Entry 4 — Infrastructure vs. Per-Prompt Context

**Decision:** Use CLAUDE.md for stable conventions, prompts for 
task-specific requirements

**Context:** Debated whether to put the BM25 formula in CLAUDE.md 
or include it in each prompt when working on ranking.

**Reasoning:** CLAUDE.md should contain things that are always true 
about the project — conventions, architecture, constraints. It should 
not contain task-specific requirements that change per session. The 
BM25 formula (k1=1.2, b=0.75, 70/30 blend) belongs in 
indexer/CLAUDE.md because it's a stable project decision. The specific 
feature requirements ("add pagination with max 20 results") belong 
in the prompt because they're task-specific. Mixing these creates a 
CLAUDE.md that's too long and has low signal-to-noise ratio. The 
mental model: CLAUDE.md = project constitution (stable, always true), 
prompts = legislation (specific, task-bounded).

**Trade-offs:** Requires discipline to keep CLAUDE.md stable and not 
add task-specific detail. But a focused CLAUDE.md that AI reads 
completely is more effective than a comprehensive one AI skims.

**AI Code note:** The clearest sign that something belongs in CLAUDE.md 
vs. the prompt: if you'd have to repeat it in more than 3 sessions, 
it belongs in CLAUDE.md. The .unwrap() prohibition appeared in 
session 1, session 3, and session 5 before I added it to CLAUDE.md. 
That repetition was the signal.

---

### Entry 5 — Context Infrastructure vs. Per-Session Prompting

**Decision:** Invest 2 hours in CLAUDE.md rather than writing better 
per-session prompts

**Context:** After Week 3, I could write detailed prompts that 
produced good output. The question was whether CLAUDE.md was worth 
the additional investment.

**Reasoning:** Per-session prompts solve the problem once. CLAUDE.md 
solves it permanently. The leverage calculation: if I spend 2 hours 
on CLAUDE.md and save 5 minutes of context-setting per session, I 
break even at 24 sessions. But the benefit compounds — better 
CLAUDE.md means better output on unexpected tasks, not just planned 
ones. When I asked Claude Code to fix a bug mid-session without any 
context prompt, it followed project conventions correctly because 
CLAUDE.md was loaded. Per-session prompting can't do that. The 
infrastructure investment is justified by session count and by 
the "unexpected task" benefit that per-session prompting can't provide.

**What I'd tell a junior engineer:** Write the prompt first. When 
you find yourself writing the same context three times, move it to 
CLAUDE.md. Don't write CLAUDE.md speculatively — write it reactively 
based on actual gaps.

**AI Code note:** The ROI calculation for context infrastructure is 
the same as for any infrastructure investment: high upfront cost, 
low marginal cost per use. The break-even point is lower than it 
feels in the moment.

---

## PART 4: REFLECTION

### Before and After

Before this week, I thought good AI output came from writing better 
prompts. Now I understand it comes from building context that makes 
the right output the path of least resistance.

The shift is from reactive to proactive. Per-session prompting is 
reactive — you notice AI did something wrong and correct it. 
CLAUDE.md is proactive — you anticipate what AI will get wrong and 
eliminate the mistake before the session starts. Senior engineers 
do this constantly in non-AI contexts: they write ADRs so future 
engineers don't re-litigate settled decisions, they write runbooks 
so on-call engineers don't have to improvise under pressure. CLAUDE.md 
is the same pattern applied to AI collaboration.

### What Surprised Me

The most impactful CLAUDE.md content was the prohibitions, not the 
conventions. "Use snake_case" is useful. "Never use .unwrap() in 
handler code, including on RwLock reads" is transformative. AI 
already knows snake_case — it doesn't need to be told. AI doesn't 
know your project's specific stance on panic-ability — it does need 
to be told. The highest-value CLAUDE.md lines are the ones that 
encode decisions that can't be inferred from general Rust knowledge.

### On Testing Methodology

The test-and-iterate loop is the entire point of Week 4. A CLAUDE.md 
written without testing is a hypothesis. Testing turns it into 
evidence. The two gaps found in testing (the .unwrap() generalization 
and the naming convention specificity) would have caused subtle bugs 
in every subsequent session. Finding them through a deliberate test 
task cost 20 minutes. Not finding them would have cost 5 minutes per 
session indefinitely. The testing investment had a 4-session payback 
period.

### On Context as Infrastructure

The mental model that clicked: context infrastructure is to AI 
collaboration what CI/CD is to software delivery. You could manually 
run tests and deploy on every commit. You could manually set context 
in every AI session. Both work. Neither scales. The infrastructure 
investment pays off through consistency, not just speed — just as 
CI/CD catches mistakes humans miss under time pressure, CLAUDE.md 
catches AI mistakes that per-session prompting misses when you're 
focused on the task rather than the conventions.

### What I'd Build Next

The gap in current context infrastructure is the absence of examples. 
CLAUDE.md tells AI what to do; it doesn't show AI what good output 
looks like. The next iteration would add a `examples/` directory with 
canonical implementations of a handler, a test, and an index method — 
so AI can pattern-match against working code rather than following 
abstract rules. This is the difference between a style guide and a 
style guide with annotated examples. Both are useful; the second is 
more powerful.

---

## PART 2B: MCP SERVER INTEGRATION

### Servers Configured

**Server 1: Filesystem**
- **What it is:** Gives Claude Code direct read/write access to the 
  project directory without needing to run shell commands
- **Configuration:**
```json
{
  "filesystem": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-filesystem",
    "/Users/arleencheema/Documents/AI Bootcamp Practice/polyglot-microservice"]
  }
}
```
- **What context it provides:** Claude Code can read any file in the 
  workspace directly — Cargo.toml dependency versions, actual 
  courses.json data, existing handler implementations — without me 
  pasting them into the prompt. It can also verify its own output 
  by reading what it just wrote.
- **What AI can do that it couldn't before:**
  - Read `data/courses.json` directly to understand the actual data 
    shape rather than relying on the Document struct definition
  - Cross-reference multiple files simultaneously — e.g. check that 
    a new field added to `common/src/lib.rs` is also handled in 
    `indexer/src/index.rs` and `api/src/main.rs`
  - Verify that generated code compiles by reading the actual 
    Cargo.toml for available dependencies
- **Security consideration:** Scoped to the project directory only. 
  Cannot access home directory, credentials, or system files. No 
  write access configured — read-only prevents accidental file 
  corruption during exploration tasks.
- **Before/after example:**  
  Without filesystem MCP, asking "does the indexer handle the 
  workload_score field correctly?" required me to paste 3 files 
  into the prompt. With filesystem MCP, Claude Code read 
  `common/src/lib.rs`, `indexer/src/index.rs`, and 
  `data/courses.json` autonomously and gave a complete answer 
  referencing line numbers.

---

**Server 2: Fetch**
- **What it is:** Gives Claude Code the ability to make real HTTP 
  requests to URLs — including your own deployed API
- **Configuration:**
```json
{
  "fetch": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-fetch"]
  }
}
```
- **What context it provides:** Claude Code can call the live Render 
  API and see actual search results, health status, and response 
  shapes — grounding its suggestions in real runtime behavior rather 
  than static code analysis.
- **What AI can do that it couldn't before:**
  - Verify that a deployed endpoint works before suggesting changes
  - Compare actual API response shape against what the frontend 
    expects — catching mismatches without manual curl
  - Test search quality directly: "fetch 
    /search?q=easy+low+workload and tell me if the ranking makes sense"
  - Check that Render is live before suggesting deployment changes
- **Security consideration:** Can fetch any public URL — not scoped 
  to your project. Do not use in sessions where you paste sensitive 
  URLs or auth tokens into the conversation. Acceptable for this 
  project since all endpoints are public read-only.
- **Before/after example:**  
  Without fetch MCP, debugging the Vercel → Render connection 
  required manually running curl commands and pasting results. 
  With fetch MCP, Claude Code called 
  `https://eecs-ulcs-crawler.onrender.com/health` directly, 
  confirmed the response, and then diagnosed why the frontend 
  wasn't reaching it — all in one session without me doing 
  any manual verification steps.

### What MCP Provides That CLAUDE.md Cannot

CLAUDE.md tells AI what the database schema looks like. MCP lets 
AI query the actual data. This is the fundamental difference: 
**CLAUDE.md is static documentation, MCP is live access.**

Concrete example from this project: CLAUDE.md documents that 
`data/courses.json` contains 20 courses. The filesystem MCP lets 
Claude Code read the actual file and see that EECS 445 has a 
higher workload score than EECS 493 — and reason about why the 
ranking for "easy ML course" returns 445 lower than expected. 
Static documentation cannot support that kind of grounded debugging.

The analogy: CLAUDE.md is like giving a new engineer a design doc. 
MCP is like giving them access to the actual running system. Both 
are necessary. Neither replaces the other.

---

## PART 2C: CUSTOM AGENT CONFIGURATION

### Agent Built: Code Reviewer

**Why this agent:** The most common source of inconsistency in this 
project is code that works correctly but violates project conventions 
— .unwrap() usage, logic in handlers instead of the indexer, 
inconsistent naming. A reviewer agent catches these before commit 
without requiring a human reviewer.

**When to use it:** Before every `git commit`. Give it the diff or 
the changed files and it flags violations before they reach the repo.

**Agent prompt:**

```markdown
# Reviewer Agent — EECS ULCS Search Engine

## Role
You are a code reviewer for the EECS ULCS Rust search engine. 
Your job is to review code changes for convention violations, 
architectural boundary violations, and correctness issues. 
You do NOT rewrite code — you identify specific problems with 
file and line references and explain why each is a violation.

## Standards You Enforce

### Architecture Boundaries
- Business logic belongs in the indexer crate, never in api handlers
- Handlers call index methods; they do not compute or transform data
- common/ contains types only — no logic, no functions
- crawler/ is I/O only — no scoring, no filtering

### Error Handling
- .unwrap() is never acceptable in api/ or crawler/ code paths
- This includes: .unwrap() on RwLock reads, Option unwraps, 
  Result unwraps
- Use .read().await for RwLock, ? for Results, 
  .unwrap_or_default() for Options where appropriate

### Naming Conventions
- Functions: snake_case, verb for actions, noun for accessors
- No get_ prefix on accessor methods
- Struct fields: snake_case matching Document field names exactly

### Response Format
- All API responses use serde_json::json!() macro
- No custom response structs unless adding to SearchEntry
- Consistent field names with existing endpoints

### Testing
- Any new Index method must have a unit test
- Tests go in #[cfg(test)] block in the same file

## How You Respond
- List violations as numbered items
- Each item: file/line reference, what the violation is, 
  which convention it breaks, and the correct approach
- End with a summary: "X violations found. Blocking: Y. 
  Non-blocking: Z."
- Blocking violations are architectural or correctness issues
- Non-blocking violations are naming or style issues

## What You Never Do
- Rewrite the code for the developer
- Approve code with blocking violations
- Flag issues that are intentional trade-offs documented 
  in CLAUDE.md (e.g. static JSON instead of live crawling)
- Comment on code outside the changed files
```

**How to invoke:**
In Claude Code, start a session with:
/review [paste the git diff or changed file contents]
**Test results:**

Test 1 — gave it a handler with .unwrap() and logic inside:
```rust
async fn bad_handler(State(state): State<AppState>) -> Json<Value> {
    let idx = state.index.read().unwrap();
    let count = idx.documents.values()
        .filter(|d| d.workload_score < 0.3)
        .count();
    Json(json!({ "low_workload": count }))
}
```

Agent output:
1. api/src/main.rs — .unwrap() on RwLock read (line 3)
Violation: Error Handling convention
Fix: use .read().await instead of .read().unwrap()
2. api/src/main.rs — filter/count logic in handler (lines 4-6)
Violation: Architecture boundary
Fix: this computation belongs in a method on Index in the
indexer crate. Handler should call idx.low_workload_count()

Summary: 2 violations found. Blocking: 2. Non-blocking: 0.
Result: Correct on both violations, correct severity, 
did not rewrite the code.

Test 2 — gave it correct code:
Agent output: "0 violations found. Code follows project conventions."

Result: No false positives on compliant code.

**Agent rationale — connection to EPCC framework:**
- **Encode:** The reviewer agent encodes project conventions into 
  a format AI can actively apply, not just reference
- **Provide:** It provides targeted context (only the changed files) 
  rather than the full codebase
- **Check:** It checks output against conventions automatically — 
  the review IS the check step
- **Correct:** It identifies specific corrections without making them, 
  preserving developer ownership of the fix

The reviewer agent is the EPCC loop applied to code review — it 
automates the Check step so developer attention goes to Correct.

---

## PART 3: BEFORE/AFTER COMPARISON — EXTENDED

### Comparison 1: Stats Endpoint (documented above)

### Comparison 2: MCP-assisted Debugging

**Task:** "Why does searching 'easy low workload' not return 
EECS 493 (UI Development) as the top result?"

**Without MCP:**
- Had to manually paste courses.json, index.rs scoring code, 
  and the search results into the prompt
- Claude Code reasoned from static code — couldn't see actual scores
- Answer was theoretical: "it depends on the workload_score value"
- Took 3 follow-up prompts to get a concrete answer

**With filesystem + fetch MCP:**
- Claude Code read courses.json directly — saw EECS 493 
  workload_score: 0.42 (medium, not low)
- Fetched /search?q=easy+low+workload — saw actual ranking
- Identified that EECS 493's description contains "implementation" 
  and "assignments" (high workload keywords) pushing score to 0.42
- Explained why EECS 490 (Programming Languages, score: 0.28) 
  ranked higher — lower workload signal
- Complete answer in one prompt, no manual data gathering

**What caused the difference:** Fetch MCP provided live response data. 
Filesystem MCP provided actual workload scores. Together they let 
Claude Code reason from real values rather than code structure alone.

### Comparison 3: Convention Adherence Over Time

**Metric:** Number of .unwrap() corrections needed per session

- Week 3 (no CLAUDE.md): 3-4 corrections per session
- Week 4 first draft CLAUDE.md: 1-2 corrections per session  
- Week 4 iterated CLAUDE.md (explicit RwLock rule): 0 corrections 
  in last 3 sessions

**What caused the improvement:** Specificity of the prohibition. 
General "use ? operator" → specific "never .unwrap() including 
on RwLock reads." The iteration based on testing produced the 
precise wording that eliminated the mistake.

---

## DECISION LOG — MCP AND AGENT ENTRIES

### Entry 6 — MCP Server Selection

**Decision:** Configure filesystem and fetch MCP servers, 
not SQLite or memory

**Context:** Could have configured SQLite MCP for the planned 
Supabase database, or memory MCP for persistent context across sessions.

**Reasoning:** SQLite MCP would be valuable once the database 
persistence layer is built, but currently the project uses static 
JSON — there's no database to query. Configuring it speculatively 
adds setup cost with no current benefit. Memory MCP solves a 
problem (context loss between sessions) that CLAUDE.md already 
solves for stable conventions. The filesystem and fetch servers 
solve problems that CLAUDE.md cannot solve: live file access 
and live API verification. Every MCP server should solve a problem 
that static documentation cannot.

**Trade-offs:** Filesystem MCP introduces a security surface — 
Claude Code can read any file in the project directory. Mitigated 
by scoping to the project path only and not configuring write access 
for exploration tasks.

**AI Code note:** The selection framework that worked: "What does 
this project need that CLAUDE.md cannot provide?" Filesystem = 
live file content. Fetch = live API responses. Both answers were 
concrete and immediate. SQLite and memory answers were speculative 
and future-oriented — wrong time to configure them.

---

### Entry 7 — Custom Agent Design

**Decision:** Build a reviewer agent rather than an architect 
or tester agent

**Context:** Three candidate agents: reviewer (catches convention 
violations), architect (designs new features), tester (writes tests).

**Reasoning:** The reviewer agent addresses the most frequent 
failure mode observed in Week 3 — code that works but violates 
conventions. The architect agent would be valuable for new feature 
design but this project's architecture is settled. The tester agent 
would be useful but test coverage is currently good for the indexer 
crate. The reviewer fills the gap where convention violations 
accumulated during Week 3's rapid development pace. It's also the 
agent with the clearest success criteria: zero violations on 
compliant code, correct identification on non-compliant code. 
That measurability made it the right first agent to build and test.

**Trade-offs:** Reviewer agent requires giving it file contents 
or diffs — slightly more friction than asking Claude Code inline. 
Worth it for the consistency of output and the clear violation/
not-violation structure.

**AI Code note:** Agent prompts that work best are specific about 
what the agent does NOT do. "You do not rewrite code" was the 
most important constraint — without it, the reviewer rewrote the 
handler in test 1 instead of flagging the violation. Negative 
constraints are as important in agent prompts as they are in CLAUDE.md.