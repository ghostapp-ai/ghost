pub mod schema;

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::{GhostError, Result};

/// Thread-safe database wrapper.
pub struct Database {
    conn: Mutex<Connection>,
    /// Whether sqlite-vec extension was loaded successfully.
    vec_enabled: bool,
}

impl Database {
    /// Register sqlite-vec auto-extension (must be called before opening connections).
    fn register_vec_extension() {
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
                *const (),
                unsafe extern "C" fn(
                    *mut rusqlite::ffi::sqlite3,
                    *mut *mut std::ffi::c_char,
                    *const rusqlite::ffi::sqlite3_api_routines,
                ) -> std::ffi::c_int,
            >(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }
    }

    /// Open or create the ghost vault database.
    pub fn open(path: &PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Register sqlite-vec BEFORE opening the connection
        Self::register_vec_extension();

        let conn = Connection::open(path)?;
        schema::initialize_schema(&conn)?;

        // Test if sqlite-vec loaded correctly
        let vec_enabled = Self::try_load_vec(&conn);

        Ok(Self {
            conn: Mutex::new(conn),
            vec_enabled,
        })
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        // Register sqlite-vec BEFORE opening the connection
        Self::register_vec_extension();

        let conn = Connection::open_in_memory()?;
        schema::initialize_schema(&conn)?;

        let vec_enabled = Self::try_load_vec(&conn);

        Ok(Self {
            conn: Mutex::new(conn),
            vec_enabled,
        })
    }

    /// Test if sqlite-vec is working and initialize vector tables.
    fn try_load_vec(conn: &Connection) -> bool {
        // Test if sqlite-vec is working
        match conn.query_row("SELECT vec_version()", [], |row| row.get::<_, String>(0)) {
            Ok(version) => {
                tracing::info!("sqlite-vec {} loaded successfully", version);
                if let Err(e) = schema::initialize_vec_table(conn) {
                    tracing::warn!("Failed to create chunks_vec table: {}", e);
                    return false;
                }
                true
            }
            Err(e) => {
                tracing::warn!("sqlite-vec not available: {} — vector search disabled", e);
                false
            }
        }
    }

    /// Check if vector search is available.
    pub fn is_vec_enabled(&self) -> bool {
        self.vec_enabled
    }

