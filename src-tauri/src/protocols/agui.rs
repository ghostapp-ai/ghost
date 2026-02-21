//! AG-UI Runtime — Agent↔User Interaction Protocol for Ghost.
//!
//! Implements the AG-UI event-based protocol for real-time bidirectional
//! communication between the Ghost agent (Rust backend) and the React frontend.
//!
//! AG-UI defines 30+ event types across 7 categories:
//! - Lifecycle:      RUN_STARTED, RUN_FINISHED, RUN_ERROR, STEP_STARTED, STEP_FINISHED
//! - Text Messages:  TEXT_MESSAGE_START, TEXT_MESSAGE_CONTENT, TEXT_MESSAGE_END, TEXT_MESSAGE_CHUNK
//! - Tool Calls:     TOOL_CALL_START, TOOL_CALL_ARGS, TOOL_CALL_END, TOOL_CALL_RESULT
//! - State:          STATE_SNAPSHOT, STATE_DELTA, MESSAGES_SNAPSHOT
//! - Activity:       ACTIVITY_SNAPSHOT, ACTIVITY_DELTA
//! - Reasoning:      REASONING_START, REASONING_MESSAGE_START, REASONING_MESSAGE_CONTENT,
//!   REASONING_MESSAGE_END, REASONING_END, REASONING_ENCRYPTED_VALUE
//! - Special:        RAW, CUSTOM
//!
//! Transport: Tauri events (frontend↔backend IPC) for desktop use.
//! The protocol is also exposed via SSE on the MCP HTTP server for external clients.
//!
//! Reference: https://docs.ag-ui.com/concepts/events

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::AppState;

// ---------------------------------------------------------------------------
// AG-UI Event Types (30+ per spec)
// ---------------------------------------------------------------------------

/// AG-UI event type discriminator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    // Lifecycle events
    RunStarted,
    RunFinished,
    RunError,
    StepStarted,
    StepFinished,
    // Text message events
    TextMessageStart,
    TextMessageContent,
    TextMessageEnd,
    /// Convenience chunk: combines Start + Content (or just Content).
    TextMessageChunk,
    // Tool call events
    ToolCallStart,
    ToolCallArgs,
    ToolCallEnd,
    /// Tool result (after execution) — includes content + role.
    ToolCallResult,
    /// Convenience chunk: combines Start + Args.
    ToolCallChunk,
    // State management events
    StateSnapshot,
    StateDelta,
    /// Snapshot of all messages in the thread.
    MessagesSnapshot,
    // Activity events (agent thought annotations, citations, etc.)
    ActivitySnapshot,
    ActivityDelta,
    // Reasoning/thinking events (extended reasoning models)
    ReasoningStart,
    ReasoningMessageStart,
    ReasoningMessageContent,
    ReasoningMessageEnd,
    ReasoningEnd,
    /// Encrypted reasoning for API-level privacy.
    ReasoningEncryptedValue,
    // Special events
    Raw,
    Custom,
}

/// Base AG-UI event — all events share this structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiEvent {
    /// Event type discriminator.
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Unique run identifier.
    pub run_id: String,
    /// Optional thread identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,
    /// Event-specific payload.
    #[serde(flatten)]
    pub payload: EventPayload,
}

