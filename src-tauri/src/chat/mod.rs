//! Chat engine with hardware-aware model selection and fallback chain.
//!
//! Ghost automatically detects hardware, selects the best model, and downloads it.
//! GPU acceleration is detected at runtime via llama.cpp (Vulkan/CUDA/Metal).
//! Fallback chain: Native (llama.cpp) → Ollama → None.

#[cfg(desktop)]
pub mod inference;
pub mod models;
#[cfg(desktop)]
pub mod native;

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::embeddings::hardware::HardwareInfo;
use crate::error::{GhostError, Result};

/// A single chat message (user, assistant, or system).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Download progress information.
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub phase: String,
}

/// Chat engine status for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct ChatStatus {
    pub available: bool,
    pub backend: String,
    pub model_id: String,
    pub model_name: String,
    pub loading: bool,
    pub error: Option<String>,
    pub device: String,
    pub download_progress: Option<DownloadProgress>,
}

/// Chat generation response.
#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    pub content: String,
    pub tokens_generated: usize,
    pub duration_ms: u64,
    pub model_id: String,
}

/// Unified chat engine with runtime GPU auto-detection.
///
/// Uses llama.cpp (via llama-cpp-2) for native inference on desktop.
/// GPU is detected at runtime — no compile-time flags needed for Vulkan.
/// Falls back to Ollama HTTP API if native engine fails.
/// On mobile, only Ollama is available (llama.cpp not compiled for mobile).
pub struct ChatEngine {
    #[cfg(desktop)]
    native: Mutex<Option<native::NativeChatEngine>>,
    active_model_id: Mutex<String>,
    loading: Mutex<bool>,
    error: Mutex<Option<String>>,
    hardware: HardwareInfo,
    download_progress: std::sync::Arc<Mutex<Option<DownloadProgress>>>,
}

impl ChatEngine {
    /// Create a new chat engine (does NOT load the model yet).
    pub fn new(hardware: HardwareInfo, model_id: String) -> Self {
        Self {
            #[cfg(desktop)]
            native: Mutex::new(None),
            active_model_id: Mutex::new(model_id),
            loading: Mutex::new(false),
            error: Mutex::new(None),
            hardware,
            download_progress: std::sync::Arc::new(Mutex::new(None)),
        }
    }

    /// Load the active model. Downloads from HuggingFace Hub on first run.
    /// On mobile, this is a no-op (mobile uses Ollama only).
    pub async fn load_model(&self) {
        #[cfg(not(desktop))]
        {
            tracing::info!("Native chat model loading skipped on mobile — use Ollama");
            return;
        }

        #[cfg(desktop)]
        {
            self._load_model_desktop().await;
        }
    }

    /// Desktop-only model loading implementation.
    #[cfg(desktop)]
    async fn _load_model_desktop(&self) {
        let model_id = { self.active_model_id.lock().unwrap().clone() };

        let profile = match models::find_model(&model_id) {
            Some(p) => p,
            None => {
                let msg = format!("Unknown model: {}", model_id);
                tracing::error!("{}", msg);
                *self.error.lock().unwrap() = Some(msg);
                return;
            }
        };

        {
            let mut loading = self.loading.lock().unwrap();
            if *loading {
                tracing::warn!("Model loading already in progress");
                return;
            }
            *loading = true;
        }
        *self.error.lock().unwrap() = None;
        *self.download_progress.lock().unwrap() = Some(DownloadProgress {
            downloaded_bytes: 0,
            total_bytes: profile.size_mb * 1_048_576,
            phase: "checking_cache".into(),
        });

        tracing::info!(
            "Loading model: {} ({}, ~{}MB)",
            profile.name,
            profile.parameters,
            profile.size_mb
        );

        let progress = self.download_progress.clone();
        match native::NativeChatEngine::load(profile, progress.clone()).await {
            Ok(engine) => {
                tracing::info!(
                    "Chat engine ready: {} on {} (gpu={})",
                    profile.name,
                    engine.gpu_backend(),
                    engine.is_gpu_active()
                );
                *self.native.lock().unwrap() = Some(engine);
                *self.error.lock().unwrap() = None;
            }
            Err(e) => {
                tracing::error!("Failed to load {}: {}", profile.name, e);
                *self.error.lock().unwrap() = Some(e.to_string());
            }
        }

        *self.loading.lock().unwrap() = false;
        *self.download_progress.lock().unwrap() = None;
    }

