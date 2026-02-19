import { useState, useCallback, useEffect } from "react";
import { GhostInput } from "./components/GhostInput";
import { ResultsList } from "./components/ResultsList";
import { ChatMessages } from "./components/ChatMessages";
import { StatusBar } from "./components/StatusBar";
import { Settings } from "./components/Settings";
import { DebugPanel } from "./components/DebugPanel";
import { useSearch } from "./hooks/useSearch";
import { useHotkey } from "./hooks/useHotkey";
import { detectMode, type InputMode } from "./lib/detectMode";
import {
  hideWindow,
  openFile,
  getSettings,
  startWatcher,
  chatSend,
  chatStatus as fetchChatStatus,
  chatLoadModel,
} from "./lib/tauri";
import type { ChatMessage, ChatStatus } from "./lib/types";
import "./styles/globals.css";

export default function App() {
  // --- Search state ---
  const { query, setQuery, results, isLoading: isSearching, error: searchError } = useSearch(150);
  const [selectedIndex, setSelectedIndex] = useState(0);

  // --- Chat state ---
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [chatError, setChatError] = useState<string | null>(null);
  const [tokensInfo, setTokensInfo] = useState<string | null>(null);
  const [chatSt, setChatSt] = useState<ChatStatus | null>(null);

  // --- UI state ---
  const [mode, setMode] = useState<InputMode>("search");
  const [modeOverride, setModeOverride] = useState<InputMode | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [hasDirectories, setHasDirectories] = useState<boolean | null>(null);
  const [debugOpen, setDebugOpen] = useState(false);

  // Effective mode: manual override wins, then auto-detected
  const activeMode = modeOverride ?? mode;

  // --- Poll chat status ---
  useEffect(() => {
    const refresh = () => fetchChatStatus().then(setChatSt).catch(() => {});
    refresh();
    const id = setInterval(refresh, 2000);
    return () => clearInterval(id);
  }, []);

  // --- Auto-start watcher ---
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

  // --- Auto-hide on blur ---
  useEffect(() => {
    const handleBlur = () => {
      if (!showSettings) hideWindow().catch(() => {});
    };
    window.addEventListener("blur", handleBlur);
    return () => window.removeEventListener("blur", handleBlur);
  }, [showSettings]);

  // --- Reset on focus ---
  useEffect(() => {
    const handleFocus = () => {
      handleQueryChange("");
    };
    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, []);

  // --- Input handling ---
  const handleQueryChange = useCallback(
    (q: string) => {
      setQuery(q);
      setSelectedIndex(0);
      const detected = detectMode(q, messages.length > 0);
      setMode(detected);
      setModeOverride(null);
    },
    [setQuery, messages.length]
  );

  // --- Mode toggle ---
  const handleModeToggle = useCallback(() => {
    setModeOverride((prev) => {
      const current = prev ?? mode;
      return current === "search" ? "chat" : "search";
    });
  }, [mode]);

  // --- Submit (Enter) ---
  const handleSubmit = useCallback(async () => {
    if (activeMode === "search") {
      const result = results[selectedIndex];
      if (result) {
        openFile(result.path).catch(() => {});
        hideWindow().catch(() => {});
      }
    } else {
      const trimmed = query.trim();
      if (!trimmed || isGenerating) return;

      const cleanQuery = trimmed.replace(/^[?@]\s*/, "");
      if (!cleanQuery) return;

      setChatError(null);
      setTokensInfo(null);
      const userMsg: ChatMessage = { role: "user", content: cleanQuery };
      const newMessages = [...messages, userMsg];
      setMessages(newMessages);
      setQuery("");
      setIsGenerating(true);

      try {
        const response = await chatSend(newMessages);
        const assistantMsg: ChatMessage = {
          role: "assistant",
          content: response.content,
        };
        setMessages([...newMessages, assistantMsg]);
        setTokensInfo(
          `${response.tokens_generated} tokens · ${(response.duration_ms / 1000).toFixed(1)}s · ${response.model_id}`
        );
      } catch (e) {
        setChatError(e instanceof Error ? e.message : String(e));
      } finally {
        setIsGenerating(false);
      }
    }
  }, [activeMode, results, selectedIndex, query, isGenerating, messages, setQuery]);

  // --- Clear chat ---
  const clearChat = useCallback(() => {
    setMessages([]);
    setChatError(null);
    setTokensInfo(null);
  }, []);

  // --- Keyboard navigation ---
  useHotkey("ArrowDown", () => {
    if (activeMode === "search") {
      setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
    }
  });

  useHotkey("ArrowUp", () => {
    if (activeMode === "search") {
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    }
  });

  useHotkey("Escape", () => {
    if (showSettings) {
      setShowSettings(false);
    } else if (query) {
      handleQueryChange("");
    } else if (messages.length > 0) {
      clearChat();
    } else {
      hideWindow().catch(() => {});
    }
  });

  useHotkey(",", () => setShowSettings((prev) => !prev), { ctrl: true });
  useHotkey("d", () => setDebugOpen((prev) => !prev), { ctrl: true });

  const isModelReady = chatSt?.available ?? false;

  return (
    <div className="flex flex-col h-screen bg-ghost-bg rounded-2xl overflow-hidden border border-ghost-border/50 shadow-2xl">
      {/* Draggable title bar region */}
      <div
        data-tauri-drag-region
        className="h-3 shrink-0 cursor-grab active:cursor-grabbing"
      />

      {/* Header */}
      <header className="shrink-0 px-5 pb-2">
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

          <div className="flex items-center gap-2">
            {messages.length > 0 && (
              <button
                onClick={clearChat}
                className="px-2 py-0.5 rounded-md text-[10px] text-ghost-text-dim/40 hover:text-ghost-text-dim hover:bg-ghost-surface transition-all"
                title="Nueva conversación"
              >
                Nueva chat
              </button>
            )}
            <span className="text-[10px] text-ghost-text-dim/30 font-mono">
              v0.1.0
            </span>
          </div>
        </div>

        {/* Ghost Omnibox */}
        <GhostInput
          value={query}
          onChange={handleQueryChange}
          onSubmit={handleSubmit}
          mode={activeMode}
          onModeToggle={handleModeToggle}
          isSearching={isSearching}
          isGenerating={isGenerating}
          resultCount={results.length}
          modelReady={isModelReady}
        />

        {searchError && activeMode === "search" && (
          <div className="mt-2 px-4 py-2 bg-ghost-danger/10 border border-ghost-danger/20 rounded-xl text-xs text-ghost-danger">
            {searchError}
          </div>
        )}
      </header>

      {/* Main content */}
      <main className="flex-1 overflow-hidden">
        {activeMode === "search" ? (
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
                isLoading={isSearching}
                hasQuery={!!query.trim()}
              />
            )}
          </div>
        ) : (
          <ChatMessages
            messages={messages}
            isGenerating={isGenerating}
            status={chatSt}
            tokensInfo={tokensInfo}
            error={chatError}
            onRetryDownload={() => chatLoadModel().catch(() => {})}
          />
        )}
      </main>

      {/* Debug Panel */}
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
