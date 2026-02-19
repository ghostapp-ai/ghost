# Ghost — Agent Development Instructions

> This file defines conventions, architecture rules, and workflows for AI-assisted development of Ghost.
> All agents (Claude, Copilot, or any AI assistant) MUST follow these instructions.

---

## Project Overview

**Ghost** is a private, local-first AI assistant for desktop (Windows → macOS → Linux). It indexes local files, provides hybrid semantic + keyword search, and evolves into an AI agent that can take actions on the user's OS — all without sending data to the cloud.

- **Current phase**: Phase 0 — Foundation
- **Stack**: Tauri v2 (Rust backend) + React/TypeScript (frontend) + SQLite/sqlite-vec + Ollama
- **Priority**: Make search instant and reliable. Nothing else matters until Phase 1 ships.

---

## Critical Rules

### 1. Never Break Privacy
- NEVER add telemetry, analytics, or any external network calls (except to localhost Ollama)
- NEVER include tracking pixels, error reporting services, or crash analytics
- All data processing MUST happen locally
- If a feature requires cloud access, it MUST be opt-in and clearly documented

### 2. Performance is Non-Negotiable
- App cold start: <500ms
- FTS5 keyword search: <5ms
- Semantic vector search: <500ms
- Idle RAM: <40MB
- Background indexing CPU: <10%
- Always benchmark before and after changes that touch search or indexing

### 3. Always Update the 3 Core Documents
After every significant change, update these files to reflect current state:
- **README.md** — Project description, features, architecture, getting started
- **ROADMAP.md** — Check off completed items, add new tasks discovered during implementation
- **CLAUDE.md** — Update conventions, add new patterns, document decisions

### 4. Research Before Implementing
- Before using a new crate or npm package, verify it exists and check its latest version
- Validate compatibility with our stack (Tauri v2, Rust 2021 edition, React 18)
- Check for security advisories
- Prefer well-maintained crates with >100 GitHub stars

### 5. Commits Must Be Professional
- Use conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Each commit should be atomic — one logical change per commit
- Always include what changed and why in the commit message
- Never commit secrets, API keys, or sensitive data

---

## Architecture Rules

### Rust Backend (`src-tauri/`)

```
src-tauri/src/
├── lib.rs              # Tauri app builder, plugin registration, command handlers
├── main.rs             # Entry point (DO NOT modify beyond run())
├── indexer/
│   ├── mod.rs          # Public API for indexing module
│   ├── watcher.rs      # File system watcher (notify crate)
│   ├── extractor.rs    # Text extraction (PDF, DOCX, XLSX, TXT)
│   └── chunker.rs      # Text chunking strategy (512 tokens, 64 overlap)
├── db/
│   ├── mod.rs          # Database initialization and migrations
│   ├── schema.rs       # Table definitions and migrations
│   ├── documents.rs    # Document CRUD operations
│   └── search.rs       # FTS5 + sqlite-vec hybrid search queries
├── embeddings/
│   ├── mod.rs          # EmbeddingEngine (fallback chain: Native → Ollama → None)
│   ├── native.rs       # Candle-based in-process BERT inference (all-MiniLM-L6-v2)
│   ├── ollama.rs       # OllamaEngine HTTP client (fallback engine)
│   └── hardware.rs     # Hardware detection (CPU cores, SIMD, GPU backend)
├── search/
│   ├── mod.rs          # Search engine combining FTS5 + vector
│   └── ranking.rs      # RRF (Reciprocal Rank Fusion) implementation
├── mcp/                # (Phase 3) MCP server implementation
│   ├── mod.rs
│   ├── server.rs       # axum HTTP server exposing MCP protocol
│   └── tools.rs        # Tool definitions and handlers
└── automation/         # (Phase 2+) OS UI automation
    ├── mod.rs
    ├── windows.rs      # uiautomation crate wrapper
    └── macos.rs        # AXUIElement wrapper (future)
```

