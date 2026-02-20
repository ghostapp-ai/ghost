<p align="center">
  <img src="public/ghost-logo.svg" width="120" alt="Ghost Logo" />
</p>

<h1 align="center">GHOST</h1>

<p align="center">
  <strong>The Private Agent OS for Desktop & Mobile</strong>
</p>

<p align="center">
  <a href="#features">Features</a> &bull;
  <a href="#architecture">Architecture</a> &bull;
  <a href="#getting-started">Getting Started</a> &bull;
  <a href="#roadmap">Roadmap</a> &bull;
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <a href="https://github.com/ghostapp-ai/ghost/releases/latest"><img src="https://img.shields.io/github/v/release/ghostapp-ai/ghost?style=flat-square&color=blue" alt="Release" /></a>
  <a href="https://github.com/ghostapp-ai/ghost/blob/main/LICENSE"><img src="https://img.shields.io/github/license/ghostapp-ai/ghost?style=flat-square&color=green" alt="License" /></a>
  <a href="https://github.com/ghostapp-ai/ghost/actions/workflows/ghost.yml"><img src="https://img.shields.io/github/actions/workflow/status/ghostapp-ai/ghost/ghost.yml?branch=main&style=flat-square&label=CI" alt="CI" /></a>
  <a href="https://github.com/ghostapp-ai/ghost/issues"><img src="https://img.shields.io/github/issues/ghostapp-ai/ghost?style=flat-square" alt="Issues" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20Android-lightgrey?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/privacy-100%25%20local-brightgreen?style=flat-square" alt="Privacy" />
</p>

---

Ghost is a private, local-first **Agent OS** for desktop and mobile. It indexes your files, understands your context, connects to thousands of tools via open protocols (MCP, A2A, AG-UI, A2UI, WebMCP), and takes actions on your behalf — all without sending a single byte to the cloud.

Think **Raycast + Semantic Search + Local AI Agent + Universal Protocol Hub** — but private by design.

## Why Ghost?

- **100% Local**: Your data never leaves your machine. Zero telemetry, zero cloud dependencies.
- **Instant Search**: Hybrid keyword (FTS5) + semantic vector search across all your documents.
- **Native AI**: In-process embedding inference via Candle — no Ollama or internet required after first model download.
- **Lightweight**: <10MB installer, <40MB RAM idle, <500ms launch. 70% less RAM than Electron.
- **Protocol Hub**: MCP Server + Client, A2A multi-agent coordination, AG-UI streaming, A2UI generative UI — the most connected local agent.
- **Extensible**: Skills.md plugin system + compatible with 10,000+ MCP servers + future Skills Marketplace.

## Features

### Phase 0 — Foundation (**Complete**)

- Tauri v2 desktop shell with React/TypeScript frontend
- Rust core engine with `thiserror` error handling + `tracing` logging
- SQLite + sqlite-vec (via FFI auto-extension) for unified storage (documents, vectors, text)
- FTS5 keyword search + sqlite-vec KNN vector search with RRF hybrid ranking
- **Native AI inference**: Candle (HuggingFace Rust ML) with all-MiniLM-L6-v2 (384D, ~23MB) — works on any CPU, zero external dependencies
- **Fallback chain**: Native → Ollama → FTS5-only keyword search
- Hardware detection: CPU cores, AVX2/NEON SIMD, GPU backend (CUDA/Metal/Vulkan)
- File watcher (`notify` + `notify-debouncer-mini`) for real-time document indexing
- Text extraction: PDF (lopdf), DOCX (zip), XLSX (calamine), TXT, Markdown, code files
- 27 unit tests passing, zero compiler warnings

### Phase 1 — The Search Bar (**Complete**)

