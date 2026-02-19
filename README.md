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
- **AI-Powered**: Local LLM reasoning via Ollama for natural language queries and actions.
- **Lightweight**: <10MB installer, <40MB RAM idle, <500ms launch. 70% less RAM than Electron.
- **Extensible**: MCP Server protocol for skills/plugins — compatible with the growing ecosystem of 10,000+ MCP servers.

## Features

### Phase 0 — Foundation (Current)
- Tauri v2 desktop shell with React/TypeScript frontend
- Rust core engine for maximum performance and security
- SQLite + sqlite-vec for unified storage (documents, vectors, text)
- Ollama integration with nomic-embed-text for local embeddings
- File watcher for real-time document indexing

### Phase 1 — The Search Bar
- Global hotkey launcher (Cmd/Ctrl+Space)
- Automatic file indexing with text extraction (PDF, DOCX, XLSX, TXT)
- Hybrid search: FTS5 keyword + KNN vector in a single query
- Virtualized results list with RRF ranking

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
│  File Watcher │ Text Extractor │ Embedding Pipeline│
│  Vector DB │ HTTP Server │ Encryption              │
├─────────────────────────────────────────────────┤
│              AI Layer (Local)                      │
│  Ollama + nomic-embed-text (embeddings)           │
│  Ollama + Qwen2.5-7B (reasoning/tool calling)    │
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
| Embeddings | Ollama + nomic-embed-text | 768 dims, local, surpasses OpenAI ada-002 |
| LLM | Ollama + Qwen2.5-7B Q4 | Tool calling, multilingual, 4GB VRAM |
| Agent | MCP Server (TypeScript/Rust) | Open standard, 10,000+ compatible servers |
| File Watcher | notify (Rust crate) | Cross-platform, async, <1% CPU |
| Text Extraction | lopdf + docx-rs + calamine | Pure Rust, no external dependencies |
| Encryption | ChaCha20-Poly1305 (age crate) | Modern, audited |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) >= 18 or [Bun](https://bun.sh/) >= 1.0
- [Ollama](https://ollama.com/) installed and running
- Platform-specific Tauri v2 dependencies ([see guide](https://v2.tauri.app/start/prerequisites/))

### Installation

```bash
# Clone the repo
git clone https://github.com/your-username/ghost.git
cd ghost

# Install frontend dependencies
bun install

# Pull the embedding model
ollama pull nomic-embed-text

# Run in development mode
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
│   ├── components/         # UI components
│   ├── hooks/              # Custom React hooks
│   ├── stores/             # State management
│   └── App.tsx             # Root component
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── lib.rs          # Tauri app setup + commands
│   │   ├── main.rs         # Entry point
│   │   ├── indexer/        # File watcher + text extraction
│   │   ├── db/             # SQLite + sqlite-vec + FTS5
│   │   ├── embeddings/     # Ollama embedding pipeline
│   │   ├── search/         # Hybrid search engine
│   │   ├── mcp/            # MCP server implementation
│   │   └── automation/     # OS UI automation layer
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
- **Local-only processing**: All AI inference runs on your machine via Ollama.
- **Single file database**: Your entire vault is one `.db` file you control.
- **Optional encryption**: ChaCha20-Poly1305 for vault encryption when sync is enabled.
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
