import { useRef, useEffect } from "react";
import { Search, Loader2, X } from "lucide-react";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  isLoading: boolean;
  resultCount: number;
  autoFocus?: boolean;
}

export function SearchBar({
  value,
  onChange,
  isLoading,
  resultCount,
  autoFocus = true,
}: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (autoFocus && inputRef.current) {
      inputRef.current.focus();
    }
  }, [autoFocus]);

  return (
    <div className="relative group">
      <div className="flex items-center gap-3 px-5 py-4 bg-ghost-surface border border-ghost-border rounded-2xl transition-all duration-200 focus-within:border-ghost-accent/50 focus-within:shadow-[0_0_20px_rgba(108,92,231,0.1)]">
        {isLoading ? (
          <Loader2 className="w-5 h-5 text-ghost-accent animate-spin shrink-0" />
        ) : (
          <Search className="w-5 h-5 text-ghost-text-dim shrink-0 transition-colors group-focus-within:text-ghost-accent" />
        )}

        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="Search your files..."
          className="flex-1 bg-transparent text-ghost-text text-lg outline-none placeholder:text-ghost-text-dim/50"
          spellCheck={false}
          aria-label="Search query"
        />

        {value && (
          <button
            onClick={() => onChange("")}
            className="p-1 rounded-lg text-ghost-text-dim hover:text-ghost-text hover:bg-ghost-surface-hover transition-all"
            aria-label="Clear search"
          >
            <X className="w-4 h-4" />
          </button>
        )}

        {value && !isLoading && (
          <span className="text-xs text-ghost-text-dim tabular-nums shrink-0">
            {resultCount} result{resultCount !== 1 ? "s" : ""}
          </span>
        )}
      </div>
    </div>
  );
}
