//! MCP Tool Catalog — Curated app store + Official MCP Registry integration.
//!
//! Provides:
//! - A built-in catalog of 30+ popular MCP servers (no network required)
//! - Integration with the Official MCP Registry (6,000+ servers at registry.modelcontextprotocol.io)
//! - Runtime detection (npx, node, uv, uvx, python3)
//! - One-click install: auto-configures and connects MCP servers
//! - Background sync with local cache for offline search
//!
//! Inspired by Claude Desktop Extensions, Smithery.ai, and Cursor's MCP marketplace.
//! The goal is to make installing MCP tools as easy as installing an app from an app store.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Simple percent-encoding for URL query parameters.
fn url_encode(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len() * 2);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

/// A single entry in the MCP tool catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Unique identifier (e.g., "filesystem", "github", "brave-search").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Short description of what the tool does.
    pub description: String,
    /// Category for browsing (e.g., "developer", "productivity", "data").
    pub category: String,
    /// Emoji icon for quick visual identification.
    pub icon: String,
    /// Required runtime: "node", "python", or "binary".
    pub runtime: String,
    /// Transport type: "stdio" or "http".
    pub transport: String,
    /// Command to execute (for stdio).
    pub command: String,
    /// Arguments for the command.
    pub args: Vec<String>,
    /// Whether this is an MCP App (has UI capabilities).
    pub is_mcp_app: bool,
    /// Required environment variables (keys only — user must provide values).
    pub required_env: Vec<EnvVarSpec>,
    /// Tags for search/filtering.
    pub tags: Vec<String>,
    /// Popularity rank (lower = more popular, for sorting).
    pub popularity: u32,
    /// Whether this is an official @modelcontextprotocol server.
    pub official: bool,
    /// npm/PyPI package name for version checking.
    pub package: Option<String>,
    /// GitHub repository URL for reference.
    pub repository: Option<String>,
}

/// Specification for a required environment variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarSpec {
    /// Environment variable name (e.g., "GITHUB_TOKEN").
    pub name: String,
    /// Human-readable label.
    pub label: String,
    /// Description of what this variable is for.
    pub description: String,
    /// Whether the value is sensitive (should be masked in UI).
    pub sensitive: bool,
    /// Optional placeholder/example value.
    pub placeholder: Option<String>,
    /// Whether this variable is required.
    pub required: bool,
}

/// Available runtimes detected on the user's system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    /// Whether Node.js is available.
    pub has_node: bool,
    /// Node.js version string (if available).
    pub node_version: Option<String>,
    /// Whether npx is available.
    pub has_npx: bool,
    /// Whether Python 3 is available.
    pub has_python: bool,
    /// Python version string (if available).
    pub python_version: Option<String>,
    /// Whether uv (Python package manager) is available.
    pub has_uv: bool,
    /// Whether uvx (uv tool runner) is available.
    pub has_uvx: bool,
}

/// Detect available runtimes on the system.
/// This runs quick `which`/`where` checks for each runtime.
pub async fn detect_runtimes() -> RuntimeInfo {
    let (has_node, node_version) = check_command_version("node", &["--version"]).await;
    let has_npx = check_command_exists("npx").await;
    let (has_python, python_version) = detect_python().await;
    let has_uv = check_command_exists("uv").await;
    let has_uvx = check_command_exists("uvx").await;

    RuntimeInfo {
        has_node,
        node_version,
        has_npx,
        has_python,
        python_version,
        has_uv,
        has_uvx,
    }
}

/// Check if a command exists and get its version.
async fn check_command_version(cmd: &str, args: &[&str]) -> (bool, Option<String>) {
    match tokio::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(version))
        }
        _ => (false, None),
    }
}

