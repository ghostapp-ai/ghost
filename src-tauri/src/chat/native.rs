//! Native chat engine using llama.cpp via llama-cpp-2 crate.
//!
//! Supports any GGUF model from the registry (Qwen2.5-Instruct family).
//! Runtime GPU auto-detection: Vulkan (NVIDIA/AMD/Intel), Metal (macOS), CUDA.
//! Falls back to CPU transparently if no GPU is available.

use std::sync::{Arc, OnceLock};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

use super::models::ModelProfile;
use super::ChatMessage;
use super::DownloadProgress;
use crate::error::{GhostError, Result};

/// Global singleton for the llama.cpp backend.
///
/// `LlamaBackend::init()` can only be called ONCE per process (uses an AtomicBool guard).
/// Both the chat engine and the agent executor share this single instance.
/// Wrapped in `Arc` so it can be cloned into structs that need a reference.
static LLAMA_BACKEND: OnceLock<Arc<LlamaBackend>> = OnceLock::new();

/// Get or initialize the global llama.cpp backend.
///
/// Thread-safe: the first caller initializes, subsequent callers get the cached instance.
/// This MUST be used instead of `LlamaBackend::init()` directly.
pub fn get_or_init_backend() -> Result<Arc<LlamaBackend>> {
    let backend = LLAMA_BACKEND.get_or_init(|| {
        let b = LlamaBackend::init().expect("Failed to initialize llama.cpp backend");
        Arc::new(b)
    });
    Ok(Arc::clone(backend))
}

/// Default sampling parameters.
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_TOP_P: f32 = 0.9;
const DEFAULT_SEED: u32 = 42;
const MAX_CONTEXT: u32 = 4096;
const BATCH_SIZE: u32 = 512;

/// Qwen2.5 ChatML template tokens.
const IM_START: &str = "<|im_start|>";
const IM_END: &str = "<|im_end|>";

/// Native chat engine powered by llama.cpp with runtime GPU auto-detection.
///
/// Unlike the previous Candle engine, this one:
/// - Auto-detects GPU at runtime (Vulkan/CUDA/Metal)
/// - Has proper KV cache clearing (no model reload per request)
/// - Uses llama.cpp — the reference GGUF inference implementation
pub struct NativeChatEngine {
    backend: Arc<LlamaBackend>,
    model: LlamaModel,
    model_id: String,
    model_name: String,
    temperature: f32,
    top_p: f32,
    gpu_backend_name: String,
    n_gpu_layers: u32,
}

impl NativeChatEngine {
    /// Load a chat model from a profile.
    ///
    /// Downloads the model from HuggingFace Hub on first run.
    /// Auto-detects GPU backend at runtime.
    pub async fn load(
        profile: &ModelProfile,
        progress: Arc<std::sync::Mutex<Option<DownloadProgress>>>,
    ) -> Result<Self> {
        tracing::info!(
            "Loading native chat engine: {} ({})",
            profile.name,
            profile.gguf_file,
        );

        // Download model files from HuggingFace Hub (cached after first download)
        let (model_path, _tokenizer_path) =
            Self::download_model_files(profile, progress.clone()).await?;

        // Update progress: loading model into memory
        if let Ok(mut p) = progress.lock() {
            *p = Some(DownloadProgress {
                downloaded_bytes: 0,
                total_bytes: 0,
                phase: "loading_model".into(),
            });
        }

        // Get or initialize the global llama.cpp backend singleton
        let backend = get_or_init_backend()?;

        // Detect GPU capabilities
        let has_gpu = backend.supports_gpu_offload();
        let n_gpu_layers: u32 = if has_gpu { 9999 } else { 0 };

        // Determine active GPU backend name
        let gpu_backend_name = if has_gpu {
            Self::detect_gpu_name()
        } else {
            "CPU".to_string()
        };

        tracing::info!(
            "llama.cpp backend: gpu_offload={}, n_gpu_layers={}, backend={}",
            has_gpu,
            n_gpu_layers,
            gpu_backend_name
        );

        // Load model
        let model_params = LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers);

        let model_path_str = model_path.to_string_lossy().to_string();
        let model = LlamaModel::load_from_file(&backend, &model_path_str, &model_params)
            .map_err(|e| GhostError::Chat(format!("Failed to load GGUF model: {}", e)))?;

        tracing::info!(
            "Native chat engine ready: {} (gpu={}, layers_offloaded={})",
            profile.name,
            gpu_backend_name,
            n_gpu_layers
        );

