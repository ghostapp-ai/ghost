# Ghost Roadmap

> Realistic 12-month plan for a solo developer working 30-40h/week.
> Each phase has a clear business objective before moving to the next.
> The search must be instant and reliable — nothing else matters until Phase 1 ships.
>
> **Market research completed**: See [docs/plans/2026-02-18-market-research.md](docs/plans/2026-02-18-market-research.md)
> Key insight: AI assistant market growing 44% CAGR to $21B by 2030. Rewind (dead), Raycast (Mac-only),
> Microsoft Recall (privacy backlash) all validate our direction. MCP ecosystem (5,800+ servers) ready for integration.

---

## Phase 0 — "Foundation" (Weeks 1-3)

**Goal**: Validate the stack works on your machine before writing UI.

### Technical Deliverables

- [x] **Tauri v2 shell setup**
  - Initialize project with `create-tauri-app` (React + TypeScript)
  - Configure global shortcut plugin (`tauri-plugin-global-shortcut`)
  - Configure filesystem plugin (`tauri-plugin-fs`)
  - Verify <500ms cold start on Windows
  - Test IPC bridge: Rust command → React render

- [x] **Rust core engine scaffold**
  - Create module structure: `indexer/`, `db/`, `embeddings/`, `search/`
  - Set up error handling with `thiserror` + `anyhow`
  - Configure logging with `tracing` crate

- [x] **SQLite + sqlite-vec + FTS5**
  - Integrate `rusqlite` with bundled SQLite
  - Load sqlite-vec extension via `sqlite3_auto_extension` FFI
  - Create schema: `documents`, `chunks`, `chunks_fts` (FTS5), `chunks_vec` (vec0 768-dim)
  - Validate: hybrid query combines FTS5 keyword + vector KNN via RRF
  - 16 unit tests passing

- [x] **Ollama embedding pipeline**
  - HTTP client to Ollama API (`/api/embeddings`)
  - Pull and test `nomic-embed-text` (768 dimensions)
  - Batch embedding: process multiple chunks asynchronously
  - Graceful degradation when Ollama is unavailable

- [x] **Native AI embedding engine (Phase A — Universal)**
  - Replace hard Ollama dependency with native in-process inference
  - Candle (HuggingFace Rust ML) for BERT model execution
  - all-MiniLM-L6-v2 (384D, ~23MB) — runs on any CPU, no internet after first download
  - Fallback chain: Native → Ollama → FTS5-only keyword search
  - Hardware detection: CPU cores, AVX2/NEON SIMD, GPU (CUDA/Metal/Vulkan)
  - Configurable vector dimensions (384 native, 768 Ollama)
  - Frontend StatusBar shows active AI backend (Native/Ollama/Offline)
  - Zero compilation warnings, all tests passing

- [x] **File watcher**
  - Integrate `notify` crate for filesystem events
  - Debounce rapid file changes via `notify-debouncer-mini` (500ms window)
  - `start_watcher` Tauri command spawns background tokio task
  - Automatic re-indexing on file change, de-indexing on file delete

- [x] **Text extraction pipeline**
  - PDF extraction via `lopdf`
  - DOCX extraction via `zip` crate (read `word/document.xml` from DOCX archive)
  - XLSX extraction via `calamine`
  - Plain text / Markdown passthrough
  - Chunking strategy: 512 tokens with 64 token overlap

- [x] **Search via IPC**
  - `search_query` Tauri command → hybrid FTS5 + vector results with RRF ranking
  - Full pipeline validated: file → extract → chunk → embed → store → search
  - Frontend calls via typed `invoke()` wrappers

### Exit Criteria
- [x] `bun run build` compiles frontend with zero errors (181KB JS bundle)
- [x] `cargo test` passes all 21 tests
- [x] `cargo check` compiles with zero errors, zero warnings
- [ ] Insert 100 real documents from your machine *(manual validation pending)*
- [ ] RAM usage idle <50MB *(benchmarking pending)*

---

## Phase 1 — "The Search Bar" (Weeks 4-10)

**Goal**: Launch FREE on Product Hunt, HN, Reddit. Target: 500-1000 real installations.

### Technical Deliverables

- [x] **Global search bar UI**
  - Transparent floating window activated by `Cmd/Ctrl+Space`
  - Tauri global shortcut override at OS level (`tauri-plugin-global-shortcut`)
  - Decorationless, always-on-top, skip-taskbar window (Spotlight-like)
  - Auto-hide on focus loss (blur event)
  - Escape key hides window when query is empty
  - Draggable title region for repositioning
  - Settings persisted to disk (JSON in app data directory)

