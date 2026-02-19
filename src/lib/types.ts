/** Search result from the Rust backend. */
export interface SearchResult {
  chunk_id: number;
  document_id: number;
  path: string;
  filename: string;
  extension: string | null;
  snippet: string;
  chunk_index: number;
  score: number;
  source: "fts" | "vector" | "hybrid";
}

/** Database statistics. */
export interface DbStats {
  document_count: number;
  chunk_count: number;
  embedded_chunk_count: number;
}

/** Indexing result statistics. */
export interface IndexStats {
  total: number;
  indexed: number;
  failed: number;
}

/** Application health status. */
export interface AppStatus {
  ollamaConnected: boolean;
  vecEnabled: boolean;
  stats: DbStats;
}
