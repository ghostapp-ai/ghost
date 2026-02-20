---
title: Embedding Engine
description: How Ghost generates vector embeddings natively and via Ollama fallback.
---

Ghost uses a **fallback chain** architecture for generating text embeddings used in semantic search.

## Fallback Chain

```
┌─────────────────────────────┐
│  Native Candle Engine       │  Priority 1: In-process
│  all-MiniLM-L6-v2 (384D)   │  Zero external dependencies
├─────────────────────────────┤
│  Ollama Engine              │  Priority 2: HTTP fallback
│  nomic-embed-text (768D)    │  Higher quality, needs Ollama
├─────────────────────────────┤
│  FTS5 Only                  │  Priority 3: Degraded
│  Keyword search only        │  Always works, no vectors
└─────────────────────────────┘
```

## Native Engine (Primary)

**Model**: [all-MiniLM-L6-v2](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)

| Property | Value |
|----------|-------|
| Dimensions | 384 |
| Model size | ~23MB (safetensors) |
| Framework | Candle (pure Rust) |
| Tokenizer | HuggingFace tokenizers |
| Platform | Any CPU (x86_64, ARM64) |
| SIMD | AVX2 (Intel/AMD), NEON (ARM) |
| First load | Downloads from HuggingFace Hub |
| Subsequent loads | <200ms from cache |

### How It Works

1. Text is tokenized using HuggingFace `tokenizers` crate
2. Tokens pass through BERT model layers via Candle
3. Mean pooling over token embeddings produces a 384-dim vector
4. Vector is stored in sqlite-vec for KNN search

### Hardware Detection

Ghost automatically detects optimal settings:

```rust
pub struct HardwareInfo {
    pub cpu_cores: usize,
    pub has_avx2: bool,      // x86 SIMD
    pub has_neon: bool,      // ARM SIMD
    pub gpu_backend: Option<GpuBackend>,  // CUDA/Metal/Vulkan
    pub total_ram_mb: u64,
}
```

## Ollama Engine (Fallback)

If the native engine fails (e.g., model download interrupted), Ghost falls back to Ollama:

- **Model**: `nomic-embed-text` (768 dimensions)
- **API**: `POST http://localhost:11434/api/embeddings`
- **Quality**: Higher quality than MiniLM but requires Ollama running

## Vector Dimensions

The database adapts to whichever engine is active:

| Engine | Dimensions | sqlite-vec table |
|--------|-----------|-----------------|
| Native | 384 | `FLOAT[384]` |
| Ollama | 768 | `FLOAT[768]` |

When switching engines, vectors are re-generated for consistency.

## Chunking Strategy

Before embedding, documents are split into chunks:

- **Chunk size**: 512 tokens
- **Overlap**: 64 tokens
- **Strategy**: Sentence-boundary aware splitting

This ensures each vector represents a coherent semantic unit while maintaining context across boundaries.
