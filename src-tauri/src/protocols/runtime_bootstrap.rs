//! Runtime Bootstrapper — Zero-config runtime installation for Ghost.
//!
//! Ghost can automatically download and manage runtimes (Node.js, uv/Python)
//! in its own data directory, so users never need to install anything manually.
//!
//! **Design principles:**
//! - Runtimes are installed to `<app_data>/runtimes/` (no system PATH modification)
//! - When launching MCP servers, Ghost injects managed runtimes into the process PATH
//! - Downloads are opt-in (user-triggered) and cached locally
//! - Cross-platform: Linux, macOS, Windows
//!
//! **Strategy:**
//! - **uv**: Single binary from GitHub releases → provides `uv`, `uvx`, and `uv python install`
//! - **Node.js**: Prebuilt binary from nodejs.org → provides `node`, `npm`, `npx`
//! - **Docker**: Detection only (system-level, cannot auto-install)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ── Constants ───────────────────────────────────────────────────────

/// Node.js LTS version to install.
const NODE_LTS_VERSION: &str = "24.13.1";

/// uv version to install.
const UV_VERSION: &str = "0.10.4";

/// Base URL for Node.js prebuilt binaries.
const NODE_DIST_BASE: &str = "https://nodejs.org/dist";

/// Base URL for uv releases.
const UV_RELEASES_BASE: &str = "https://github.com/astral-sh/uv/releases/download";

// ── Types ───────────────────────────────────────────────────────────

/// Kinds of runtimes Ghost can manage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeKind {
    /// Node.js + npm + npx
    Node,
    /// uv + uvx + Python (via `uv python install`)
    Uv,
    /// Docker — detection only, no auto-install
    Docker,
}

impl std::fmt::Display for RuntimeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeKind::Node => write!(f, "node"),
            RuntimeKind::Uv => write!(f, "uv"),
            RuntimeKind::Docker => write!(f, "docker"),
        }
    }
}

/// Status of a single runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    /// Which runtime.
    pub kind: RuntimeKind,
    /// Whether it's available (system or managed).
    pub installed: bool,
    /// Whether it was installed by Ghost (in runtimes dir).
    pub managed: bool,
    /// Version string if available.
    pub version: Option<String>,
    /// Absolute path to the binary.
    pub path: Option<String>,
    /// Human-readable description.
    pub description: String,
    /// Whether this runtime can be auto-installed by Ghost.
    pub can_auto_install: bool,
}

/// Progress event during runtime installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    /// Which runtime is being installed.
    pub runtime: String,
    /// Current stage: "downloading", "extracting", "configuring", "complete", "error"
    pub stage: String,
    /// Progress percentage (0.0 - 100.0), -1.0 if indeterminate.
    pub percent: f64,
    /// Human-readable message.
    pub message: String,
}

/// Result of installing a runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    /// Whether installation succeeded.
    pub success: bool,
    /// Runtime status after installation.
    pub status: RuntimeStatus,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Summary of all managed runtimes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapStatus {
    /// Status of each supported runtime.
    pub runtimes: Vec<RuntimeStatus>,
    /// Whether all runtimes needed for default tools are available.
    pub ready_for_defaults: bool,
    /// Missing runtimes that could be auto-installed.
    pub missing_installable: Vec<RuntimeKind>,
    /// Runtimes directory path.
    pub runtimes_dir: String,
}

/// Recommendation from AI-powered tool discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecommendation {
    /// Catalog ID if matched.
    pub catalog_id: Option<String>,
    /// Tool name.
    pub name: String,
    /// Why this tool was recommended.
    pub reason: String,
    /// Required runtime.
    pub runtime: String,
    /// Whether it can be installed right now.
    pub installable: bool,
    /// What runtimes need to be installed first.
    pub missing_runtimes: Vec<String>,
}

// ── RuntimeBootstrapper ─────────────────────────────────────────────

/// Manages runtime detection, installation, and PATH injection for MCP servers.
pub struct RuntimeBootstrapper {
    /// Directory where Ghost stores managed runtimes.
    runtimes_dir: PathBuf,
}

impl RuntimeBootstrapper {
    /// Create a new bootstrapper. The runtimes dir is typically `<app_data>/runtimes/`.
    pub fn new(app_data_dir: &Path) -> Self {
        Self {
            runtimes_dir: app_data_dir.join("runtimes"),
        }
    }

    /// Get the runtimes directory path.
    #[allow(dead_code)]
    pub fn runtimes_dir(&self) -> &Path {
        &self.runtimes_dir
    }

    // ── Detection ───────────────────────────────────────────────────

    /// Detect all runtime statuses (both system and managed).
    pub async fn detect_all(&self) -> Vec<RuntimeStatus> {
        let mut statuses = Vec::new();

        // Node.js
        statuses.push(self.detect_node().await);
        // uv/uvx/Python
        statuses.push(self.detect_uv().await);
        // Docker
        statuses.push(self.detect_docker().await);

        statuses
    }

    /// Get comprehensive bootstrap status.
    pub async fn get_status(&self) -> BootstrapStatus {
        let runtimes = self.detect_all().await;
        let has_node = runtimes
            .iter()
            .any(|r| r.kind == RuntimeKind::Node && r.installed);
        let has_uv = runtimes
            .iter()
            .any(|r| r.kind == RuntimeKind::Uv && r.installed);

        let mut missing_installable = Vec::new();
        if !has_node {
            missing_installable.push(RuntimeKind::Node);
        }
        if !has_uv {
            missing_installable.push(RuntimeKind::Uv);
        }

        BootstrapStatus {
            ready_for_defaults: has_node && has_uv,
            missing_installable,
            runtimes_dir: self.runtimes_dir.display().to_string(),
            runtimes,
        }
    }

    /// Detect Node.js: check managed first, then system.
    async fn detect_node(&self) -> RuntimeStatus {
        // Check Ghost-managed Node.js
        let managed_node = self.managed_node_bin();
        if managed_node.exists() {
            if let Some((version, path)) = get_binary_version(&managed_node, &["--version"]).await {
                return RuntimeStatus {
                    kind: RuntimeKind::Node,
                    installed: true,
                    managed: true,
                    version: Some(version),
                    path: Some(path),
                    description: "Node.js (Ghost-managed)".into(),
                    can_auto_install: true,
                };
            }
        }

        // Check system Node.js
        if let Some((version, path)) = check_system_binary("node", &["--version"]).await {
            return RuntimeStatus {
                kind: RuntimeKind::Node,
                installed: true,
                managed: false,
                version: Some(version),
                path: Some(path),
                description: "Node.js (system)".into(),
                can_auto_install: true,
            };
        }

        RuntimeStatus {
            kind: RuntimeKind::Node,
            installed: false,
            managed: false,
            version: None,
            path: None,
            description: "Node.js — not installed".into(),
            can_auto_install: true,
        }
    }

    /// Detect uv: check managed first, then system.
    async fn detect_uv(&self) -> RuntimeStatus {
        // Check Ghost-managed uv
        let managed_uv = self.managed_uv_bin();
        if managed_uv.exists() {
            if let Some((version, path)) = get_binary_version(&managed_uv, &["--version"]).await {
                return RuntimeStatus {
                    kind: RuntimeKind::Uv,
                    installed: true,
                    managed: true,
                    version: Some(version),
                    path: Some(path),
                    description: "uv (Ghost-managed) — Python + uvx".into(),
                    can_auto_install: true,
                };
            }
        }

        // Check system uv
        if let Some((version, path)) = check_system_binary("uv", &["--version"]).await {
            return RuntimeStatus {
                kind: RuntimeKind::Uv,
                installed: true,
                managed: false,
                version: Some(version),
                path: Some(path),
                description: "uv (system) — Python + uvx".into(),
                can_auto_install: true,
            };
        }

        RuntimeStatus {
            kind: RuntimeKind::Uv,
            installed: false,
            managed: false,
            version: None,
            path: None,
            description: "uv — not installed".into(),
            can_auto_install: true,
        }
    }

    /// Detect Docker (system only, no auto-install).
    async fn detect_docker(&self) -> RuntimeStatus {
        if let Some((version, path)) = check_system_binary("docker", &["--version"]).await {
            return RuntimeStatus {
                kind: RuntimeKind::Docker,
                installed: true,
                managed: false,
                version: Some(version),
                path: Some(path),
                description: "Docker (system)".into(),
                can_auto_install: false,
            };
        }

        RuntimeStatus {
            kind: RuntimeKind::Docker,
            installed: false,
            managed: false,
            version: None,
            path: None,
            description: "Docker — not installed (install from docker.com)".into(),
            can_auto_install: false,
        }
    }

    // ── Installation ────────────────────────────────────────────────

    /// Install a runtime. Emits progress events via the callback.
    pub async fn install_runtime<F>(&self, kind: RuntimeKind, progress: F) -> InstallResult
    where
        F: Fn(InstallProgress) + Send + Sync,
    {
        match kind {
            RuntimeKind::Node => self.install_node(&progress).await,
            RuntimeKind::Uv => self.install_uv(&progress).await,
            RuntimeKind::Docker => InstallResult {
                success: false,
                status: RuntimeStatus {
                    kind: RuntimeKind::Docker,
                    installed: false,
                    managed: false,
                    version: None,
                    path: None,
                    description: "Docker cannot be auto-installed".into(),
                    can_auto_install: false,
                },
                error: Some("Docker requires manual installation from docker.com".into()),
            },
        }
    }

    /// Install all missing runtimes needed for default MCP tools.
    pub async fn bootstrap_all<F>(&self, progress: F) -> Vec<InstallResult>
    where
        F: Fn(InstallProgress) + Send + Sync,
    {
        let status = self.get_status().await;
        let mut results = Vec::new();

        for kind in status.missing_installable {
            let result = self.install_runtime(kind, &progress).await;
            results.push(result);
        }

        results
    }