- **Spotlight-like floating window**: `Ctrl/Cmd+Space` global shortcut, decorationless, always-on-top, transparent
- Auto-hide on focus loss, Escape to dismiss, draggable title region
- Dark ghost-themed UI with Tailwind CSS v4
- Debounced search input (150ms) with loading skeletons
- Virtualized results list (`@tanstack/react-virtual`) with file type icons
- **Open files**: Enter key or double-click opens files with system default app
- Keyboard navigation: arrow keys, Enter to open, Esc to dismiss
- Settings panel: watched directory management, persistent settings (JSON)
- **Zero-config auto-indexing**: auto-discovers user directories (Documents, Desktop, Downloads, Pictures) on first launch — like Spotlight/Alfred
- Cross-platform directory detection via `dirs` crate (XDG, Windows Known Folders, macOS standard paths)
- Auto-start file watcher on launch with saved directories
- **Reliable window dragging**: programmatic `startDragging()` fallback for Linux/Wayland compatibility
- 50+ source code extensions indexed (rs, py, js, ts, go, java, cpp, etc.)
- Status bar: document count, AI engine status, vector search status, chat model status
- Cross-platform CI/CD: Windows, macOS (ARM64 + Intel), Linux installers
- 27 tests, zero warnings, ~203KB JS bundle

### Native Chat Engine (**Complete**)

- **Hardware-aware auto-selection**: Detects CPU cores, RAM, GPU at startup → recommends largest fitting model
- **Model registry**: Qwen2.5-Instruct GGUF family (0.5B/1.5B/3B/7B) with Q4_K_M quantization
- **Zero-config flow**: detect hardware → pick model → auto-download from HuggingFace Hub → background load
- **Device selection**: CPU (default), CUDA (`--features cuda`), Metal (`--features metal`)
- **Chat UI**: Tab-based interface (Search ↔ Chat), message bubbles, model status, loading states
- **Debug panel**: Collapsible log viewer with pause/resume and color-coded levels (Ctrl+D)
- **Fallback chain**: Native Candle GGUF → Ollama HTTP API → offline
- **Settings**: model, device, max_tokens, temperature — all configurable, all with sensible defaults
- **RAM detection**: Linux (/proc/meminfo), macOS (sysctl), Windows (PowerShell Get-CimInstance)

### First-Launch Experience & Installer (**Complete**)

- **Onboarding wizard**: Multi-step setup shown only on first launch
  - Welcome screen with Ghost branding
  - Hardware auto-detection (CPU, RAM, GPU, SIMD)
  - Recommended model display with specs (size, RAM requirements, parameters)
  - One-click model download with real-time progress bar
  - Setup complete summary with keyboard shortcut reminder
  - Skip option for power users who want to configure later
- **System tray icon**: Background presence with Show/Quit menu, left-click focus
- **Professional installer configuration**:
  - Windows: NSIS with language selector, custom icons, WebView2 silent bootstrap
  - macOS: DMG with custom layout, minimum macOS 10.15
  - Linux: DEB (Debian/Ubuntu), RPM (Fedora/RHEL), AppImage (universal)
- **Filesystem browser**: Navigate directories visually from Settings
- **OneDrive-aware indexing**: Detects cloud placeholders, indexes metadata only
- **Zero-config**: Auto-discovers Documents, Desktop, Downloads, Pictures on first launch
- **Settings persistence**: `setup_complete`, `launch_on_startup`, all chat preferences with serde defaults

### Phase 1.7 — Multiplatform (**Complete**)

- **Android APK**: Full Tauri v2 mobile build (39MB APK, 16MB AAB for aarch64)
- **Conditional compilation**: `#[cfg(desktop)]` / `#[cfg(mobile)]` for platform-specific code
- **TLS migration**: 100% rustls — zero OpenSSL, Android NDK cross-compilation safe
- **Desktop-only gating**: llama-cpp-2, file watcher, system tray, global shortcuts, MCP stdio
- **Responsive frontend**: all components adapted with `isMobile` prop, 44px+ touch targets, safe areas
- **Platform detection**: `usePlatform()` hook for runtime UI adaptation
- **iOS ready**: backend + frontend fully adapted, scaffold requires macOS

