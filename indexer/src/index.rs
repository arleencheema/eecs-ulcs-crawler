use std::collections::HashMap;

use common::{AppError, Document};

const STOPWORDS: &[&str] = &[
    "an", "a", "is", "it", "in", "of", "to", "the", "and", "or", "for", "on", "at", "by",
];

/// Replace non-alphanumeric chars with spaces, lowercase, split on whitespace,
/// filter tokens under 2 chars and common stopwords.
pub fn tokenize(text: &str) -> Vec<String> {
    text.chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .filter(|t| t.len() >= 2 && !STOPWORDS.contains(&t.as_str()))
        .collect()
}

const HIGH_WORKLOAD: &[&str] = &[
    "exam", "exams", "project", "projects", "weekly", "problem set",
    "homework", "lab", "quiz", "quizzes", "implementation", "assignments",
];

const LOW_WORKLOAD: &[&str] = &[
    "participation", "reading", "discussion", "survey", "overview", "seminar",
];

/// Returns a workload score in [0.0, 1.0].
/// High values mean heavy workload; low values mean light workload.
pub fn compute_workload_score(description: &str) -> f32 {
    let lower = description.to_lowercase();
    let high: f32 = HIGH_WORKLOAD.iter().filter(|&&kw| lower.contains(kw)).count() as f32 * 0.15;
    let low: f32  = LOW_WORKLOAD.iter().filter(|&&kw| lower.contains(kw)).count() as f32 * 0.1;
    (high - low).clamp(0.0, 1.0)
}

/// Aggregate statistics returned by [`Index::stats`].
#[derive(Debug)]
pub struct IndexStats {
    pub total_courses:       usize,
    pub avg_workload_score:  f32,
    pub high_workload_count: usize,
    pub low_workload_count:  usize,
    /// Course count keyed by 100-level bucket (e.g. 300, 400).
    pub levels: HashMap<u32, usize>,
}

/// Inverted index with BM25 ranking support.
pub struct Index {
    /// term -> [(doc_id, term_freq)]
    postings: HashMap<String, Vec<(u32, u32)>>,
    /// doc_id -> Document
    docs: HashMap<u32, Document>,
    /// Total number of documents
    doc_count: u32,
    /// Sum of all document lengths (in tokens), used to compute avg_dl
    total_tokens: u64,
}

impl Index {
    pub fn new() -> Self {
        Self {
            postings: HashMap::new(),
            docs: HashMap::new(),
            doc_count: 0,
            total_tokens: 0,
        }
    }

    /// Tokenize a document's title + description, update posting lists, and store the document.
    pub fn add_document(&mut self, doc: Document) -> Result<(), AppError> {
        if self.docs.contains_key(&doc.id) {
            return Err(AppError::IndexError(format!(
                "document id {} already indexed",
                doc.id
            )));
        }

        let text = format!("{} {}", doc.title, doc.description);
        let tokens = tokenize(&text);

        self.total_tokens += tokens.len() as u64;
        self.doc_count += 1;

        // Count term frequencies within this document
        let mut tf: HashMap<String, u32> = HashMap::new();
        for token in tokens {
            *tf.entry(token).or_insert(0) += 1;
        }

        let doc_id = doc.id;
        let mut doc = doc;
        doc.workload_score = compute_workload_score(&doc.description);
        self.docs.insert(doc_id, doc);

        for (term, freq) in tf {
            self.postings.entry(term).or_default().push((doc_id, freq));
        }

        Ok(())
    }