/// Check if a command exists via `which` (Unix) or `where` (Windows).
async fn check_command_exists(cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    let check_cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let check_cmd = "which";

    tokio::process::Command::new(check_cmd)
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Detect Python 3, trying `python3` first then `python`.
async fn detect_python() -> (bool, Option<String>) {
    // Try python3 first
    let (found, version) = check_command_version("python3", &["--version"]).await;
    if found {
        return (true, version);
    }
    // Fallback to python (check it's Python 3)
    let (found, version) = check_command_version("python", &["--version"]).await;
    if found {
        if let Some(ref v) = version {
            if v.contains("3.") {
                return (true, version);
            }
        }
    }
    (false, None)
}

/// Get the built-in curated catalog of popular MCP servers.
pub fn get_catalog() -> Vec<CatalogEntry> {
    vec![
        // ─── Developer Tools ───────────────────────────────────
        CatalogEntry {
            id: "filesystem".into(),
            name: "Filesystem".into(),
            description: "Read, write, and manage files and directories on your computer".into(),
            category: "developer".into(),
            icon: "📁".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-filesystem".into(),
                "{HOME}".into(),
            ],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "files".into(),
                "filesystem".into(),
                "read".into(),
                "write".into(),
            ],
            popularity: 1,
            official: true,
            package: Some("@modelcontextprotocol/server-filesystem".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "github".into(),
            name: "GitHub".into(),
            description: "Manage repositories, issues, PRs, and code on GitHub".into(),
            category: "developer".into(),
            icon: "🐙".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@modelcontextprotocol/server-github".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "GITHUB_PERSONAL_ACCESS_TOKEN".into(),
                label: "GitHub Token".into(),
                description: "Personal access token from github.com/settings/tokens".into(),
                sensitive: true,
                placeholder: Some("ghp_xxxxxxxxxxxxxxxxxxxx".into()),
                required: true,
            }],
            tags: vec![
                "git".into(),
                "github".into(),
                "repos".into(),
                "issues".into(),
                "pr".into(),
            ],
            popularity: 2,
            official: true,
            package: Some("@modelcontextprotocol/server-github".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "git".into(),
            name: "Git".into(),
            description:
                "Interact with local Git repositories — log, diff, blame, branch, and more".into(),
            category: "developer".into(),
            icon: "🔀".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "uvx".into(),
            args: vec!["mcp-server-git".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "git".into(),
                "version-control".into(),
                "diff".into(),
                "log".into(),
            ],
            popularity: 8,
            official: true,
            package: Some("mcp-server-git".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "sequential-thinking".into(),
            name: "Sequential Thinking".into(),
            description: "Enhanced chain-of-thought reasoning for complex problems".into(),
            category: "developer".into(),
            icon: "🧠".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-sequential-thinking".into(),
            ],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "thinking".into(),
                "reasoning".into(),
                "chain-of-thought".into(),
            ],
            popularity: 3,
            official: true,
            package: Some("@modelcontextprotocol/server-sequential-thinking".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "playwright".into(),
            name: "Playwright".into(),
            description: "Browser automation — navigate, click, fill forms, take screenshots"
                .into(),
            category: "developer".into(),
            icon: "🎭".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@playwright/mcp@latest".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "browser".into(),
                "automation".into(),
                "testing".into(),
                "web".into(),
            ],
            popularity: 5,
            official: false,
            package: Some("@playwright/mcp".into()),
            repository: Some("https://github.com/microsoft/playwright-mcp".into()),
        },
        CatalogEntry {
            id: "puppeteer".into(),
            name: "Puppeteer".into(),
            description: "Chrome browser automation for web scraping and testing".into(),
            category: "developer".into(),
            icon: "🌐".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@modelcontextprotocol/server-puppeteer".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "browser".into(),
                "chrome".into(),
                "scraping".into(),
                "automation".into(),
            ],
            popularity: 10,
            official: true,
            package: Some("@modelcontextprotocol/server-puppeteer".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        // ─── Search & Knowledge ─────────────────────────────
        CatalogEntry {
            id: "brave-search".into(),
            name: "Brave Search".into(),
            description: "Web search and local search using the Brave Search API".into(),
            category: "search".into(),
            icon: "🦁".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-brave-search".into(),
            ],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "BRAVE_API_KEY".into(),
                label: "Brave API Key".into(),
                description: "API key from brave.com/search/api".into(),
                sensitive: true,
                placeholder: Some("BSAxxxxxxxxxxxxxxxxx".into()),
                required: true,
            }],
            tags: vec!["search".into(), "web".into(), "brave".into()],
            popularity: 4,
            official: true,
            package: Some("@modelcontextprotocol/server-brave-search".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "fetch".into(),
            name: "Fetch".into(),
            description: "Make HTTP requests — GET, POST, and fetch any URL content".into(),
            category: "search".into(),
            icon: "🔗".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@kazuph/mcp-fetch".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec!["http".into(), "fetch".into(), "api".into(), "web".into()],
            popularity: 6,
            official: false,
            package: Some("@kazuph/mcp-fetch".into()),
            repository: Some("https://github.com/kazuph/mcp-fetch".into()),
        },
        CatalogEntry {
            id: "memory".into(),
            name: "Memory".into(),
            description: "Persistent key-value memory with knowledge graph for agents".into(),
            category: "search".into(),
            icon: "💾".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@modelcontextprotocol/server-memory".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "memory".into(),
                "knowledge".into(),
                "graph".into(),
                "persistence".into(),
            ],
            popularity: 7,
            official: true,
            package: Some("@modelcontextprotocol/server-memory".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "context7".into(),
            name: "Context7".into(),
            description:
                "Up-to-date documentation for any library — always current, never hallucinated"
                    .into(),
            category: "search".into(),
            icon: "📚".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@upstash/context7-mcp@latest".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "docs".into(),
                "documentation".into(),
                "libraries".into(),
                "context".into(),
            ],
            popularity: 9,
            official: false,
            package: Some("@upstash/context7-mcp".into()),
            repository: Some("https://github.com/upstash/context7-mcp".into()),
        },
        // ─── Productivity ─────────────────────────────────────
        CatalogEntry {
            id: "google-drive".into(),
            name: "Google Drive".into(),
            description: "Search and read files from your Google Drive".into(),
            category: "productivity".into(),
            icon: "📄".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@modelcontextprotocol/server-gdrive".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "GDRIVE_CREDENTIALS_PATH".into(),
                label: "Credentials Path".into(),
                description: "Path to your Google OAuth credentials JSON file".into(),
                sensitive: false,
                placeholder: Some("/path/to/credentials.json".into()),
                required: true,
            }],
            tags: vec![
                "google".into(),
                "drive".into(),
                "docs".into(),
                "files".into(),
            ],
            popularity: 14,
            official: true,
            package: Some("@modelcontextprotocol/server-gdrive".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "google-maps".into(),
            name: "Google Maps".into(),
            description: "Geocoding, directions, places search, and elevation data".into(),
            category: "productivity".into(),
            icon: "🗺️".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-google-maps".into(),
            ],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "GOOGLE_MAPS_API_KEY".into(),
                label: "Maps API Key".into(),
                description: "API key from Google Cloud Console".into(),
                sensitive: true,
                placeholder: Some("AIzaSyxxxxxxxxxxxxxxxxx".into()),
                required: true,
            }],
            tags: vec![
                "maps".into(),
                "location".into(),
                "geocoding".into(),
                "directions".into(),
            ],
            popularity: 15,
            official: true,
            package: Some("@modelcontextprotocol/server-google-maps".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "slack".into(),
            name: "Slack".into(),
            description: "Read messages, post updates, and manage Slack channels".into(),
            category: "productivity".into(),
            icon: "💬".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@modelcontextprotocol/server-slack".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "SLACK_BOT_TOKEN".into(),
                label: "Slack Bot Token".into(),
                description: "Bot token from api.slack.com/apps".into(),
                sensitive: true,
                placeholder: Some("xoxb-xxxxxxxxxxxx".into()),
                required: true,
            }],
            tags: vec![
                "slack".into(),
                "chat".into(),
                "messaging".into(),
                "team".into(),
            ],
            popularity: 12,
            official: true,
            package: Some("@modelcontextprotocol/server-slack".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "notion".into(),
            name: "Notion".into(),
            description: "Search, read, create, and update Notion pages and databases".into(),
            category: "productivity".into(),
            icon: "📝".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "notion-mcp-server".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "NOTION_API_KEY".into(),
                label: "Notion API Key".into(),
                description: "Integration token from notion.so/my-integrations".into(),
                sensitive: true,
                placeholder: Some("ntn_xxxxxxxxxxxx".into()),
                required: true,
            }],
            tags: vec![
                "notion".into(),
                "notes".into(),
                "wiki".into(),
                "docs".into(),
            ],
            popularity: 11,
            official: false,
            package: Some("notion-mcp-server".into()),
            repository: None,
        },
        CatalogEntry {
            id: "linear".into(),
            name: "Linear".into(),
            description: "Manage issues, projects, and workflows in Linear".into(),
            category: "productivity".into(),
            icon: "📐".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "mcp-linear".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "LINEAR_API_KEY".into(),
                label: "Linear API Key".into(),
                description: "API key from Linear settings".into(),
                sensitive: true,
                placeholder: Some("lin_api_xxxxxxxxxxxx".into()),
                required: true,
            }],
            tags: vec![
                "linear".into(),
                "issues".into(),
                "project-management".into(),
            ],
            popularity: 16,
            official: false,
            package: Some("mcp-linear".into()),
            repository: None,
        },
        // ─── Data & Database ───────────────────────────────────
        CatalogEntry {
            id: "sqlite".into(),
            name: "SQLite".into(),
            description: "Query and manage SQLite databases with SQL".into(),
            category: "data".into(),
            icon: "🗄️".into(),
            runtime: "python".into(),
            transport: "stdio".into(),
            command: "uvx".into(),
            args: vec![
                "mcp-server-sqlite".into(),
                "--db-path".into(),
                "{DB_PATH}".into(),
            ],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "sqlite".into(),
                "database".into(),
                "sql".into(),
                "query".into(),
            ],
            popularity: 13,
            official: true,
            package: Some("mcp-server-sqlite".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "postgres".into(),
            name: "PostgreSQL".into(),
            description: "Connect and query PostgreSQL databases".into(),
            category: "data".into(),
            icon: "🐘".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@modelcontextprotocol/server-postgres".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "POSTGRES_CONNECTION_STRING".into(),
                label: "Connection String".into(),
                description: "PostgreSQL connection string (e.g., postgresql://user:pass@host/db)"
                    .into(),
                sensitive: true,
                placeholder: Some("postgresql://user:password@localhost:5432/mydb".into()),
                required: true,
            }],
            tags: vec!["postgres".into(), "database".into(), "sql".into()],
            popularity: 17,
            official: true,
            package: Some("@modelcontextprotocol/server-postgres".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "redis".into(),
            name: "Redis".into(),
            description: "Interact with Redis for caching and data storage".into(),
            category: "data".into(),
            icon: "🔴".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "@modelcontextprotocol/server-redis".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "REDIS_URL".into(),
                label: "Redis URL".into(),
                description: "Redis connection URL".into(),
                sensitive: true,
                placeholder: Some("redis://localhost:6379".into()),
                required: true,
            }],
            tags: vec!["redis".into(), "cache".into(), "database".into()],
            popularity: 22,
            official: true,
            package: Some("@modelcontextprotocol/server-redis".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        // ─── Cloud & DevOps ────────────────────────────────────
        CatalogEntry {
            id: "docker".into(),
            name: "Docker".into(),
            description: "Manage Docker containers, images, volumes, and networks".into(),
            category: "devops".into(),
            icon: "🐳".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "mcp-docker".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec!["docker".into(), "containers".into(), "devops".into()],
            popularity: 18,
            official: false,
            package: Some("mcp-docker".into()),
            repository: None,
        },
        CatalogEntry {
            id: "kubernetes".into(),
            name: "Kubernetes".into(),
            description: "Manage Kubernetes clusters, pods, deployments, and services".into(),
            category: "devops".into(),
            icon: "☸️".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "mcp-kubernetes".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![
                "kubernetes".into(),
                "k8s".into(),
                "clusters".into(),
                "devops".into(),
            ],
            popularity: 25,
            official: false,
            package: Some("mcp-kubernetes".into()),
            repository: None,
        },
        CatalogEntry {
            id: "aws".into(),
            name: "AWS".into(),
            description: "Interact with AWS services — S3, EC2, Lambda, and more".into(),
            category: "devops".into(),
            icon: "☁️".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "mcp-aws".into()],
            is_mcp_app: false,
            required_env: vec![
                EnvVarSpec {
                    name: "AWS_ACCESS_KEY_ID".into(),
                    label: "AWS Access Key".into(),
                    description: "AWS IAM access key ID".into(),
                    sensitive: true,
                    placeholder: Some("AKIAxxxxxxxxxxxxxxxxx".into()),
                    required: true,
                },
                EnvVarSpec {
                    name: "AWS_SECRET_ACCESS_KEY".into(),
                    label: "AWS Secret Key".into(),
                    description: "AWS IAM secret access key".into(),
                    sensitive: true,
                    placeholder: None,
                    required: true,
                },
            ],
            tags: vec!["aws".into(), "cloud".into(), "s3".into(), "lambda".into()],
            popularity: 20,
            official: false,
            package: Some("mcp-aws".into()),
            repository: None,
        },
        // ─── MCP Apps (with UI) ────────────────────────────────
        CatalogEntry {
            id: "mcp-app-threejs".into(),
            name: "3D Viewer".into(),
            description: "Interactive 3D scenes and visualizations rendered in the chat".into(),
            category: "apps".into(),
            icon: "🎨".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "--silent".into(),
                "@modelcontextprotocol/server-threejs".into(),
                "--stdio".into(),
            ],
            is_mcp_app: true,
            required_env: vec![],
            tags: vec![
                "3d".into(),
                "visualization".into(),
                "threejs".into(),
                "ui".into(),
            ],
            popularity: 30,
            official: true,
            package: Some("@modelcontextprotocol/server-threejs".into()),
            repository: Some("https://github.com/anthropics/mcp-apps".into()),
        },
        CatalogEntry {
            id: "mcp-app-map".into(),
            name: "Map".into(),
            description: "Interactive maps rendered directly in the conversation".into(),
            category: "apps".into(),
            icon: "🗺️".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "--silent".into(),
                "@modelcontextprotocol/server-map".into(),
                "--stdio".into(),
            ],
            is_mcp_app: true,
            required_env: vec![],
            tags: vec![
                "map".into(),
                "location".into(),
                "geography".into(),
                "ui".into(),
            ],
            popularity: 31,
            official: true,
            package: Some("@modelcontextprotocol/server-map".into()),
            repository: Some("https://github.com/anthropics/mcp-apps".into()),
        },
        CatalogEntry {
            id: "mcp-app-pdf".into(),
            name: "PDF Viewer".into(),
            description: "View and navigate PDF documents inline in the chat".into(),
            category: "apps".into(),
            icon: "📕".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "--silent".into(),
                "@modelcontextprotocol/server-pdf".into(),
                "--stdio".into(),
            ],
            is_mcp_app: true,
            required_env: vec![],
            tags: vec![
                "pdf".into(),
                "document".into(),
                "viewer".into(),
                "ui".into(),
            ],
            popularity: 32,
            official: true,
            package: Some("@modelcontextprotocol/server-pdf".into()),
            repository: Some("https://github.com/anthropics/mcp-apps".into()),
        },
        CatalogEntry {
            id: "mcp-app-system-monitor".into(),
            name: "System Monitor".into(),
            description: "Real-time system monitoring dashboard — CPU, memory, disk".into(),
            category: "apps".into(),
            icon: "📊".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "--silent".into(),
                "@modelcontextprotocol/server-system-monitor".into(),
                "--stdio".into(),
            ],
            is_mcp_app: true,
            required_env: vec![],
            tags: vec![
                "system".into(),
                "monitor".into(),
                "dashboard".into(),
                "ui".into(),
            ],
            popularity: 33,
            official: true,
            package: Some("@modelcontextprotocol/server-system-monitor".into()),
            repository: Some("https://github.com/anthropics/mcp-apps".into()),
        },
        CatalogEntry {
            id: "mcp-app-budget".into(),
            name: "Budget Allocator".into(),
            description: "Interactive budget planning and allocation tool".into(),
            category: "apps".into(),
            icon: "💰".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "--silent".into(),
                "@modelcontextprotocol/server-budget-allocator".into(),
                "--stdio".into(),
            ],
            is_mcp_app: true,
            required_env: vec![],
            tags: vec![
                "budget".into(),
                "finance".into(),
                "planning".into(),
                "ui".into(),
            ],
            popularity: 34,
            official: true,
            package: Some("@modelcontextprotocol/server-budget-allocator".into()),
            repository: Some("https://github.com/anthropics/mcp-apps".into()),
        },
        CatalogEntry {
            id: "mcp-app-shadertoy".into(),
            name: "Shadertoy".into(),
            description: "Live shader editor and renderer — create visual effects in the chat"
                .into(),
            category: "apps".into(),
            icon: "✨".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "--silent".into(),
                "@modelcontextprotocol/server-shadertoy".into(),
                "--stdio".into(),
            ],
            is_mcp_app: true,
            required_env: vec![],
            tags: vec![
                "shader".into(),
                "graphics".into(),
                "webgl".into(),
                "ui".into(),
            ],
            popularity: 35,
            official: true,
            package: Some("@modelcontextprotocol/server-shadertoy".into()),
            repository: Some("https://github.com/anthropics/mcp-apps".into()),
        },
        CatalogEntry {
            id: "mcp-app-wiki".into(),
            name: "Wiki Explorer".into(),
            description: "Browse and navigate Wikipedia articles with interactive UI".into(),
            category: "apps".into(),
            icon: "🌍".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "--silent".into(),
                "@modelcontextprotocol/server-wiki-explorer".into(),
                "--stdio".into(),
            ],
            is_mcp_app: true,
            required_env: vec![],
            tags: vec![
                "wiki".into(),
                "wikipedia".into(),
                "knowledge".into(),
                "ui".into(),
            ],
            popularity: 36,
            official: true,
            package: Some("@modelcontextprotocol/server-wiki-explorer".into()),
            repository: Some("https://github.com/anthropics/mcp-apps".into()),
        },
        CatalogEntry {
            id: "mcp-app-sheet-music".into(),
            name: "Sheet Music".into(),
            description: "Render and display sheet music notation interactively".into(),
            category: "apps".into(),
            icon: "🎵".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "--silent".into(),
                "@modelcontextprotocol/server-sheet-music".into(),
                "--stdio".into(),
            ],
            is_mcp_app: true,
            required_env: vec![],
            tags: vec![
                "music".into(),
                "notation".into(),
                "sheet-music".into(),
                "ui".into(),
            ],
            popularity: 37,
            official: true,
            package: Some("@modelcontextprotocol/server-sheet-music".into()),
            repository: Some("https://github.com/anthropics/mcp-apps".into()),
        },
        // ─── Communication & Integrations ───────────────────────
        CatalogEntry {
            id: "discord".into(),
            name: "Discord".into(),
            description: "Read messages, manage channels, and interact with Discord servers".into(),
            category: "communication".into(),
            icon: "💜".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "mcp-discord".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "DISCORD_BOT_TOKEN".into(),
                label: "Discord Bot Token".into(),
                description: "Bot token from Discord Developer Portal".into(),
                sensitive: true,
                placeholder: None,
                required: true,
            }],
            tags: vec!["discord".into(), "chat".into(), "community".into()],
            popularity: 23,
            official: false,
            package: Some("mcp-discord".into()),
            repository: None,
        },
        CatalogEntry {
            id: "gmail".into(),
            name: "Gmail".into(),
            description: "Search, read, and compose emails in Gmail".into(),
            category: "communication".into(),
            icon: "✉️".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "mcp-gmail".into()],
            is_mcp_app: false,
            required_env: vec![EnvVarSpec {
                name: "GMAIL_CREDENTIALS_PATH".into(),
                label: "Credentials Path".into(),
                description: "Path to your Google OAuth credentials JSON".into(),
                sensitive: false,
                placeholder: Some("/path/to/credentials.json".into()),
                required: true,
            }],
            tags: vec!["email".into(), "gmail".into(), "google".into()],
            popularity: 19,
            official: false,
            package: Some("mcp-gmail".into()),
            repository: None,
        },
        // ─── Misc / Utility ────────────────────────────────────
        CatalogEntry {
            id: "everything".into(),
            name: "Everything".into(),
            description:
                "Reference MCP server implementing all protocol features — great for testing".into(),
            category: "utility".into(),
            icon: "🧪".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-everything".into(),
            ],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec!["testing".into(), "reference".into(), "everything".into()],
            popularity: 50,
            official: true,
            package: Some("@modelcontextprotocol/server-everything".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
        CatalogEntry {
            id: "time".into(),
            name: "Time".into(),
            description: "Get current time, timezone conversions, and time math".into(),
            category: "utility".into(),
            icon: "🕐".into(),
            runtime: "python".into(),
            transport: "stdio".into(),
            command: "uvx".into(),
            args: vec!["mcp-server-time".into()],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec!["time".into(), "timezone".into(), "datetime".into()],
            popularity: 24,
            official: true,
            package: Some("mcp-server-time".into()),
            repository: Some("https://github.com/modelcontextprotocol/servers".into()),
        },
    ]
}

