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
pub fn initialize_vec_table_with_dims(conn: &Connection, dimensions: usize) -> Result<()> {
    let sql = format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_vec USING vec0(
            chunk_id INTEGER PRIMARY KEY,
            embedding FLOAT[{}]
        );",
        dimensions
    );
    conn.execute_batch(&sql)?;
    tracing::info!(
        "sqlite-vec chunks_vec table initialized ({}D)",
        dimensions
    );
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
