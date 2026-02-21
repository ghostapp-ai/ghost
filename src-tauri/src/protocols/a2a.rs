//! A2A Protocol — Agent-to-Agent coordination for Ghost.
//!
//! Implements Google's Agent2Agent (A2A) protocol, enabling Ghost to:
//! - **Act as an A2A Server**: Publish an Agent Card, accept tasks from other agents
//! - **Act as an A2A Client**: Discover remote agents and delegate tasks to them
//!
//! ## Protocol Overview
//! A2A uses JSON-RPC 2.0 over HTTP(S) with optional SSE streaming.
//!
//! ### Agent Card (published at `GET /.well-known/agent.json`)
//! Describes Ghost's identity, capabilities, and skills.
//!
//! ### Core JSON-RPC Methods
//! - `message/send` (SendMessage) — Initiate a task, returns Task or Message
//! - `message/stream` (SendStreamingMessage) — Send task with SSE streaming
//! - `tasks/get` (GetTask) — Retrieve task status and history
//! - `tasks/cancel` (CancelTask) — Cancel an in-progress task
//! - `tasks/list` (ListTasks) — Query tasks with filtering
//! - `tasks/pushNotificationConfig/set` — Register webhook for async updates
//!
//! ### Task Lifecycle
//! ```text
//! WORKING → COMPLETED
//!         → FAILED
//!         → CANCELED
//!         → REJECTED
//!         → INPUT_REQUIRED  (awaiting client input)
//!         → AUTH_REQUIRED   (awaiting authentication)
//! ```
//!
//! Reference: https://a2a-protocol.org/latest/specification/
//! Spec version: v0.3.0 (with gRPC + signed cards)

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Agent Card (/.well-known/agent.json)
// ---------------------------------------------------------------------------

/// Ghost's A2A Agent Card — published at `/.well-known/agent.json`.
///
/// Describes Ghost's identity, capabilities, and available skills.
/// External agents use this to discover what Ghost can do.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCard {
    /// Unique identifier for this agent instance.
    pub id: String,
    /// Display name: "Ghost".
    pub display_name: String,
    /// Short description of what Ghost does.
    pub description: String,
    /// Ghost version.
    pub version: String,
    /// Provider information.
    pub provider: AgentProvider,
    /// Supported A2A capabilities.
    pub capabilities: AgentCapabilities,
    /// List of skills this agent offers.
    pub skills: Vec<AgentSkill>,
    /// Accepted input modes (e.g., "text/plain", "application/json").
    #[serde(default)]
    pub default_input_modes: Vec<String>,
    /// Accepted output modes.
    #[serde(default)]
    pub default_output_modes: Vec<String>,
    /// Supported protocol interfaces.
    #[serde(default)]
    pub interfaces: Vec<AgentInterface>,
}

/// Agent provider information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProvider {
    /// Organization name.
    pub organization: String,
    /// Provider URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Agent capabilities declaration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    /// Whether this agent supports SSE streaming.
    #[serde(default)]
    pub streaming: bool,
    /// Whether this agent supports push notification webhooks.
    #[serde(default)]
    pub push_notifications: bool,
    /// Whether this agent exposes an extended (authenticated) agent card.
    #[serde(default)]
    pub extended_agent_card: bool,
}

/// A skill offered by this agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkill {
    /// Unique skill identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// What this skill does.
    pub description: String,
    /// Tags for discoverability.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Example inputs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
}

/// Protocol interface supported by this agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInterface {
    /// Protocol type (e.g., "a2a", "mcp", "agui").
    pub protocol: String,
    /// Endpoint URL.
    pub url: String,
}