    /// Execute a closure with access to the database connection.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.conn.lock().map_err(|e| {
            GhostError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(format!("Lock poisoned: {}", e)),
            ))
        })?;
        f(&conn)
    }

    /// Insert or update a document in the database. Returns the document ID.
    pub fn upsert_document(
        &self,
        path: &str,
        filename: &str,
        extension: Option<&str>,
        size_bytes: i64,
        hash: &str,
        modified_at: &str,
    ) -> Result<i64> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO documents (path, filename, extension, size_bytes, hash, modified_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(path) DO UPDATE SET
                    filename = excluded.filename,
                    extension = excluded.extension,
                    size_bytes = excluded.size_bytes,
                    hash = excluded.hash,
                    modified_at = excluded.modified_at,
                    indexed_at = datetime('now')",
                rusqlite::params![path, filename, extension, size_bytes, hash, modified_at],
            )?;
            Ok(conn.last_insert_rowid())
        })
    }

    /// Insert a chunk for a document.
    pub fn insert_chunk(
        &self,
        document_id: i64,
        chunk_index: i32,
        content: &str,
        token_count: i32,
    ) -> Result<i64> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO chunks (document_id, chunk_index, content, token_count)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![document_id, chunk_index, content, token_count],
            )?;
            Ok(conn.last_insert_rowid())
        })
    }

    /// Delete all chunks for a document.
    pub fn delete_chunks_for_document(&self, document_id: i64) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM chunks WHERE document_id = ?1",
                rusqlite::params![document_id],
            )?;
            Ok(())
        })
    }

    /// Mark a chunk as having an embedding.
    pub fn mark_chunk_embedded(&self, chunk_id: i64) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE chunks SET has_embedding = 1 WHERE id = ?1",
                rusqlite::params![chunk_id],
            )?;
            Ok(())
        })
    }

    /// Get chunks that don't have embeddings yet.
    pub fn get_unembedded_chunks(&self, limit: usize) -> Result<Vec<(i64, String)>> {
        self.with_conn(|conn| {
            let mut stmt =
                conn.prepare("SELECT id, content FROM chunks WHERE has_embedding = 0 LIMIT ?1")?;
            let rows = stmt.query_map(rusqlite::params![limit], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    /// FTS5 keyword search. Returns (chunk_id, rank) pairs.
    pub fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(i64, f64)>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT rowid, rank FROM chunks_fts WHERE chunks_fts MATCH ?1 ORDER BY rank LIMIT ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![query, limit], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    /// Get chunk details by ID.
    pub fn get_chunk_with_document(&self, chunk_id: i64) -> Result<Option<ChunkWithDocument>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT c.id, c.content, c.chunk_index, d.id, d.path, d.filename, d.extension
                 FROM chunks c
                 JOIN documents d ON c.document_id = d.id
                 WHERE c.id = ?1",
            )?;
            let result = stmt
                .query_row(rusqlite::params![chunk_id], |row| {
                    Ok(ChunkWithDocument {
                        chunk_id: row.get(0)?,
                        content: row.get(1)?,
                        chunk_index: row.get(2)?,
                        document_id: row.get(3)?,
                        path: row.get(4)?,
                        filename: row.get(5)?,
                        extension: row.get(6)?,
                    })
                })
                .ok();
            Ok(result)
        })
    }

    /// Get document by path, returns (id, hash) if found.
    pub fn get_document_by_path(&self, path: &str) -> Result<Option<(i64, String)>> {
        self.with_conn(|conn| {
            let result = conn
                .query_row(
                    "SELECT id, hash FROM documents WHERE path = ?1",
                    rusqlite::params![path],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
                )
                .ok();
            Ok(result)
        })
    }

    /// Get total document and chunk counts.
    pub fn get_stats(&self) -> Result<DbStats> {
        self.with_conn(|conn| {
            let doc_count: i64 =
                conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
            let chunk_count: i64 =
                conn.query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;
            let embedded_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM chunks WHERE has_embedding = 1",
                [],
                |row| row.get(0),
            )?;
            Ok(DbStats {
                document_count: doc_count,
                chunk_count,
                embedded_chunk_count: embedded_count,
            })
        })
    }

    /// Get recently indexed documents, ordered by indexed_at descending.
    pub fn get_recent_documents(&self, limit: usize) -> Result<Vec<RecentDocument>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT path, filename, extension, size_bytes, indexed_at \
                 FROM documents ORDER BY indexed_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map(rusqlite::params![limit], |row| {
                Ok(RecentDocument {
                    path: row.get(0)?,
                    filename: row.get(1)?,
                    extension: row.get(2)?,
                    size_bytes: row.get(3)?,
                    indexed_at: row.get(4)?,
                })
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    // --- Vector operations (sqlite-vec) ---

    /// Insert an embedding vector for a chunk.
    pub fn insert_embedding(&self, chunk_id: i64, embedding: &[f32]) -> Result<()> {
        if !self.vec_enabled {
            return Err(GhostError::Search("sqlite-vec not loaded".into()));
        }
        self.with_conn(|conn| {
            let blob = embedding
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<u8>>();
            conn.execute(
                "INSERT OR REPLACE INTO chunks_vec(chunk_id, embedding) VALUES (?1, ?2)",
                rusqlite::params![chunk_id, blob],
            )?;
            Ok(())
        })
    }

    /// Delete all embeddings for chunks belonging to a document.
    pub fn delete_embeddings_for_document(&self, document_id: i64) -> Result<()> {
        if !self.vec_enabled {
            return Ok(());
        }
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM chunks_vec WHERE chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)",
                rusqlite::params![document_id],
            )?;
            Ok(())
        })
    }

    /// KNN vector search. Returns (chunk_id, distance) pairs, ordered by distance ascending.
    pub fn vec_search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(i64, f64)>> {
        if !self.vec_enabled {
            return Ok(vec![]);
        }
        self.with_conn(|conn| {
            let blob = query_embedding
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<u8>>();
            let mut stmt = conn.prepare(
                "SELECT chunk_id, distance FROM chunks_vec WHERE embedding MATCH ?1 ORDER BY distance LIMIT ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![blob, limit], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChunkWithDocument {
    pub chunk_id: i64,
    pub content: String,
    pub chunk_index: i32,
    pub document_id: i64,
    pub path: String,
    pub filename: String,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DbStats {
    pub document_count: i64,
    pub chunk_count: i64,
    pub embedded_chunk_count: i64,
}

/// A recently indexed document.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentDocument {
    pub path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub indexed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_and_search() {
        let db = Database::open_in_memory().unwrap();

        let doc_id = db
            .upsert_document(
                "/test/file.txt",
                "file.txt",
                Some("txt"),
                1024,
                "abc123",
                "2026-02-18T00:00:00Z",
            )
            .unwrap();

        db.insert_chunk(doc_id, 0, "hello world this is a test document", 7)
            .unwrap();

        let results = db.fts_search("hello world", 10).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_stats() {
        let db = Database::open_in_memory().unwrap();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.document_count, 0);
    }
}
