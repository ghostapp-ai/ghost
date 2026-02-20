//! MCP Server — Ghost exposes its tools to external AI clients.
//!
//! When running, Claude Desktop, Cursor, VS Code Copilot, and any MCP-compatible
//! client can connect to Ghost and use its local search, indexing, and file tools.
//!
//! Transport: Streamable HTTP on localhost (configurable port).
//! Protocol: MCP v2025-11-25 via `rmcp` crate.

use std::sync::Arc;

use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::AppState;

// ---------------------------------------------------------------------------
// Tool parameter/result types
// ---------------------------------------------------------------------------

/// Parameters for the `ghost_search` tool.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchParams {
    /// The search query (keywords or natural language).
    pub query: String,
    /// Maximum number of results to return (default: 10).
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

/// A single search result returned by `ghost_search`.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchResultItem {
    /// Absolute path to the file.
    pub path: String,
    /// Filename with extension.
    pub filename: String,
    /// Relevant text snippet from the document.
    pub snippet: String,
    /// Relevance score (higher is better).
    pub score: f64,
    /// Result source: "fts", "vector", or "hybrid".
    pub source: String,
}

/// Parameters for the `ghost_recent_files` tool.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RecentFilesParams {
    /// Maximum number of recent files to return (default: 20).
    #[serde(default = "default_recent_limit")]
    pub limit: usize,
}

fn default_recent_limit() -> usize {
    20
}

/// A recently indexed file entry.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)]
pub struct RecentFileItem {
    /// Absolute path to the file.
    pub path: String,
    /// Filename with extension.
    pub filename: String,
    /// File extension (e.g., "pdf", "md").
    pub extension: Option<String>,
    /// File size in bytes.
    pub size_bytes: i64,
    /// When the file was last indexed (ISO 8601).
    pub indexed_at: String,
}

/// Index status information.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexStatusResult {
    /// Total number of indexed documents.
    pub document_count: i64,
    /// Total number of text chunks.
    pub chunk_count: i64,
    /// Number of chunks with vector embeddings.
    pub embedded_chunk_count: i64,
    /// Whether vector search (sqlite-vec) is available.
    pub vector_search_enabled: bool,
    /// Currently watched directories.
    pub watched_directories: Vec<String>,
}

// ---------------------------------------------------------------------------
// Ghost MCP Server Handler
// ---------------------------------------------------------------------------

/// The Ghost MCP Server handler — exposes search, indexing, and file tools.
#[derive(Clone)]
pub struct GhostMcpServer {
    state: Arc<AppState>,
    tool_router: ToolRouter<Self>,
}

impl GhostMcpServer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl GhostMcpServer {
    /// Search across all indexed local files using hybrid keyword + semantic search.
    /// Returns matching documents with snippets and relevance scores.
    #[tool(
        name = "ghost_search",
        description = "Search across locally indexed files using hybrid keyword + semantic search. Returns matching documents with relevant text snippets and scores."
    )]
    async fn ghost_search(
        &self,
        params: Parameters<SearchParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let query = &params.0.query;
        let limit = params.0.limit;

        tracing::info!("MCP ghost_search: query='{}', limit={}", query, limit);

        match crate::search::hybrid_search(
            &self.state.db,
            &self.state.embedding_engine,
            query,
            limit,
        )
        .await
        {
            Ok(results) => {
                let items: Vec<SearchResultItem> = results
                    .iter()
                    .map(|r| SearchResultItem {
                        path: r.path.clone(),
                        filename: r.filename.clone(),
                        snippet: r.snippet.clone(),
                        score: r.score,
                        source: r.source.clone(),
                    })
                    .collect();

                let json = serde_json::to_string_pretty(&items).unwrap_or_default();
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => {
                tracing::error!("MCP ghost_search error: {}", e);
                Err(rmcp::ErrorData::internal_error(
                    format!("Search failed: {}", e),
                    None,
                ))
            }
        }
    }

    /// Get the current indexing status: document count, chunk count, vector status,
    /// and watched directories.
    #[tool(
        name = "ghost_index_status",
        description = "Get Ghost's current indexing status including document count, chunk count, vector search availability, and watched directories."
    )]
    async fn ghost_index_status(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tracing::info!("MCP ghost_index_status");

        let stats =
            self.state.db.get_stats().map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Stats error: {}", e), None)
            })?;

        let watched = self
            .state
            .settings
            .lock()
            .map(|s| s.watched_directories.clone())
            .unwrap_or_default();

        let result = IndexStatusResult {
            document_count: stats.document_count,
            chunk_count: stats.chunk_count,
            embedded_chunk_count: stats.embedded_chunk_count,
            vector_search_enabled: self.state.db.is_vec_enabled(),
            watched_directories: watched,
        };

        let json = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// List recently indexed files, ordered by indexing time (most recent first).
    #[tool(
        name = "ghost_recent_files",
        description = "List recently indexed files ordered by indexing time. Returns file paths, names, sizes, and timestamps."
    )]
    async fn ghost_recent_files(
        &self,
        params: Parameters<RecentFilesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = params.0.limit;
        tracing::info!("MCP ghost_recent_files: limit={}", limit);

        let recent = self.state.db.get_recent_documents(limit).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Recent files error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&recent).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