/// Build Ghost's default Agent Card.
pub fn ghost_agent_card(base_url: &str) -> AgentCard {
    AgentCard {
        id: "ghost-local".to_string(),
        display_name: "Ghost".to_string(),
        description: "A private, local-first AI agent. Searches your files, browses the web, and runs tools — all on your machine without sending data to the cloud.".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        provider: AgentProvider {
            organization: "ghostapp-ai".to_string(),
            url: Some("https://github.com/ghostapp-ai/ghost".to_string()),
        },
        capabilities: AgentCapabilities {
            streaming: true,
            push_notifications: false,
            extended_agent_card: false,
        },
        skills: vec![
            AgentSkill {
                id: "file-search".to_string(),
                name: "File Search".to_string(),
                description: "Hybrid semantic + keyword search across locally indexed files. Finds documents, code, notes, and more.".to_string(),
                tags: vec!["search".to_string(), "files".to_string(), "local".to_string()],
                examples: vec![
                    "Find my notes about React hooks".to_string(),
                    "Search for documents containing 'quarterly report'".to_string(),
                ],
            },
            AgentSkill {
                id: "file-read".to_string(),
                name: "Read File".to_string(),
                description: "Read the contents of any locally indexed file.".to_string(),
                tags: vec!["files".to_string(), "read".to_string(), "local".to_string()],
                examples: vec![
                    "Read ~/Documents/notes.md".to_string(),
                ],
            },
            AgentSkill {
                id: "shell-command".to_string(),
                name: "Run Command".to_string(),
                description: "Execute shell commands on the local system (with safety checks).".to_string(),
                tags: vec!["shell".to_string(), "execute".to_string(), "local".to_string()],
                examples: vec![
                    "Run git status in ~/repos/myproject".to_string(),
                    "List files in ~/Downloads".to_string(),
                ],
            },
            AgentSkill {
                id: "mcp-tools".to_string(),
                name: "MCP Tools".to_string(),
                description: "Access any of the connected MCP servers — GitHub, filesystem, databases, and more.".to_string(),
                tags: vec!["mcp".to_string(), "tools".to_string(), "extendable".to_string()],
                examples: vec![
                    "Search GitHub for issues mentioning 'memory leak'".to_string(),
                    "Query the local SQLite database".to_string(),
                ],
            },
        ],
        default_input_modes: vec!["text/plain".to_string(), "application/json".to_string()],
        default_output_modes: vec!["text/plain".to_string(), "application/json".to_string()],
        interfaces: vec![
            AgentInterface {
                protocol: "a2a".to_string(),
                url: format!("{}/a2a", base_url),
            },
            AgentInterface {
                protocol: "mcp".to_string(),
                url: format!("{}/mcp", base_url),
            },
            AgentInterface {
                protocol: "agui".to_string(),
                url: format!("{}/agui", base_url),
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Task Lifecycle
// ---------------------------------------------------------------------------

/// A2A task state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskState {
    /// Task is being actively processed.
    Working,
    /// Task completed successfully.
    Completed,
    /// Task failed with an error.
    Failed,
    /// Task was canceled by the client.
    Canceled,
    /// Agent declined to process the task.
    Rejected,
    /// Awaiting additional input from the client.
    InputRequired,
    /// Awaiting authentication credentials.
    AuthRequired,
}

/// A2A task — the fundamental unit of work.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Unique task identifier.
    pub id: String,
    /// Context identifier (groups related tasks).
    pub context_id: String,
    /// Current task state.
    pub status: TaskStatus,
    /// History of messages exchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<A2aMessage>,
    /// Output artifacts produced by this task.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<Artifact>,
    /// Arbitrary metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task status with optional message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub state: TaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<A2aMessage>,
    pub timestamp: String,
}

/// An A2A message (input or output).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2aMessage {
    /// Unique message ID (for idempotency).
    pub message_id: String,
    /// "user" or "agent".
    pub role: String,
    /// Message parts (text, files, data).
    pub parts: Vec<Part>,
    /// Optional context ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    /// Optional task ID this message belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

/// A single part of a multi-part message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    /// Plain text.
    Text { text: String },
    /// Structured data (JSON).
    Data { data: serde_json::Value },
    /// File attachment.
    File { file: FileRef },
}

/// File reference in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRef {
    /// File name.
    pub name: String,
    /// MIME type.
    pub mime_type: String,
    /// Inline bytes (base64) or URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// Task output artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    /// Artifact ID.
    pub artifact_id: String,
    /// Display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Parts making up this artifact.
    pub parts: Vec<Part>,
}

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 Types
// ---------------------------------------------------------------------------

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Standard A2A JSON-RPC error codes.
pub mod error_codes {
    pub const TASK_NOT_FOUND: i32 = -32001;
    pub const TASK_NOT_CANCELABLE: i32 = -32002;
    pub const PUSH_NOTIFICATION_NOT_SUPPORTED: i32 = -32003;
    pub const UNSUPPORTED_OPERATION: i32 = -32004;
    pub const CONTENT_TYPE_NOT_SUPPORTED: i32 = -32005;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn ok(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<serde_json::Value>, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// SendMessage params
// ---------------------------------------------------------------------------

/// Parameters for `message/send` and `message/stream`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageParams {
    /// The message to send.
    pub message: A2aMessage,
    /// Optional configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<SendMessageConfig>,
}

/// Configuration for SendMessage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageConfig {
    /// If true, wait until task reaches terminal state before returning.
    #[serde(default)]
    pub blocking: bool,
    /// Max messages to include in history (0 = none, unset = default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_length: Option<usize>,
    /// Accepted output MIME types.
    #[serde(default)]
    pub accepted_output_modes: Vec<String>,
}

