import { useState, useCallback, useEffect, memo } from "react";
import { GhostInput } from "./components/GhostInput";
import { ResultsList as _ResultsList } from "./components/ResultsList";
import { ChatMessages as _ChatMessages } from "./components/ChatMessages";
import { StatusBar as _StatusBar } from "./components/StatusBar";
import { Settings } from "./components/Settings";
import { DebugPanel as _DebugPanel } from "./components/DebugPanel";
import { Onboarding } from "./components/Onboarding";
import { useSearch } from "./hooks/useSearch";
import { useHotkey } from "./hooks/useHotkey";
import { useAgui } from "./hooks/useAgui";
import { usePlatform } from "./hooks/usePlatform";
import { useUpdater } from "./hooks/useUpdater";
import { UpdateNotification } from "./components/UpdateNotification";
import { detectMode, type InputMode } from "./lib/detectMode";

// Memoize heavy child components to prevent re-renders from parent state changes
const ResultsList = memo(_ResultsList);
const ChatMessages = memo(_ChatMessages);
const StatusBar = memo(_StatusBar);
const DebugPanel = memo(_DebugPanel);
import {
  hideWindow,
  openFile,
  getSettings,
  startWatcher,
  chatSend,
  chatStatus as fetchChatStatus,
  chatLoadModel,
  startDragging,
  minimizeWindow,
  toggleMaximizeWindow,
  closeWindow,
} from "./lib/tauri";
import type { ChatMessage, ChatStatus } from "./lib/types";
import "./styles/globals.css";

