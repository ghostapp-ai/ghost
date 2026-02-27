//! Tool registry — collects and converts tools for LLM consumption.
//!
//! Gathers tools from three sources:
//! 1. **Built-in tools**: Ghost's own capabilities (search, index, file ops)
//! 2. **MCP external tools**: From connected MCP servers
//! 3. **Skill tools**: From loaded SKILL.md definitions (future)
//!
//! Converts all tools into OpenAI-compatible JSON schemas used by
//! llama.cpp's `apply_chat_template_with_tools_oaicompat` for native
//! grammar-constrained tool calling.

use serde_json::json;
use std::sync::Arc;

use super::{AgentTool, AgentToolFunction};
use crate::protocols::mcp_client::{McpClientManager, ToolInfo};

/// A registered tool with its source information.
#[derive(Debug, Clone)]
pub struct RegisteredTool {
    /// The tool definition in OpenAI-compatible format.
    pub definition: AgentTool,
    /// Source: "builtin", "mcp:<server_name>", or "skill:<skill_name>".
    pub source: String,
    /// Whether this tool requires user approval before execution.
    #[allow(dead_code)]
    pub requires_approval: bool,
}

/// Collect all available tools from all sources.
pub async fn collect_all_tools(mcp_client: &McpClientManager) -> Vec<RegisteredTool> {
    let mut tools = Vec::new();

    // 1. Built-in Ghost tools
    tools.extend(builtin_tools());

    // 2. MCP external tools
    let mcp_tools = mcp_client.all_tools().await;
    for (server_name, tool_info) in mcp_tools {
        tools.push(mcp_tool_to_registered(&server_name, &tool_info));
    }

    tools
}

/// Ghost's built-in tools available to the agent.
pub fn builtin_tools() -> Vec<RegisteredTool> {
    vec![
        RegisteredTool {
            definition: AgentTool {
                tool_type: "function".into(),
                function: AgentToolFunction {
                    name: "ghost_search".into(),
                    description: "Search the user's indexed local files using hybrid semantic + keyword search. Use when the user asks about their files, documents, or stored content. Returns file snippets with paths and relevance scores. Do NOT use for general knowledge questions — answer those directly.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Natural language or keyword search query about the user's files"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum results to return (default: 10, max: 50)",
                                "default": 10
                            }
                        },
                        "required": ["query"]
                    }),
                },
            },
            source: "builtin".into(),
            requires_approval: false,
        },
        RegisteredTool {
            definition: AgentTool {
                tool_type: "function".into(),
                function: AgentToolFunction {
                    name: "ghost_read_file".into(),
                    description: "Read the text content of a file. Use after ghost_search finds a relevant file, or when the user provides a specific file path. Returns up to 100KB of text. Do NOT guess file paths — use ghost_search or ghost_list_directory first to discover them.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Absolute path to the file (must exist on disk)"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            source: "builtin".into(),
            requires_approval: false,
        },
        RegisteredTool {
            definition: AgentTool {
                tool_type: "function".into(),
                function: AgentToolFunction {
                    name: "ghost_list_directory".into(),
                    description: "List files and subdirectories in a directory. Returns names, sizes, and types (file/dir). Use to explore the filesystem before reading specific files. Hidden files (starting with .) are excluded.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Absolute path to the directory to list"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            source: "builtin".into(),
            requires_approval: false,
        },
        RegisteredTool {
            definition: AgentTool {
                tool_type: "function".into(),
                function: AgentToolFunction {
                    name: "ghost_index_status".into(),
                    description: "Get the current indexing statistics: total documents, text chunks, and semantic embeddings in the user's vault. Use when the user asks about what has been indexed or how much content is searchable.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                },
            },
            source: "builtin".into(),
            requires_approval: false,
        },
        RegisteredTool {
            definition: AgentTool {
                tool_type: "function".into(),
                function: AgentToolFunction {
                    name: "ghost_write_file".into(),
                    description: "Write text content to a file. Creates the file if it doesn't exist, overwrites if it does. Creates parent directories automatically. Only use when the user explicitly asks to create or modify a file. Requires approval for sensitive paths.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Absolute path to the file to write (parent dirs created automatically)"
                            },
                            "content": {
                                "type": "string",
                                "description": "The complete text content to write"
                            }
                        },
                        "required": ["path", "content"]
                    }),
                },
            },
            source: "builtin".into(),
            requires_approval: true, // Writing files is a destructive operation
        },
        RegisteredTool {
            definition: AgentTool {
                tool_type: "function".into(),
                function: AgentToolFunction {
                    name: "ghost_run_command".into(),
                    description: "Execute a shell command on the user's system. DANGEROUS: always requires explicit user approval. ONLY use when the user explicitly asks to run a command or perform a system operation. NEVER use proactively or to gather information that other tools can provide. Commands time out after 30 seconds. Explain what the command does before executing.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "Shell command to execute (single command, avoid chaining with && or pipes unless necessary)"
                            },
                            "working_directory": {
                                "type": "string",
                                "description": "Working directory (optional, defaults to user's home directory)"
                            }
                        },
                        "required": ["command"]
                    }),
                },
            },
            source: "builtin".into(),
            requires_approval: true, // Always require approval for shell commands
        },
    ]
}

