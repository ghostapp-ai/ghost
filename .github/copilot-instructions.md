# Ghost — Copilot Instructions

## Project Overview

Ghost is a privacy-first AI desktop assistant built with Tauri v2 (Rust backend) + React/TypeScript/Vite (frontend). All AI inference runs locally via Candle + GGUF models. No cloud dependencies for core functionality.

## Tech Stack

- **Framework:** Tauri v2
- **Backend:** Rust (edition 2021)
- **Frontend:** React + TypeScript + Vite
- **Database:** SQLite via rusqlite (FTS5 for full-text search)
- **ML:** Candle (Hugging Face) — all-MiniLM-L6-v2 for embeddings (384D), Qwen2.5-Instruct GGUF for generation
- **Agent:** ReAct engine with grammar-constrained tool calling
- **Protocols:** MCP (rmcp crate), AG-UI, A2UI, A2A
- **Licensing:** ghost-pro crate (freemium feature gating)

## Build & Test

```bash
# Build
cargo build --release

# Run in dev mode (Tauri)
cargo tauri dev

# Run tests
cargo test

# Run specific test
cargo test test_name

# Clippy lints
cargo clippy -- -D warnings

# Frontend dev
cd frontend && pnpm dev

# Frontend build
cd frontend && pnpm build
```

## Module Structure

```
src-tauri/
├── src/
│   ├── main.rs              # Tauri app entry, window setup
│   ├── lib.rs               # Core library exports
│   ├── agent/               # ReAct agent engine
│   │   ├── mod.rs            # Agent orchestrator
│   │   ├── react.rs          # ReAct loop (think → act → observe)
│   │   ├── grammar.rs        # Grammar-constrained generation
│   │   └── tools/            # Built-in tool implementations
│   ├── search/               # Hybrid search
│   │   ├── fts.rs            # FTS5 full-text search (<5ms)
│   │   ├── semantic.rs       # Vector search via Candle embeddings
│   │   └── hybrid.rs         # Combined ranking
│   ├── mcp/                  # MCP Server + Client
│   │   ├── server.rs         # Ghost as MCP server
│   │   ├── client.rs         # Connect to external MCP servers
│   │   └── catalog.rs        # 30+ curated + registry (6000+)
│   ├── extraction/           # File processing
│   │   ├── watcher.rs        # File system watcher
│   │   └── extractors/       # PDF, DOCX, 50+ formats
│   ├── models/               # Local AI model management
│   │   ├── loader.rs         # GGUF model loading via Candle
│   │   └── inference.rs      # Text generation pipeline
│   ├── db/                   # Database layer
│   │   ├── mod.rs            # Connection pool, migrations
│   │   ├── schema.rs         # Table definitions
│   │   └── migrations/       # SQL migrations
│   └── pro/                  # ghost-pro crate integration
│       ├── license.rs        # License key validation
│       └── features.rs       # Feature gating logic
├── Cargo.toml
└── tauri.conf.json
```

## Rust Conventions

- **Error handling:** Use `anyhow::Result` for application errors, `thiserror` for library errors. Never `unwrap()` in production paths.
- **Async:** Tokio runtime (Tauri v2 default). Use `#[tauri::command]` for frontend-callable functions.
- **Tauri commands:** All Tauri commands go in dedicated command modules, registered in `main.rs` via `invoke_handler`.
- **Database:** All SQL goes through rusqlite with parameterized queries. Never interpolate strings into SQL.
- **Logging:** Use `tracing` crate with structured fields. `tracing::info!`, `tracing::error!`, etc.
- **Testing:** Unit tests in the same file (`#[cfg(test)]` module). Integration tests in `tests/` directory.

## How to Add a New Tool

Tools are functions the ReAct agent can call during reasoning.

1. Create the tool implementation in `src/agent/tools/`:
```rust
// src/agent/tools/my_tool.rs
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct MyToolInput {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct MyToolOutput {
    pub result: String,
}

pub async fn execute(input: MyToolInput) -> Result<MyToolOutput> {
    // Implementation here
    Ok(MyToolOutput { result: "done".into() })
}
```

2. Register in `src/agent/tools/mod.rs`:
```rust
pub mod my_tool;

// Add to the tool registry
registry.register("my_tool", ToolDef {
    name: "my_tool",
    description: "What this tool does",
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "The search query" }
        },
        "required": ["query"]
    }),
    handler: |input| Box::pin(my_tool::execute(serde_json::from_value(input)?)),
});
```

3. Add tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_tool_basic() {
        let input = MyToolInput { query: "test".into() };
        let output = execute(input).await.unwrap();
        assert!(!output.result.is_empty());
    }
}
```

4. Run `cargo test` and `cargo clippy` before committing.

## How to Add a New MCP Server

MCP servers extend Ghost's capabilities via the Model Context Protocol.

1. Add to curated catalog in `src/mcp/catalog.rs`:
```rust
CatalogEntry {
    id: "my-server",
    name: "My Server",
    description: "What it does",
    transport: Transport::Stdio {
        command: "npx",
        args: &["-y", "@my-org/mcp-server"],
    },
    // OR for SSE:
    // transport: Transport::Sse { url: "https://my-server.example.com/sse" },
    category: Category::Productivity,
    requires_config: vec!["API_KEY"],  // env vars needed
    tier: Tier::Free,  // or Tier::Pro
}
```

2. For programmatic MCP client connections:
```rust
use crate::mcp::client::McpClient;

let client = McpClient::connect("my-server", transport_config).await?;
let tools = client.list_tools().await?;
let result = client.call_tool("tool_name", params).await?;
```

3. Test with `cargo test mcp` and verify the server appears in the UI catalog.

## Deploy & Release

- **CI/CD:** GitHub Actions builds for Windows (.msi), macOS (.dmg), Linux (.AppImage)
- **Release:** Tag with `vX.Y.Z`, push triggers release workflow
- **Signing:** macOS requires notarization credentials in GitHub Secrets
- **Auto-update:** Tauri v2 updater plugin checks GitHub Releases

## Key Dependencies

| Crate | Purpose |
|---|---|
| `tauri` | Desktop framework (v2) |
| `rusqlite` | SQLite + FTS5 |
| `candle-core` / `candle-nn` | Local ML inference |
| `candle-transformers` | Model architectures |
| `rmcp` | MCP protocol client/server |
| `tokenizers` | Tokenization for embeddings |
| `serde` / `serde_json` | Serialization |
| `tokio` | Async runtime |
| `tracing` | Structured logging |
| `anyhow` / `thiserror` | Error handling |

## Common Pitfalls

- **Candle models:** Always load on a dedicated thread — model loading blocks and is heavy. Use `tokio::task::spawn_blocking`.
- **FTS5 queries:** Escape special characters in user input before passing to FTS5 MATCH.
- **Tauri commands:** Must return `Result<T, String>` (Tauri serialization constraint). Wrap errors with `.map_err(|e| e.to_string())`.
- **Database migrations:** Always add new migrations, never modify existing ones. Run in order on first launch.
- **Feature gating:** Check `ghost_pro::is_feature_enabled("feature_name")` before executing Pro-tier logic. Free tier must never crash when Pro features are unavailable.
