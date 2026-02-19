import { useState, useEffect } from "react";
import {
  Database,
  FileText,
  Layers,
  CircleCheck,
  CircleX,
  ChevronRight,
  Zap,
} from "lucide-react";
import { getStats, checkAiStatus, getVecStatus } from "../lib/tauri";
import type { DbStats, AiStatus } from "../lib/types";

interface StatusBarProps {
  onSettingsClick: () => void;
}

export function StatusBar({ onSettingsClick }: StatusBarProps) {
  const [stats, setStats] = useState<DbStats | null>(null);
  const [aiStatus, setAiStatus] = useState<AiStatus | null>(null);
  const [vecOk, setVecOk] = useState(false);

  useEffect(() => {
    async function refresh() {
      try {
        const [s, ai, v] = await Promise.all([
          getStats(),
          checkAiStatus(),
          getVecStatus(),
        ]);
        setStats(s);
        setAiStatus(ai);
        setVecOk(v);
      } catch {
        // Silently handle — status bar is informational
      }
    }
    refresh();
    const interval = setInterval(refresh, 10_000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex items-center justify-between px-4 py-2 border-t border-ghost-border text-[11px] text-ghost-text-dim/60">
      <div className="flex items-center gap-4">
        <StatusPill
          icon={<Database className="w-3 h-3" />}
          label={`${stats?.document_count ?? 0} docs`}
        />
        <StatusPill
          icon={<FileText className="w-3 h-3" />}
          label={`${stats?.chunk_count ?? 0} chunks`}
        />
        <StatusPill
          icon={<Layers className="w-3 h-3" />}
          label={`${stats?.embedded_chunk_count ?? 0} embedded`}
        />
      </div>

      <div className="flex items-center gap-4">
        <StatusIndicator
          icon={<Zap className="w-3 h-3" />}
          label={`AI: ${aiStatus?.backend === "Native" ? "Native" : aiStatus?.backend === "Ollama" ? "Ollama" : "Offline"}`}
          ok={aiStatus?.backend !== "None"}
        />
        <StatusIndicator
          icon={<Layers className="w-3 h-3" />}
          label="Vector"
          ok={vecOk}
        />
        <button
          onClick={onSettingsClick}
          className="flex items-center gap-1 text-ghost-text-dim/60 hover:text-ghost-text transition-colors"
          aria-label="Settings"
        >
          Settings
          <ChevronRight className="w-3 h-3" />
        </button>
      </div>
    </div>
  );
}

function StatusPill({
  icon,
  label,
}: {
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <span className="flex items-center gap-1.5 tabular-nums">
      {icon}
      {label}
    </span>
  );
}

function StatusIndicator({
  icon,
  label,
  ok,
}: {
  icon: React.ReactNode;
  label: string;
  ok: boolean;
}) {
  return (
    <span className="flex items-center gap-1.5">
      {icon}
      {label}
      {ok ? (
        <CircleCheck className="w-3 h-3 text-ghost-success" />
      ) : (
        <CircleX className="w-3 h-3 text-ghost-danger" />
      )}
    </span>
  );
}