/// Event-specific payload variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    /// RUN_STARTED event.
    RunStarted {
        #[serde(rename = "threadId", skip_serializing_if = "Option::is_none")]
        thread_id: Option<String>,
    },
    /// RUN_FINISHED event.
    RunFinished {},
    /// RUN_ERROR event.
    RunError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
    /// STEP_STARTED event.
    StepStarted {
        #[serde(rename = "stepName")]
        step_name: String,
        #[serde(rename = "stepIndex", skip_serializing_if = "Option::is_none")]
        step_index: Option<usize>,
    },
    /// STEP_FINISHED event.
    StepFinished {
        #[serde(rename = "stepName")]
        step_name: String,
    },
    /// TEXT_MESSAGE_START — beginning of a text message stream.
    TextMessageStart {
        #[serde(rename = "messageId")]
        message_id: String,
        role: String,
    },
    /// TEXT_MESSAGE_CONTENT — a chunk of streaming text.
    TextMessageContent {
        #[serde(rename = "messageId")]
        message_id: String,
        delta: String,
    },
    /// TEXT_MESSAGE_END — end of a text message stream.
    TextMessageEnd {
        #[serde(rename = "messageId")]
        message_id: String,
    },
    /// TOOL_CALL_START — agent is invoking a tool.
    ToolCallStart {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        #[serde(rename = "toolCallName")]
        tool_call_name: String,
        #[serde(rename = "parentMessageId", skip_serializing_if = "Option::is_none")]
        parent_message_id: Option<String>,
    },
    /// TOOL_CALL_ARGS — streaming tool call arguments.
    ToolCallArgs {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        delta: String,
    },
    /// TOOL_CALL_END — tool call completed with result.
    ToolCallEnd {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },
    /// TOOL_CALL_RESULT — structured result after execution.
    ToolCallResult {
        #[serde(rename = "messageId")]
        message_id: String,
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        /// The tool result content (text or JSON).
        content: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
    },
    /// TOOL_CALL_CHUNK — convenience event combining Start + Args.
    ToolCallChunk {
        #[serde(rename = "toolCallId", skip_serializing_if = "Option::is_none")]
        tool_call_id: Option<String>,
        #[serde(rename = "toolCallName", skip_serializing_if = "Option::is_none")]
        tool_call_name: Option<String>,
        #[serde(rename = "parentMessageId", skip_serializing_if = "Option::is_none")]
        parent_message_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        delta: Option<String>,
    },
    /// STATE_SNAPSHOT — full state replacement.
    StateSnapshot { snapshot: serde_json::Value },
    /// STATE_DELTA — incremental state update (JSON Patch, RFC 6902).
    StateDelta { delta: Vec<serde_json::Value> },
    /// MESSAGES_SNAPSHOT — full snapshot of all thread messages.
    MessagesSnapshot { messages: Vec<serde_json::Value> },
    /// ACTIVITY_SNAPSHOT — agent annotation (citations, thoughts, etc.).
    ActivitySnapshot {
        #[serde(rename = "messageId")]
        message_id: String,
        #[serde(rename = "activityType")]
        activity_type: String,
        content: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        replace: Option<bool>,
    },
    /// ACTIVITY_DELTA — incremental activity update (JSON Patch).
    ActivityDelta {
        #[serde(rename = "messageId")]
        message_id: String,
        #[serde(rename = "activityType")]
        activity_type: String,
        patch: Vec<serde_json::Value>,
    },
    /// REASONING_START — beginning of a reasoning block.
    ReasoningStart {
        #[serde(rename = "messageId")]
        message_id: String,
    },
    /// REASONING_MESSAGE_START — beginning of a reasoning message stream.
    ReasoningMessageStart {
        #[serde(rename = "messageId")]
        message_id: String,
        role: String,
    },
    /// REASONING_MESSAGE_CONTENT — streaming reasoning text chunk.
    ReasoningMessageContent {
        #[serde(rename = "messageId")]
        message_id: String,
        delta: String,
    },
    /// REASONING_MESSAGE_END — end of a reasoning message stream.
    ReasoningMessageEnd {
        #[serde(rename = "messageId")]
        message_id: String,
    },
    /// REASONING_END — end of a reasoning block.
    ReasoningEnd {
        #[serde(rename = "messageId")]
        message_id: String,
    },
    /// REASONING_ENCRYPTED_VALUE — encrypted reasoning for privacy.
    ReasoningEncryptedValue {
        subtype: String,
        #[serde(rename = "entityId")]
        entity_id: String,
        #[serde(rename = "encryptedValue")]
        encrypted_value: String,
    },
    /// RAW — passthrough from external system.
    RawEvent {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        event: serde_json::Value,
    },
    /// CUSTOM — application-specific event.
    CustomEvent {
        name: String,
        value: serde_json::Value,
    },
    /// Empty payload (for simple events).
    Empty {},
}

// ---------------------------------------------------------------------------
// Event Builder (convenience constructors)
// ---------------------------------------------------------------------------

impl AgUiEvent {
    /// Get current timestamp in milliseconds.
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Create a RUN_STARTED event.
    pub fn run_started(run_id: &str) -> Self {
        Self {
            event_type: EventType::RunStarted,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::RunStarted { thread_id: None },
        }
    }

