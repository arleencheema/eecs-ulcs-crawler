use serde::{Deserialize, Serialize};

/// A crawled document ready to be indexed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub credits: String,
    pub level: u32,
    pub url: String,
    pub workload_score: f32,
}
