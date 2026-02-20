//! Protocol Hub — Ghost's universal protocol integration layer.
//!
//! Ghost speaks multiple agent protocols, making it a Universal Protocol Hub:
//! - **MCP Server**: Expose Ghost tools (search, index, stats) to external AI clients
//! - **MCP Client**: Connect to external MCP servers (filesystem, GitHub, databases, etc.)
//! - **AG-UI**: Agent↔User interaction streaming (Phase 1.5+)
//! - **A2UI**: Generative UI from JSON schemas (Phase 1.5+)
//! - **A2A**: Agent-to-Agent coordination (Phase 2)
//! - **WebMCP**: Browser tool contracts (Phase 2.5)

pub mod mcp_client;
pub mod mcp_server;

use std::sync::Arc;

use crate::AppState;

/// MCP Server configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerConfig {
    /// Whether the MCP server is enabled.
    #[serde(default = "default_mcp_enabled")]
    pub enabled: bool,
    /// Port for the MCP HTTP server (default: 6774 — "GHST" on phone keypad).
    #[serde(default = "default_mcp_port")]
    pub port: u16,
    /// Hostname to bind (default: 127.0.0.1 for security).
    #[serde(default = "default_mcp_host")]
    pub host: String,
}

fn default_mcp_enabled() -> bool {
    true
}
fn default_mcp_port() -> u16 {
    6774
}
fn default_mcp_host() -> String {
    "127.0.0.1".into()
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            enabled: default_mcp_enabled(),
            port: default_mcp_port(),
            host: default_mcp_host(),
        }
    }
}

/// Configuration for an external MCP server connection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerEntry {
    /// Display name for this server.
    pub name: String,
    /// Transport type: "stdio" or "http".
    pub transport: String,
    /// For stdio: the command to execute (e.g., "npx", "uvx", "python").
    pub command: Option<String>,
    /// For stdio: arguments to the command.
    #[serde(default)]
    pub args: Vec<String>,
    /// For http: the server URL (e.g., "http://localhost:8000/mcp").
    pub url: Option<String>,
    /// Whether this server connection is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Environment variables to pass to stdio processes.
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

/// Start the MCP server in the background.
/// Returns the address it's listening on.
pub async fn start_mcp_server(
    state: Arc<AppState>,
    config: &McpServerConfig,
) -> anyhow::Result<String> {
    if !config.enabled {
        tracing::info!("MCP server disabled in settings");
        return Ok("disabled".to_string());
    }

    let addr = format!("{}:{}", config.host, config.port);
    mcp_server::start_server(state, &addr).await
}