    /// Create a RUN_FINISHED event.
    pub fn run_finished(run_id: &str) -> Self {
        Self {
            event_type: EventType::RunFinished,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::RunFinished {},
        }
    }

    /// Create a RUN_ERROR event.
    pub fn run_error(run_id: &str, message: &str) -> Self {
        Self {
            event_type: EventType::RunError,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::RunError {
                message: message.to_string(),
                code: None,
            },
        }
    }

    /// Create a STEP_STARTED event.
    pub fn step_started(run_id: &str, step_name: &str, step_index: Option<usize>) -> Self {
        Self {
            event_type: EventType::StepStarted,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::StepStarted {
                step_name: step_name.to_string(),
                step_index,
            },
        }
    }

    /// Create a STEP_FINISHED event.
    pub fn step_finished(run_id: &str, step_name: &str) -> Self {
        Self {
            event_type: EventType::StepFinished,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::StepFinished {
                step_name: step_name.to_string(),
            },
        }
    }

    /// Create a TEXT_MESSAGE_START event.
    pub fn text_message_start(run_id: &str, message_id: &str, role: &str) -> Self {
        Self {
            event_type: EventType::TextMessageStart,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::TextMessageStart {
                message_id: message_id.to_string(),
                role: role.to_string(),
            },
        }
    }

    /// Create a TEXT_MESSAGE_CONTENT event (streaming delta).
    pub fn text_message_content(run_id: &str, message_id: &str, delta: &str) -> Self {
        Self {
            event_type: EventType::TextMessageContent,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::TextMessageContent {
                message_id: message_id.to_string(),
                delta: delta.to_string(),
            },
        }
    }

    /// Create a TEXT_MESSAGE_END event.
    pub fn text_message_end(run_id: &str, message_id: &str) -> Self {
        Self {
            event_type: EventType::TextMessageEnd,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::TextMessageEnd {
                message_id: message_id.to_string(),
            },
        }
    }

    /// Create a TOOL_CALL_START event.
    pub fn tool_call_start(
        run_id: &str,
        tool_call_id: &str,
        tool_name: &str,
        parent_message_id: Option<&str>,
    ) -> Self {
        Self {
            event_type: EventType::ToolCallStart,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::ToolCallStart {
                tool_call_id: tool_call_id.to_string(),
                tool_call_name: tool_name.to_string(),
                parent_message_id: parent_message_id.map(|s| s.to_string()),
            },
        }
    }

    /// Create a TOOL_CALL_ARGS event (streaming arguments).
    pub fn tool_call_args(run_id: &str, tool_call_id: &str, delta: &str) -> Self {
        Self {
            event_type: EventType::ToolCallArgs,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::ToolCallArgs {
                tool_call_id: tool_call_id.to_string(),
                delta: delta.to_string(),
            },
        }
    }

    /// Create a TOOL_CALL_END event.
    pub fn tool_call_end(run_id: &str, tool_call_id: &str, result: Option<&str>) -> Self {
        Self {
            event_type: EventType::ToolCallEnd,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::ToolCallEnd {
                tool_call_id: tool_call_id.to_string(),
                result: result.map(|s| s.to_string()),
            },
        }
    }

    /// Create a STATE_SNAPSHOT event.
    pub fn state_snapshot(run_id: &str, snapshot: serde_json::Value) -> Self {
        Self {
            event_type: EventType::StateSnapshot,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::StateSnapshot { snapshot },
        }
    }

    /// Create a STATE_DELTA event (JSON Patch).
    pub fn state_delta(run_id: &str, delta: Vec<serde_json::Value>) -> Self {
        Self {
            event_type: EventType::StateDelta,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::StateDelta { delta },
        }
    }

    /// Create a MESSAGES_SNAPSHOT event.
    pub fn messages_snapshot(run_id: &str, messages: Vec<serde_json::Value>) -> Self {
        Self {
            event_type: EventType::MessagesSnapshot,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::MessagesSnapshot { messages },
        }
    }

    /// Create a TOOL_CALL_RESULT event (structured result after execution).
    pub fn tool_call_result(
        run_id: &str,
        message_id: &str,
        tool_call_id: &str,
        content: serde_json::Value,
    ) -> Self {
        Self {
            event_type: EventType::ToolCallResult,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::ToolCallResult {
                message_id: message_id.to_string(),
                tool_call_id: tool_call_id.to_string(),
                content,
                role: Some("tool".to_string()),
            },
        }
    }

