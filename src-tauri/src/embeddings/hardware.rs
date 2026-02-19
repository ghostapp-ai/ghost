use crate::error::{GhostError, Result};

/// Detect available hardware capabilities for AI inference.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HardwareInfo {
    /// Number of logical CPU cores.
    pub cpu_cores: usize,
    /// Whether the CPU supports AVX2 instructions (x86 SIMD).
    pub has_avx2: bool,
    /// Whether the CPU supports NEON instructions (ARM SIMD).
    pub has_neon: bool,
    /// Detected GPU backend (if any).
    pub gpu_backend: Option<GpuBackend>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum GpuBackend {
    Cuda,
    Metal,
    Vulkan,
}

impl HardwareInfo {
    /// Detect hardware capabilities of the current system.
    pub fn detect() -> Self {
        let cpu_cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);

        let has_avx2 = Self::detect_avx2();
        let has_neon = Self::detect_neon();
        let gpu_backend = Self::detect_gpu();

        let info = Self {
            cpu_cores,
            has_avx2,
            has_neon,
            gpu_backend,
        };

        tracing::info!(
            "Hardware detected: {} cores, AVX2={}, NEON={}, GPU={:?}",
            info.cpu_cores,
            info.has_avx2,
            info.has_neon,
            info.gpu_backend
        );

        info
    }

    /// Recommend the number of threads for inference.
    pub fn recommended_threads(&self) -> usize {
        // Use half the cores to avoid starving the UI/OS
        (self.cpu_cores / 2).max(1)
    }

    /// Check if hardware supports SIMD acceleration.
    pub fn has_simd(&self) -> bool {
        self.has_avx2 || self.has_neon
    }

    fn detect_avx2() -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            std::arch::is_x86_feature_detected!("avx2")
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false
        }
    }

    fn detect_neon() -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            // ARM NEON is mandatory on aarch64
            true
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            false
        }
    }

    fn detect_gpu() -> Option<GpuBackend> {
        // Metal detection (macOS)
        #[cfg(target_os = "macos")]
        {
            return Some(GpuBackend::Metal);
        }

        // CUDA detection (check for nvidia-smi)
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/usr/bin/nvidia-smi").exists()
                || std::path::Path::new("/usr/local/cuda").exists()
            {
                return Some(GpuBackend::Cuda);
            }
        }

        #[cfg(target_os = "windows")]
        {
            if std::path::Path::new("C:\\Windows\\System32\\nvidia-smi.exe").exists() {
                return Some(GpuBackend::Cuda);
            }
        }

        // Vulkan detection (check for vulkaninfo or libvulkan)
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/usr/bin/vulkaninfo").exists()
                || std::path::Path::new("/usr/lib/libvulkan.so").exists()
                || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libvulkan.so").exists()
            {
                return Some(GpuBackend::Vulkan);
            }
        }

        #[allow(unreachable_code)]
        None
    }
}

/// Get the default model storage directory.
pub fn models_dir() -> Result<std::path::PathBuf> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.ghost.app")
        .join("models");
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| GhostError::NativeModel(format!("Failed to create models dir: {}", e)))?;
    Ok(data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let info = HardwareInfo::detect();
        assert!(info.cpu_cores >= 1);
        assert!(info.recommended_threads() >= 1);
    }

    #[test]
    fn test_models_dir() {
        let dir = models_dir().unwrap();
        assert!(dir.exists());
    }
}
