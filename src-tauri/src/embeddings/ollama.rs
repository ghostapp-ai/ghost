use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{GhostError, Result};

const OLLAMA_BASE: &str = "http://localhost:11434";
const EMBEDDING_MODEL: &str = "nomic-embed-text";

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

/// Client for generating embeddings via Ollama HTTP API.
#[derive(Clone)]
pub struct OllamaEngine {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaEngine {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: OLLAMA_BASE.to_string(),
            model: EMBEDDING_MODEL.to_string(),
        }
    }

    /// Check if Ollama is reachable and the model is available.
    pub async fn health_check(&self) -> Result<bool> {
        let resp = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await;

        match resp {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get embedding dimensions (768 for nomic-embed-text).
    pub fn dimensions(&self) -> usize {
        768
    }

    /// Generate an embedding for a single text.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| GhostError::OllamaUnavailable(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GhostError::Embedding(format!(
                "Ollama returned {}: {}",
                status, body
            )));
        }

        let result: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| GhostError::Embedding(format!("Failed to parse embedding: {}", e)))?;

        if result.embedding.is_empty() {
            return Err(GhostError::Embedding("Empty embedding returned".into()));
        }

        Ok(result.embedding)
    }

    /// Generate embeddings for a batch of texts.
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            let embedding = self.embed(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }
}

impl Default for OllamaEngine {
    fn default() -> Self {
        Self::new()
    }
}
