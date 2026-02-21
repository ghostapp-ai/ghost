use rusqlite::Connection;

use crate::error::Result;

/// Initialize the database schema with all required tables.
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Documents table: one row per indexed file
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            filename TEXT NOT NULL,
            extension TEXT,
            size_bytes INTEGER,
            hash TEXT NOT NULL,
            indexed_at TEXT NOT NULL DEFAULT (datetime('now')),
            modified_at TEXT NOT NULL
        );

        -- Chunks table: document split into embeddable pieces
        CREATE TABLE IF NOT EXISTS chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            chunk_index INTEGER NOT NULL,
            content TEXT NOT NULL,
            token_count INTEGER,
            has_embedding INTEGER NOT NULL DEFAULT 0,
            UNIQUE(document_id, chunk_index)
        );

        -- FTS5 virtual table for keyword search
        CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
            content,
            content=chunks,
            content_rowid=id,
            tokenize='porter unicode61'
        );

        -- Triggers to keep FTS5 in sync with chunks table
        CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
            INSERT INTO chunks_fts(rowid, content) VALUES (new.id, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON chunks BEGIN
            INSERT INTO chunks_fts(chunks_fts, rowid, content) VALUES('delete', old.id, old.content);
        END;

        CREATE TRIGGER IF NOT EXISTS chunks_au AFTER UPDATE ON chunks BEGIN
            INSERT INTO chunks_fts(chunks_fts, rowid, content) VALUES('delete', old.id, old.content);
            INSERT INTO chunks_fts(rowid, content) VALUES (new.id, new.content);
        END;

        -- Indexes for performance
        CREATE INDEX IF NOT EXISTS idx_documents_path ON documents(path);
        CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
        CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id);
        CREATE INDEX IF NOT EXISTS idx_chunks_has_embedding ON chunks(has_embedding);

        -- Enable WAL mode for concurrent reads
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        -- Performance PRAGMAs (2-5x speedup for reads/writes)
        -- synchronous=NORMAL is safe with WAL mode (data survives process crash, not power loss)
        PRAGMA synchronous=NORMAL;
        -- 16MB page cache (vs default 2MB) — keeps hot pages in memory
        PRAGMA cache_size=-16000;
        -- 256MB mmap for read-heavy workloads — OS page cache handles eviction
        PRAGMA mmap_size=268435456;
        -- Keep temp tables and indices in RAM (faster sorts, GROUP BY, etc.)
        PRAGMA temp_store=MEMORY;
        -- 5s busy timeout to avoid SQLITE_BUSY under concurrent access
        PRAGMA busy_timeout=5000;
        -- 4KB page size matches OS page size for optimal IO
        PRAGMA page_size=4096;
        ",
    )?;

    Ok(())
}

/// Initialize the sqlite-vec virtual table for vector search.
/// Must be called AFTER loading the sqlite-vec extension.
/// Dimensions default to 384 (all-MiniLM-L6-v2) for native engine,
/// or 768 (nomic-embed-text) for Ollama.
pub fn initialize_vec_table(conn: &Connection) -> Result<()> {
    initialize_vec_table_with_dims(conn, 384)
}

/// Initialize the sqlite-vec virtual table with specific dimensions.
///
/// Schema includes:
/// - `document_id INTEGER PARTITION KEY` — enables pre-filtering by document for 10x faster search
/// - `extension TEXT` — enables file-type filtering during KNN (e.g., only search PDFs)
/// - `embedding FLOAT[N]` — vector column for KNN distance computation
///
/// If the old schema (without partition keys) exists, it is dropped and recreated.
/// Re-embedding happens naturally via the `has_embedding` flag on chunks.
pub fn initialize_vec_table_with_dims(conn: &Connection, dimensions: usize) -> Result<()> {
    // Check if we need to migrate from old schema (no partition key)
    migrate_vec_table_if_needed(conn)?;

    let sql = format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_vec USING vec0(
            chunk_id INTEGER PRIMARY KEY,
            document_id INTEGER PARTITION KEY,
            extension TEXT,
            embedding FLOAT[{}]
        );",
        dimensions
    );
    conn.execute_batch(&sql)?;
    tracing::info!(
        "sqlite-vec chunks_vec table initialized ({}D, partition_key=document_id, metadata=extension)",
        dimensions
    );
    Ok(())
}

/// Migrate the vec table from old schema (no partition key) to new schema.
///
/// sqlite-vec virtual tables cannot be ALTERed, so we:
/// 1. Detect if the old table exists without partition key columns
/// 2. Drop it and let it be recreated with the new schema
/// 3. Reset `has_embedding` flags on all chunks to trigger re-embedding
fn migrate_vec_table_if_needed(conn: &Connection) -> Result<()> {
    // Check if chunks_vec exists at all
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='chunks_vec'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !table_exists {
        return Ok(());
    }

    // Try to detect if it's the old schema by checking if document_id column works.
    // Old schema: (chunk_id, embedding) — no document_id partition key.
    // If we can select document_id, the new schema is already in place.
    let has_partition_key = conn
        .execute_batch("SELECT document_id FROM chunks_vec LIMIT 0")
        .is_ok();

    if has_partition_key {
        tracing::debug!("chunks_vec already has partition key schema, no migration needed");
        return Ok(());
    }

    tracing::warn!(
        "Migrating chunks_vec: dropping old table (no partition key) → new schema. \
         Embeddings will be regenerated automatically."
    );

    // Drop old table
    conn.execute_batch("DROP TABLE IF EXISTS chunks_vec")?;

    // Reset has_embedding flags so indexer regenerates embeddings
    conn.execute_batch("UPDATE chunks SET has_embedding = 0")?;

    tracing::info!("chunks_vec migration complete. All chunks marked for re-embedding.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        // Verify tables exist
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='documents'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chunks'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
