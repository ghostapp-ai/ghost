//! Native embedding engine using Candle (HuggingFace's Rust ML framework).
//!
//! Runs embedding models directly in-process without external dependencies.
//! Supports BERT-family models in safetensors format, downloaded from HuggingFace Hub.

use std::path::PathBuf;

use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig, DTYPE};
use tokenizers::Tokenizer;

use crate::error::{GhostError, Result};
use super::hardware;

/// Default model: all-MiniLM-L6-v2 — fast, small (23MB), 384-dim, excellent quality.
const DEFAULT_MODEL_REPO: &str = "sentence-transformers/all-MiniLM-L6-v2";
const DEFAULT_DIMENSIONS: usize = 384;

/// Native embedding engine that runs models directly via Candle.
pub struct NativeEngine {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    dimensions: usize,
    normalize: bool,
}

impl NativeEngine {
    /// Load the embedding model from HuggingFace Hub (downloads on first run).
    ///
    /// Uses CPU device by default. The model is cached in the Ghost data directory.
    pub async fn load() -> Result<Self> {
        let hw = hardware::HardwareInfo::detect();
        tracing::info!(
            "Loading native embedding model ({} cores, SIMD={})",
            hw.cpu_cores,
            hw.has_simd()
        );

        // Use CPU for now — GPU backends (Metal, CUDA) can be added later
        let device = Device::Cpu;

        // Download or use cached model files from HuggingFace Hub
        let (model_path, tokenizer_path, config_path) =
            Self::ensure_model_files().await?;

        // Load config
        let config_str = std::fs::read_to_string(&config_path).map_err(|e| {
            GhostError::NativeModel(format!("Failed to read config: {}", e))
        })?;
        let config: BertConfig = serde_json::from_str(&config_str).map_err(|e| {
            GhostError::NativeModel(format!("Failed to parse config: {}", e))
        })?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|e| {
            GhostError::NativeModel(format!("Failed to load tokenizer: {}", e))
        })?;

        // Load model weights
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                &[model_path],
                DTYPE,
                &device,
            )
            .map_err(|e| {
                GhostError::NativeModel(format!("Failed to load model weights: {}", e))
            })?
        };

        let model = BertModel::load(vb, &config).map_err(|e| {
            GhostError::NativeModel(format!("Failed to build BERT model: {}", e))
        })?;

        tracing::info!(
            "Native embedding model loaded: {} ({}D, device={:?})",
            DEFAULT_MODEL_REPO,
            DEFAULT_DIMENSIONS,
            device
        );

        Ok(Self {
            model,
            tokenizer,
            device,
            dimensions: DEFAULT_DIMENSIONS,
            normalize: true,
        })
    }

    /// Get the embedding dimensions for database schema.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Generate an embedding for a single text.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| GhostError::Embedding(format!("Tokenization failed: {}", e)))?;

        let token_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        let token_type_ids = encoding.get_type_ids();

        let tokens = Tensor::new(token_ids, &self.device)
            .map_err(|e| GhostError::Embedding(format!("Tensor creation failed: {}", e)))?
            .unsqueeze(0)
            .map_err(|e| GhostError::Embedding(format!("Unsqueeze failed: {}", e)))?;

        let attention = Tensor::new(attention_mask, &self.device)
            .map_err(|e| GhostError::Embedding(format!("Attention mask failed: {}", e)))?
            .unsqueeze(0)
            .map_err(|e| GhostError::Embedding(format!("Unsqueeze failed: {}", e)))?;

        let type_ids = Tensor::new(token_type_ids, &self.device)
            .map_err(|e| GhostError::Embedding(format!("Type IDs failed: {}", e)))?
            .unsqueeze(0)
            .map_err(|e| GhostError::Embedding(format!("Unsqueeze failed: {}", e)))?;

        // Forward pass
        let output = self
            .model
            .forward(&tokens, &type_ids, Some(&attention))
            .map_err(|e| GhostError::Embedding(format!("Model forward pass failed: {}", e)))?;

        // Mean pooling over token dimension (ignoring padding via attention mask)
        let attention_f = attention
            .to_dtype(candle_core::DType::F32)
            .map_err(|e| GhostError::Embedding(format!("Dtype conversion failed: {}", e)))?
            .unsqueeze(2)
            .map_err(|e| GhostError::Embedding(format!("Unsqueeze failed: {}", e)))?;

        let masked = output
            .broadcast_mul(&attention_f)
            .map_err(|e| GhostError::Embedding(format!("Broadcast mul failed: {}", e)))?;

        let summed = masked
            .sum(1)
            .map_err(|e| GhostError::Embedding(format!("Sum failed: {}", e)))?;

        let count = attention_f
            .sum(1)
            .map_err(|e| GhostError::Embedding(format!("Attention sum failed: {}", e)))?;

        let pooled = summed
            .broadcast_div(&count)
            .map_err(|e| GhostError::Embedding(format!("Division failed: {}", e)))?;

        // L2 normalization
        let embedding = if self.normalize {
            let norm = pooled
                .sqr()
                .map_err(|e| GhostError::Embedding(format!("Sqr failed: {}", e)))?
                .sum_keepdim(1)
                .map_err(|e| GhostError::Embedding(format!("Sum keepdim failed: {}", e)))?
                .sqrt()
                .map_err(|e| GhostError::Embedding(format!("Sqrt failed: {}", e)))?;
            pooled
                .broadcast_div(&norm)
                .map_err(|e| GhostError::Embedding(format!("Normalize failed: {}", e)))?
        } else {
            pooled
        };

        // Extract as Vec<f32>
        let vec: Vec<f32> = embedding
            .squeeze(0)
            .map_err(|e| GhostError::Embedding(format!("Squeeze failed: {}", e)))?
            .to_vec1()
            .map_err(|e| GhostError::Embedding(format!("To vec failed: {}", e)))?;

        Ok(vec)
    }

    /// Generate embeddings for a batch of texts.
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            let embedding = self.embed(text)?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    /// Download model files from HuggingFace Hub if not already cached.
    async fn ensure_model_files() -> Result<(PathBuf, PathBuf, PathBuf)> {
        let _models_dir = hardware::models_dir()?;

        let api = hf_hub::api::sync::Api::new().map_err(|e| {
            GhostError::NativeModel(format!("Failed to init HuggingFace Hub API: {}", e))
        })?;

        let repo = api.model(DEFAULT_MODEL_REPO.to_string());

        tracing::info!("Ensuring model files for {}", DEFAULT_MODEL_REPO);

        let model_path = repo.get("model.safetensors").map_err(|e| {
            GhostError::NativeModel(format!(
                "Failed to download model.safetensors: {}. Check your internet for first-time setup.",
                e
            ))
        })?;

        let tokenizer_path = repo.get("tokenizer.json").map_err(|e| {
            GhostError::NativeModel(format!("Failed to download tokenizer.json: {}", e))
        })?;

        let config_path = repo.get("config.json").map_err(|e| {
            GhostError::NativeModel(format!("Failed to download config.json: {}", e))
        })?;

        tracing::info!(
            "Model files ready: model={}, tokenizer={}, config={}",
            model_path.display(),
            tokenizer_path.display(),
            config_path.display()
        );

        Ok((model_path, tokenizer_path, config_path))
    }
}
