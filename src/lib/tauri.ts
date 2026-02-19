import { invoke } from "@tauri-apps/api/core";
import type { SearchResult, DbStats, IndexStats, AiStatus } from "./types";

/** Perform hybrid search (FTS5 + vector) across indexed documents. */
export async function search(
  query: string,
  limit?: number
): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_query", { query, limit });
}

/** Index all supported files in a directory recursively. */
export async function indexDirectory(path: string): Promise<IndexStats> {
  return invoke<IndexStats>("index_directory", { path });
}

/** Index a single file. */
export async function indexFile(path: string): Promise<void> {
  return invoke<void>("index_file", { path });
}

/** Get database statistics (document/chunk counts). */
export async function getStats(): Promise<DbStats> {
  return invoke<DbStats>("get_stats");
}

/** Check if Ollama is running and reachable. */
export async function checkOllama(): Promise<boolean> {
  return invoke<boolean>("check_ollama");
}

/** Get AI engine status (backend, model, hardware). */
export async function checkAiStatus(): Promise<AiStatus> {
  return invoke<AiStatus>("check_ai_status");
}

/** Start watching directories for file changes. */
export async function startWatcher(directories: string[]): Promise<void> {
  return invoke<void>("start_watcher", { directories });
}

/** Check if vector search (sqlite-vec) is available. */
export async function getVecStatus(): Promise<boolean> {
  return invoke<boolean>("get_vec_status");
}
