//! MCP Client — Ghost connects to external MCP servers.
//!
//! Ghost can act as an MCP host, connecting to external servers like:
//! - filesystem servers, GitHub servers, database servers, etc.
//! - Any of the 10,000+ MCP servers in the ecosystem.
//!
//! Supports both stdio (child process) and HTTP (streamable) transports.

use std::collections::HashMap;

use rmcp::{
    model::{CallToolRequestParams, ListToolsResult, RawContent},
    ServiceExt,
};
use tokio::sync::RwLock;

use super::McpServerEntry;

/// Represents a connected MCP server with its available tools.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectedServer {
    /// Name of the server.
    pub name: String,
    /// Whether the server is currently connected.
    pub connected: bool,
    /// Available tools from this server.
    pub tools: Vec<ToolInfo>,
    /// Transport type used.
    pub transport: String,
    /// Error message if connection failed.
    pub error: Option<String>,
}

/// Information about an MCP tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInfo {
    /// Tool name.
    pub name: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// JSON schema for input parameters.
    pub input_schema: Option<serde_json::Value>,
}

/// A running MCP client service handle.
type McpClientService = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;

/// Manager for all external MCP server connections.
pub struct McpClientManager {
    /// Map of server name → running service.
    services: RwLock<HashMap<String, McpClientService>>,
    /// Cached server info (for quick frontend queries).
    server_info: RwLock<HashMap<String, ConnectedServer>>,
}

