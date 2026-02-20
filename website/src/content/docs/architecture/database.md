---
title: Database Schema
description: Ghost's SQLite + sqlite-vec + FTS5 database design for unified storage.
---

Ghost stores everything in a single SQLite database file — documents, vectors, full-text indexes, chat history, and settings.

## Core Tables

### Documents

```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    extension TEXT,
    size_bytes INTEGER,
    hash TEXT NOT NULL,              -- SHA-256 for change detection
    indexed_at TEXT NOT NULL,        -- ISO 8601
    modified_at TEXT NOT NULL,       -- File's mtime
    metadata TEXT                    -- JSON blob
);
```

### Chunks

```sql
CREATE TABLE chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL
        REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER,
    UNIQUE(document_id, chunk_index)
);
```

### FTS5 Full-Text Search

```sql
CREATE VIRTUAL TABLE chunks_fts USING fts5(
    content,
    content=chunks,
    content_rowid=id,
    tokenize='porter unicode61'
);
```

### Vector Search (sqlite-vec)

```sql
CREATE VIRTUAL TABLE chunks_vec USING vec0(
    chunk_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]  -- 384 for native, 768 for Ollama
);
```

### Conversation Memory

```sql
CREATE TABLE conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id INTEGER NOT NULL
        REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL,     -- 'user', 'assistant', 'system', 'tool'
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

## Hybrid Search Query Pattern

```sql
-- 1. FTS5 keyword search (fast, <5ms)
SELECT rowid, rank FROM chunks_fts
WHERE chunks_fts MATCH ?;

-- 2. KNN vector search (semantic, <500ms)
SELECT chunk_id, distance FROM chunks_vec
WHERE embedding MATCH ?
ORDER BY distance LIMIT 20;

-- 3. Combine with RRF in Rust
-- RRF score = sum(1 / (k + rank_i))
-- k = 60 (standard RRF constant)
```

## Why Single-File SQLite?

- **Portable**: One `.db` file = your entire vault
- **Atomic**: Transactions ensure consistency
- **Fast**: FTS5 + sqlite-vec both use SQLite's optimized I/O
- **Backup**: Copy one file to backup everything
- **No server**: Zero infrastructure, zero configuration
