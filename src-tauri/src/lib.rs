mod agent;
mod chat;
mod db;
mod embeddings;
mod error;
mod indexer;
mod protocols;
mod search;
mod settings;

// Pro features (only compiled when `pro` feature flag is enabled)
#[cfg(feature = "pro")]
use ghost_pro;

use std::path::PathBuf;
use std::sync::Arc;

/// Install ring as the default TLS crypto provider (idempotent).
///
/// Must be called before any TLS connection (Ollama, MCP, HF Hub).
/// Uses ring instead of aws-lc-rs because aws-lc-sys fails on MSVC
/// (missing `__builtin_bswap*` GCC intrinsics).
pub(crate) fn ensure_tls_provider() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install ring as default TLS crypto provider");
    });
}

use db::Database;
use embeddings::hardware::HardwareInfo;
use embeddings::{AiStatus, EmbeddingEngine};
use search::SearchResult;
use settings::Settings;

// Desktop-only imports
#[cfg(desktop)]
use tauri::menu::{Menu, MenuItem};
#[cfg(desktop)]
use tauri::tray::TrayIconBuilder;
#[cfg(desktop)]
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use tauri::Emitter;
#[cfg(desktop)]
use tauri::Manager;

/// Application state shared across commands.
pub struct AppState {
    pub db: Database,
    pub embedding_engine: EmbeddingEngine,
    pub chat_engine: chat::ChatEngine,
    pub hardware: HardwareInfo,
    pub settings: std::sync::Mutex<Settings>,
    pub mcp_client: protocols::mcp_client::McpClientManager,
    pub agui_event_bus: protocols::agui::AgUiEventBus,
}

/// A structured log entry for the debug panel.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Thread-safe log collector for the frontend debug panel.
static LOG_BUFFER: std::sync::LazyLock<std::sync::Mutex<Vec<LogEntry>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));

/// Push a log entry into the global buffer.
fn push_log(level: &str, message: String) {
    if let Ok(mut logs) = LOG_BUFFER.lock() {
        logs.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            level: level.to_string(),
            message,
        });
        // Keep buffer bounded
        if logs.len() > 500 {
            logs.drain(0..100);
        }
    }
}

/// Get the app data directory.
fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.ghost.app")
}

/// Get the default vault database path.
fn get_db_path() -> PathBuf {
    get_app_data_dir().join("ghost_vault.db")
}

// --- Window Management ---

/// Toggle window visibility (show/hide). Desktop only — tray icon interaction.
#[cfg(desktop)]
fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[tauri::command]
async fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    #[cfg(not(desktop))]
    let _ = &app; // suppress unused warning
    Ok(())
}

#[tauri::command]
async fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    #[cfg(not(desktop))]
    let _ = &app; // suppress unused warning
    Ok(())
}

/// Programmatic window drag — fallback for data-tauri-drag-region issues on Linux.
/// No-op on mobile.
#[tauri::command]
async fn start_dragging(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window("main") {
        window.start_dragging().map_err(|e| e.to_string())?;
    }
    #[cfg(not(desktop))]
    let _ = &app; // suppress unused warning
    Ok(())
}

/// Minimize the main window.
#[tauri::command]
async fn minimize_window(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
    }
    #[cfg(not(desktop))]
    let _ = &app;
    Ok(())
}

/// Toggle maximize / restore the main window.
#[tauri::command]
async fn toggle_maximize_window(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window("main") {
        if window.is_maximized().unwrap_or(false) {
            window.unmaximize().map_err(|e| e.to_string())?;
        } else {
            window.maximize().map_err(|e| e.to_string())?;
        }
    }
    #[cfg(not(desktop))]
    let _ = &app;
    Ok(())
}

/// Close the main window (exit the app).
#[tauri::command]
async fn close_window(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window("main") {
        window.close().map_err(|e| e.to_string())?;
    }
    #[cfg(not(desktop))]
    let _ = &app;
    Ok(())
}

// --- Default Directories ---

/// Get default user directories for auto-indexing (zero-config).
/// Follows how Spotlight/Alfred/Everything auto-detect user content directories.
#[tauri::command]
async fn get_default_directories() -> Result<Vec<String>, String> {
    let mut found_dirs = Vec::new();

    let mut try_add = |path: std::path::PathBuf| {
        if path.exists() {
            let s = path.to_string_lossy().to_string();
            if !found_dirs.contains(&s) {
                found_dirs.push(s);
            }
        }
    };

    // XDG/platform directories
    if let Some(doc) = dirs::document_dir() {
        try_add(doc);
    }
    if let Some(desk) = dirs::desktop_dir() {
        try_add(desk);
    }
    if let Some(dl) = dirs::download_dir() {
        try_add(dl);
    }
    if let Some(pic) = dirs::picture_dir() {
        try_add(pic);
    }

    // Additional common directories
    if let Some(home) = dirs::home_dir() {
        let extras = [
            "Documents",
            "Documentos",
            "Desktop",
            "Escritorio",
            "Downloads",
            "Descargas",
            "Pictures",
            "Imágenes",
            "Notes",
            "Obsidian",
            "org",
        ];
        for name in &extras {
            try_add(home.join(name));
        }
    }

    Ok(found_dirs)
}

// --- Search & Indexing Commands ---

