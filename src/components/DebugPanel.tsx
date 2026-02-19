import { useState, useEffect, useRef, useCallback } from "react";
import {
  Terminal,
  Trash2,
  ChevronDown,
  ChevronUp,
  Pause,
  Play,
} from "lucide-react";
import { getLogs, clearLogs } from "../lib/tauri";
import type { LogEntry } from "../lib/types";

interface DebugPanelProps {
  isOpen: boolean;
  onToggle: () => void;
}

export function DebugPanel({ isOpen, onToggle }: DebugPanelProps) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [paused, setPaused] = useState(false);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Poll for new logs
  useEffect(() => {
    if (!isOpen || paused) return;

    const refresh = async () => {
      try {
        const newLogs = await getLogs(0);
        setLogs(newLogs);
      } catch {
        // Silently handle
      }
    };

    refresh();
    const interval = setInterval(refresh, 2000);
    return () => clearInterval(interval);
  }, [isOpen, paused]);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (!paused) {
      logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, paused]);

  const handleClear = useCallback(async () => {
    await clearLogs().catch(() => {});
    setLogs([]);
  }, []);

  if (!isOpen) {
    return (
      <button
        onClick={onToggle}
        className="flex items-center gap-1.5 px-3 py-1 text-[10px] text-ghost-text-dim/40 hover:text-ghost-text-dim/60 transition-colors"
        title="Open debug panel"
      >
        <Terminal className="w-3 h-3" />
        <span>Debug</span>
        <ChevronUp className="w-3 h-3" />
      </button>
    );
  }

  return (
    <div className="border-t border-ghost-border bg-ghost-bg">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-ghost-border/50">
        <button
          onClick={onToggle}
          className="flex items-center gap-1.5 text-[10px] text-ghost-text-dim/60 hover:text-ghost-text-dim transition-colors"
        >
          <Terminal className="w-3 h-3" />
          <span className="font-medium">Debug Log</span>
          <span className="text-ghost-text-dim/30">({logs.length})</span>
          <ChevronDown className="w-3 h-3" />
        </button>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setPaused(!paused)}
            className="p-1 rounded text-ghost-text-dim/40 hover:text-ghost-text-dim transition-colors"
            title={paused ? "Resume" : "Pause"}
          >
            {paused ? (
              <Play className="w-3 h-3" />
            ) : (
              <Pause className="w-3 h-3" />
            )}
          </button>
          <button
            onClick={handleClear}
            className="p-1 rounded text-ghost-text-dim/40 hover:text-ghost-danger transition-colors"
            title="Clear logs"
          >
            <Trash2 className="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* Log entries */}
      <div
        ref={containerRef}
        className="h-36 overflow-y-auto font-mono text-[10px] leading-4 px-2 py-1"
      >
        {logs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-ghost-text-dim/30">
            No logs yet
          </div>
        ) : (
          logs.map((log, i) => (
            <div key={i} className="flex gap-2 py-px hover:bg-ghost-surface-hover/30">
              <span className="text-ghost-text-dim/30 shrink-0 w-20">
                {log.timestamp}
              </span>
              <span
                className={`shrink-0 w-10 uppercase font-semibold ${
                  log.level === "error"
                    ? "text-ghost-danger"
                    : log.level === "warn"
                      ? "text-ghost-warning"
                      : "text-ghost-text-dim/40"
                }`}
              >
                {log.level}
              </span>
              <span className="text-ghost-text-dim/70 break-all">
                {log.message}
              </span>
            </div>
          ))
        )}
        <div ref={logsEndRef} />
      </div>
    </div>
  );
}
