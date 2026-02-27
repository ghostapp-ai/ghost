pub mod chunker;
pub mod extractor;

/// File watcher — desktop only (requires `notify` crate with OS-level file events).
#[cfg(desktop)]
pub mod watcher;

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::db::Database;
use crate::embeddings::EmbeddingEngine;
use crate::error::{GhostError, Result};

/// Index a single file: extract text, chunk, store in DB, and optionally embed.
/// For cloud placeholder files (OneDrive, iCloud), only index metadata without
/// downloading the file content.
pub async fn index_file(
    db: &Database,
    embedding_engine: &EmbeddingEngine,
    path: &Path,
) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();

    // Read file metadata
    let metadata = std::fs::metadata(path).map_err(|e| {
        GhostError::Indexer(format!(
            "Cannot read metadata for {}: {}",
            path.display(),
            e
        ))
    })?;

    // Check if file is a cloud placeholder (OneDrive Files On-Demand)
    if is_cloud_placeholder(&metadata) {
        tracing::debug!(
            "Cloud placeholder (not downloaded), metadata-only index: {}",
            path.display()
        );
        return index_file_metadata_only(db, path, &metadata);
    }

    let size_bytes = metadata.len() as i64;
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| chrono_format_timestamp(d.as_secs()))
        })
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

    // Compute file hash for change detection
    let file_bytes = std::fs::read(path)?;
    let hash = hex::encode(Sha256::digest(&file_bytes));

    // Check if file already indexed with same hash
    if let Some((_, existing_hash)) = db.get_document_by_path(&path_str)? {
        if existing_hash == hash {
            tracing::debug!("File unchanged, skipping: {}", path.display());
            return Ok(());
        }
    }

    // Extract text
    let text = extractor::extract_text(path)?;
    if text.trim().is_empty() {
        tracing::warn!("No text extracted from: {}", path.display());
        return Ok(());
    }

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown");
    let extension = path.extension().and_then(|e| e.to_str());

    // Upsert document
    let doc_id = db.upsert_document(
        &path_str,
        filename,
        extension,
        size_bytes,
        &hash,
        &modified_at,
    )?;

    // Delete old chunks and embeddings, then re-chunk
    db.delete_embeddings_for_document(doc_id)?;
    db.delete_chunks_for_document(doc_id)?;

    let chunks = chunker::chunk_text_default(&text);
    tracing::info!(
        "Indexing {} ({} chunks): {}",
        filename,
        chunks.len(),
        path.display()
    );

    // Insert chunks in a single transaction (10-50x faster than individual inserts)
    db.with_transaction(|conn| {
        for chunk in &chunks {
            conn.execute(
                "INSERT OR REPLACE INTO chunks (document_id, chunk_index, content, token_count)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![doc_id, chunk.index, chunk.content, chunk.token_count],
            )?;
        }
        Ok(())
    })?;

    // Try to generate embeddings in batches (graceful degradation if engine is down)
    if embedding_engine.health_check().await.unwrap_or(false) {
        let unembedded = db.get_unembedded_chunks(chunks.len())?;
        if !unembedded.is_empty() {
            // Batch embed all chunks at once (2-5x faster with tensor batching)
            let texts: Vec<String> = unembedded.iter().map(|(_, c)| c.clone()).collect();
            match embedding_engine.embed_batch(&texts).await {
                Ok(embeddings) => {
                    // Store all embeddings in a single transaction
                    // Includes document_id (partition key) and extension (metadata)
                    // for 10x faster filtered vector search
                    let chunk_ids: Vec<i64> = unembedded.iter().map(|(id, _)| *id).collect();
                    db.with_transaction(|conn| {
                        for (i, embedding) in embeddings.iter().enumerate() {
                            let chunk_id = chunk_ids[i];
                            let blob = embedding
                                .iter()
                                .flat_map(|f| f.to_le_bytes())
                                .collect::<Vec<u8>>();
                            // Only mark has_embedding=1 if vec insert actually succeeds
                            match conn.execute(
                                "INSERT OR REPLACE INTO chunks_vec(chunk_id, document_id, extension, embedding) VALUES (?1, ?2, ?3, ?4)",
                                rusqlite::params![chunk_id, doc_id, extension, blob],
                            ) {
                                Ok(_) => {
                                    conn.execute(
                                        "UPDATE chunks SET has_embedding = 1 WHERE id = ?1",
                                        rusqlite::params![chunk_id],
                                    )?;
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to insert embedding for chunk {}: {}", chunk_id, e);
                                }
                            }
                        }
                        Ok(())
                    })?;
                    tracing::debug!(
                        "Batch embedded {} chunks for {}",
                        embeddings.len(),
                        filename
                    );
                }
                Err(e) => {
                    tracing::warn!("Batch embedding failed for {}: {}", filename, e);
                }
            }
        }
    } else {
        tracing::info!("No embedding engine available — skipping embeddings, FTS5 index created");
    }

    Ok(())
}

