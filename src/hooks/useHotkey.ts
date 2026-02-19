import { useEffect, useCallback } from "react";

type KeyHandler = (e: KeyboardEvent) => void;

/** Hook for registering global keyboard shortcuts. */
export function useHotkey(
  key: string,
  handler: KeyHandler,
  modifiers: { ctrl?: boolean; shift?: boolean; alt?: boolean; meta?: boolean } = {}
) {
  const callback = useCallback(
    (e: KeyboardEvent) => {
      const matchesKey = e.key.toLowerCase() === key.toLowerCase();
      const matchesCtrl = modifiers.ctrl ? e.ctrlKey || e.metaKey : true;
      const matchesShift = modifiers.shift ? e.shiftKey : true;
      const matchesAlt = modifiers.alt ? e.altKey : true;

      if (matchesKey && matchesCtrl && matchesShift && matchesAlt) {
        e.preventDefault();
        handler(e);
      }
    },
    [key, handler, modifiers.ctrl, modifiers.shift, modifiers.alt]
  );

  useEffect(() => {
    window.addEventListener("keydown", callback);
    return () => window.removeEventListener("keydown", callback);
  }, [callback]);
}
