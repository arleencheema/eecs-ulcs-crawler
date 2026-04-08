# Indexer Crate

Pure logic — no I/O, no network, no filesystem. Everything is unit-testable.

## BM25 Implementation

Parameters: k1 = 1.2, b = 0.75 (standard defaults, well-suited for short academic descriptions).

Formula per term t in query, per document d:
  score(d,t) = IDF(t) * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * dl/avgdl))

Where:
  tf    = term frequency of t in d
  dl    = document length (token count)
  avgdl = average document length across all indexed docs
  IDF   = log((N - df + 0.5) / (df + 0.5) + 1)  where N=total docs, df=docs containing t

Final score blends BM25 with workload signal:
  final = 0.7 * bm25_score + 0.3 * (1.0 - doc.workload_score)

The (1.0 - workload_score) inversion means low-workload courses rank higher,
which matches user queries like "easy ULCS with low workload".

## Tokenizer

Pipeline: lowercase → replace non-alphanumeric with space → split on whitespace →
filter stopwords → filter tokens under 2 chars.

Stopword list is defined as a const in tokenize.rs. Length-based filtering alone
was dropped because it removed meaningful CS terms like "os", "ml", "ai", "db".
The stopword list handles common English words explicitly instead.

Short tokens kept intentionally: os, ml, ai, db, io, ui

## Workload Signal

compute_workload_score(description) returns 0.0–1.0.

High workload keywords push score toward 1.0:
  "exam", "exams", "project", "projects", "weekly", "problem set",
  "homework", "lab", "quiz", "quizzes", "implementation"

Low workload keywords push score toward 0.0:
  "participation", "reading", "discussion", "survey", "overview", "seminar"

Score = (high_count - low_count + low_count_total) / normalizer, clamped to 0.0–1.0.

## Test Coverage

All tests live in indexer/src/index.rs under #[cfg(test)].
Run with: cargo test -p indexer

Key test cases:
- tokenize_filters_stopwords: "an", "the", "of" must be absent
- tokenize_keeps_cs_terms: "os", "ml" must be present
- search_returns_relevant_doc_first: BM25 correctness on 3 fake docs
- search_low_workload_ranks_higher: signal fusion correctness
- add_duplicate_doc_returns_error: index integrity