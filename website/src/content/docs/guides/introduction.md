---
title: Introduction
description: What is Ghost and why it exists — a private, local-first Agent OS for desktop and mobile.
---

Ghost is a private, local-first **Agent OS** for desktop and mobile. It indexes your files, understands your context, connects to thousands of tools via open protocols (MCP, A2A, AG-UI, A2UI, WebMCP), and takes actions on your behalf — all without sending a single byte to the cloud.

Think **Raycast + Semantic Search + Local AI Agent + Universal Protocol Hub** — but private by design.

## Key Principles

### 1. Privacy is Non-Negotiable
- Zero telemetry, zero analytics, zero cloud dependencies
- All AI inference runs locally (native Candle or optional Ollama)
- Your data never leaves your machine

### 2. Performance Matters
- App cold start: <500ms
- FTS5 keyword search: <5ms
- Semantic vector search: <500ms
- Idle RAM: <40MB
- Installer size: <10MB

### 3. Zero-Config Experience
- Auto-discovers Documents, Desktop, Downloads, Pictures on first launch
- Hardware-aware model selection — detects RAM/CPU/GPU and picks the best model
- Works without Ollama, without GPU, without internet (after first model download)

## Who Is Ghost For?

- **Privacy-conscious users** who want AI assistance without cloud surveillance
- **Developers** who need fast local search across code, docs, and notes
- **Power users** who want a Spotlight/Raycast alternative that works on all platforms
- **AI enthusiasts** who want to run agents locally with full control

## Current Status

Ghost is in **Phase 1.5+** — the Protocol Bridge phase. Core search, chat, and protocol support are complete. The agent engine with ReAct loop and tool calling is functional. See the [Roadmap](/ghost/reference/roadmap/) for the full development plan.

## Tech Stack

| Component | Technology | Why |
|-----------|-----------|-----|
| Shell/UI | Tauri v2 + React/TypeScript | <10MB installer, native performance |
| Database | SQLite + sqlite-vec + FTS5 | Single .db file, vectors + text + metadata |
| Native Embeddings | Candle + all-MiniLM-L6-v2 | 384D, ~23MB, in-process, zero external deps |
| LLM / Chat | Candle GGUF + Qwen2.5-Instruct | Native inference, 0.5B–7B tiers |
| Agent Engine | Qwen3 via Ollama + ReAct loop | Hardware-adaptive (0.6B–32B), tool calling |
| MCP Protocol | rmcp (official Rust SDK) | Server + Client, 10,000+ servers |
| Agent Interaction | AG-UI + A2UI | Real-time streaming + generative UI |

## License

Ghost core is [MIT licensed](https://github.com/ghostapp-ai/ghost/blob/main/LICENSE) — fully open source and auditable. A Pro tier with premium features (sync, encryption, unlimited automations) is planned.