/// Get all unique categories from the catalog.
pub fn get_categories() -> Vec<CatalogCategory> {
    vec![
        CatalogCategory {
            id: "all".into(),
            name: "All".into(),
            icon: "🏠".into(),
        },
        CatalogCategory {
            id: "developer".into(),
            name: "Developer".into(),
            icon: "⚡".into(),
        },
        CatalogCategory {
            id: "search".into(),
            name: "Search & Knowledge".into(),
            icon: "🔍".into(),
        },
        CatalogCategory {
            id: "productivity".into(),
            name: "Productivity".into(),
            icon: "📋".into(),
        },
        CatalogCategory {
            id: "data".into(),
            name: "Data & Database".into(),
            icon: "🗄️".into(),
        },
        CatalogCategory {
            id: "devops".into(),
            name: "Cloud & DevOps".into(),
            icon: "☁️".into(),
        },
        CatalogCategory {
            id: "apps".into(),
            name: "MCP Apps".into(),
            icon: "🎨".into(),
        },
        CatalogCategory {
            id: "communication".into(),
            name: "Communication".into(),
            icon: "💬".into(),
        },
        CatalogCategory {
            id: "utility".into(),
            name: "Utility".into(),
            icon: "🔧".into(),
        },
    ]
}

/// Category metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
}