/// Convert an MCP tool to a registered tool.
fn mcp_tool_to_registered(server_name: &str, tool_info: &ToolInfo) -> RegisteredTool {
    let parameters = tool_info
        .input_schema
        .clone()
        .unwrap_or_else(|| json!({"type": "object", "properties": {}, "required": []}));

    RegisteredTool {
        definition: AgentTool {
            tool_type: "function".into(),
            function: AgentToolFunction {
                name: tool_info.name.clone(),
                description: tool_info
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("Tool from MCP server '{}'", server_name)),
                parameters,
            },
        },
        source: format!("mcp:{}", server_name),
        requires_approval: false, // MCP tools are trusted by connection
    }
}

/// Convert registered tools to OpenAI-compatible tool definitions.
///
/// The resulting JSON array is passed to llama.cpp's
/// `apply_chat_template_with_tools_oaicompat` which generates
/// model-specific prompts with grammar constraints.
pub fn to_tool_definitions(tools: &[RegisteredTool]) -> Vec<&AgentTool> {
    tools.iter().map(|t| &t.definition).collect()
}

/// Find a registered tool by name.
pub fn find_tool<'a>(tools: &'a [RegisteredTool], name: &str) -> Option<&'a RegisteredTool> {
    tools.iter().find(|t| t.definition.function.name == name)
}

