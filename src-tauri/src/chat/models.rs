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
    /// Number of transformer layers in the model architecture.
    /// Used for smart GPU layer allocation (partial offload).
    pub n_layers: u32,
    /// Model family identifier for grouping in UI.
    pub family: &'static str,
    /// Whether this model supports thinking mode (/think, /no_think).
    pub supports_thinking: bool,
}

/// All available models, ordered from smallest to largest.
///
/// Qwen3 models are preferred (better tool calling, thinking mode, multilingual).
/// Qwen2.5 kept as fallback for lower RAM requirements and proven stability.
pub const MODEL_REGISTRY: &[ModelProfile] = &[
    // ── Qwen3 family (recommended — superior tool calling + thinking mode) ──
    ModelProfile {
        id: "qwen3-0.6b",
        name: "Qwen3 0.6B",
        description: "Ultra-fast with thinking mode. Ideal for low-end hardware and phones.",
        repo_id: "Qwen/Qwen3-0.6B-GGUF",
        gguf_file: "Qwen3-0.6B-Q8_0.gguf",
        tokenizer_repo: "Qwen/Qwen3-0.6B",
        size_mb: 639,
        min_ram_mb: 1024,
        parameters: "0.6B",
        quality_tier: 1,
        n_layers: 28,
        family: "qwen3",
        supports_thinking: true,
    },
    ModelProfile {
        id: "qwen3-1.7b",
        name: "Qwen3 1.7B",
        description: "Great balance of speed and quality with thinking mode.",
        repo_id: "Qwen/Qwen3-1.7B-GGUF",
        gguf_file: "Qwen3-1.7B-Q8_0.gguf",
        tokenizer_repo: "Qwen/Qwen3-1.7B",
        size_mb: 1830,
        min_ram_mb: 2560,
        parameters: "1.7B",
        quality_tier: 2,
        n_layers: 28,
        family: "qwen3",
        supports_thinking: true,
    },
    ModelProfile {
        id: "qwen3-4b",
        name: "Qwen3 4B",
        description: "Strong reasoning and tool calling. Recommended for 8GB+ systems.",
        repo_id: "Qwen/Qwen3-4B-GGUF",
        gguf_file: "Qwen3-4B-Q4_K_M.gguf",
        tokenizer_repo: "Qwen/Qwen3-4B",
        size_mb: 2500,
        min_ram_mb: 4096,
        parameters: "4B",
        quality_tier: 3,
        n_layers: 36,
        family: "qwen3",
        supports_thinking: true,
    },
    ModelProfile {
        id: "qwen3-8b",
        name: "Qwen3 8B",
        description: "Best quality. Thinking mode + superior tool calling. Needs 10GB+ RAM.",
        repo_id: "Qwen/Qwen3-8B-GGUF",
        gguf_file: "Qwen3-8B-Q4_K_M.gguf",
        tokenizer_repo: "Qwen/Qwen3-8B",
        size_mb: 5030,
        min_ram_mb: 10240,
        parameters: "8B",
        quality_tier: 4,
        n_layers: 36,
        family: "qwen3",
        supports_thinking: true,
    },
    // ── Qwen2.5 family (legacy — proven stability, lower RAM requirements) ──
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
        n_layers: 24,
        family: "qwen2.5",
        supports_thinking: false,
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
        n_layers: 28,
        family: "qwen2.5",
        supports_thinking: false,
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
        n_layers: 36,
        family: "qwen2.5",
        supports_thinking: false,
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
        n_layers: 28,
        family: "qwen2.5",
        supports_thinking: false,
    },
];

/// Find a model by ID. Returns None if not found.
pub fn find_model(id: &str) -> Option<&'static ModelProfile> {
    MODEL_REGISTRY.iter().find(|m| m.id == id)
}