/// Resolve template variables in args.
/// Supports: {HOME}, {DB_PATH}, etc.
pub fn resolve_args(args: &[String], env_overrides: &HashMap<String, String>) -> Vec<String> {
    args.iter()
        .map(|arg| {
            let mut resolved = arg.clone();
            // Resolve {HOME}
            if resolved.contains("{HOME}") {
                let home = dirs::home_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                resolved = resolved.replace("{HOME}", &home);
            }
            // Resolve any {VAR_NAME} from env_overrides
            for (key, value) in env_overrides {
                let placeholder = format!("{{{}}}", key);
                resolved = resolved.replace(&placeholder, value);
            }
            resolved
        })
        .collect()
}

/// Check if a catalog entry can be installed with the current runtimes.
pub fn can_install(entry: &CatalogEntry, runtimes: &RuntimeInfo) -> bool {
    match entry.runtime.as_str() {
        "node" => {
            // Check if the command is npx or node
            match entry.command.as_str() {
                "npx" => runtimes.has_npx,
                "node" => runtimes.has_node,
                _ => runtimes.has_npx || runtimes.has_node,
            }
        }
        "python" => {
            // Check if uvx or python is available
            match entry.command.as_str() {
                "uvx" => runtimes.has_uvx,
                "uv" => runtimes.has_uv,
                "python" | "python3" => runtimes.has_python,
                _ => runtimes.has_python || runtimes.has_uvx,
            }
        }
        "binary" => true, // Binaries are self-contained
        _ => false,
    }
}

