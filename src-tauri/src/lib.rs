mod chat;
mod db;
mod embeddings;
mod error;
mod indexer;
mod search;
mod settings;

use std::path::PathBuf;
use std::sync::Arc;

use db::Database;
use embeddings::{AiStatus, EmbeddingEngine};
use embeddings::hardware::HardwareInfo;
use search::SearchResult;
use settings::Settings;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// Application state shared across commands.
pub struct AppState {
    pub db: Database,
    pub embedding_engine: EmbeddingEngine,
    pub chat_engine: chat::ChatEngine,
    pub hardware: HardwareInfo,
    pub settings: std::sync::Mutex<Settings>,
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

/// Toggle window visibility (show/hide).
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
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
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

#[tauri::command]
async fn start_watcher(
    directories: Vec<String>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let dirs: Vec<PathBuf> = directories.iter().map(PathBuf::from).collect();

    let app_state = state.inner().clone();

    // Start watcher in a background thread
    let rx = indexer::watcher::start_watching(dirs).map_err(|e| e.to_string())?;

    tokio::spawn(async move {
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
        state.settings.lock().map(|s| s.chat_max_tokens).unwrap_or(512)
    });
    push_log(
        "info",
        format!("Chat: {} messages, max_tokens={}", messages.len(), max_tokens),
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

#[tauri::command]
async fn chat_status(state: tauri::State<'_, Arc<AppState>>) -> Result<chat::ChatStatus, String> {
    Ok(state.chat_engine.status())
}

#[tauri::command]
async fn chat_load_model(state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    let state = state.inner().clone();
    tokio::spawn(async move {
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
async fn get_hardware_info(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<HardwareInfo, String> {
    Ok(state.hardware.clone())
}

#[tauri::command]
async fn get_available_models(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<chat::models::ModelInfo>, String> {
    Ok(state.chat_engine.available_models())
}

#[tauri::command]
async fn get_recommended_model(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    Ok(state.chat_engine.recommended_model_id())
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

// --- App Setup ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            "Settings: {} dirs, model={}, device={}",
            settings.watched_directories.len(),
            settings.chat_model,
            settings.chat_device,
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
    let chat_engine = chat::ChatEngine::new(
        hardware.clone(),
        model_id.clone(),
        settings.chat_device.clone(),
    );
    push_log(
        "info",
        format!(
            "Chat engine created (model={}, device={}). Loading in background...",
            model_id, settings.chat_device
        ),
    );

    let app_state = Arc::new(AppState {
        db,
        embedding_engine,
        chat_engine,
        hardware,
        settings: std::sync::Mutex::new(settings),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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
            // Chat
            chat_send,
            chat_status,
            chat_load_model,
            chat_switch_model,
            // Hardware & models
            get_hardware_info,
            get_available_models,
            get_recommended_model,
            // Debug
            get_logs,
            clear_logs,
            // Settings
            get_settings,
            save_settings,
        ])
        .setup(move |app| {
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

            // --- Background model loading ---
            // Don't block app startup — load the chat model in a background task
            let state_for_loading = app_state.clone();
            tokio::spawn(async move {
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Ghost");
}
