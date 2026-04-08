use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Crawl error: {0}")]
    CrawlError(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Database error: {0}")]
    DbError(String),
}