- [x] **Search input with instant feedback**
  - Debounced input (150ms) triggers hybrid search
  - Loading skeleton while results compute
  - Keyboard navigation: arrow keys, Enter to open, Esc to close
  - Clear button, result count display

- [x] **Results view**
  - Virtualized list with `@tanstack/react-virtual` (handles 10,000+ results)
  - Hybrid ranking: RRF (Reciprocal Rank Fusion) combining FTS5 + vector scores
  - Show: file name, path, snippet with highlighted match, relevance score
  - File type icons (PDF, DOCX, XLSX, TXT, MD, code) and source badges (hybrid/fts/vector)
  - **Open files**: Enter key or double-click opens with system default app
  - Window auto-hides after opening a file

- [x] **Automatic indexing**
  - Background indexing via `start_watcher` Tauri command
  - Watch configured directories for changes (add/remove in Settings)
  - Embeddings stored in sqlite-vec automatically during indexing
  - **Auto-start**: watcher starts automatically on app launch with saved directories
  - First-time onboarding: guides user to add directories when none configured

- [x] **Settings UI**
  - Watched directories management (add/remove) with persistence
  - Manual "Index Now" trigger with progress state
  - Save button persists directories to disk (survives restarts)
  - Status dashboard: files indexed, chunks, Ollama health, vector status
  - Dark theme integrated with Ghost aesthetic

- [x] **Cross-platform installers via GitHub Actions CI/CD**
  - NSIS installer for Windows (x64)
  - DMG for macOS (Apple Silicon + Intel via cross-compilation)
  - DEB + AppImage for Linux (x64)
  - Single consolidated workflow: CI on push + Release on tag
  - CI pipeline: Rust tests + clippy + `cargo fmt --check` + `cargo audit` + frontend TypeScript check
  - ~11MB installer size (Linux .deb)
  - *(Pending)*: Code signing, auto-start option, system tray icon

- [x] **Custom branding & visual identity**
  - Custom Ghost icon (friendly ghost with glowing eyes, purple gradient)
  - Full icon set: rounded, circle, monochrome, tray, wordmark variants
  - Platform-specific icons: Windows (ICO + Square logos), macOS (ICNS), Linux (PNG)
  - Web favicons: SVG, ICO, 16px, 32px, apple-touch-icon
  - Web manifest (`site.webmanifest`) for PWA metadata
  - Brand guidelines document (`branding/BRAND_GUIDELINES.md`)
  - Icon generation script (`branding/scripts/generate-icons.sh`)
  - Social media assets: OG card, GitHub avatar

- [x] **Repository configuration & developer tooling**
  - `.editorconfig` for consistent code style across editors
  - `rustfmt.toml` + `clippy.toml` for Rust linting/formatting
  - GitHub Issue templates (bug report + feature request)
  - GitHub Pull Request template with privacy checklist
  - `dependabot.yml` for automated dependency updates
  - `CODEOWNERS` for PR review assignment
  - `CONTRIBUTING.md` + `SECURITY.md`
  - VS Code recommended extensions

- [x] **Native chat engine (local LLM inference)**
  - Hardware-aware model auto-selection based on CPU, RAM, GPU
  - Model registry: Qwen2.5-Instruct GGUF family (0.5B/1.5B/3B/7B, Q4_K_M)
  - Candle GGUF inference engine with ChatML prompt format
  - Device selection: CPU (default), CUDA (feature flag), Metal (feature flag)
  - Auto-download models from HuggingFace Hub on first use
  - Zero-config flow: detect hardware → recommend model → background download → ready
  - Settings overrides: model, device, max_tokens, temperature (all with serde defaults)
  - Fallback chain: Native Candle → Ollama HTTP → None
  - Chat UI: message bubbles, model status, loading states, error handling
  - Debug panel: collapsible log viewer with pause/resume, clear, color-coded levels
  - Tab system in App.tsx: Search and Chat modes with keyboard shortcuts (Ctrl+1/2)
  - StatusBar with chat/model status indicator alongside existing AI/Vec indicators
  - RAM detection: Linux (/proc/meminfo), macOS (sysctl+vm_stat), Windows (PowerShell)
  - Per-request model reload for clean KV cache (no public clear method in quantized_qwen2)
  - Background model loading via `tokio::spawn` in `.setup()` — doesn't block app startup