/// Build the McpServerEntry from a catalog entry + user-provided env vars.
pub fn build_server_entry(
    entry: &CatalogEntry,
    env_vars: HashMap<String, String>,
) -> super::McpServerEntry {
    let resolved_args = resolve_args(&entry.args, &env_vars);

    super::McpServerEntry {
        name: entry.name.clone(),
        transport: entry.transport.clone(),
        command: Some(entry.command.clone()),
        args: resolved_args,
        url: if entry.transport == "http" {
            // For remote servers, the URL is in args[0] or env
            entry.args.first().cloned()
        } else {
            None
        },
        enabled: true,
        env: env_vars,
    }
}

// ─── Official MCP Registry Client ─────────────────────────────

/// Base URL for the official MCP Registry API.
const REGISTRY_BASE_URL: &str = "https://registry.modelcontextprotocol.io";

/// API version path.
const REGISTRY_API_VERSION: &str = "v0.1";

/// Maximum entries per page (registry limit).
const REGISTRY_PAGE_LIMIT: u32 = 100;

/// Maximum pages to fetch in a single sync (safety limit).
const REGISTRY_MAX_PAGES: u32 = 80;

/// Cache file name.
const REGISTRY_CACHE_FILE: &str = "mcp_registry_cache.json";

/// Cache metadata file name.
const REGISTRY_CACHE_META: &str = "mcp_registry_cache_meta.json";

/// Cache TTL in seconds (24 hours).
const REGISTRY_CACHE_TTL_SECS: u64 = 86400;

/// A server entry from the official MCP Registry (server.json format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryServer {
    /// Unique reverse-domain name (e.g., "io.github.user/server-name").
    pub name: String,
    /// Human-readable title.
    pub title: Option<String>,
    /// Server description.
    pub description: String,
    /// Semantic version.
    pub version: String,
    /// Installable packages (npm, pypi, oci, nuget, mcpb).
    #[serde(default)]
    pub packages: Vec<RegistryPackage>,
    /// Remote server endpoints (streamable-http, sse).
    #[serde(default)]
    pub remotes: Vec<RegistryRemote>,
    /// Server icons.
    #[serde(default)]
    pub icons: Vec<RegistryIcon>,
    /// Source repository.
    pub repository: Option<RegistryRepository>,
    /// Website URL.
    #[serde(rename = "websiteUrl")]
    pub website_url: Option<String>,
}

/// A package in the MCP Registry (how to install/run the server).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPackage {
    /// Package registry type: "npm", "pypi", "oci", "nuget", "mcpb".
    #[serde(rename = "registryType")]
    pub registry_type: String,
    /// Package identifier (e.g., "@modelcontextprotocol/server-filesystem").
    pub identifier: String,
    /// Package version.
    pub version: Option<String>,
    /// Transport configuration.
    pub transport: Option<RegistryTransport>,
    /// Required environment variables.
    #[serde(rename = "environmentVariables", default)]
    pub environment_variables: Vec<RegistryEnvVar>,
    /// Runtime hint (e.g., "node", "python", "docker", "dnx").
    #[serde(rename = "runtimeHint")]
    pub runtime_hint: Option<String>,
}

/// Transport info from registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryTransport {
    /// Transport type: "stdio", "streamable-http", "sse".
    #[serde(rename = "type")]
    pub transport_type: String,
    /// URL for HTTP transports.
    pub url: Option<String>,
}

/// Environment variable from registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEnvVar {
    /// Variable name.
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: String,
    /// Whether required.
    #[serde(rename = "isRequired", default)]
    pub is_required: bool,
    /// Whether the value is secret (API keys, tokens).
    #[serde(rename = "isSecret", default)]
    pub is_secret: bool,
    /// Default value.
    pub default: Option<String>,
}

/// Remote server endpoint from registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryRemote {
    /// Type: "streamable-http", "sse".
    #[serde(rename = "type")]
    pub remote_type: String,
    /// Server URL.
    pub url: String,
}

