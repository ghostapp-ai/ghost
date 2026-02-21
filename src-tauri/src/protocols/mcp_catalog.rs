//! MCP Tool Catalog — Curated app store for one-click MCP server installation.
//!
//! Provides:
//! - A built-in catalog of 30+ popular MCP servers (no network required)
//! - Runtime detection (npx, node, uv, uvx, python3)
//! - One-click install: auto-configures and connects MCP servers
//!
//! Inspired by Claude Desktop Extensions, Smithery.ai, and Cursor's MCP marketplace.
//! The goal is to make installing MCP tools as easy as installing an app from an app store.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
        url: None,
        enabled: true,
        env: env_vars,
    }
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
}
