//! ReAct Executor — Reason + Act agent loop with NATIVE llama.cpp inference.
//!
//! Implements the core agent loop:
//! 1. Build system prompt with tool schemas + skill instructions
//! 2. Apply model's chat template with grammar-constrained tool calling
//! 3. Run native llama.cpp inference with GBNF grammar constraints
//! 4. Parse tool calls using llama.cpp's built-in response parser
//! 5. Execute tools → feed results back → repeat until text-only response
//! 6. Stream everything via AG-UI events
//!
//! **Fully native** — ZERO external dependencies (no Ollama, no server, no network).
//! Uses the SAME Qwen2.5-Instruct GGUF models from the chat model registry,
//! with Hermes 2 Pro tool-calling format + GBNF grammar-constrained generation.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;

use serde::Deserialize;
use serde_json::json;

#[cfg(desktop)]
use llama_cpp_2::context::params::LlamaContextParams;
#[cfg(desktop)]
use llama_cpp_2::llama_batch::LlamaBatch;
#[cfg(desktop)]
use llama_cpp_2::model::params::LlamaModelParams;
#[cfg(desktop)]
use llama_cpp_2::model::{AddBos, ChatTemplateResult, LlamaChatMessage, LlamaModel};
#[cfg(desktop)]
use llama_cpp_2::sampling::LlamaSampler;

use super::config::{self, AgentConfig};
use super::safety::{self, RiskLevel};
use super::tools::{self, RegisteredTool};
use super::{AgentChatMessage, AgentRunResult, ExecutedToolCall, ToolCall, ToolCallFunction};
use crate::chat::ChatMessage;
use crate::error::GhostError;
use crate::protocols::agui::{AgUiEvent, AgUiEventBus};
use crate::AppState;

/// Default sampling parameters for agent inference.
const AGENT_TOP_P: f32 = 0.9;
const AGENT_SEED: u32 = 42;
const BATCH_SIZE: u32 = 512;

/// Parsed LLM response — either text, tool calls, or both.
struct LlmResponse {
    content: String,
    tool_calls: Vec<ToolCall>,
}

/// OpenAI-compatible parsed message (from `parse_response_oaicompat`).
#[derive(Debug, Deserialize)]
struct ParsedOaiMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ParsedToolCall>>,
}

#[derive(Debug, Deserialize)]
struct ParsedToolCall {
    function: ParsedToolCallFunction,
}

#[derive(Debug, Deserialize)]
struct ParsedToolCallFunction {
    name: String,
    arguments: serde_json::Value,
}

/// The agent executor — runs ReAct loops with native tool calling.
pub struct AgentExecutor {
    state: Arc<AppState>,
}

impl AgentExecutor {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Execute an agent run with the ReAct loop.
    ///
    /// Streams AG-UI events through the event bus for real-time frontend updates.
    /// Persists messages to conversation memory if `conversation_id` is provided.
    pub async fn run(
        &self,
        run_id: &str,
        messages: &[ChatMessage],
        conversation_id: Option<i64>,
        event_bus: &AgUiEventBus,
    ) -> Result<AgentRunResult, GhostError> {
        let start = Instant::now();

        // Get agent config
        let agent_config = self
            .state
            .settings
            .lock()
            .map(|s| s.agent_config.clone())
            .unwrap_or_default();

        // Resolve model
        let (model_id, context_window) =
            config::resolve_agent_model(&agent_config, &self.state.hardware);

        tracing::info!(
            "Agent run {}: model={}, ctx={}, max_iter={}",
            run_id,
            model_id,
            context_window,
            agent_config.max_iterations
        );

        // 1. Emit RUN_STARTED
        event_bus.emit(AgUiEvent::run_started(run_id));

        // 2. Collect available tools
        let registered_tools = tools::collect_all_tools(&self.state.mcp_client).await;
        let tool_definitions = tools::to_tool_definitions(&registered_tools);

        tracing::info!(
            "Agent has {} tools available ({} built-in, {} external)",
            registered_tools.len(),
            tools::builtin_tools().len(),
            registered_tools.len() - tools::builtin_tools().len()
        );

        // 3. Build system prompt
        let system_prompt = build_system_prompt(&self.state, messages);

        // 4. Build initial conversation
        let mut conversation: Vec<AgentChatMessage> = Vec::new();

        // System message
        conversation.push(AgentChatMessage {
            role: "system".into(),
            content: system_prompt,
            tool_calls: None,
        });

        // User messages
        for msg in messages {
            conversation.push(AgentChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                tool_calls: None,
            });
        }

        // 5. Serialize tools to OpenAI-compatible JSON for the chat template
        let tools_json = if tool_definitions.is_empty() {
            None
        } else {
            let json_str = serde_json::to_string(&tool_definitions)
                .map_err(|e| GhostError::Agent(format!("Failed to serialize tools: {}", e)))?;
            Some(json_str)
        };

        // 6. ReAct loop
        let mut iterations = 0;
        let mut all_tool_calls: Vec<ExecutedToolCall> = Vec::new();
        let mut final_content = String::new();