/// Check if GPU offload is available at runtime via llama.cpp.
///
/// This detects actual GPU availability (Vulkan, CUDA, Metal) — not just
/// compile-time feature flags. Returns true if llama.cpp can offload to GPU.
/// On mobile, always returns false (llama.cpp not available).
pub fn has_gpu_runtime() -> bool {
    #[cfg(desktop)]
    {
        // Check if any GPU-type device is reported by llama.cpp backends
        let devices = llama_cpp_2::list_llama_ggml_backend_devices();
        devices.iter().any(|d| {
            matches!(
                d.device_type,
                llama_cpp_2::LlamaBackendDeviceType::Gpu
                    | llama_cpp_2::LlamaBackendDeviceType::IntegratedGpu
            )
        })
    }

    #[cfg(not(desktop))]
    {
        false
    }
}

/// Recommend the best model that fits the available hardware.
///
/// Strategy:
/// - Prefers Qwen3 family (better tool calling, thinking mode, multilingual)
/// - CPU-only (no GPU detected at runtime): cap at quality tier 2 for interactive speed
/// - GPU available: pick the largest that fits in RAM
/// - Always leave 512MB headroom for the OS and app
pub fn recommend_model(hardware: &HardwareInfo) -> &'static ModelProfile {
    let available = hardware.available_ram_mb;
    let has_gpu = has_gpu_runtime();

    // CPU-only: cap at tier 2 for acceptable inference speed.
    // 3B+ on CPU takes 10+ seconds per response which feels broken.
    let max_quality_tier: u8 = if has_gpu { 4 } else { 2 };

    // Prefer Qwen3 models (first in registry), fall back to Qwen2.5
    MODEL_REGISTRY
        .iter()
        .filter(|m| m.family == "qwen3") // Prefer Qwen3
        .rev() // Start from largest
        .find(|m| available >= m.min_ram_mb + 512 && m.quality_tier <= max_quality_tier)
        .or_else(|| {
            // Fall back to Qwen2.5 if no Qwen3 model fits
            MODEL_REGISTRY
                .iter()
                .filter(|m| m.family == "qwen2.5")
                .rev()
                .find(|m| available >= m.min_ram_mb + 512 && m.quality_tier <= max_quality_tier)
        })
        .unwrap_or(&MODEL_REGISTRY[0]) // Absolute fallback: smallest Qwen3
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
    pub family: String,
    pub supports_thinking: bool,
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
            family: profile.family.to_string(),
            supports_thinking: profile.supports_thinking,
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
    fn test_model_registry_has_both_families() {
        let qwen3_count = MODEL_REGISTRY
            .iter()
            .filter(|m| m.family == "qwen3")
            .count();
        let qwen25_count = MODEL_REGISTRY
            .iter()
            .filter(|m| m.family == "qwen2.5")
            .count();
        assert!(qwen3_count >= 4, "Expected at least 4 Qwen3 models");
        assert!(qwen25_count >= 4, "Expected at least 4 Qwen2.5 models");
    }

    #[test]
    fn test_find_model() {
        assert!(find_model("qwen3-0.6b").is_some());
        assert!(find_model("qwen2.5-0.5b").is_some());
        assert!(find_model("nonexistent").is_none());
    }

    #[test]
    fn test_qwen3_models_have_thinking() {
        for m in MODEL_REGISTRY.iter().filter(|m| m.family == "qwen3") {
            assert!(m.supports_thinking, "{} should support thinking", m.id);
        }
        for m in MODEL_REGISTRY.iter().filter(|m| m.family == "qwen2.5") {
            assert!(!m.supports_thinking, "{} should not support thinking", m.id);
        }
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
        // Should recommend smallest Qwen3 (0.6B)
        assert_eq!(model.id, "qwen3-0.6b");
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
        // Without GPU at runtime, caps at tier 2 — Qwen3-1.7B
        // With GPU, would select Qwen3-8B
        if has_gpu_runtime() {
            assert_eq!(model.id, "qwen3-8b");
        } else {
            assert_eq!(model.id, "qwen3-1.7b");
        }
    }

    #[test]
    fn test_models_quality_tiers_valid() {
        for m in MODEL_REGISTRY {
            assert!(
                m.quality_tier >= 1 && m.quality_tier <= 4,
                "{} has invalid quality tier {}",
                m.id,
                m.quality_tier
            );
        }
    }
}