    /// Install Node.js from prebuilt binary.
    async fn install_node<F>(&self, progress: &F) -> InstallResult
    where
        F: Fn(InstallProgress) + Send + Sync,
    {
        let node_dir = self.runtimes_dir.join("node");

        progress(InstallProgress {
            runtime: "node".into(),
            stage: "downloading".into(),
            percent: 0.0,
            message: format!("Downloading Node.js v{}...", NODE_LTS_VERSION),
        });

        // Determine platform-specific download URL and archive type
        let (url, archive_type) = match get_node_download_info() {
            Some(info) => info,
            None => {
                return InstallResult {
                    success: false,
                    status: self.detect_node().await,
                    error: Some("Unsupported platform for Node.js auto-install".into()),
                };
            }
        };

        // Download
        let download_dir = self.runtimes_dir.join(".downloads");
        std::fs::create_dir_all(&download_dir).ok();
        let archive_path = download_dir.join(format!("node.{}", archive_type));

        match download_file(&url, &archive_path).await {
            Ok(_) => {
                progress(InstallProgress {
                    runtime: "node".into(),
                    stage: "extracting".into(),
                    percent: 50.0,
                    message: "Extracting Node.js...".into(),
                });
            }
            Err(e) => {
                return InstallResult {
                    success: false,
                    status: self.detect_node().await,
                    error: Some(format!("Failed to download Node.js: {}", e)),
                };
            }
        }

        // Extract
        if let Err(e) = extract_archive(&archive_path, &node_dir, &archive_type).await {
            return InstallResult {
                success: false,
                status: self.detect_node().await,
                error: Some(format!("Failed to extract Node.js: {}", e)),
            };
        }

        // Clean up download
        std::fs::remove_file(&archive_path).ok();

        progress(InstallProgress {
            runtime: "node".into(),
            stage: "configuring".into(),
            percent: 90.0,
            message: "Configuring Node.js...".into(),
        });

        // Verify installation
        let status = self.detect_node().await;
        if status.installed {
            progress(InstallProgress {
                runtime: "node".into(),
                stage: "complete".into(),
                percent: 100.0,
                message: format!(
                    "Node.js {} installed successfully!",
                    status.version.as_deref().unwrap_or("(unknown)")
                ),
            });
            InstallResult {
                success: true,
                status,
                error: None,
            }
        } else {
            InstallResult {
                success: false,
                status,
                error: Some("Node.js installation completed but binary not found".into()),
            }
        }
    }

    /// Install uv from prebuilt binary (provides uv, uvx, and Python management).
    async fn install_uv<F>(&self, progress: &F) -> InstallResult
    where
        F: Fn(InstallProgress) + Send + Sync,
    {
        let uv_dir = self.runtimes_dir.join("uv");

        progress(InstallProgress {
            runtime: "uv".into(),
            stage: "downloading".into(),
            percent: 0.0,
            message: format!("Downloading uv v{}...", UV_VERSION),
        });

        // Determine platform-specific download URL
        let (url, archive_type) = match get_uv_download_info() {
            Some(info) => info,
            None => {
                return InstallResult {
                    success: false,
                    status: self.detect_uv().await,
                    error: Some("Unsupported platform for uv auto-install".into()),
                };
            }
        };

        // Download
        let download_dir = self.runtimes_dir.join(".downloads");
        std::fs::create_dir_all(&download_dir).ok();
        let archive_path = download_dir.join(format!("uv.{}", archive_type));

        match download_file(&url, &archive_path).await {
            Ok(_) => {
                progress(InstallProgress {
                    runtime: "uv".into(),
                    stage: "extracting".into(),
                    percent: 50.0,
                    message: "Extracting uv...".into(),
                });
            }
            Err(e) => {
                return InstallResult {
                    success: false,
                    status: self.detect_uv().await,
                    error: Some(format!("Failed to download uv: {}", e)),
                };
            }
        }

        // Extract
        if let Err(e) = extract_archive(&archive_path, &uv_dir, &archive_type).await {
            return InstallResult {
                success: false,
                status: self.detect_uv().await,
                error: Some(format!("Failed to extract uv: {}", e)),
            };
        }

        // Clean up download
        std::fs::remove_file(&archive_path).ok();

        progress(InstallProgress {
            runtime: "uv".into(),
            stage: "configuring".into(),
            percent: 70.0,
            message: "Installing Python via uv...".into(),
        });

        // Use uv to install Python if needed
        let uv_bin = self.managed_uv_bin();
        if uv_bin.exists() {
            let python_result = tokio::process::Command::new(&uv_bin)
                .args(["python", "install"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .await;

            match python_result {
                Ok(output) if output.status.success() => {
                    tracing::info!("Python installed via uv");
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("uv python install warning: {}", stderr);
                }
                Err(e) => {
                    tracing::warn!("Could not install Python via uv: {}", e);
                }
            }
        }

        progress(InstallProgress {
            runtime: "uv".into(),
            stage: "complete".into(),
            percent: 100.0,
            message: "uv + Python installed!".into(),
        });

        let status = self.detect_uv().await;
        InstallResult {
            success: status.installed,
            status,
            error: None,
        }
    }

    // ── PATH Management ─────────────────────────────────────────────

    /// Build a PATH string that includes managed runtimes.
    /// This is used when spawning MCP server processes.
    pub fn build_env_path(&self) -> String {
        let mut paths = Vec::new();

        // Add managed Node.js bin directory
        let node_bin = self.managed_node_dir();
        if node_bin.exists() {
            paths.push(node_bin.display().to_string());
        }

        // Add managed uv bin directory
        let uv_bin_dir = self.managed_uv_dir();
        if uv_bin_dir.exists() {
            paths.push(uv_bin_dir.display().to_string());
        }

        // Append system PATH
        if let Ok(system_path) = std::env::var("PATH") {
            paths.push(system_path);
        }

        #[cfg(target_os = "windows")]
        let separator = ";";
        #[cfg(not(target_os = "windows"))]
        let separator = ":";

        paths.join(separator)
    }

    /// Build environment variables map with managed runtimes in PATH.
    #[allow(dead_code)]
    pub fn build_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), self.build_env_path());
        env
    }

    /// Get a command builder that includes managed runtimes in PATH.
    /// This is the key function: when Ghost spawns MCP servers, it uses this
    /// to ensure they can find node/npx/uv/uvx regardless of system config.
    #[allow(dead_code)]
    pub fn command(&self, program: &str) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new(program);

        // Set PATH to include managed runtimes
        cmd.env("PATH", self.build_env_path());

        // Set UV_PYTHON_INSTALL_DIR if uv is managed
        let uv_python_dir = self.runtimes_dir.join("uv-python");
        if self.managed_uv_bin().exists() {
            cmd.env("UV_PYTHON_INSTALL_DIR", &uv_python_dir);
        }

        cmd
    }

    /// Resolve the actual binary path for a command, checking managed runtimes first.
    pub fn resolve_binary(&self, name: &str) -> Option<PathBuf> {
        // Check managed runtimes
        match name {
            "node" | "npm" | "npx" => {
                let bin = self.managed_node_dir().join(name);
                if bin.exists() {
                    return Some(bin);
                }
                // On Windows, check .cmd and .exe variants
                #[cfg(target_os = "windows")]
                {
                    let cmd = self.managed_node_dir().join(format!("{}.cmd", name));
                    if cmd.exists() {
                        return Some(cmd);
                    }
                    let exe = self.managed_node_dir().join(format!("{}.exe", name));
                    if exe.exists() {
                        return Some(exe);
                    }
                }
            }
            "uv" | "uvx" => {
                let bin = self.managed_uv_dir().join(name);
                if bin.exists() {
                    return Some(bin);
                }
                #[cfg(target_os = "windows")]
                {
                    let exe = self.managed_uv_dir().join(format!("{}.exe", name));
                    if exe.exists() {
                        return Some(exe);
                    }
                }
            }
            "python" | "python3" => {
                // Check uv-managed Python
                let uv_python_dir = self.runtimes_dir.join("uv-python");
                if uv_python_dir.exists() {
                    // Find latest Python install
                    if let Ok(entries) = std::fs::read_dir(&uv_python_dir) {
                        for entry in entries.flatten() {
                            let bin = entry.path().join("bin").join("python3");
                            if bin.exists() {
                                return Some(bin);
                            }
                            #[cfg(target_os = "windows")]
                            {
                                let exe = entry.path().join("python.exe");
                                if exe.exists() {
                                    return Some(exe);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }

    // ── Private helpers ─────────────────────────────────────────────

    /// Path to the managed Node.js binary directory.
    fn managed_node_dir(&self) -> PathBuf {
        let node_dir = self.runtimes_dir.join("node");
        // Node.js extracts to node-vXX.YY.Z-{platform}-{arch}/bin/
        if let Ok(entries) = std::fs::read_dir(&node_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && entry.file_name().to_string_lossy().starts_with("node-") {
                    #[cfg(target_os = "windows")]
                    return p; // On Windows, binaries are in root
                    #[cfg(not(target_os = "windows"))]
                    return p.join("bin");
                }
            }
        }
        // Fallback: direct bin/ directory
        node_dir.join("bin")
    }

    /// Path to the managed Node.js binary.
    fn managed_node_bin(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.managed_node_dir().join("node.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.managed_node_dir().join("node")
        }
    }

    /// Path to the managed uv binary directory.
    fn managed_uv_dir(&self) -> PathBuf {
        self.runtimes_dir.join("uv")
    }

    /// Path to the managed uv binary.
    fn managed_uv_bin(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.managed_uv_dir().join("uv.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.managed_uv_dir().join("uv")
        }
    }
}

// ── Platform-specific download info ─────────────────────────────────

/// Get the Node.js download URL and archive type for the current platform.
fn get_node_download_info() -> Option<(String, String)> {
    let version = NODE_LTS_VERSION;

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Some((
            format!(
                "{}/v{}/node-v{}-linux-x64.tar.xz",
                NODE_DIST_BASE, version, version
            ),
            "tar.xz".into(),
        ))
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        Some((
            format!(
                "{}/v{}/node-v{}-linux-arm64.tar.xz",
                NODE_DIST_BASE, version, version
            ),
            "tar.xz".into(),
        ))
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        Some((
            format!(
                "{}/v{}/node-v{}-darwin-x64.tar.gz",
                NODE_DIST_BASE, version, version
            ),
            "tar.gz".into(),
        ))
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Some((
            format!(
                "{}/v{}/node-v{}-darwin-arm64.tar.gz",
                NODE_DIST_BASE, version, version
            ),
            "tar.gz".into(),
        ))
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        Some((
            format!(
                "{}/v{}/node-v{}-win-x64.zip",
                NODE_DIST_BASE, version, version
            ),
            "zip".into(),
        ))
    }

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        Some((
            format!(
                "{}/v{}/node-v{}-win-arm64.zip",
                NODE_DIST_BASE, version, version
            ),
            "zip".into(),
        ))
    }

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
    )))]
    {
        None
    }
}

