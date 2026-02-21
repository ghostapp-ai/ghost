//! Hardware-adaptive inference configuration for llama.cpp.
//!
//! Provides a unified `InferenceProfile` that auto-detects the best parameters
//! for any hardware — from 2-core/2GB ARM devices to 32-core/64GB workstations
//! with discrete GPUs. Used by both the chat engine and the agent executor.
//!
//! # Key decisions made automatically:
//! - **GPU layers**: Detects VRAM via llama.cpp backends, calculates how many
//!   model layers fit in GPU memory (partial offload if needed).
//! - **Thread count**: Separate counts for generation (memory-bound, fewer threads)
//!   and batch prefill (compute-bound, more threads).
//! - **Batch & context sizes**: Scaled to available RAM.
//! - **KV cache quantization**: Q8_0 for ≥4GB RAM, Q4_0 for low-memory systems.
//! - **Flash attention**: AUTO — llama.cpp enables it when the backend supports it.
//! - **mlock**: Locks model pages in RAM when enough memory headroom exists.

use llama_cpp_2::context::params::{KvCacheType, LlamaContextParams};
use llama_cpp_2::model::params::LlamaModelParams;
use std::num::NonZeroU32;

/// Detected GPU information from llama.cpp backends.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GpuInfo {
    /// Human-readable device name (e.g., "NVIDIA GeForce RTX 3060 (Vulkan)")
    pub name: String,
    /// Total device VRAM in megabytes.
    pub vram_total_mb: u64,
    /// Free (available) VRAM in megabytes.
    pub vram_free_mb: u64,
    /// Device type string (e.g., "Gpu", "IntegratedGpu").
    pub device_type: String,
}

/// Hardware-adaptive inference configuration.
///
/// All parameters are pre-computed from hardware detection and model size.
/// Call `InferenceProfile::auto()` to get the optimal config for the current system.
#[derive(Debug, Clone)]
pub struct InferenceProfile {
    /// Number of model layers to offload to GPU (0 = CPU-only, 9999 = all).
    pub n_gpu_layers: u32,
    /// Thread count for token generation (decode step — memory-bandwidth bound).
    pub n_threads_gen: i32,
    /// Thread count for batch prefill (prompt processing — compute bound).
    pub n_threads_batch: i32,
    /// Maximum tokens per decode() batch call.
    pub n_batch: u32,
    /// Context window size (max prompt + response tokens).
    pub n_ctx: u32,
    /// KV cache quantization type (Q8_0 or Q4_0).
    pub kv_cache_type: KvCacheType,
    /// Whether to lock model pages in RAM (prevent swap).
    pub use_mlock: bool,
    /// Flash attention policy: -1=AUTO, 0=disabled, 1=enabled.
    pub flash_attn_policy: i32,
    /// Whether to offload KQV (attention) operations to GPU.
    pub offload_kqv: bool,
    /// Detected GPU info (None if CPU-only).
    pub gpu: Option<GpuInfo>,
}

/// VRAM overhead reserved for KV cache, compute buffers, and framework allocations.
/// Conservative estimate — prevents OOM on edge cases.
const GPU_OVERHEAD_MB: u64 = 300;

/// LLAMA_FLASH_ATTN_TYPE_AUTO from llama_cpp_sys_2 (not a direct dependency).
const FLASH_ATTN_AUTO: i32 = -1;

