<p align="center">
  <img src="public/ghost-logo.svg" width="100" alt="Ghost Logo" />
</p>

<h1 align="center">Ghost</h1>

<p align="center">
  <strong>The Private Agent OS for Desktop & Mobile</strong><br/>
  <sub>Index files · Run AI agents · Connect to 10,000+ tools — all without sending data to the cloud</sub>
</p>

<p align="center">
  <a href="https://ghostapp-ai.github.io/ghost">Website</a> ·
  <a href="https://ghostapp-ai.github.io/ghost/guides/installation/">Download</a> ·
  <a href="#features">Features</a> ·
  <a href="https://ghostapp-ai.github.io/ghost/architecture/overview/">Architecture</a> ·
  <a href="ROADMAP.md">Roadmap</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

<p align="center">
  <a href="https://github.com/ghostapp-ai/ghost/releases/latest"><img src="https://img.shields.io/github/v/release/ghostapp-ai/ghost?style=flat-square&color=7c3aed&label=latest" alt="Release" /></a>
  <a href="https://github.com/ghostapp-ai/ghost/blob/main/LICENSE"><img src="https://img.shields.io/github/license/ghostapp-ai/ghost?style=flat-square&color=10b981" alt="License" /></a>
  <a href="https://github.com/ghostapp-ai/ghost/actions/workflows/ghost.yml"><img src="https://img.shields.io/github/actions/workflow/status/ghostapp-ai/ghost/ghost.yml?branch=main&style=flat-square&label=CI" alt="CI" /></a>
  <a href="https://github.com/ghostapp-ai/ghost/stargazers"><img src="https://img.shields.io/github/stars/ghostapp-ai/ghost?style=flat-square&color=f59e0b" alt="Stars" /></a>
  <a href="https://github.com/ghostapp-ai/ghost/issues"><img src="https://img.shields.io/github/issues/ghostapp-ai/ghost?style=flat-square" alt="Issues" /></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Windows-0078D6?style=flat-square&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/macOS-000000?style=flat-square&logo=apple&logoColor=white" alt="macOS" />
  <img src="https://img.shields.io/badge/Linux-FCC624?style=flat-square&logo=linux&logoColor=black" alt="Linux" />
  <img src="https://img.shields.io/badge/Android-34A853?style=flat-square&logo=android&logoColor=white" alt="Android" />
  <img src="https://img.shields.io/badge/Privacy-100%25_Local-7c3aed?style=flat-square" alt="Privacy" />
</p>

---

Ghost is a private, local-first **Agent OS** for desktop and mobile. It indexes your files, understands your context, connects to thousands of tools via open protocols, and takes actions on your behalf — without sending a single byte to the cloud.

**Your data should never leave your machine to get things done.**

Ghost runs AI natively on your hardware — no cloud APIs, no GPU requirements, no external dependencies. From semantic search to agentic tool calling, everything happens locally. It speaks the complete 2026 agent protocol stack (MCP, AG-UI, A2UI, A2A) so you connect to every AI ecosystem without giving up privacy.

## Features

### Search — Instant & Intelligent

