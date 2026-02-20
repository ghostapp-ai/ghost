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
  FsEntry,
  McpServerStatus,
  McpServerEntry,
  ConnectedServer,
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

/** Mark initial setup/onboarding as complete. */
export async function completeSetup(): Promise<void> {
  return invoke<void>("complete_setup");
}

// --- System ---

/** Open a file with the system default application. */
export async function openFile(path: string): Promise<void> {
  return openPath(path);
}

// --- Pro Edition ---

/** Check if this build includes Ghost Pro features. */
export async function isPro(): Promise<boolean> {
  return invoke<boolean>("is_pro");
}

// --- Filesystem Browsing ---

/** List contents of a directory for the file browser. */
export async function listDirectory(path: string): Promise<FsEntry[]> {
  return invoke<FsEntry[]>("list_directory", { path });
}

/** Get the user's home directory path. */
export async function getHomeDirectory(): Promise<string> {
  return invoke<string>("get_home_directory");
}

/** Get common root directories for filesystem browsing. */
export async function getRootDirectories(): Promise<FsEntry[]> {
  return invoke<FsEntry[]>("get_root_directories");
}

/** Add a directory to watched directories and start indexing. */
export async function addWatchDirectory(path: string): Promise<void> {
  return invoke<void>("add_watch_directory", { path });
}

/** Remove a directory from watched directories. */
export async function removeWatchDirectory(path: string): Promise<void> {
  return invoke<void>("remove_watch_directory", { path });
}

// --- MCP Protocol ---

/** Get MCP server status (enabled, host, port, url). */
export async function getMcpServerStatus(): Promise<McpServerStatus> {
  return invoke<McpServerStatus>("get_mcp_server_status");
}

/** List all configured external MCP servers and their connection status. */
export async function listMcpServers(): Promise<ConnectedServer[]> {
  return invoke<ConnectedServer[]>("list_mcp_servers");
}

/** Connect to an external MCP server by name. */
export async function connectMcpServer(name: string): Promise<ConnectedServer> {
  return invoke<ConnectedServer>("connect_mcp_server", { name });
}

/** Disconnect from an external MCP server. */
export async function disconnectMcpServer(name: string): Promise<void> {
  return invoke<void>("disconnect_mcp_server", { name });
}

/** Call a tool on a connected external MCP server. */
export async function callMcpTool(
  serverName: string,
  toolName: string,
  toolArguments?: Record<string, unknown>
): Promise<string> {
  return invoke<string>("call_mcp_tool", { serverName, toolName, arguments: toolArguments });
}

/** Get all available tools from all connected MCP servers. */
export async function listMcpTools(): Promise<
  Array<{ server: string; name: string; description: string | null }>
> {
  return invoke("list_mcp_tools");
}

/** Add a new MCP server entry to settings. */
export async function addMcpServerEntry(entry: McpServerEntry): Promise<void> {
  return invoke<void>("add_mcp_server_entry", { entry });
}

/** Remove an MCP server entry from settings. */
export async function removeMcpServerEntry(name: string): Promise<void> {
  return invoke<void>("remove_mcp_server_entry", { name });
}

// --- Platform Detection ---

/** Platform information from the Rust backend. */
export interface PlatformInfo {
  platform: "android" | "ios" | "macos" | "windows" | "linux" | "unknown";
  is_desktop: boolean;
  is_mobile: boolean;
  has_file_watcher: boolean;
  has_system_tray: boolean;
  has_global_shortcuts: boolean;
  has_stdio_mcp: boolean;
}

/** Get current platform info for UI adaptation. */
export async function getPlatformInfo(): Promise<PlatformInfo> {
  return invoke<PlatformInfo>("get_platform_info");
}

// --- AG-UI Streaming Chat ---

/** Start a streaming chat using the AG-UI event protocol.
 *  Returns the run_id. Listen for AG-UI events via `useAgui` hook. */
export async function chatSendStreaming(
  messages: ChatMessage[],
  maxTokens?: number
): Promise<string> {
  return invoke<string>("chat_send_streaming", { messages, maxTokens });
}