- [ ] **Performance optimization**
  - Cold start <500ms
  - Search results <100ms for keyword, <500ms for semantic
  - Idle RAM <40MB
  - Background indexing uses <10% CPU

### Exit Criteria
- [x] Installers generated for Windows, macOS, Linux (~11MB)
- [ ] 500+ real installations within 60 days of launch
- [ ] Search feels instant for keyword queries
- [ ] Users return after day 7 (>30% retention)

---

## Phase 1.5 — "MCP Bridge" (Weeks 8-10)

**Goal**: Make Ghost accessible from any MCP-compatible AI client (Claude Desktop, Cursor, etc.). Instant ecosystem integration.

> Added after market research (Feb 2026). MCP ecosystem has 5,800+ servers. Being MCP-compatible
> is a competitive differentiator that connects Ghost to the entire AI tool ecosystem immediately.

### Technical Deliverables

- [ ] **Basic MCP server**
  - HTTP server via `axum` on localhost (configurable port)
  - MCP protocol v2025-11-25 compliance
  - Tool: `ghost_search` — hybrid search across indexed files
  - Tool: `ghost_index_status` — report indexing stats
  - Resource: expose indexed documents metadata

- [ ] **Integration testing**
  - Test with Claude Desktop as MCP client
  - Test with Cursor as MCP client
  - Documentation: "How to connect Ghost to Claude Desktop"

### Exit Criteria
- [ ] Claude Desktop can search local files through Ghost MCP
- [ ] <100ms overhead added by MCP layer
- [ ] Setup guide published

---

## Phase 2 — "The Memory" (Weeks 11-18)

**Goal**: 500 users → first paying users. Launch $5-8/mo Pro tier. Target: 50-100 paying.

> Pricing adjusted after market research: Raycast charges $8/mo, ChatGPT $20/mo.
> Ghost Pro at $5-8/mo undercuts both while offering unique local AI search.

### Technical Deliverables

- [ ] **Browser history indexing**
  - Windows: read Chrome/Edge SQLite history DB
  - Index page titles and URLs with timestamps
  - Respect browser private/incognito mode

- [ ] **App activity via UI Automation**
  - Windows: `uiautomation` Rust crate for reading control trees
  - Capture active window title, focused control text
  - Activity timeline: "What was I doing at 3pm yesterday?"
  - <1% CPU overhead, sample every 5 seconds

- [ ] **Clipboard history**
  - Monitor clipboard changes
  - Store text clips with timestamp and source app
  - Semantic search across clipboard history
  - Privacy: configurable exclusion rules (e.g., password managers)

- [ ] **Activity timeline UI**
  - Chronological view of all indexed activity
  - Filter by: date range, app, content type
  - Natural language queries: "show me what I worked on last Tuesday"

- [ ] **Premium features (paid tier)**
  - Vault encryption with ChaCha20-Poly1305 (age crate)
  - Encrypted sync between devices (optional, user-controlled)
  - Access to more powerful local models (Qwen2.5-14B)
  - Priority support

- [ ] **Licensing system**
  - License key validation (offline-capable)
  - Free tier: core search + 3 watched directories
  - Premium tier ($5-8/mo): unlimited directories, memory, encryption, sync

### Exit Criteria
- [ ] Activity timeline shows accurate history
- [ ] 50-100 paying users at $9/mo
- [ ] Encryption passes basic security review
- [ ] Mac investigation started (WebKit compatibility testing)

---

## Phase 3 — "The Agent" (Weeks 19-30)

**Goal**: Launch Ghost Pro at $15-20/mo. Target: 200-500 paying users ($3,000-10,000 MRR).

### Technical Deliverables

- [ ] **Local MCP Server**
  - HTTP server via `axum` (Rust) exposing MCP protocol
  - Tool definitions: file operations, app control, web search, clipboard
  - Runs on localhost, <5MB RAM overhead

- [ ] **LLM integration for tool calling**
  - Ollama + Qwen2.5-7B Q4 for reasoning and tool selection
  - Structured output parsing for tool call arguments
  - Fallback: Claude API for complex tasks (user opt-in, paid tier)

- [ ] **Agent actions**
  - Open/focus applications
  - Copy text to clipboard
  - Create/edit files
  - Search the web (via default browser)
  - Send keyboard shortcuts to active window

- [ ] **Action Preview (safety layer)**
  - Before executing, Ghost shows what it will do
  - Step-by-step action plan with confirmation
  - Undo support for reversible actions
  - Audit log of all executed actions