### Phase 1.5 — The Protocol Bridge *(In Progress)*

- **MCP Server**: ✅ Expose Ghost tools (search, index, stats) to Claude, Cursor, VS Code via `rmcp` + HTTP streamable transport
- **MCP Client**: ✅ Connect to external MCP servers (filesystem, GitHub, databases, 10,000+) via stdio + HTTP
- **AG-UI Runtime**: ✅ Bidirectional agent↔user streaming (~16 event types) — event bus, SSE endpoint, `useAgui` React hook, streaming chat
- **A2UI Renderer**: ✅ Google A2UI v0.9 generative UI — 17+ component types (Text, Button, TextField, Card, Row, Column, etc.) rendered natively in React/Tailwind with data binding
- **Skills.md**: ✅ SKILL.md parser with YAML frontmatter, trigger matching, SkillRegistry, tool schemas — OpenClaw-compatible
- **Agent Engine**: ✅ ReAct (Reason + Act) agent with native llama.cpp inference, grammar-constrained tool calling, 3-tier safety, conversation memory, hardware-adaptive Qwen2.5-Instruct (0.5B–7B)
- **MCP Apps**: Interactive tool UIs in sandboxed iframes within Ghost conversations

### Phase 2 — The Agent OS

- **A2A Protocol**: Agent-to-Agent coordination — Ghost delegates to specialized agents
- **Tool Calling Engine**: Qwen2.5-7B selects + invokes MCP tools from schemas
- **OS Integration**: UI Automation (Windows), accessibility APIs, clipboard history, browser history
- **Micro-agents**: Background agents (file organizer, meeting summarizer, email drafter)
- Premium features: sync, encryption, advanced models, unlimited automations

### Phase 2.5 — The Web Agent

- **WebMCP**: Read tool contracts from websites via W3C `navigator.modelContext` API
- **Browser Extension**: Bridge between Ghost desktop agent and web-based tool contracts
- Structured web interactions without scraping

### Phase 3 — The Platform

- Skills Marketplace: third-party skill distribution and monetization
- Integrations: Obsidian, VS Code, Slack, Notion, browsers
- Multi-agent orchestration: A2A task delegation between local agents
- B2B/Teams: shared vaults, SSO, audit trails, compliance

## Architecture

Ghost uses a 6-layer **Agent OS** architecture where each layer is independently replaceable:

```text
┌──────────────────────────────────────────────────────┐
│              Frontend (React/TypeScript)               │
│  Omnibox │ Results │ Chat │ A2UI Renderer │ Settings  │
├──────────────────────────────────────────────────────┤
│         AG-UI Runtime (Agent ↔ User Streaming)        │
│  ~16 event types │ Human-in-the-loop │ State sync      │
├──────────────────────────────────────────────────────┤
│              Tauri v2 IPC Bridge                       │
├──────────────────────────────────────────────────────┤
│              Agent Engine (ReAct Loop)                 │
│  Executor │ Tools │ Safety │ Memory │ Skills           │
├──────────────────────────────────────────────────────┤
│              Protocol Hub (Rust — rmcp + custom)       │
│  MCP Server │ MCP Client │ A2A │ WebMCP │ Skills       │
├──────────────────────────────────────────────────────┤
│              Core Engine (Rust)                        │
│  File Watcher │ Text Extractor │ Embedding Engine      │
│  Vector DB │ OS Automation │ Micro-agents              │
├──────────────────────────────────────────────────────┤
│              AI Layer (Local — Zero Dependencies)      │
│  Native: Candle + all-MiniLM-L6-v2 (384D embeddings)  │
│  Fallback: Ollama + nomic-embed-text (768D)            │
│  Chat: Qwen2.5-Instruct GGUF (0.5B–7B, native)       │
│  Agent: Qwen2.5-Instruct GGUF (0.5B–7B, tool calling) │
└──────────────────────────────────────────────────────┘
```

### Hybrid Trigger System

Ghost uses a two-speed architecture to feel instant without burning CPU:

| Layer            | When      | Speed       | Resource Usage   |
| ---------------- | --------- | ----------- | ---------------- |
| **Fast Layer**   | Always    | <10ms       | 0% GPU, <1% CPU  |
| **Smart Layer**  | On demand | 200-2000ms  | Activates native AI |

The Fast Layer uses OS accessibility APIs and FTS5 keyword search. The Smart Layer activates only when the user asks a natural language question, requests an action, or a new file needs indexing.

### Tech Stack

| Component          | Technology                       | Why                                                |
| ------------------ | -------------------------------- | -------------------------------------------------- |
| Shell/UI           | Tauri v2 + React/TypeScript      | <10MB installer, native performance                |
| Database           | SQLite + sqlite-vec + FTS5       | Single .db file, vectors + text + metadata         |
| Native Embeddings  | Candle + all-MiniLM-L6-v2        | 384D, ~23MB, in-process, no external deps          |
| Fallback Embeddings| Ollama + nomic-embed-text        | 768D, optional, higher quality for large models    |
| LLM / Chat         | Candle GGUF + Qwen2.5-Instruct   | Native inference, tool calling, 0.5B–7B tiers      |
| Agent Engine       | llama.cpp GGUF + ReAct loop      | Hardware-adaptive (0.5B–7B), grammar-constrained tool calling |
| Safety Layer       | 3-tier risk classification       | Safe/Moderate/Dangerous, human-in-the-loop         |
| Skills System      | SKILL.md + YAML frontmatter      | OpenClaw-compatible, trigger matching, extensible   |
| MCP Protocol       | rmcp (official Rust SDK)         | Server + Client, `#[tool]` macro, 10,000+ servers  |
| Agent Interaction  | AG-UI + A2UI                     | Real-time streaming + generative UI from JSON       |
| Agent Coordination | A2A (Google)                     | Multi-agent task delegation, Agent Cards            |
| Web Agent          | WebMCP (W3C)                     | Browser tool contracts, no scraping                 |
| File Watcher       | notify (Rust crate)              | Cross-platform, async, <1% CPU                     |
| Text Extraction    | lopdf + zip + calamine           | Pure Rust, no external dependencies                |
| Encryption         | ChaCha20-Poly1305 (age crate)    | Modern, audited (Pro)                              |

## Getting Started

### Download & Install

