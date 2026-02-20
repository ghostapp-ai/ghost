# Ghost Roadmap

> Realistic 12-month plan for a solo developer working 30-40h/week.
> Each phase has a clear business objective before moving to the next.
> The search must be instant and reliable — nothing else matters until Phase 1 ships.
>
> **Market research completed (Feb 2026)**.
> Key insight: AI assistant market $3.35B (2025) → $24.53B (2034) at 24.8% CAGR. Rewind (dead), Raycast (Mac-only),
> Microsoft Recall (privacy backlash) all validate our direction. MCP ecosystem (10,000+ servers) ready for integration.
>
> **Vision update (Feb 2026)**: Ghost evolves from "search bar" to **Agent OS** — a local-first Universal Protocol Hub
> that speaks MCP + A2A + AG-UI + A2UI + WebMCP, connecting users to the entire AI agent ecosystem
> while keeping all data private. Not replacing the file explorer — sitting above the OS as its intelligence layer.
>
> **Protocol stack**: MCP (tools) → A2A (agent-to-agent) → AG-UI (agent↔user runtime) → A2UI/MCP Apps (generative UI) → WebMCP (web agents)

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
- [x] `cargo test` passes all 27 tests
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
  - **Zero-config auto-discovery**: on first launch, auto-detects user directories (Documents, Desktop, Downloads, Pictures) using `dirs` crate — like Spotlight/Alfred
  - Cross-platform directory detection: XDG dirs on Linux, standard dirs on Windows/macOS, Spanish locale fallbacks
  - Automatic background indexing + file watcher setup on first run
  - Settings panel still available for manual customization
  - Source code file support: 50+ programming languages indexed (rs, py, js, ts, go, etc.)

- [x] **Settings UI**
  - Watched directories management (add/remove) with persistence
  - Manual "Index Now" trigger with progress state
  - Save button persists directories to disk (survives restarts)
  - Status dashboard: files indexed, chunks, Ollama health, vector status
  - Dark theme integrated with Ghost aesthetic

- [x] **Cross-platform installers via GitHub Actions CI/CD**
  - NSIS installer for Windows (x64) with language selector, custom icons, WebView2 bootstrap
  - DMG for macOS (Apple Silicon + Intel via cross-compilation) with custom positioning
  - DEB + RPM + AppImage for Linux (x64)
  - CI pipeline: Rust tests + clippy + `cargo fmt --check` + `cargo audit` + frontend TypeScript check
  - ~11MB installer size (Linux .deb)
  - *(Complete)*: System tray icon, onboarding wizard, professional installer config

- [x] **Fully automatic release pipeline (semantic-release)**
  - 100% automatic: push conventional commits → CI → release. Zero interaction needed
  - semantic-release analyzes commits → bumps SemVer → generates CHANGELOG.md
  - Creates git tag + GitHub Release + attaches cross-platform installers
  - Version sync across `package.json`, `Cargo.toml`, `tauri.conf.json` via custom script
  - No PRs to merge, no tags to create — just push code and it ships

- [x] **Repository configuration & best practices**
  - Auto-delete merged branches
  - Auto-merge enabled for PRs
  - Squash merge with PR title/body for clean git history
  - Vulnerability alerts + automated security fixes enabled
  - GitHub Actions allowed to create/approve PRs
  - Repository topics for discoverability (ai, local-first, privacy, tauri, etc.)

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

- [x] **Model download progress tracking**
  - Filesystem monitoring of HF Hub cache `.incomplete` files every 500ms
  - DownloadProgress struct with phases: checking_cache, downloading, loading_model, cached
  - Visual progress bar with shimmer animation, MB counters, percentage display
  - Reported via `chat_status` polling (2s interval)

