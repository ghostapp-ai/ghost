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

/** Persistent settings stored on disk. */
export interface Settings {
  watched_directories: string[];
  shortcut: string;
  chat_model: string;
  chat_device: string;
  chat_max_tokens: number;
  chat_temperature: number;
  setup_complete: boolean;
  launch_on_startup: boolean;
}

/** Hardware info from the Rust backend. */
export interface HardwareInfo {
  cpu_cores: number;
  has_avx2: boolean;
  has_neon: boolean;
  gpu_backend: "Cuda" | "Metal" | "Vulkan" | null;
  total_ram_mb: number;
  available_ram_mb: number;
}

/** AI engine status from the Rust backend. */
export interface AiStatus {
  backend: "Native" | "Ollama" | "None";
  model_name: string;
  dimensions: number;
  hardware: HardwareInfo;
}

/** Chat message (user, assistant, or system). */
export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

/** Chat response from the Rust backend. */
export interface ChatResponse {
  content: string;
  tokens_generated: number;
  duration_ms: number;
  model_id: string;
}

/** Download progress information. */
export interface DownloadProgress {
  downloaded_bytes: number;
  total_bytes: number;
  phase: "checking_cache" | "downloading" | "download_complete" | "loading_model" | "cached";
}

/** Chat engine status. */
export interface ChatStatus {
  available: boolean;
  backend: "native" | "ollama" | "loading" | "none";
  model_id: string;
  model_name: string;
  loading: boolean;
  error: string | null;
  device: string;
  download_progress: DownloadProgress | null;
}

/** A structured log entry from the Rust backend. */
export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

/** Model info with runtime status (downloaded, active, recommended). */
export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  size_mb: number;
  min_ram_mb: number;
  parameters: string;
  quality_tier: number;
  downloaded: boolean;
  active: boolean;
  recommended: boolean;
  fits_hardware: boolean;
}

/** Filesystem entry for the file browser. */
export interface FsEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size_bytes: number;
  modified: string;
  extension: string | null;
  is_cloud_placeholder: boolean;
  is_local: boolean;
}

// --- MCP Protocol Types ---

/** MCP Server status. */
export interface McpServerStatus {
  enabled: boolean;
  host: string;
  port: number;
  url: string;
}

/** Configuration for an external MCP server entry. */
export interface McpServerEntry {
  name: string;
  transport: string;
  command: string | null;
  args: string[];
  url: string | null;
  enabled: boolean;
  env: Record<string, string>;
}

/** A connected external MCP server with its tools. */
export interface ConnectedServer {
  name: string;
  connected: boolean;
  tools: McpToolInfo[];
  transport: string;
  error: string | null;
}

/** Information about a single MCP tool. */
export interface McpToolInfo {
  name: string;
  description: string | null;
  input_schema: unknown | null;
}

// --- AG-UI Protocol Types ---

/** AG-UI event type discriminator. */
export type AgUiEventType =
  | "RUN_STARTED"
  | "RUN_FINISHED"
  | "RUN_ERROR"
  | "STEP_STARTED"
  | "STEP_FINISHED"
  | "TEXT_MESSAGE_START"
  | "TEXT_MESSAGE_CONTENT"
  | "TEXT_MESSAGE_END"
  | "TOOL_CALL_START"
  | "TOOL_CALL_ARGS"
  | "TOOL_CALL_END"
  | "STATE_SNAPSHOT"
  | "STATE_DELTA"
  | "RAW"
  | "CUSTOM";

/** Base AG-UI event from the Rust backend. */
export interface AgUiEvent {
  type: AgUiEventType;
  runId: string;
  threadId?: string;
  timestamp: number;
  // Flattened payload fields (varies by event type):
  // TEXT_MESSAGE_START / CONTENT / END
  messageId?: string;
  role?: string;
  delta?: string;
  // TOOL_CALL_START / ARGS / END
  toolCallId?: string;
  toolCallName?: string;
  parentMessageId?: string;
  result?: string;
  // STEP_STARTED / FINISHED
  stepName?: string;
  stepIndex?: number;
  // RUN_ERROR
  message?: string;
  code?: string;
  // STATE_SNAPSHOT / DELTA
  snapshot?: unknown;
  // CUSTOM
  name?: string;
  value?: unknown;
}

/** State of a streaming AG-UI run. */
export interface AgUiRunState {
  runId: string;
  status: "running" | "finished" | "error";
  /** Accumulated response text from TEXT_MESSAGE_CONTENT deltas. */
  content: string;
  /** Current step name (from STEP_STARTED). */
  currentStep: string | null;
  /** Active tool calls in progress. */
  activeToolCalls: Map<string, { name: string; args: string; result?: string }>;
  /** Error message if status is "error". */
  error: string | null;
  /** Generation metadata from CUSTOM event. */
  metadata: Record<string, unknown> | null;
}