#[tauri::command]
async fn search_query(
    query: String,
    limit: Option<usize>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<SearchResult>, String> {
    let limit = limit.unwrap_or(20);
    search::hybrid_search(&state.db, &state.embedding_engine, &query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn index_directory(
    path: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<indexer::IndexStats, String> {
    let dir = PathBuf::from(&path);
    indexer::index_directory(&state.db, &state.embedding_engine, &dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn index_file(path: String, state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    let file_path = PathBuf::from(&path);
    indexer::index_file(&state.db, &state.embedding_engine, &file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_stats(state: tauri::State<'_, Arc<AppState>>) -> Result<db::DbStats, String> {
    state.db.get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_ollama(state: tauri::State<'_, Arc<AppState>>) -> Result<bool, String> {
    state
        .embedding_engine
        .health_check()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_ai_status(state: tauri::State<'_, Arc<AppState>>) -> Result<AiStatus, String> {
    Ok(state.embedding_engine.status())
}

/// Start file watcher on directories. Desktop only — notify crate requires OS file events.
#[cfg(desktop)]
#[tauri::command]
async fn start_watcher(
    directories: Vec<String>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let dirs: Vec<PathBuf> = directories.iter().map(PathBuf::from).collect();

    let app_state = state.inner().clone();

    // Start watcher in a background thread
    let rx = indexer::watcher::start_watching(dirs).map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        while let Ok(events) = rx.recv() {
            for event in events {
                match event {
                    indexer::watcher::FileEvent::Changed(path) => {
                        tracing::info!("File changed, re-indexing: {}", path.display());
                        if let Err(e) =
                            indexer::index_file(&app_state.db, &app_state.embedding_engine, &path)
                                .await
                        {
                            tracing::warn!("Failed to re-index {}: {}", path.display(), e);
                        }
                    }
                    indexer::watcher::FileEvent::Removed(path) => {
                        tracing::info!("File removed: {}", path.display());
                        let path_str = path.to_string_lossy().to_string();
                        if let Ok(Some((doc_id, _))) = app_state.db.get_document_by_path(&path_str)
                        {
                            let _ = app_state.db.delete_embeddings_for_document(doc_id);
                            let _ = app_state.db.delete_chunks_for_document(doc_id);
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

/// Mobile stub — file watching not available on mobile platforms.
#[cfg(mobile)]
#[tauri::command]
async fn start_watcher(
    _directories: Vec<String>,
    _state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    tracing::info!("File watcher not available on this platform");
    Ok(())
}

#[tauri::command]
async fn get_vec_status(state: tauri::State<'_, Arc<AppState>>) -> Result<bool, String> {
    Ok(state.db.is_vec_enabled())
}

// --- Chat Commands ---

#[tauri::command]
async fn chat_send(
    messages: Vec<chat::ChatMessage>,
    max_tokens: Option<usize>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<chat::ChatResponse, String> {
    let max_tokens = max_tokens.unwrap_or_else(|| {
        state
            .settings
            .lock()
            .map(|s| s.chat_max_tokens)
            .unwrap_or(512)
    });
    push_log(
        "info",
        format!(
            "Chat: {} messages, max_tokens={}",
            messages.len(),
            max_tokens
        ),
    );
    state
        .chat_engine
        .chat(&messages, max_tokens)
        .await
        .map_err(|e| {
            push_log("error", format!("Chat error: {}", e));
            e.to_string()
        })
}

/// AG-UI streaming chat — emits events through Tauri event system.
///
/// Returns the run_id immediately. The frontend listens to
/// `agui://event` Tauri events for the streaming response.
#[tauri::command]
async fn chat_send_streaming(
    messages: Vec<chat::ChatMessage>,
    max_tokens: Option<usize>,
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let max_tokens = max_tokens.unwrap_or_else(|| {
        state
            .settings
            .lock()
            .map(|s| s.chat_max_tokens)
            .unwrap_or(512)
    });

    let run_id = format!(
        "run-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    push_log(
        "info",
        format!(
            "AG-UI streaming chat: run_id={}, {} messages, max_tokens={}",
            run_id,
            messages.len(),
            max_tokens
        ),
    );

    let state_inner = state.inner().clone();
    let run_id_clone = run_id.clone();

    // Subscribe to AG-UI events and forward to Tauri event system
    let mut rx = state_inner.agui_event_bus.subscribe();
    let app_clone = app.clone();
    let run_id_for_listener = run_id.clone();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    // Only forward events for this run
                    if event.run_id == run_id_for_listener {
                        let is_terminal = matches!(
                            event.event_type,
                            protocols::agui::EventType::RunFinished
                                | protocols::agui::EventType::RunError
                        );
                        let _ = app_clone.emit("agui://event", &event);
                        if is_terminal {
                            break;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("AG-UI event listener lagged by {} events", n);
                }
            }
        }
    });

    // Spawn the agent runner in a background task
    tokio::spawn(async move {
        let executor = agent::executor::AgentExecutor::new(state_inner.clone());
        if let Err(e) = executor
            .run(
                &run_id_clone,
                &messages,
                None, // No conversation ID for legacy streaming endpoint
                &state_inner.agui_event_bus,
            )
            .await
        {
            tracing::error!("AG-UI run failed: {}", e);
        }
    });

    Ok(run_id)
}

#[tauri::command]
async fn chat_status(state: tauri::State<'_, Arc<AppState>>) -> Result<chat::ChatStatus, String> {
    Ok(state.chat_engine.status())
}

#[tauri::command]
async fn chat_load_model(state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        state.chat_engine.load_model().await;
    });
    Ok(())
}

#[tauri::command]
async fn chat_switch_model(
    model_id: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Update settings
    {
        let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.chat_model = model_id.clone();
        let _ = settings.save(&get_app_data_dir().join("settings.json"));
    }
    push_log("info", format!("Switching to model: {}", model_id));

    state
        .chat_engine
        .switch_model(&model_id)
        .await
        .map_err(|e| e.to_string())
}

// --- Hardware & Model Commands ---

#[tauri::command]
async fn get_hardware_info(state: tauri::State<'_, Arc<AppState>>) -> Result<HardwareInfo, String> {
    Ok(state.hardware.clone())
}

#[tauri::command]
async fn get_available_models(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<chat::models::ModelInfo>, String> {
    Ok(state.chat_engine.available_models())
}

#[tauri::command]
async fn get_recommended_model(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    Ok(state.chat_engine.recommended_model_id())
}

// --- Platform Detection ---

/// Get the current platform information for the frontend.
/// Allows the UI to adapt layout and features based on platform.
#[tauri::command]
async fn get_platform_info() -> Result<serde_json::Value, String> {
    let platform = if cfg!(target_os = "android") {
        "android"
    } else if cfg!(target_os = "ios") {
        "ios"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let is_desktop = cfg!(desktop);
    let is_mobile = cfg!(mobile);

    Ok(serde_json::json!({
        "platform": platform,
        "is_desktop": is_desktop,
        "is_mobile": is_mobile,
        "has_file_watcher": is_desktop,
        "has_system_tray": is_desktop,
        "has_global_shortcuts": is_desktop,
        "has_stdio_mcp": is_desktop,
    }))
}

// --- Debug Commands ---

#[tauri::command]
async fn get_logs(since_index: Option<usize>) -> Result<Vec<LogEntry>, String> {
    let logs = LOG_BUFFER.lock().map_err(|e| e.to_string())?;
    let since = since_index.unwrap_or(0);
    Ok(logs.iter().skip(since).cloned().collect())
}

#[tauri::command]
async fn clear_logs() -> Result<(), String> {
    let mut logs = LOG_BUFFER.lock().map_err(|e| e.to_string())?;
    logs.clear();
    Ok(())
}

// --- Pro Edition Commands ---

/// Check if this build includes Ghost Pro features
#[tauri::command]
async fn is_pro() -> bool {
    #[cfg(feature = "pro")]
    {
        ghost_pro::is_licensed()
    }
    #[cfg(not(feature = "pro"))]
    {
        false
    }
}

// --- MCP Protocol Commands ---

/// Get MCP server status (whether it's running and on what address).
#[tauri::command]
async fn get_mcp_server_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let config = state
        .settings
        .lock()
        .map(|s| s.mcp_server.clone())
        .unwrap_or_default();

    Ok(serde_json::json!({
        "enabled": config.enabled,
        "host": config.host,
        "port": config.port,
        "url": format!("http://{}:{}/mcp", config.host, config.port),
    }))
}

/// List all configured external MCP servers and their connection status.
#[tauri::command]
async fn list_mcp_servers(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<protocols::mcp_client::ConnectedServer>, String> {
    Ok(state.mcp_client.list_servers().await)
}

/// Connect to an external MCP server by name.
#[tauri::command]
async fn connect_mcp_server(
    name: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<protocols::mcp_client::ConnectedServer, String> {
    let entry = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings
            .mcp_servers
            .iter()
            .find(|s| s.name == name)
            .cloned()
            .ok_or_else(|| format!("MCP server '{}' not found in settings", name))?
    };
    Ok(state.mcp_client.connect(&entry).await)
}

/// Disconnect from an external MCP server.
#[tauri::command]
async fn disconnect_mcp_server(
    name: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state
        .mcp_client
        .disconnect(&name)
        .await
        .map_err(|e| e.to_string())
}

/// Call a tool on a connected external MCP server.
#[tauri::command]
async fn call_mcp_tool(
    server_name: String,
    tool_name: String,
    arguments: Option<serde_json::Value>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    state
        .mcp_client
        .call_tool(&server_name, &tool_name, arguments)
        .await
        .map_err(|e| e.to_string())
}

/// Get all available tools from all connected MCP servers.
#[tauri::command]
async fn list_mcp_tools(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let tools = state.mcp_client.all_tools().await;
    let result: Vec<serde_json::Value> = tools
        .iter()
        .map(|(server, tool)| {
            serde_json::json!({
                "server": server,
                "name": tool.name,
                "description": tool.description,
            })
        })
        .collect();
    Ok(result)
}

/// Add a new MCP server entry to settings.
#[tauri::command]
async fn add_mcp_server_entry(
    entry: protocols::McpServerEntry,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    // Avoid duplicates
    settings.mcp_servers.retain(|s| s.name != entry.name);
    settings.mcp_servers.push(entry);
    settings
        .save(&get_app_data_dir().join("settings.json"))
        .map_err(|e| e.to_string())
}

/// Remove an MCP server entry from settings.
#[tauri::command]
async fn remove_mcp_server_entry(
    name: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Disconnect first
    let _ = state.mcp_client.disconnect(&name).await;
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.mcp_servers.retain(|s| s.name != name);
    settings
        .save(&get_app_data_dir().join("settings.json"))
        .map_err(|e| e.to_string())
}

// --- Filesystem Browsing Commands ---

/// Entry in a directory listing for the file browser.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified: String,
    pub extension: Option<String>,
    /// Whether the file is a cloud placeholder (OneDrive, iCloud, etc.)
    pub is_cloud_placeholder: bool,
    /// Whether the file is locally available
    pub is_local: bool,
}

/// List contents of a directory for the file browser.
/// Returns sorted entries: directories first, then files.
#[tauri::command]
async fn list_directory(path: String) -> Result<Vec<FsEntry>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(&dir).map_err(|e| format!("Cannot read directory: {}", e))?;

    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/directories
        if name.starts_with('.') {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let size_bytes = if is_dir { 0 } else { metadata.len() };
        let extension = if is_dir {
            None
        } else {
            entry_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
        };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| {
                    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_default()
                })
            })
            .unwrap_or_default();

        // Detect cloud placeholder files (OneDrive, iCloud, etc.)
        let (is_cloud_placeholder, is_local) = detect_cloud_status(&metadata);

        entries.push(FsEntry {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_dir,
            size_bytes,
            modified,
            extension,
            is_cloud_placeholder,
            is_local,
        });
    }

    // Sort: directories first, then alphabetical
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

/// Detect if a file is a cloud placeholder (OneDrive Files On-Demand, iCloud, etc.)
#[cfg(target_os = "windows")]
fn detect_cloud_status(metadata: &std::fs::Metadata) -> (bool, bool) {
    use std::os::windows::fs::MetadataExt;
    let attrs = metadata.file_attributes();
    // FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS = 0x00400000 (4194304)
    // FILE_ATTRIBUTE_RECALL_ON_OPEN = 0x00040000 (262144)
    // FILE_ATTRIBUTE_OFFLINE = 0x00001000 (4096)
    const RECALL_ON_DATA: u32 = 0x00400000;
    const RECALL_ON_OPEN: u32 = 0x00040000;
    const OFFLINE: u32 = 0x00001000;

    let is_cloud =
        (attrs & RECALL_ON_DATA) != 0 || (attrs & RECALL_ON_OPEN) != 0 || (attrs & OFFLINE) != 0;
    let is_local = !is_cloud;
    (is_cloud, is_local)
}

#[cfg(not(target_os = "windows"))]
fn detect_cloud_status(_metadata: &std::fs::Metadata) -> (bool, bool) {
    // On Linux/macOS, files are generally local.
    // macOS iCloud detection could be added later via extended attributes.
    (false, true)
}

/// Get the user's home directory.
#[tauri::command]
async fn get_home_directory() -> Result<String, String> {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "Could not determine home directory".to_string())
}

/// Get common root directories for filesystem browsing.
#[tauri::command]
async fn get_root_directories() -> Result<Vec<FsEntry>, String> {
    let mut roots = Vec::new();

    // Add home directory
    if let Some(home) = dirs::home_dir() {
        roots.push(FsEntry {
            name: "Home".to_string(),
            path: home.to_string_lossy().to_string(),
            is_dir: true,
            size_bytes: 0,
            modified: String::new(),
            extension: None,
            is_cloud_placeholder: false,
            is_local: true,
        });
    }

    // Platform-specific roots
    #[cfg(target_os = "windows")]
    {
        // Add common Windows drives
        for letter in ['C', 'D', 'E', 'F'] {
            let drive = format!("{}:\\", letter);
            if PathBuf::from(&drive).exists() {
                roots.push(FsEntry {
                    name: format!("{}: Drive", letter),
                    path: drive,
                    is_dir: true,
                    size_bytes: 0,
                    modified: String::new(),
                    extension: None,
                    is_cloud_placeholder: false,
                    is_local: true,
                });
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Add filesystem root
        roots.push(FsEntry {
            name: "/".to_string(),
            path: "/".to_string(),
            is_dir: true,
            size_bytes: 0,
            modified: String::new(),
            extension: None,
            is_cloud_placeholder: false,
            is_local: true,
        });

        // Add common mount points
        for mount in ["/mnt", "/media", "/run/media"] {
            if PathBuf::from(mount).exists() {
                roots.push(FsEntry {
                    name: mount.to_string(),
                    path: mount.to_string(),
                    is_dir: true,
                    size_bytes: 0,
                    modified: String::new(),
                    extension: None,
                    is_cloud_placeholder: false,
                    is_local: true,
                });
            }
        }
    }

    // Add user directories
    let user_dirs: Vec<(&str, Option<PathBuf>)> = vec![
        ("Documents", dirs::document_dir()),
        ("Desktop", dirs::desktop_dir()),
        ("Downloads", dirs::download_dir()),
        ("Pictures", dirs::picture_dir()),
    ];

    for (label, dir_opt) in user_dirs {
        if let Some(dir) = dir_opt {
            if dir.exists() {
                roots.push(FsEntry {
                    name: label.to_string(),
                    path: dir.to_string_lossy().to_string(),
                    is_dir: true,
                    size_bytes: 0,
                    modified: String::new(),
                    extension: None,
                    is_cloud_placeholder: false,
                    is_local: true,
                });
            }
        }
    }

    // Detect OneDrive directories (Windows)
    #[cfg(target_os = "windows")]
    {
        if let Some(home) = dirs::home_dir() {
            let onedrive_paths = [home.join("OneDrive"), home.join("OneDrive - Personal")];
            for od in &onedrive_paths {
                if od.exists() {
                    roots.push(FsEntry {
                        name: "OneDrive".to_string(),
                        path: od.to_string_lossy().to_string(),
                        is_dir: true,
                        size_bytes: 0,
                        modified: String::new(),
                        extension: None,
                        is_cloud_placeholder: false,
                        is_local: true,
                    });
                }
            }
        }
    }

    Ok(roots)
}

/// Add a directory to watched directories and start indexing it.
#[tauri::command]
async fn add_watch_directory(
    path: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("Invalid directory: {}", path));
    }

    // Add to settings
    {
        let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
        if !settings.watched_directories.contains(&path) {
            settings.watched_directories.push(path.clone());
            let _ = settings.save(&get_app_data_dir().join("settings.json"));
        }
    }

    // Start indexing in background
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        push_log("info", format!("Indexing new directory: {}", path));
        match indexer::index_directory(&app_state.db, &app_state.embedding_engine, &dir).await {
            Ok(stats) => {
                push_log(
                    "info",
                    format!(
                        "Indexed {}: {} files ({} ok, {} failed)",
                        path, stats.total, stats.indexed, stats.failed
                    ),
                );
            }
            Err(e) => {
                push_log("warn", format!("Failed to index {}: {}", path, e));
            }
        }
    });

    Ok(())
}

/// Remove a directory from watched directories.
#[tauri::command]
async fn remove_watch_directory(
    path: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.watched_directories.retain(|d| d != &path);
    let _ = settings.save(&get_app_data_dir().join("settings.json"));
    Ok(())
}

// --- Settings Commands ---

#[tauri::command]
async fn get_settings(state: tauri::State<'_, Arc<AppState>>) -> Result<Settings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
async fn save_settings(
    new_settings: Settings,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings;
    settings
        .save(&get_app_data_dir().join("settings.json"))
        .map_err(|e| e.to_string())
}

/// Mark initial setup/onboarding as complete.
#[tauri::command]
async fn complete_setup(state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.setup_complete = true;
    settings
        .save(&get_app_data_dir().join("settings.json"))
        .map_err(|e| e.to_string())?;
    tracing::info!("Initial setup marked as complete");
    Ok(())
}

// --- Agent Commands ---

/// Run the agent on a conversation with the full ReAct loop + tool calling.
/// Streams AG-UI events through the Tauri event system.
/// Returns the run_id immediately.
#[tauri::command]
async fn agent_chat(
    messages: Vec<chat::ChatMessage>,
    conversation_id: Option<i64>,
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let run_id = format!(
        "run-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    push_log(
        "info",
        format!(
            "Agent chat: run_id={}, {} messages, conv_id={:?}",
            run_id,
            messages.len(),
            conversation_id
        ),
    );

    let state_inner = state.inner().clone();
    let run_id_clone = run_id.clone();

    // Subscribe to AG-UI events and forward to Tauri event system
    let mut rx = state_inner.agui_event_bus.subscribe();
    let app_clone = app.clone();
    let run_id_for_listener = run_id.clone();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if event.run_id == run_id_for_listener {
                        let is_terminal = matches!(
                            event.event_type,
                            protocols::agui::EventType::RunFinished
                                | protocols::agui::EventType::RunError
                        );
                        let _ = app_clone.emit("agui://event", &event);
                        if is_terminal {
                            break;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("AG-UI event listener lagged by {} events", n);
                }
            }
        }
    });

    // Spawn the agent executor in a background task
    tokio::spawn(async move {
        let executor = agent::executor::AgentExecutor::new(state_inner.clone());
        if let Err(e) = executor
            .run(
                &run_id_clone,
                &messages,
                conversation_id,
                &state_inner.agui_event_bus,
            )
            .await
        {
            tracing::error!("Agent run failed: {}", e);
            push_log("error", format!("Agent run failed: {}", e));
        }
    });

    Ok(run_id)
}

/// Create a new conversation.
#[tauri::command]
async fn create_conversation(
    title: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<i64, String> {
    agent::memory::create_conversation(&state.db, &title).map_err(|e| e.to_string())
}

/// List all conversations.
#[tauri::command]
async fn list_conversations(
    limit: Option<usize>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<agent::memory::Conversation>, String> {
    let limit = limit.unwrap_or(50);
    agent::memory::list_conversations(&state.db, limit).map_err(|e| e.to_string())
}

/// Get messages for a conversation.
#[tauri::command]
async fn get_conversation_messages(
    conversation_id: i64,
    limit: Option<usize>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<agent::memory::Message>, String> {
    agent::memory::get_messages(&state.db, conversation_id, limit).map_err(|e| e.to_string())
}

/// Delete a conversation.
#[tauri::command]
async fn delete_conversation(
    conversation_id: i64,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    agent::memory::delete_conversation(&state.db, conversation_id).map_err(|e| e.to_string())
}

/// Update conversation title.
#[tauri::command]
async fn update_conversation_title(
    conversation_id: i64,
    title: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    agent::memory::update_conversation_title(&state.db, conversation_id, &title)
        .map_err(|e| e.to_string())
}

/// Search across conversation memory.
#[tauri::command]
async fn search_memory(
    query: String,
    limit: Option<usize>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<agent::memory::Message>, String> {
    let limit = limit.unwrap_or(20);
    agent::memory::search_conversations(&state.db, &query, limit).map_err(|e| e.to_string())
}

/// Get agent configuration.
#[tauri::command]
async fn get_agent_config(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<agent::config::AgentConfig, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.agent_config.clone())
}

/// Save agent configuration.
#[tauri::command]
async fn save_agent_config(
    config: agent::config::AgentConfig,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.agent_config = config;
    settings
        .save(&get_app_data_dir().join("settings.json"))
        .map_err(|e| e.to_string())
}

/// List available agent model tiers and which one is recommended.
#[tauri::command]
async fn get_agent_model_tiers(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let (recommended, ctx) = agent::config::recommend_agent_model(&state.hardware);
    let tiers: Vec<serde_json::Value> = agent::config::AGENT_MODEL_TIERS
        .iter()
        .map(|t| {
            serde_json::json!({
                "model_tag": t.model_tag,
                "name": t.name,
                "min_ram_mb": t.min_ram_mb,
                "recommended_ctx": t.recommended_ctx,
                "tool_calling_reliable": t.tool_calling_reliable,
                "quality": t.quality,
                "approx_usage_mb": t.approx_usage_mb,
                "is_recommended": t.model_tag == recommended.model_tag,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "tiers": tiers,
        "recommended_model": recommended.model_tag,
        "recommended_ctx": ctx,
        "available_ram_mb": state.hardware.available_ram_mb,
    }))
}

/// List loaded skills.
#[tauri::command]
async fn list_skills(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<agent::skills::Skill>, String> {
    let skills_dir = state
        .settings
        .lock()
        .map(|s| s.agent_config.skills_dir.clone())
        .unwrap_or_default();

    let mut registry = agent::skills::SkillRegistry::new();
    registry.load_from_directory(std::path::Path::new(&skills_dir));
    Ok(registry.all_skills().into_iter().cloned().collect())
}

// --- App Setup ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure ring TLS crypto provider is installed before any network I/O.
    ensure_tls_provider();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("ghost=info")),
        )
        .init();

    tracing::info!("Starting Ghost v{}", env!("CARGO_PKG_VERSION"));
    push_log(
        "info",
        format!("Starting Ghost v{}", env!("CARGO_PKG_VERSION")),
    );

    // Log edition info
    #[cfg(feature = "pro")]
    {
        push_log(
            "info",
            format!("Edition: Ghost Pro v{}", ghost_pro::version()),
        );
    }
    #[cfg(not(feature = "pro"))]
    {
        push_log("info", "Edition: Ghost Community (open source)".to_string());
    }

    // --- Step 1: Detect hardware ---
    let hardware = HardwareInfo::detect();
    push_log(
        "info",
        format!(
            "Hardware: {} cores, {}MB RAM ({}MB free), GPU={:?}, AVX2={}, NEON={}",
            hardware.cpu_cores,
            hardware.total_ram_mb,
            hardware.available_ram_mb,
            hardware.gpu_backend,
            hardware.has_avx2,
            hardware.has_neon,
        ),
    );

    // --- Step 2: Load settings ---
    let settings_path = get_app_data_dir().join("settings.json");
    let settings = Settings::load(&settings_path);
    push_log(
        "info",
        format!(
            "Settings: {} dirs, model={}",
            settings.watched_directories.len(),
            settings.chat_model,
        ),
    );

    // --- Step 3: Initialize database ---
    let db_path = get_db_path();
    tracing::info!("Database path: {}", db_path.display());
    let db = Database::open(&db_path).expect("Failed to open database");
    push_log(
        "info",
        format!("Database opened (vec_enabled={})", db.is_vec_enabled()),
    );

    // Initialize conversation memory tables
    if let Err(e) = agent::memory::initialize_memory_schema(&db) {
        tracing::warn!("Failed to initialize conversation memory schema: {}", e);
        push_log("warn", format!("Memory schema init failed: {}", e));
    } else {
        push_log("info", "Conversation memory schema initialized".to_string());
    }

    // --- Step 4: Initialize embedding engine ---
    let embedding_engine = tauri::async_runtime::block_on(async {
        tracing::info!("Initializing AI embedding engine...");
        let engine = EmbeddingEngine::initialize().await;
        tracing::info!("AI backend active: {}", engine.backend());
        push_log("info", format!("Embedding engine: {}", engine.backend()));
        engine
    });

    // --- Step 5: Determine chat model ---
    let model_id = if settings.chat_model == "auto" {
        let recommended = chat::models::recommend_model(&hardware);
        push_log(
            "info",
            format!(
                "Auto-selected model: {} ({}, ~{}MB)",
                recommended.name, recommended.parameters, recommended.size_mb
            ),
        );
        recommended.id.to_string()
    } else {
        push_log(
            "info",
            format!("Using configured model: {}", settings.chat_model),
        );
        settings.chat_model.clone()
    };

    // --- Step 6: Create chat engine (deferred loading) ---
    // llama.cpp auto-detects GPU at runtime — no device preference needed.
    let chat_engine = chat::ChatEngine::new(hardware.clone(), model_id.clone());
    push_log(
        "info",
        format!(
            "Chat engine created (model={}). GPU auto-detection on load...",
            model_id
        ),
    );

    let app_state = Arc::new(AppState {
        db,
        embedding_engine,
        chat_engine,
        hardware,
        settings: std::sync::Mutex::new(settings),
        mcp_client: protocols::mcp_client::McpClientManager::new(),
        agui_event_bus: protocols::agui::AgUiEventBus::new(256),
    });

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    // Desktop-only plugins
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());
    }

    builder
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            // Search & indexing
            search_query,
            index_directory,
            index_file,
            get_stats,
            check_ollama,
            check_ai_status,
            start_watcher,
            get_vec_status,
            // Window
            hide_window,
            show_window,
            start_dragging,
            minimize_window,
            toggle_maximize_window,
            close_window,
            // Auto-indexing
            get_default_directories,
            // Chat
            chat_send,
            chat_send_streaming,
            chat_status,
            chat_load_model,
            chat_switch_model,
            // Hardware & models
            get_hardware_info,
            get_available_models,
            get_recommended_model,
            // Platform
            get_platform_info,
            // Debug
            get_logs,
            clear_logs,
            // Settings
            get_settings,
            save_settings,
            complete_setup,
            // Pro
            is_pro,
            // Filesystem browsing
            list_directory,
            get_home_directory,
            get_root_directories,
            add_watch_directory,
            remove_watch_directory,
            // MCP Protocol
            get_mcp_server_status,
            list_mcp_servers,
            connect_mcp_server,
            disconnect_mcp_server,
            call_mcp_tool,
            list_mcp_tools,
            add_mcp_server_entry,
            remove_mcp_server_entry,
            // Agent
            agent_chat,
            create_conversation,
            list_conversations,
            get_conversation_messages,
            delete_conversation,
            update_conversation_title,
            search_memory,
            get_agent_config,
            save_agent_config,
            get_agent_model_tiers,
            list_skills,
        ])
        .setup(move |app| {
            // --- Desktop-only setup: System Tray + Global Shortcuts ---
            #[cfg(desktop)]
            {
                let show_item = MenuItem::with_id(app, "show", "Show Ghost", true, None::<&str>)?;
                let quit_item = MenuItem::with_id(app, "quit", "Quit Ghost", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

                let tray_handle = app.handle().clone();
                let tray_icon = app.default_window_icon().cloned();
                let mut tray_builder = TrayIconBuilder::new()
                    .menu(&menu)
                    .tooltip("Ghost — AI Assistant");
                if let Some(icon) = tray_icon {
                    tray_builder = tray_builder.icon(icon);
                }
                tray_builder
                    .on_menu_event(move |_app, event| match event.id().as_ref() {
                        "show" => {
                            toggle_window(&tray_handle);
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Left,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;

                tracing::info!("System tray icon created");

                // Register global shortcut: Ctrl+Space (or Cmd+Space on macOS)
                use tauri_plugin_global_shortcut::ShortcutState;
                let handle = app.handle().clone();

                app.global_shortcut()
                    .on_shortcut("CmdOrCtrl+Space", move |_app, shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            tracing::debug!("Global shortcut pressed: {:?}", shortcut);
                            toggle_window(&handle);
                        }
                    })
                    .unwrap_or_else(|e| {
                        tracing::warn!("Failed to register global shortcut CmdOrCtrl+Space: {}", e);
                    });

                tracing::info!("Global shortcut registered: CmdOrCtrl+Space");
            }

            // Suppress unused `app` warning on mobile (tray/shortcuts are desktop-only)
            #[cfg(not(desktop))]
            let _ = &app;

            // --- Start MCP Server ---
            let mcp_state = app_state.clone();
            let mcp_config = mcp_state
                .settings
                .lock()
                .map(|s| s.mcp_server.clone())
                .unwrap_or_default();

            tauri::async_runtime::spawn(async move {
                match protocols::start_mcp_server(mcp_state.clone(), &mcp_config).await {
                    Ok(addr) => {
                        push_log("info", format!("MCP server started on {}", addr));
                        tracing::info!("MCP server started on {}", addr);
                    }
                    Err(e) => {
                        push_log("warn", format!("MCP server failed to start: {}", e));
                        tracing::warn!("MCP server failed to start: {}", e);
                    }
                }

                // Connect to configured external MCP servers
                let mcp_servers = mcp_state
                    .settings
                    .lock()
                    .map(|s| s.mcp_servers.clone())
                    .unwrap_or_default();

                for entry in &mcp_servers {
                    if entry.enabled {
                        let result = mcp_state.mcp_client.connect(entry).await;
                        if result.connected {
                            push_log(
                                "info",
                                format!(
                                    "MCP client connected to '{}' ({} tools)",
                                    result.name,
                                    result.tools.len(),
                                ),
                            );
                        } else if let Some(err) = &result.error {
                            push_log(
                                "warn",
                                format!(
                                    "MCP client failed to connect to '{}': {}",
                                    result.name, err
                                ),
                            );
                        }
                    }
                }
            });

            // --- Background model loading ---
            // Don't block app startup — load the chat model in a background task
            let state_for_loading = app_state.clone();
            tauri::async_runtime::spawn(async move {
                tracing::info!("Background: starting chat model load...");
                state_for_loading.chat_engine.load_model().await;
                let status = state_for_loading.chat_engine.status();
                push_log(
                    "info",
                    format!(
                        "Chat engine ready: {} ({}) [{}]",
                        status.backend, status.model_name, status.device
                    ),
                );
                tracing::info!(
                    "Chat model loaded: {} ({})",
                    status.backend,
                    status.model_name
                );
            });

            // --- Auto-indexing on first launch ---
            // Like Spotlight/Alfred: auto-detect user content directories and index them.
            // Only triggers when no directories are configured (first run).
            let state_for_autoindex = app_state.clone();
            tauri::async_runtime::spawn(async move {
                let needs_auto_setup = {
                    let settings = state_for_autoindex.settings.lock().unwrap();
                    settings.watched_directories.is_empty()
                };

                if needs_auto_setup {
                    tracing::info!("First launch detected — auto-discovering user directories...");
                    push_log(
                        "info",
                        "First launch: auto-discovering user directories".into(),
                    );

                    let mut auto_dirs = Vec::new();

                    // Detect directories using dirs crate (cross-platform)
                    if let Some(home) = dirs::home_dir() {
                        let candidates = [
                            dirs::document_dir(),
                            dirs::desktop_dir(),
                            dirs::download_dir(),
                            dirs::picture_dir(),
                        ];

                        for dir in candidates.into_iter().flatten() {
                            if dir.exists() {
                                let s = dir.to_string_lossy().to_string();
                                if !auto_dirs.contains(&s) {
                                    auto_dirs.push(s);
                                }
                            }
                        }

                        // Additional common directories
                        let extra = [home.join("Notes"), home.join("Obsidian"), home.join("org")];
                        for dir in &extra {
                            if dir.exists() {
                                let s = dir.to_string_lossy().to_string();
                                if !auto_dirs.contains(&s) {
                                    auto_dirs.push(s);
                                }
                            }
                        }
                    }

                    if !auto_dirs.is_empty() {
                        push_log(
                            "info",
                            format!(
                                "Auto-discovered {} directories: {:?}",
                                auto_dirs.len(),
                                auto_dirs
                            ),
                        );
                        tracing::info!("Auto-discovered {} directories", auto_dirs.len());

                        // Save to settings so this only happens once
                        {
                            let mut settings = state_for_autoindex.settings.lock().unwrap();
                            settings.watched_directories = auto_dirs.clone();
                            let _ = settings.save(&get_app_data_dir().join("settings.json"));
                        }

                        // Start indexing each directory
                        for dir_path in &auto_dirs {
                            let path = std::path::PathBuf::from(dir_path);
                            push_log("info", format!("Auto-indexing: {}", dir_path));
                            match crate::indexer::index_directory(
                                &state_for_autoindex.db,
                                &state_for_autoindex.embedding_engine,
                                &path,
                            )
                            .await
                            {
                                Ok(stats) => {
                                    push_log(
                                        "info",
                                        format!(
                                            "Indexed {}: {} files ({} ok, {} failed)",
                                            dir_path, stats.total, stats.indexed, stats.failed
                                        ),
                                    );
                                }
                                Err(e) => {
                                    push_log(
                                        "warn",
                                        format!("Failed to index {}: {}", dir_path, e),
                                    );
                                    tracing::warn!("Auto-index failed for {}: {}", dir_path, e);
                                }
                            }
                        }

                        // Start file watcher on discovered directories (desktop only)
                        #[cfg(desktop)]
                        {
                            let watch_dirs: Vec<std::path::PathBuf> =
                                auto_dirs.iter().map(std::path::PathBuf::from).collect();
                            match crate::indexer::watcher::start_watching(watch_dirs) {
                                Ok(rx) => {
                                    let watcher_state = state_for_autoindex.clone();
                                    tauri::async_runtime::spawn(async move {
                                        while let Ok(events) = rx.recv() {
                                            for event in events {
                                                match event {
                                                    crate::indexer::watcher::FileEvent::Changed(
                                                        path,
                                                    ) => {
                                                        tracing::info!(
                                                            "File changed, re-indexing: {}",
                                                            path.display()
                                                        );
                                                        if let Err(e) = crate::indexer::index_file(
                                                            &watcher_state.db,
                                                            &watcher_state.embedding_engine,
                                                            &path,
                                                        )
                                                        .await
                                                        {
                                                            tracing::warn!(
                                                                "Failed to re-index {}: {}",
                                                                path.display(),
                                                                e
                                                            );
                                                        }
                                                    }
                                                    crate::indexer::watcher::FileEvent::Removed(
                                                        path,
                                                    ) => {
                                                        tracing::info!(
                                                            "File removed: {}",
                                                            path.display()
                                                        );
                                                        let path_str =
                                                            path.to_string_lossy().to_string();
                                                        if let Ok(Some((doc_id, _))) = watcher_state
                                                            .db
                                                            .get_document_by_path(&path_str)
                                                        {
                                                            let _ = watcher_state
                                                                .db
                                                                .delete_embeddings_for_document(
                                                                    doc_id,
                                                                );
                                                            let _ = watcher_state
                                                                .db
                                                                .delete_chunks_for_document(doc_id);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });
                                    push_log(
                                        "info",
                                        "File watcher started on auto-discovered directories"
                                            .into(),
                                    );
                                }
                                Err(e) => {
                                    push_log("warn", format!("Failed to start watcher: {}", e));
                                }
                            }
                        }

                        push_log("info", "Auto-indexing complete!".into());
                        tracing::info!("Auto-indexing complete");
                    } else {
                        push_log("warn", "No user directories found for auto-indexing".into());
                    }
                }
            });

            // --- Periodic re-indexing (every 5 minutes) ---
            // Catches new files that the watcher might miss (e.g., files added
            // while Ghost was closed, or files in newly added subdirectories).
            let state_for_reindex = app_state.clone();
            tauri::async_runtime::spawn(async move {
                // Wait 60 seconds before first re-index to let initial indexing finish
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    let dirs = {
                        state_for_reindex
                            .settings
                            .lock()
                            .map(|s| s.watched_directories.clone())
                            .unwrap_or_default()
                    };
                    if dirs.is_empty() {
                        continue;
                    }
                    push_log(
                        "info",
                        format!("Periodic re-index: scanning {} directories", dirs.len()),
                    );
                    for dir_path in &dirs {
                        let path = std::path::PathBuf::from(dir_path);
                        if let Err(e) = crate::indexer::index_directory(
                            &state_for_reindex.db,
                            &state_for_reindex.embedding_engine,
                            &path,
                        )
                        .await
                        {
                            tracing::warn!("Periodic re-index failed for {}: {}", dir_path, e);
                        }
                    }
                    if let Ok(stats) = state_for_reindex.db.get_stats() {
                        push_log(
                            "info",
                            format!(
                                "Re-index complete: {} docs, {} chunks",
                                stats.document_count, stats.chunk_count
                            ),
                        );
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Ghost");
}
