//! Model registry for Ghost chat engine.
//!
//! Defines available models with hardware requirements and auto-selection logic.
//! All models use the Qwen2.5-Instruct family (ChatML format, Apache 2.0 license).

use serde::Serialize;

use crate::embeddings::hardware::HardwareInfo;

/// A downloadable chat model profile.
#[derive(Debug, Clone, Serialize)]
pub struct ModelProfile {
    /// Unique identifier (e.g., "qwen2.5-0.5b").
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Short description for the UI.
    pub description: &'static str,
    /// HuggingFace repo containing the GGUF weights.
    pub repo_id: &'static str,
    /// GGUF filename within the repo.
    pub gguf_file: &'static str,
    /// HuggingFace repo containing the tokenizer.
    pub tokenizer_repo: &'static str,
    /// Approximate download size in MB.
    pub size_mb: u64,
    /// Minimum available RAM to run comfortably (MB).
    pub min_ram_mb: u64,
    /// Parameter count string (e.g., "0.5B").
    pub parameters: &'static str,
    /// Quality tier: 1=basic, 2=good, 3=better, 4=best.
    pub quality_tier: u8,
}

/// All available models, ordered from smallest to largest.
pub const MODEL_REGISTRY: &[ModelProfile] = &[
    ModelProfile {
        id: "qwen2.5-0.5b",
        name: "Qwen2.5 0.5B",
        description: "Ultra-fast, basic quality. Ideal for low-end hardware.",
        repo_id: "Qwen/Qwen2.5-0.5B-Instruct-GGUF",
        gguf_file: "qwen2.5-0.5b-instruct-q4_k_m.gguf",
        tokenizer_repo: "Qwen/Qwen2.5-0.5B-Instruct",
        size_mb: 400,
        min_ram_mb: 1024,
        parameters: "0.5B",
        quality_tier: 1,
    },
    ModelProfile {
        id: "qwen2.5-1.5b",
        name: "Qwen2.5 1.5B",
        description: "Good balance of speed and quality. Recommended for most PCs.",
        repo_id: "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
        gguf_file: "qwen2.5-1.5b-instruct-q4_k_m.gguf",
        tokenizer_repo: "Qwen/Qwen2.5-1.5B-Instruct",
        size_mb: 1000,
        min_ram_mb: 2048,
        parameters: "1.5B",
        quality_tier: 2,
    },
    ModelProfile {
        id: "qwen2.5-3b",
        name: "Qwen2.5 3B",
        description: "Higher quality responses. Needs 4GB+ free RAM.",
        repo_id: "Qwen/Qwen2.5-3B-Instruct-GGUF",
        gguf_file: "qwen2.5-3b-instruct-q4_k_m.gguf",
        tokenizer_repo: "Qwen/Qwen2.5-3B-Instruct",
        size_mb: 2000,
        min_ram_mb: 4096,
        parameters: "3B",
        quality_tier: 3,
    },
    ModelProfile {
        id: "qwen2.5-7b",
        name: "Qwen2.5 7B",
        description: "Best quality. Needs 8GB+ free RAM and patience.",
        repo_id: "Qwen/Qwen2.5-7B-Instruct-GGUF",
        gguf_file: "qwen2.5-7b-instruct-q3_k_m.gguf",
        tokenizer_repo: "Qwen/Qwen2.5-7B-Instruct",
        size_mb: 3810,
        min_ram_mb: 8192,
        parameters: "7B",
        quality_tier: 4,
    },
];

/// Find a model by ID. Returns None if not found.
pub fn find_model(id: &str) -> Option<&'static ModelProfile> {
    MODEL_REGISTRY.iter().find(|m| m.id == id)
}

/// Recommend the best model that fits the available hardware.
///
/// Strategy: pick the largest model that comfortably fits in available RAM,
/// leaving a 512MB headroom for the OS and app.
pub fn recommend_model(hardware: &HardwareInfo) -> &'static ModelProfile {
    let available = hardware.available_ram_mb;

    MODEL_REGISTRY
        .iter()
        .rev() // Start from largest
        .find(|m| available >= m.min_ram_mb + 512) // 512MB headroom
        .unwrap_or(&MODEL_REGISTRY[0]) // Fall back to smallest
}

/// Check if a model's GGUF file exists in the HuggingFace Hub cache.
pub fn is_model_cached(profile: &ModelProfile) -> bool {
    let cache_base = get_hf_cache_dir();
    let repo_name = profile.repo_id.replace('/', "--");
    let snapshots = cache_base
        .join(format!("models--{}", repo_name))
        .join("snapshots");

    if let Ok(entries) = std::fs::read_dir(&snapshots) {
        for entry in entries.flatten() {
            if entry.path().join(profile.gguf_file).exists() {
                return true;
            }
        }
    }
    false
}

/// Get the HuggingFace Hub cache directory.
pub fn get_hf_cache_dir() -> std::path::PathBuf {
    // Respect HF environment variables
    if let Ok(cache) = std::env::var("HF_HUB_CACHE") {
        return std::path::PathBuf::from(cache);
    }
    if let Ok(home) = std::env::var("HF_HOME") {
        return std::path::PathBuf::from(home).join("hub");
    }
    // Default cache location
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("huggingface")
        .join("hub")
}

/// Model info enriched with runtime status (for the frontend).
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub size_mb: u64,
    pub min_ram_mb: u64,
    pub parameters: String,
    pub quality_tier: u8,
    pub downloaded: bool,
    pub active: bool,
    pub recommended: bool,
    pub fits_hardware: bool,
}

/// Build a list of all available models with runtime status.
pub fn list_models(hardware: &HardwareInfo, active_model_id: &str) -> Vec<ModelInfo> {
    let recommended = recommend_model(hardware);

    MODEL_REGISTRY
        .iter()
        .map(|profile| ModelInfo {
            id: profile.id.to_string(),
            name: profile.name.to_string(),
            description: profile.description.to_string(),
            size_mb: profile.size_mb,
            min_ram_mb: profile.min_ram_mb,
            parameters: profile.parameters.to_string(),
            quality_tier: profile.quality_tier,
            downloaded: is_model_cached(profile),
            active: profile.id == active_model_id,
            recommended: profile.id == recommended.id,
            fits_hardware: hardware.available_ram_mb >= profile.min_ram_mb + 512,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_not_empty() {
        assert!(!MODEL_REGISTRY.is_empty());
    }

    #[test]
    fn test_find_model() {
        assert!(find_model("qwen2.5-0.5b").is_some());
        assert!(find_model("nonexistent").is_none());
    }

    #[test]
    fn test_recommend_low_ram() {
        let hw = HardwareInfo {
            cpu_cores: 2,
            has_avx2: false,
            has_neon: false,
            gpu_backend: None,
            total_ram_mb: 2048,
            available_ram_mb: 1500,
        };
        let model = recommend_model(&hw);
        assert_eq!(model.id, "qwen2.5-0.5b");
    }

    #[test]
    fn test_recommend_high_ram() {
        let hw = HardwareInfo {
            cpu_cores: 16,
            has_avx2: true,
            has_neon: false,
            gpu_backend: None,
            total_ram_mb: 32768,
            available_ram_mb: 24000,
        };
        let model = recommend_model(&hw);
        assert_eq!(model.id, "qwen2.5-7b");
    }

    #[test]
    fn test_models_ordered_by_size() {
        for window in MODEL_REGISTRY.windows(2) {
            assert!(window[0].size_mb <= window[1].size_mb);
        }
    }
}
