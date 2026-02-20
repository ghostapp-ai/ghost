//! Agent configuration — hardware-adaptive native model selection.
//!
//! Auto-detects system resources (RAM, VRAM, GPU) and selects the best
//! local GGUF model for agentic tool-calling use cases.
//!
//! Uses the SAME Qwen2.5-Instruct GGUF models already used for chat.
//! These models natively support tool calling via Hermes 2 Pro format
//! in llama.cpp, with GBNF grammar-constrained JSON output.
//!
//! Strategy:
//! - Detects available RAM via HardwareInfo
//! - Selects the largest Qwen2.5-Instruct model that fits comfortably
//! - For tool calling: minimum 1.5B recommended (smaller models hallucinate tools)
//! - All settings are auto-configured but user-overridable via Settings
//!
//! ZERO external dependencies — no Ollama, no server, no network after first download.

use serde::{Deserialize, Serialize};

use crate::embeddings::hardware::HardwareInfo;

/// Agent-specific configuration, persisted in Settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Native GGUF model for agent/tool-calling tasks (e.g., "qwen2.5-3b").
    /// "auto" = hardware-adaptive selection from the chat model registry.
    /// Uses the same Qwen2.5-Instruct family used for chat.
    #[serde(default = "default_agent_model")]
    pub agent_model: String,

    /// Maximum ReAct loop iterations before forced stop.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    /// Maximum tokens per LLM generation call.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    /// Context window size for inference.
    /// "auto" dynamically calculated from available RAM.
    #[serde(default = "default_context_window")]
    pub context_window: usize,

    /// Temperature for agent responses (lower = more deterministic tool calls).
    #[serde(default = "default_agent_temperature")]
    pub temperature: f64,

    /// Whether the agent can execute tools without user confirmation.
    /// When false, dangerous operations require explicit approval.
    #[serde(default = "default_auto_approve_safe")]
    pub auto_approve_safe: bool,

    /// Skills directory path (default: ~/.ghost/skills/).
    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
}

fn default_agent_model() -> String {
    "auto".into()
}
fn default_max_iterations() -> usize {
    10
}
fn default_max_tokens() -> usize {
    2048
}
fn default_context_window() -> usize {
    4096
}
fn default_agent_temperature() -> f64 {
    0.3
}
fn default_auto_approve_safe() -> bool {
    true
}
fn default_skills_dir() -> String {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".ghost")
        .join("skills")
        .to_string_lossy()
        .to_string()
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_model: default_agent_model(),
            max_iterations: default_max_iterations(),
            max_tokens: default_max_tokens(),
            context_window: default_context_window(),
            temperature: default_agent_temperature(),
            auto_approve_safe: default_auto_approve_safe(),
            skills_dir: default_skills_dir(),
        }
    }
}

/// Native GGUF model tier for agent use, ordered by capability.
///
/// Uses the same Qwen2.5-Instruct GGUF models from the chat registry.
/// These models support tool calling via Hermes 2 Pro format in llama.cpp.
#[derive(Debug, Clone, Serialize)]
pub struct AgentModelTier {
    /// Model ID from the chat model registry (e.g., "qwen2.5-3b").
    pub model_id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Minimum RAM in MB to run this model.
    pub min_ram_mb: u64,
    /// Recommended context window for this tier.
    pub recommended_ctx: usize,
    /// Whether this model supports reliable tool calling.
    pub tool_calling_reliable: bool,
    /// Quality tier: 1-4.
    pub quality: u8,
    /// Approximate RAM usage in MB with recommended ctx.
    pub approx_usage_mb: u64,
}

/// Available native GGUF model tiers for agent tasks.
///
/// Qwen2.5-Instruct family — Apache 2.0 license, ChatML format,
/// Hermes 2 Pro tool calling, grammar-constrained JSON output.
/// Uses the SAME models downloaded for the chat engine (no double download).
pub const AGENT_MODEL_TIERS: &[AgentModelTier] = &[
    AgentModelTier {
        model_id: "qwen2.5-0.5b",
        name: "Qwen2.5 0.5B (Agent)",
        min_ram_mb: 1024,
        recommended_ctx: 2048,
        tool_calling_reliable: false,
        quality: 1,
        approx_usage_mb: 600,
    },
    AgentModelTier {
        model_id: "qwen2.5-1.5b",
        name: "Qwen2.5 1.5B (Agent)",
        min_ram_mb: 2048,
        recommended_ctx: 4096,
        tool_calling_reliable: true,
        quality: 2,
        approx_usage_mb: 1200,
    },
    AgentModelTier {
        model_id: "qwen2.5-3b",
        name: "Qwen2.5 3B (Agent)",
        min_ram_mb: 4096,
        recommended_ctx: 4096,
        tool_calling_reliable: true,
        quality: 3,
        approx_usage_mb: 2400,
    },
    AgentModelTier {
        model_id: "qwen2.5-7b",
        name: "Qwen2.5 7B (Agent)",
        min_ram_mb: 8192,
        recommended_ctx: 4096,
        tool_calling_reliable: true,
        quality: 4,
        approx_usage_mb: 4500,
    },
];

