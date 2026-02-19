import { useState, useCallback } from "react";
import { SearchBar } from "./components/SearchBar";
import { ResultsList } from "./components/ResultsList";
import { StatusBar } from "./components/StatusBar";
import { Settings } from "./components/Settings";
import { useSearch } from "./hooks/useSearch";
import { useHotkey } from "./hooks/useHotkey";
import "./styles/globals.css";

export default function App() {
  const { query, setQuery, results, isLoading, error } = useSearch(150);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showSettings, setShowSettings] = useState(false);

  // Reset selection when results change
  const handleQueryChange = useCallback(
    (q: string) => {
      setQuery(q);
      setSelectedIndex(0);
    },
    [setQuery]
  );

  // Keyboard navigation
  useHotkey("ArrowDown", () => {
    setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
  });

  useHotkey("ArrowUp", () => {
    setSelectedIndex((prev) => Math.max(prev - 1, 0));
  });

  useHotkey("Escape", () => {
    if (showSettings) {
      setShowSettings(false);
    } else if (query) {
      handleQueryChange("");
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
    <div className="flex flex-col h-screen bg-ghost-bg">
      {/* Header */}
      <header className="shrink-0 px-5 pt-5 pb-3">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 rounded-lg bg-ghost-accent/20 flex items-center justify-center">
              <span className="text-ghost-accent text-sm font-bold">G</span>
            </div>
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
        <ResultsList
          results={results}
          selectedIndex={selectedIndex}
          onSelect={setSelectedIndex}
          isLoading={isLoading}
          hasQuery={!!query.trim()}
        />
      </main>

      {/* Status Bar */}
      <StatusBar onSettingsClick={() => setShowSettings(true)} />

      {/* Settings Modal */}
      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </div>
  );
}