// ---------------------------------------------------------------------------
// A2A Method Dispatcher (server side)
// ---------------------------------------------------------------------------

/// Dispatch an incoming A2A JSON-RPC request to the appropriate handler.
///
/// Currently returns "Method Not Implemented" — full implementation is Phase 2.
pub async fn dispatch_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();
    match request.method.as_str() {
        "message/send" | "message/stream" => {
            // Phase 2: wire up to the agent executor
            JsonRpcResponse::error(
                id,
                error_codes::UNSUPPORTED_OPERATION,
                "A2A message handling is planned for Phase 2",
            )
        }
        "tasks/get" | "tasks/list" | "tasks/cancel" => JsonRpcResponse::error(
            id,
            error_codes::TASK_NOT_FOUND,
            "Task management is planned for Phase 2",
        ),
        "tasks/pushNotificationConfig/set"
        | "tasks/pushNotificationConfig/get"
        | "tasks/pushNotificationConfig/list"
        | "tasks/pushNotificationConfig/delete" => JsonRpcResponse::error(
            id,
            error_codes::PUSH_NOTIFICATION_NOT_SUPPORTED,
            "Push notifications are not yet supported",
        ),
        _ => JsonRpcResponse::error(
            request.id,
            error_codes::UNSUPPORTED_OPERATION,
            &format!("Unknown method: {}", request.method),
        ),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_card_serializes() {
        let card = ghost_agent_card("http://localhost:6774");
        let json = serde_json::to_string_pretty(&card).unwrap();
        assert!(json.contains("\"displayName\": \"Ghost\""));
        assert!(json.contains("\"streaming\": true"));
        assert!(json.contains("file-search"));
        assert!(json.contains("mcp-tools"));
    }

    #[test]
    fn test_agent_card_interfaces() {
        let card = ghost_agent_card("http://localhost:6774");
        let a2a_if = card.interfaces.iter().find(|i| i.protocol == "a2a");
        assert!(a2a_if.is_some());
        assert_eq!(a2a_if.unwrap().url, "http://localhost:6774/a2a");
    }

    #[test]
    fn test_task_state_serialization() {
        assert_eq!(
            serde_json::to_string(&TaskState::Working).unwrap(),
            "\"WORKING\""
        );
        assert_eq!(
            serde_json::to_string(&TaskState::Completed).unwrap(),
            "\"COMPLETED\""
        );
        assert_eq!(
            serde_json::to_string(&TaskState::InputRequired).unwrap(),
            "\"INPUT_REQUIRED\""
        );
    }

    #[test]
    fn test_jsonrpc_ok_response() {
        let resp = JsonRpcResponse::ok(
            Some(serde_json::json!(1)),
            serde_json::json!({"status": "ok"}),
        );
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_jsonrpc_error_response() {
        let resp = JsonRpcResponse::error(
            Some(serde_json::json!(1)),
            error_codes::TASK_NOT_FOUND,
            "Task not found",
        );
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32001"));
        assert!(!json.contains("\"result\""));
    }

    #[tokio::test]
    async fn test_dispatch_unknown_method() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "unknown/method".to_string(),
            params: None,
        };
        let resp = dispatch_request(req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, error_codes::UNSUPPORTED_OPERATION);
    }
}