/// Select the best agent model based on available hardware.
///
/// Prioritizes tool-calling reliability over raw quality.
/// Returns the model tier and recommended context window.
pub fn recommend_agent_model(hardware: &HardwareInfo) -> (&'static AgentModelTier, usize) {
    let available = hardware.available_ram_mb;

    // Reserve 1GB for OS + Ghost app overhead
    let usable = available.saturating_sub(1024);

    // Find the largest model that fits
    let tier = AGENT_MODEL_TIERS
        .iter()
        .rev()
        .find(|t| usable >= t.approx_usage_mb)
        .unwrap_or(&AGENT_MODEL_TIERS[0]);

    // Calculate optimal context window based on remaining RAM after model load.
    let ram_after_model = usable.saturating_sub(tier.approx_usage_mb);
    let ctx_from_ram = if tier.approx_usage_mb > 0 {
        let mb_per_1k_ctx = tier.approx_usage_mb as f64 / 80.0;
        let extra_ctx = (ram_after_model as f64 / mb_per_1k_ctx) as usize * 1024;
        (tier.recommended_ctx + extra_ctx).min(8192) // Cap at 8K for agent (tool prompts are large)
    } else {
        tier.recommended_ctx
    };

    // Never go below 2048 context
    let ctx = ctx_from_ram.max(2048);

    (tier, ctx)
}

/// Resolve the actual model ID from config (handles "auto").
///
/// Returns a model ID from the chat model registry (e.g., "qwen2.5-3b")
/// and the recommended context window size.
pub fn resolve_agent_model(config: &AgentConfig, hardware: &HardwareInfo) -> (String, usize) {
    if config.agent_model == "auto" {
        let (tier, ctx) = recommend_agent_model(hardware);
        (tier.model_id.to_string(), ctx)
    } else {
        // User specified a model — use their context window config
        (config.agent_model.clone(), config.context_window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert_eq!(config.agent_model, "auto");
        assert_eq!(config.max_iterations, 10);
        assert!(config.auto_approve_safe);
    }

    #[test]
    fn test_recommend_low_ram() {
        let hw = HardwareInfo {
            cpu_cores: 2,
            has_avx2: false,
            has_neon: false,
            gpu_backend: None,
            total_ram_mb: 4096,
            available_ram_mb: 2048,
        };
        let (tier, _ctx) = recommend_agent_model(&hw);
        assert_eq!(tier.model_id, "qwen2.5-0.5b");
    }

    #[test]
    fn test_recommend_8gb_ram() {
        let hw = HardwareInfo {
            cpu_cores: 8,
            has_avx2: true,
            has_neon: false,
            gpu_backend: None,
            total_ram_mb: 8192,
            available_ram_mb: 6144,
        };
        let (tier, _ctx) = recommend_agent_model(&hw);
        // 6144 - 1024 (OS) = 5120 usable → qwen2.5-7b needs 4500
        assert_eq!(tier.model_id, "qwen2.5-7b");
    }

    #[test]
    fn test_recommend_16gb_ram() {
        let hw = HardwareInfo {
            cpu_cores: 12,
            has_avx2: true,
            has_neon: false,
            gpu_backend: None,
            total_ram_mb: 16384,
            available_ram_mb: 12288,
        };
        let (tier, _ctx) = recommend_agent_model(&hw);
        // 12288 - 1024 = 11264 usable → qwen2.5-7b needs 4500 (largest)
        assert_eq!(tier.model_id, "qwen2.5-7b");
    }

    #[test]
    fn test_resolve_auto() {
        let config = AgentConfig::default();
        let hw = HardwareInfo {
            cpu_cores: 8,
            has_avx2: true,
            has_neon: false,
            gpu_backend: None,
            total_ram_mb: 16384,
            available_ram_mb: 8192,
        };
        let (model, ctx) = resolve_agent_model(&config, &hw);
        assert!(!model.is_empty());
        assert!(ctx >= 2048);
    }

    #[test]
    fn test_resolve_manual() {
        let config = AgentConfig {
            agent_model: "qwen2.5-3b".into(),
            context_window: 4096,
            ..Default::default()
        };
        let hw = HardwareInfo {
            cpu_cores: 4,
            has_avx2: false,
            has_neon: false,
            gpu_backend: None,
            total_ram_mb: 8192,
            available_ram_mb: 4096,
        };
        let (model, ctx) = resolve_agent_model(&config, &hw);
        assert_eq!(model, "qwen2.5-3b");
        assert_eq!(ctx, 4096);
    }

    #[test]
    fn test_tiers_use_chat_registry_models() {
        // Verify all agent model tiers reference valid chat model IDs
        for tier in AGENT_MODEL_TIERS {
            assert!(
                crate::chat::models::find_model(tier.model_id).is_some(),
                "Agent tier '{}' references unknown chat model '{}'",
                tier.name,
                tier.model_id
            );
        }
    }
}
