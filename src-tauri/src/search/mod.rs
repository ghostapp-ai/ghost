pub mod ranking;

use crate::db::Database;
use crate::embeddings::EmbeddingEngine;
use crate::error::Result;

/// A search result combining document info with relevance score.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub chunk_id: i64,
    pub document_id: i64,
    pub path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub snippet: String,
    pub chunk_index: i32,
    pub score: f64,
    pub source: String, // "fts", "vector", or "hybrid"
}

/// Perform hybrid search combining FTS5 keyword search with vector KNN search.
/// Uses RRF (Reciprocal Rank Fusion) to merge results from both systems.
pub async fn hybrid_search(
    db: &Database,
    embedding_engine: &EmbeddingEngine,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    hybrid_search_filtered(db, embedding_engine, query, limit, None).await
}

/// Perform hybrid search with optional file extension filter.
///
/// When `extension_filter` is provided (e.g., "pdf"), vector search uses sqlite-vec
/// metadata filtering for up to 10x faster results by pre-filtering before distance computation.
pub async fn hybrid_search_filtered(
    db: &Database,
    embedding_engine: &EmbeddingEngine,
    query: &str,
    limit: usize,
    extension_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    // FTS5 keyword search
    let fts_results = db.fts_search(query, limit * 2)?;

    // Vector search (if sqlite-vec is available and embedding engine works)
    let vec_results = if db.is_vec_enabled() {
        match embedding_engine.embed(query).await {
            Ok(query_embedding) => {
                db.vec_search_filtered(&query_embedding, limit * 2, extension_filter)?
            }
            Err(e) => {
                tracing::debug!("Vector search unavailable: {} — using FTS5 only", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    // Combine with Reciprocal Rank Fusion
    let ranked = ranking::reciprocal_rank_fusion(&fts_results, &vec_results);

    let mut results = Vec::new();
    for ranked_item in ranked.iter().take(limit) {
        if let Some(chunk) = db.get_chunk_with_document(ranked_item.chunk_id)? {
            results.push(SearchResult {
                chunk_id: chunk.chunk_id,
                document_id: chunk.document_id,
                path: chunk.path,
                filename: chunk.filename,
                extension: chunk.extension,
                snippet: truncate_snippet(&chunk.content, 200),
                chunk_index: chunk.chunk_index,
                score: ranked_item.rrf_score,
                source: if ranked_item.vec_rank.is_some() && ranked_item.fts_rank.is_some() {
                    "hybrid".to_string()
                } else if ranked_item.vec_rank.is_some() {
                    "vector".to_string()
                } else {
                    "fts".to_string()
                },
            });
        }
    }

    Ok(results)
}

/// Truncate text to a maximum character length, ending at a word boundary.
/// Uses char_indices to avoid panicking on multi-byte UTF-8 boundaries.
fn truncate_snippet(text: &str, max_chars: usize) -> String {
    // Fast path: if total chars fit, return as-is
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    // Find the byte offset of the max_chars-th character (safe for UTF-8)
    let byte_end = text
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    let truncated = &text[..byte_end];

    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &text[..last_space])
    } else {
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_snippet() {
        let short = "hello world";
        assert_eq!(truncate_snippet(short, 200), "hello world");

        let long = "a ".repeat(200);
        let result = truncate_snippet(&long, 50);
        assert!(result.len() <= 53); // 50 + "..."
        assert!(result.ends_with("..."));
    }

    #[tokio::test]
    async fn test_hybrid_search() {
        crate::ensure_tls_provider();
        let db = Database::open_in_memory().unwrap();
        let embedding_engine = EmbeddingEngine::initialize().await;

        let doc_id = db
            .upsert_document(
                "/test/doc.txt",
                "doc.txt",
                Some("txt"),
                100,
                "hash1",
                "2026-01-01T00:00:00Z",
            )
            .unwrap();

        db.insert_chunk(doc_id, 0, "rust programming language systems", 4)
            .unwrap();
        db.insert_chunk(doc_id, 1, "python scripting language data science", 5)
            .unwrap();

        let results = hybrid_search(&db, &embedding_engine, "rust programming", 10)
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].filename, "doc.txt");
    }
}
