//! Native chat engine using Candle GGUF quantized models.
//!
//! Supports any Qwen2.5-Instruct GGUF model from the registry.
//! Automatically selects the optimal device (CPU/CUDA/Metal).
//! Downloads models from HuggingFace Hub on first use.

use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::quantized_qwen2::ModelWeights;
use tokenizers::Tokenizer;

use super::models::ModelProfile;
use super::ChatMessage;
use super::DownloadProgress;
use crate::error::{GhostError, Result};

/// Default sampling parameters.
const DEFAULT_TEMPERATURE: f64 = 0.7;
const DEFAULT_TOP_P: f64 = 0.9;
const DEFAULT_SEED: u64 = 42;

/// Qwen2.5 ChatML template tokens.
const IM_START: &str = "<|im_start|>";
const IM_END: &str = "<|im_end|>";

/// Native chat engine powered by Candle + quantized GGUF models.
///
/// The model is loaded from disk for each conversation to ensure a clean
/// KV cache. This adds ~0.5-3s overhead but guarantees correctness.
pub struct NativeChatEngine {
    model_path: std::path::PathBuf,
    tokenizer: Tokenizer,
    device: Device,
    eos_token_id: u32,
    model_id: String,
    model_name: String,
    temperature: f64,
    top_p: f64,
}

impl NativeChatEngine {
    /// Load a chat model from a profile.
    ///
    /// Downloads the model from HuggingFace Hub on first run.
    /// Uses the specified device (CPU/CUDA/Metal).
    pub async fn load(
        profile: &ModelProfile,
        device: Device,
        progress: std::sync::Arc<std::sync::Mutex<Option<DownloadProgress>>>,
    ) -> Result<Self> {
        tracing::info!(
            "Loading native chat engine: {} ({}) on {:?}",
            profile.name,
            profile.gguf_file,
            device
        );

        // Download model files from HuggingFace Hub (cached after first download)
        let (model_path, tokenizer_path) =
            Self::download_model_files(profile, progress.clone()).await?;

        // Update progress: loading model into memory
        if let Ok(mut p) = progress.lock() {
            *p = Some(DownloadProgress {
                downloaded_bytes: 0,
                total_bytes: 0,
                phase: "loading_model".into(),
            });
        }

        // Verify model loads correctly by doing a test load
        tracing::info!("Validating GGUF model from {}...", model_path.display());
        {
            let mut file = std::fs::File::open(&model_path)
                .map_err(|e| GhostError::Chat(format!("Failed to open GGUF: {}", e)))?;
            let content = gguf_file::Content::read(&mut file)
                .map_err(|e| GhostError::Chat(format!("Failed to read GGUF: {}", e)))?;
            let _model = ModelWeights::from_gguf(content, &mut file, &device)
                .map_err(|e| GhostError::Chat(format!("Failed to load model: {}", e)))?;
        }

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| GhostError::Chat(format!("Failed to load tokenizer: {}", e)))?;

        // Find EOS token ID
        let eos_token_id = tokenizer
            .token_to_id("<|im_end|>")
            .or_else(|| tokenizer.token_to_id("<|endoftext|>"))
            .unwrap_or(151643);

        tracing::info!(
            "Native chat engine ready: {} (device={:?}, eos={})",
            profile.name,
            device,
            eos_token_id
        );