/// Index all supported files in a directory recursively.
pub async fn index_directory(
    db: &Database,
    embedding_engine: &EmbeddingEngine,
    dir: &Path,
) -> Result<IndexStats> {
    let mut stats = IndexStats::default();

    if !dir.exists() {
        return Err(GhostError::Indexer(format!(
            "Directory does not exist: {}",
            dir.display()
        )));
    }

    let entries = walk_directory(dir)?;

    for path in entries {
        match index_file(db, embedding_engine, &path).await {
            Ok(()) => stats.indexed += 1,
            Err(e) => {
                tracing::warn!("Failed to index {}: {}", path.display(), e);
                stats.failed += 1;
            }
        }
    }

    stats.total = stats.indexed + stats.failed;
    tracing::info!(
        "Indexing complete: {} indexed, {} failed, {} total",
        stats.indexed,
        stats.failed,
        stats.total
    );

    Ok(stats)
}

/// Walk a directory iteratively and collect all supported files.
/// Uses an explicit stack instead of recursion to avoid stack overflow on deep trees.
fn walk_directory(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        let entries = match std::fs::read_dir(&current_dir) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Cannot read directory {}: {}", current_dir.display(), e);
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!(
                        "Skipping unreadable entry in {}: {}",
                        current_dir.display(),
                        e
                    );
                    continue;
                }
            };
            let path = entry.path();

            // Skip hidden files/directories
            if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }

            if path.is_dir() {
                stack.push(path);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extractor::is_supported_extension(ext) {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

/// Format a UNIX timestamp (seconds since epoch) to ISO 8601 UTC string.
/// Uses the `chrono` crate (already a dependency) instead of manual date arithmetic.
fn chrono_format_timestamp(secs: u64) -> String {
    chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// Detect if a file is a cloud placeholder (OneDrive Files On-Demand, iCloud, etc.)
/// On Windows, checks FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS and related flags.
/// Cloud-only files should NOT be read (reading triggers a download).
#[cfg(target_os = "windows")]
fn is_cloud_placeholder(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    let attrs = metadata.file_attributes();
    const RECALL_ON_DATA: u32 = 0x00400000;
    const RECALL_ON_OPEN: u32 = 0x00040000;
    const OFFLINE: u32 = 0x00001000;
    (attrs & RECALL_ON_DATA) != 0 || (attrs & RECALL_ON_OPEN) != 0 || (attrs & OFFLINE) != 0
}

#[cfg(not(target_os = "windows"))]
fn is_cloud_placeholder(_metadata: &std::fs::Metadata) -> bool {
    false
}

/// Index only the file's metadata (name, path, extension, size) without reading content.
/// Used for cloud placeholder files that shouldn't be downloaded just for indexing.
fn index_file_metadata_only(
    db: &Database,
    path: &Path,
    metadata: &std::fs::Metadata,
) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown");
    let extension = path.extension().and_then(|e| e.to_str());
    let size_bytes = metadata.len() as i64;
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| chrono_format_timestamp(d.as_secs()))
        })
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

    // Use path as hash for cloud files (can't compute content hash without downloading)
    let hash = format!("cloud:{}", path_str);

    // Check if already indexed
    if let Some((_, existing_hash)) = db.get_document_by_path(&path_str)? {
        if existing_hash == hash {
            return Ok(());
        }
    }

    // Upsert document with just metadata
    let doc_id = db.upsert_document(
        &path_str,
        filename,
        extension,
        size_bytes,
        &hash,
        &modified_at,
    )?;

    // Create a single chunk with filename info for FTS5 search
    db.delete_chunks_for_document(doc_id)?;
    db.delete_embeddings_for_document(doc_id)?;

    let searchable_text = format!(
        "{} {} (cloud file — not downloaded)",
        filename,
        extension.unwrap_or("")
    );
    db.insert_chunk(doc_id, 0, &searchable_text, 0)?;

    tracing::debug!("Metadata-only index for cloud file: {}", path.display());
    Ok(())
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct IndexStats {
    pub total: usize,
    pub indexed: usize,
    pub failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_directory() {
        let dir = std::env::temp_dir().join("ghost_test_walk");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("test.txt"), "hello").unwrap();
        std::fs::write(dir.join("test.exe"), "binary").unwrap();
        std::fs::write(dir.join(".hidden"), "hidden").unwrap();

        let files = walk_directory(&dir).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("test.txt"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_chrono_format() {
        let ts = chrono_format_timestamp(0);
        assert_eq!(ts, "1970-01-01T00:00:00Z");
    }
}
