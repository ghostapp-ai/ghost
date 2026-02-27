//! Embedding engine abstraction with fallback chain: Native → Ollama → FTS5-only.
//!
//! Ghost tries to use the best available embedding engine:
//! 1. **Native** (Candle) — zero dependencies, runs in-process
//! 2. **Ollama** — optional, higher quality with larger models
//! 3. **FTS5-only** — keyword search still works with no embeddings
//!
//! The engine uses **deferred loading** — the app starts instantly with FTS5-only,
//! then loads the native model in the background. This prevents blocking the UI
//! during model download (~23MB) or loading (~200ms cached).

pub mod hardware;
pub mod native;
pub mod ollama;

use std::sync::Mutex;

use crate::error::{GhostError, Result};

/// The active AI backend for the status bar and diagnostics.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
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
    pub loading: bool,
    pub hardware: hardware::HardwareInfo,
}

/// Unified embedding engine with deferred loading (like ChatEngine).
///
/// Starts immediately with no backend (FTS5-only). The native model is
/// loaded asynchronously in the background after the UI is visible.
/// Uses interior mutability (Mutex) to allow background loading.
pub struct EmbeddingEngine {
    native: Mutex<Option<native::NativeEngine>>,
    ollama: ollama::OllamaEngine,
    active_backend: Mutex<AiBackend>,
    loading: Mutex<bool>,
    error: Mutex<Option<String>>,
    hardware: hardware::HardwareInfo,
}

impl EmbeddingEngine {
    /// Create a new engine (does NOT load the model yet).
    ///
    /// The engine starts in FTS5-only mode. Call `load()` in a background task
    /// to initialize the native/Ollama backend without blocking the UI.
    pub fn new(hardware: hardware::HardwareInfo) -> Self {
        Self {
            native: Mutex::new(None),
            ollama: ollama::OllamaEngine::new(),
            active_backend: Mutex::new(AiBackend::None),
            loading: Mutex::new(false),
            error: Mutex::new(None),
            hardware,
        }
    }

    /// Create an engine with no backend (FTS5 keyword search only).
    /// Useful for tests and mobile builds.
    pub fn none() -> Self {
        Self {
            native: Mutex::new(None),
            ollama: ollama::OllamaEngine::new(),
            active_backend: Mutex::new(AiBackend::None),
            loading: Mutex::new(false),
            error: Mutex::new(None),
            hardware: hardware::HardwareInfo {
                cpu_cores: 1,
                has_avx2: false,
                has_neon: false,
                gpu_backend: None,
                total_ram_mb: 0,
                available_ram_mb: 0,
            },
        }
    }

    /// Load the embedding engine in the background: try native first, fall back to Ollama.
    ///
    /// This is safe to call multiple times — it guards against concurrent loads.
    /// If the engine is already loaded or currently loading, this is a no-op.
    pub async fn load(&self) {
        // Guard: already loaded?
        {
            let backend = self
                .active_backend
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if *backend != AiBackend::None {
                tracing::debug!("Embedding engine already loaded ({})", backend);
                return;
            }
        }

        // Guard: already loading?
        {
            let mut loading = self.loading.lock().unwrap_or_else(|e| e.into_inner());
            if *loading {
                tracing::warn!("Embedding engine loading already in progress");
                return;
            }
            *loading = true;
        }
        *self.error.lock().unwrap_or_else(|e| e.into_inner()) = None;

        // Ensure TLS provider is installed (needed for Ollama health check & HF Hub downloads).
        crate::ensure_tls_provider();

        // Try native engine first
        match native::NativeEngine::load(&self.hardware).await {
            Ok(engine) => {
                tracing::info!("Native embedding engine loaded (Candle, 384D)");
                *self.native.lock().unwrap_or_else(|e| e.into_inner()) = Some(engine);
                *self
                    .active_backend
                    .lock()
                    .unwrap_or_else(|e| e.into_inner()) = AiBackend::Native;
                *self.loading.lock().unwrap_or_else(|e| e.into_inner()) = false;
                return;
            }
            Err(e) => {
                tracing::warn!("Native engine unavailable: {} — trying Ollama fallback", e);
            }
        }

        // Fall back to Ollama
        let ollama_ok = self.ollama.health_check().await.unwrap_or(false);

        if ollama_ok {
            tracing::info!("Ollama engine connected (768D)");
            *self
                .active_backend
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = AiBackend::Ollama;
        } else {
            tracing::warn!("No embedding engine available — FTS5 keyword search only");
            *self.error.lock().unwrap_or_else(|e| e.into_inner()) =
                Some("No embedding engine available".to_string());
        }

        *self.loading.lock().unwrap_or_else(|e| e.into_inner()) = false;
    }

    /// Initialize the engine synchronously (for tests and backward compat).
    /// Prefer `new()` + `load()` for production use.
    pub async fn initialize() -> Self {
        let hw = hardware::HardwareInfo::detect();
        let engine = Self::new(hw);
        engine.load().await;
        engine
    }

    /// Get the currently active backend.
    pub fn backend(&self) -> AiBackend {
        self.active_backend
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Get the embedding dimensions for the active backend.
    pub fn dimensions(&self) -> usize {
        let backend = self
            .active_backend
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match *backend {
            AiBackend::Native => self
                .native
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
                .map(|n| n.dimensions())
                .unwrap_or(384),
            AiBackend::Ollama => self.ollama.dimensions(),
            AiBackend::None => 0,
        }
    }

    /// Check if the engine is currently loading.
    pub fn is_loading(&self) -> bool {
        *self.loading.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Get the AI status for the frontend.
    pub fn status(&self) -> AiStatus {
        let backend = self
            .active_backend
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let loading = *self.loading.lock().unwrap_or_else(|e| e.into_inner());
        AiStatus {
            backend: backend.clone(),
            model_name: match &backend {
                AiBackend::Native => "all-MiniLM-L6-v2".to_string(),
                AiBackend::Ollama => "nomic-embed-text".to_string(),
                AiBackend::None => "none".to_string(),
            },
            dimensions: self.dimensions(),
            loading,
            hardware: self.hardware.clone(),
        }
    }

    /// Check if any embedding engine is available.
    pub async fn health_check(&self) -> Result<bool> {
        let backend = self
            .active_backend
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        match backend {
            AiBackend::Native => Ok(self
                .native
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some()),
            AiBackend::Ollama => self.ollama.health_check().await,
            AiBackend::None => Ok(false),
        }
    }

    /// Generate an embedding for a single text.
    /// Uses the active backend: Native → Ollama.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Try native first
        {
            let native_guard = self.native.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(ref native) = *native_guard {
                match native.embed(text) {
                    Ok(embedding) => return Ok(embedding),
                    Err(e) => {
                        tracing::warn!("Native embed failed, trying Ollama: {}", e);
                    }
                }
            }
        } // drop MutexGuard before async Ollama call

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
        {
            let native_guard = self.native.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(ref native) = *native_guard {
                match native.embed_batch(texts) {
                    Ok(embeddings) => return Ok(embeddings),
                    Err(e) => {
                        tracing::warn!("Native batch embed failed, trying Ollama: {}", e);
                    }
                }
            }
        } // drop MutexGuard before async Ollama call

        // Fall back to Ollama
        self.ollama.embed_batch(texts).await
    }
}