        Ok(Self {
            model_path,
            tokenizer,
            device,
            eos_token_id,
            model_id: profile.id.to_string(),
            model_name: profile.name.to_string(),
            temperature: DEFAULT_TEMPERATURE,
            top_p: DEFAULT_TOP_P,
        })
    }

    /// Get the active model ID.
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Get the active model name.
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Update sampling parameters.
    #[allow(dead_code)]
    pub fn set_sampling(&mut self, temperature: f64, top_p: f64) {
        self.temperature = temperature.clamp(0.0, 2.0);
        self.top_p = top_p.clamp(0.0, 1.0);
    }

    /// Generate a response for a list of chat messages.
    ///
    /// Loads a fresh model instance for each call to ensure clean KV cache.
    /// The GGUF file is in OS page cache after first load, so reload is fast.
    pub fn generate(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String> {
        // Load fresh model (clean KV cache)
        let mut model = self.load_fresh_model()?;

        let prompt = Self::format_chat_prompt(messages);
        self.generate_text(&mut model, &prompt, max_tokens)
    }

    /// Load a fresh model instance from the cached GGUF file.
    fn load_fresh_model(&self) -> Result<ModelWeights> {
        let mut file = std::fs::File::open(&self.model_path)
            .map_err(|e| GhostError::Chat(format!("Failed to open GGUF: {}", e)))?;
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| GhostError::Chat(format!("Failed to read GGUF: {}", e)))?;
        ModelWeights::from_gguf(content, &mut file, &self.device)
            .map_err(|e| GhostError::Chat(format!("Failed to load model: {}", e)))
    }

    /// Format messages into Qwen2.5 ChatML prompt format.
    fn format_chat_prompt(messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();

        // Add system message if not present
        let has_system = messages.iter().any(|m| m.role == "system");
        if !has_system {
            prompt.push_str(IM_START);
            prompt.push_str("system\nYou are Ghost, a helpful local AI assistant running natively on the user's computer with zero cloud dependencies. Be concise, helpful, and direct.");
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

    /// Generate text from a prompt using the quantized model.
    fn generate_text(
        &self,
        model: &mut ModelWeights,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<String> {
        let max_tokens = max_tokens.min(2048);

        // Tokenize the prompt
        let encoding = self
            .tokenizer
            .encode(prompt, false)
            .map_err(|e| GhostError::Chat(format!("Tokenization failed: {}", e)))?;

        let prompt_tokens = encoding.get_ids().to_vec();
        let prompt_len = prompt_tokens.len();

        tracing::debug!(
            "Chat prompt: {} tokens, generating up to {} tokens",
            prompt_len,
            max_tokens
        );

        // Create logits processor for sampling
        let sampling = if self.temperature <= 0.01 {
            Sampling::ArgMax
        } else {
            Sampling::TopP {
                p: self.top_p,
                temperature: self.temperature,
            }
        };
        let mut logits_processor = LogitsProcessor::from_sampling(DEFAULT_SEED, sampling);

        // Process prompt through model (prefill)
        let input = Tensor::new(prompt_tokens.as_slice(), &self.device)
            .map_err(|e| GhostError::Chat(format!("Tensor creation failed: {}", e)))?
            .unsqueeze(0)
            .map_err(|e| GhostError::Chat(format!("Unsqueeze failed: {}", e)))?;

        let logits = model
            .forward(&input, 0)
            .map_err(|e| GhostError::Chat(format!("Model forward (prefill) failed: {}", e)))?;

        let logits = logits
            .squeeze(0)
            .map_err(|e| GhostError::Chat(format!("Squeeze failed: {}", e)))?;

        // Sample first token
        let mut next_token = logits_processor
            .sample(&logits)
            .map_err(|e| GhostError::Chat(format!("Sampling failed: {}", e)))?;

        let mut generated_tokens: Vec<u32> = vec![next_token];
        let mut pos = prompt_len;

        // Auto-regressive generation loop
        for _ in 1..max_tokens {
            if next_token == self.eos_token_id {
                break;
            }

            let input = Tensor::new(&[next_token], &self.device)
                .map_err(|e| GhostError::Chat(format!("Tensor failed: {}", e)))?
                .unsqueeze(0)
                .map_err(|e| GhostError::Chat(format!("Unsqueeze failed: {}", e)))?;

            let logits = model
                .forward(&input, pos)
                .map_err(|e| GhostError::Chat(format!("Forward failed at pos {}: {}", pos, e)))?;

            let logits = logits
                .squeeze(0)
                .map_err(|e| GhostError::Chat(format!("Squeeze failed: {}", e)))?;

            next_token = logits_processor
                .sample(&logits)
                .map_err(|e| GhostError::Chat(format!("Sampling failed: {}", e)))?;

            generated_tokens.push(next_token);
            pos += 1;
        }

        // Remove EOS token if present at end
        if generated_tokens.last() == Some(&self.eos_token_id) {
            generated_tokens.pop();
        }

        // Decode tokens to text
        let response = self
            .tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| GhostError::Chat(format!("Decoding failed: {}", e)))?;

        tracing::debug!(
            "Generated {} tokens from {}",
            generated_tokens.len(),
            self.model_name
        );

        Ok(response.trim().to_string())
    }

    /// Download model files from HuggingFace Hub if not already cached.
    ///
    /// Monitors the filesystem during download to report progress.
    async fn download_model_files(
        profile: &ModelProfile,
        progress: std::sync::Arc<std::sync::Mutex<Option<DownloadProgress>>>,
    ) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
        let repo_id = profile.repo_id.to_string();
        let gguf_file = profile.gguf_file.to_string();
        let tokenizer_repo = profile.tokenizer_repo.to_string();
        let expected_bytes = profile.size_mb * 1_048_576; // MB to bytes

        // Check if already cached — skip monitoring if so
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
        } else {
            // Set initial download progress
            if let Ok(mut p) = progress.lock() {
                *p = Some(DownloadProgress {
                    downloaded_bytes: 0,
                    total_bytes: expected_bytes,
                    phase: "downloading".into(),
                });
            }
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

        // Run sync HF Hub downloads in a blocking task to avoid blocking the async runtime
        let result = tokio::task::spawn_blocking(move || {
            let api = hf_hub::api::sync::Api::new().map_err(|e| {
                GhostError::Chat(format!("Failed to init HuggingFace Hub API: {}", e))
            })?;

            // Download GGUF model weights
            tracing::info!("Ensuring model files: {}/{}", repo_id, gguf_file);
            let model_repo = api.model(repo_id.clone());
            let model_path = model_repo.get(&gguf_file).map_err(|e| {
                GhostError::Chat(format!(
                    "Failed to download {}/{}: {}. Internet required for first-time setup.",
                    repo_id, gguf_file, e
                ))
            })?;

            // Download tokenizer
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

        // Stop the monitor thread (it will exit on next check since download is done)
        if let Some(handle) = monitor_handle {
            // Signal completion
            if let Ok(mut p) = progress.lock() {
                if let Some(ref mut dp) = *p {
                    dp.phase = "download_complete".into();
                    dp.downloaded_bytes = expected_bytes;
                }
            }
            // Wait a short time then drop the thread (it checks phase to exit)
            let _ = handle.join();
        }

        result
    }

    /// Monitor the HuggingFace cache directory for download progress.
    ///
    /// Checks every 500ms for growing blob files and updates the progress tracker.
    fn monitor_download_progress(
        repo_id: &str,
        expected_bytes: u64,
        progress: std::sync::Arc<std::sync::Mutex<Option<DownloadProgress>>>,
    ) {
        let cache_base = super::models::get_hf_cache_dir();
        let repo_name = repo_id.replace('/', "--");
        let blobs_dir = cache_base
            .join(format!("models--{}", repo_name))
            .join("blobs");

        loop {
            // Check if we should stop
            if let Ok(p) = progress.lock() {
                if let Some(ref dp) = *p {
                    if dp.phase == "download_complete" || dp.phase == "loading_model" {
                        return;
                    }
                } else {
                    return; // Progress was cleared, stop monitoring
                }
            }

            // Scan blobs directory for the largest growing file (likely our download)
            let mut max_incomplete_size: u64 = 0;
            if let Ok(entries) = std::fs::read_dir(&blobs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    // hf_hub uses .incomplete suffix for in-progress downloads
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

            // Also check for completed blobs without .incomplete (in case it finished between checks)
            if max_incomplete_size == 0 {
                if let Ok(entries) = std::fs::read_dir(&blobs_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(meta) = std::fs::metadata(&path) {
                                // Only count files that are reasonably close to expected size
                                // (avoid counting small metadata files)
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
