---
title: Chat Engine
description: Local LLM chat with hardware-aware model auto-selection and native inference.
---

Ghost includes a full chat engine powered by local LLMs — no cloud APIs, no subscriptions.

## How It Works

1. **Hardware Detection**: Ghost scans your CPU, RAM, GPU at startup
2. **Model Selection**: Automatically picks the largest model that fits comfortably
3. **Background Download**: Model downloads from HuggingFace Hub in the background
4. **Native Inference**: Runs via Candle GGUF (desktop) or Ollama fallback

## Chat Models

### Desktop (Native Candle GGUF)

| Tier | Model | Size | RAM Required |
|------|-------|------|-------------|
| Tiny | Qwen2.5-0.5B-Instruct-Q4_K_M | ~400MB | 2 GB |
| Small | Qwen2.5-1.5B-Instruct-Q4_K_M | ~1.1GB | 4 GB |
| Medium | Qwen2.5-3B-Instruct-Q4_K_M | ~2.0GB | 8 GB |
| Large | Qwen2.5-7B-Instruct-Q4_K_M | ~4.3GB | 16 GB |

### Agent Mode (Ollama — optional)

For the ReAct agent loop with tool calling, Ghost uses Qwen3 via Ollama:

| Tier | Model | RAM Required |
|------|-------|-------------|
| Micro | Qwen3-0.6B | 2 GB |
| Tiny | Qwen3-1.7B | 4 GB |
| Small | Qwen3-4B | 6 GB |
| Medium | Qwen3-8B | 10 GB |
| Large | Qwen3-14B | 18 GB |
| XL | Qwen3-32B | 36 GB |

## Chat Interface

- **Unified Omnibox**: Type naturally — Ghost auto-detects chat intent
- **Streaming responses**: Token-by-token output via AG-UI events
- **Conversation memory**: Persisted in SQLite with FTS5 search across past conversations
- **Debug panel**: See reasoning, tool calls, and timing with Ctrl+D

## Configuration

All chat settings are configurable via Settings (Ctrl+,):

- **Model**: Auto-select or manual choice
- **Temperature**: 0.0 (deterministic) to 2.0 (creative)
- **Max tokens**: Response length limit
- **Device**: CPU (default), CUDA, Metal
