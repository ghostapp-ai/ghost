---
title: Hybrid Search
description: How Ghost combines keyword and semantic search for instant, accurate results.
---

Ghost's search engine combines two complementary approaches for the best possible results.

## How It Works

### 1. FTS5 Keyword Search (< 5ms)

Uses SQLite's Full-Text Search 5 with Porter stemming and Unicode tokenization:

```sql
SELECT rowid, rank FROM chunks_fts WHERE chunks_fts MATCH ?;
```

**Best for**: exact filenames, code symbols, specific phrases.

### 2. Vector Semantic Search (< 500ms)

Uses sqlite-vec for K-Nearest Neighbor search on embedding vectors:

```sql
SELECT chunk_id, distance FROM chunks_vec
WHERE embedding MATCH ? ORDER BY distance LIMIT 20;
```

**Best for**: conceptual queries, "find files about X", natural language.

### 3. Reciprocal Rank Fusion (RRF)

Both result sets are combined using RRF scoring:

```
RRF_score = Σ(1 / (k + rank_i)) for each ranking system
```

Where `k = 60` (standard constant). This ensures results that appear in both keyword and semantic search rank highest.

## Embedding Engine

Ghost uses a fallback chain for embeddings:

| Priority | Engine | Dimensions | Size | Speed |
|----------|--------|-----------|------|-------|
| 1 (Primary) | **Native Candle** — all-MiniLM-L6-v2 | 384 | ~23MB | Instant (in-process) |
| 2 (Fallback) | **Ollama** — nomic-embed-text | 768 | ~274MB | HTTP call |
| 3 (Degraded) | **FTS5 only** — no vectors | N/A | 0 | <5ms keyword only |

The native engine runs in-process with zero external dependencies — no Ollama, no GPU, no internet after first model download.

## File Types Indexed

Ghost extracts text from:

- **Documents**: PDF, DOCX, XLSX, TXT, Markdown
- **Code**: 50+ extensions (`.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.cpp`, `.c`, `.rb`, `.php`, etc.)
- **Data**: JSON, YAML, TOML, XML, CSV

## Performance Benchmarks

| Metric | Target | Actual |
|--------|--------|--------|
| FTS5 keyword search | <5ms | ✅ <3ms typical |
| Semantic vector search | <500ms | ✅ <200ms typical |
| File indexing (per file) | <100ms | ✅ ~50ms typical |
| Background CPU usage | <10% | ✅ ~5% during indexing |
