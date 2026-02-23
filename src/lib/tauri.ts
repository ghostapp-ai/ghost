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

/** Programmatic window drag — uses direct Tauri API (no IPC roundtrip).
 * Must be called synchronously on mousedown for the OS drag loop to take over.
 * Using invoke() caused focus toggle → minimize bugs on Windows (#10767). */
export async function startDragging(): Promise<void> {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow().startDragging();
}

/** Minimize the main window. */
export async function minimizeWindow(): Promise<void> {
  return invoke<void>("minimize_window");
}

/** Toggle maximize / restore the main window. */
export async function toggleMaximizeWindow(): Promise<void> {
  return invoke<void>("toggle_maximize_window");
}

/** Close the main window (exit the app). */
export async function closeWindow(): Promise<void> {
  return invoke<void>("close_window");
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

/** Push a log entry from the frontend into the backend log buffer.
 * This makes frontend errors visible in the DebugPanel alongside backend logs.
 * Fire-and-forget: errors in logging must not crash the app. */
export function logFromFrontend(level: "error" | "warn" | "info" | "debug", message: string): void {
  invoke<void>("log_from_frontend", { level, message }).catch(() => {
    // If backend logging fails (e.g. during startup), silently swallow the error
  });
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

// --- MCP Catalog (App Store) ---

import type { CatalogEntry, CatalogResponse, RuntimeInfo, RegistrySyncResult, RegistryStatus, PackageVerification, BootstrapStatus, RuntimeKind, RuntimeInstallResult, ToolRecommendation } from "./types";

/** Get the curated MCP tool catalog with categories. */
export async function getMcpCatalog(): Promise<CatalogResponse> {
  return invoke<CatalogResponse>("get_mcp_catalog");
}

/** Detect available runtimes on the system (Node.js, Python, etc.). */
export async function detectRuntimes(): Promise<RuntimeInfo> {
  return invoke<RuntimeInfo>("detect_runtimes");
}

/** Install an MCP server from the catalog with one click.
 *  Provide required environment variables as key-value pairs. */
export async function installMcpFromCatalog(
  catalogId: string,
  envVars: Record<string, string> = {}
): Promise<ConnectedServer> {
  return invoke<ConnectedServer>("install_mcp_from_catalog", { catalogId, envVars });
}

/** Uninstall an MCP server (disconnect + remove from settings). */
export async function uninstallMcpServer(name: string): Promise<void> {
  return invoke<void>("uninstall_mcp_server", { name });
}

// --- MCP Zero-Config & Auto-Provisioning ---

/** Get all catalog entries that require zero configuration (no env vars). */
export async function getZeroConfigTools(): Promise<CatalogEntry[]> {
  return invoke<CatalogEntry[]>("get_zero_config_tools");
}

/** Get the curated default MCP tools (filesystem, sequential-thinking, memory, fetch, everything). */
export async function getDefaultTools(): Promise<CatalogEntry[]> {
  return invoke<CatalogEntry[]>("get_default_tools");
}

/** Verify an MCP server package by spawning it and performing a JSON-RPC initialize handshake. */
export async function verifyMcpPackage(catalogId: string): Promise<PackageVerification> {
  return invoke<PackageVerification>("verify_mcp_package", { catalogId });
}

/** Auto-provision default MCP tools: detects runtimes, builds configs, saves to settings.
 *  Returns the number of tools provisioned. Skips tools whose runtimes are unavailable. */
export async function autoProvisionMcpDefaults(): Promise<number> {
  return invoke<number>("auto_provision_mcp_defaults");
}

// --- MCP Registry ---

/** Install an MCP server from a CatalogEntry directly (for registry entries).
 *  Unlike installMcpFromCatalog (which looks up by curated catalog ID),
 *  this accepts a full CatalogEntry — used for servers discovered from the registry. */
export async function installMcpEntry(
  entry: CatalogEntry,
  envVars: Record<string, string> = {}
): Promise<ConnectedServer> {
  return invoke<ConnectedServer>("install_mcp_entry", { entry, envVars });
}

/** Sync the official MCP Registry to local cache.
 *  Fetches all 6,000+ servers from registry.modelcontextprotocol.io.
 *  This is opt-in — only triggered when user explicitly browses the registry. */
export async function syncMcpRegistry(): Promise<RegistrySyncResult> {
  return invoke<RegistrySyncResult>("sync_mcp_registry");
}

/** Search the cached MCP Registry for servers matching a query.
 *  Requires a prior syncMcpRegistry() call to populate the cache. */
export async function searchMcpRegistry(
  query: string,
  limit?: number
): Promise<CatalogEntry[]> {
  return invoke<CatalogEntry[]>("search_mcp_registry", { query, limit });
}

/** Get the registry cache status (synced, fresh, metadata). */
export async function getRegistryStatus(): Promise<RegistryStatus> {
  return invoke<RegistryStatus>("get_registry_status");
}

// --- Runtime Bootstrap ---

/** Get the status of all managed runtimes (Node.js, uv/Python, Docker).
 *  Reports whether each is installed, managed by Ghost, version, and path. */
export async function getRuntimeBootstrapStatus(): Promise<BootstrapStatus> {
  return invoke<BootstrapStatus>("get_runtime_bootstrap_status");
}

/** Install a specific runtime managed by Ghost.
 *  Emits `runtime-install-progress` events during installation.
 *  @param kind - "node", "uv", or "docker" */
export async function installRuntime(kind: RuntimeKind): Promise<RuntimeInstallResult> {
  return invoke<RuntimeInstallResult>("install_runtime", { kind });
}

/** Bootstrap all missing runtimes needed for default MCP tools.
 *  Installs uv + Node.js if not present. Emits progress events. */
export async function bootstrapAllRuntimes(): Promise<RuntimeInstallResult[]> {
  return invoke<RuntimeInstallResult[]>("bootstrap_all_runtimes");
}

/** AI-powered tool recommendation: find MCP tools matching a natural language query. */
export async function recommendMcpTools(query: string): Promise<ToolRecommendation[]> {
  return invoke<ToolRecommendation[]>("recommend_mcp_tools", { query });
}

/** Check what a tool needs before it can be installed.
 *  Returns runtime requirements and whether they're met. */
export async function checkToolRequirements(catalogId: string): Promise<{
  id: string;
  name: string;
  runtime: string;
  runtime_installed: boolean;
  can_auto_install_runtime: boolean;
  required_env: string[];
  ready: boolean;
}> {
  return invoke("check_tool_requirements", { catalogId });
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

// --- Agent ---

import type {
  Conversation,
  AgentMessage,
  AgentConfig,
  AgentModelTiersResponse,
  Skill,
} from "./types";

/** Run the agent with ReAct loop + tool calling.
 *  Returns the run_id immediately. Listen for AG-UI events via `useAgui` hook. */
export async function agentChat(
  messages: ChatMessage[],
  conversationId?: number | null
): Promise<string> {
  return invoke<string>("agent_chat", { messages, conversationId });
}

/** Create a new conversation. Returns the conversation ID. */
export async function createConversation(title: string): Promise<number> {
  return invoke<number>("create_conversation", { title });
}

/** List all conversations, ordered by most recent. */
export async function listConversations(limit?: number): Promise<Conversation[]> {
  return invoke<Conversation[]>("list_conversations", { limit });
}

/** Get messages for a specific conversation. */
export async function getConversationMessages(
  conversationId: number,
  limit?: number
): Promise<AgentMessage[]> {
  return invoke<AgentMessage[]>("get_conversation_messages", { conversationId, limit });
}

/** Delete a conversation and all its messages. */
export async function deleteConversation(conversationId: number): Promise<void> {
  return invoke<void>("delete_conversation", { conversationId });
}

/** Update conversation title. */
export async function updateConversationTitle(
  conversationId: number,
  title: string
): Promise<void> {
  return invoke<void>("update_conversation_title", { conversationId, title });
}

/** Search across conversation memory via FTS5. */
export async function searchMemory(query: string, limit?: number): Promise<AgentMessage[]> {
  return invoke<AgentMessage[]>("search_memory", { query, limit });
}

/** Get current agent configuration. */
export async function getAgentConfig(): Promise<AgentConfig> {
  return invoke<AgentConfig>("get_agent_config");
}

/** Save agent configuration. */
export async function saveAgentConfig(config: AgentConfig): Promise<void> {
  return invoke<void>("save_agent_config", { config });
}

/** Get available agent model tiers and hardware recommendation. */
export async function getAgentModelTiers(): Promise<AgentModelTiersResponse> {
  return invoke<AgentModelTiersResponse>("get_agent_model_tiers");
}

/** List all loaded skills from the skills directory. */
export async function listSkills(): Promise<Skill[]> {
  return invoke<Skill[]>("list_skills");
}
