/// Reciprocal Rank Fusion (RRF) for combining multiple ranked lists.
///
/// RRF score = sum(1 / (k + rank_i)) for each ranking system.
/// k = 60 is the standard constant from the original paper.
const RRF_K: f64 = 60.0;

/// A search result with scores from different ranking systems.
#[derive(Debug, Clone)]
pub struct RankedResult {
    pub chunk_id: i64,
    pub fts_rank: Option<usize>,
    pub vec_rank: Option<usize>,
    pub rrf_score: f64,
}

/// Combine FTS5 and vector search results using RRF.
///
/// - `fts_results`: chunk IDs from FTS5, ordered by relevance (best first)
/// - `vec_results`: chunk IDs from vector search, ordered by distance (closest first)
pub fn reciprocal_rank_fusion(
    fts_results: &[(i64, f64)],
    vec_results: &[(i64, f64)],
) -> Vec<RankedResult> {
    use std::collections::HashMap;

    let mut scores: HashMap<i64, RankedResult> = HashMap::new();

    // Process FTS5 results
    for (rank, (chunk_id, _score)) in fts_results.iter().enumerate() {
        let entry = scores.entry(*chunk_id).or_insert(RankedResult {
            chunk_id: *chunk_id,
            fts_rank: None,
            vec_rank: None,
            rrf_score: 0.0,
        });
        entry.fts_rank = Some(rank + 1);
        entry.rrf_score += 1.0 / (RRF_K + (rank + 1) as f64);
    }

    // Process vector results
    for (rank, (chunk_id, _distance)) in vec_results.iter().enumerate() {
        let entry = scores.entry(*chunk_id).or_insert(RankedResult {
            chunk_id: *chunk_id,
            fts_rank: None,
            vec_rank: None,
            rrf_score: 0.0,
        });
        entry.vec_rank = Some(rank + 1);
        entry.rrf_score += 1.0 / (RRF_K + (rank + 1) as f64);
    }

    // Sort by RRF score descending (NaN-safe — treat NaN as lowest)
    let mut results: Vec<RankedResult> = scores.into_values().collect();
    results.sort_by(|a, b| {
        b.rrf_score
            .partial_cmp(&a.rrf_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_both_systems() {
        let fts = vec![(1, -1.0), (2, -2.0), (3, -3.0)];
        let vec = vec![(2, 0.1), (1, 0.2), (4, 0.3)];

        let results = reciprocal_rank_fusion(&fts, &vec);

        // Chunk 1 and 2 appear in both systems, should rank highest
        assert!(results[0].chunk_id == 1 || results[0].chunk_id == 2);
        assert!(results[0].rrf_score > results.last().unwrap().rrf_score);
    }

    #[test]
    fn test_rrf_fts_only() {
        let fts = vec![(1, -1.0), (2, -2.0)];
        let vec: Vec<(i64, f64)> = vec![];

        let results = reciprocal_rank_fusion(&fts, &vec);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].chunk_id, 1); // Higher FTS rank
    }

    #[test]
    fn test_rrf_empty() {
        let results = reciprocal_rank_fusion(&[], &[]);
        assert!(results.is_empty());
    }
}