- [ ] **Chat interface**
  - Streaming token display via Tauri events
  - Context-aware: Ghost knows your recent files and activity
  - Conversation history (stored locally)

- [ ] **Ghost Pro tier**
  - Agent capabilities (tool calling + actions)
  - Advanced models and longer context
  - Team sharing features (shared vaults)

### Exit Criteria
- [ ] Agent can reliably execute 5+ action types
- [ ] Action Preview shows correct plan >95% of the time
- [ ] 200+ paying users
- [ ] MRR reaches $3,000+

---

## Phase 4 — "The Platform" (Months 8-12)

**Goal**: $10,000+ MRR. Partnerships. Explore B2B/teams model.

### Technical Deliverables

- [ ] **Skills SDK**
  - Documented API for creating MCP servers that integrate with Ghost
  - NPM package: `@ghost/skills-sdk`
  - Rust crate: `ghost-skills`
  - Example skills: GitHub integration, Notion sync, Slack search

- [ ] **Third-party integrations**
  - Obsidian vault indexing
  - VS Code extension (Ghost as search backend, side panel)
  - Slack message search
  - Browser extension for page content indexing

- [ ] **Mac port**
  - macOS build with Tauri v2 (WebKit)
  - Accessibility API via AXUIElement for UI automation
  - Spotlight-like search bar behavior
  - Code signing + notarization for distribution

- [ ] **B2B/Teams features**
  - Shared team vaults with role-based access
  - Admin dashboard for managing team licenses
  - Compliance features: audit trail, data retention policies
  - SSO integration

### Exit Criteria
- [ ] 3-5 third-party skills published
- [ ] Mac version stable
- [ ] $10,000+ MRR
- [ ] At least one B2B pilot customer

---

## Pivot Signals

If the primary consumer plan underperforms, these pivots use the same core technology:

| Signal | Threshold | Pivot |
| ------ | --------- | ----- |
| Low installations | <200 in 60 days | **Pivot A**: Dev tools skin, distribute via VS Code Marketplace |
| No retention | >70% churn after day 7 | Product problem, not marketing. Talk to users who stayed |
| Low conversion | <1% free→paid in 3 months | **Pivot B**: Target SMBs for compliance ($200-500/mo/company) |
| Platform blocked | Apple/Microsoft block accessibility APIs | Fallback to OCR. Focus on Linux/Windows corporate |

### Pivot A — Dev Tools (Minimum Risk)

Same product, different skin: semantic search across code, logs, docs, and project context. Compete with Sourcegraph/Cursor but 100% local. Distribute via VS Code Marketplace to 500k+ devs.

### Pivot B — Compliance for SMBs (Higher Revenue)

The indexing + search engine becomes a data discovery tool: detect PII, map sensitive data, generate audit trails. Sell to Data Protection Officers at $200-500/mo per company. 10x the revenue per customer.

### Pivot C — MCP Infrastructure Provider (B2B Pure)

Sell the local MCP server infrastructure to enterprises that want their AI agents to have context from employee PCs without sending data to the cloud. SaaS B2B at $2,000-10,000/year per company.

---

## Key Technical References

| Resource | URL | Notes |
| -------- | --- | ----- |
| Tauri v2 | tauri.app | v2.10.2 stable. Plugins: global-shortcut, fs, shell |
| sqlite-vec | github.com/asg017/sqlite-vec | SIMD-accelerated KNN. Works with FTS5 |
| Ollama | ollama.com | Local LLM runtime. Supports Qwen2.5 tool calling |
| nomic-embed-text | ollama.com/library/nomic-embed-text | 768 dims, surpasses ada-002, ~274MB |
| MCP Spec | modelcontextprotocol.io | v2025-11-25 spec. Linux Foundation / AAIF |
| uiautomation | crates.io/crates/uiautomation | Windows UI Automation wrapper for Rust |
| notify | crates.io/crates/notify | Cross-platform filesystem watcher |
| candle | github.com/huggingface/candle | Rust ML framework, BERT/LLM inference |
| all-MiniLM-L6-v2 | huggingface.co/sentence-transformers/all-MiniLM-L6-v2 | 384 dims, 23MB, excellent quality |
| hf-hub | crates.io/crates/hf-hub | HuggingFace model download/cache |
| mcp-desktop-automation | github.com/tanob/mcp-desktop-automation | Reference MCP server for desktop control |
| Screenpipe | github.com/mediar-ai/screenpipe | Architecture reference (not code) |