        Ok(Self {
            backend,
            model,
            model_id: profile.id.to_string(),
            model_name: profile.name.to_string(),
            temperature: DEFAULT_TEMPERATURE,
            top_p: DEFAULT_TOP_P,
            gpu_backend_name,
            n_gpu_layers,
        })
    }

    /// Detect the active GPU backend name from llama.cpp devices.
    fn detect_gpu_name() -> String {
        let devices = llama_cpp_2::list_llama_ggml_backend_devices();
        for dev in &devices {
            if matches!(
                dev.device_type,
                llama_cpp_2::LlamaBackendDeviceType::Gpu
                    | llama_cpp_2::LlamaBackendDeviceType::IntegratedGpu
            ) {
                return format!("{} ({})", dev.backend, dev.description);
            }
        }
        "GPU".to_string()
    }

    /// Get the active model ID.
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Get the active model name.
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Get the active GPU backend description.
    pub fn gpu_backend(&self) -> &str {
        &self.gpu_backend_name
    }

    /// Whether GPU offload is active.
    pub fn is_gpu_active(&self) -> bool {
        self.n_gpu_layers > 0
    }

    /// Generate a response for a list of chat messages.
    ///
    /// Uses proper KV cache clearing between calls — no model reload needed.
    pub fn generate(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String> {
        let max_tokens = max_tokens.min(2048);

        let prompt = Self::format_chat_prompt(messages);

        // Tokenize
        let tokens = self
            .model
            .str_to_token(&prompt, AddBos::Never)
            .map_err(|e| GhostError::Chat(format!("Tokenization failed: {}", e)))?;

        let prompt_len = tokens.len();
        tracing::debug!(
            "Chat prompt: {} tokens, generating up to {} tokens",
            prompt_len,
            max_tokens
        );

        if prompt_len >= MAX_CONTEXT as usize {
            return Err(GhostError::Chat(format!(
                "Prompt too long: {} tokens (max {})",
                prompt_len, MAX_CONTEXT
            )));
        }

        // Create context for this generation
        let n_threads = (std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            / 2)
        .max(1) as i32;

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(std::num::NonZeroU32::new(MAX_CONTEXT).unwrap()))
            .with_n_batch(BATCH_SIZE)
            .with_n_threads(n_threads)
            .with_n_threads_batch(n_threads);

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| GhostError::Chat(format!("Failed to create context: {}", e)))?;

        // Setup sampler
        let mut sampler = if self.temperature <= 0.01 {
            LlamaSampler::greedy()
        } else {
            LlamaSampler::chain_simple([
                LlamaSampler::temp(self.temperature),
                LlamaSampler::top_p(self.top_p, 1),
                LlamaSampler::dist(DEFAULT_SEED),
            ])
        };

        // Prefill: submit all prompt tokens as a batch
        let mut batch = LlamaBatch::new(BATCH_SIZE as usize, 1);
        let last_idx = tokens.len() as i32 - 1;
        for (i, &token) in tokens.iter().enumerate() {
            batch
                .add(token, i as i32, &[0], i as i32 == last_idx)
                .map_err(|e| GhostError::Chat(format!("Batch add failed: {}", e)))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| GhostError::Chat(format!("Prefill decode failed: {}", e)))?;

        // Generation loop
        let mut n_cur = batch.n_tokens();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut output = String::new();

        for _ in 0..max_tokens {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            // Check end of generation
            if self.model.is_eog_token(token) {
                break;
            }

            // Decode token to text
            match self.model.token_to_piece(token, &mut decoder, true, None) {
                Ok(piece) => output.push_str(&piece),
                Err(e) => tracing::warn!("Token decode error: {}", e),
            }

            // Prepare next batch
            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| GhostError::Chat(format!("Batch add failed: {}", e)))?;

            n_cur += 1;

            ctx.decode(&mut batch)
                .map_err(|e| GhostError::Chat(format!("Decode failed at pos {}: {}", n_cur, e)))?;
        }

        tracing::debug!("Generated ~{} chars from {}", output.len(), self.model_name);

        Ok(output.trim().to_string())
    }

    /// Format messages into Qwen2.5 ChatML prompt format.
    fn format_chat_prompt(messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();

        // Add system message if not present
        let has_system = messages.iter().any(|m| m.role == "system");
        if !has_system {
            prompt.push_str(IM_START);
            prompt.push_str("system\nYou are Ghost, a helpful local AI assistant running natively on the user's computer with zero cloud dependencies. Be concise, helpful, and direct. Respond in the same language the user writes in.");
            prompt.push_str(IM_END);
            prompt.push('\n');
        }

        for msg in messages {
            prompt.push_str(IM_START);
            prompt.push_str(&msg.role);
            prompt.push('\n');
            prompt.push_str(&msg.content);
            prompt.push_str(IM_END);
            prompt.push('\n');
        }

        // Start assistant response
        prompt.push_str(IM_START);
        prompt.push_str("assistant\n");

        prompt
    }

    /// Download model files from HuggingFace Hub if not already cached.
    async fn download_model_files(
        profile: &ModelProfile,
        progress: Arc<std::sync::Mutex<Option<DownloadProgress>>>,
    ) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
        let repo_id = profile.repo_id.to_string();
        let gguf_file = profile.gguf_file.to_string();
        let tokenizer_repo = profile.tokenizer_repo.to_string();
        let expected_bytes = profile.size_mb * 1_048_576;

        // Check if already cached
        let already_cached = super::models::is_model_cached(profile);
        if already_cached {
            tracing::info!("Model already cached, skipping download");
            if let Ok(mut p) = progress.lock() {
                *p = Some(DownloadProgress {
                    downloaded_bytes: expected_bytes,
                    total_bytes: expected_bytes,
                    phase: "cached".into(),
                });
            }
        } else if let Ok(mut p) = progress.lock() {
            *p = Some(DownloadProgress {
                downloaded_bytes: 0,
                total_bytes: expected_bytes,
                phase: "downloading".into(),
            });
        }

        // Start filesystem monitor for download progress (only if not cached)
        let monitor_handle = if !already_cached {
            let progress_monitor = progress.clone();
            let repo_id_monitor = repo_id.clone();
            let expected = expected_bytes;
            Some(std::thread::spawn(move || {
                Self::monitor_download_progress(&repo_id_monitor, expected, progress_monitor);
            }))
        } else {
            None
        };

        // Run sync HF Hub downloads in a blocking task
        let result = tokio::task::spawn_blocking(move || {
            let api = hf_hub::api::sync::Api::new().map_err(|e| {
                GhostError::Chat(format!("Failed to init HuggingFace Hub API: {}", e))
            })?;

            tracing::info!("Ensuring model files: {}/{}", repo_id, gguf_file);
            let model_repo = api.model(repo_id.clone());
            let model_path = model_repo.get(&gguf_file).map_err(|e| {
                GhostError::Chat(format!(
                    "Failed to download {}/{}: {}. Internet required for first-time setup.",
                    repo_id, gguf_file, e
                ))
            })?;

            let tok_repo = api.model(tokenizer_repo.clone());
            let tokenizer_path = tok_repo.get("tokenizer.json").map_err(|e| {
                GhostError::Chat(format!(
                    "Failed to download tokenizer from {}: {}",
                    tokenizer_repo, e
                ))
            })?;

            tracing::info!(
                "Model files ready: model={}, tokenizer={}",
                model_path.display(),
                tokenizer_path.display()
            );

            Ok((model_path, tokenizer_path))
        })
        .await
        .map_err(|e| GhostError::Chat(format!("Download task panicked: {}", e)))?;

        if let Some(handle) = monitor_handle {
            if let Ok(mut p) = progress.lock() {
                if let Some(ref mut dp) = *p {
                    dp.phase = "download_complete".into();
                    dp.downloaded_bytes = expected_bytes;
                }
            }
            let _ = handle.join();
        }

        result
    }

    /// Monitor the HuggingFace cache directory for download progress.
    fn monitor_download_progress(
        repo_id: &str,
        expected_bytes: u64,
        progress: Arc<std::sync::Mutex<Option<DownloadProgress>>>,
    ) {
        let cache_base = super::models::get_hf_cache_dir();
        let repo_name = repo_id.replace('/', "--");
        let blobs_dir = cache_base
            .join(format!("models--{}", repo_name))
            .join("blobs");

        loop {
            if let Ok(p) = progress.lock() {
                if let Some(ref dp) = *p {
                    if dp.phase == "download_complete" || dp.phase == "loading_model" {
                        return;
                    }
                } else {
                    return;
                }
            }

            let mut max_incomplete_size: u64 = 0;
            if let Ok(entries) = std::fs::read_dir(&blobs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if name.ends_with(".incomplete") {
                        if let Ok(meta) = std::fs::metadata(&path) {
                            max_incomplete_size = max_incomplete_size.max(meta.len());
                        }
                    }
                }
            }

            if max_incomplete_size == 0 {
                if let Ok(entries) = std::fs::read_dir(&blobs_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(meta) = std::fs::metadata(&path) {
                                if meta.len() > expected_bytes / 10 {
                                    max_incomplete_size = max_incomplete_size.max(meta.len());
                                }
                            }
                        }
                    }
                }
            }

            if max_incomplete_size > 0 {
                if let Ok(mut p) = progress.lock() {
                    if let Some(ref mut dp) = *p {
                        dp.downloaded_bytes = max_incomplete_size.min(expected_bytes);
                        dp.phase = "downloading".into();
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_init_backend_succeeds() {
        let result = get_or_init_backend();
        assert!(result.is_ok(), "get_or_init_backend() should succeed");
    }

    #[test]
    fn test_get_or_init_backend_returns_same_instance() {
        let b1 = get_or_init_backend().expect("first call");
        let b2 = get_or_init_backend().expect("second call");
        // Arc::ptr_eq checks they point to the exact same allocation
        assert!(
            Arc::ptr_eq(&b1, &b2),
            "Multiple calls must return the same Arc<LlamaBackend> instance"
        );
    }

    #[test]
    fn test_get_or_init_backend_concurrent() {
        use std::thread;

        let handles: Vec<_> = (0..8)
            .map(|_| {
                thread::spawn(|| get_or_init_backend().expect("concurrent init should succeed"))
            })
            .collect();

        let backends: Vec<Arc<LlamaBackend>> =
            handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All threads must get the same instance
        for b in &backends[1..] {
            assert!(
                Arc::ptr_eq(&backends[0], b),
                "All concurrent callers must get the same Arc<LlamaBackend>"
            );
        }
    }
}