    /// BM25 search. Returns (Document, score) pairs sorted by score descending.
    pub fn search(&self, query: &str) -> Vec<(Document, f32)> {
        const K1: f32 = 1.2;
        const B: f32 = 0.75;

        if self.doc_count == 0 {
            return vec![];
        }

        let avg_dl = self.total_tokens as f32 / self.doc_count as f32;
        let n = self.doc_count as f32;

        let query_terms = tokenize(query);
        let mut scores: HashMap<u32, f32> = HashMap::new();

        for term in &query_terms {
            let Some(posting) = self.postings.get(term) else {
                continue;
            };

            // df = number of documents containing this term
            let df = posting.len() as f32;
            // IDF (Robertson-Sparck Jones variant, clamped to 0)
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

            for &(doc_id, tf) in posting {
                let doc_len = self.doc_len(doc_id) as f32;
                let tf_norm =
                    (tf as f32 * (K1 + 1.0)) / (tf as f32 + K1 * (1.0 - B + B * doc_len / avg_dl));
                *scores.entry(doc_id).or_insert(0.0) += idf * tf_norm;
            }
        }

        let mut results: Vec<(Document, f32)> = scores
            .into_iter()
            .filter_map(|(doc_id, bm25_score)| {
                self.docs.get(&doc_id).map(|doc| {
                    let final_score = 0.7 * bm25_score + 0.3 * (1.0 - doc.workload_score);
                    (doc.clone(), final_score)
                })
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Compute aggregate statistics over all indexed documents.
    pub fn stats(&self) -> IndexStats {
        let total_courses = self.doc_count as usize;
        let docs: Vec<&Document> = self.docs.values().collect();

        let avg_workload_score = if total_courses == 0 {
            0.0
        } else {
            docs.iter().map(|d| d.workload_score).sum::<f32>() / total_courses as f32
        };

        let high_workload_count = docs.iter().filter(|d| d.workload_score > 0.5).count();
        let low_workload_count  = docs.iter().filter(|d| d.workload_score <= 0.3).count();

        let mut levels: HashMap<u32, usize> = HashMap::new();
        for doc in &docs {
            let bucket = (doc.level / 100) * 100;
            *levels.entry(bucket).or_insert(0) += 1;
        }

        IndexStats {
            total_courses,
            avg_workload_score,
            high_workload_count,
            low_workload_count,
            levels,
        }
    }

    /// Count tokens for a stored document (used inside search for dl normalization).
    fn doc_len(&self, doc_id: u32) -> usize {
        self.docs
            .get(&doc_id)
            .map(|doc| tokenize(&format!("{} {}", doc.title, doc.description)).len())
            .unwrap_or(0)
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(id: u32, title: &str, description: &str) -> Document {
        Document {
            id,
            title: title.to_string(),
            description: description.to_string(),
            credits: "3".to_string(),
            level: 400,
            url: format!("https://example.com/{id}"),
            workload_score: 0.0,
        }
    }

    // ── tokenize ─────────────────────────────────────────────────────────────

    #[test]
    fn tokenize_lowercases() {
        let tokens = tokenize("Web SYSTEMS");
        assert!(tokens.contains(&"web".to_string()));
        assert!(tokens.contains(&"systems".to_string()));
    }

    #[test]
    fn tokenize_strips_punctuation() {
        let tokens = tokenize("easy, low-workload!");
        assert!(tokens.contains(&"easy".to_string()));
        // hyphens become spaces, so "low-workload" splits into two tokens
        assert!(tokens.contains(&"low".to_string()));
        assert!(tokens.contains(&"workload".to_string()));
        assert!(!tokens.iter().any(|t| t.contains('-')));
    }

    #[test]
    fn tokenize_filters_short_tokens_and_stopwords() {
        let tokens = tokenize("a an the systems");
        // "a" dropped (len < 2); "an" and "the" dropped (stopwords)
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"an".to_string()));
        assert!(!tokens.contains(&"the".to_string()));
        assert!(tokens.contains(&"systems".to_string()));
    }

    #[test]
    fn tokenize_empty_string() {
        assert!(tokenize("").is_empty());
    }

    // ── Index::search (BM25) ─────────────────────────────────────────────────

    fn build_test_index() -> Index {
        let mut idx = Index::new();
        idx.add_document(make_doc(
            1,
            "Web Systems",
            "Large scale web systems search engines distributed computing",
        ))
        .unwrap();
        idx.add_document(make_doc(
            2,
            "Operating Systems",
            "Kernel design process scheduling memory management low level systems",
        ))
        .unwrap();
        idx.add_document(make_doc(
            3,
            "Mobile App Development",
            "Build iOS Android apps project based easy low workload practical",
        ))
        .unwrap();
        idx
    }

    #[test]
    fn search_returns_relevant_doc_first() {
        let idx = build_test_index();
        let results = idx.search("web systems");
        assert!(!results.is_empty(), "expected at least one result");
        assert_eq!(results[0].0.id, 1, "doc 1 (Web Systems) should rank first");
    }

    #[test]
    fn search_low_workload_ranks_mobile() {
        let idx = build_test_index();
        let results = idx.search("easy low workload");
        assert!(!results.is_empty());
        assert_eq!(
            results[0].0.id, 3,
            "doc 3 (Mobile, easy/low/workload) should rank first"
        );
    }

    #[test]
    fn search_unknown_query_returns_empty() {
        let idx = build_test_index();
        let results = idx.search("xyzzy frobnicator");
        assert!(results.is_empty());
    }

    #[test]
    fn search_results_sorted_descending() {
        let idx = build_test_index();
        let results = idx.search("systems");
        let scores: Vec<f32> = results.iter().map(|(_, s)| *s).collect();
        for w in scores.windows(2) {
            assert!(w[0] >= w[1], "results not sorted descending");
        }
    }

    // ── compute_workload_score ────────────────────────────────────────────────

    #[test]
    fn high_workload_description_scores_higher() {
        let heavy = "exam project weekly homework lab implementation quiz";
        let light = "reading discussion participation overview seminar";
        let heavy_score = compute_workload_score(heavy);
        let light_score = compute_workload_score(light);
        assert!(
            heavy_score > light_score,
            "heavy ({heavy_score}) should exceed light ({light_score})"
        );
    }

    #[test]
    fn workload_score_clamped_to_unit_interval() {
        let very_heavy = "exam exams project projects weekly homework lab quiz quizzes implementation assignments problem set";
        let score = compute_workload_score(very_heavy);
        assert!((0.0..=1.0).contains(&score), "score {score} out of range");
        assert_eq!(score, 1.0, "saturated description should clamp to 1.0");
    }

    #[test]
    fn workload_score_light_description_near_zero() {
        let score = compute_workload_score("reading discussion participation");
        assert!(score == 0.0, "score should be 0.0, got {score}");
    }

    #[test]
    fn add_duplicate_doc_returns_error() {
        let mut idx = Index::new();
        idx.add_document(make_doc(1, "Web Systems", "web systems course"))
            .unwrap();
        let err = idx.add_document(make_doc(1, "Duplicate", "duplicate entry"));
        assert!(err.is_err());
    }
}