/// Implement the MCP ServerHandler trait for Ghost.
#[tool_handler]
impl ServerHandler for GhostMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "ghost".into(),
                title: Some("Ghost — Agent OS".into()),
                version: env!("CARGO_PKG_VERSION").into(),
                description: Some(
                    "Private, local-first Agent OS for desktop. \
                     Indexes local files and provides hybrid semantic + keyword search."
                        .into(),
                ),
                icons: None,
                website_url: Some("https://github.com/ghostapp-ai/ghost".into()),
            },
            instructions: Some(
                "Ghost is a private, local-first Agent OS for desktop. \
                 It indexes local files and provides hybrid semantic + keyword search. \
                 Use ghost_search to find documents, ghost_index_status to check indexing progress, \
                 and ghost_recent_files to see recently indexed files."
                    .into(),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Server startup
// ---------------------------------------------------------------------------

/// Start the Ghost MCP server on the given address.
/// Returns the address string on success.
pub async fn start_server(state: Arc<AppState>, addr: &str) -> anyhow::Result<String> {
    let addr_owned = addr.to_string();

    let handler = GhostMcpServer::new(state.clone());

    // Build the streamable HTTP service
    let service = rmcp::transport::streamable_http_server::StreamableHttpService::new(
        move || Ok(handler.clone()),
        rmcp::transport::streamable_http_server::session::local::LocalSessionManager::default()
            .into(),
        Default::default(),
    );

    // AG-UI SSE endpoint: streams AG-UI events to external clients
    let agui_state = state.clone();
    let agui_sse_handler = axum::routing::get(move || {
        let bus = &agui_state.agui_event_bus;
        let rx = bus.subscribe();
        async move {
            let stream = async_stream::stream! {
                let mut rx = rx;
                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            if let Ok(json) = serde_json::to_string(&event) {
                                yield Ok::<_, std::convert::Infallible>(
                                    axum::response::sse::Event::default()
                                        .event(format!("{:?}", event.event_type))
                                        .data(json)
                                );
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("AG-UI SSE client lagged by {} events", n);
                            yield Ok::<_, std::convert::Infallible>(
                                axum::response::sse::Event::default()
                                    .event("error")
                                    .data(format!("{{\"lagged\":{}}}", n))
                            );
                        }
                    }
                }
            };
            axum::response::sse::Sse::new(stream)
                .keep_alive(axum::response::sse::KeepAlive::default())
        }
    });

    let router = axum::Router::new()
        .route("/mcp", axum::routing::any_service(service))
        .route("/agui", agui_sse_handler);

    let listener = tokio::net::TcpListener::bind(&addr_owned).await?;
    let actual_addr = listener.local_addr()?;
    let addr_str = actual_addr.to_string();

    tracing::info!("MCP server listening on http://{}/mcp", addr_str);
    tracing::info!("AG-UI SSE endpoint on http://{}/agui", addr_str);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            tracing::error!("MCP server error: {}", e);
        }
    });

    Ok(addr_str)
}
