use thiserror::Error;

#[derive(Error, Debug)]
pub enum GhostError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Indexer error: {0}")]
    Indexer(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Ollama not available: {0}")]
    OllamaUnavailable(String),

    #[error("Native model error: {0}")]
    NativeModel(String),

    #[error("Chat error: {0}")]
    Chat(String),
}

impl serde::Serialize for GhostError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GhostError>;