- **<5ms keyword search** (FTS5) + **<500ms semantic search** (sqlite-vec KNN) fused via [Reciprocal Rank Fusion](https://plg.uwaterloo.ca/~gplatt/tutorials/tutcomb.pdf)
- Native **all-MiniLM-L6-v2** embeddings (384D, ~23MB) via Candle — zero external dependencies
- Fallback chain: Native Candle → Ollama (768D) → keyword-only
- Real-time file watcher for PDF, DOCX, XLSX, TXT, Markdown, and 50+ code formats

### AI — Native & Hardware-Adaptive

- Auto-detects CPU/RAM/GPU → selects optimal **Qwen2.5-Instruct GGUF** (0.5B–7B, Q4_K_M)
- **ReAct agent**: Reason → Act → Observe with grammar-constrained tool calling and 3-tier safety
- Zero-config: detect hardware → select model → download from HuggingFace Hub → load in background
- Graceful fallback: Native GGUF → Ollama HTTP → offline mode

### Protocols — The Complete 2026 Agent Stack

Ghost is the first desktop app implementing every major agent protocol — no vendor lock-in, no proprietary APIs.

| Protocol | Status | What it does |
|----------|--------|--------------|
| **MCP** | ✅ Server + Client | Expose Ghost tools + connect to 10,000+ external servers via `rmcp` |
| **MCP Apps** | 🔜 Next | Render interactive tool UIs in-conversation (official MCP extension) |
| **AG-UI** | ✅ Runtime | Bidirectional agent↔user streaming — ~16 event types, SSE endpoint |
| **A2UI** | ✅ Renderer | Generative UI — 17+ component types rendered natively as React/Tailwind |
| **Skills** | ✅ Registry | YAML frontmatter skill definitions + trigger matching |
| **A2A** | 🔜 Next | Multi-agent coordination via Agent Cards + JSON-RPC 2.0 |
| **WebMCP** | 🔜 Planned | W3C browser bridge for structured web interactions |

### Platforms — One Codebase, Five Targets

- **Windows** (NSIS) · **macOS** (DMG × 2) · **Linux** (DEB/RPM/AppImage) · **Android** (APK/AAB) · **iOS** (ready)
- **<10MB installer** · **<40MB RAM** idle · **<500ms** cold start
- Conditional compilation (`#[cfg(desktop)]` / `#[cfg(mobile)]`) — single Rust codebase
- Onboarding wizard, system tray, zero-config file discovery

### Roadmap

See [**ROADMAP.md →**](ROADMAP.md) for the full development plan.

- **Next**: MCP Apps interactive UIs, A2A multi-agent coordination, OS UI automation
- **Then**: WebMCP browser bridge, Skills Marketplace, B2B/Teams

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
│  MCP Server │ MCP Client │ MCP Apps │ A2A │ WebMCP     │
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

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Shell | Tauri v2 + React 18 | <10MB installer, native perf, 5 platforms |
| Database | SQLite + sqlite-vec + FTS5 | Vectors + text + metadata in one `.db` |
| Embeddings | Candle + all-MiniLM-L6-v2 | 384D, ~23MB, in-process, zero deps |
| Chat / Agent | Candle GGUF + Qwen2.5-Instruct | 0.5B–7B tiers, tool calling, ReAct |
| Protocols | rmcp · AG-UI · A2UI · MCP Apps · A2A | MCP server+client, streaming, gen UI |
| Extraction | lopdf · zip · calamine | PDF, DOCX, XLSX — pure Rust |

## Download

Get the latest release from [**Releases**](https://github.com/ghostapp-ai/ghost/releases/latest) or the [**website**](https://ghostapp-ai.github.io/ghost/guides/installation/):

| Platform | Format | Notes |
|----------|--------|-------|
| **Windows** x64 | `.exe` (NSIS) | No admin required, WebView2 auto-bootstrap |
| **macOS** Apple Silicon | `.dmg` | M1 / M2 / M3 / M4 |
| **macOS** Intel | `.dmg` | x64, macOS ≥ 10.15 |
| **Linux** x64 | `.deb` `.rpm` `.AppImage` | Debian, Fedora, universal |
| **Android** ARM64 | `.apk` `.aab` | Min SDK 24, Tauri v2 WebView |

> **No external dependencies.** Ghost ships with native AI — no Ollama, no GPU, no internet after first install.

## Build from Source

**Prerequisites**: [Rust](https://rustup.rs/) (stable) · [Bun](https://bun.sh/) ≥ 1.0 (or Node ≥ 18) · [Tauri v2 deps](https://v2.tauri.app/start/prerequisites/)

```bash
git clone https://github.com/ghostapp-ai/ghost.git && cd ghost
bun install
bun run tauri dev          # Dev mode — native model downloads ~23MB on first run
bun run tauri build        # Production build → src-tauri/target/release/bundle/
```

```bash
# Android (requires SDK + NDK 27+)
bun run tauri android build --target aarch64
```

Optionally install [Ollama](https://ollama.com/) and pull `nomic-embed-text` for higher-quality 768D embeddings.

## Project Structure

```
src/                  # React 18 + TypeScript frontend
  components/         # Onboarding, GhostInput, ChatMessages, A2UIRenderer, Settings …
  hooks/              # useSearch, useAgui, usePlatform, useHotkey
  lib/                # Tauri IPC wrappers, types, mode detection

src-tauri/src/        # Rust backend
  indexer/            # File watcher + text extraction + chunking
  db/                 # SQLite · sqlite-vec · FTS5
  embeddings/         # Native Candle + Ollama fallback + hardware detection
  search/             # Hybrid search + RRF ranking
  chat/               # Candle GGUF inference + model registry
  agent/              # ReAct executor + tools + safety + memory + skills
  protocols/          # MCP server/client · AG-UI · A2UI · A2A · WebMCP

website/              # Astro Starlight documentation (GitHub Pages)
branding/             # Icons, social, brand guidelines
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

Contributions welcome! Read [CONTRIBUTING.md](CONTRIBUTING.md) for setup and conventions. Security issues → [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE) — free and open source.

## Acknowledgments

[Tauri](https://tauri.app/) · [Candle](https://github.com/huggingface/candle) · [sqlite-vec](https://github.com/asg017/sqlite-vec) · [rmcp](https://crates.io/crates/rmcp) · [Ollama](https://ollama.com/) · [MCP](https://modelcontextprotocol.io/) · [A2A](https://google.github.io/A2A) · [AG-UI](https://github.com/CopilotKit/ag-ui) · [OpenClaw](https://github.com/nicepkg/OpenClaw)

---

<p align="center">
  <strong>Your data · Your machine · Your ghost</strong><br>
  <a href="https://ghostapp-ai.github.io/ghost">Website</a> · <a href="https://github.com/ghostapp-ai/ghost/releases/latest">Download</a> · <a href="https://github.com/ghostapp-ai/ghost/discussions">Discussions</a>
</p>