impl InferenceProfile {
    /// Auto-detect the optimal inference configuration for the current hardware.
    ///
    /// # Arguments
    /// * `model_size_mb` - Approximate model file size in megabytes (from ModelProfile).
    /// * `model_n_layers` - Number of transformer layers in the model (from ModelProfile).
    ///
    /// # Strategy
    /// 1. Detect GPU devices via llama.cpp backend enumeration
    /// 2. Query VRAM to calculate how many layers fit on GPU
    /// 3. Detect CPU cores and RAM to set threads, batch size, context window
    /// 4. Select KV cache quantization based on available memory
    pub fn auto(model_size_mb: u64, model_n_layers: u32) -> Self {
        let gpu = Self::detect_best_gpu();
        let (total_ram_mb, available_ram_mb) = Self::detect_ram();
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        // ── GPU layer allocation ─────────────────────────────────────

        let n_gpu_layers = match &gpu {
            Some(g) if g.vram_free_mb > 0 => {
                Self::calculate_gpu_layers(model_size_mb, model_n_layers, g.vram_free_mb)
            }
            _ => 0,
        };

        // ── Thread optimization ──────────────────────────────────────
        // Research findings (Reddit r/LocalLLaMA, llama.cpp benchmarks):
        //
        // Token generation (decode): memory-bandwidth limited, NOT compute-limited.
        //   Using too many threads causes L1/L2 cache thrashing between hyperthreads.
        //   Optimal: physical cores only (~50-66% of logical cores).
        //   On small core counts (≤4), use all cores — no room for waste.
        //
        // Batch prefill: compute-bound (matrix multiplication).
        //   More threads help here. Use 75-100% of available cores.
        //   Still leave headroom for UI thread and OS.

        let (n_threads_gen, n_threads_batch) = Self::optimal_threads(cores);

        // ── Context window and batch size based on RAM ───────────────
        // Larger context = more memory for KV cache.
        // Larger batch = faster prefill but more memory during prompt processing.
        //
        // RAM tiers:
        //   ≤2GB:  Embedded/old devices — minimal config
        //   2-4GB: Budget laptops, phones — conservative
        //   4-8GB: Average desktop — standard
        //   8-16GB: Good desktop/laptop — generous
        //   16GB+: Workstation/gaming — maximum

        let (n_ctx, n_batch) = Self::adaptive_context_batch(available_ram_mb, model_size_mb);

        // ── KV cache quantization ────────────────────────────────────
        // Q8_0: ~50% less memory than F16, negligible quality loss.
        //        Best default for ≥4GB RAM.
        // Q4_0: ~75% less memory than F16, very slight quality loss.
        //        Critical for low-memory systems where every MB counts.

        let kv_cache_type = if available_ram_mb < 4096 {
            KvCacheType::Q4_0
        } else {
            KvCacheType::Q8_0
        };

        // ── mlock: lock model in RAM ─────────────────────────────────
        // Prevents OS from swapping model pages to disk during inference.
        // Only enable when we have enough headroom (model + 50% + 1GB for OS).
        // On low-memory systems, mlock can starve other processes.

        let mlock_supported = llama_cpp_2::mlock_supported();
        let mlock_headroom = model_size_mb * 3 / 2 + 1024; // model×1.5 + 1GB OS headroom
        let use_mlock = mlock_supported && available_ram_mb > mlock_headroom;

        let offload_kqv = n_gpu_layers > 0;

        let profile = Self {
            n_gpu_layers,
            n_threads_gen,
            n_threads_batch,
            n_batch,
            n_ctx,
            kv_cache_type,
            use_mlock,
            flash_attn_policy: FLASH_ATTN_AUTO,
            offload_kqv,
            gpu: gpu.clone(),
        };

        tracing::info!(
            "InferenceProfile: gpu_layers={}, threads_gen={}, threads_batch={}, \
             batch={}, ctx={}, kv_cache={}, mlock={}, flash_attn=AUTO, offload_kqv={}, \
             gpu={}, ram={}MB/{}MB",
            profile.n_gpu_layers,
            profile.n_threads_gen,
            profile.n_threads_batch,
            profile.n_batch,
            profile.n_ctx,
            if available_ram_mb < 4096 {
                "Q4_0"
            } else {
                "Q8_0"
            },
            profile.use_mlock,
            profile.offload_kqv,
            gpu.as_ref()
                .map(|g| format!(
                    "{} ({}MB free/{}MB total)",
                    g.name, g.vram_free_mb, g.vram_total_mb
                ))
                .unwrap_or_else(|| "none".into()),
            available_ram_mb,
            total_ram_mb,
        );

        profile
    }

    /// Build `LlamaModelParams` from this profile.
    pub fn model_params(&self) -> LlamaModelParams {
        LlamaModelParams::default()
            .with_n_gpu_layers(self.n_gpu_layers)
            .with_use_mlock(self.use_mlock)
    }

