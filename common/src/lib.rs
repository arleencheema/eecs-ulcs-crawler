use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub credits: String,
    pub level: u32,
    pub url: String,
    #[serde(default)]
    pub workload_score: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Crawl error: {0}")]
    CrawlError(String),
    #[error("Index error: {0}")]
    IndexError(String),
    #[error("DB error: {0}")]
    DbError(String),
}