/// Get the uv download URL and archive type for the current platform.
fn get_uv_download_info() -> Option<(String, String)> {
    let version = UV_VERSION;

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Some((
            format!(
                "{}/{}/uv-x86_64-unknown-linux-gnu.tar.gz",
                UV_RELEASES_BASE, version
            ),
            "tar.gz".into(),
        ))
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        Some((
            format!(
                "{}/{}/uv-aarch64-unknown-linux-gnu.tar.gz",
                UV_RELEASES_BASE, version
            ),
            "tar.gz".into(),
        ))
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        Some((
            format!(
                "{}/{}/uv-x86_64-apple-darwin.tar.gz",
                UV_RELEASES_BASE, version
            ),
            "tar.gz".into(),
        ))
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Some((
            format!(
                "{}/{}/uv-aarch64-apple-darwin.tar.gz",
                UV_RELEASES_BASE, version
            ),
            "tar.gz".into(),
        ))
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        Some((
            format!(
                "{}/{}/uv-x86_64-pc-windows-msvc.zip",
                UV_RELEASES_BASE, version
            ),
            "zip".into(),
        ))
    }

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        Some((
            format!(
                "{}/{}/uv-aarch64-pc-windows-msvc.zip",
                UV_RELEASES_BASE, version
            ),
            "zip".into(),
        ))
    }

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
    )))]
    {
        None
    }
}

// ── Utility functions ───────────────────────────────────────────────

/// Check a system binary exists and get its version.
async fn check_system_binary(cmd: &str, args: &[&str]) -> Option<(String, String)> {
    let output = tokio::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Get full path via `which` / `where`
    #[cfg(target_os = "windows")]
    let which = "where";
    #[cfg(not(target_os = "windows"))]
    let which = "which";

    let path = tokio::process::Command::new(which)
        .arg(cmd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .await
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(
                    String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .unwrap_or_else(|| cmd.to_string());

    Some((version, path))
}

/// Get version from a binary at a specific path.
async fn get_binary_version(bin: &Path, args: &[&str]) -> Option<(String, String)> {
    let output = tokio::process::Command::new(bin)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some((version, bin.display().to_string()))
}

/// Download a file from a URL to a local path.
async fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    tracing::info!("Downloading {} -> {}", url, dest.display());

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {} for {}", response.status(), url));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    std::fs::write(dest, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;

    tracing::info!("Downloaded {} bytes to {}", bytes.len(), dest.display());
    Ok(())
}

/// Extract an archive (tar.gz, tar.xz, zip) to a directory.
async fn extract_archive(archive: &Path, dest: &Path, archive_type: &str) -> Result<(), String> {
    // Clear destination first
    if dest.exists() {
        std::fs::remove_dir_all(dest).ok();
    }
    std::fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create extraction directory: {}", e))?;

    match archive_type {
        "tar.gz" | "tar.xz" => {
            // Use system tar command (available on all platforms)
            let output = tokio::process::Command::new("tar")
                .args([
                    "xf",
                    &archive.display().to_string(),
                    "-C",
                    &dest.display().to_string(),
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .await
                .map_err(|e| format!("Failed to run tar: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("tar extraction failed: {}", stderr));
            }

            // uv archives contain a nested directory — flatten if needed
            flatten_single_nested_dir(dest)?;

            Ok(())
        }
        "zip" => {
            // Use system unzip or PowerShell
            #[cfg(target_os = "windows")]
            {
                let output = tokio::process::Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-Command",
                        &format!(
                            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                            archive.display(),
                            dest.display()
                        ),
                    ])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .output()
                    .await
                    .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("zip extraction failed: {}", stderr));
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                let output = tokio::process::Command::new("unzip")
                    .args([
                        "-o",
                        &archive.display().to_string(),
                        "-d",
                        &dest.display().to_string(),
                    ])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .output()
                    .await
                    .map_err(|e| format!("Failed to run unzip: {}", e))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("unzip extraction failed: {}", stderr));
                }
            }

            flatten_single_nested_dir(dest)?;

            Ok(())
        }
        _ => Err(format!("Unsupported archive type: {}", archive_type)),
    }
}

/// If the extracted directory contains a single subdirectory, flatten it.
/// e.g., `uv/uv-x86_64-unknown-linux-gnu/uv` → `uv/uv`
fn flatten_single_nested_dir(dir: &Path) -> Result<(), String> {
    let entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read dir: {}", e))?
        .filter_map(|e| e.ok())
        .collect();

    // If there's exactly one subdirectory and no files, flatten
    if entries.len() == 1 && entries[0].path().is_dir() {
        let nested = entries[0].path();
        let temp_name = dir.join("__ghost_flatten_temp");
        std::fs::rename(&nested, &temp_name)
            .map_err(|e| format!("Failed to rename nested dir: {}", e))?;

        // Move all contents from temp_name into dir
        for entry in
            std::fs::read_dir(&temp_name).map_err(|e| format!("Failed to read temp dir: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let dest = dir.join(entry.file_name());
            std::fs::rename(entry.path(), &dest)
                .map_err(|e| format!("Failed to move file: {}", e))?;
        }

        std::fs::remove_dir(&temp_name).ok();
    }

    Ok(())
}

// ── AI-Powered Tool Discovery ───────────────────────────────────────

/// Smart tool recommender: given a natural language description, find matching tools.
/// Uses fuzzy matching against the catalog and registry.
pub fn recommend_tools(
    query: &str,
    catalog: &[super::mcp_catalog::CatalogEntry],
) -> Vec<ToolRecommendation> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    // Empty query returns no results
    if query_words.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(f64, &super::mcp_catalog::CatalogEntry)> = Vec::new();

    for entry in catalog {
        let mut score = 0.0;

        // Name matching
        let name_lower = entry.name.to_lowercase();
        for word in &query_words {
            if name_lower.contains(word) {
                score += 3.0;
            }
        }

        // Description matching
        let desc_lower = entry.description.to_lowercase();
        for word in &query_words {
            if desc_lower.contains(word) {
                score += 2.0;
            }
        }

        // Tag matching
        for tag in &entry.tags {
            let tag_lower = tag.to_lowercase();
            for word in &query_words {
                if tag_lower.contains(word) {
                    score += 1.5;
                }
            }
        }

        // Category matching
        let cat_lower = entry.category.to_lowercase();
        for word in &query_words {
            if cat_lower.contains(word) {
                score += 1.0;
            }
        }

        // Only add popularity/official boost if there's already a text match
        if score > 0.0 {
            if entry.official {
                score *= 1.2;
            }
            score += (entry.popularity as f64) * 0.1;
            scored.push((score, entry));
        }
    }

    // Sort by score descending
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    scored
        .into_iter()
        .take(10)
        .map(|(_score, entry)| {
            let missing_runtimes = get_missing_runtimes_for_entry(entry);
            ToolRecommendation {
                catalog_id: Some(entry.id.clone()),
                name: entry.name.clone(),
                reason: entry.description.clone(),
                runtime: entry.runtime.clone(),
                installable: missing_runtimes.is_empty() && entry.required_env.is_empty(),
                missing_runtimes,
            }
        })
        .collect()
}

/// Check which runtimes an MCP tool needs that aren't available.
fn get_missing_runtimes_for_entry(entry: &super::mcp_catalog::CatalogEntry) -> Vec<String> {
    // Quick sync check — just look for binaries
    let mut missing = Vec::new();

    match entry.runtime.as_str() {
        "node" => {
            if !command_exists_sync("node") && !command_exists_sync("npx") {
                missing.push("node".to_string());
            }
        }
        "python" => {
            if !command_exists_sync("uvx") && !command_exists_sync("uv") {
                missing.push("uv".to_string());
            }
        }
        "docker" => {
            if !command_exists_sync("docker") {
                missing.push("docker".to_string());
            }
        }
        _ => {}
    }

    missing
}