    /// Create an ACTIVITY_SNAPSHOT event (agent thought, citation, etc.).
    pub fn activity_snapshot(
        run_id: &str,
        message_id: &str,
        activity_type: &str,
        content: serde_json::Value,
    ) -> Self {
        Self {
            event_type: EventType::ActivitySnapshot,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::ActivitySnapshot {
                message_id: message_id.to_string(),
                activity_type: activity_type.to_string(),
                content,
                replace: Some(true),
            },
        }
    }

    /// Create a REASONING_START event.
    pub fn reasoning_start(run_id: &str, message_id: &str) -> Self {
        Self {
            event_type: EventType::ReasoningStart,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::ReasoningStart {
                message_id: message_id.to_string(),
            },
        }
    }

    /// Create a REASONING_MESSAGE_CONTENT event (streaming reasoning chunk).
    pub fn reasoning_content(run_id: &str, message_id: &str, delta: &str) -> Self {
        Self {
            event_type: EventType::ReasoningMessageContent,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::ReasoningMessageContent {
                message_id: message_id.to_string(),
                delta: delta.to_string(),
            },
        }
    }

    /// Create a REASONING_END event.
    pub fn reasoning_end(run_id: &str, message_id: &str) -> Self {
        Self {
            event_type: EventType::ReasoningEnd,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::ReasoningEnd {
                message_id: message_id.to_string(),
            },
        }
    }