- [x] **First-launch onboarding wizard**
  - Multi-step setup shown only on first launch (`setup_complete` flag in Settings)
  - Phase 1: Welcome screen with Ghost branding and auto-advance
  - Phase 2: Hardware auto-detection (CPU cores, RAM, GPU, SIMD capabilities)
  - Phase 3: Recommended model display with specs, one-click download button
  - Phase 4: Real-time download progress with tips carousel while waiting
  - Phase 5: Setup complete summary with keyboard shortcut and privacy info
  - Skip button for power users — marks setup as complete immediately
  - Onboarding component in `src/components/Onboarding.tsx`
  - `complete_setup` Tauri command persists `setup_complete` to settings.json
  - App.tsx routes: loading → onboarding → main UI based on settings

- [x] **Professional cross-platform installer configuration**
  - Windows: NSIS with language selector, custom icons, installer/sidebar images, WebView2 silent bootstrap
  - macOS: DMG with custom app/Applications folder positioning, window dimensions, minimum 10.15
  - Linux: DEB (Debian/Ubuntu), RPM (Fedora/RHEL), AppImage (universal binary)
  - Proper categories, license, copyright, descriptions for all platforms

- [x] **System tray icon**
  - TrayIconBuilder with Show Ghost / Quit Ghost menu items
  - Left-click toggles window visibility, tooltip display
  - Tray icon feature enabled in Cargo.toml (`tray-icon`)
  - trayIcon config in tauri.conf.json with 32x32 icon

- [x] **Filesystem browser & directory management**
  - `list_directory`, `get_home_directory`, `get_root_directories` Tauri commands
  - OneDrive cloud file detection (`FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS` on Windows)
  - Metadata-only indexing for cloud placeholder files
  - `add_watch_directory` / `remove_watch_directory` with live indexing
  - Settings redesigned: 3 tabs (General, AI Models, Directories)
  - Visual filesystem browser in Directories tab
  - FsEntry type with `is_cloud_placeholder`, `is_local` flags

- [x] **Settings enhancements**
  - `setup_complete` boolean for first-launch tracking
  - `launch_on_startup` boolean for auto-start preference
  - All fields with `#[serde(default)]` for backward compatibility
  - Settings.json survives upgrades — new fields get sensible defaults

- [x] **Unified Omnibox (intelligent single input)**
  - Replace tab system (Search/Chat) with a single smart input
  - Auto-detection: file patterns → search, conversational starters → chat
  - Sticky chat mode: stays in chat when conversation is active
  - Manual mode override via toggle button
  - GhostInput component with auto-resize textarea, mode indicator, keyboard hints
  - Progressive Escape: clear query → clear chat → hide window
  - Chat history persists across mode switches

- [x] **Open source project configuration**
  - Migrated from `AngelAlexQC/ghost` to `ghostapp-ai/ghost` organization
  - Repository made PUBLIC for open source (MIT license)
  - `ghost-pro` as private git submodule (`ghostapp-ai/ghost-pro`)
  - Dynamic GitHub badges (release, CI, license, issues)
  - Community files: SUPPORT.md, FUNDING.yml, CODEOWNERS, issue/PR templates
  - package.json: full metadata (author, license, repo, keywords, homepage)
  - Cargo.toml: proper authors, license, repository, homepage, rust-version
  - Vulnerability alerts + automated security fixes enabled on both repos
  - Allow branch update, delete-branch-on-merge, squash merge defaults

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

## Phase 1.5 — "Protocol Bridge" (Weeks 8-12)

**Goal**: Make Ghost a Universal Protocol Hub — both an MCP host AND client, with AG-UI runtime for interactive agent experiences. This is the competitive differentiator that no other desktop app offers.

> Added after market research + protocol landscape analysis (Feb 2026).
> MCP ecosystem: 10,000+ servers. AG-UI: 12K+ GitHub stars, Rust SDK available.
> A2UI: Google-backed generative UI spec. A2A: 50+ launch partners, Linux Foundation.
> Being the first local desktop app to speak ALL agent protocols is a once-in-a-generation opportunity.

### Technical Deliverables

