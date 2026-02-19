use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};

use crate::error::{GhostError, Result};
use crate::indexer::extractor;

/// Events emitted by the file watcher.
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// A file was created or modified and should be (re)indexed.
    Changed(PathBuf),
    /// A file was removed and should be deindexed.
    Removed(PathBuf),
}

/// Start watching a directory for file changes.
/// Returns a receiver that emits FileEvents.
pub fn start_watching(
    directories: Vec<PathBuf>,
) -> Result<mpsc::Receiver<Vec<FileEvent>>> {
    let (tx, rx) = mpsc::channel();

    let (debounced_tx, debounced_rx) = mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(300), debounced_tx)
        .map_err(|e| GhostError::Indexer(format!("Failed to create debouncer: {}", e)))?;

    for dir in &directories {
        if dir.exists() {
            debouncer
                .watcher()
                .watch(dir, RecursiveMode::Recursive)
                .map_err(|e| {
                    GhostError::Indexer(format!("Failed to watch {}: {}", dir.display(), e))
                })?;
            tracing::info!("Watching directory: {}", dir.display());
        } else {
            tracing::warn!("Directory does not exist, skipping: {}", dir.display());
        }
    }

    // Spawn a thread to process debounced events
    std::thread::spawn(move || {
        // Keep the debouncer alive
        let _debouncer = debouncer;

        loop {
            match debounced_rx.recv() {
                Ok(Ok(events)) => {
                    let file_events = process_events(events);
                    if !file_events.is_empty()
                        && tx.send(file_events).is_err()
                    {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!("Watch error: {:?}", e);
                }
                Err(_) => break,
            }
        }
    });

    Ok(rx)
}

fn process_events(events: Vec<DebouncedEvent>) -> Vec<FileEvent> {
    let mut file_events = Vec::new();

    for event in events {
        let path = event.path;

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Skip hidden files and system files
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            if filename.starts_with('.') || filename.starts_with('~') {
                continue;
            }
        }

        // Only process supported file types
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !extractor::is_supported_extension(ext) {
                continue;
            }
        } else {
            continue;
        }

        if path.exists() {
            file_events.push(FileEvent::Changed(path));
        } else {
            file_events.push(FileEvent::Removed(path));
        }
    }

    file_events
}
