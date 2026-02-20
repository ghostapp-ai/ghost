//! Ghost Agent Engine — The intelligent core of Ghost OS.
//!
//! Implements a ReAct (Reason + Act) agent loop that:
//! 1. Accepts user messages with conversation context
//! 2. Detects available tools (built-in + MCP external)
//! 3. Constructs system prompts with tool schemas
//! 4. Sends to native LLM (llama.cpp via llama-cpp-2) with grammar-constrained tool calling
//! 5. Parses tool call responses using llama.cpp's built-in chat template parser
//! 6. Feeds results back to LLM for next iteration
//! 7. Streams all events via AG-UI protocol
//!
//! Fully native — ZERO external dependencies (no Ollama, no server, no network).
//! Uses the SAME Qwen2.5-Instruct GGUF models already used for chat,
//! with Hermes 2 Pro tool-calling format + GBNF grammar-constrained generation.
//!
//! Hardware-adaptive: auto-selects the best local GGUF model based on
//! detected RAM/VRAM, with user-configurable overrides.

pub mod config;
pub mod executor;
pub mod memory;
pub mod safety;
pub mod skills;
pub mod tools;

use serde::{Deserialize, Serialize};

/// A tool call parsed from the LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool function details.
    pub function: ToolCallFunction,
}

/// Tool call function with name and arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    /// The name of the tool to call.
    pub name: String,
    /// JSON arguments for the tool.
    pub arguments: serde_json::Value,
}

/// Tool definition in OpenAI-compatible format.
///
/// Used by llama.cpp's `apply_chat_template_with_tools_oaicompat` to generate
/// model-specific tool-calling prompts with grammar constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTool {
    /// Always "function".
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition.
    pub function: AgentToolFunction,
}

/// Function definition within a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolFunction {
    /// Tool name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for parameters.
    pub parameters: serde_json::Value,
}

/// A message in the agent conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatMessage {
    /// Role: "system", "user", "assistant", or "tool".
    pub role: String,
    /// Text content (empty string for tool-call-only responses).
    #[serde(default)]
    pub content: String,
    /// Tool calls made by the assistant (present in assistant messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Result of an agent execution run.
#[derive(Debug, Clone, Serialize)]
pub struct AgentRunResult {
    /// Final text response from the agent.
    pub content: String,
    /// Number of ReAct iterations performed.
    pub iterations: usize,
    /// Tool calls executed during this run.
    pub tool_calls_executed: Vec<ExecutedToolCall>,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    /// Model used for this run.
    pub model: String,
}

/// Record of a tool call that was executed.
#[derive(Debug, Clone, Serialize)]
pub struct ExecutedToolCall {
    /// Tool name.
    pub name: String,
    /// Arguments passed.
    pub arguments: serde_json::Value,
    /// Result returned.
    pub result: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Risk level assessed by safety layer.
    pub risk_level: safety::RiskLevel,
}