    /// Switch to a different model.
    pub async fn switch_model(&self, model_id: &str) -> Result<()> {
        let profile = models::find_model(model_id)
            .ok_or_else(|| GhostError::Chat(format!("Unknown model: {}", model_id)))?;

        if self.hardware.available_ram_mb < profile.min_ram_mb {
            return Err(GhostError::Chat(format!(
                "Insufficient RAM: {} needs {}MB, only {}MB available",
                profile.name, profile.min_ram_mb, self.hardware.available_ram_mb
            )));
        }

        // Unload current model (desktop only)
        #[cfg(desktop)]
        {
            *self.native.lock().unwrap() = None;
        }
        *self.active_model_id.lock().unwrap() = model_id.to_string();

        self.load_model().await;

        if self.error.lock().unwrap().is_some() {
            Err(GhostError::Chat("Failed to load model".into()))
        } else {
            Ok(())
        }
    }

    /// Get chat engine status for the frontend.
    pub fn status(&self) -> ChatStatus {
        #[cfg(desktop)]
        let native = self.native.lock().unwrap();
        let loading = *self.loading.lock().unwrap();
        let error = self.error.lock().unwrap().clone();
        let model_id = self.active_model_id.lock().unwrap().clone();
        let progress = self.download_progress.lock().unwrap().clone();

        #[cfg(desktop)]
        if let Some(ref engine) = *native {
            return ChatStatus {
                available: true,
                backend: "native".into(),
                model_id: engine.model_id().to_string(),
                model_name: engine.model_name().to_string(),
                loading: false,
                error: None,
                device: engine.gpu_backend().to_string(),
                download_progress: None,
            };
        }

        if loading {
            let model_name = models::find_model(&model_id)
                .map(|p| p.name.to_string())
                .unwrap_or(model_id.clone());
            ChatStatus {
                available: false,
                backend: "loading".into(),
                model_id,
                model_name,
                loading: true,
                error: None,
                device: "detecting".into(),
                download_progress: progress,
            }
        } else if check_ollama_sync() {
            ChatStatus {
                available: true,
                backend: "ollama".into(),
                model_id: "ollama-default".into(),
                model_name: "Ollama".into(),
                loading: false,
                error: None,
                device: "external".into(),
                download_progress: None,
            }
        } else {
            let model_name = models::find_model(&model_id)
                .map(|p| p.name.to_string())
                .unwrap_or("none".into());
            ChatStatus {
                available: false,
                backend: "none".into(),
                model_id,
                model_name,
                loading: false,
                error,
                device: "none".into(),
                download_progress: None,
            }
        }
    }

    /// Generate a chat response.
    pub async fn chat(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<ChatResponse> {
        let start = std::time::Instant::now();

        // Try native engine first (desktop only)
        #[cfg(desktop)]
        {
            let model_id = self.active_model_id.lock().unwrap().clone();
            let native = self.native.lock().unwrap();
            if let Some(ref engine) = *native {
                let content = engine.generate(messages, max_tokens)?;
                let duration = start.elapsed();
                let token_count = content.split_whitespace().count();
                return Ok(ChatResponse {
                    content,
                    tokens_generated: token_count,
                    duration_ms: duration.as_millis() as u64,
                    model_id,
                });
            }
        }

        // Fall back to Ollama
        let content = ollama_chat(messages, max_tokens).await?;
        let duration = start.elapsed();
        let token_count = content.split_whitespace().count();

        Ok(ChatResponse {
            content,
            tokens_generated: token_count,
            duration_ms: duration.as_millis() as u64,
            model_id: "ollama".into(),
        })
    }

    /// Get list of available models with runtime status.
    pub fn available_models(&self) -> Vec<models::ModelInfo> {
        let active = self.active_model_id.lock().unwrap().clone();
        models::list_models(&self.hardware, &active)
    }

    /// Get the recommended model ID for this hardware.
    pub fn recommended_model_id(&self) -> String {
        models::recommend_model(&self.hardware).id.to_string()
    }
}

/// Quick synchronous check if Ollama is reachable.
fn check_ollama_sync() -> bool {
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], 11434)),
        std::time::Duration::from_millis(100),
    )
    .is_ok()
}

/// Chat with Ollama via HTTP API.
async fn ollama_chat(messages: &[ChatMessage], max_tokens: usize) -> Result<String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "qwen2.5:0.5b",
        "messages": messages,
        "stream": false,
        "options": {
            "num_predict": max_tokens,
        }
    });

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| GhostError::Chat(format!("Ollama unavailable: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(GhostError::Chat(format!(
            "Ollama returned {}: {}",
            status, body
        )));
    }

    #[derive(Deserialize)]
    struct OllamaResponse {
        message: OllamaMessage,
    }
    #[derive(Deserialize)]
    struct OllamaMessage {
        content: String,
    }

    let result: OllamaResponse = response
        .json()
        .await
        .map_err(|e| GhostError::Chat(format!("Failed to parse Ollama response: {}", e)))?;

    Ok(result.message.content)
}
