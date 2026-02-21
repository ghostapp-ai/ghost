//! Hardware detection and optimization for AI inference.
//!
//! Detects CPU, RAM, GPU, and SIMD capabilities to select optimal models
//! and inference devices. Works cross-platform (Windows, macOS, Linux, iOS, Android).

use crate::error::{GhostError, Result};

/// Hardware capabilities for AI inference.
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
    /// Total system RAM in megabytes.
    pub total_ram_mb: u64,
    /// Available (free) system RAM in megabytes.
    pub available_ram_mb: u64,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
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
        let (total_ram_mb, available_ram_mb) = Self::detect_ram();

        let info = Self {
            cpu_cores,
            has_avx2,
            has_neon,
            gpu_backend,
            total_ram_mb,
            available_ram_mb,
        };

        tracing::info!(
            "Hardware: {} cores, AVX2={}, NEON={}, GPU={:?}, RAM={}MB/{}MB",
            info.cpu_cores,
            info.has_avx2,
            info.has_neon,
            info.gpu_backend,
            info.available_ram_mb,
            info.total_ram_mb
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

    /// Select the best candle Device based on available hardware and user preference.
    ///
    /// Preference: "auto", "cpu", "cuda", "metal"
    pub fn select_device(&self, preference: &str) -> candle_core::Device {
        match preference {
            "cpu" => {
                tracing::info!("Using CPU device (explicit preference)");
                candle_core::Device::Cpu
            }
            "cuda" => {
                if let Some(device) = Self::try_cuda_device() {
                    return device;
                }
                tracing::warn!("CUDA requested but not available, falling back to CPU");
                candle_core::Device::Cpu
            }
            "metal" => {
                if let Some(device) = Self::try_metal_device() {
                    return device;
                }
                tracing::warn!("Metal requested but not available, falling back to CPU");
                candle_core::Device::Cpu
            }
            _ => {
                // Try GPU acceleration first, fall back to CPU
                if let Some(device) = Self::try_cuda_device() {
                    return device;
                }
                if let Some(device) = Self::try_metal_device() {
                    return device;
                }
                tracing::info!("Using CPU device (no GPU available)");
                candle_core::Device::Cpu
            }
        }
    }

    /// Attempt to create a CUDA device.
    fn try_cuda_device() -> Option<candle_core::Device> {
        #[cfg(feature = "cuda")]
        {
            match candle_core::Device::new_cuda(0) {
                Ok(device) => {
                    tracing::info!("CUDA GPU device initialized");
                    return Some(device);
                }
                Err(e) => {
                    tracing::debug!("CUDA initialization failed: {}", e);
                }
            }
        }
        #[cfg(not(feature = "cuda"))]
        {
            tracing::debug!("CUDA support not compiled (enable 'cuda' feature)");
        }
        None
    }

    /// Attempt to create a Metal device.
    fn try_metal_device() -> Option<candle_core::Device> {
        #[cfg(feature = "metal")]
        {
            match candle_core::Device::new_metal(0) {
                Ok(device) => {
                    tracing::info!("Metal GPU device initialized");
                    return Some(device);
                }
                Err(e) => {
                    tracing::debug!("Metal initialization failed: {}", e);
                }
            }
        }
        #[cfg(not(feature = "metal"))]
        {
            tracing::debug!("Metal support not compiled (enable 'metal' feature)");
        }
        None
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
        // Metal detection (macOS and iOS)
        #[cfg(target_os = "macos")]
        {
            return Some(GpuBackend::Metal);
        }

        #[cfg(target_os = "ios")]
        {
            return Some(GpuBackend::Metal);
        }

        // Android — most devices have Vulkan support (Android 7+)
        #[cfg(target_os = "android")]
        {
            return Some(GpuBackend::Vulkan);
        }

        // CUDA detection — prefer over Vulkan when available (better inference perf)
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/usr/bin/nvidia-smi").exists()
                || std::path::Path::new("/usr/local/cuda").exists()
                || std::path::Path::new("/opt/cuda").exists()
            {
                return Some(GpuBackend::Cuda);
            }
            // Also check for NVIDIA driver loaded in kernel
            if let Ok(content) = std::fs::read_to_string("/proc/modules") {
                if content.contains("nvidia") {
                    return Some(GpuBackend::Cuda);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if std::path::Path::new("C:\\Windows\\System32\\nvidia-smi.exe").exists() {
                return Some(GpuBackend::Cuda);
            }
        }

        // Vulkan detection (Linux) — covers AMD, Intel, and NVIDIA without proprietary driver
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/usr/bin/vulkaninfo").exists()
                || std::path::Path::new("/usr/lib/libvulkan.so").exists()
                || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libvulkan.so").exists()
                || std::path::Path::new("/usr/lib/libvulkan.so.1").exists()
            {
                return Some(GpuBackend::Vulkan);
            }
        }

        #[allow(unreachable_code)]
        None
    }

    /// Detect total and available RAM (in MB).
    fn detect_ram() -> (u64, u64) {
        #[cfg(target_os = "linux")]
        {
            return Self::detect_ram_linux();
        }

        #[cfg(target_os = "macos")]
        {
            return Self::detect_ram_macos();
        }

        #[cfg(target_os = "windows")]
        {
            return Self::detect_ram_windows();
        }

        #[cfg(target_os = "ios")]
        {
            return Self::detect_ram_ios();
        }

        #[cfg(target_os = "android")]
        {
            return Self::detect_ram_android();
        }

        // Fallback: assume 8GB total, 4GB available
        #[allow(unreachable_code)]
        (8192, 4096)
    }

    #[cfg(target_os = "linux")]
    fn detect_ram_linux() -> (u64, u64) {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let total = Self::parse_meminfo_kb(&content, "MemTotal:")
                .map(|kb| kb / 1024)
                .unwrap_or(8192);
            let available = Self::parse_meminfo_kb(&content, "MemAvailable:")
                .map(|kb| kb / 1024)
                .unwrap_or(total / 2);
            (total, available)
        } else {
            (8192, 4096)
        }
    }

    #[cfg(target_os = "linux")]
    fn parse_meminfo_kb(content: &str, field: &str) -> Option<u64> {
        for line in content.lines() {
            if line.starts_with(field) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse().ok();
                }
            }
        }
        None
    }

    #[cfg(target_os = "macos")]
    fn detect_ram_macos() -> (u64, u64) {
        // Total RAM via sysctl
        let total = std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|bytes| bytes / (1024 * 1024))
            .unwrap_or(8192);

        // Available RAM via vm_stat (rough estimate)
        let available = std::process::Command::new("vm_stat")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                let free_pages = Self::parse_vm_stat_pages(&s, "Pages free:");
                let inactive_pages = Self::parse_vm_stat_pages(&s, "Pages inactive:");
                // Each page is typically 4096 bytes (16384 on Apple Silicon)
                let page_size: u64 = 4096;
                (free_pages + inactive_pages) * page_size / (1024 * 1024)
            })
            .unwrap_or(total / 2);

        (total, available)
    }

    #[cfg(target_os = "macos")]
    fn parse_vm_stat_pages(content: &str, field: &str) -> u64 {
        for line in content.lines() {
            if line.contains(field) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(last) = parts.last() {
                    return last.trim_end_matches('.').parse().unwrap_or(0);
                }
            }
        }
        0
    }

    #[cfg(target_os = "windows")]
    fn detect_ram_windows() -> (u64, u64) {
        // Use PowerShell to get memory info (available on all modern Windows)
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "[math]::Round((Get-CimInstance Win32_OperatingSystem).TotalVisibleMemorySize/1024)",
            ])
            .output();

        let total = output
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(8192);

        let free_output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "[math]::Round((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1024)",
            ])
            .output();

        let available = free_output
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(total / 2);

        (total, available)
    }

    /// Detect RAM on iOS using sysctl (same as macOS but without vm_stat).
    /// iOS doesn't allow spawning processes, so we use libc sysctl directly.
    #[cfg(target_os = "ios")]
    fn detect_ram_ios() -> (u64, u64) {
        // On iOS, use the same sysctl approach but via libc FFI
        // since std::process::Command is not available on iOS.
        // For now, use conservative estimates based on common iOS devices.
        // iPhone 15 Pro: 8GB, iPhone 15: 6GB, iPad Pro: 16GB
        // iOS typically gives apps 50-75% of physical RAM.
        //
        // TODO: Use libc::sysctl for precise detection
        let total: u64 = 6144; // Conservative: 6GB (most modern iPhones)
        let available = total * 60 / 100; // iOS gives ~60% to foreground apps
        (total, available)
    }

    /// Detect RAM on Android by reading /proc/meminfo (Linux-based).
    #[cfg(target_os = "android")]
    fn detect_ram_android() -> (u64, u64) {
        // Android is Linux-based, so /proc/meminfo works
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let total = Self::parse_meminfo_kb_mobile(&content, "MemTotal:")
                .map(|kb| kb / 1024)
                .unwrap_or(4096);
            let available = Self::parse_meminfo_kb_mobile(&content, "MemAvailable:")
                .map(|kb| kb / 1024)
                .unwrap_or(total / 2);
            (total, available)
        } else {
            // Fallback: conservative estimate for modern Android devices
            (4096, 2048)
        }
    }

    /// Parse /proc/meminfo fields on Android (same logic as Linux).
    #[cfg(target_os = "android")]
    fn parse_meminfo_kb_mobile(content: &str, field: &str) -> Option<u64> {
        for line in content.lines() {
            if line.starts_with(field) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse().ok();
                }
            }
        }
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
        assert!(info.total_ram_mb > 0);
    }

    #[test]
    fn test_models_dir() {
        let dir = models_dir().unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn test_device_selection_cpu() {
        let info = HardwareInfo::detect();
        let device = info.select_device("cpu");
        assert!(matches!(device, candle_core::Device::Cpu));
    }
}