    /// Build `LlamaContextParams` from this profile.
    ///
    /// # Arguments
    /// * `ctx_override` - Optional context size override (e.g., agent may need larger context).
    pub fn context_params(&self, ctx_override: Option<u32>) -> LlamaContextParams {
        let n_ctx = ctx_override.unwrap_or(self.n_ctx);

        LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(n_ctx))
            .with_n_batch(self.n_batch)
            .with_n_threads(self.n_threads_gen)
            .with_n_threads_batch(self.n_threads_batch)
            .with_flash_attention_policy(self.flash_attn_policy)
            .with_type_k(self.kv_cache_type)
            .with_type_v(self.kv_cache_type)
            .with_offload_kqv(self.offload_kqv)
    }

    /// Detect the best available GPU device from llama.cpp backends.
    ///
    /// Queries all registered backend devices and selects the GPU with the most
    /// free VRAM. Returns `None` if no GPU devices are found (CPU-only system,
    /// or no GPU feature compiled).
    fn detect_best_gpu() -> Option<GpuInfo> {
        let devices = llama_cpp_2::list_llama_ggml_backend_devices();

        // Log all detected devices for diagnostics
        for dev in &devices {
            tracing::debug!(
                "llama.cpp device: {} ({}) — type={:?}, vram={}MB/{}MB",
                dev.backend,
                dev.description,
                dev.device_type,
                dev.memory_free / (1024 * 1024),
                dev.memory_total / (1024 * 1024),
            );
        }

        devices
            .iter()
            .filter(|d| {
                matches!(
                    d.device_type,
                    llama_cpp_2::LlamaBackendDeviceType::Gpu
                        | llama_cpp_2::LlamaBackendDeviceType::IntegratedGpu
                )
            })
            .max_by_key(|d| d.memory_free)
            .map(|d| GpuInfo {
                name: format!("{} ({})", d.description, d.backend),
                vram_total_mb: (d.memory_total / (1024 * 1024)) as u64,
                vram_free_mb: (d.memory_free / (1024 * 1024)) as u64,
                device_type: format!("{:?}", d.device_type),
            })
    }

    /// Calculate how many model layers can be offloaded to GPU.
    ///
    /// # Strategy
    /// - If the full model + overhead fits in VRAM → offload everything (9999).
    /// - Otherwise → calculate proportional partial offload.
    /// - Returns 0 if practically no layers fit.
    ///
    /// Note: n_gpu_layers=9999 is a convention meaning "all layers" — llama.cpp
    /// clamps it to the actual layer count internally.
    fn calculate_gpu_layers(model_size_mb: u64, n_layers: u32, gpu_free_mb: u64) -> u32 {
        if gpu_free_mb == 0 || model_size_mb == 0 || n_layers == 0 {
            return 0;
        }

        let available_mb = gpu_free_mb.saturating_sub(GPU_OVERHEAD_MB);

        // Full model fits with 20% headroom → offload everything
        if available_mb >= model_size_mb * 120 / 100 {
            tracing::info!(
                "GPU: full model fits ({}MB available > {}MB × 1.2)",
                available_mb,
                model_size_mb
            );
            return 9999;
        }

        // Partial offload: calculate how many layers fit
        let mb_per_layer = model_size_mb / n_layers as u64;
        if mb_per_layer == 0 {
            return 0;
        }

        let layers = (available_mb / mb_per_layer) as u32;

        if layers < 2 {
            // Less than 2 layers isn't worth the CPU↔GPU transfer overhead
            tracing::info!(
                "GPU: only {} layers fit ({}MB available, {}MB/layer) — staying on CPU",
                layers,
                available_mb,
                mb_per_layer,
            );
            return 0;
        }

        tracing::info!(
            "GPU: partial offload {} of {} layers ({}MB available, {}MB/layer)",
            layers,
            n_layers,
            available_mb,
            mb_per_layer,
        );

        layers
    }

    /// Calculate optimal thread counts for generation and batch processing.
    ///
    /// Returns `(gen_threads, batch_threads)`.
    fn optimal_threads(logical_cores: usize) -> (i32, i32) {
        // Token generation: memory-bandwidth limited
        // Research consensus: using all hyperthreads HURTS performance.
        // Physical cores ≈ logical_cores / 2 on Intel/AMD (HT/SMT).
        // On ARM (no SMT typically), logical = physical.
        let gen_threads = match logical_cores {
            1..=2 => logical_cores,
            3..=4 => logical_cores,                  // Small CPUs: use all
            5..=8 => (logical_cores * 2 / 3).max(2), // ~66% (≈ physical cores)
            9..=16 => (logical_cores / 2).max(4),    // ~50% (= physical cores on HT)
            _ => (logical_cores / 2).max(4),         // Large CPUs: physical cores
        };

        // Batch prefill: compute-bound (matrix multiplication)
        // More threads are beneficial. Use 75-85% of cores.
        let batch_threads = match logical_cores {
            1..=4 => logical_cores,
            5..=8 => (logical_cores * 3 / 4).max(2),
            _ => (logical_cores * 4 / 5).max(4), // 80% for large core counts
        };

        (gen_threads as i32, batch_threads as i32)
    }

    /// Calculate adaptive context window and batch size based on available RAM.
    fn adaptive_context_batch(available_ram_mb: u64, model_size_mb: u64) -> (u32, u32) {
        // RAM available AFTER model is loaded (rough estimate)
        let ram_after_model = available_ram_mb.saturating_sub(model_size_mb);

        match ram_after_model {
            0..=1023 => (2048, 256),     // Very tight: minimal config
            1024..=2047 => (2048, 512),  // Low: standard batch, short context
            2048..=4095 => (4096, 512),  // Medium: full context
            4096..=8191 => (4096, 512),  // Good: full context, standard batch
            8192..=16383 => (8192, 512), // Great: extended context
            _ => (8192, 1024),           // Abundant: extended context, large batch
        }
    }

    /// Detect system RAM (total and available) in megabytes.
    ///
    /// Cross-platform: reads /proc/meminfo on Linux/Android, sysctl on macOS.
    fn detect_ram() -> (u64, u64) {
        // Reuse the existing HardwareInfo detection
        let hw = crate::embeddings::hardware::HardwareInfo::detect();
        (hw.total_ram_mb, hw.available_ram_mb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_layers_no_gpu() {
        assert_eq!(InferenceProfile::calculate_gpu_layers(1000, 28, 0), 0);
    }

    #[test]
    fn test_gpu_layers_full_fit() {
        // 2000MB model, 28 layers, 4000MB VRAM → full fit
        let layers = InferenceProfile::calculate_gpu_layers(2000, 28, 4000);
        assert_eq!(layers, 9999);
    }

    #[test]
    fn test_gpu_layers_partial_fit() {
        // 4000MB model, 28 layers, 2300MB VRAM → partial
        // available = 2300 - 300 = 2000MB, per_layer = 4000/28 ≈ 142MB
        // layers = 2000 / 142 ≈ 14
        let layers = InferenceProfile::calculate_gpu_layers(4000, 28, 2300);
        assert!(layers > 5 && layers < 28, "got {layers}");
    }

    #[test]
    fn test_gpu_layers_too_small() {
        // 4000MB model, 28 layers, 400MB VRAM → not enough for 2+ layers
        let layers = InferenceProfile::calculate_gpu_layers(4000, 28, 400);
        assert_eq!(layers, 0, "got {}", layers);
    }

    #[test]
    fn test_optimal_threads_small_cpu() {
        let (gen, batch) = InferenceProfile::optimal_threads(4);
        assert_eq!(gen, 4);
        assert_eq!(batch, 4);
    }

    #[test]
    fn test_optimal_threads_medium_cpu() {
        let (gen, batch) = InferenceProfile::optimal_threads(8);
        // gen: 8*2/3 = 5, batch: 8*3/4 = 6
        assert!((2..=8).contains(&gen), "gen={}", gen);
        assert!((2..=8).contains(&batch), "batch={}", batch);
    }

    #[test]
    fn test_optimal_threads_large_cpu() {
        let (gen, batch) = InferenceProfile::optimal_threads(16);
        // gen: 16/2 = 8, batch: 16*4/5 = 12
        assert_eq!(gen, 8);
        assert!((8..=16).contains(&batch), "batch={}", batch);
    }

    #[test]
    fn test_adaptive_context_low_ram() {
        let (ctx, batch) = InferenceProfile::adaptive_context_batch(2048, 1000);
        // 2048 - 1000 = 1048 → (2048, 512)
        assert_eq!(ctx, 2048);
        assert_eq!(batch, 512);
    }

    #[test]
    fn test_adaptive_context_high_ram() {
        let (ctx, batch) = InferenceProfile::adaptive_context_batch(32000, 4000);
        // 32000 - 4000 = 28000 → (8192, 1024)
        assert_eq!(ctx, 8192);
        assert_eq!(batch, 1024);
    }
}
