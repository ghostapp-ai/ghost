import { useState, useCallback, useEffect } from "react";
import { SearchBar } from "./components/SearchBar";
import { ResultsList } from "./components/ResultsList";
import { StatusBar } from "./components/StatusBar";
import { Settings } from "./components/Settings";
import { useSearch } from "./hooks/useSearch";
import { useHotkey } from "./hooks/useHotkey";
import { hideWindow, openFile, getSettings, startWatcher } from "./lib/tauri";
import "./styles/globals.css";

export default function App() {
  const { query, setQuery, results, isLoading, error } = useSearch(150);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showSettings, setShowSettings] = useState(false);
  const [hasDirectories, setHasDirectories] = useState<boolean | null>(null);

  // Reset selection when results change
  const handleQueryChange = useCallback(
    (q: string) => {
      setQuery(q);
      setSelectedIndex(0);
    },
    [setQuery]
  );

  // Auto-hide window on blur (Spotlight-like behavior)
  useEffect(() => {
    const handleBlur = () => {
      // Don't hide when settings are open (user is interacting)
      if (!showSettings) {
        hideWindow().catch(() => {});
      }
    };
    window.addEventListener("blur", handleBlur);
    return () => window.removeEventListener("blur", handleBlur);
  }, [showSettings]);

  // Reset query when window becomes visible again
  useEffect(() => {
    const handleFocus = () => {
      // Clear previous search when re-activated
      handleQueryChange("");
    };
    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, [handleQueryChange]);

  // Auto-start file watcher with saved directories on app launch
  useEffect(() => {
    getSettings()
      .then((s) => {
        setHasDirectories(s.watched_directories.length > 0);
        if (s.watched_directories.length > 0) {
          startWatcher(s.watched_directories).catch(() => {});
        }
      })
      .catch(() => setHasDirectories(false));
  }, []);

  // Open the currently selected file
  const openSelectedFile = useCallback(() => {
    const result = results[selectedIndex];
    if (result) {
      openFile(result.path).catch(() => {});
      hideWindow().catch(() => {});
    }
  }, [results, selectedIndex]);

  // Keyboard navigation
  useHotkey("ArrowDown", () => {
    setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
  });

  useHotkey("ArrowUp", () => {
    setSelectedIndex((prev) => Math.max(prev - 1, 0));
  });

  useHotkey("Enter", () => {
    openSelectedFile();
  });

  useHotkey("Escape", () => {
    if (showSettings) {
      setShowSettings(false);
    } else if (query) {
      handleQueryChange("");
    } else {
      // Hide window when pressing Escape with empty query
      hideWindow().catch(() => {});
    }
  });

  useHotkey(
    ",",
    () => {
      setShowSettings((prev) => !prev);
    },
    { ctrl: true }
  );

  return (
    <div className="flex flex-col h-screen bg-ghost-bg rounded-2xl overflow-hidden border border-ghost-border/50 shadow-2xl">
      {/* Draggable title bar region */}
      <div
        data-tauri-drag-region
        className="h-3 shrink-0 cursor-grab active:cursor-grabbing"
      />

      {/* Header */}
      <header className="shrink-0 px-5 pb-3">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2.5">
            <img
              src="/ghost-logo.svg"
              alt="Ghost"
              className="w-7 h-7 rounded-lg"
              draggable={false}
            />
            <h1 className="text-sm font-semibold text-ghost-text tracking-tight">
              Ghost
            </h1>
          </div>
          <span className="text-[10px] text-ghost-text-dim/40 font-mono">
            v0.1.0
          </span>
        </div>

        <SearchBar
          value={query}
          onChange={handleQueryChange}
          isLoading={isLoading}
          resultCount={results.length}
        />

        {error && (
          <div className="mt-2 px-4 py-2 bg-ghost-danger/10 border border-ghost-danger/20 rounded-xl text-xs text-ghost-danger">
            {error}
          </div>
        )}
      </header>

      {/* Results */}
      <main className="flex-1 overflow-hidden px-3">
        {hasDirectories === false && !query.trim() ? (
          <div className="flex flex-col items-center justify-center h-64 text-ghost-text-dim/60 gap-4">
            <div className="w-14 h-14 rounded-2xl bg-ghost-accent/10 flex items-center justify-center">
              <span className="text-ghost-accent text-2xl">📁</span>
            </div>
            <div className="text-center space-y-1">
              <p className="text-sm font-medium text-ghost-text">Welcome to Ghost</p>
              <p className="text-xs text-ghost-text-dim/50 max-w-[280px]">
                Add a directory in Settings to start indexing your files, then search instantly.
              </p>
            </div>
            <button
              onClick={() => setShowSettings(true)}
              className="px-4 py-2 bg-ghost-accent text-white rounded-xl text-sm font-medium hover:bg-ghost-accent-dim transition-all"
            >
              Open Settings
            </button>
          </div>
        ) : (
          <ResultsList
            results={results}
            selectedIndex={selectedIndex}
            onSelect={setSelectedIndex}
            onOpen={(index) => {
              const result = results[index];
              if (result) {
                openFile(result.path).catch(() => {});
                hideWindow().catch(() => {});
              }
            }}
            isLoading={isLoading}
            hasQuery={!!query.trim()}
          />
        )}
      </main>

      {/* Status Bar */}
      <StatusBar onSettingsClick={() => setShowSettings(true)} />

      {/* Settings Modal */}
      {showSettings && (
        <Settings
          onClose={() => {
            setShowSettings(false);
            // Refresh directories status after settings change
            getSettings()
              .then((s) => setHasDirectories(s.watched_directories.length > 0))
              .catch(() => {});
          }}
        />
      )}
    </div>
  );
}
