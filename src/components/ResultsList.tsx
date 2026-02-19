import { useRef, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ResultItem } from "./ResultItem";
import { SearchIcon } from "lucide-react";
import type { SearchResult } from "../lib/types";

interface ResultsListProps {
  results: SearchResult[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  isLoading: boolean;
  hasQuery: boolean;
}

export function ResultsList({
  results,
  selectedIndex,
  onSelect,
  isLoading,
  hasQuery,
}: ResultsListProps) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: results.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 88,
    overscan: 5,
  });

  const scrollToIndex = useCallback(
    (index: number) => {
      virtualizer.scrollToIndex(index, { align: "auto" });
    },
    [virtualizer]
  );

  // Scroll when selected index changes
  if (selectedIndex >= 0 && selectedIndex < results.length) {
    scrollToIndex(selectedIndex);
  }

  if (!hasQuery) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-ghost-text-dim/50 gap-3">
        <SearchIcon className="w-10 h-10" />
        <p className="text-sm">Type to search your indexed files</p>
        <div className="flex gap-3 text-xs text-ghost-text-dim/30">
          <kbd className="px-2 py-1 rounded bg-ghost-surface border border-ghost-border">
            ↑↓
          </kbd>
          <span>navigate</span>
          <kbd className="px-2 py-1 rounded bg-ghost-surface border border-ghost-border">
            ↵
          </kbd>
          <span>open</span>
          <kbd className="px-2 py-1 rounded bg-ghost-surface border border-ghost-border">
            esc
          </kbd>
          <span>clear</span>
        </div>
      </div>
    );
  }

  if (isLoading && results.length === 0) {
    return (
      <div className="space-y-2 p-2">
        {[...Array(5)].map((_, i) => (
          <div
            key={i}
            className="h-20 rounded-xl bg-ghost-surface animate-pulse"
            style={{ animationDelay: `${i * 75}ms` }}
          />
        ))}
      </div>
    );
  }

  if (results.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-48 text-ghost-text-dim/50 gap-2">
        <p className="text-sm">No results found</p>
        <p className="text-xs">Try a different query or index more files</p>
      </div>
    );
  }

  return (
    <div
      ref={parentRef}
      className="flex-1 overflow-auto"
      role="listbox"
      aria-label="Search results"
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => (
          <div
            key={virtualItem.key}
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: "100%",
              height: `${virtualItem.size}px`,
              transform: `translateY(${virtualItem.start}px)`,
            }}
          >
            <ResultItem
              result={results[virtualItem.index]}
              isSelected={virtualItem.index === selectedIndex}
              onSelect={() => onSelect(virtualItem.index)}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
