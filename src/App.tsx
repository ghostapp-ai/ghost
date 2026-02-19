import { useState, useCallback, useEffect } from "react";
import { SearchBar } from "./components/SearchBar";
import { ResultsList } from "./components/ResultsList";
import { StatusBar } from "./components/StatusBar";
import { Settings } from "./components/Settings";
import { ChatPanel } from "./components/ChatPanel";
import { DebugPanel } from "./components/DebugPanel";
import { useSearch } from "./hooks/useSearch";
import { useHotkey } from "./hooks/useHotkey";
import { hideWindow, openFile, getSettings, startWatcher } from "./lib/tauri";
import "./styles/globals.css";

type AppMode = "search" | "chat";

export default function App() {
  const { query, setQuery, results, isLoading, error } = useSearch(150);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showSettings, setShowSettings] = useState(false);
  const [hasDirectories, setHasDirectories] = useState<boolean | null>(null);
  const [mode, setMode] = useState<AppMode>("chat");
  const [debugOpen, setDebugOpen] = useState(false);

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
    if (mode === "search") {
      setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
    }
  });

  useHotkey("ArrowUp", () => {
    if (mode === "search") {
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    }
  });

  useHotkey("Enter", () => {
    if (mode === "search") {
      openSelectedFile();
    }
  });

  useHotkey("Escape", () => {
    if (showSettings) {
      setShowSettings(false);
    } else if (mode === "search" && query) {
      handleQueryChange("");
    } else {
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

  // Tab switching: Ctrl+1 = Search, Ctrl+2 = Chat
  useHotkey("1", () => setMode("search"), { ctrl: true });
  useHotkey("2", () => setMode("chat"), { ctrl: true });

  // Toggle debug panel
  useHotkey(
    "d",
    () => setDebugOpen((prev) => !prev),
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
        <div className="flex items-center justify-between mb-3">
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

          {/* Mode tabs */}
          <div className="flex items-center bg-ghost-surface rounded-lg p-0.5 border border-ghost-border/50">
            <TabButton
              active={mode === "search"}
              onClick={() => setMode("search")}
              label="Search"
              shortcut="⌃1"
            />
            <TabButton
              active={mode === "chat"}
              onClick={() => setMode("chat")}
              label="Chat"
              shortcut="⌃2"
            />
          </div>

          <span className="text-[10px] text-ghost-text-dim/40 font-mono">
            v0.1.0
          </span>
        </div>

        {/* Search bar (only in search mode) */}
        {mode === "search" && (
          <>
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
          </>
        )}
      </header>

      {/* Main content */}
      <main className="flex-1 overflow-hidden">
        {mode === "search" ? (
          <div className="h-full px-3">
            {hasDirectories === false && !query.trim() ? (
              <div className="flex flex-col items-center justify-center h-64 text-ghost-text-dim/60 gap-4">
                <div className="w-14 h-14 rounded-2xl bg-ghost-accent/10 flex items-center justify-center">
                  <span className="text-ghost-accent text-2xl">📁</span>
                </div>
                <div className="text-center space-y-1">
                  <p className="text-sm font-medium text-ghost-text">
                    Bienvenido a Ghost
                  </p>
                  <p className="text-xs text-ghost-text-dim/50 max-w-70">
                    Agrega un directorio en Settings para indexar tus archivos.
                  </p>
                </div>
                <button
                  onClick={() => setShowSettings(true)}
                  className="px-4 py-2 bg-ghost-accent text-white rounded-xl text-sm font-medium hover:bg-ghost-accent-dim transition-all"
                >
                  Abrir Settings
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
          </div>
        ) : (
          <ChatPanel />
        )}
      </main>

      {/* Debug Panel (collapsible) */}
      <DebugPanel isOpen={debugOpen} onToggle={() => setDebugOpen(!debugOpen)} />

      {/* Status Bar */}
      <StatusBar onSettingsClick={() => setShowSettings(true)} />

      {/* Settings Modal */}
      {showSettings && (
        <Settings
          onClose={() => {
            setShowSettings(false);
            getSettings()
              .then((s) => setHasDirectories(s.watched_directories.length > 0))
              .catch(() => {});
          }}
        />
      )}
    </div>
  );
}

function TabButton({
  active,
  onClick,
  label,
  shortcut,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
  shortcut: string;
}) {
  return (
    <button
      onClick={onClick}
      className={`px-3 py-1 rounded-md text-xs font-medium transition-all ${
        active
          ? "bg-ghost-accent/20 text-ghost-accent"
          : "text-ghost-text-dim/50 hover:text-ghost-text-dim"
      }`}
      title={shortcut}
    >
      {label}
    </button>
  );
}