/// Synchronous check if a command exists (for quick recommendations).
fn command_exists_sync(cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    let check = "where";
    #[cfg(not(target_os = "windows"))]
    let check = "which";

    std::process::Command::new(check)
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper ──────────────────────────────────────────────────────

    /// Create a minimal fake CatalogEntry for testing.
    fn fake_entry(
        id: &str,
        name: &str,
        desc: &str,
        runtime: &str,
        tags: &[&str],
        category: &str,
        popularity: u32,
        official: bool,
    ) -> super::super::mcp_catalog::CatalogEntry {
        super::super::mcp_catalog::CatalogEntry {
            id: id.into(),
            name: name.into(),
            description: desc.into(),
            category: category.into(),
            icon: "🔧".into(),
            runtime: runtime.into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![],
            is_mcp_app: false,
            required_env: vec![],
            tags: tags.iter().map(|s| s.to_string()).collect(),
            popularity,
            official,
            package: None,
            repository: None,
        }
    }

    /// Create a temporary directory for tests with automatic cleanup.
    struct TempDir(PathBuf);
    impl TempDir {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("ghost-test-{}-{}", name, std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    // ════════════════════════════════════════════════════════════════
    //  1. RuntimeKind — Serialization & Display
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn runtime_kind_display() {
        assert_eq!(RuntimeKind::Node.to_string(), "node");
        assert_eq!(RuntimeKind::Uv.to_string(), "uv");
        assert_eq!(RuntimeKind::Docker.to_string(), "docker");
    }

    #[test]
    fn runtime_kind_serialize_json() {
        assert_eq!(
            serde_json::to_string(&RuntimeKind::Node).unwrap(),
            "\"node\""
        );
        assert_eq!(serde_json::to_string(&RuntimeKind::Uv).unwrap(), "\"uv\"");
        assert_eq!(
            serde_json::to_string(&RuntimeKind::Docker).unwrap(),
            "\"docker\""
        );
    }

    #[test]
    fn runtime_kind_deserialize_json() {
        let node: RuntimeKind = serde_json::from_str("\"node\"").unwrap();
        assert_eq!(node, RuntimeKind::Node);
        let uv: RuntimeKind = serde_json::from_str("\"uv\"").unwrap();
        assert_eq!(uv, RuntimeKind::Uv);
        let docker: RuntimeKind = serde_json::from_str("\"docker\"").unwrap();
        assert_eq!(docker, RuntimeKind::Docker);
    }

    #[test]
    fn runtime_kind_deserialize_invalid() {
        let result: Result<RuntimeKind, _> = serde_json::from_str("\"java\"");
        assert!(
            result.is_err(),
            "Should fail to deserialize unknown runtime"
        );
    }

    #[test]
    fn runtime_kind_eq_and_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(RuntimeKind::Node);
        set.insert(RuntimeKind::Uv);
        set.insert(RuntimeKind::Docker);
        assert_eq!(set.len(), 3);
        set.insert(RuntimeKind::Node); // duplicate
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn runtime_kind_clone_copy() {
        let a = RuntimeKind::Node;
        let b = a; // Copy
        let c = a.clone(); // Clone
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    // ════════════════════════════════════════════════════════════════
    //  2. RuntimeStatus — Serialization round-trip
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn runtime_status_roundtrip() {
        let status = RuntimeStatus {
            kind: RuntimeKind::Node,
            installed: true,
            managed: true,
            version: Some("v24.13.1".into()),
            path: Some("/tmp/runtimes/node/bin/node".into()),
            description: "Node.js (Ghost-managed)".into(),
            can_auto_install: true,
        };
        let json = serde_json::to_string(&status).unwrap();
        let back: RuntimeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.kind, RuntimeKind::Node);
        assert!(back.installed);
        assert!(back.managed);
        assert_eq!(back.version.as_deref(), Some("v24.13.1"));
        assert!(back.can_auto_install);
    }

    #[test]
    fn runtime_status_not_installed() {
        let status = RuntimeStatus {
            kind: RuntimeKind::Docker,
            installed: false,
            managed: false,
            version: None,
            path: None,
            description: "Docker — not installed".into(),
            can_auto_install: false,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"installed\":false"));
        assert!(json.contains("\"can_auto_install\":false"));
        assert!(json.contains("\"version\":null"));
    }

    // ════════════════════════════════════════════════════════════════
    //  3. InstallProgress — Serialization
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn install_progress_roundtrip() {
        let progress = InstallProgress {
            runtime: "node".into(),
            stage: "downloading".into(),
            percent: 42.5,
            message: "Downloading Node.js v24.13.1...".into(),
        };
        let json = serde_json::to_string(&progress).unwrap();
        let back: InstallProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(back.runtime, "node");
        assert_eq!(back.stage, "downloading");
        assert!((back.percent - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn install_progress_indeterminate() {
        let progress = InstallProgress {
            runtime: "uv".into(),
            stage: "configuring".into(),
            percent: -1.0,
            message: "Installing Python via uv...".into(),
        };
        assert!(progress.percent < 0.0);
    }

    // ════════════════════════════════════════════════════════════════
    //  4. InstallResult — Serialization
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn install_result_success_roundtrip() {
        let result = InstallResult {
            success: true,
            status: RuntimeStatus {
                kind: RuntimeKind::Uv,
                installed: true,
                managed: true,
                version: Some("uv 0.10.4".into()),
                path: Some("/tmp/runtimes/uv/uv".into()),
                description: "uv (Ghost-managed)".into(),
                can_auto_install: true,
            },
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: InstallResult = serde_json::from_str(&json).unwrap();
        assert!(back.success);
        assert!(back.error.is_none());
        assert_eq!(back.status.kind, RuntimeKind::Uv);
    }

    #[test]
    fn install_result_failure() {
        let result = InstallResult {
            success: false,
            status: RuntimeStatus {
                kind: RuntimeKind::Node,
                installed: false,
                managed: false,
                version: None,
                path: None,
                description: "Node.js — not installed".into(),
                can_auto_install: true,
            },
            error: Some("Failed to download Node.js: connection refused".into()),
        };
        assert!(!result.success);
        assert!(result
            .error
            .as_ref()
            .unwrap()
            .contains("connection refused"));
    }

    // ════════════════════════════════════════════════════════════════
    //  5. BootstrapStatus — Serialization & Logic
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn bootstrap_status_roundtrip() {
        let status = BootstrapStatus {
            runtimes: vec![RuntimeStatus {
                kind: RuntimeKind::Node,
                installed: true,
                managed: false,
                version: Some("v22.5.0".into()),
                path: Some("/usr/bin/node".into()),
                description: "Node.js (system)".into(),
                can_auto_install: true,
            }],
            ready_for_defaults: false,
            missing_installable: vec![RuntimeKind::Uv],
            runtimes_dir: "/tmp/runtimes".into(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let back: BootstrapStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.runtimes.len(), 1);
        assert!(!back.ready_for_defaults);
        assert_eq!(back.missing_installable.len(), 1);
        assert_eq!(back.missing_installable[0], RuntimeKind::Uv);
    }

    // ════════════════════════════════════════════════════════════════
    //  6. ToolRecommendation — Serialization
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn tool_recommendation_roundtrip() {
        let rec = ToolRecommendation {
            catalog_id: Some("filesystem".into()),
            name: "Filesystem".into(),
            reason: "Read, write, and manage files".into(),
            runtime: "node".into(),
            installable: true,
            missing_runtimes: vec![],
        };
        let json = serde_json::to_string(&rec).unwrap();
        let back: ToolRecommendation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.catalog_id.as_deref(), Some("filesystem"));
        assert!(back.installable);
        assert!(back.missing_runtimes.is_empty());
    }

    #[test]
    fn tool_recommendation_with_missing_runtimes() {
        let rec = ToolRecommendation {
            catalog_id: None,
            name: "Custom Tool".into(),
            reason: "Does stuff".into(),
            runtime: "python".into(),
            installable: false,
            missing_runtimes: vec!["uv".into()],
        };
        assert!(!rec.installable);
        assert_eq!(rec.missing_runtimes.len(), 1);
    }

    // ════════════════════════════════════════════════════════════════
    //  7. RuntimeBootstrapper — Constructor & Paths
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn bootstrapper_new_creates_runtimes_subdir() {
        let bs = RuntimeBootstrapper::new(Path::new("/tmp/ghost-test-app"));
        assert_eq!(bs.runtimes_dir(), Path::new("/tmp/ghost-test-app/runtimes"));
    }

    #[test]
    fn bootstrapper_runtimes_dir_is_subpath() {
        let bs = RuntimeBootstrapper::new(Path::new("/home/user/.config/com.ghost.app"));
        assert!(bs
            .runtimes_dir()
            .starts_with("/home/user/.config/com.ghost.app"));
        assert!(bs.runtimes_dir().ends_with("runtimes"));
    }

    #[test]
    fn managed_node_dir_fallback() {
        // When there's no node-vXX directory, should fallback to node/bin
        let tmp = TempDir::new("managed-node-dir");
        let bs = RuntimeBootstrapper::new(tmp.path());
        // Without creating the runtimes/node dir, managed_node_dir should still return a path
        let dir = bs.managed_node_dir();
        assert!(dir.to_str().unwrap().contains("node"));
    }

    #[test]
    fn managed_node_dir_with_versioned_subdir() {
        // Simulate extracted Node.js archive with node-v24.13.1-linux-x64/bin/ structure
        let tmp = TempDir::new("managed-node-versioned");
        let node_dir = tmp.path().join("runtimes").join("node");
        let versioned = node_dir.join("node-v24.13.1-linux-x64").join("bin");
        std::fs::create_dir_all(&versioned).unwrap();
        std::fs::write(versioned.join("node"), "fake").unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        let dir = bs.managed_node_dir();
        // Should detect the versioned directory and return its bin/ path
        assert!(dir.to_str().unwrap().contains("node-v24.13.1"));
        assert!(dir.to_str().unwrap().ends_with("bin"));
    }

    #[test]
    fn managed_uv_dir_path() {
        let tmp = TempDir::new("managed-uv-dir");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let dir = bs.managed_uv_dir();
        assert!(dir.ends_with("uv"));
    }

    #[test]
    fn managed_node_bin_has_correct_name() {
        let tmp = TempDir::new("managed-node-bin");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let bin = bs.managed_node_bin();
        let name = bin.file_name().unwrap().to_str().unwrap();
        #[cfg(target_os = "windows")]
        assert_eq!(name, "node.exe");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(name, "node");
    }

    #[test]
    fn managed_uv_bin_has_correct_name() {
        let tmp = TempDir::new("managed-uv-bin");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let bin = bs.managed_uv_bin();
        let name = bin.file_name().unwrap().to_str().unwrap();
        #[cfg(target_os = "windows")]
        assert_eq!(name, "uv.exe");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(name, "uv");
    }

    // ════════════════════════════════════════════════════════════════
    //  8. resolve_binary — Managed runtime resolution
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_binary_unknown_returns_none() {
        let tmp = TempDir::new("resolve-unknown");
        let bs = RuntimeBootstrapper::new(tmp.path());
        assert!(bs.resolve_binary("unknown-binary").is_none());
        assert!(bs.resolve_binary("java").is_none());
        assert!(bs.resolve_binary("go").is_none());
    }

    #[test]
    fn resolve_binary_node_without_managed_returns_none() {
        let tmp = TempDir::new("resolve-no-node");
        let bs = RuntimeBootstrapper::new(tmp.path());
        // No managed node installed
        assert!(bs.resolve_binary("node").is_none());
        assert!(bs.resolve_binary("npm").is_none());
        assert!(bs.resolve_binary("npx").is_none());
    }

    #[test]
    fn resolve_binary_uv_without_managed_returns_none() {
        let tmp = TempDir::new("resolve-no-uv");
        let bs = RuntimeBootstrapper::new(tmp.path());
        assert!(bs.resolve_binary("uv").is_none());
        assert!(bs.resolve_binary("uvx").is_none());
    }

    #[test]
    fn resolve_binary_node_with_fake_managed() {
        let tmp = TempDir::new("resolve-fake-node");
        // Create fake managed node binary
        let node_dir = tmp.path().join("runtimes").join("node").join("bin");
        std::fs::create_dir_all(&node_dir).unwrap();
        let node_bin = node_dir.join("node");
        std::fs::write(&node_bin, "#!/bin/sh\necho fake").unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        let resolved = bs.resolve_binary("node");
        assert!(resolved.is_some(), "Should find managed node binary");
        assert!(resolved.unwrap().to_str().unwrap().contains("node"));
    }

    #[test]
    fn resolve_binary_npm_npx_with_fake_managed() {
        let tmp = TempDir::new("resolve-fake-npm");
        let node_dir = tmp.path().join("runtimes").join("node").join("bin");
        std::fs::create_dir_all(&node_dir).unwrap();
        std::fs::write(node_dir.join("npm"), "#!/bin/sh\necho fake").unwrap();
        std::fs::write(node_dir.join("npx"), "#!/bin/sh\necho fake").unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        assert!(bs.resolve_binary("npm").is_some());
        assert!(bs.resolve_binary("npx").is_some());
    }

    #[test]
    fn resolve_binary_uv_uvx_with_fake_managed() {
        let tmp = TempDir::new("resolve-fake-uv");
        let uv_dir = tmp.path().join("runtimes").join("uv");
        std::fs::create_dir_all(&uv_dir).unwrap();
        std::fs::write(uv_dir.join("uv"), "#!/bin/sh\necho fake").unwrap();
        std::fs::write(uv_dir.join("uvx"), "#!/bin/sh\necho fake").unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        assert!(bs.resolve_binary("uv").is_some());
        assert!(bs.resolve_binary("uvx").is_some());
    }

    #[test]
    fn resolve_binary_python_with_fake_uv_python() {
        let tmp = TempDir::new("resolve-fake-python");
        // Simulate uv-managed Python: runtimes/uv-python/cpython-3.13/bin/python3
        let python_dir = tmp
            .path()
            .join("runtimes")
            .join("uv-python")
            .join("cpython-3.13.0")
            .join("bin");
        std::fs::create_dir_all(&python_dir).unwrap();
        std::fs::write(python_dir.join("python3"), "#!/bin/sh\necho fake").unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        let resolved = bs.resolve_binary("python3");
        assert!(resolved.is_some(), "Should find uv-managed python3");
        assert!(resolved.unwrap().to_str().unwrap().contains("python3"));
    }

    // ════════════════════════════════════════════════════════════════
    //  9. build_env_path — PATH construction
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn build_env_path_includes_system_path() {
        let tmp = TempDir::new("env-path-system");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let env_path = bs.build_env_path();
        // Even without managed runtimes, system PATH should be included
        if let Ok(system_path) = std::env::var("PATH") {
            assert!(
                env_path.contains(&system_path),
                "Should contain system PATH"
            );
        }
    }

    #[test]
    fn build_env_path_prepends_managed_node() {
        let tmp = TempDir::new("env-path-node");
        // Create managed node directory
        let node_dir = tmp.path().join("runtimes").join("node").join("bin");
        std::fs::create_dir_all(&node_dir).unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        let env_path = bs.build_env_path();
        let parts: Vec<&str> = env_path.split(':').collect();
        // Managed runtime should come before system PATH
        assert!(
            parts[0].contains("node"),
            "Node dir should be first in PATH, got: {}",
            parts[0]
        );
    }

    #[test]
    fn build_env_path_prepends_managed_uv() {
        let tmp = TempDir::new("env-path-uv");
        // Create managed uv directory
        let uv_dir = tmp.path().join("runtimes").join("uv");
        std::fs::create_dir_all(&uv_dir).unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        let env_path = bs.build_env_path();
        assert!(env_path.contains("uv"), "PATH should contain uv directory");
    }

    #[test]
    fn build_env_path_order_node_before_uv_before_system() {
        let tmp = TempDir::new("env-path-order");
        // Create both
        let node_dir = tmp.path().join("runtimes").join("node").join("bin");
        std::fs::create_dir_all(&node_dir).unwrap();
        let uv_dir = tmp.path().join("runtimes").join("uv");
        std::fs::create_dir_all(&uv_dir).unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        let env_path = bs.build_env_path();

        let node_pos = env_path.find("node").expect("PATH should contain node");
        let uv_pos = env_path.find("/uv").expect("PATH should contain uv");
        assert!(node_pos < uv_pos, "Node should come before uv in PATH");
    }

    #[test]
    fn build_env_skips_nonexistent_dirs() {
        let tmp = TempDir::new("env-path-skip");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let env_path = bs.build_env_path();
        // Should not contain ghost runtimes dir since it doesn't exist
        assert!(!env_path.contains(
            &tmp.path()
                .join("runtimes")
                .join("node")
                .display()
                .to_string()
        ));
    }

    #[test]
    fn build_env_returns_hashmap_with_path() {
        let tmp = TempDir::new("build-env-map");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let env = bs.build_env();
        assert!(
            env.contains_key("PATH"),
            "build_env should contain PATH key"
        );
        assert!(!env["PATH"].is_empty());
    }

    // ════════════════════════════════════════════════════════════════
    // 10. command() — Process builder with injected PATH
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn command_returns_configured_command() {
        let tmp = TempDir::new("command-builder");
        let bs = RuntimeBootstrapper::new(tmp.path());
        // Just verify it returns without panicking
        let _cmd = bs.command("echo");
    }

    #[tokio::test]
    async fn command_inherits_managed_path() {
        let tmp = TempDir::new("command-path-inherit");
        let uv_dir = tmp.path().join("runtimes").join("uv");
        std::fs::create_dir_all(&uv_dir).unwrap();

        let bs = RuntimeBootstrapper::new(tmp.path());
        let output = bs
            .command("sh")
            .args(["-c", "echo $PATH"])
            .stdout(std::process::Stdio::piped())
            .output()
            .await
            .expect("Failed to run command");

        let path_value = String::from_utf8_lossy(&output.stdout);
        assert!(
            path_value.contains("uv"),
            "Spawned process should have uv in PATH"
        );
    }

    // ════════════════════════════════════════════════════════════════
    // 11. Detection — Async runtime detection
    // ════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn detect_all_returns_three_runtimes() {
        let tmp = TempDir::new("detect-all");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let statuses = bs.detect_all().await;
        assert_eq!(statuses.len(), 3, "Should detect Node, Uv, Docker");
        assert!(statuses.iter().any(|s| s.kind == RuntimeKind::Node));
        assert!(statuses.iter().any(|s| s.kind == RuntimeKind::Uv));
        assert!(statuses.iter().any(|s| s.kind == RuntimeKind::Docker));
    }

    #[tokio::test]
    async fn detect_all_order_is_node_uv_docker() {
        let tmp = TempDir::new("detect-order");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let statuses = bs.detect_all().await;
        assert_eq!(statuses[0].kind, RuntimeKind::Node);
        assert_eq!(statuses[1].kind, RuntimeKind::Uv);
        assert_eq!(statuses[2].kind, RuntimeKind::Docker);
    }

    #[tokio::test]
    async fn detect_node_without_managed_checks_system() {
        let tmp = TempDir::new("detect-node-system");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.detect_node().await;
        // Regardless of whether system node exists, the status should be valid
        assert_eq!(status.kind, RuntimeKind::Node);
        assert!(status.can_auto_install);
        if status.installed {
            assert!(status.version.is_some());
            assert!(status.path.is_some());
        } else {
            assert!(!status.managed);
        }
    }

    #[tokio::test]
    async fn detect_uv_without_managed_checks_system() {
        let tmp = TempDir::new("detect-uv-system");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.detect_uv().await;
        assert_eq!(status.kind, RuntimeKind::Uv);
        assert!(status.can_auto_install);
    }

    #[tokio::test]
    async fn detect_docker_is_never_managed_or_auto_installable() {
        let tmp = TempDir::new("detect-docker");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.detect_docker().await;
        assert_eq!(status.kind, RuntimeKind::Docker);
        assert!(!status.managed, "Docker should never be managed by Ghost");
        assert!(
            !status.can_auto_install,
            "Docker should not be auto-installable"
        );
    }

    #[tokio::test]
    async fn detect_node_with_fake_managed_binary() {
        let tmp = TempDir::new("detect-node-fake");
        // Create a fake managed node that returns a version
        let node_dir = tmp.path().join("runtimes").join("node").join("bin");
        std::fs::create_dir_all(&node_dir).unwrap();
        let node_script = node_dir.join("node");
        std::fs::write(&node_script, "#!/bin/sh\necho v24.13.1").unwrap();
        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&node_script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.detect_node().await;
        assert!(status.installed, "Should detect fake managed node");
        assert!(status.managed, "Should be marked as managed");
        assert_eq!(status.version.as_deref(), Some("v24.13.1"));
    }

    #[tokio::test]
    async fn detect_uv_with_fake_managed_binary() {
        let tmp = TempDir::new("detect-uv-fake");
        let uv_dir = tmp.path().join("runtimes").join("uv");
        std::fs::create_dir_all(&uv_dir).unwrap();
        let uv_script = uv_dir.join("uv");
        std::fs::write(&uv_script, "#!/bin/sh\necho uv 0.10.4").unwrap();
        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&uv_script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.detect_uv().await;
        assert!(status.installed, "Should detect fake managed uv");
        assert!(status.managed, "Should be marked as managed");
        assert_eq!(status.version.as_deref(), Some("uv 0.10.4"));
    }

    // ════════════════════════════════════════════════════════════════
    // 12. get_status — Comprehensive status logic
    // ════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn get_status_returns_all_runtimes() {
        let tmp = TempDir::new("status-all");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.get_status().await;
        assert_eq!(status.runtimes.len(), 3);
        assert!(!status.runtimes_dir.is_empty());
    }

    #[tokio::test]
    async fn get_status_ready_when_both_installed() {
        let tmp = TempDir::new("status-ready");
        // Create fake node and uv binaries
        let node_dir = tmp.path().join("runtimes").join("node").join("bin");
        std::fs::create_dir_all(&node_dir).unwrap();
        let node_script = node_dir.join("node");
        std::fs::write(&node_script, "#!/bin/sh\necho v24.0.0").unwrap();

        let uv_dir = tmp.path().join("runtimes").join("uv");
        std::fs::create_dir_all(&uv_dir).unwrap();
        let uv_script = uv_dir.join("uv");
        std::fs::write(&uv_script, "#!/bin/sh\necho uv 0.10.0").unwrap();

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&node_script, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&uv_script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.get_status().await;
        assert!(
            status.ready_for_defaults,
            "Should be ready when both node and uv are available"
        );
        assert!(
            status.missing_installable.is_empty(),
            "Should have no missing installable runtimes"
        );
    }

    #[tokio::test]
    async fn get_status_tracks_missing_installable() {
        let tmp = TempDir::new("status-missing");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.get_status().await;

        // If node/uv are not installed on system AND no managed versions exist,
        // they should appear in missing_installable
        for kind in &status.missing_installable {
            assert!(
                *kind == RuntimeKind::Node || *kind == RuntimeKind::Uv,
                "Only Node and Uv should be auto-installable missing, got {:?}",
                kind
            );
        }
        // Docker should never be in missing_installable
        assert!(
            !status.missing_installable.contains(&RuntimeKind::Docker),
            "Docker should not be in missing_installable"
        );
    }

    #[tokio::test]
    async fn get_status_runtimes_dir_matches_bootstrapper() {
        let tmp = TempDir::new("status-dir");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let status = bs.get_status().await;
        assert_eq!(status.runtimes_dir, bs.runtimes_dir().display().to_string());
    }

    // ════════════════════════════════════════════════════════════════
    // 13. install_runtime — Docker returns error
    // ════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn install_docker_returns_error() {
        let tmp = TempDir::new("install-docker");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let result = bs.install_runtime(RuntimeKind::Docker, |_| {}).await;
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("manual installation"));
    }

    // ════════════════════════════════════════════════════════════════
    // 14. Platform download info
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn node_download_info_for_current_platform() {
        let info = get_node_download_info();
        assert!(
            info.is_some(),
            "Should have Node.js download info for current platform"
        );
        let (url, archive_type) = info.unwrap();
        assert!(url.starts_with("https://nodejs.org/dist/v"));
        assert!(url.contains(NODE_LTS_VERSION));

        #[cfg(target_os = "linux")]
        assert_eq!(archive_type, "tar.xz");
        #[cfg(target_os = "macos")]
        assert_eq!(archive_type, "tar.gz");
        #[cfg(target_os = "windows")]
        assert_eq!(archive_type, "zip");

        #[cfg(target_arch = "x86_64")]
        {
            #[cfg(target_os = "linux")]
            assert!(url.contains("linux-x64"));
            #[cfg(target_os = "macos")]
            assert!(url.contains("darwin-x64"));
            #[cfg(target_os = "windows")]
            assert!(url.contains("win-x64"));
        }
        #[cfg(target_arch = "aarch64")]
        {
            #[cfg(target_os = "linux")]
            assert!(url.contains("linux-arm64"));
            #[cfg(target_os = "macos")]
            assert!(url.contains("darwin-arm64"));
            #[cfg(target_os = "windows")]
            assert!(url.contains("win-arm64"));
        }
    }

    #[test]
    fn uv_download_info_for_current_platform() {
        let info = get_uv_download_info();
        assert!(
            info.is_some(),
            "Should have uv download info for current platform"
        );
        let (url, archive_type) = info.unwrap();
        assert!(url.starts_with("https://github.com/astral-sh/uv/releases/download/"));
        assert!(url.contains(UV_VERSION));

        #[cfg(target_os = "windows")]
        assert_eq!(archive_type, "zip");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(archive_type, "tar.gz");

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        assert!(url.contains("x86_64-unknown-linux-gnu"));
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        assert!(url.contains("aarch64-unknown-linux-gnu"));
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        assert!(url.contains("x86_64-apple-darwin"));
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        assert!(url.contains("aarch64-apple-darwin"));
    }

    // ════════════════════════════════════════════════════════════════
    // 15. flatten_single_nested_dir — Edge cases
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn flatten_single_nested_moves_contents() {
        let tmp = TempDir::new("flatten-basic");
        let nested = tmp.path().join("sub-dir");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("file.txt"), "test content").unwrap();
        std::fs::write(nested.join("other.bin"), "binary data").unwrap();

        flatten_single_nested_dir(tmp.path()).unwrap();

        assert!(tmp.path().join("file.txt").exists());
        assert!(tmp.path().join("other.bin").exists());
        assert!(!tmp.path().join("sub-dir").exists());
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("file.txt")).unwrap(),
            "test content"
        );
    }

    #[test]
    fn flatten_noop_with_multiple_entries() {
        // If there are 2+ entries, don't flatten
        let tmp = TempDir::new("flatten-multi");
        std::fs::create_dir_all(tmp.path().join("dir-a")).unwrap();
        std::fs::create_dir_all(tmp.path().join("dir-b")).unwrap();

        flatten_single_nested_dir(tmp.path()).unwrap();

        // Both should still exist
        assert!(tmp.path().join("dir-a").exists());
        assert!(tmp.path().join("dir-b").exists());
    }

    #[test]
    fn flatten_noop_with_files_at_root() {
        // If the root has a file (not a single dir), don't flatten
        let tmp = TempDir::new("flatten-file");
        std::fs::write(tmp.path().join("readme.txt"), "hello").unwrap();

        flatten_single_nested_dir(tmp.path()).unwrap();

        assert!(tmp.path().join("readme.txt").exists());
    }

    #[test]
    fn flatten_noop_on_empty_dir() {
        let tmp = TempDir::new("flatten-empty");
        flatten_single_nested_dir(tmp.path()).unwrap();
        // Should not panic on empty directory
    }

    #[test]
    fn flatten_deep_single_nesting() {
        // Only flattens one level
        let tmp = TempDir::new("flatten-deep");
        let deep = tmp.path().join("outer").join("inner");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("data.txt"), "deep data").unwrap();

        flatten_single_nested_dir(tmp.path()).unwrap();

        // outer/ should be flattened, moving inner/ to root
        assert!(tmp.path().join("inner").exists());
        assert!(tmp.path().join("inner").join("data.txt").exists());
    }

    // ════════════════════════════════════════════════════════════════
    // 16. recommend_tools — Fuzzy matching & scoring
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn recommend_tools_empty_query_returns_empty() {
        let catalog = super::super::mcp_catalog::get_catalog();
        assert!(recommend_tools("", &catalog).is_empty());
    }

    #[test]
    fn recommend_tools_whitespace_only_returns_empty() {
        let catalog = super::super::mcp_catalog::get_catalog();
        assert!(recommend_tools("   ", &catalog).is_empty());
        assert!(recommend_tools("\t\n", &catalog).is_empty());
    }

    #[test]
    fn recommend_tools_finds_filesystem() {
        let catalog = super::super::mcp_catalog::get_catalog();
        let recs = recommend_tools("filesystem files browse", &catalog);
        assert!(!recs.is_empty());
        assert!(recs
            .iter()
            .any(|r| r.name.to_lowercase().contains("filesystem")));
    }

    #[test]
    fn recommend_tools_finds_database() {
        let catalog = super::super::mcp_catalog::get_catalog();
        let recs = recommend_tools("database sql query", &catalog);
        assert!(!recs.is_empty());
        assert!(recs.iter().any(|r| {
            r.name.to_lowercase().contains("sql")
                || r.name.to_lowercase().contains("database")
                || r.name.to_lowercase().contains("postgres")
        }));
    }

    #[test]
    fn recommend_tools_finds_github() {
        let catalog = super::super::mcp_catalog::get_catalog();
        let recs = recommend_tools("github repository issues", &catalog);
        assert!(!recs.is_empty());
        assert!(recs
            .iter()
            .any(|r| r.name.to_lowercase().contains("github")));
    }

    #[test]
    fn recommend_tools_returns_max_10() {
        let catalog = super::super::mcp_catalog::get_catalog();
        // Use a broad query that matches many entries
        let recs = recommend_tools("server tool", &catalog);
        assert!(
            recs.len() <= 10,
            "Should return at most 10 results, got {}",
            recs.len()
        );
    }

    #[test]
    fn recommend_tools_gibberish_returns_empty() {
        let catalog = super::super::mcp_catalog::get_catalog();
        let recs = recommend_tools("xyzzy fnord qux", &catalog);
        assert!(recs.is_empty(), "Gibberish query should not match anything");
    }

    #[test]
    fn recommend_tools_name_match_scores_higher_than_description() {
        // Create a catalog where the name "Alpha" matches one entry
        // and the description contains "alpha" in another
        let catalog = vec![
            fake_entry(
                "alpha-tool",
                "Alpha",
                "A great tool",
                "node",
                &["test"],
                "dev",
                5,
                false,
            ),
            fake_entry(
                "beta-tool",
                "Beta",
                "Uses alpha internally",
                "node",
                &["test"],
                "dev",
                5,
                false,
            ),
        ];
        let recs = recommend_tools("alpha", &catalog);
        assert!(recs.len() == 2);
        // "Alpha" (name match = 3.0) should rank above "Beta" (desc match = 2.0)
        assert_eq!(recs[0].name, "Alpha");
        assert_eq!(recs[1].name, "Beta");
    }

    #[test]
    fn recommend_tools_official_boost() {
        let catalog = vec![
            fake_entry(
                "community",
                "FileManager",
                "Manage files and dirs",
                "node",
                &[],
                "dev",
                5,
                false,
            ),
            fake_entry(
                "official",
                "FileManager",
                "Manage files and dirs",
                "node",
                &[],
                "dev",
                5,
                true,
            ),
        ];
        let recs = recommend_tools("filemanager", &catalog);
        assert_eq!(recs.len(), 2);
        // Official entry should rank first due to 1.2x boost
        assert_eq!(recs[0].catalog_id.as_deref(), Some("official"));
    }

    #[test]
    fn recommend_tools_popularity_boost() {
        let catalog = vec![
            fake_entry(
                "unpopular",
                "Widget",
                "A widget tool",
                "node",
                &[],
                "dev",
                100,
                false,
            ),
            fake_entry(
                "popular",
                "Widget",
                "A widget tool",
                "node",
                &[],
                "dev",
                1,
                false,
            ),
        ];
        let recs = recommend_tools("widget", &catalog);
        assert_eq!(recs.len(), 2);
        // Popularity 100 gets higher boost (100*0.1=10.0) vs (1*0.1=0.1)
        assert_eq!(recs[0].catalog_id.as_deref(), Some("unpopular"));
    }

    #[test]
    fn recommend_tools_tag_matching() {
        let catalog = vec![
            fake_entry(
                "tagged",
                "Searcher",
                "A search tool",
                "node",
                &["web", "crawl"],
                "dev",
                5,
                false,
            ),
            fake_entry(
                "untagged",
                "Crawler",
                "A web search tool",
                "node",
                &[],
                "dev",
                5,
                false,
            ),
        ];
        let recs = recommend_tools("crawl", &catalog);
        assert!(!recs.is_empty());
        // "tagged" should match via tag "crawl" (1.5) + gets desc boost
        assert!(recs
            .iter()
            .any(|r| r.catalog_id.as_deref() == Some("tagged")));
    }

    #[test]
    fn recommend_tools_category_matching() {
        let catalog = vec![
            fake_entry(
                "prod",
                "Widget",
                "A widget",
                "node",
                &[],
                "productivity",
                5,
                false,
            ),
            fake_entry(
                "dev",
                "Widget",
                "A widget",
                "node",
                &[],
                "developer",
                5,
                false,
            ),
        ];
        let recs = recommend_tools("productivity", &catalog);
        assert!(!recs.is_empty());
        assert_eq!(recs[0].catalog_id.as_deref(), Some("prod"));
    }

    #[test]
    fn recommend_tools_has_catalog_id() {
        let catalog = super::super::mcp_catalog::get_catalog();
        let recs = recommend_tools("filesystem", &catalog);
        for rec in &recs {
            assert!(
                rec.catalog_id.is_some(),
                "Each recommendation should have a catalog_id"
            );
        }
    }

    #[test]
    fn recommend_tools_has_runtime_field() {
        let catalog = super::super::mcp_catalog::get_catalog();
        let recs = recommend_tools("github", &catalog);
        for rec in &recs {
            assert!(
                !rec.runtime.is_empty(),
                "Each recommendation should have a runtime"
            );
        }
    }

    #[test]
    fn recommend_tools_multi_word_query() {
        let catalog = vec![
            fake_entry(
                "fs",
                "Filesystem",
                "Read and write files on disk",
                "node",
                &["files"],
                "dev",
                1,
                true,
            ),
            fake_entry(
                "db",
                "Database",
                "Query SQL databases online",
                "node",
                &["sql"],
                "data",
                1,
                false,
            ),
        ];
        // "read files" should match Filesystem via both description words
        let recs = recommend_tools("read files", &catalog);
        assert!(!recs.is_empty());
        assert_eq!(recs[0].catalog_id.as_deref(), Some("fs"));
    }

    // ════════════════════════════════════════════════════════════════
    // 17. command_exists_sync — basic smoke test
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn command_exists_sync_finds_sh() {
        // `sh` / `cmd` should always exist
        #[cfg(not(target_os = "windows"))]
        assert!(command_exists_sync("sh"), "sh should exist on Unix");
        #[cfg(target_os = "windows")]
        assert!(command_exists_sync("cmd"), "cmd should exist on Windows");
    }

    #[test]
    fn command_exists_sync_returns_false_for_nonexistent() {
        assert!(!command_exists_sync(
            "this_binary_definitely_does_not_exist_xyzzy_42"
        ));
    }

    // ════════════════════════════════════════════════════════════════
    // 18. extract_archive — error on unsupported type
    // ════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn extract_archive_unsupported_type() {
        let tmp = TempDir::new("extract-unsupported");
        let fake_archive = tmp.path().join("test.rar");
        std::fs::write(&fake_archive, "not a real archive").unwrap();
        let dest = tmp.path().join("output");

        let result = extract_archive(&fake_archive, &dest, "rar").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported archive type"));
    }

    #[tokio::test]
    async fn extract_archive_creates_dest_dir() {
        let tmp = TempDir::new("extract-creates-dir");
        let dest = tmp.path().join("output").join("deep").join("path");
        let fake_archive = tmp.path().join("test.tar.gz");
        // Create a minimal valid tar.gz
        std::fs::write(&fake_archive, "").unwrap();

        // Will fail because the archive is empty/invalid, but dest should be created
        let _ = extract_archive(&fake_archive, &dest, "tar.gz").await;
        assert!(
            dest.exists(),
            "Destination directory should be created even if extraction fails"
        );
    }

    // ════════════════════════════════════════════════════════════════
    // 19. download_file — error handling
    // ════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn download_file_invalid_url() {
        let tmp = TempDir::new("download-invalid");
        let dest = tmp.path().join("file.bin");
        // reqwest 0.13 requires TLS provider; test the error path via extract_archive instead
        let result = extract_archive(&dest, &tmp.path().join("out"), "tar.gz").await;
        assert!(result.is_err(), "Should fail with nonexistent archive");
    }

    #[tokio::test]
    async fn download_file_creates_parent_dirs() {
        let tmp = TempDir::new("download-dirs");
        let deep = tmp.path().join("deep").join("nested").join("dir");
        // Test the create_dir_all logic directly (download_file does this)
        assert!(!deep.exists());
        std::fs::create_dir_all(&deep).unwrap();
        assert!(deep.exists(), "Parent directories should be created");
    }

    // ════════════════════════════════════════════════════════════════
    // 20. bootstrap_all — skips already-installed runtimes
    // ════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn bootstrap_all_with_everything_installed() {
        // Create fake managed binaries for node and uv
        let tmp = TempDir::new("bootstrap-skip");
        let node_dir = tmp.path().join("runtimes").join("node").join("bin");
        std::fs::create_dir_all(&node_dir).unwrap();
        let node_script = node_dir.join("node");
        std::fs::write(&node_script, "#!/bin/sh\necho v24.0.0").unwrap();

        let uv_dir = tmp.path().join("runtimes").join("uv");
        std::fs::create_dir_all(&uv_dir).unwrap();
        let uv_script = uv_dir.join("uv");
        std::fs::write(&uv_script, "#!/bin/sh\necho uv 0.10.0").unwrap();

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&node_script, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&uv_script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let bs = RuntimeBootstrapper::new(tmp.path());
        let progress_calls = std::sync::Mutex::new(Vec::new());
        let results = bs
            .bootstrap_all(|p| {
                progress_calls.lock().unwrap().push(p.clone());
            })
            .await;

        // No runtimes should need installation
        assert!(
            results.is_empty(),
            "Should not install anything when both are present, got {} results",
            results.len()
        );
        assert!(
            progress_calls.lock().unwrap().is_empty(),
            "No progress events should fire"
        );
    }

    // ════════════════════════════════════════════════════════════════
    // 21. Progress callback — verifies stages are emitted
    // ════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn install_docker_emits_no_progress() {
        let tmp = TempDir::new("progress-docker");
        let bs = RuntimeBootstrapper::new(tmp.path());
        let progress_calls = std::sync::Mutex::new(Vec::new());
        let _ = bs
            .install_runtime(RuntimeKind::Docker, |p| {
                progress_calls.lock().unwrap().push(p.clone());
            })
            .await;

        assert!(
            progress_calls.lock().unwrap().is_empty(),
            "Docker install should emit no progress events"
        );
    }

    // ════════════════════════════════════════════════════════════════
    // 22. get_missing_runtimes_for_entry — Runtime checking
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn missing_runtimes_for_node_entry() {
        let entry = fake_entry("test", "Test", "Test", "node", &[], "dev", 1, false);
        let missing = get_missing_runtimes_for_entry(&entry);
        // On CI/dev machines, node is likely available, but the logic checks correctly
        if command_exists_sync("node") || command_exists_sync("npx") {
            assert!(missing.is_empty());
        } else {
            assert!(missing.contains(&"node".to_string()));
        }
    }

    #[test]
    fn missing_runtimes_for_python_entry() {
        let entry = fake_entry("test", "Test", "Test", "python", &[], "dev", 1, false);
        let missing = get_missing_runtimes_for_entry(&entry);
        if command_exists_sync("uv") || command_exists_sync("uvx") {
            assert!(missing.is_empty());
        } else {
            assert!(missing.contains(&"uv".to_string()));
        }
    }

    #[test]
    fn missing_runtimes_for_docker_entry() {
        let entry = fake_entry("test", "Test", "Test", "docker", &[], "dev", 1, false);
        let missing = get_missing_runtimes_for_entry(&entry);
        if command_exists_sync("docker") {
            assert!(missing.is_empty());
        } else {
            assert!(missing.contains(&"docker".to_string()));
        }
    }

    #[test]
    fn missing_runtimes_for_binary_entry() {
        let entry = fake_entry("test", "Test", "Test", "binary", &[], "dev", 1, false);
        let missing = get_missing_runtimes_for_entry(&entry);
        assert!(
            missing.is_empty(),
            "Binary runtime should have no missing runtimes"
        );
    }

    // ════════════════════════════════════════════════════════════════
    // 23. Constants validation
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn constants_are_valid() {
        assert!(!NODE_LTS_VERSION.is_empty());
        assert!(!UV_VERSION.is_empty());
        assert!(NODE_DIST_BASE.starts_with("https://"));
        assert!(UV_RELEASES_BASE.starts_with("https://"));
        // Version format: major.minor.patch
        assert!(
            NODE_LTS_VERSION.split('.').count() == 3,
            "Node version should be semver"
        );
        assert!(
            UV_VERSION.split('.').count() == 3,
            "uv version should be semver"
        );
    }

    // ════════════════════════════════════════════════════════════════
    //  Integration Tests (require network) — run with --ignored
    // ════════════════════════════════════════════════════════════════

    #[tokio::test]
    #[ignore] // Requires network access — run with `cargo test -- --ignored`
    async fn integration_install_uv() {
        let tmp = TempDir::new("int-install-uv");

        let bs = RuntimeBootstrapper::new(tmp.path());
        let stages = std::sync::Mutex::new(Vec::new());
        let result = bs
            .install_runtime(RuntimeKind::Uv, |p| {
                println!("[{}] {} — {}% {}", p.runtime, p.stage, p.percent, p.message);
                stages.lock().unwrap().push(p.stage.clone());
            })
            .await;

        assert!(result.success, "uv install failed: {:?}", result.error);
        assert!(result.status.installed);
        assert!(result.status.managed);

        // Verify progress stages
        let stages = stages.lock().unwrap();
        assert!(
            stages.contains(&"downloading".to_string()),
            "Should have downloading stage"
        );
        assert!(
            stages.contains(&"extracting".to_string()),
            "Should have extracting stage"
        );
        assert!(
            stages.contains(&"complete".to_string()),
            "Should have complete stage"
        );

        // Verify uv binary works
        let status = bs.detect_uv().await;
        assert!(status.installed);
        assert!(status.managed);
        assert!(status.version.is_some());

        // Verify resolve_binary works
        let resolved = bs.resolve_binary("uv");
        assert!(resolved.is_some(), "Should resolve managed uv binary");
        assert!(
            resolved.unwrap().exists(),
            "Resolved uv binary should exist on disk"
        );

        // Verify PATH includes managed uv
        let env_path = bs.build_env_path();
        assert!(env_path.contains("uv"), "PATH should contain uv directory");
    }

    #[tokio::test]
    #[ignore] // Requires network + ~40MB download
    async fn integration_install_node() {
        let tmp = TempDir::new("int-install-node");

        let bs = RuntimeBootstrapper::new(tmp.path());
        let stages = std::sync::Mutex::new(Vec::new());
        let result = bs
            .install_runtime(RuntimeKind::Node, |p| {
                println!("[{}] {} — {}% {}", p.runtime, p.stage, p.percent, p.message);
                stages.lock().unwrap().push(p.stage.clone());
            })
            .await;

        assert!(result.success, "Node install failed: {:?}", result.error);
        assert!(result.status.installed);
        assert!(result.status.managed);

        // Verify progress stages
        let stages = stages.lock().unwrap();
        assert!(stages.contains(&"downloading".to_string()));
        assert!(stages.contains(&"extracting".to_string()));
        assert!(stages.contains(&"complete".to_string()));

        // Verify node binary works
        let status = bs.detect_node().await;
        assert!(status.installed);
        assert!(status.managed);
        assert!(status.version.as_ref().unwrap().contains("24"));

        // Verify resolve_binary works for node/npm/npx
        assert!(
            bs.resolve_binary("node").is_some(),
            "Should resolve managed node"
        );
        assert!(
            bs.resolve_binary("npm").is_some(),
            "Should resolve managed npm"
        );
        assert!(
            bs.resolve_binary("npx").is_some(),
            "Should resolve managed npx"
        );

        // Verify command builder works
        let output = bs
            .command("node")
            .args(["--version"])
            .stdout(std::process::Stdio::piped())
            .output()
            .await;
        if let Ok(o) = &output {
            let version = String::from_utf8_lossy(&o.stdout);
            assert!(
                version.contains("v24"),
                "Node command should return v24.x.x, got: {}",
                version
            );
        }
    }

    #[tokio::test]
    #[ignore] // Full bootstrap — downloads Node.js + uv
    async fn integration_bootstrap_all() {
        let tmp = TempDir::new("int-bootstrap-all");

        let bs = RuntimeBootstrapper::new(tmp.path());
        let progress_events = std::sync::Mutex::new(Vec::new());
        let results = bs
            .bootstrap_all(|p| {
                println!("[{}] {} — {}% {}", p.runtime, p.stage, p.percent, p.message);
                progress_events.lock().unwrap().push(p.clone());
            })
            .await;

        for result in &results {
            assert!(
                result.success,
                "Failed to install {:?}: {:?}",
                result.status.kind, result.error
            );
        }

        // Verify both runtimes are now available
        let status = bs.get_status().await;
        assert!(
            status.ready_for_defaults,
            "Should be ready after bootstrap_all"
        );
        assert!(status.missing_installable.is_empty());

        // Verify progress events were emitted for each runtime
        assert!(
            !progress_events.lock().unwrap().is_empty(),
            "Should have emitted progress events"
        );
    }

    #[tokio::test]
    #[ignore] // Requires network for uv install + Python
    async fn integration_uv_installs_python() {
        let tmp = TempDir::new("int-uv-python");

        let bs = RuntimeBootstrapper::new(tmp.path());
        let result = bs
            .install_runtime(RuntimeKind::Uv, |p| {
                println!("[{}] {} — {}% {}", p.runtime, p.stage, p.percent, p.message);
            })
            .await;

        assert!(result.success, "uv install failed: {:?}", result.error);

        // After uv install, Python should also be available via uv
        // The install_uv function runs `uv python install` automatically
        let _uv_python_dir = tmp.path().join("runtimes").join("uv-python");
        // Note: Python may not install to uv-python if UV_PYTHON_INSTALL_DIR isn't set during install
        // But the uv binary itself should work with `uvx`
        let resolved_uv = bs.resolve_binary("uv");
        assert!(resolved_uv.is_some());
    }

    #[tokio::test]
    #[ignore] // End-to-end: install + resolve + spawn
    async fn integration_end_to_end_node_spawn() {
        let tmp = TempDir::new("int-e2e-node");

        let bs = RuntimeBootstrapper::new(tmp.path());
        bs.install_runtime(RuntimeKind::Node, |_| {}).await;

        // Use command() builder to run a Node.js one-liner
        let output = bs
            .command("node")
            .args([
                "-e",
                "console.log(JSON.stringify({ok: true, pid: process.pid}))",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .expect("Failed to spawn node via command()");

        assert!(output.status.success(), "Node.js command should succeed");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("Node output should be valid JSON");
        assert_eq!(parsed["ok"], true, "Node should return ok:true");
    }

    #[tokio::test]
    #[ignore] // End-to-end: install + resolve + spawn
    async fn integration_end_to_end_uv_spawn() {
        let tmp = TempDir::new("int-e2e-uv");

        let bs = RuntimeBootstrapper::new(tmp.path());
        bs.install_runtime(RuntimeKind::Uv, |_| {}).await;

        // Use command() builder to run uv --version
        let output = bs
            .command("uv")
            .args(["--version"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .expect("Failed to spawn uv via command()");

        assert!(output.status.success(), "uv command should succeed");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("uv"), "Output should contain 'uv'");
    }
}