#### MCP Server (Ghost as a tool for external AI clients)
- [x] **MCP Server via `rmcp` crate** (official Rust SDK, v0.16)
  - HTTP Streamable transport via axum on localhost (configurable port 6774)
  - MCP protocol v2024-11-05 compliance (latest rmcp)
  - Tool: `ghost_search` — hybrid search across indexed files
  - Tool: `ghost_index_status` — report indexing stats + watched dirs
  - Tool: `ghost_recent_files` — list recently indexed files
  - `#[tool]` / `#[tool_handler]` / `#[tool_router]` macros for clean definitions
  - ServerHandler impl with GhostMcpServer struct
  - Auto-starts on app launch when enabled
  - [ ] Resource: expose indexed documents metadata
  - [ ] Integration tested with Claude Desktop and Cursor

#### MCP Client (Ghost connects to external MCP servers)
- [x] **MCP Client Host via `rmcp`**
  - Configuration: `mcp_servers` array in settings.json
  - Transport support: stdio (TokioChildProcess) + HTTP Streamable (StreamableHttpClientTransport)
  - Tool discovery: fetch and cache tool schemas from connected servers
  - Tool invocation: `call_tool()` with JSON arguments
  - Dynamic tool loading: add/remove servers without restart (8 Tauri commands)
  - Auto-connect to enabled servers on app startup
  - Settings UI: MCP tab with server management (add, connect, disconnect, remove)
  - [ ] Free tier: 3 MCP server connections; Pro: unlimited

#### AG-UI Runtime (Agent ↔ User Interaction Protocol)
- [x] **AG-UI event system in Rust backend**
  - Implement ~16 AG-UI event types (TEXT_MESSAGE_CONTENT, TOOL_CALL_START, STATE_DELTA, etc.)
  - Event bus (broadcast channel) for fan-out to multiple consumers
  - SSE endpoint on `/agui` for external clients alongside MCP `/mcp` endpoint
  - AgentRunner orchestrates chat lifecycle as AG-UI event stream
  - Tool call event emission (TOOL_CALL_START/ARGS/END) infrastructure ready
- [x] **AG-UI React client**
  - `useAgui` hook: listens to Tauri `agui://event` events, manages run state
  - Streaming text display with real-time TEXT_MESSAGE_CONTENT deltas
  - `chat_send_streaming` Tauri command for event-driven chat
  - Fallback to non-streaming `chat_send` when model not natively available
- [ ] **AG-UI advanced features (next iteration)**
  - Bidirectional streaming: user actions → agent (human-in-the-loop)
  - Approval gates for dangerous tool actions
  - True token-by-token streaming from llama.cpp (replace word-chunking)
  - State synchronization between Rust agent and React UI

#### A2UI Renderer (Generative UI)
- [ ] **A2UI JSON → React component renderer**
  - Parse A2UI declarative JSON specs from agent responses
  - Render native-feeling components: forms, tables, charts, date pickers
  - Sandboxed execution: A2UI specs cannot access host APIs directly
  - Theme integration with Ghost's dark UI
- [ ] **MCP Apps support**
  - Sandboxed iframe renderer for MCP App HTML content
  - Communication bridge: MCP App ↔ Ghost host via postMessage
  - Security: CSP headers, no external network access from sandbox

#### Skills System
- [ ] **Skills.md format support**
  - Define agent capabilities in markdown files (OpenClaw-compatible)
  - Skill discovery: scan `~/.ghost/skills/` directory
  - Skill loading: register tools from skill definitions
  - Community skills: shareable via Git repos

### Exit Criteria
- [x] Claude Desktop can search local files through Ghost MCP server
- [ ] Ghost chat can invoke tools from at least 2 external MCP servers
- [x] AG-UI event stream renders streaming text + tool progress in React
- [ ] A2UI renders at least 3 component types (form, table, text block)
- [ ] <100ms overhead added by MCP protocol layer
- [ ] Setup guide published for MCP server + client configuration

