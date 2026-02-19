//! Persistent application settings.
//!
//! Settings are stored as JSON in the app data directory and survive restarts.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Application settings persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Directories to watch and index.
    pub watched_directories: Vec<String>,
    /// Global shortcut key combination (default: "CmdOrCtrl+Space").
    pub shortcut: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            watched_directories: Vec::new(),
            shortcut: "CmdOrCtrl+Space".to_string(),
        }
    }
}

impl Settings {
    /// Load settings from a JSON file. Returns defaults if file doesn't exist.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                tracing::warn!("Failed to parse settings file: {} — using defaults", e);
                Self::default()
            }),
            Err(_) => {
                tracing::info!("No settings file found, using defaults");
                Self::default()
            }
        }
    }

    /// Save settings to a JSON file.
    pub fn save(&self, path: &Path) -> crate::error::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        tracing::info!("Settings saved to {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.watched_directories.is_empty());
        assert_eq!(settings.shortcut, "CmdOrCtrl+Space");
    }

    #[test]
    fn test_save_and_load() {
        let tmp = std::env::temp_dir().join("ghost_test_settings.json");
        let settings = Settings {
            watched_directories: vec!["/home/user/docs".to_string()],
            shortcut: "CmdOrCtrl+Space".to_string(),
        };
        settings.save(&tmp).unwrap();

        let loaded = Settings::load(&tmp);
        assert_eq!(loaded.watched_directories, vec!["/home/user/docs"]);

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_load_missing_file() {
        let settings = Settings::load(&PathBuf::from("/nonexistent/settings.json"));
        assert!(settings.watched_directories.is_empty());
    }
}