/// Execute a built-in Ghost tool.
///
/// Returns the tool result as a string, or an error.
pub async fn execute_builtin_tool(
    name: &str,
    arguments: &serde_json::Value,
    state: &Arc<crate::AppState>,
) -> Result<String, String> {
    match name {
        "ghost_search" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'query' argument")?;
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as usize;

            let results =
                crate::search::hybrid_search(&state.db, &state.embedding_engine, query, limit)
                    .await
                    .map_err(|e| format!("Search failed: {}", e))?;

            if results.is_empty() {
                Ok("No results found.".into())
            } else {
                let formatted: Vec<String> = results
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        format!(
                            "{}. [{}] {} (score: {:.2})\n   {}",
                            i + 1,
                            r.filename,
                            r.path,
                            r.score,
                            r.snippet.chars().take(200).collect::<String>()
                        )
                    })
                    .collect();
                Ok(formatted.join("\n\n"))
            }
        }

        "ghost_read_file" => {
            let path = arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path' argument")?;

            // Safety: only read files, max 100KB
            let content = tokio::fs::read_to_string(path)
                .await
                .map_err(|e| format!("Failed to read file: {}", e))?;

            if content.len() > 102400 {
                // Safe UTF-8 truncation: find a char boundary at or before 100KB
                let safe_end = (0..=102400)
                    .rev()
                    .find(|&i| content.is_char_boundary(i))
                    .unwrap_or(0);
                Ok(format!(
                    "{}...\n\n[Truncated: file is {} bytes, showing first ~100KB]",
                    &content[..safe_end],
                    content.len()
                ))
            } else {
                Ok(content)
            }
        }

        "ghost_list_directory" => {
            let path = arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path' argument")?;

            let mut entries = tokio::fs::read_dir(path)
                .await
                .map_err(|e| format!("Failed to read directory: {}", e))?;

            let mut items = Vec::new();
            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| format!("Error reading entry: {}", e))?
            {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue; // Skip hidden files
                }
                let metadata = entry.metadata().await.ok();
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let suffix = if is_dir { "/" } else { "" };
                items.push(format!("{}{} ({})", name, suffix, format_bytes(size)));
            }
            items.sort();
            if items.is_empty() {
                Ok("Directory is empty.".into())
            } else {
                Ok(items.join("\n"))
            }
        }

        "ghost_index_status" => {
            let stats = state
                .db
                .get_stats()
                .map_err(|e| format!("DB error: {}", e))?;
            Ok(format!(
                "Indexed {} documents, {} chunks ({} with embeddings)",
                stats.document_count, stats.chunk_count, stats.embedded_chunk_count
            ))
        }

        "ghost_write_file" => {
            let path = arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path' argument")?;
            let content = arguments
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content' argument")?;

            // Create parent directories if needed
            if let Some(parent) = std::path::Path::new(path).parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("Failed to create directories: {}", e))?;
            }

            tokio::fs::write(path, content)
                .await
                .map_err(|e| format!("Failed to write file: {}", e))?;

            Ok(format!(
                "File written successfully: {} ({} bytes)",
                path,
                content.len()
            ))
        }

        "ghost_run_command" => {
            let command = arguments
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'command' argument")?;

            // Validate command is not empty or whitespace-only
            let command = command.trim();
            if command.is_empty() {
                return Err("Command cannot be empty".into());
            }

            // Reject null bytes (command injection vector)
            if command.contains('\0') {
                return Err("Command contains invalid null bytes".into());
            }

            let working_dir = arguments
                .get("working_directory")
                .and_then(|v| v.as_str())
                .unwrap_or("~");

            let home = dirs::home_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let cwd = working_dir.replace('~', &home);

            // Verify working directory exists
            if !std::path::Path::new(&cwd).is_dir() {
                return Err(format!("Working directory does not exist: {}", cwd));
            }

            // Execute with timeout (30 seconds default)
            // Use platform-appropriate shell
            #[cfg(target_os = "windows")]
            let child = tokio::process::Command::new("cmd")
                .arg("/C")
                .arg(command)
                .current_dir(&cwd)
                .env_remove("GITHUB_TOKEN")
                .env_remove("GH_TOKEN")
                .env_remove("AWS_SECRET_ACCESS_KEY")
                .env_remove("OPENAI_API_KEY")
                .env_remove("ANTHROPIC_API_KEY")
                .output();
            #[cfg(not(target_os = "windows"))]
            let child = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&cwd)
                .env_remove("GITHUB_TOKEN")
                .env_remove("GH_TOKEN")
                .env_remove("AWS_SECRET_ACCESS_KEY")
                .env_remove("OPENAI_API_KEY")
                .env_remove("ANTHROPIC_API_KEY")
                .output();

            let output = tokio::time::timeout(std::time::Duration::from_secs(30), child)
                .await
                .map_err(|_| {
                    format!(
                        "Command timed out after 30 seconds: {}",
                        &command[..command.len().min(100)]
                    )
                })?
                .map_err(|e| format!("Failed to execute command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let mut result = String::new();
            if !stdout.is_empty() {
                // Limit output to prevent context overflow (safe UTF-8 truncation)
                let truncated = if stdout.len() > 10000 {
                    let safe_end = (0..=10000)
                        .rev()
                        .find(|&i| stdout.is_char_boundary(i))
                        .unwrap_or(0);
                    format!(
                        "{}...\n[Truncated: {} bytes total]",
                        &stdout[..safe_end],
                        stdout.len()
                    )
                } else {
                    stdout.to_string()
                };
                result.push_str(&format!("stdout:\n{}", redact_secrets(&truncated)));
            }
            if !stderr.is_empty() {
                let truncated = if stderr.len() > 5000 {
                    let safe_end = (0..=5000)
                        .rev()
                        .find(|&i| stderr.is_char_boundary(i))
                        .unwrap_or(0);
                    format!("{}...\n[Truncated]", &stderr[..safe_end])
                } else {
                    stderr.to_string()
                };
                result.push_str(&format!("\nstderr:\n{}", redact_secrets(&truncated)));
            }
            if result.is_empty() {
                result = format!(
                    "Command completed with exit code: {:?}",
                    output.status.code()
                );
            }

            Ok(result)
        }

        _ => Err(format!("Unknown built-in tool: {}", name)),
    }
}