/// Icon from registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIcon {
    /// Icon URL.
    pub src: String,
    /// MIME type.
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// Repository from registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryRepository {
    /// Repository URL.
    pub url: Option<String>,
    /// Source type (e.g., "github").
    pub source: Option<String>,
}

/// API response wrapper from registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryApiResponse {
    servers: Vec<RegistryServerWrapper>,
    metadata: RegistryMetadata,
}

/// Wrapper around a server entry in the API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryServerWrapper {
    server: RegistryServer,
}

/// Pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistryMetadata {
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
    count: Option<u32>,
}

/// Cache metadata — tracks sync state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryCacheMeta {
    /// When the cache was last fully synced (ISO 8601).
    pub last_sync: String,
    /// Total number of cached servers.
    pub total_servers: usize,
    /// Number of installable servers (have packages).
    pub installable_count: usize,
}

/// Result of a registry sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySyncResult {
    /// Whether the sync was successful.
    pub success: bool,
    /// Total servers in cache after sync.
    pub total_servers: usize,
    /// Number of installable servers.
    pub installable_count: usize,
    /// Error message if sync failed.
    pub error: Option<String>,
    /// Whether data was loaded from cache (not freshly synced).
    pub from_cache: bool,
}

/// Sync the official MCP Registry to a local cache.
///
/// Fetches all servers from `registry.modelcontextprotocol.io` using cursor-based
/// pagination (100 per page) and saves to a local JSON file for offline search.
///
/// This is an opt-in operation — Ghost never phones home without user action.
pub async fn sync_registry(cache_dir: &Path) -> RegistrySyncResult {
    tracing::info!("MCP Registry: starting sync from {}", REGISTRY_BASE_URL);

    let client = match reqwest::Client::builder()
        .user_agent("Ghost/0.11 (https://github.com/ghostapp-ai/ghost)")
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return RegistrySyncResult {
                success: false,
                total_servers: 0,
                installable_count: 0,
                error: Some(format!("Failed to create HTTP client: {}", e)),
                from_cache: false,
            }
        }
    };

    let mut all_servers: Vec<RegistryServer> = Vec::new();
    let mut cursor: Option<String> = None;
    let mut pages_fetched = 0u32;

    loop {
        if pages_fetched >= REGISTRY_MAX_PAGES {
            tracing::warn!(
                "MCP Registry: hit max pages limit ({}), stopping",
                REGISTRY_MAX_PAGES
            );
            break;
        }

        let mut url = format!(
            "{}/{}/servers?limit={}",
            REGISTRY_BASE_URL, REGISTRY_API_VERSION, REGISTRY_PAGE_LIMIT
        );
        if let Some(ref c) = cursor {
            url.push_str(&format!("&cursor={}", url_encode(c)));
        }

        match client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    tracing::error!("MCP Registry: API returned {}", status);
                    return RegistrySyncResult {
                        success: false,
                        total_servers: all_servers.len(),
                        installable_count: 0,
                        error: Some(format!("Registry API returned HTTP {}", status)),
                        from_cache: false,
                    };
                }

                match resp.json::<RegistryApiResponse>().await {
                    Ok(data) => {
                        let count = data.servers.len();
                        for wrapper in data.servers {
                            all_servers.push(wrapper.server);
                        }
                        pages_fetched += 1;

                        tracing::debug!(
                            "MCP Registry: page {} — {} servers (total: {})",
                            pages_fetched,
                            count,
                            all_servers.len()
                        );

                        // Check if we've reached the end
                        if count < REGISTRY_PAGE_LIMIT as usize
                            || data.metadata.next_cursor.is_none()
                        {
                            break;
                        }
                        cursor = data.metadata.next_cursor;
                    }
                    Err(e) => {
                        tracing::error!("MCP Registry: failed to parse response: {}", e);
                        return RegistrySyncResult {
                            success: false,
                            total_servers: all_servers.len(),
                            installable_count: 0,
                            error: Some(format!("Failed to parse registry response: {}", e)),
                            from_cache: false,
                        };
                    }
                }
            }
            Err(e) => {
                tracing::error!("MCP Registry: network error: {}", e);
                return RegistrySyncResult {
                    success: false,
                    total_servers: 0,
                    installable_count: 0,
                    error: Some(format!("Network error: {}", e)),
                    from_cache: false,
                };
            }
        }
    }

    let installable = all_servers
        .iter()
        .filter(|s| !s.packages.is_empty())
        .count();

    tracing::info!(
        "MCP Registry: synced {} servers ({} installable) in {} pages",
        all_servers.len(),
        installable,
        pages_fetched
    );

    // Save cache
    let cache_path = cache_dir.join(REGISTRY_CACHE_FILE);
    let meta_path = cache_dir.join(REGISTRY_CACHE_META);

    if let Err(e) = std::fs::create_dir_all(cache_dir) {
        tracing::error!("MCP Registry: failed to create cache dir: {}", e);
    }

    if let Ok(json) = serde_json::to_string(&all_servers) {
        if let Err(e) = std::fs::write(&cache_path, &json) {
            tracing::error!("MCP Registry: failed to write cache: {}", e);
        }
    }

    let meta = RegistryCacheMeta {
        last_sync: chrono::Utc::now().to_rfc3339(),
        total_servers: all_servers.len(),
        installable_count: installable,
    };
    if let Ok(json) = serde_json::to_string_pretty(&meta) {
        let _ = std::fs::write(&meta_path, &json);
    }

    RegistrySyncResult {
        success: true,
        total_servers: all_servers.len(),
        installable_count: installable,
        error: None,
        from_cache: false,
    }
}

/// Load the registry cache from disk.
pub fn load_registry_cache(cache_dir: &Path) -> Option<Vec<RegistryServer>> {
    let cache_path = cache_dir.join(REGISTRY_CACHE_FILE);
    let meta_path = cache_dir.join(REGISTRY_CACHE_META);

    // Check if cache exists and is fresh
    if let Ok(meta_str) = std::fs::read_to_string(&meta_path) {
        if let Ok(meta) = serde_json::from_str::<RegistryCacheMeta>(&meta_str) {
            if let Ok(last_sync) = chrono::DateTime::parse_from_rfc3339(&meta.last_sync) {
                let age = chrono::Utc::now()
                    .signed_duration_since(last_sync)
                    .num_seconds();
                if age > REGISTRY_CACHE_TTL_SECS as i64 {
                    tracing::info!(
                        "MCP Registry: cache expired (age: {}s > TTL: {}s)",
                        age,
                        REGISTRY_CACHE_TTL_SECS
                    );
                    // Cache is stale but we can still return it
                }
            }
        }
    }

    match std::fs::read_to_string(&cache_path) {
        Ok(json) => match serde_json::from_str::<Vec<RegistryServer>>(&json) {
            Ok(servers) => {
                tracing::info!("MCP Registry: loaded {} servers from cache", servers.len());
                Some(servers)
            }
            Err(e) => {
                tracing::error!("MCP Registry: failed to parse cache: {}", e);
                None
            }
        },
        Err(_) => None,
    }
}

