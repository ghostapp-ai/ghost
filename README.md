<p align="center">
  <img src="public/ghost-logo.svg" width="120" alt="Ghost Logo" />
</p>

<h1 align="center">GHOST</h1>

<p align="center">
  <strong>Private Local AI Superpowers for Your OS</strong>
</p>

<p align="center">
  <a href="#features">Features</a> &bull;
  <a href="#architecture">Architecture</a> &bull;
  <a href="#getting-started">Getting Started</a> &bull;
  <a href="#roadmap">Roadmap</a> &bull;
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version" />
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License" />
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey" alt="Platform" />
  <img src="https://img.shields.io/badge/privacy-100%25%20local-brightgreen" alt="Privacy" />
</p>

---

Ghost is a private, local-first AI assistant that lives in your OS. It indexes your files, understands your context, and helps you find anything — all without sending a single byte to the cloud.

Think **Raycast + Semantic Search + Local AI Agent** — but private by design.

## Why Ghost?

- **100% Local**: Your data never leaves your machine. Zero telemetry, zero cloud dependencies.
- **Instant Search**: Hybrid keyword (FTS5) + semantic vector search across all your documents.
- **Native AI**: In-process embedding inference via Candle — no Ollama or internet required after first model download.
- **Lightweight**: <10MB installer, <40MB RAM idle, <500ms launch. 70% less RAM than Electron.
- **Extensible**: MCP Server protocol for skills/plugins — compatible with the growing ecosystem of 10,000+ MCP servers.

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
- 21 unit tests passing, zero compiler warnings

### Phase 1 — The Search Bar (**Complete**)
- **Spotlight-like floating window**: `Ctrl/Cmd+Space` global shortcut, decorationless, always-on-top, transparent
- Auto-hide on focus loss, Escape to dismiss, draggable title region
- Dark ghost-themed UI with Tailwind CSS v4
- Debounced search input (150ms) with loading skeletons
- Virtualized results list (`@tanstack/react-virtual`) with file type icons
- **Open files**: Enter key or double-click opens files with system default app
- Keyboard navigation: arrow keys, Enter to open, Esc to dismiss
- Settings panel: watched directory management, persistent settings (JSON)
- Auto-start file watcher on launch with saved directories
- Onboarding flow for first-time users
- Status bar: document count, AI engine status, vector search status
- Cross-platform CI/CD: Windows, macOS (ARM64 + Intel), Linux installers
- 21 tests, zero warnings, ~185KB JS bundle

### Phase 2 — The Memory
- Browser history indexing via UI Automation
- App activity timeline ("What was I doing last Tuesday?")
- Clipboard history with semantic search
- Premium features: sync, encryption, more powerful models

### Phase 3 — The Agent
- Local MCP Server with Qwen2.5-7B for tool calling
- Actions: open apps, copy text, search web, create files
- Action Preview: see what Ghost will do before it executes
- Natural language OS control

### Phase 4 — The Platform
- Skills SDK for third-party MCP servers
- Integrations: Obsidian, VS Code, Slack, browsers
- Mac port
- B2B/teams licensing

## Architecture

Ghost uses a 4-layer architecture where each layer is independently replaceable:

```
┌─────────────────────────────────────────────────┐
│              Frontend (React/TypeScript)          │
│  Search Bar │ Results │ Chat │ Settings │ Vault   │
├─────────────────────────────────────────────────┤
│                 Tauri v2 IPC Bridge               │
├─────────────────────────────────────────────────┤
│              Core Engine (Rust)                    │
│  File Watcher │ Text Extractor │ Embedding Engine  │
│  Vector DB │ HTTP Server │ Encryption              │
├─────────────────────────────────────────────────┤
│              AI Layer (Local — Zero Dependencies)  │
│  Native: Candle + all-MiniLM-L6-v2 (384D)        │
│  Fallback: Ollama + nomic-embed-text (768D)       │
│  Future: Qwen2.5-7B (reasoning/tool calling)     │
│  MCP Server (agent actions)                       │
└─────────────────────────────────────────────────┘
```

### Hybrid Trigger System

Ghost uses a two-speed architecture to feel instant without burning CPU:

