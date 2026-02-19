import { useEffect, useState, useCallback } from "react";
import {
  getStats,
  checkAiStatus,
  getVecStatus,
  chatStatus,
} from "../lib/tauri";
import type { DbStats, AiStatus, ChatStatus } from "../lib/types";

interface StatusBarProps {
  onSettingsClick: () => void;
}

export function StatusBar({ onSettingsClick }: StatusBarProps) {
  const [stats, setStats] = useState<DbStats | null>(null);
  const [ai, setAi] = useState<AiStatus | null>(null);
  const [vecOk, setVecOk] = useState<boolean | null>(null);
  const [chat, setChat] = useState<ChatStatus | null>(null);

  const refresh = useCallback(() => {
    getStats().then(setStats).catch(() => {});
    checkAiStatus().then(setAi).catch(() => {});
    getVecStatus().then(setVecOk).catch(() => {});
    chatStatus().then(setChat).catch(() => {});
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 5000);
    return () => clearInterval(id);
  }, [refresh]);

  return (
    <footer className="shrink-0 px-4 py-2 border-t border-ghost-border/30 flex items-center gap-3 text-[10px] font-mono text-ghost-text-dim/50 select-none">
      {/* DB stats */}
      {stats && (
        <span title="Documentos / Chunks / Embeddings">
          {stats.document_count}d · {stats.chunk_count}c ·{" "}
          {stats.embedded_chunk_count}e
        </span>
      )}

      <Separator />

      {/* AI embedding engine */}
      <Pill
        ok={ai ? ai.backend !== "None" : null}
        label={ai ? `AI: ${ai.backend}` : "AI: …"}
        title={
          ai
            ? `${ai.backend} — ${ai.model_name} (${ai.dimensions}D)`
            : "Detectando motor AI…"
        }
      />

      {/* Vector search */}
      <Pill
        ok={vecOk}
        label={vecOk === null ? "Vec: …" : vecOk ? "Vec: OK" : "Vec: OFF"}
        title="sqlite-vec búsqueda vectorial"
      />

      {/* Chat model */}
      <ChatPill chat={chat} />

      {/* Spacer */}
      <div className="flex-1" />

      {/* Settings button */}
      <button
        onClick={onSettingsClick}
        className="px-2 py-0.5 rounded-md hover:bg-ghost-surface transition-colors text-ghost-text-dim/60 hover:text-ghost-text-dim"
        title="Settings (Ctrl+,)"
      >
        ⚙
      </button>
    </footer>
  );
}

function Pill({
  ok,
  label,
  title,
}: {
  ok: boolean | null;
  label: string;
  title: string;
}) {
  const color =
    ok === null
      ? "text-ghost-text-dim/40"
      : ok
      ? "text-green-400"
      : "text-ghost-text-dim/30";

  return (
    <span className={color} title={title}>
      {label}
    </span>
  );
}

function ChatPill({ chat }: { chat: ChatStatus | null }) {
  if (!chat) {
    return (
      <span className="text-ghost-text-dim/40" title="Cargando estado del chat…">
        Chat: …
      </span>
    );
  }

  if (chat.loading) {
    return (
      <span className="text-yellow-400 animate-pulse" title="Descargando/cargando modelo…">
        Chat: ⏳ {chat.model_name || "cargando…"}
      </span>
    );
  }

  if (chat.error) {
    return (
      <span className="text-red-400" title={chat.error}>
        Chat: ❌
      </span>
    );
  }

  if (chat.available) {
    const deviceLabel = chat.device !== "Cpu" ? ` (${chat.device})` : "";
    return (
      <span className="text-green-400" title={`${chat.model_name} en ${chat.device}`}>
        Chat: {chat.model_name || chat.backend}{deviceLabel}
      </span>
    );
  }

  return (
    <span className="text-ghost-text-dim/30" title="Sin modelo de chat">
      Chat: OFF
    </span>
  );
}

function Separator() {
  return <span className="text-ghost-border/40">·</span>;
}