/// Get cache metadata (sync status, counts).
pub fn get_cache_meta(cache_dir: &Path) -> Option<RegistryCacheMeta> {
    let meta_path = cache_dir.join(REGISTRY_CACHE_META);
    std::fs::read_to_string(&meta_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Check if the cache is fresh (within TTL).
pub fn is_cache_fresh(cache_dir: &Path) -> bool {
    get_cache_meta(cache_dir)
        .and_then(|meta| chrono::DateTime::parse_from_rfc3339(&meta.last_sync).ok())
        .map(|last_sync| {
            chrono::Utc::now()
                .signed_duration_since(last_sync)
                .num_seconds()
                < REGISTRY_CACHE_TTL_SECS as i64
        })
        .unwrap_or(false)
}

/// Convert a registry server entry to a Ghost CatalogEntry for the UI.
///
/// Maps `registryType` to the appropriate local command:
/// - `npm` → `npx -y <identifier>` (stdio)
/// - `pypi` → `uvx <identifier>` (stdio)
/// - `oci` → `docker run -i --rm <identifier>` (stdio)
/// - Remote servers → HTTP transport with URL
///
/// Returns `None` if the server can't be auto-installed.
pub fn registry_to_catalog_entry(server: &RegistryServer) -> Option<CatalogEntry> {
    // Try installable packages first (prefer npm > pypi > oci)
    let preferred_order = ["npm", "pypi", "oci"];

    let package = preferred_order
        .iter()
        .find_map(|rt| server.packages.iter().find(|p| p.registry_type == *rt))
        .or_else(|| server.packages.first());

    // Determine runtime, command, args, transport
    let (runtime, transport, command, args, url) = if let Some(pkg) = package {
        match pkg.registry_type.as_str() {
            "npm" => (
                "node".to_string(),
                "stdio".to_string(),
                "npx".to_string(),
                vec!["-y".to_string(), pkg.identifier.clone()],
                None,
            ),
            "pypi" => (
                "python".to_string(),
                "stdio".to_string(),
                "uvx".to_string(),
                vec![pkg.identifier.clone()],
                None,
            ),
            "oci" => (
                "docker".to_string(),
                "stdio".to_string(),
                "docker".to_string(),
                vec![
                    "run".to_string(),
                    "-i".to_string(),
                    "--rm".to_string(),
                    pkg.identifier.clone(),
                ],
                None,
            ),
            _ => return None, // nuget, mcpb — not yet supported
        }
    } else if let Some(remote) = server.remotes.first() {
        // Remote-only server — no local install needed
        (
            "remote".to_string(),
            "http".to_string(),
            String::new(),
            vec![remote.url.clone()],
            Some(remote.url.clone()),
        )
    } else {
        return None; // No installable package and no remote — skip
    };

    // Extract env vars from the best package
    let required_env: Vec<EnvVarSpec> = package
        .map(|pkg| {
            pkg.environment_variables
                .iter()
                .map(|ev| EnvVarSpec {
                    name: ev.name.clone(),
                    label: ev.name.replace('_', " ").to_string(),
                    description: ev.description.clone(),
                    sensitive: ev.is_secret,
                    placeholder: ev.default.clone(),
                    required: ev.is_required,
                })
                .collect()
        })
        .unwrap_or_default();

    // Generate a short display name from the registry name
    let display_name = server
        .title
        .clone()
        .unwrap_or_else(|| {
            // "io.github.user/server-name" → "Server Name"
            server
                .name
                .rsplit('/')
                .next()
                .unwrap_or(&server.name)
                .replace('-', " ")
                .replace("mcp", "")
                .replace("server", "")
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .trim()
        .to_string();

    // Use first icon or generate from category
    let icon = if url.is_some() { "🌐" } else { "📦" }.to_string();

    // Derive category from name/description heuristics
    let desc_lower = server.description.to_lowercase();
    let name_lower = server.name.to_lowercase();
    let category = if desc_lower.contains("database")
        || desc_lower.contains("sql")
        || name_lower.contains("postgres")
        || name_lower.contains("redis")
        || name_lower.contains("mongo")
    {
        "data"
    } else if desc_lower.contains("github")
        || desc_lower.contains("git")
        || desc_lower.contains("code")
        || desc_lower.contains("developer")
    {
        "developer"
    } else if desc_lower.contains("search")
        || desc_lower.contains("browse")
        || desc_lower.contains("web")
    {
        "search"
    } else if desc_lower.contains("slack")
        || desc_lower.contains("discord")
        || desc_lower.contains("email")
        || desc_lower.contains("chat")
    {
        "communication"
    } else if desc_lower.contains("docker")
        || desc_lower.contains("kubernetes")
        || desc_lower.contains("aws")
        || desc_lower.contains("cloud")
        || desc_lower.contains("deploy")
    {
        "devops"
    } else if desc_lower.contains("file")
        || desc_lower.contains("document")
        || desc_lower.contains("pdf")
        || desc_lower.contains("note")
    {
        "productivity"
    } else {
        "utility"
    }
    .to_string();

    let repo_url = server.repository.as_ref().and_then(|r| r.url.clone());

    let pkg_name = package.map(|p| p.identifier.clone());

    Some(CatalogEntry {
        id: server.name.clone(),
        name: display_name,
        description: server.description.clone(),
        category,
        icon,
        runtime,
        transport,
        command,
        args,
        is_mcp_app: false,
        required_env,
        tags: vec![],
        popularity: 1000, // Registry entries sort after curated
        official: false,
        package: pkg_name,
        repository: repo_url,
    })
}

/// Search the registry cache, converting matches to CatalogEntry format.
///
/// Filters by query string (matched against name, title, description).
/// Ignores entries that already exist in the curated catalog (by package name).
pub fn search_registry(cache_dir: &Path, query: &str, limit: usize) -> Vec<CatalogEntry> {
    let servers = match load_registry_cache(cache_dir) {
        Some(s) => s,
        None => return vec![],
    };

    // Get curated package names to avoid duplicates
    let curated_ids: std::collections::HashSet<String> = get_catalog()
        .into_iter()
        .filter_map(|e| e.package)
        .collect();

    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    servers
        .iter()
        .filter(|s| {
            // Skip servers already in curated catalog
            if s.packages
                .iter()
                .any(|p| curated_ids.contains(&p.identifier))
            {
                return false;
            }

            // Must have installable package or remote
            if s.packages.is_empty() && s.remotes.is_empty() {
                return false;
            }

            // Match query words (all words must match in name, title, or description)
            if query_words.is_empty() {
                return true;
            }
            let haystack = format!(
                "{} {} {}",
                s.name.to_lowercase(),
                s.title.as_deref().unwrap_or("").to_lowercase(),
                s.description.to_lowercase()
            );
            query_words.iter().all(|w| haystack.contains(w))
        })
        .take(limit * 2) // Take extra to account for conversion failures
        .filter_map(registry_to_catalog_entry)
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_not_empty() {
        let catalog = get_catalog();
        assert!(
            catalog.len() >= 25,
            "Catalog should have at least 25 entries"
        );
    }

    #[test]
    fn test_all_entries_have_required_fields() {
        for entry in get_catalog() {
            assert!(!entry.id.is_empty(), "Entry must have id");
            assert!(!entry.name.is_empty(), "Entry must have name");
            assert!(!entry.description.is_empty(), "Entry must have description");
            assert!(!entry.category.is_empty(), "Entry must have category");
            assert!(!entry.icon.is_empty(), "Entry must have icon");
            assert!(!entry.command.is_empty(), "Entry must have command");
        }
    }

    #[test]
    fn test_resolve_args_home() {
        let args = vec!["{HOME}/documents".to_string()];
        let resolved = resolve_args(&args, &HashMap::new());
        assert!(!resolved[0].contains("{HOME}"));
    }

    #[test]
    fn test_resolve_args_custom_vars() {
        let args = vec!["--db-path".to_string(), "{DB_PATH}".to_string()];
        let mut env = HashMap::new();
        env.insert("DB_PATH".to_string(), "/tmp/test.db".to_string());
        let resolved = resolve_args(&args, &env);
        assert_eq!(resolved[1], "/tmp/test.db");
    }

    #[test]
    fn test_can_install_node() {
        let entry = CatalogEntry {
            id: "test".into(),
            name: "Test".into(),
            description: "".into(),
            category: "".into(),
            icon: "".into(),
            runtime: "node".into(),
            transport: "stdio".into(),
            command: "npx".into(),
            args: vec![],
            is_mcp_app: false,
            required_env: vec![],
            tags: vec![],
            popularity: 0,
            official: false,
            package: None,
            repository: None,
        };

        let runtimes_with_npx = RuntimeInfo {
            has_node: true,
            node_version: Some("v20.0.0".into()),
            has_npx: true,
            has_python: false,
            python_version: None,
            has_uv: false,
            has_uvx: false,
        };
        assert!(can_install(&entry, &runtimes_with_npx));

        let runtimes_without = RuntimeInfo {
            has_node: false,
            node_version: None,
            has_npx: false,
            has_python: false,
            python_version: None,
            has_uv: false,
            has_uvx: false,
        };
        assert!(!can_install(&entry, &runtimes_without));
    }

    #[test]
    fn test_build_server_entry() {
        let entry = &get_catalog()[0]; // filesystem
        let server = build_server_entry(entry, HashMap::new());
        assert_eq!(server.name, "Filesystem");
        assert_eq!(server.transport, "stdio");
        assert!(server.command.is_some());
    }

    #[test]
    fn test_categories() {
        let cats = get_categories();
        assert!(cats.len() >= 5);
        assert_eq!(cats[0].id, "all");
    }

    #[test]
    fn test_registry_to_catalog_entry_npm() {
        let server = RegistryServer {
            name: "io.github.user/my-mcp-server".into(),
            title: Some("My MCP Server".into()),
            description: "A useful database tool for querying SQL".into(),
            version: "1.0.0".into(),
            packages: vec![RegistryPackage {
                registry_type: "npm".into(),
                identifier: "@user/my-mcp-server".into(),
                version: Some("1.0.0".into()),
                transport: None,
                environment_variables: vec![RegistryEnvVar {
                    name: "API_KEY".into(),
                    description: "Your API key".into(),
                    is_required: true,
                    is_secret: true,
                    default: None,
                }],
                runtime_hint: None,
            }],
            remotes: vec![],
            icons: vec![],
            repository: None,
            website_url: None,
        };

        let entry = registry_to_catalog_entry(&server).expect("Should convert npm server");
        assert_eq!(entry.name, "My MCP Server");
        assert_eq!(entry.runtime, "node");
        assert_eq!(entry.command, "npx");
        assert_eq!(entry.args, vec!["-y", "@user/my-mcp-server"]);
        assert_eq!(entry.category, "data");
        assert_eq!(entry.required_env.len(), 1);
        assert_eq!(entry.required_env[0].name, "API_KEY");
        assert!(entry.required_env[0].sensitive);
    }

    #[test]
    fn test_registry_to_catalog_entry_remote() {
        let server = RegistryServer {
            name: "com.example/remote-tool".into(),
            title: None,
            description: "A remote search tool".into(),
            version: "2.0.0".into(),
            packages: vec![],
            remotes: vec![RegistryRemote {
                remote_type: "streamable-http".into(),
                url: "https://example.com/mcp".into(),
            }],
            icons: vec![],
            repository: None,
            website_url: None,
        };

        let entry = registry_to_catalog_entry(&server).expect("Should convert remote server");
        assert_eq!(entry.name, "Remote Tool");
        assert_eq!(entry.transport, "http");
        assert_eq!(entry.runtime, "remote");
        assert_eq!(entry.icon, "🌐");
    }

    #[test]
    fn test_registry_to_catalog_entry_no_packages_no_remotes() {
        let server = RegistryServer {
            name: "empty".into(),
            title: None,
            description: "Nothing here".into(),
            version: "0.0.0".into(),
            packages: vec![],
            remotes: vec![],
            icons: vec![],
            repository: None,
            website_url: None,
        };
        assert!(registry_to_catalog_entry(&server).is_none());
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("a+b=c"), "a%2Bb%3Dc");
        assert_eq!(
            url_encode("safe-string_123.test~ok"),
            "safe-string_123.test~ok"
        );
    }
}
