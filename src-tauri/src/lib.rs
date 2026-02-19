mod db;
mod embeddings;
mod error;
mod indexer;
mod search;

use std::path::PathBuf;
use std::sync::Arc;

use db::Database;
use embeddings::{AiStatus, EmbeddingEngine};
use search::SearchResult;

/// Application state shared across commands.
pub struct AppState {
    pub db: Database,
    pub embedding_engine: EmbeddingEngine,
}

/// Get the default vault database path.
fn get_db_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.ghost.app");
    data_dir.join("ghost_vault.db")
}

// --- Tauri Commands ---

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
async fn index_file(
    path: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let file_path = PathBuf::from(&path);
    indexer::index_file(&state.db, &state.embedding_engine, &file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_stats(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<db::DbStats, String> {
    state.db.get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_ollama(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    state
        .embedding_engine
        .health_check()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_ai_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<AiStatus, String> {
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
                        if let Ok(Some((doc_id, _))) =
                            app_state.db.get_document_by_path(&path_str)
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
async fn get_vec_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    Ok(state.db.is_vec_enabled())
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

    // Initialize database
    let db_path = get_db_path();
    tracing::info!("Database path: {}", db_path.display());

    let db = Database::open(&db_path).expect("Failed to open database");

    // Initialize embedding engine (tries Native → Ollama → None).
    // Uses block_on since model loads from cache are fast (<200ms).
    // First-time download happens here too (requires network once).
    let embedding_engine = tauri::async_runtime::block_on(async {
        tracing::info!("Initializing AI embedding engine...");
        let engine = EmbeddingEngine::initialize().await;
        tracing::info!("AI backend active: {}", engine.backend());
        engine
    });

    let app_state = Arc::new(AppState {
        db,
        embedding_engine,
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            search_query,
            index_directory,
            index_file,
            get_stats,
            check_ollama,
            check_ai_status,
            start_watcher,
            get_vec_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ghost");
}