---

## Phase 1.7 — "Multiplatform" (Weeks 12-16)

**Goal**: Adapt Ghost for Android and iOS via Tauri v2 mobile targets, making it the first local-first AI assistant on every platform.

> Market insight: AI assistant market $16.29B (2024) → $73.8B (2033), 18.8% CAGR.
> Mobile is 60%+ of compute time. Ghost is the only local-first, open-source, multiplatform AI assistant.
> TV platforms rejected as not viable (closed ecosystems, no file system access).

### Technical Deliverables

#### Backend (Rust) Adaptation
- [x] **Conditional compilation with `#[cfg(desktop)]` / `#[cfg(mobile)]`**
  - Use Tauri's built-in platform macros (set by target triple, not Cargo features)
  - Target-specific deps: notify, notify-debouncer-mini, tauri-plugin-global-shortcut, tray-icon → desktop only
  - `[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]` in Cargo.toml
- [x] **Platform-gated features**
  - File watcher (`indexer::watcher`): desktop only; mobile no background file monitoring
  - System tray + global shortcuts: desktop only
  - Window toggle (`toggle_window`): desktop only
  - MCP stdio transport: desktop only; mobile falls back to HTTP transport with clear error
  - `start_watcher()`: desktop version starts notify; mobile stub returns Ok(())
- [x] **Mobile hardware detection (embeddings/hardware.rs)**
  - iOS: Metal GPU backend, conservative 6GB RAM estimate (no std::process::Command)
  - Android: Vulkan GPU backend, /proc/meminfo RAM detection (Linux-based)
- [x] **Platform info command (`get_platform_info`)**
  - Returns: platform, is_desktop, is_mobile, has_file_watcher, has_system_tray, has_global_shortcuts, has_stdio_mcp
  - Frontend uses this to adapt UI dynamically

#### Capabilities Split
- [x] **Tauri capabilities separated by platform**
  - `default.json`: core permissions only (all platforms)
  - `desktop.json`: global-shortcut permissions (linux, macOS, windows)
  - `mobile.json`: core permissions (iOS, android)

#### Frontend Responsive Adaptation
- [x] **`usePlatform()` hook** — detects platform via Tauri command, fallback to navigator.userAgent
  - Computed properties: isMobile, isDesktop, isIos, isAndroid, isMacos, modKey, activationShortcut
