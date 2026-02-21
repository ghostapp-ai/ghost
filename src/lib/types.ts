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

// --- MCP Catalog Types (App Store) ---

/** A single entry in the curated MCP tool catalog. */
export interface CatalogEntry {
  id: string;
  name: string;
  description: string;
  category: string;
  icon: string;
  runtime: "node" | "python" | "binary";
  transport: string;
  command: string;
  args: string[];
  is_mcp_app: boolean;
  required_env: EnvVarSpec[];
  tags: string[];
  popularity: number;
  official: boolean;
  package: string | null;
  repository: string | null;
}

/** Specification for a required environment variable. */
export interface EnvVarSpec {
  name: string;
  label: string;
  description: string;
  sensitive: boolean;
  placeholder: string | null;
  required: boolean;
}

/** Category in the MCP catalog. */
export interface CatalogCategory {
  id: string;
  name: string;
  icon: string;
}

/** Available runtimes on the user's system. */
export interface RuntimeInfo {
  has_node: boolean;
  node_version: string | null;
  has_npx: boolean;
  has_python: boolean;
  python_version: string | null;
  has_uv: boolean;
  has_uvx: boolean;
}

/** Response from get_mcp_catalog. */
export interface CatalogResponse {
  entries: CatalogEntry[];
  categories: CatalogCategory[];
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
  /** A2UI surfaces received via CUSTOM events. */
  a2uiSurfaces: Map<string, A2uiSurfaceState>;
}

// --- A2UI Protocol Types (Google A2UI v0.9) ---

/** Top-level A2UI message envelope. */
export interface A2uiMessage {
  version: string;
  createSurface?: A2uiCreateSurface;
  updateComponents?: A2uiUpdateComponents;
  updateDataModel?: A2uiUpdateDataModel;
  deleteSurface?: A2uiDeleteSurface;
}

/** createSurface — initialize a new UI surface. */
export interface A2uiCreateSurface {
  surfaceId: string;
  catalogId: string;
  theme?: A2uiTheme;
  sendDataModel?: boolean;
}

/** updateComponents — provide/update component definitions. */
export interface A2uiUpdateComponents {
  surfaceId: string;
  components: A2uiComponent[];
}

/** updateDataModel — insert or replace data. */
export interface A2uiUpdateDataModel {
  surfaceId: string;
  path?: string;
  value?: unknown;
}

/** deleteSurface — remove a surface. */
export interface A2uiDeleteSurface {
  surfaceId: string;
}

/** A2UI theme properties. */
export interface A2uiTheme {
  primaryColor?: string;
  iconUrl?: string;
  agentDisplayName?: string;
}

/** A single A2UI component definition (adjacency list model). */
export interface A2uiComponent {
  id: string;
  component: string;
  child?: string;
  children?: string[] | { path: string; componentId: string };
  text?: string | { path: string } | { call: string; args: unknown };
  label?: string | { path: string };
  variant?: string;
  value?: string | number | boolean | { path: string };
  action?: A2uiAction;
  url?: string | { path: string };
  name?: string;
  align?: string;
  justify?: string;
  weight?: number;
  axis?: string;
  options?: { label: string; value: string }[];
  min?: number;
  max?: number;
  step?: number;
  checks?: unknown[];
  [key: string]: unknown;
}

/** A2UI action definition. */
export interface A2uiAction {
  event?: { name: string; context?: unknown };
  functionCall?: unknown;
}

/** Runtime surface state — tracks components + data model. */
export interface A2uiSurfaceState {
  surfaceId: string;
  catalogId: string;
  theme?: A2uiTheme;
  sendDataModel?: boolean;
  components: Map<string, A2uiComponent>;
  dataModel: Record<string, unknown>;
  /** Root component IDs (components not referenced as children). */
  rootIds: string[];
}

// --- Agent Types ---

/** A conversation with metadata. */
export interface Conversation {
  id: number;
  title: string;
  created_at: string;
  updated_at: string;
  message_count: number;
  summary: string | null;
}

/** A single message in a conversation. */
export interface AgentMessage {
  id: number;
  conversation_id: number;
  role: string;
  content: string;
  created_at: string;
  tool_calls: string | null;
  tool_result: string | null;
  model: string | null;
}

/** Agent-specific configuration. */
export interface AgentConfig {
  agent_model: string;
  max_iterations: number;
  max_tokens: number;
  context_window: number;
  temperature: number;
  auto_approve_safe: boolean;
  skills_dir: string;
}

/** An agent model tier with hardware requirements. */
export interface AgentModelTier {
  model_id: string;
  name: string;
  min_ram_mb: number;
  recommended_ctx: number;
  tool_calling_reliable: boolean;
  quality: number;
  approx_usage_mb: number;
  is_recommended: boolean;
}

/** Response from get_agent_model_tiers command. */
export interface AgentModelTiersResponse {
  tiers: AgentModelTier[];
  recommended_model: string;
  recommended_ctx: number;
  available_ram_mb: number;
}

/** A loaded skill definition. */
export interface Skill {
  name: string;
  description: string;
  triggers: string[];
  instructions: string;
  source: string;
  enabled: boolean;
  tools: SkillTool[];
}

/** A tool defined within a skill. */
export interface SkillTool {
  name: string;
  description: string;
  parameters: unknown;
}