Download the latest release for your platform from [**GitHub Releases**](https://github.com/ghostapp-ai/ghost/releases/latest):

| Platform | File | Notes |
|----------|------|-------|
| **Windows** (64-bit) | `ghost_x.x.x_x64-setup.exe` | NSIS installer, no admin required |
| **macOS** (Apple Silicon) | `ghost_x.x.x_aarch64.dmg` | M1/M2/M3/M4 Macs |
| **macOS** (Intel) | `ghost_x.x.x_x64.dmg` | Intel-based Macs |
| **Linux** (64-bit) | `ghost_x.x.x_amd64.deb` | Debian/Ubuntu |
| **Linux** (64-bit) | `ghost_x.x.x_amd64.AppImage` | Universal Linux |

| **Android** (ARM64) | `app-universal-release.apk` | Tauri v2 WebView, min SDK 24 |

> **No external dependencies required.** Ghost ships with native AI inference — no Ollama, no GPU, no internet needed after installation.

### Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) >= 18 or [Bun](https://bun.sh/) >= 1.0
- [Ollama](https://ollama.com/) (optional — Ghost uses native AI by default)
- Platform-specific Tauri v2 dependencies ([see guide](https://v2.tauri.app/start/prerequisites/))

### Installation

```bash
# Clone the repo
git clone https://github.com/ghostapp-ai/ghost.git
cd ghost

# Install frontend dependencies
bun install

# (Optional) Pull Ollama model for higher-quality 768D embeddings
# ollama pull nomic-embed-text

# Run in development mode (native AI model downloads on first run ~23MB)
bun run tauri dev
```

### Build for Production

```bash
# Desktop
bun run tauri build

# Android (requires Android SDK + NDK 27+)
bun run tauri android build --target aarch64
```

The desktop installer will be generated in `src-tauri/target/release/bundle/`.
The Android APK will be in `src-tauri/gen/android/app/build/outputs/apk/`.

## Project Structure

```
ghost/
├── src/                    # Frontend (React/TypeScript)
│   ├── components/         # UI components (Onboarding, GhostInput, ResultsList, Settings, StatusBar)
│   ├── hooks/              # Custom React hooks (useSearch, useHotkey, usePlatform)
│   ├── lib/                # Tauri IPC wrappers + TypeScript types + mode detection
│   ├── styles/             # Global CSS (Tailwind v4 theme, safe areas, touch targets)
│   └── App.tsx             # Root component (onboarding → main UI routing, platform-aware)
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── lib.rs          # Tauri commands: search, index, watcher, settings, platform info
│   │   ├── main.rs         # Entry point
│   │   ├── error.rs        # Error types (thiserror)
│   │   ├── settings.rs     # Persistent settings (JSON)
│   │   ├── chat/           # Chat engine: native Candle GGUF (desktop) + Ollama fallback
│   │   ├── indexer/        # File watcher (desktop) + text extraction + chunking
│   │   ├── db/             # SQLite + sqlite-vec + FTS5 (cross-platform FFI types)
│   │   ├── embeddings/     # Native Candle + Ollama engines + hardware detection
│   │   ├── search/         # Hybrid search engine + RRF ranking
│   │   └── protocols/      # MCP server/client, AG-UI, A2A, A2UI, WebMCP
│   ├── gen/android/        # Generated Android Gradle project (Tauri v2)
│   ├── capabilities/       # Platform-split permissions (default, desktop, mobile)
│   ├── Cargo.toml          # Rust deps (target-specific for desktop/mobile)
│   └── tauri.conf.json     # Tauri configuration + bundler config
├── branding/               # Brand assets (SVGs, PNGs, social, scripts)
├── ROADMAP.md              # Detailed development roadmap
├── CLAUDE.md               # Agent instructions for AI-assisted development
├── CONTRIBUTING.md         # Contribution guidelines
├── SECURITY.md             # Security policy & vulnerability disclosure
└── package.json            # Frontend dependencies
```

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the detailed development plan with phases, milestones, and technical deliverables.

## Privacy & Security

- **Zero telemetry**: Ghost collects no usage data, no analytics, no crash reports.
- **Local-only processing**: All AI inference runs on your machine — native Candle engine or optional Ollama.
- **Single file database**: Your entire vault is one `.db` file you control.
- **Optional encryption**: ChaCha20-Poly1305 for vault encryption when sync is enabled (Phase 2).
- **Open source core**: The engine is fully auditable.

## Contributing

Ghost is currently in early development. Contributions are welcome!

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding conventions, and PR guidelines.

For security vulnerabilities, see [SECURITY.md](SECURITY.md).

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) — Desktop framework
- [Ollama](https://ollama.com/) — Local LLM runtime
- [sqlite-vec](https://github.com/asg017/sqlite-vec) — Vector search for SQLite
- [Candle](https://github.com/huggingface/candle) — Rust ML framework for native AI inference
- [rmcp](https://crates.io/crates/rmcp) — Official Rust MCP SDK
- [MCP](https://modelcontextprotocol.io/) — Model Context Protocol (Linux Foundation / AAIF)
- [A2A](https://google.github.io/A2A) — Agent-to-Agent Protocol (Google / Linux Foundation)
- [AG-UI](https://github.com/CopilotKit/ag-ui) — Agent-User Interaction Protocol (CopilotKit)
- [OpenClaw](https://github.com/nicepkg/OpenClaw) — Model-agnostic agent infrastructure

---

<p align="center">
  <strong>Your data. Your machine. Your ghost.</strong>
</p>
