import { useState, useCallback, useRef, useEffect } from "react";
import { search } from "../lib/tauri";
import type { SearchResult } from "../lib/types";

/** Hook for debounced search with loading state. */
export function useSearch(debounceMs = 150) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const abortRef = useRef(0);

  const doSearch = useCallback(
    async (q: string) => {
      if (!q.trim()) {
        setResults([]);
        setIsLoading(false);
        return;
      }

      const searchId = ++abortRef.current;
      setIsLoading(true);
      setError(null);

      try {
        const res = await search(q, 20);
        // Only update if this is still the latest search
        if (searchId === abortRef.current) {
          setResults(res);
        }
      } catch (e) {
        if (searchId === abortRef.current) {
          setError(e instanceof Error ? e.message : String(e));
          setResults([]);
        }
      } finally {
        if (searchId === abortRef.current) {
          setIsLoading(false);
        }
      }
    },
    []
  );

  const updateQuery = useCallback(
    (q: string) => {
      setQuery(q);
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => doSearch(q), debounceMs);
    },
    [doSearch, debounceMs]
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  return { query, setQuery: updateQuery, results, isLoading, error };
}