        loop {
            iterations += 1;

            if iterations > agent_config.max_iterations {
                tracing::warn!(
                    "Agent hit max iterations ({}), forcing response",
                    agent_config.max_iterations
                );
                event_bus.emit(AgUiEvent::custom(
                    run_id,
                    "max_iterations_reached",
                    json!({"iterations": iterations - 1}),
                ));
                break;
            }

            // Emit step started
            let step_name = if iterations == 1 {
                "thinking"
            } else {
                "reasoning"
            };
            event_bus.emit(AgUiEvent::step_started(
                run_id,
                step_name,
                Some(iterations - 1),
            ));

            // 7. Run native inference with tool calling
            let response = self
                .generate_native(
                    &model_id,
                    &conversation,
                    tools_json.as_deref(),
                    &agent_config,
                    context_window,
                )
                .await;

            match response {
                Ok(resp) => {
                    let has_tool_calls = !resp.tool_calls.is_empty();
                    let has_content = !resp.content.trim().is_empty();

                    if has_tool_calls {
                        // LLM wants to call tools → execute them
                        tracing::info!(
                            "Agent iteration {}: {} tool calls",
                            iterations,
                            resp.tool_calls.len()
                        );

                        // Add assistant message with tool calls to conversation
                        conversation.push(AgentChatMessage {
                            role: "assistant".into(),
                            content: resp.content.clone(),
                            tool_calls: Some(resp.tool_calls.clone()),
                        });

                        // If there's also text content, stream it
                        if has_content {
                            let msg_id =
                                format!("msg-{}-{}", &run_id[..8.min(run_id.len())], iterations);
                            self.stream_text(run_id, &msg_id, &resp.content, event_bus)
                                .await;
                        }

                        // Execute each tool call
                        for tc in &resp.tool_calls {
                            let tool_result = self
                                .execute_tool_call(
                                    run_id,
                                    &tc.function.name,
                                    &tc.function.arguments,
                                    &registered_tools,
                                    &agent_config,
                                    event_bus,
                                )
                                .await;

                            match tool_result {
                                Ok(executed) => {
                                    // Add tool result to conversation
                                    conversation.push(AgentChatMessage {
                                        role: "tool".into(),
                                        content: executed.result.clone(),
                                        tool_calls: None,
                                    });
                                    all_tool_calls.push(executed);
                                }
                                Err(e) => {
                                    // Tool execution failed — tell the LLM
                                    let error_msg =
                                        format!("Tool '{}' failed: {}", tc.function.name, e);
                                    conversation.push(AgentChatMessage {
                                        role: "tool".into(),
                                        content: error_msg.clone(),
                                        tool_calls: None,
                                    });
                                    all_tool_calls.push(ExecutedToolCall {
                                        name: tc.function.name.clone(),
                                        arguments: tc.function.arguments.clone(),
                                        result: error_msg,
                                        duration_ms: 0,
                                        risk_level: RiskLevel::Safe,
                                    });
                                }
                            }
                        }

                        event_bus.emit(AgUiEvent::step_finished(run_id, step_name));

                        // Continue the loop — LLM will process tool results
                        continue;
                    }

                    // No tool calls — this is the final text response
                    final_content = resp.content;

                    // Stream the final response
                    let msg_id = format!("msg-{}-final", &run_id[..8.min(run_id.len())]);
                    self.stream_text(run_id, &msg_id, &final_content, event_bus)
                        .await;

                    event_bus.emit(AgUiEvent::step_finished(run_id, step_name));
                    break;
                }
                Err(e) => {
                    tracing::error!("Native inference failed: {}", e);
                    event_bus.emit(AgUiEvent::step_finished(run_id, step_name));
                    event_bus.emit(AgUiEvent::run_error(run_id, &e.to_string()));
                    return Err(e);
                }
            }
        }

        let duration = start.elapsed();

        // Save to conversation memory if conversation_id provided
        if let Some(conv_id) = conversation_id {
            // Save user message
            if let Some(last_user) = messages.iter().rev().find(|m| m.role == "user") {
                let _ = super::memory::add_message(
                    &self.state.db,
                    conv_id,
                    "user",
                    &last_user.content,
                    None,
                    None,
                    None,
                );
            }

            // Save tool calls as JSON
            let tool_calls_json = if all_tool_calls.is_empty() {
                None
            } else {
                serde_json::to_string(&all_tool_calls).ok()
            };

            // Save assistant response
            let _ = super::memory::add_message(
                &self.state.db,
                conv_id,
                "assistant",
                &final_content,
                tool_calls_json.as_deref(),
                None,
                Some(&model_id),
            );
        }

        // Emit generation stats
        event_bus.emit(AgUiEvent::custom(
            run_id,
            "generation_stats",
            json!({
                "iterations": iterations,
                "tool_calls": all_tool_calls.len(),
                "duration_ms": duration.as_millis() as u64,
                "model": model_id,
            }),
        ));

        // Emit RUN_FINISHED
        event_bus.emit(AgUiEvent::run_finished(run_id));

