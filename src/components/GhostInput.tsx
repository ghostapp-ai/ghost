import { useRef, useEffect, useCallback } from "react";
import { Search, MessageCircle, Loader2, X, ArrowUpDown } from "lucide-react";
import type { InputMode } from "../lib/detectMode";

interface GhostInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  mode: InputMode;
  onModeToggle: () => void;
  isSearching: boolean;
  isGenerating: boolean;
  resultCount: number;
  modelReady: boolean;
  autoFocus?: boolean;
}

export function GhostInput({
  value,
  onChange,
  onSubmit,
  mode,
  onModeToggle,
  isSearching,
  isGenerating,
  resultCount,
  modelReady,
  autoFocus = true,
}: GhostInputProps) {
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (autoFocus && inputRef.current) {
      inputRef.current.focus();
    }
  }, [autoFocus]);

  // Auto-resize textarea
  const handleInput = useCallback((e: React.FormEvent<HTMLTextAreaElement>) => {
    const el = e.target as HTMLTextAreaElement;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 120) + "px";
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        onSubmit();
      }
    },
    [onSubmit]
  );

  const isLoading = mode === "search" ? isSearching : isGenerating;

  const placeholder =
    mode === "chat"
      ? modelReady
        ? "Pregunta a Ghost..."
        : "Modelo cargando..."
      : "Busca archivos...";

  return (
    <div className="relative group">
      <div className="flex items-start gap-2.5 px-4 py-3 bg-ghost-surface border border-ghost-border rounded-2xl transition-all duration-200 focus-within:border-ghost-accent/50 focus-within:shadow-[0_0_20px_rgba(108,92,231,0.1)]">
        {/* Mode indicator button */}
        <button
          onClick={onModeToggle}
          className="shrink-0 mt-0.5 p-1.5 rounded-lg transition-all hover:bg-ghost-surface-hover"
          title={`Modo: ${mode === "search" ? "Búsqueda" : "Chat"} (click para cambiar)`}
          aria-label="Toggle mode"
        >
          {isLoading ? (
            <Loader2 className="w-4.5 h-4.5 text-ghost-accent animate-spin" />
          ) : mode === "search" ? (
            <Search className="w-4.5 h-4.5 text-ghost-text-dim transition-colors group-focus-within:text-ghost-accent" />
          ) : (
            <MessageCircle className="w-4.5 h-4.5 text-ghost-accent" />
          )}
        </button>

        {/* Input */}
        <textarea
          ref={inputRef}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          onInput={handleInput}
          placeholder={placeholder}
          rows={1}
          className="flex-1 bg-transparent text-ghost-text text-[15px] leading-relaxed outline-none placeholder:text-ghost-text-dim/40 resize-none"
          style={{ minHeight: "1.5rem", maxHeight: "120px" }}
          spellCheck={false}
          aria-label="Ghost input"
          disabled={mode === "chat" && !modelReady && !isGenerating}
        />

        {/* Right side controls */}
        <div className="flex items-center gap-1.5 shrink-0 mt-0.5">
          {/* Clear button */}
          {value && (
            <button
              onClick={() => onChange("")}
              className="p-1 rounded-lg text-ghost-text-dim/40 hover:text-ghost-text hover:bg-ghost-surface-hover transition-all"
              aria-label="Clear"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}

          {/* Result count (search mode) */}
          {mode === "search" && value && !isSearching && (
            <span className="text-[10px] text-ghost-text-dim/40 tabular-nums whitespace-nowrap">
              {resultCount}
            </span>
          )}

          {/* Mode badge */}
          <span
            className={`px-1.5 py-0.5 rounded text-[9px] font-medium uppercase tracking-wider transition-all ${
              mode === "chat"
                ? "bg-ghost-accent/15 text-ghost-accent"
                : "bg-ghost-surface-hover text-ghost-text-dim/50"
            }`}
          >
            {mode === "chat" ? "chat" : "search"}
          </span>
        </div>
      </div>

      {/* Keyboard hint */}
      <div className="flex items-center justify-between px-2 mt-1.5 text-[9px] text-ghost-text-dim/25">
        <div className="flex items-center gap-2">
          <span>
            <kbd className="px-1 py-0.5 rounded bg-ghost-surface/50 border border-ghost-border/30 text-[8px]">
              ↵
            </kbd>{" "}
            {mode === "chat" ? "enviar" : "abrir"}
          </span>
          <span>
            <kbd className="px-1 py-0.5 rounded bg-ghost-surface/50 border border-ghost-border/30 text-[8px]">
              esc
            </kbd>{" "}
            limpiar
          </span>
        </div>
        <button
          onClick={onModeToggle}
          className="flex items-center gap-1 opacity-40 hover:opacity-80 transition-opacity"
        >
          <ArrowUpDown className="w-2.5 h-2.5" />
          cambiar modo
        </button>
      </div>
    </div>
  );
}
