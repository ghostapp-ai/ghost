---
title: Native AI Engine
description: Ghost's in-process Candle-based AI inference — zero external dependencies.
---

Ghost runs AI models **natively in-process** using [Candle](https://github.com/huggingface/candle), HuggingFace's Rust ML framework. No Ollama, no Python, no GPU required.

## Embedding Engine

### all-MiniLM-L6-v2

- **Dimensions**: 384
- **Model size**: ~23MB (safetensors)
- **Inference**: In-process via Candle — zero HTTP overhead
- **Platform**: Any CPU (x86_64 or ARM64)
- **SIMD**: Auto-detects AVX2 (Intel/AMD) or NEON (ARM)

```rust
// How Ghost generates embeddings natively
let engine = NativeEngine::load().await?;
let embedding: Vec<f32> = engine.embed("search query")?; // 384-dim
```

The model downloads once from HuggingFace Hub (~23MB) and is cached locally. Subsequent loads take <200ms.

## Chat Engine

### Qwen2.5-Instruct GGUF

Ghost auto-selects the largest chat model that fits in your RAM:

| RAM | Model | Parameters | VRAM/RAM |
|-----|-------|-----------|----------|
| 2-4 GB | Qwen2.5-0.5B-Instruct | 0.5B | ~0.5 GB |
| 4-8 GB | Qwen2.5-1.5B-Instruct | 1.5B | ~1.2 GB |
| 8-16 GB | Qwen2.5-3B-Instruct | 3B | ~2.2 GB |
| 16+ GB | Qwen2.5-7B-Instruct | 7B | ~4.5 GB |

All models use **Q4_K_M quantization** for efficient inference on consumer hardware.

## Hardware Detection

On startup, Ghost detects:

- **CPU cores** — for parallel chunk processing
- **RAM** — for automatic model tier selection
- **SIMD** — AVX2 (Intel/AMD) or NEON (ARM) for faster tensor operations
- **GPU** — CUDA (NVIDIA), Metal (Apple), Vulkan (cross-platform)

This information is displayed in the onboarding wizard and status bar.

## Fallback Chain

Ghost gracefully degrades based on available resources:

```
Native Candle (in-process, zero deps)
    ↓ if native fails
Ollama HTTP API (localhost:11434)
    ↓ if Ollama unavailable
FTS5 keyword-only search (always works)
```

This ensures Ghost **always works** — even on low-end hardware with no internet connection.