        Ok(AgentRunResult {
            content: final_content,
            iterations,
            tool_calls_executed: all_tool_calls,
            duration_ms: duration.as_millis() as u64,
            model: model_id,
        })
    }

    /// Run native llama.cpp inference with grammar-constrained tool calling.
    ///
    /// 1. Loads the model (from HF cache or downloads once)
    /// 2. Gets the model's chat template for tool-calling format
    /// 3. Applies template with tools → gets prompt + GBNF grammar
    /// 4. Runs generation with grammar constraints
    /// 5. Parses response for tool calls using llama.cpp's built-in parser
    #[cfg(desktop)]
    async fn generate_native(
        &self,
        model_id: &str,
        conversation: &[AgentChatMessage],
        tools_json: Option<&str>,
        config: &AgentConfig,
        context_window: usize,
    ) -> Result<LlmResponse, GhostError> {
        use crate::chat::models;

        // 1. Get model profile
        let profile = models::find_model(model_id)
            .ok_or_else(|| GhostError::Agent(format!("Unknown agent model: {}", model_id)))?;

        // 2. Download model if needed (cached after first download)
        let model_path = Self::ensure_model_downloaded(profile).await?;

        // 3. Run inference in a blocking task (llama.cpp is synchronous)
        let conversation = conversation.to_vec();
        let tools_json = tools_json.map(String::from);
        let max_tokens = config.max_tokens;
        let temperature = config.temperature as f32;
        let ctx_window = context_window;

        tokio::task::spawn_blocking(move || {
            Self::run_inference(
                &model_path,
                &conversation,
                tools_json.as_deref(),
                max_tokens,
                temperature,
                ctx_window,
            )
        })
        .await
        .map_err(|e| GhostError::Agent(format!("Inference task panicked: {}", e)))?
    }

    /// Mobile stub — native inference not available on mobile.
    #[cfg(not(desktop))]
    async fn generate_native(
        &self,
        _model_id: &str,
        _conversation: &[AgentChatMessage],
        _tools_json: Option<&str>,
        _config: &AgentConfig,
        _context_window: usize,
    ) -> Result<LlmResponse, GhostError> {
        Err(GhostError::Agent(
            "Native agent inference is not available on mobile. Use Ollama fallback.".into(),
        ))
    }

    /// Download model files from HuggingFace Hub if not already cached.
    #[cfg(desktop)]
    async fn ensure_model_downloaded(
        profile: &crate::chat::models::ModelProfile,
    ) -> Result<std::path::PathBuf, GhostError> {
        let repo_id = profile.repo_id.to_string();
        let gguf_file = profile.gguf_file.to_string();

        // Check if already cached
        if crate::chat::models::is_model_cached(profile) {
            // Resolve cached path
            let path = tokio::task::spawn_blocking(move || {
                let api = hf_hub::api::sync::Api::new()
                    .map_err(|e| GhostError::Agent(format!("HF Hub API init failed: {}", e)))?;
                let repo = api.model(repo_id);
                repo.get(&gguf_file).map_err(|e| {
                    GhostError::Agent(format!("Failed to resolve cached model: {}", e))
                })
            })
            .await
            .map_err(|e| GhostError::Agent(format!("Task panicked: {}", e)))??;

            return Ok(path);
        }

        // Download (first time only)
        tracing::info!("Downloading agent model: {}/{}", repo_id, gguf_file);

        let path = tokio::task::spawn_blocking(move || {
            let api = hf_hub::api::sync::Api::new()
                .map_err(|e| GhostError::Agent(format!("HF Hub API init failed: {}", e)))?;
            let repo = api.model(repo_id.clone());
            repo.get(&gguf_file).map_err(|e| {
                GhostError::Agent(format!(
                    "Failed to download {}/{}: {}. Internet required for first-time setup.",
                    repo_id, gguf_file, e
                ))
            })
        })
        .await
        .map_err(|e| GhostError::Agent(format!("Download task panicked: {}", e)))??;

        tracing::info!("Agent model ready: {}", path.display());
        Ok(path)
    }

    /// Run synchronous llama.cpp inference with tool-calling support.
    ///
    /// This runs on a blocking thread. It:
    /// 1. Initializes the llama.cpp backend and loads the model
    /// 2. Gets the model's chat template
    /// 3. Applies the template with tools → prompt + grammar
    /// 4. Tokenizes and runs generation with grammar constraints
    /// 5. Parses the response for tool calls
    #[cfg(desktop)]
    fn run_inference(
        model_path: &std::path::Path,
        conversation: &[AgentChatMessage],
        tools_json: Option<&str>,
        max_tokens: usize,
        temperature: f32,
        context_window: usize,
    ) -> Result<LlmResponse, GhostError> {
        let gen_start = Instant::now();

        // 1. Get the global shared backend (initialized once, shared with chat engine)
        let backend = crate::chat::native::get_or_init_backend()
            .map_err(|e| GhostError::Agent(format!("Failed to init llama.cpp backend: {}", e)))?;

        let has_gpu = backend.supports_gpu_offload();
        let n_gpu_layers: u32 = if has_gpu { 9999 } else { 0 };

        let model_params = LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers);
        let model_path_str = model_path.to_string_lossy().to_string();

        let model = LlamaModel::load_from_file(&backend, &model_path_str, &model_params)
            .map_err(|e| GhostError::Agent(format!("Failed to load agent model: {}", e)))?;

        tracing::debug!(
            "Agent model loaded: gpu={}, layers={}",
            has_gpu,
            n_gpu_layers
        );

        // 2. Get the model's chat template
        let chat_template = model
            .chat_template(None)
            .map_err(|e| GhostError::Agent(format!("Model has no chat template: {}", e)))?;

        // 3. Convert conversation to LlamaChatMessage format
        let llama_messages: Vec<LlamaChatMessage> = conversation
            .iter()
            .map(|msg| {
                // For assistant messages with tool_calls, format them inline
                let content = if msg.role == "assistant" {
                    if let Some(ref tcs) = msg.tool_calls {
                        if tcs.is_empty() {
                            msg.content.clone()
                        } else {
                            // Include tool calls as JSON in content for the template
                            let tc_json = serde_json::to_string(tcs).unwrap_or_default();
                            if msg.content.is_empty() {
                                tc_json
                            } else {
                                format!("{}\n{}", msg.content, tc_json)
                            }
                        }
                    } else {
                        msg.content.clone()
                    }
                } else {
                    msg.content.clone()
                };

                LlamaChatMessage::new(msg.role.clone(), content)
                    .map_err(|e| GhostError::Agent(format!("Invalid message content: {}", e)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // 4. Apply chat template with tools → get prompt + grammar
        let template_result = model
            .apply_chat_template_with_tools_oaicompat(
                &chat_template,
                &llama_messages,
                tools_json,
                None, // no JSON schema
                true, // add generation prompt
            )
            .map_err(|e| GhostError::Agent(format!("Failed to apply chat template: {}", e)))?;

        tracing::debug!(
            "Chat template applied: prompt_len={}, grammar={}, lazy={}, triggers={}, stops={:?}, parse_tool_calls={}",
            template_result.prompt.len(),
            template_result.grammar.is_some(),
            template_result.grammar_lazy,
            template_result.grammar_triggers.len(),
            template_result.additional_stops,
            template_result.parse_tool_calls,
        );

        // 5. Tokenize the prompt
        let tokens = model
            .str_to_token(&template_result.prompt, AddBos::Never)
            .map_err(|e| GhostError::Agent(format!("Tokenization failed: {}", e)))?;

        let prompt_len = tokens.len();
        let ctx_size = context_window.max(prompt_len + max_tokens + 64);

        tracing::debug!(
            "Agent prompt: {} tokens, ctx_size={}, max_gen={}",
            prompt_len,
            ctx_size,
            max_tokens
        );

        if prompt_len >= ctx_size {
            return Err(GhostError::Agent(format!(
                "Prompt too long: {} tokens exceeds context window {}",
                prompt_len, ctx_size
            )));
        }

        // 6. Create context
        let n_threads = (std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            / 2)
        .max(1) as i32;

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(ctx_size as u32))
            .with_n_batch(BATCH_SIZE)
            .with_n_threads(n_threads)
            .with_n_threads_batch(n_threads);

        let mut ctx = model
            .new_context(&backend, ctx_params)
            .map_err(|e| GhostError::Agent(format!("Failed to create context: {}", e)))?;

        // 7. Build sampler chain with optional grammar constraint
        let mut sampler = Self::build_sampler(&model, &template_result, temperature)?;

        // 8. Prefill: submit prompt tokens in chunks of BATCH_SIZE
        //    When the prompt exceeds BATCH_SIZE tokens, we process it in
        //    multiple batch-decode passes. Only the very last token in the
        //    full prompt needs logits=true (for sampling the first generated token).
        let mut batch = LlamaBatch::new(BATCH_SIZE as usize, 1);
        let last_idx = tokens.len() as i32 - 1;
        for (i, &token) in tokens.iter().enumerate() {
            let is_last = i as i32 == last_idx;
            batch
                .add(token, i as i32, &[0], is_last)
                .map_err(|e| GhostError::Agent(format!("Batch add failed: {}", e)))?;

            // When the batch is full, or we've added the last token, decode it
            if batch.n_tokens() as u32 >= BATCH_SIZE || is_last {
                ctx.decode(&mut batch)
                    .map_err(|e| GhostError::Agent(format!("Prefill decode failed: {}", e)))?;
                // Only clear between intermediate chunks — keep the last one so
                // batch.n_tokens()-1 remains valid for the first sampler call.
                if !is_last {
                    batch.clear();
                }
            }
        }

        // 9. Generation loop
        let mut n_cur = tokens.len() as i32;
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut generated_text = String::new();

        for _ in 0..max_tokens {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            // Check end of generation
            if model.is_eog_token(token) {
                break;
            }

            // Decode token to text
            match model.token_to_piece(token, &mut decoder, true, None) {
                Ok(piece) => {
                    generated_text.push_str(&piece);

                    // Check additional stop sequences
                    if Self::check_stop_sequences(
                        &generated_text,
                        &template_result.additional_stops,
                    ) {
                        // Trim the stop sequence from output
                        for stop in &template_result.additional_stops {
                            if generated_text.ends_with(stop) {
                                let new_len = generated_text.len() - stop.len();
                                generated_text.truncate(new_len);
                                break;
                            }
                        }
                        break;
                    }
                }
                Err(e) => tracing::warn!("Token decode error: {}", e),
            }

            // Prepare next batch
            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| GhostError::Agent(format!("Batch add failed: {}", e)))?;

            n_cur += 1;

            ctx.decode(&mut batch)
                .map_err(|e| GhostError::Agent(format!("Decode failed at pos {}: {}", n_cur, e)))?;
        }

        let gen_duration = gen_start.elapsed();
        tracing::info!(
            "Agent generated {} chars in {:?} ({} tokens prompt)",
            generated_text.len(),
            gen_duration,
            prompt_len,
        );

        // 10. Parse the response for tool calls
        Self::parse_llm_response(&template_result, &generated_text)
    }

    /// Build a sampler chain with optional grammar constraint from the template.
    #[cfg(desktop)]
    fn build_sampler(
        model: &LlamaModel,
        template_result: &ChatTemplateResult,
        temperature: f32,
    ) -> Result<LlamaSampler, GhostError> {
        // Base samplers: temperature + top-p + distribution
        let base_samplers = if temperature <= 0.01 {
            vec![LlamaSampler::greedy()]
        } else {
            vec![
                LlamaSampler::temp(temperature),
                LlamaSampler::top_p(AGENT_TOP_P, 1),
                LlamaSampler::dist(AGENT_SEED),
            ]
        };

        // If we have a grammar, add grammar constraint
        if let Some(ref grammar_str) = template_result.grammar {
            if !grammar_str.is_empty() {
                if template_result.grammar_lazy && !template_result.grammar_triggers.is_empty() {
                    // Lazy grammar: only activates when trigger words/tokens are generated
                    let mut trigger_words: Vec<String> = Vec::new();
                    let mut trigger_tokens: Vec<llama_cpp_2::token::LlamaToken> = Vec::new();

                    for trigger in &template_result.grammar_triggers {
                        match trigger.trigger_type {
                            llama_cpp_2::model::GrammarTriggerType::Token => {
                                if let Some(token) = trigger.token {
                                    trigger_tokens.push(token);
                                }
                            }
                            llama_cpp_2::model::GrammarTriggerType::Word => {
                                trigger_words.push(trigger.value.clone());
                            }
                            // Pattern triggers use the grammar_lazy_patterns variant
                            _ => {
                                trigger_words.push(trigger.value.clone());
                            }
                        }
                    }

                    match LlamaSampler::grammar_lazy(
                        model,
                        grammar_str,
                        "root",
                        trigger_words.iter().map(|s| s.as_bytes()),
                        &trigger_tokens,
                    ) {
                        Ok(grammar_sampler) => {
                            let mut all_samplers = base_samplers;
                            all_samplers.push(grammar_sampler);
                            return Ok(LlamaSampler::chain_simple(all_samplers));
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to create lazy grammar sampler, falling back to unconstrained: {}",
                                e
                            );
                        }
                    }
                } else {
                    // Strict grammar: always enforced
                    match LlamaSampler::grammar(model, grammar_str, "root") {
                        Ok(grammar_sampler) => {
                            let mut all_samplers = base_samplers;
                            all_samplers.push(grammar_sampler);
                            return Ok(LlamaSampler::chain_simple(all_samplers));
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to create grammar sampler, falling back to unconstrained: {}",
                                e
                            );
                        }
                    }
                }
            }
        }

        // Fallback: no grammar constraint
        Ok(LlamaSampler::chain_simple(base_samplers))
    }

    /// Check if generated text ends with any stop sequence.
    #[cfg(desktop)]
    fn check_stop_sequences(text: &str, stops: &[String]) -> bool {
        stops.iter().any(|stop| text.ends_with(stop))
    }

    /// Parse the generated text into content + tool calls.
    ///
    /// Uses llama.cpp's `parse_response_oaicompat()` when the model's template
    /// supports tool call parsing, otherwise returns raw text.
    #[cfg(desktop)]
    fn parse_llm_response(
        template_result: &ChatTemplateResult,
        generated_text: &str,
    ) -> Result<LlmResponse, GhostError> {
        if !template_result.parse_tool_calls {
            // Model/template doesn't support tool calling — return raw text
            return Ok(LlmResponse {
                content: generated_text.trim().to_string(),
                tool_calls: Vec::new(),
            });
        }

        // Use llama.cpp's native parser
        let parsed_json = template_result
            .parse_response_oaicompat(generated_text, false)
            .map_err(|e| GhostError::Agent(format!("Failed to parse LLM response: {}", e)))?;

        tracing::debug!("Parsed response JSON: {}", parsed_json);

        // Deserialize the OpenAI-compatible message
        let parsed: ParsedOaiMessage = serde_json::from_str(&parsed_json).map_err(|e| {
            tracing::warn!(
                "Failed to deserialize parsed response, treating as text: {}",
                e
            );
            GhostError::Agent(format!("Failed to deserialize parsed response: {}", e))
        })?;

        let content = parsed.content.unwrap_or_default();
        let tool_calls = parsed
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| {
                // Ensure arguments is a proper JSON object
                let arguments = if tc.function.arguments.is_string() {
                    // Parse string arguments into JSON value
                    serde_json::from_str(tc.function.arguments.as_str().unwrap_or("{}"))
                        .unwrap_or(tc.function.arguments)
                } else {
                    tc.function.arguments
                };

                ToolCall {
                    function: ToolCallFunction {
                        name: tc.function.name,
                        arguments,
                    },
                }
            })
            .collect();

        Ok(LlmResponse {
            content,
            tool_calls,
        })
    }

    /// Execute a single tool call with safety checks and AG-UI events.
    async fn execute_tool_call(
        &self,
        run_id: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
        registered_tools: &[RegisteredTool],
        config: &AgentConfig,
        event_bus: &AgUiEventBus,
    ) -> Result<ExecutedToolCall, String> {
        let start = Instant::now();

        // Safety classification
        let risk = safety::classify_risk(tool_name, arguments);
        let auto_approve = safety::should_auto_approve(risk, config.auto_approve_safe);

        let tool_call_id = format!("tc-{}-{}", tool_name, &run_id[..6.min(run_id.len())]);

        // Emit TOOL_CALL_START
        event_bus.emit(AgUiEvent::tool_call_start(
            run_id,
            &tool_call_id,
            tool_name,
            None,
        ));

        // Emit tool call arguments
        let args_str = serde_json::to_string(arguments).unwrap_or_default();
        event_bus.emit(AgUiEvent::tool_call_args(run_id, &tool_call_id, &args_str));

        // Check approval
        if !auto_approve {
            // Emit a custom event requesting approval
            let description = safety::describe_action(tool_name, arguments);
            event_bus.emit(AgUiEvent::custom(
                run_id,
                "tool_approval_required",
                json!({
                    "tool_call_id": tool_call_id,
                    "tool_name": tool_name,
                    "arguments": arguments,
                    "risk_level": risk,
                    "description": description,
                }),
            ));

            // For now, auto-deny dangerous operations that need approval
            // TODO: Implement bidirectional approval flow via AG-UI
            let deny_msg = format!(
                "Tool '{}' requires user approval (risk: {:?}). Action: {}. Skipped for safety.",
                tool_name,
                risk,
                safety::describe_action(tool_name, arguments)
            );

            event_bus.emit(AgUiEvent::tool_call_end(
                run_id,
                &tool_call_id,
                Some(&deny_msg),
            ));

            return Ok(ExecutedToolCall {
                name: tool_name.into(),
                arguments: arguments.clone(),
                result: deny_msg,
                duration_ms: start.elapsed().as_millis() as u64,
                risk_level: risk,
            });
        }

        // Execute the tool
        let result = if let Some(tool) = tools::find_tool(registered_tools, tool_name) {
            if tool.source == "builtin" {
                // Execute built-in tool
                tools::execute_builtin_tool(tool_name, arguments, &self.state).await
            } else if tool.source.starts_with("mcp:") {
                // Execute MCP tool
                let server_name = tool.source.strip_prefix("mcp:").unwrap_or("");
                self.state
                    .mcp_client
                    .call_tool(server_name, tool_name, Some(arguments.clone()))
                    .await
                    .map_err(|e| format!("MCP tool error: {}", e))
            } else {
                Err(format!("Unknown tool source: {}", tool.source))
            }
        } else {
            Err(format!("Tool '{}' not found in registry", tool_name))
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(result_text) => {
                // Truncate result if too long (prevent context explosion)
                let truncated = if result_text.len() > 8000 {
                    format!(
                        "{}...\n[Result truncated: {} bytes total]",
                        &result_text[..8000],
                        result_text.len()
                    )
                } else {
                    result_text
                };

                event_bus.emit(AgUiEvent::tool_call_end(
                    run_id,
                    &tool_call_id,
                    Some(&truncated),
                ));

                tracing::info!(
                    "Tool '{}' completed in {}ms ({} bytes)",
                    tool_name,
                    duration_ms,
                    truncated.len()
                );

                Ok(ExecutedToolCall {
                    name: tool_name.into(),
                    arguments: arguments.clone(),
                    result: truncated,
                    duration_ms,
                    risk_level: risk,
                })
            }
            Err(e) => {
                let error_msg = format!("Error: {}", e);
                event_bus.emit(AgUiEvent::tool_call_end(
                    run_id,
                    &tool_call_id,
                    Some(&error_msg),
                ));

                Ok(ExecutedToolCall {
                    name: tool_name.into(),
                    arguments: arguments.clone(),
                    result: error_msg,
                    duration_ms,
                    risk_level: risk,
                })
            }
        }
    }

    /// Stream text content via AG-UI events (word-chunking simulation).
    async fn stream_text(
        &self,
        run_id: &str,
        message_id: &str,
        content: &str,
        event_bus: &AgUiEventBus,
    ) {
        event_bus.emit(AgUiEvent::text_message_start(
            run_id,
            message_id,
            "assistant",
        ));

        // Stream word-by-word groups for smooth UI update
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunk_size = 3.max(words.len() / 20).min(10);

        for (i, chunk) in words.chunks(chunk_size).enumerate() {
            let delta = if i == 0 {
                chunk.join(" ")
            } else {
                format!(" {}", chunk.join(" "))
            };
            event_bus.emit(AgUiEvent::text_message_content(run_id, message_id, &delta));
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        }

        event_bus.emit(AgUiEvent::text_message_end(run_id, message_id));
    }
}