- [x] **App.tsx multiplatform**
  - Auto-hide on blur: desktop only (mobile apps don't hide)
  - Drag region: desktop only (no title bar drag on mobile)
  - `h-dvh` instead of `h-screen` for mobile browser chrome
  - Window hide after file open: desktop only (spotlight-style)
  - Escape key: doesn't hide window on mobile
  - Responsive border/shadow: `md:rounded-2xl md:border md:shadow-2xl`
- [x] **GhostInput.tsx mobile**
  - `isMobile` prop: larger touch targets (48px buttons), larger icons
  - Font size: `text-base` on mobile vs `text-[15px]` desktop
  - Keyboard hints (kbd elements): hidden on mobile
  - Clear button: larger tap area on mobile
- [x] **StatusBar.tsx compact mode**
  - `compact` prop: fewer details on mobile (hide DB stats, vector status)
  - Larger settings button touch target on mobile
  - Safe area bottom padding (`pb-safe`)
- [x] **Settings.tsx mobile**
  - Full-screen modal on mobile (no rounded corners, full viewport)
  - Horizontally scrollable tabs on small screens
  - Directories tab hidden on mobile (no file watcher)
  - Larger close button touch target
  - Safe area top padding
- [x] **Onboarding.tsx mobile**
  - Conditional drag region (desktop only)
  - Safe area padding, adapted content padding
  - `h-dvh` viewport height
- [x] **Responsive CSS (globals.css)**
  - Mobile safe area insets: `pt-safe`, `pb-safe`, `pl-safe`, `pr-safe`
  - `h-dvh` utility (100dvh for mobile browser chrome)
  - `scrollbar-none` utility for hidden scrollbars
  - Touch-friendly overrides `@media (pointer: coarse)`: min 44px touch targets
  - Momentum scrolling for touch devices

#### Mobile Project Scaffold
- [x] **Android setup**: `tauri android init`, Gradle config, minimum SDK 24+
  - Full Gradle project scaffolded with `npx tauri android init`
  - 4 Android Rust targets installed: aarch64, armv7, i686, x86_64
  - OpenSSL eliminated — entire dep tree uses rustls (cross-compilation safe)
  - `llama-cpp-2` gated to desktop-only (avoids C++ cross-compilation on mobile)
  - `std::ffi::c_char` / `c_int` for sqlite-vec FFI (Android C char is unsigned)
  - Window management APIs (`hide`, `show`, `start_dragging`) gated with `#[cfg(desktop)]`
  - `ChatEngine.native` field and `chat::native` module gated desktop-only
  - APK: 39MB (unsigned, aarch64), AAB: 16MB — zero Rust warnings
- [ ] **iOS setup**: `tauri ios init`, Xcode project, minimum iOS 15+
  - Requires macOS — `tauri ios` subcommand only available on macOS
  - Backend Rust code already adapted (conditional compilation ready)
  - Frontend already responsive with iOS safe areas
- [ ] **Mobile CI/CD**: GitHub Actions for Android APK + iOS IPA builds
- [ ] **App Store assets**: screenshots, descriptions, privacy policies
- [ ] **Mobile-specific features**: share sheet integration, notification support

#### Dead Code Removal
- [x] **Removed ChatPanel.tsx** — superseded by unified Omnibox (GhostInput)
- [x] **Removed SearchBar.tsx** — superseded by unified Omnibox (GhostInput)

#### Component Adaptation (All Components)
- [x] **ChatMessages.tsx** — `isMobile` prop, platform-aware tips (no keyboard shortcuts on mobile), `useMemo` for tips
- [x] **ResultsList.tsx** — `isMobile` prop, conditional kbd elements, mobile-friendly empty state text
- [x] **ResultItem.tsx** — `isMobile` prop, single-tap opens on mobile (vs double-click desktop), 44px+ touch targets, `active:` state
- [x] **DownloadProgress.tsx** — reviewed, already responsive (no changes needed)

### Exit Criteria
- [x] `bun run build` compiles frontend with zero errors (245.96 KB JS)
- [x] `cargo check` + `cargo test` (34 tests) + `cargo clippy -- -D warnings` all pass clean
- [x] All responsive adaptations compile and render correctly on desktop
- [x] `tauri android build --target aarch64` produces a working APK (39MB) + AAB (16MB)
- [ ] `tauri ios build` produces a working IPA (requires macOS)
- [ ] Mobile app tested on device/emulator and shows Onboarding on first launch

---

## Phase 2 — "The Agent OS" (Weeks 13-22)

**Goal**: Transform Ghost from a search tool into a true local Agent OS. Users interact with their computer through natural language. Growing paid user base.

### Technical Deliverables

#### A2A Protocol (Agent-to-Agent Communication)
- [ ] **Ghost Agent Card**
  - Publish `/.well-known/agent.json` on localhost
  - Advertise Ghost's capabilities (search, file ops, OS control)
  - OAuth 2.0 / API key authentication for local agent-to-agent calls
- [ ] **A2A Client**
  - Discover and connect to other A2A-compatible agents (OpenClaw, NanoClaw instances)
  - Task delegation: send tasks to specialized remote agents
  - SSE streaming for long-running delegated tasks
  - Task lifecycle: submitted → working → input-required → completed
- [ ] **A2A Server**
  - Accept tasks from external agents via JSON-RPC 2.0
  - Expose Ghost tools as A2A skills
  - Multi-agent orchestration: break complex requests into sub-tasks

#### Tool Calling Engine
- [ ] **LLM-driven tool selection**
  - Qwen2.5-Instruct structured output for tool call arguments
  - Tool schema injection into chat context (MCP tool list → system prompt)
  - Multi-step planning: chain multiple tool calls for complex tasks
  - Fallback: Ollama API `/api/chat` with tool calling for larger models

#### OS Integration Layer
- [ ] **Browser history indexing**
  - Read Chrome/Edge/Firefox SQLite history DBs (Windows/macOS/Linux)
  - Index page titles, URLs, timestamps
  - Respect private/incognito mode flags

- [ ] **App activity via UI Automation**
  - Windows: `uiautomation` Rust crate for reading control trees
  - macOS: `accessibility` crate for AXUIElement tree walking
  - Linux: AT-SPI2 D-Bus interface for accessibility tree
  - Capture: active window title, focused control text, app name
  - Activity timeline: "What was I doing at 3pm yesterday?"
  - <1% CPU overhead, sample every 5 seconds

- [ ] **Clipboard history**
  - Monitor clipboard changes cross-platform
  - Store text clips with timestamp and source app
  - Semantic search across clipboard history
  - Privacy: configurable exclusion rules (password managers, banking apps)

- [ ] **Agent actions (OS control)**
  - Open/focus applications
  - Create/move/rename/delete files
  - Copy text to clipboard
  - Send keyboard shortcuts to active window
  - Search and open URLs in default browser

- [ ] **Action Preview (safety layer)**
  - Before executing, Ghost shows a plan via A2UI components
  - Step-by-step action visualization with confirm/cancel
  - Undo support for reversible actions (file ops)
  - Audit log of all executed actions (local SQLite table)

#### Micro-Agents (PicoClaw-inspired)
- [ ] **Background task agents**
  - Spawn lightweight background tasks that monitor conditions
  - Example: "Watch this folder and organize new PDFs by date"
  - Example: "Alert me when file X changes"
  - Cron-like scheduling for recurring tasks
  - Status dashboard showing active micro-agents

#### Activity Timeline UI
- [ ] Chronological view of all indexed activity
- [ ] Filter by: date range, app, content type
- [ ] Natural language queries: "show me what I worked on last Tuesday"
- [ ] A2UI-rendered rich cards for different activity types

### Premium Features (Ghost Pro — `ghost-pro` submodule)
- [ ] Vault encryption with ChaCha20-Poly1305 (`age` crate)
- [ ] Encrypted sync between devices (optional, user-controlled)
- [ ] Unlimited MCP server connections
- [ ] WebMCP browser agent (Phase 2.5)
- [ ] A2A multi-agent orchestration
- [ ] Advanced models (Qwen2.5-7B+, or custom model support)
- [ ] OS automation: unlimited actions/day (free: 5/day)
- [ ] Micro-agents (free: 1 active, Pro: unlimited)

### Licensing System
- [ ] License key validation (offline-capable, cryptographic)
- [ ] Feature gating via `#[cfg(feature = "pro")]` in Rust
- [ ] Free tier: core search + chat + 3 MCP servers + 5 actions/day
- [ ] Pro tier: everything unlimited + premium features

### Exit Criteria
- [ ] Agent can reliably execute 5+ action types on the OS
- [ ] Action Preview shows correct plan >95% of the time
- [ ] A2A discovery works with at least 1 external agent
- [ ] Paying users onboarded
- [ ] Encryption passes basic security review
- [ ] Mac + Linux automation working alongside Windows

---

## Phase 2.5 — "Web Agent" (Weeks 23-26)

**Goal**: Extend Ghost's reach to the web via WebMCP, enabling structured interactions with websites.

### Technical Deliverables

- [ ] **WebMCP Consumer**
  - Parse WebMCP tool contracts from website `navigator.modelContext` declarations
  - Bridge: Ghost sends structured tool calls to browser extension
  - Ghost browser extension (Chrome/Firefox) that reads WebMCP tool contracts
  - Invoke website tools on behalf of the user (booking, forms, data entry)

- [ ] **Browser Extension**
  - Lightweight extension exposing current page's WebMCP tools to Ghost
  - Communication via native messaging (Chrome Native Messaging API)
  - Page context: current URL, page title, selected text → Ghost search context
  - Ghost can request the extension to perform WebMCP tool calls

### Exit Criteria
- [ ] Ghost can read WebMCP tools from at least 3 websites
- [ ] Browser extension communicates bidirectionally with Ghost desktop
- [ ] User can ask "Book me a flight to Madrid" and Ghost interacts with a travel site

---

## Phase 3 — "The Platform" (Months 7-12)

**Goal**: Partnerships, marketplace, and platform expansion. Explore B2B/teams model.

### Technical Deliverables

- [ ] **Skills Marketplace**
  - Community-created skills with install/uninstall from Ghost UI
  - Skills categories: productivity, development, data, creative, automation
  - Revenue share: 70% creator / 30% Ghost (Pro feature to sell skills)
  - NPM package: `@ghost/skills-sdk` for JavaScript skill development
  - Rust crate: `ghost-skills` for native Rust skill development
  - Example skills: GitHub integration, Notion sync, Slack search, calendar

- [ ] **Third-party integrations**
  - Obsidian vault indexing (direct SQLite reading)
  - VS Code extension (Ghost as search backend, sidebar panel)
  - Slack message search via MCP server
  - Browser extension v2: page content indexing, reader mode

- [ ] **Multi-agent orchestration**
  - A2A agent team composition: assign specialized agents to sub-tasks
  - Agent routing: analyze request → pick best agent(s) → coordinate results
  - Visual agent workflow builder (A2UI-rendered)

- [ ] **B2B/Teams features (Ghost Pro Teams)**
  - Shared team vaults with role-based access control
  - Admin dashboard: manage team licenses, usage analytics
  - Compliance: audit trail, data retention policies, export controls
  - SSO integration (SAML 2.0 / OIDC)
  - Team-shared MCP server configurations
  - Centralized skill management for team deployments

- [ ] **Platform stability**
  - Mac port fully tested (WebKit, Accessibility API)
  - Linux Wayland support verified
  - Performance profiling: cold start, search latency, memory footprint
  - Crash recovery: auto-save state, graceful degradation

### Exit Criteria
- [ ] 10+ community skills published in marketplace
- [ ] Mac version stable with full feature parity
- [ ] At least one B2B pilot customer
- [ ] A2A working with 3+ external agent platforms
- [ ] $50K+ ARR from Pro + Teams subscriptions

---

## Key Technical References

| Resource | URL | Notes |
| -------- | --- | ----- |
| Tauri v2 | tauri.app | v2.10.2 stable. Plugins: global-shortcut, fs, shell |
| sqlite-vec | github.com/asg017/sqlite-vec | SIMD-accelerated KNN. Works with FTS5 |
| rmcp | crates.io/crates/rmcp | Official Rust MCP SDK v0.15+. Client + Server. `#[tool]` macro |
| AG-UI | github.com/CopilotKit/ag-ui | Agent↔User protocol. 12K+ stars. Rust SDK available |
| A2UI | github.com/AgenturAI/a2ui | Google-backed declarative generative UI. JSON → native components |
| A2A | google.github.io/A2A | Agent-to-Agent protocol. Agent Cards + JSON-RPC 2.0 + SSE. Linux Foundation |
| WebMCP | developer.chrome.com/blog/webmcp-epp | W3C proposal. `navigator.modelContext` browser API |
| MCP Apps | blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps | Tools return interactive UI (iframes). Anthropic spec |
| MCP Spec | modelcontextprotocol.io | v2025-11-25 spec. Linux Foundation / AAIF |
| OpenClaw | github.com/nicepkg/OpenClaw | Model-agnostic agent infra. Skills.md format. 100K+ stars |
| Ollama | ollama.com | Local LLM runtime. Supports Qwen2.5 tool calling |
| nomic-embed-text | ollama.com/library/nomic-embed-text | 768 dims, surpasses ada-002, ~274MB |
| uiautomation | crates.io/crates/uiautomation | Windows UI Automation wrapper for Rust |
| notify | crates.io/crates/notify | Cross-platform filesystem watcher |
| candle | github.com/huggingface/candle | Rust ML framework, BERT/LLM inference |
| all-MiniLM-L6-v2 | huggingface.co/sentence-transformers/all-MiniLM-L6-v2 | 384 dims, 23MB, excellent quality |
| hf-hub | crates.io/crates/hf-hub | HuggingFace model download/cache |

## Protocol Stack Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  USER (natural language via Ghost Omnibox)                       │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│  AG-UI Runtime (Agent ↔ User Interaction Layer)                 │
│  ├── ~16 event types: text, tool calls, state, lifecycle        │
│  ├── Bidirectional streaming (Rust ↔ React)                     │
│  └── Human-in-the-loop approval gates                           │
├─────────────────────────────────────────────────────────────────┤
│  A2UI / MCP Apps (Generative UI Layer)                          │
│  ├── A2UI: JSON → native React components (safe, sandboxed)     │
│  └── MCP Apps: iframe-based interactive tool UIs                │
├─────────────────────────────────────────────────────────────────┤
│  MCP Host (Tool Layer — rmcp crate)                             │
│  ├── MCP Server: Ghost tools exposed to Claude/Cursor/etc.      │
│  ├── MCP Client: connects to 10,000+ external MCP servers       │
│  └── Tool calling: LLM selects + invokes tools from schemas     │
├─────────────────────────────────────────────────────────────────┤
│  A2A (Agent Coordination Layer)                                 │
│  ├── Agent Card at /.well-known/agent.json                      │
│  ├── Task delegation to specialized agents                      │
│  └── SSE streaming for long-running multi-agent tasks           │
├─────────────────────────────────────────────────────────────────┤
│  WebMCP (Web Agent Layer — Phase 2.5)                           │
│  ├── Read tool contracts from websites                          │
│  └── Browser extension bridge for structured web interactions   │
├─────────────────────────────────────────────────────────────────┤
│  OS Integration Layer                                           │
│  ├── Filesystem: index, watch, CRUD (already implemented)       │
│  ├── UI Automation: Windows + macOS + Linux accessibility       │
│  ├── Clipboard history, browser history                         │
│  └── App activity monitoring, micro-agents                      │
└─────────────────────────────────────────────────────────────────┘
```

## Business Model

### Open Core (ghostapp-ai/ghost MIT + ghostapp-ai/ghost-pro proprietary)

| Feature | Free | Pro ($8/mo) | Teams ($15/user/mo) |
|---------|------|-------------|---------------------|
| Local search (FTS5+vector) | ✅ | ✅ | ✅ |
| Chat AI (Candle native) | ✅ | ✅ | ✅ |
| MCP Server (Ghost as tool) | ✅ | ✅ | ✅ |
| MCP Client (3 servers) | ✅ | — | — |
| MCP Client (unlimited) | — | ✅ | ✅ |
| A2UI Generative UI | Basic | Full | Full |
| A2A Multi-agent | — | ✅ | ✅ |
| WebMCP Browser Agent | — | ✅ | ✅ |
| Skills Marketplace | Free only | Free + Premium | Free + Premium |
| OS Automation (actions/day) | 5 | Unlimited | Unlimited |
| Micro-agents | 1 active | Unlimited | Unlimited |
| Vault encryption | — | ✅ | ✅ |
| Cross-device sync | — | ✅ | ✅ |
| Team vaults + SSO | — | — | ✅ |
| Audit trail + compliance | — | — | ✅ |
| Advanced models (7B+) | — | ✅ | ✅ |