/// Redact potential secrets from tool output.
///
/// Matches common patterns for API keys, tokens, passwords, and secrets
/// to prevent them from leaking into the LLM context.
fn redact_secrets(text: &str) -> String {
    // Patterns: key=value, key: value, "key": "value" where key suggests a secret
    let secret_key_patterns = [
        "api_key",
        "apikey",
        "api-key",
        "secret",
        "password",
        "passwd",
        "token",
        "access_key",
        "secret_key",
        "private_key",
        "auth_token",
        "bearer",
        "credential",
    ];

    let mut result = text.to_string();

    for pattern in &secret_key_patterns {
        // Match: PATTERN=value (env var style)
        let env_prefix = format!("{}=", pattern.to_uppercase());
        if let Some(pos) = result.to_uppercase().find(&env_prefix.to_uppercase()) {
            let start = pos + env_prefix.len();
            if let Some(end) = result[start..].find(|c: char| c.is_whitespace() || c == '\n') {
                let end = start + end;
                result.replace_range(start..end, "[REDACTED]");
            } else {
                result.replace_range(start.., "[REDACTED]");
            }
        }
    }

    // Redact common token formats: ghp_, sk-, gho_, xoxb-, Bearer <token>
    let token_prefixes = [
        "ghp_", "gho_", "ghs_", "sk-", "xoxb-", "xoxp-", "sk_live_", "pk_live_",
    ];
    for prefix in &token_prefixes {
        while let Some(pos) = result.find(prefix) {
            let end = result[pos..]
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '\n')
                .map(|e| pos + e)
                .unwrap_or(result.len());
            result.replace_range(pos..end, "[REDACTED]");
        }
    }

    result
}

/// Format bytes into human-readable string.
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_tools_not_empty() {
        let tools = builtin_tools();
        assert!(tools.len() >= 4);
    }

    #[test]
    fn test_find_tool() {
        let tools = builtin_tools();
        assert!(find_tool(&tools, "ghost_search").is_some());
        assert!(find_tool(&tools, "nonexistent").is_none());
    }

    #[test]
    fn test_to_tool_definitions() {
        let tools = builtin_tools();
        let defs = to_tool_definitions(&tools);
        assert_eq!(defs.len(), tools.len());
        for t in &defs {
            assert_eq!(t.tool_type, "function");
            assert!(!t.function.name.is_empty());
        }
    }

    #[test]
    fn test_mcp_tool_conversion() {
        let tool_info = ToolInfo {
            name: "read_file".into(),
            description: Some("Read a file".into()),
            input_schema: Some(
                json!({"type": "object", "properties": {"path": {"type": "string"}}}),
            ),
        };
        let registered = mcp_tool_to_registered("filesystem", &tool_info);
        assert_eq!(registered.definition.function.name, "read_file");
        assert_eq!(registered.source, "mcp:filesystem");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_redact_secrets_env_vars() {
        let input = "API_KEY=sk-abc123xyz SECRET=mysecret other=safe";
        let output = redact_secrets(input);
        assert!(output.contains("[REDACTED]"), "Should redact API_KEY value");
        assert!(
            !output.contains("sk-abc123xyz"),
            "Should not contain the key"
        );
        assert!(
            output.contains("other=safe"),
            "Should keep non-secret values"
        );
    }

    #[test]
    fn test_redact_secrets_token_prefixes() {
        let input = "token is ghp_abc123XYZ456 and sk-proj-abcdef";
        let output = redact_secrets(input);
        assert!(!output.contains("ghp_abc123"), "Should redact GitHub token");
        assert!(
            !output.contains("sk-proj-abcdef"),
            "Should redact OpenAI key"
        );
        assert_eq!(output.matches("[REDACTED]").count(), 2);
    }

    #[test]
    fn test_redact_secrets_no_false_positives() {
        let input = "Hello world\nThis is a normal output\nFiles: api_docs.txt token_counter.py";
        let output = redact_secrets(input);
        assert_eq!(input, output, "Should not modify text without secrets");
    }
}