/// Build the agent system prompt with context about available tools and skills.
pub(crate) fn build_system_prompt(state: &AppState, messages: &[ChatMessage]) -> String {
    let mut prompt = String::from(
        "You are Ghost, a private local-first AI assistant that runs entirely on the user's device. \
         You have access to tools that let you search the user's files, read documents, list directories, \
         and perform actions on their behalf.\n\n\
         ## Guidelines\n\
         - Be concise and helpful\n\
         - Use tools when the user asks about their files, documents, or needs information from their system\n\
         - For file searches, use ghost_search with natural language queries\n\
         - Read specific files with ghost_read_file when you need their full content\n\
         - If you're unsure, search first before answering\n\
         - Never fabricate file contents or paths — always verify with tools\n\
         - Respect the user's privacy — you only have access to their indexed directories\n\
         - When using ghost_run_command, explain what the command does before executing\n\
         - Prefer read-only operations unless the user explicitly asks for changes\n"
    );

    // Add index stats context
    if let Ok(stats) = state.db.get_stats() {
        prompt.push_str(&format!(
            "\n## Your Knowledge Base\n\
             You have access to {} indexed documents with {} searchable text chunks ({} with semantic embeddings).\n",
            stats.document_count, stats.chunk_count, stats.embedded_chunk_count
        ));
    }

    // Add skills context
    let skills_dir = state
        .settings
        .lock()
        .map(|s| s.agent_config.skills_dir.clone())
        .unwrap_or_default();

    if !skills_dir.is_empty() {
        let mut registry = super::skills::SkillRegistry::new();
        registry.load_from_directory(std::path::Path::new(&skills_dir));

        // Check if any user message triggers a skill
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let skill_prompt = registry.build_prompt_for_query(last_user_msg);
        if !skill_prompt.is_empty() {
            prompt.push_str(&skill_prompt);
        }
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::ChatMessage;
    use serde_json::json;

    // --- Helper: build a minimal AppState for testing ---
    fn test_app_state() -> Arc<AppState> {
        crate::ensure_tls_provider();
        let db = crate::db::Database::open_in_memory().unwrap();
        let hardware = crate::embeddings::hardware::HardwareInfo {
            cpu_cores: 4,
            has_avx2: false,
            has_neon: false,
            gpu_backend: None,
            total_ram_mb: 8192,
            available_ram_mb: 4096,
        };
        let chat_engine = crate::chat::ChatEngine::new(hardware.clone(), "qwen2.5-0.5b".into());
        let settings = crate::settings::Settings::default();

        Arc::new(AppState {
            db,
            embedding_engine: crate::embeddings::EmbeddingEngine::none(),
            chat_engine,
            hardware,
            settings: std::sync::Mutex::new(settings),
            mcp_client: crate::protocols::mcp_client::McpClientManager::new(),
            agui_event_bus: crate::protocols::agui::AgUiEventBus::new(32),
        })
    }

    // ==========================================
    // build_system_prompt tests
    // ==========================================

    #[test]
    fn test_system_prompt_contains_ghost_identity() {
        let state = test_app_state();
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "Hello".into(),
        }];
        let prompt = build_system_prompt(&state, &messages);
        assert!(
            prompt.contains("Ghost"),
            "System prompt should mention Ghost"
        );
        assert!(
            prompt.contains("private"),
            "System prompt should mention privacy"
        );
    }

    #[test]
    fn test_system_prompt_contains_guidelines() {
        let state = test_app_state();
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "Search for files".into(),
        }];
        let prompt = build_system_prompt(&state, &messages);
        assert!(
            prompt.contains("Guidelines"),
            "Should have guidelines section"
        );
        assert!(
            prompt.contains("ghost_search"),
            "Should mention ghost_search tool"
        );
        assert!(
            prompt.contains("ghost_read_file"),
            "Should mention ghost_read_file tool"
        );
    }

    #[test]
    fn test_system_prompt_includes_index_stats() {
        let state = test_app_state();
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "test".into(),
        }];
        let prompt = build_system_prompt(&state, &messages);
        // In-memory DB has 0 documents but get_stats() should succeed
        assert!(
            prompt.contains("Knowledge Base"),
            "Should include knowledge base section"
        );
        assert!(
            prompt.contains("0 indexed documents"),
            "Should show zero documents for empty DB"
        );
    }

    #[test]
    fn test_system_prompt_empty_messages() {
        let state = test_app_state();
        let messages: Vec<ChatMessage> = vec![];
        let prompt = build_system_prompt(&state, &messages);
        // Should still produce a valid prompt
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Ghost"));
    }

    // ==========================================
    // check_stop_sequences tests
    // ==========================================

    #[cfg(desktop)]
    #[test]
    fn test_check_stop_sequences_match() {
        let stops = vec!["<|end|>".to_string(), "<|im_end|>".to_string()];
        assert!(AgentExecutor::check_stop_sequences(
            "Hello world<|end|>",
            &stops
        ));
        assert!(AgentExecutor::check_stop_sequences(
            "Some text<|im_end|>",
            &stops
        ));
    }

    #[cfg(desktop)]
    #[test]
    fn test_check_stop_sequences_no_match() {
        let stops = vec!["<|end|>".to_string(), "<|im_end|>".to_string()];
        assert!(!AgentExecutor::check_stop_sequences("Hello world", &stops));
        assert!(!AgentExecutor::check_stop_sequences(
            "Some text<|end|> more text",
            &stops
        ));
    }

    #[cfg(desktop)]
    #[test]
    fn test_check_stop_sequences_empty() {
        let stops: Vec<String> = vec![];
        assert!(!AgentExecutor::check_stop_sequences(
            "Any text at all",
            &stops
        ));
    }

    // ==========================================
    // ParsedOaiMessage deserialization tests
    // ==========================================

    #[test]
    fn test_parse_oai_message_text_only() {
        let json = r#"{"content": "Hello, I can help you with that."}"#;
        let msg: ParsedOaiMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.content.unwrap(), "Hello, I can help you with that.");
        assert!(msg.tool_calls.is_none());
    }

    #[test]
    fn test_parse_oai_message_with_tool_calls() {
        let json = r#"{
            "content": "",
            "tool_calls": [
                {
                    "function": {
                        "name": "ghost_search",
                        "arguments": {"query": "rust files"}
                    }
                }
            ]
        }"#;
        let msg: ParsedOaiMessage = serde_json::from_str(json).unwrap();
        assert!(msg.content.unwrap().is_empty());
        let tc = msg.tool_calls.unwrap();
        assert_eq!(tc.len(), 1);
        assert_eq!(tc[0].function.name, "ghost_search");
        assert_eq!(tc[0].function.arguments["query"], "rust files");
    }

    #[test]
    fn test_parse_oai_message_string_arguments() {
        // Some models return arguments as a JSON string, not an object
        let json = r#"{
            "tool_calls": [
                {
                    "function": {
                        "name": "ghost_read_file",
                        "arguments": "{\"path\": \"/tmp/test.txt\"}"
                    }
                }
            ]
        }"#;
        let msg: ParsedOaiMessage = serde_json::from_str(json).unwrap();
        let tc = msg.tool_calls.unwrap();
        assert_eq!(tc[0].function.name, "ghost_read_file");
        // arguments is a JSON string — our code handles this in parse_llm_response
        assert!(tc[0].function.arguments.is_string());
    }

    #[test]
    fn test_parse_oai_message_no_content_field() {
        let json = r#"{"tool_calls": []}"#;
        let msg: ParsedOaiMessage = serde_json::from_str(json).unwrap();
        assert!(msg.content.is_none());
        assert!(msg.tool_calls.unwrap().is_empty());
    }

    #[test]
    fn test_parse_oai_message_empty() {
        let json = r#"{}"#;
        let msg: ParsedOaiMessage = serde_json::from_str(json).unwrap();
        assert!(msg.content.is_none());
        assert!(msg.tool_calls.is_none());
    }

    #[test]
    fn test_parse_oai_message_multiple_tool_calls() {
        let json = r#"{
            "content": "Let me search and then read the file.",
            "tool_calls": [
                {"function": {"name": "ghost_search", "arguments": {"query": "config"}}},
                {"function": {"name": "ghost_read_file", "arguments": {"path": "/etc/hosts"}}}
            ]
        }"#;
        let msg: ParsedOaiMessage = serde_json::from_str(json).unwrap();
        assert!(msg.content.unwrap().contains("search"));
        let tc = msg.tool_calls.unwrap();
        assert_eq!(tc.len(), 2);
        assert_eq!(tc[0].function.name, "ghost_search");
        assert_eq!(tc[1].function.name, "ghost_read_file");
    }

    // ==========================================
    // LlmResponse construction tests
    // ==========================================

    #[test]
    fn test_llm_response_text_only() {
        let resp = LlmResponse {
            content: "The answer is 42".into(),
            tool_calls: vec![],
        };
        assert_eq!(resp.content, "The answer is 42");
        assert!(resp.tool_calls.is_empty());
    }

    #[test]
    fn test_llm_response_with_tool_calls() {
        let resp = LlmResponse {
            content: String::new(),
            tool_calls: vec![ToolCall {
                function: ToolCallFunction {
                    name: "ghost_search".into(),
                    arguments: json!({"query": "test"}),
                },
            }],
        };
        assert!(resp.content.is_empty());
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].function.name, "ghost_search");
    }

    // ==========================================
    // AgentExecutor construction/creation tests
    // ==========================================

    #[test]
    fn test_executor_creation() {
        let state = test_app_state();
        let executor = AgentExecutor::new(state);
        // Should not panic, executor is created successfully
        assert!(std::mem::size_of_val(&executor) > 0);
    }

    // ==========================================
    // Agent run with AG-UI event streaming tests
    // ==========================================

    #[tokio::test]
    async fn test_agent_run_emits_events() {
        let state = test_app_state();
        let event_bus = &state.agui_event_bus;
        let mut rx = event_bus.subscribe();

        let executor = AgentExecutor::new(state.clone());

        // The run will fail (no model available in CI) but should emit RUN_STARTED + RUN_ERROR
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "Hello".into(),
        }];

        let run_id = "test-run-001";
        let result = executor.run(run_id, &messages, None, event_bus).await;

        // We expect an error since there's no model downloaded in test
        assert!(result.is_err(), "Should fail without a downloaded model");

        // But we should have received at least RUN_STARTED event
        let mut got_run_started = false;
        let mut got_run_error = false;

        // Drain available events
        while let Ok(event) = rx.try_recv() {
            if event.run_id == run_id {
                match event.event_type {
                    crate::protocols::agui::EventType::RunStarted => {
                        got_run_started = true;
                    }
                    crate::protocols::agui::EventType::RunError => {
                        got_run_error = true;
                    }
                    _ => {}
                }
            }
        }

        assert!(got_run_started, "Should have emitted RUN_STARTED event");
        assert!(
            got_run_error,
            "Should have emitted RUN_ERROR event on failure"
        );
    }

    // ==========================================
    // Tool execution tests
    // ==========================================

    #[tokio::test]
    async fn test_execute_tool_call_unknown_tool() {
        let state = test_app_state();
        let event_bus = &state.agui_event_bus;
        let executor = AgentExecutor::new(state.clone());

        let config = AgentConfig::default();
        let registered_tools = crate::agent::tools::builtin_tools();

        let result = executor
            .execute_tool_call(
                "run-test",
                "nonexistent_tool",
                &json!({}),
                &registered_tools,
                &config,
                event_bus,
            )
            .await;

        let executed = result.unwrap();
        assert_eq!(executed.name, "nonexistent_tool");
        assert!(
            executed.result.contains("not found"),
            "Should indicate tool not found: {}",
            executed.result
        );
    }

    #[tokio::test]
    async fn test_execute_tool_call_index_status() {
        let state = test_app_state();
        let event_bus = &state.agui_event_bus;
        let executor = AgentExecutor::new(state.clone());

        let config = AgentConfig::default();
        let registered_tools = crate::agent::tools::builtin_tools();

        let result = executor
            .execute_tool_call(
                "run-test",
                "ghost_index_status",
                &json!({}),
                &registered_tools,
                &config,
                event_bus,
            )
            .await;

        let executed = result.unwrap();
        assert_eq!(executed.name, "ghost_index_status");
        assert!(
            executed.result.contains("Indexed"),
            "Should contain indexing info: {}",
            executed.result
        );
        assert_eq!(executed.risk_level, RiskLevel::Safe);
    }

    #[tokio::test]
    async fn test_execute_tool_call_dangerous_denied() {
        let state = test_app_state();
        let event_bus = &state.agui_event_bus;
        let executor = AgentExecutor::new(state.clone());

        let config = AgentConfig {
            auto_approve_safe: false, // won't approve moderate
            ..Default::default()
        };
        let registered_tools = crate::agent::tools::builtin_tools();

        let result = executor
            .execute_tool_call(
                "run-test",
                "ghost_run_command",
                &json!({"command": "ls"}),
                &registered_tools,
                &config,
                event_bus,
            )
            .await;

        let executed = result.unwrap();
        assert_eq!(executed.name, "ghost_run_command");
        assert_eq!(executed.risk_level, RiskLevel::Dangerous);
        assert!(
            executed.result.contains("requires user approval"),
            "Dangerous tool should be denied: {}",
            executed.result
        );
    }

    #[tokio::test]
    async fn test_execute_tool_call_search_empty_db() {
        let state = test_app_state();
        let event_bus = &state.agui_event_bus;
        let executor = AgentExecutor::new(state.clone());

        let config = AgentConfig::default();
        let registered_tools = crate::agent::tools::builtin_tools();

        let result = executor
            .execute_tool_call(
                "run-test",
                "ghost_search",
                &json!({"query": "test document"}),
                &registered_tools,
                &config,
                event_bus,
            )
            .await;

        let executed = result.unwrap();
        assert_eq!(executed.name, "ghost_search");
        assert_eq!(executed.risk_level, RiskLevel::Safe);
        // Empty DB → no results
        assert!(
            executed.result.contains("No results found"),
            "Empty DB should return no results: {}",
            executed.result
        );
    }

    #[tokio::test]
    async fn test_execute_tool_emits_agui_events() {
        let state = test_app_state();
        let event_bus = &state.agui_event_bus;
        let mut rx = event_bus.subscribe();

        let executor = AgentExecutor::new(state.clone());
        let config = AgentConfig::default();
        let registered_tools = crate::agent::tools::builtin_tools();

        executor
            .execute_tool_call(
                "run-test",
                "ghost_index_status",
                &json!({}),
                &registered_tools,
                &config,
                event_bus,
            )
            .await
            .unwrap();

        let mut got_start = false;
        let mut got_args = false;
        let mut got_end = false;

        while let Ok(event) = rx.try_recv() {
            match event.event_type {
                crate::protocols::agui::EventType::ToolCallStart => got_start = true,
                crate::protocols::agui::EventType::ToolCallArgs => got_args = true,
                crate::protocols::agui::EventType::ToolCallEnd => got_end = true,
                _ => {}
            }
        }

        assert!(got_start, "Should emit TOOL_CALL_START");
        assert!(got_args, "Should emit TOOL_CALL_ARGS");
        assert!(got_end, "Should emit TOOL_CALL_END");
    }

    // ==========================================
    // Streaming text tests
    // ==========================================

    #[tokio::test]
    async fn test_stream_text_emits_message_events() {
        let state = test_app_state();
        let event_bus = &state.agui_event_bus;
        let mut rx = event_bus.subscribe();

        let executor = AgentExecutor::new(state.clone());

        executor
            .stream_text(
                "run-test",
                "msg-01",
                "Hello world from the agent",
                event_bus,
            )
            .await;

        let mut got_start = false;
        let mut got_content = false;
        let mut got_end = false;

        while let Ok(event) = rx.try_recv() {
            match event.event_type {
                crate::protocols::agui::EventType::TextMessageStart => got_start = true,
                crate::protocols::agui::EventType::TextMessageContent => got_content = true,
                crate::protocols::agui::EventType::TextMessageEnd => got_end = true,
                _ => {}
            }
        }

        assert!(got_start, "Should emit TEXT_MESSAGE_START");
        assert!(got_content, "Should emit TEXT_MESSAGE_CONTENT");
        assert!(got_end, "Should emit TEXT_MESSAGE_END");
    }

    #[tokio::test]
    async fn test_stream_text_empty_content() {
        let state = test_app_state();
        let event_bus = &state.agui_event_bus;
        let mut rx = event_bus.subscribe();

        let executor = AgentExecutor::new(state.clone());
        executor
            .stream_text("run-test", "msg-01", "", event_bus)
            .await;

        let mut got_start = false;
        let mut got_end = false;

        while let Ok(event) = rx.try_recv() {
            match event.event_type {
                crate::protocols::agui::EventType::TextMessageStart => got_start = true,
                crate::protocols::agui::EventType::TextMessageEnd => got_end = true,
                _ => {}
            }
        }

        assert!(got_start, "Should still emit start for empty content");
        assert!(got_end, "Should still emit end for empty content");
    }

    // ==========================================
    // Conversation memory integration tests
    // ==========================================

    #[test]
    fn test_conversation_memory_roundtrip() {
        let state = test_app_state();

        // Initialize memory schema
        crate::agent::memory::initialize_memory_schema(&state.db).unwrap();

        // Create a conversation
        let conv_id =
            crate::agent::memory::create_conversation(&state.db, "Test Agent Run").unwrap();
        assert!(conv_id > 0);

        // Add user message
        crate::agent::memory::add_message(
            &state.db,
            conv_id,
            "user",
            "Find my Rust files",
            None,
            None,
            None,
        )
        .unwrap();

        // Add assistant response with tool calls
        let tool_calls_json = serde_json::to_string(&vec![ExecutedToolCall {
            name: "ghost_search".into(),
            arguments: json!({"query": "Rust files"}),
            result: "Found 3 results".into(),
            duration_ms: 50,
            risk_level: RiskLevel::Safe,
        }])
        .unwrap();

        crate::agent::memory::add_message(
            &state.db,
            conv_id,
            "assistant",
            "I found 3 Rust files in your project.",
            Some(&tool_calls_json),
            None,
            Some("qwen2.5-3b"),
        )
        .unwrap();

        // Verify messages
        let messages = crate::agent::memory::get_messages(&state.db, conv_id, None).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert!(messages[1].tool_calls.is_some());
        assert_eq!(messages[1].model.as_deref(), Some("qwen2.5-3b"));
    }

    // ==========================================
    // Type serialization roundtrip tests
    // ==========================================

    #[test]
    fn test_tool_call_serde_roundtrip() {
        let tc = ToolCall {
            function: ToolCallFunction {
                name: "ghost_search".into(),
                arguments: json!({"query": "hello", "limit": 5}),
            },
        };

        let json_str = serde_json::to_string(&tc).unwrap();
        let deserialized: ToolCall = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.function.name, "ghost_search");
        assert_eq!(deserialized.function.arguments["query"], "hello");
        assert_eq!(deserialized.function.arguments["limit"], 5);
    }

    #[test]
    fn test_agent_chat_message_serde() {
        let msg = AgentChatMessage {
            role: "assistant".into(),
            content: "Let me search for that.".into(),
            tool_calls: Some(vec![ToolCall {
                function: ToolCallFunction {
                    name: "ghost_search".into(),
                    arguments: json!({"query": "test"}),
                },
            }]),
        };

        let json_str = serde_json::to_string(&msg).unwrap();
        assert!(json_str.contains("ghost_search"));
        assert!(json_str.contains("tool_calls"));

        let deserialized: AgentChatMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.role, "assistant");
        assert!(deserialized.tool_calls.is_some());
        assert_eq!(deserialized.tool_calls.unwrap().len(), 1);
    }

    #[test]
    fn test_agent_chat_message_no_tool_calls_omitted() {
        let msg = AgentChatMessage {
            role: "user".into(),
            content: "Hello".into(),
            tool_calls: None,
        };

        let json_str = serde_json::to_string(&msg).unwrap();
        // tool_calls should be omitted (skip_serializing_if)
        assert!(!json_str.contains("tool_calls"));
    }

    #[test]
    fn test_agent_run_result_serialization() {
        let result = AgentRunResult {
            content: "Here are your files.".into(),
            iterations: 2,
            tool_calls_executed: vec![ExecutedToolCall {
                name: "ghost_search".into(),
                arguments: json!({"query": "files"}),
                result: "Found 5 results".into(),
                duration_ms: 100,
                risk_level: RiskLevel::Safe,
            }],
            duration_ms: 1500,
            model: "qwen2.5-3b".into(),
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(value["content"], "Here are your files.");
        assert_eq!(value["iterations"], 2);
        assert_eq!(value["duration_ms"], 1500);
        assert_eq!(value["model"], "qwen2.5-3b");
        assert_eq!(value["tool_calls_executed"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_executed_tool_call_risk_level_serialization() {
        let tc = ExecutedToolCall {
            name: "ghost_run_command".into(),
            arguments: json!({"command": "ls"}),
            result: "file1\nfile2".into(),
            duration_ms: 50,
            risk_level: RiskLevel::Dangerous,
        };

        let json_str = serde_json::to_string(&tc).unwrap();
        assert!(
            json_str.contains("\"dangerous\""),
            "Risk level should serialize as lowercase: {}",
            json_str
        );
    }
}