    /// Create a CUSTOM event.
    pub fn custom(run_id: &str, name: &str, value: serde_json::Value) -> Self {
        Self {
            event_type: EventType::Custom,
            run_id: run_id.to_string(),
            thread_id: None,
            timestamp: Self::now_ms(),
            payload: EventPayload::CustomEvent {
                name: name.to_string(),
                value,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// AG-UI Event Bus (broadcast channel for real-time streaming)
// ---------------------------------------------------------------------------

/// The AG-UI event bus — distributes events to all subscribers.
///
/// Uses tokio broadcast channel for fan-out to multiple consumers:
/// - Tauri IPC events (frontend)
/// - SSE endpoint (external clients)
/// - Internal logging
pub struct AgUiEventBus {
    sender: broadcast::Sender<AgUiEvent>,
}

impl AgUiEventBus {
    /// Create a new event bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Emit an event to all subscribers.
    pub fn emit(&self, event: AgUiEvent) {
        let event_type = format!("{:?}", event.event_type);
        match self.sender.send(event) {
            Ok(n) => {
                tracing::trace!("AG-UI event emitted: {} ({} receivers)", event_type, n);
            }
            Err(_) => {
                // No subscribers — that's ok, events are fire-and-forget.
                tracing::trace!("AG-UI event emitted: {} (no receivers)", event_type);
            }
        }
    }

    /// Subscribe to the event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<AgUiEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

// ---------------------------------------------------------------------------
// AG-UI Agent Runner — orchestrates chat + tool calls as AG-UI event stream
// ---------------------------------------------------------------------------

/// An agent run — orchestrates a chat request through the AG-UI event lifecycle.
///
/// Flow:
/// 1. RUN_STARTED
/// 2. STEP_STARTED("thinking")
/// 3. TEXT_MESSAGE_START → TEXT_MESSAGE_CONTENT* → TEXT_MESSAGE_END
/// 4. [Optional] TOOL_CALL_START → TOOL_CALL_ARGS → TOOL_CALL_END
/// 5. STEP_FINISHED("thinking")
/// 6. RUN_FINISHED or RUN_ERROR
///
/// NOTE: This is the legacy simple runner (no tool calling).
/// The new `agent::executor::AgentExecutor` handles the full ReAct loop.
/// Kept as fallback for simple non-agentic chat if needed.
#[allow(dead_code)]
pub struct AgentRunner {
    state: Arc<AppState>,
}

#[allow(dead_code)]
impl AgentRunner {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Execute an agent run, emitting AG-UI events through the event bus.
    ///
    /// This is the main entry point for AG-UI chat — replaces the simple
    /// request-response `chat_send` with a streaming event-based flow.
    pub async fn run(
        &self,
        run_id: &str,
        messages: &[crate::chat::ChatMessage],
        max_tokens: usize,
        event_bus: &AgUiEventBus,
    ) -> Result<(), crate::error::GhostError> {
        let message_id = format!("msg-{}", &run_id[..8.min(run_id.len())]);

        // 1. RUN_STARTED
        event_bus.emit(AgUiEvent::run_started(run_id));

        // 2. STEP_STARTED
        event_bus.emit(AgUiEvent::step_started(run_id, "thinking", Some(0)));

        // 3. Generate response
        event_bus.emit(AgUiEvent::text_message_start(
            run_id,
            &message_id,
            "assistant",
        ));

        match self.state.chat_engine.chat(messages, max_tokens).await {
            Ok(response) => {
                // Simulate streaming by chunking the response into word groups.
                // When we add true token-by-token streaming, this will emit
                // real deltas as they come from the model.
                let words: Vec<&str> = response.content.split_whitespace().collect();
                let chunk_size = 3.max(words.len() / 20).min(10); // 3-10 words per chunk

                for chunk in words.chunks(chunk_size) {
                    let delta = if chunk == words.chunks(chunk_size).next().unwrap_or_default() {
                        chunk.join(" ")
                    } else {
                        format!(" {}", chunk.join(" "))
                    };
                    event_bus.emit(AgUiEvent::text_message_content(run_id, &message_id, &delta));
                    // Small delay to simulate streaming cadence
                    tokio::time::sleep(std::time::Duration::from_millis(15)).await;
                }

                // 4. TEXT_MESSAGE_END
                event_bus.emit(AgUiEvent::text_message_end(run_id, &message_id));

                // 5. Emit metadata as custom event
                event_bus.emit(AgUiEvent::custom(
                    run_id,
                    "generation_stats",
                    serde_json::json!({
                        "tokens_generated": response.tokens_generated,
                        "duration_ms": response.duration_ms,
                        "model_id": response.model_id,
                    }),
                ));

                // 6. STEP_FINISHED
                event_bus.emit(AgUiEvent::step_finished(run_id, "thinking"));

                // 7. RUN_FINISHED
                event_bus.emit(AgUiEvent::run_finished(run_id));

                Ok(())
            }
            Err(e) => {
                event_bus.emit(AgUiEvent::text_message_end(run_id, &message_id));
                event_bus.emit(AgUiEvent::step_finished(run_id, "thinking"));
                event_bus.emit(AgUiEvent::run_error(run_id, &e.to_string()));
                Err(e)
            }
        }
    }

    /// Execute a tool call as part of an agent run.
    #[allow(dead_code)] // Will be used when agentic tool calling is implemented
    pub async fn run_tool_call(
        &self,
        run_id: &str,
        server_name: &str,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
        event_bus: &AgUiEventBus,
    ) -> Result<String, crate::error::GhostError> {
        let tool_call_id = format!("tc-{}-{}", tool_name, &run_id[..6.min(run_id.len())]);

        // TOOL_CALL_START
        event_bus.emit(AgUiEvent::tool_call_start(
            run_id,
            &tool_call_id,
            tool_name,
            None,
        ));

        // Emit arguments if present
        if let Some(ref args) = arguments {
            let args_str = serde_json::to_string(args).unwrap_or_default();
            event_bus.emit(AgUiEvent::tool_call_args(run_id, &tool_call_id, &args_str));
        }

        // Execute the tool call
        match self
            .state
            .mcp_client
            .call_tool(server_name, tool_name, arguments)
            .await
        {
            Ok(result) => {
                event_bus.emit(AgUiEvent::tool_call_end(
                    run_id,
                    &tool_call_id,
                    Some(&result),
                ));
                Ok(result)
            }
            Err(e) => {
                event_bus.emit(AgUiEvent::tool_call_end(
                    run_id,
                    &tool_call_id,
                    Some(&format!("Error: {}", e)),
                ));
                Err(crate::error::GhostError::Chat(format!(
                    "Tool call failed: {}",
                    e
                )))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = AgUiEvent::run_started("run-123");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"RUN_STARTED\""));
        assert!(json.contains("\"runId\":\"run-123\""));
    }

    #[test]
    fn test_text_message_content_event() {
        let event = AgUiEvent::text_message_content("run-456", "msg-1", "Hello world");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"TEXT_MESSAGE_CONTENT\""));
        assert!(json.contains("\"delta\":\"Hello world\""));
        assert!(json.contains("\"messageId\":\"msg-1\""));
    }

    #[test]
    fn test_tool_call_event() {
        let event = AgUiEvent::tool_call_start("run-789", "tc-1", "ghost_search", None);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"TOOL_CALL_START\""));
        assert!(json.contains("\"toolCallName\":\"ghost_search\""));
    }

    #[test]
    fn test_custom_event() {
        let event = AgUiEvent::custom(
            "run-abc",
            "generation_stats",
            serde_json::json!({"tokens": 42}),
        );
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"CUSTOM\""));
        assert!(json.contains("\"name\":\"generation_stats\""));
    }

    #[test]
    fn test_event_bus_no_subscribers() {
        let bus = AgUiEventBus::new(32);
        // Should not panic even with no subscribers
        bus.emit(AgUiEvent::run_started("run-test"));
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_event_bus_with_subscriber() {
        let bus = AgUiEventBus::new(32);
        let mut rx = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        bus.emit(AgUiEvent::run_started("run-sub"));
        let event = rx.try_recv().unwrap();
        assert_eq!(event.event_type, EventType::RunStarted);
        assert_eq!(event.run_id, "run-sub");
    }

    #[test]
    fn test_event_type_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&EventType::RunStarted).unwrap(),
            "\"RUN_STARTED\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::TextMessageContent).unwrap(),
            "\"TEXT_MESSAGE_CONTENT\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::ToolCallStart).unwrap(),
            "\"TOOL_CALL_START\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::ReasoningStart).unwrap(),
            "\"REASONING_START\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::ActivitySnapshot).unwrap(),
            "\"ACTIVITY_SNAPSHOT\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::MessagesSnapshot).unwrap(),
            "\"MESSAGES_SNAPSHOT\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::ToolCallResult).unwrap(),
            "\"TOOL_CALL_RESULT\""
        );
    }

    #[test]
    fn test_tool_call_result_event() {
        let event = AgUiEvent::tool_call_result(
            "run-1",
            "msg-1",
            "tc-1",
            serde_json::json!({"files": ["a.txt", "b.txt"]}),
        );
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"TOOL_CALL_RESULT\""));
        assert!(json.contains("\"toolCallId\":\"tc-1\""));
        assert!(json.contains("\"role\":\"tool\""));
        assert!(json.contains("a.txt"));
    }

    #[test]
    fn test_reasoning_events() {
        let start = AgUiEvent::reasoning_start("run-1", "msg-r1");
        let start_json = serde_json::to_string(&start).unwrap();
        assert!(start_json.contains("\"type\":\"REASONING_START\""));
        assert!(start_json.contains("\"messageId\":\"msg-r1\""));

        let content = AgUiEvent::reasoning_content("run-1", "msg-r1", "hmm, thinking...");
        let content_json = serde_json::to_string(&content).unwrap();
        assert!(content_json.contains("\"type\":\"REASONING_MESSAGE_CONTENT\""));
        assert!(content_json.contains("\"delta\":\"hmm, thinking...\""));

        let end = AgUiEvent::reasoning_end("run-1", "msg-r1");
        let end_json = serde_json::to_string(&end).unwrap();
        assert!(end_json.contains("\"type\":\"REASONING_END\""));
    }

    #[test]
    fn test_activity_snapshot_event() {
        let event = AgUiEvent::activity_snapshot(
            "run-1",
            "msg-1",
            "citation",
            serde_json::json!({"source": "ghost://file.txt", "quote": "relevant text"}),
        );
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"ACTIVITY_SNAPSHOT\""));
        assert!(json.contains("\"activityType\":\"citation\""));
        assert!(json.contains("\"replace\":true"));
    }

    #[test]
    fn test_messages_snapshot_event() {
        let msgs = vec![
            serde_json::json!({"role": "user", "content": "hello"}),
            serde_json::json!({"role": "assistant", "content": "hi there"}),
        ];
        let event = AgUiEvent::messages_snapshot("run-1", msgs);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"MESSAGES_SNAPSHOT\""));
        assert!(json.contains("\"messages\""));
        assert!(json.contains("\"hello\""));
    }
}