#### Rust Conventions
- Use `thiserror` for library errors, `anyhow` for application errors
- All async operations use `tokio` runtime (Tauri's default)
- Database access through a connection pool — never hold connections across await points
- Expose functionality to frontend via `#[tauri::command]` functions in `lib.rs`
- Use `tracing` for structured logging (not `println!` or `log`)
- All public functions must have doc comments
- Use `Result<T, E>` return types — never `unwrap()` in production code (only in tests)

#### Key Rust Crates
```toml
# Core
tauri = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }

# Database
rusqlite = { version = "0.32", features = ["bundled", "vtab"] }
# sqlite-vec loaded as extension at runtime

# File watching
notify = "7"

# Text extraction
lopdf = "0.34"           # PDF
zip = "2"                # DOCX
calamine = "0.26"        # XLSX

# Native AI inference (in-process, no external deps)
candle-core = "0.9"      # Tensor operations
candle-nn = "0.9"        # Neural network layers
candle-transformers = "0.9" # BERT, GPT, etc.
hf-hub = "0.4"           # Model download from HuggingFace
tokenizers = "0.22"      # Fast text tokenization

# HTTP (for Ollama fallback + MCP server)
reqwest = { version = "0.12", features = ["json"] }
axum = "0.8"             # (Phase 3, MCP server)

# Error handling
thiserror = "2"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Encryption (Phase 2+)
# age = "0.10"           # ChaCha20-Poly1305
```

### Frontend (`src/`)

```
src/
├── App.tsx              # Root component, router setup
├── main.tsx             # React entry point
├── components/
│   ├── SearchBar.tsx    # Global search input
│   ├── ResultsList.tsx  # Virtualized search results
│   ├── ResultItem.tsx   # Single search result row
│   ├── ChatPanel.tsx    # (Phase 3) Agent chat interface
│   ├── Settings.tsx     # Settings panel
│   └── VaultBrowser.tsx # File browser for indexed vault
├── hooks/
│   ├── useSearch.ts     # Search query + results state
│   ├── useTauriCommand.ts  # Generic Tauri IPC wrapper
│   └── useHotkey.ts    # Global shortcut handling
├── stores/
│   └── appStore.ts     # Global app state (zustand or similar)
├── lib/
│   ├── tauri.ts        # Tauri invoke wrappers with types
│   └── types.ts        # Shared TypeScript types
└── styles/
    └── globals.css     # Global styles (Tailwind or vanilla CSS)
```

#### Frontend Conventions
- React 18 with functional components only — no class components
- TypeScript strict mode — no `any` types
- State management: start with React context, migrate to Zustand if needed
- Styling: Tailwind CSS preferred. If not installed, use CSS modules
- All Tauri commands wrapped in typed async functions in `lib/tauri.ts`
- Use `react-virtual` for any list that could exceed 100 items
- Keyboard navigation must work everywhere — Ghost is a keyboard-first app
- Accessibility: all interactive elements need proper ARIA labels

#### Frontend Rules
- The frontend is "thin" — all heavy logic lives in Rust
- Never process files or run AI in the frontend
- Communication with Rust only via Tauri IPC (`invoke()`)
- No external API calls from frontend (privacy rule)
- Bundle size budget: <500KB JS (excluding Tauri runtime)

---

## Database Schema

### Core Tables

```sql
-- Documents table: one row per indexed file
CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    extension TEXT,
    size_bytes INTEGER,
    hash TEXT NOT NULL,              -- SHA-256 for change detection
    indexed_at TEXT NOT NULL,        -- ISO 8601
    modified_at TEXT NOT NULL,       -- File's mtime
    metadata TEXT                    -- JSON blob for extra info
);

-- Chunks table: document split into embeddable pieces
CREATE TABLE chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,    -- Order within document
    content TEXT NOT NULL,           -- Raw text content
    token_count INTEGER,
    UNIQUE(document_id, chunk_index)
);

-- FTS5 virtual table for keyword search
CREATE VIRTUAL TABLE chunks_fts USING fts5(
    content,
    content=chunks,
    content_rowid=id,
    tokenize='porter unicode61'
);

-- Vector table via sqlite-vec for semantic search
CREATE VIRTUAL TABLE chunks_vec USING vec0(
    chunk_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]             -- all-MiniLM-L6-v2 (native) or FLOAT[768] (Ollama)
);
```

### Search Query Pattern (Hybrid RRF)

```sql
-- 1. FTS5 keyword search (fast, <5ms)
SELECT rowid, rank FROM chunks_fts WHERE chunks_fts MATCH ?;

-- 2. KNN vector search (semantic, <500ms)
SELECT chunk_id, distance FROM chunks_vec
WHERE embedding MATCH ? ORDER BY distance LIMIT 20;

-- 3. Combine with RRF in Rust (not SQL)
-- RRF score = sum(1 / (k + rank_i)) for each ranking system
-- k = 60 (standard RRF constant)
```

---

## Development Workflow

### Before Starting Any Task
1. Read `ROADMAP.md` to understand current phase and priorities
2. Check if the task aligns with current phase goals
3. If the task requires a new dependency, research it first

### While Working
1. Write tests for new Rust modules (`#[cfg(test)]` inline or `tests/` dir)
2. Run `cargo clippy` before committing Rust code
3. Run `bun run build` to verify frontend compiles
4. Test on Windows first (primary target)

### After Completing a Task
1. Update the three core documents (README.md, ROADMAP.md, CLAUDE.md)
2. Create a professional commit with conventional commit format
3. Verify the app still builds: `bun run tauri build`

### Testing
```bash
# Rust tests
cd src-tauri && cargo test

# Rust linting
cd src-tauri && cargo clippy -- -D warnings

# Frontend type checking
bun run build

# Full app dev mode
bun run tauri dev
```

---

## Tauri v2 IPC Pattern

All communication between frontend and backend uses Tauri commands:

### Rust Side (defining a command)
```rust
#[tauri::command]
async fn search(query: String, limit: Option<usize>) -> Result<Vec<SearchResult>, String> {
    let limit = limit.unwrap_or(20);
    let results = search::hybrid_search(&query, limit)
        .await
        .map_err(|e| e.to_string())?;
    Ok(results)
}

// Register in lib.rs:
.invoke_handler(tauri::generate_handler![search])
```

### Frontend Side (calling a command)
```typescript
// lib/tauri.ts
import { invoke } from "@tauri-apps/api/core";

export interface SearchResult {
  id: number;
  path: string;
  filename: string;
  snippet: string;
  score: number;
}

export async function search(query: string, limit?: number): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search", { query, limit });
}
```

---

## Embedding Engine Architecture

Ghost uses a fallback chain for embeddings: **Native → Ollama → FTS5-only**

### Native Engine (Primary — Zero Dependencies)
```rust
// embeddings/native.rs — runs in-process via Candle
// Model: all-MiniLM-L6-v2 (384 dimensions, ~23MB safetensors)
// Downloads once from HuggingFace Hub, cached locally
// Works on any CPU — no GPU, Ollama, or internet required after first run
let engine = NativeEngine::load().await?;
let embedding: Vec<f32> = engine.embed("search query")?; // 384-dim
```

### Ollama Engine (Fallback — Optional)
```rust
// embeddings/ollama.rs — HTTP client to localhost Ollama
const OLLAMA_BASE: &str = "http://localhost:11434";
const EMBEDDING_MODEL: &str = "nomic-embed-text"; // 768 dimensions
```

### Unified Engine (mod.rs)
```rust
// embeddings/mod.rs — tries Native first, falls back to Ollama
let engine = EmbeddingEngine::initialize().await;
// engine.backend() returns AiBackend::Native, ::Ollama, or ::None
let embedding = engine.embed("query").await?;
```

### Important AI Engine Notes
- Ghost works WITHOUT Ollama — native Candle engine is the primary backend
- Ollama is a fallback for users who want larger/better models (nomic-embed-text 768D)
- First app launch downloads the native model (~23MB) from HuggingFace Hub (requires internet once)
- Subsequent launches load the cached model in <200ms
- Embedding calls in NativeEngine are synchronous (no HTTP overhead)
- If both Native and Ollama fail, Ghost falls back to FTS5 keyword-only search
- Model availability checked at startup and reported in StatusBar
- For Phase 3 (agent): use Qwen2.5-7B with tool calling via Ollama `/api/chat`

---

## File Naming Conventions

| Type | Convention | Example |
| ---- | ---------- | ------- |
| Rust modules | snake_case | `file_watcher.rs` |
| Rust types | PascalCase | `SearchResult` |
| Rust functions | snake_case | `hybrid_search()` |
| React components | PascalCase | `SearchBar.tsx` |
| React hooks | camelCase with `use` prefix | `useSearch.ts` |
| TypeScript utils | camelCase | `formatBytes.ts` |
| CSS files | kebab-case or component name | `search-bar.css` |
| Config files | kebab-case | `tauri.conf.json` |

---

## Git Conventions

### Branch Naming
```
feature/search-bar
feature/file-watcher
fix/fts5-unicode-tokenizer
refactor/db-connection-pool
docs/update-roadmap
```

### Commit Message Format
```
<type>(<scope>): <description>

[optional body with more details]

[optional footer with references]
```

#### Types
- `feat` — New feature
- `fix` — Bug fix
- `refactor` — Code change that neither fixes a bug nor adds a feature
- `docs` — Documentation only
- `test` — Adding or updating tests
- `chore` — Build process, tooling, or dependency updates
- `perf` — Performance improvement

#### Examples
```
feat(search): implement hybrid FTS5 + vector search with RRF ranking

Combines SQLite FTS5 keyword results with sqlite-vec KNN results
using Reciprocal Rank Fusion (k=60). Returns top 20 results
sorted by combined score.

Refs: ROADMAP.md Phase 1
```

```
fix(indexer): handle PDF files with encrypted content gracefully

Previously crashed on password-protected PDFs. Now skips them
and logs a warning with the file path.
```

---

## Environment Setup

### Required Tools
- **Rust**: latest stable via `rustup`
- **Node.js/Bun**: Bun >= 1.0 preferred (used in tauri.conf.json)
- **Ollama**: installed and running on localhost:11434
- **Tauri v2 CLI**: `bun add -D @tauri-apps/cli`
- **Platform dependencies**: see https://v2.tauri.app/start/prerequisites/

### Environment Variables
Ghost does NOT use environment variables for configuration. All settings are stored locally in:
- **Windows**: `%APPDATA%/com.ghost.app/config.json`
- **macOS**: `~/Library/Application Support/com.ghost.app/config.json`
- **Linux**: `~/.config/com.ghost.app/config.json`

### Ollama Models (Optional)
```bash
# Optional: for higher-quality 768D embeddings
ollama pull nomic-embed-text    # Embeddings (fallback)
ollama pull qwen2.5:7b          # Reasoning + tool calling (Phase 3)
```

---

## Decision Log

| Date | Decision | Rationale |
| ---- | -------- | --------- |
| 2026-02-18 | Tauri v2 over Electron | 70% less RAM, <10MB installer, Rust backend, mobile future |
| 2026-02-18 | SQLite + sqlite-vec over dedicated vector DB | Single file, zero infra, FTS5 + vectors in one query |
| 2026-02-18 | nomic-embed-text over OpenAI ada-002 | Free, local, 768 dims, surpasses ada-002 on benchmarks |
| 2026-02-18 | MCP over custom tool protocol | Open standard, 10,000+ servers, Linux Foundation backed |
| 2026-02-18 | Windows-first over Mac-first | 73% of PCs, Raycast/Alfred are Mac-only, market gap |
| 2026-02-18 | Bun over npm | Faster installs, native TypeScript, used in tauri.conf.json |
| 2026-02-18 | Candle over Burn/ONNX for embeddings | Same org as HF Hub/tokenizers, mature BERT support, pure Rust |
| 2026-02-18 | all-MiniLM-L6-v2 over nomic-embed-text for native | 384D vs 768D, 23MB vs 274MB, faster, no external deps |
| 2026-02-18 | Fallback chain over hard Ollama dep | Ghost works offline/without Ollama, graceful degradation |
| 2026-02-18 | GitHub Actions + tauri-action for CI/CD | Cross-platform builds (Win/Mac/Linux), automated releases on tag push |
| 2026-02-18 | softprops/action-gh-release for releases | Mature, supports draft/prerelease, auto-attaches artifacts |
| 2026-02-18 | Dependabot for dependency updates | Automated weekly PRs for Cargo, npm, and GitHub Actions |
| 2026-02-18 | cargo audit in CI pipeline | Security scanning for Rust dependencies on every push/PR |
| 2026-02-18 | Custom Ghost branding over default Tauri icons | Distinctive identity, professional look for store listings |