| Layer | When | Speed | Resource Usage |
|-------|------|-------|----------------|
| **Fast Layer** | Always | <10ms | 0% GPU, <1% CPU |
| **Smart Layer** | On demand | 200-2000ms | Activates Ollama |

The Fast Layer uses OS accessibility APIs and FTS5 keyword search. The Smart Layer activates only when the user asks a natural language question, requests an action, or a new file needs indexing.

### Tech Stack

| Component | Technology | Why |
|-----------|-----------|-----|
| Shell/UI | Tauri v2 + React/TypeScript | <10MB installer, native performance |
| Database | SQLite + sqlite-vec + FTS5 | Single .db file, vectors + text + metadata |
| Native Embeddings | Candle + all-MiniLM-L6-v2 | 384D, ~23MB, in-process, no external deps |
| Fallback Embeddings | Ollama + nomic-embed-text | 768D, optional, higher quality for large models |
| LLM | Ollama + Qwen2.5-7B Q4 | Tool calling, multilingual, 4GB VRAM |
| Agent | MCP Server (TypeScript/Rust) | Open standard, 10,000+ compatible servers |
| File Watcher | notify (Rust crate) | Cross-platform, async, <1% CPU |
| Text Extraction | lopdf + zip + calamine | Pure Rust, no external dependencies |
| Encryption | ChaCha20-Poly1305 (age crate) | Modern, audited |

## Getting Started

### Download & Install

Download the latest release for your platform from [**GitHub Releases**](https://github.com/AngelAlexQC/ghost/releases/latest):

| Platform | File | Notes |
|----------|------|-------|
| **Windows** (64-bit) | `ghost_x.x.x_x64-setup.exe` | NSIS installer, no admin required |
| **macOS** (Apple Silicon) | `ghost_x.x.x_aarch64.dmg` | M1/M2/M3/M4 Macs |
| **macOS** (Intel) | `ghost_x.x.x_x64.dmg` | Intel-based Macs |
| **Linux** (64-bit) | `ghost_x.x.x_amd64.deb` | Debian/Ubuntu |
| **Linux** (64-bit) | `ghost_x.x.x_amd64.AppImage` | Universal Linux |

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
git clone https://github.com/AngelAlexQC/ghost.git
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
bun run tauri build
```

The installer will be generated in `src-tauri/target/release/bundle/`.

## Project Structure

```
ghost/
├── src/                    # Frontend (React/TypeScript)
│   ├── components/         # UI components (SearchBar, ResultsList, Settings, StatusBar)
│   ├── hooks/              # Custom React hooks (useSearch, useHotkey)
│   ├── lib/                # Tauri IPC wrappers + TypeScript types
│   ├── styles/             # Global CSS (Tailwind v4 theme)
│   └── App.tsx             # Root component with keyboard navigation
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── lib.rs          # Tauri commands: search, index, watcher, settings, window mgmt
│   │   ├── main.rs         # Entry point
│   │   ├── error.rs        # Error types (thiserror)
│   │   ├── settings.rs     # Persistent settings (JSON)
│   │   ├── indexer/        # File watcher + text extraction + chunking
│   │   ├── db/             # SQLite + sqlite-vec + FTS5 (schema + CRUD)
│   │   ├── embeddings/     # Native Candle + Ollama embedding engines
│   │   └── search/         # Hybrid search engine + RRF ranking
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── ROADMAP.md              # Detailed development roadmap
├── CLAUDE.md               # Agent instructions for AI-assisted development
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

Ghost is currently in early development. Contributions are welcome:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please read [CLAUDE.md](CLAUDE.md) for development conventions and coding standards.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) — Desktop framework
- [Ollama](https://ollama.com/) — Local LLM runtime
- [sqlite-vec](https://github.com/asg017/sqlite-vec) — Vector search for SQLite
- [nomic-embed-text](https://ollama.com/library/nomic-embed-text) — Embedding model
- [MCP](https://modelcontextprotocol.io/) — Model Context Protocol (Linux Foundation / Agentic AI Foundation)

---

<p align="center">
  <strong>Your data. Your machine. Your ghost.</strong>
</p>
