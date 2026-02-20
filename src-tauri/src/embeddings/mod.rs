//! Embedding engine abstraction with fallback chain: Native → Ollama → FTS5-only.
//!
//! Ghost tries to use the best available embedding engine:
//! 1. **Native** (Candle) — zero dependencies, runs in-process
//! 2. **Ollama** — optional, higher quality with larger models
//! 3. **FTS5-only** — keyword search still works with no embeddings

pub mod hardware;
pub mod native;
pub mod ollama;

use crate::error::{GhostError, Result};

/// The active AI backend for the status bar and diagnostics.
#[derive(Debug, Clone, serde::Serialize)]
pub enum AiBackend {
    /// Native Candle engine running in-process.
    Native,
    /// Ollama HTTP API.
    Ollama,
    /// No embedding engine available — FTS5 keyword search only.
    None,
}

impl std::fmt::Display for AiBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiBackend::Native => write!(f, "native"),
            AiBackend::Ollama => write!(f, "ollama"),
            AiBackend::None => write!(f, "none"),
        }
    }
}

/// AI status returned to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AiStatus {
    pub backend: AiBackend,
    pub model_name: String,
    pub dimensions: usize,
    pub hardware: hardware::HardwareInfo,
}

/// Unified embedding engine that tries Native → Ollama fallback.
pub struct EmbeddingEngine {
    native: Option<native::NativeEngine>,
    ollama: ollama::OllamaEngine,
    active_backend: AiBackend,
    hardware: hardware::HardwareInfo,
}

impl EmbeddingEngine {
    /// Initialize the engine: try native first, fall back to Ollama.
    pub async fn initialize() -> Self {
        // Ensure TLS provider is installed (needed for Ollama health check & HF Hub downloads).
        // Idempotent — safe to call from both run() and tests.
        crate::ensure_tls_provider();

        let hw = hardware::HardwareInfo::detect();

        // Try native engine first
        match native::NativeEngine::load().await {
            Ok(engine) => {
                tracing::info!("Native embedding engine loaded (Candle, 384D)");
                return Self {
                    native: Some(engine),
                    ollama: ollama::OllamaEngine::new(),
                    active_backend: AiBackend::Native,
                    hardware: hw,
                };
            }
            Err(e) => {
                tracing::warn!("Native engine unavailable: {} — trying Ollama fallback", e);
            }
        }

        // Fall back to Ollama
        let ollama = ollama::OllamaEngine::new();
        let ollama_ok = ollama.health_check().await.unwrap_or(false);

        if ollama_ok {
            tracing::info!("Ollama engine connected (768D)");
            Self {
                native: None,
                ollama,
                active_backend: AiBackend::Ollama,
                hardware: hw,
            }
        } else {
            tracing::warn!("No embedding engine available — FTS5 keyword search only");
            Self {
                native: None,
                ollama,
                active_backend: AiBackend::None,
                hardware: hw,
            }
        }
    }

    /// Get the currently active backend.
    pub fn backend(&self) -> &AiBackend {
        &self.active_backend
    }

    /// Get the embedding dimensions for the active backend.
    pub fn dimensions(&self) -> usize {
        match &self.active_backend {
            AiBackend::Native => self.native.as_ref().map(|n| n.dimensions()).unwrap_or(384),
            AiBackend::Ollama => self.ollama.dimensions(),
            AiBackend::None => 0,
        }
    }

    /// Get the AI status for the frontend.
    pub fn status(&self) -> AiStatus {
        AiStatus {
            backend: self.active_backend.clone(),
            model_name: match &self.active_backend {
                AiBackend::Native => "all-MiniLM-L6-v2".to_string(),
                AiBackend::Ollama => "nomic-embed-text".to_string(),
                AiBackend::None => "none".to_string(),
            },
            dimensions: self.dimensions(),
            hardware: self.hardware.clone(),
        }
    }

    /// Check if any embedding engine is available.
    pub async fn health_check(&self) -> Result<bool> {
        match &self.active_backend {
            AiBackend::Native => Ok(self.native.is_some()),
            AiBackend::Ollama => self.ollama.health_check().await,
            AiBackend::None => Ok(false),
        }
    }

    /// Generate an embedding for a single text.
    /// Uses the active backend: Native → Ollama.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Try native first
        if let Some(ref native) = self.native {
            match native.embed(text) {
                Ok(embedding) => return Ok(embedding),
                Err(e) => {
                    tracing::warn!("Native embed failed, trying Ollama: {}", e);
                }
            }
        }

        // Fall back to Ollama
        match self.ollama.embed(text).await {
            Ok(embedding) => Ok(embedding),
            Err(e) => Err(GhostError::Embedding(format!(
                "All embedding engines failed. Last error: {}",
                e
            ))),
        }
    }

    /// Generate embeddings for a batch of texts.
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Try native first (synchronous, no HTTP overhead)
        if let Some(ref native) = self.native {
            match native.embed_batch(texts) {
                Ok(embeddings) => return Ok(embeddings),
                Err(e) => {
                    tracing::warn!("Native batch embed failed, trying Ollama: {}", e);
                }
            }
        }

        // Fall back to Ollama
        self.ollama.embed_batch(texts).await
    }
}
