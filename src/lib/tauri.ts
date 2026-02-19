import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import type {
  SearchResult,
  DbStats,
  IndexStats,
  AiStatus,
  Settings,
  ChatMessage,
  ChatResponse,
  ChatStatus,
  LogEntry,
  HardwareInfo,
  ModelInfo,
} from "./types";

// --- Search & Indexing ---

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

// --- Window ---

/** Hide the main window. */
export async function hideWindow(): Promise<void> {
  return invoke<void>("hide_window");
}

/** Show the main window and focus it. */
export async function showWindow(): Promise<void> {
  return invoke<void>("show_window");
}

/** Programmatic window drag — fallback for data-tauri-drag-region issues. */
export async function startDragging(): Promise<void> {
  return invoke<void>("start_dragging");
}

// --- Auto-Indexing ---

/** Get auto-detected default user directories for indexing. */
export async function getDefaultDirectories(): Promise<string[]> {
  return invoke<string[]>("get_default_directories");
}

// --- Chat ---

/** Send chat messages and get a response. */
export async function chatSend(
  messages: ChatMessage[],
  maxTokens?: number
): Promise<ChatResponse> {
  return invoke<ChatResponse>("chat_send", { messages, maxTokens });
}

/** Get chat engine status. */
export async function chatStatus(): Promise<ChatStatus> {
  return invoke<ChatStatus>("chat_status");
}

/** Trigger background model loading. */
export async function chatLoadModel(): Promise<void> {
  return invoke<void>("chat_load_model");
}

/** Switch to a different chat model. */
export async function chatSwitchModel(modelId: string): Promise<void> {
  return invoke<void>("chat_switch_model", { modelId });
}

// --- Hardware & Models ---

/** Get detected hardware info. */
export async function getHardwareInfo(): Promise<HardwareInfo> {
  return invoke<HardwareInfo>("get_hardware_info");
}

/** Get available models with runtime status. */
export async function getAvailableModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>("get_available_models");
}

/** Get the recommended model ID for this hardware. */
export async function getRecommendedModel(): Promise<string> {
  return invoke<string>("get_recommended_model");
}

// --- Debug ---

/** Get log entries from the backend. */
export async function getLogs(sinceIndex?: number): Promise<LogEntry[]> {
  return invoke<LogEntry[]>("get_logs", { sinceIndex });
}

/** Clear the backend log buffer. */
export async function clearLogs(): Promise<void> {
  return invoke<void>("clear_logs");
}

// --- Settings ---

/** Load persisted settings. */
export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

/** Save settings to disk. */
export async function saveSettings(newSettings: Settings): Promise<void> {
  return invoke<void>("save_settings", { newSettings });
}

// --- System ---

/** Open a file with the system default application. */
export async function openFile(path: string): Promise<void> {
  return openPath(path);
}