impl McpClientManager {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            server_info: RwLock::new(HashMap::new()),
        }
    }

    /// Connect to an MCP server via stdio transport (child process).
    /// Desktop only — spawning child processes is not supported on mobile platforms.
    #[cfg(desktop)]
    pub async fn connect_stdio(&self, entry: &McpServerEntry) -> anyhow::Result<ConnectedServer> {
        let command = entry
            .command
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No command specified for stdio transport"))?;

        tracing::info!(
            "MCP Client: connecting to '{}' via stdio ({})",
            entry.name,
            command
        );

        let mut cmd = tokio::process::Command::new(command);
        for arg in &entry.args {
            cmd.arg(arg);
        }
        for (key, value) in &entry.env {
            cmd.env(key, value);
        }

        let transport = rmcp::transport::TokioChildProcess::new(cmd)?;
        let service = ().serve(transport).await?;

        // Discover tools
        let tools_result = service.list_tools(Default::default()).await?;
        let tools = extract_tools(&tools_result);

        let info = ConnectedServer {
            name: entry.name.clone(),
            connected: true,
            tools: tools.clone(),
            transport: "stdio".to_string(),
            error: None,
        };

        // Store handles
        self.services
            .write()
            .await
            .insert(entry.name.clone(), service);
        self.server_info
            .write()
            .await
            .insert(entry.name.clone(), info.clone());

        tracing::info!(
            "MCP Client: connected to '{}' — {} tools available",
            entry.name,
            tools.len()
        );

        Ok(info)
    }

    /// Connect to an MCP server via HTTP (streamable HTTP) transport.
    pub async fn connect_http(&self, entry: &McpServerEntry) -> anyhow::Result<ConnectedServer> {
        let url = entry
            .url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No URL specified for HTTP transport"))?;

        tracing::info!(
            "MCP Client: connecting to '{}' via HTTP ({})",
            entry.name,
            url
        );

        let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(url.as_str());
        let service = ().serve(transport).await?;

        // Discover tools
        let tools_result = service.list_tools(Default::default()).await?;
        let tools = extract_tools(&tools_result);

        let info = ConnectedServer {
            name: entry.name.clone(),
            connected: true,
            tools: tools.clone(),
            transport: "http".to_string(),
            error: None,
        };

        self.services
            .write()
            .await
            .insert(entry.name.clone(), service);
        self.server_info
            .write()
            .await
            .insert(entry.name.clone(), info.clone());

        tracing::info!(
            "MCP Client: connected to '{}' — {} tools available",
            entry.name,
            tools.len()
        );

        Ok(info)
    }

    /// Connect to an MCP server based on its configuration entry.
    pub async fn connect(&self, entry: &McpServerEntry) -> ConnectedServer {
        if !entry.enabled {
            return ConnectedServer {
                name: entry.name.clone(),
                connected: false,
                tools: vec![],
                transport: entry.transport.clone(),
                error: Some("Server disabled".to_string()),
            };
        }

        let result = match entry.transport.as_str() {
            #[cfg(desktop)]
            "stdio" => self.connect_stdio(entry).await,
            #[cfg(mobile)]
            "stdio" => Err(anyhow::anyhow!(
                "Stdio transport not available on this platform. Use HTTP transport instead."
            )),
            "http" | "streamable-http" => self.connect_http(entry).await,
            other => Err(anyhow::anyhow!("Unknown transport: {}", other)),
        };

        match result {
            Ok(info) => info,
            Err(e) => {
                let error_msg = format!("{}", e);
                tracing::warn!(
                    "MCP Client: failed to connect to '{}': {}",
                    entry.name,
                    error_msg
                );
                ConnectedServer {
                    name: entry.name.clone(),
                    connected: false,
                    tools: vec![],
                    transport: entry.transport.clone(),
                    error: Some(error_msg),
                }
            }
        }
    }

    /// Call a tool on a connected MCP server.
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> anyhow::Result<String> {
        let services = self.services.read().await;
        let service = services
            .get(server_name)
            .ok_or_else(|| anyhow::anyhow!("Server '{}' not connected", server_name))?;

        tracing::info!(
            "MCP Client: calling tool '{}' on server '{}'",
            tool_name,
            server_name
        );

        let result = service
            .call_tool(CallToolRequestParams {
                meta: None,
                name: tool_name.to_string().into(),
                arguments: arguments.and_then(|v| v.as_object().cloned()),
                task: None,
            })
            .await?;

        // Extract text content from the result
        let text: String = result
            .content
            .iter()
            .filter_map(|c| {
                if let RawContent::Text(tc) = &c.raw {
                    Some(tc.text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }

    /// Disconnect from a specific MCP server.
    pub async fn disconnect(&self, server_name: &str) -> anyhow::Result<()> {
        if let Some(service) = self.services.write().await.remove(server_name) {
            let _ = service.cancel().await;
            tracing::info!("MCP Client: disconnected from '{}'", server_name);
        }
        if let Some(info) = self.server_info.write().await.get_mut(server_name) {
            info.connected = false;
        }
        Ok(())
    }

    /// Disconnect all connected servers.
    pub async fn disconnect_all(&self) {
        let mut services = self.services.write().await;
        for (name, service) in services.drain() {
            let _ = service.cancel().await;
            tracing::info!("MCP Client: disconnected from '{}'", name);
        }
        let mut info = self.server_info.write().await;
        for (_, server) in info.iter_mut() {
            server.connected = false;
        }
    }

    /// Get status of all known servers.
    pub async fn list_servers(&self) -> Vec<ConnectedServer> {
        self.server_info.read().await.values().cloned().collect()
    }

    /// Get all available tools from all connected servers.
    pub async fn all_tools(&self) -> Vec<(String, ToolInfo)> {
        let info = self.server_info.read().await;
        let mut tools = Vec::new();
        for (server_name, server) in info.iter() {
            if server.connected {
                for tool in &server.tools {
                    tools.push((server_name.clone(), tool.clone()));
                }
            }
        }
        tools
    }
}

/// Extract tool information from an MCP ListToolsResult.
fn extract_tools(result: &ListToolsResult) -> Vec<ToolInfo> {
    result
        .tools
        .iter()
        .map(|t| ToolInfo {
            name: t.name.to_string(),
            description: t.description.as_ref().map(|s| s.to_string()),
            input_schema: serde_json::to_value(&t.input_schema).ok(),
        })
        .collect()
}
