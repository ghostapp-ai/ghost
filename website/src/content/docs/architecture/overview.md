---
title: Architecture Overview
description: Ghost's 6-layer Agent OS architecture — from frontend to AI inference.
---

Ghost uses a 6-layer architecture where each layer is independently replaceable.

## Layer Diagram

```
┌──────────────────────────────────────────────────────┐
│  Layer 6: Frontend (React/TypeScript)                 │
│  Omnibox │ Results │ Chat │ A2UI Renderer │ Settings  │
├──────────────────────────────────────────────────────┤
│  Layer 5: AG-UI Runtime (Agent ↔ User Streaming)      │
│  ~16 event types │ Human-in-the-loop │ State sync      │
├──────────────────────────────────────────────────────┤
│  Layer 4: Tauri v2 IPC Bridge                         │
│  invoke() commands │ event system │ platform APIs      │
├──────────────────────────────────────────────────────┤
│  Layer 3: Agent Engine (ReAct Loop)                   │
│  Executor │ Tools │ Safety │ Memory │ Skills           │
├──────────────────────────────────────────────────────┤
│  Layer 2: Protocol Hub (Rust)                         │
│  MCP Server │ MCP Client │ A2A │ WebMCP │ Skills       │
├──────────────────────────────────────────────────────┤
│  Layer 1: Core Engine (Rust)                          │
│  File Watcher │ Text Extractor │ Embedding Engine      │
│  Vector DB │ Hybrid Search │ Settings                  │
├──────────────────────────────────────────────────────┤
│  Layer 0: AI Layer (Local)                            │
│  Candle BERT │ Qwen2.5 GGUF │ Qwen3 Ollama            │
└──────────────────────────────────────────────────────┘
```

## Project Structure

```
ghost/
├── src/                    # Frontend (React/TypeScript)
│   ├── components/         # UI components
│   ├── hooks/              # Custom React hooks
│   ├── lib/                # Tauri IPC wrappers + types
│   └── styles/             # Tailwind CSS v4
├── src-tauri/              # Backend (Rust)
│   └── src/
│       ├── lib.rs          # Tauri commands
│       ├── agent/          # ReAct agent engine
│       ├── chat/           # Chat engine (native + Ollama)
│       ├── db/             # SQLite + FTS5 + sqlite-vec
│       ├── embeddings/     # Candle + Ollama engines
│       ├── indexer/        # File watcher + text extraction
│       ├── protocols/      # MCP, AG-UI, A2A, A2UI
│       └── search/         # Hybrid search + RRF
├── website/                # This documentation site
└── branding/               # Brand assets (SVG, PNG)
```

## Hybrid Trigger System

Ghost uses a two-speed architecture:

| Layer | When | Speed | Resource Usage |
|-------|------|-------|---------------|
| **Fast Layer** | Always | <10ms | 0% GPU, <1% CPU |
| **Smart Layer** | On demand | 200-2000ms | Activates AI |

The **Fast Layer** uses FTS5 keyword search and OS accessibility APIs — instant, zero GPU.

The **Smart Layer** activates only when:
- User asks a natural language question
- User requests an action
- New file needs embedding

## Communication Flow

```
User Input → Frontend (React)
    ↓ invoke()
Tauri IPC Bridge
    ↓
Rust Command Handler (lib.rs)
    ↓
Core Engine (search/index/chat)
    ↓ emit()
AG-UI Event Bus → Frontend (streaming updates)
```

All heavy computation happens in Rust. The frontend is a thin rendering layer.