export default function App() {
  // --- Platform detection ---
  const platform = usePlatform();

  // --- Auto-updater (desktop only, silent auto-check on launch) ---
  const updater = useUpdater(platform.isDesktop);

  // --- Setup / onboarding state ---
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);

  // --- Search state ---
  const { query, setQuery, results, isLoading: isSearching, error: searchError } = useSearch(150);
  const [selectedIndex, setSelectedIndex] = useState(0);

  // --- Chat state ---
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [chatError, setChatError] = useState<string | null>(null);
  const [tokensInfo, setTokensInfo] = useState<string | null>(null);
  const [chatSt, setChatSt] = useState<ChatStatus | null>(null);

  // --- AG-UI streaming ---
  const { runState, isStreaming, sendStreaming, reset: resetAgui } = useAgui();

  // Sync AG-UI streaming state → message list when run finishes
  useEffect(() => {
    if (runState?.status === "finished" && runState.content) {
      const assistantMsg: ChatMessage = {
        role: "assistant",
        content: runState.content,
      };
      setMessages((prev) => [...prev, assistantMsg]);
      setIsGenerating(false);
      if (runState.metadata) {
        const m = runState.metadata;
        const tokens = m.tokens_generated ?? 0;
        const duration = Number(m.duration_ms ?? 0) / 1000;
        const model = m.model_id ?? "";
        setTokensInfo(`${tokens} tokens · ${duration.toFixed(1)}s · ${model}`);
      }
    } else if (runState?.status === "error") {
      setChatError(runState.error ?? "Unknown error");
      setIsGenerating(false);
    }
  }, [runState?.status, runState?.content, runState?.error, runState?.metadata]);

  // --- UI state ---
  const [mode, setMode] = useState<InputMode>("search");
  const [modeOverride, setModeOverride] = useState<InputMode | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [hasDirectories, setHasDirectories] = useState<boolean | null>(null);
  const [debugOpen, setDebugOpen] = useState(false);

  // Effective mode: manual override wins, then auto-detected
  const activeMode = modeOverride ?? mode;

  // --- Smart adaptive chat status polling ---
  // Polls quickly during loading (2s), slows to 10s when stable, stops when available.
  // Immediate refresh triggered by user actions (sendStreaming, chatLoadModel).
  const refreshChatStatus = useCallback(() => {
    fetchChatStatus().then(setChatSt).catch(() => {});
  }, []);

  useEffect(() => {
    refreshChatStatus();
    // Adaptive interval: fast during loading, slow when idle
    const getInterval = () => {
      if (chatSt?.loading) return 2000;     // Loading → poll fast
      if (chatSt?.available) return 30000;  // Ready → poll rarely (health check)
      return 10000;                          // Not available → moderate poll
    };
    const id = setInterval(refreshChatStatus, getInterval());
    return () => clearInterval(id);
  }, [chatSt?.loading, chatSt?.available, refreshChatStatus]);

  // --- Auto-start watcher ---
  useEffect(() => {
    getSettings()
      .then((s) => {
        // Check if onboarding is needed
        setSetupComplete(s.setup_complete);

        const hasDirs = s.watched_directories.length > 0;
        setHasDirectories(hasDirs);
        if (hasDirs) {
          startWatcher(s.watched_directories).catch(() => {});
        } else {
          // Auto-indexing is happening in the background (Rust setup)
          // Poll settings to detect when auto-discovery finishes
          const poll = setInterval(() => {
            getSettings().then((updated) => {
              if (updated.watched_directories.length > 0) {
                setHasDirectories(true);
                startWatcher(updated.watched_directories).catch(() => {});
                clearInterval(poll);
              }
            }).catch(() => {});
          }, 2000);
          // Clean up after 60 seconds max
          setTimeout(() => clearInterval(poll), 60000);
        }
      })
      .catch(() => {
        setSetupComplete(true); // Default to showing app on error
        setHasDirectories(false);
      });
  }, []);

  // --- Auto-hide on blur (desktop only, with startup grace period) ---
  const [blurEnabled, setBlurEnabled] = useState(false);
  useEffect(() => {
    // Only enable auto-hide on desktop — mobile apps don't hide on blur
    if (!platform.isDesktop) return;
    // Give the app 2 seconds to stabilize before enabling auto-hide.
    // This prevents the window from disappearing immediately on startup
    // when focus hasn't been established yet (common on Linux/WSL2).
    const timer = setTimeout(() => setBlurEnabled(true), 2000);
    return () => clearTimeout(timer);
  }, [platform.isDesktop]);

  useEffect(() => {
    if (!blurEnabled || !platform.isDesktop) return;
    const handleBlur = () => {
      if (!showSettings) hideWindow().catch(() => {});
    };
    window.addEventListener("blur", handleBlur);
    return () => window.removeEventListener("blur", handleBlur);
  }, [showSettings, blurEnabled, platform.isDesktop]);

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
        // Only hide window on desktop (spotlight-style)
        if (platform.isDesktop) {
          hideWindow().catch(() => {});
        }
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
        // Use AG-UI streaming when chat model is available natively
        if (chatSt?.backend === "native" || chatSt?.backend === "ollama") {
          await sendStreaming(newMessages);
          // Response will arrive via AG-UI events → useEffect sync above
        } else {
          // Fallback: non-streaming chat for compatibility
          const response = await chatSend(newMessages);
          const assistantMsg: ChatMessage = {
            role: "assistant",
            content: response.content,
          };
          setMessages([...newMessages, assistantMsg]);
          setTokensInfo(
            `${response.tokens_generated} tokens · ${(response.duration_ms / 1000).toFixed(1)}s · ${response.model_id}`
          );
          setIsGenerating(false);
        }
      } catch (e) {
        setChatError(e instanceof Error ? e.message : String(e));
        setIsGenerating(false);
      }
    }
  }, [activeMode, results, selectedIndex, query, isGenerating, messages, setQuery, chatSt?.backend, sendStreaming]);

  // --- Clear chat ---
  const clearChat = useCallback(() => {
    setMessages([]);
    setChatError(null);
    setTokensInfo(null);
    resetAgui();
  }, [resetAgui]);

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
    } else if (platform.isDesktop) {
      hideWindow().catch(() => {});
    }
  });

  useHotkey(",", () => setShowSettings((prev) => !prev), { ctrl: true });
  useHotkey("d", () => setDebugOpen((prev) => !prev), { ctrl: true });

  const isModelReady = chatSt?.available ?? false;

  // Loading state — waiting for settings to load
  if (setupComplete === null) {
    return (
      <div className="flex flex-col h-dvh bg-ghost-bg rounded-2xl overflow-hidden border border-ghost-border/50 shadow-2xl items-center justify-center">
        <div className="w-3 h-3 rounded-full bg-ghost-accent animate-pulse" />
      </div>
    );
  }

  // First launch — show onboarding
  if (!setupComplete) {
    return (
      <Onboarding
        onComplete={() => setSetupComplete(true)}
        isMobile={platform.isMobile}
      />
    );
  }

  return (
    <div className="flex flex-col h-dvh bg-ghost-bg overflow-hidden md:rounded-2xl md:border md:border-ghost-border/50 md:shadow-2xl">
      {/* Custom titlebar with drag region + window controls — desktop only */}
      {platform.isDesktop && (
        <div
          className="flex items-center justify-between shrink-0 h-9 select-none"
          onMouseDown={(e) => {
            const target = e.target as HTMLElement;
            // Only initiate drag on left-click, not on buttons/interactive elements
            if (e.button === 0 && !target.closest('button') && !target.closest('a') && !target.closest('input')) {
              e.preventDefault(); // Prevent focus toggle cycle that causes minimize on Windows
              startDragging().catch(() => {});
            }
          }}
          onDoubleClick={(e) => {
            const target = e.target as HTMLElement;
            if (!target.closest('button')) {
              toggleMaximizeWindow().catch(() => {});
            }
          }}
        >
          {/* Left spacer for drag area */}
          <div className="flex-1" />
          {/* Window control buttons */}
          <div className="flex items-center h-full">
            <button
              onClick={() => minimizeWindow().catch(() => {})}
              className="inline-flex items-center justify-center w-11 h-full text-ghost-text-dim/60 hover:text-ghost-text hover:bg-ghost-surface-hover transition-colors"
              title="Minimizar"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24">
                <path fill="currentColor" d="M19 13H5v-2h14z" />
              </svg>
            </button>
            <button
              onClick={() => toggleMaximizeWindow().catch(() => {})}
              className="inline-flex items-center justify-center w-11 h-full text-ghost-text-dim/60 hover:text-ghost-text hover:bg-ghost-surface-hover transition-colors"
              title="Maximizar"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24">
                <path fill="currentColor" d="M4 4h16v16H4zm2 4v10h12V8z" />
              </svg>
            </button>
            <button
              onClick={() => closeWindow().catch(() => {})}
              className="inline-flex items-center justify-center w-11 h-full text-ghost-text-dim/60 hover:text-ghost-text hover:bg-ghost-danger/80 transition-colors rounded-tr-2xl"
              title="Cerrar"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24">
                <path fill="currentColor" d="M13.46 12L19 17.54V19h-1.46L12 13.46L6.46 19H5v-1.46L10.54 12L5 6.46V5h1.46L12 10.54L17.54 5H19v1.46z" />
              </svg>
            </button>
          </div>
        </div>
      )}

      {/* Header */}
      <header
        className="shrink-0 px-5 pb-2 pt-safe"
      >
        <div
          className="flex items-center justify-between mb-3"
        >
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
              v{__APP_VERSION__}
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
          isMobile={platform.isMobile}
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
                  <span className="text-2xl">🔍</span>
                </div>
                <div className="text-center space-y-1">
                  <p className="text-sm font-medium text-ghost-text">
                    Bienvenido a Ghost
                  </p>
                  <p className="text-xs text-ghost-text-dim/50 max-w-70">
                    Indexando tus archivos automáticamente...
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-ghost-accent animate-pulse" />
                  <span className="text-xs text-ghost-text-dim/40">
                    Descubriendo directorios
                  </span>
                </div>
              </div>
            ) : hasDirectories === null && !query.trim() ? (
              <div className="flex flex-col items-center justify-center h-64 text-ghost-text-dim/60 gap-3">
                <div className="w-3 h-3 rounded-full bg-ghost-accent animate-pulse" />
                <p className="text-xs text-ghost-text-dim/40">Cargando...</p>
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
                    if (platform.isDesktop) {
                      hideWindow().catch(() => {});
                    }
                  }
                }}
                isLoading={isSearching}
                hasQuery={!!query.trim()}
                isMobile={platform.isMobile}
              />
            )}
          </div>
        ) : (
          <ChatMessages
            messages={messages}
            isGenerating={isGenerating}
            streamingContent={isStreaming ? (runState?.content ?? "") : undefined}
            status={chatSt}
            tokensInfo={tokensInfo}
            error={chatError}
            onRetryDownload={() => chatLoadModel().then(refreshChatStatus).catch(() => {})}
            isMobile={platform.isMobile}
            a2uiSurfaces={runState?.a2uiSurfaces}
          />
        )}
      </main>

      {/* Debug Panel — desktop only */}
      {platform.isDesktop && (
        <DebugPanel isOpen={debugOpen} onToggle={() => setDebugOpen(!debugOpen)} />
      )}

      {/* Status Bar */}
      <StatusBar onSettingsClick={() => setShowSettings(true)} compact={platform.isMobile} />

      {/* Update notification */}
      {platform.isDesktop && (
        <UpdateNotification
          state={updater}
          onInstall={updater.installUpdate}
          onDismiss={updater.dismiss}
        />
      )}

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
