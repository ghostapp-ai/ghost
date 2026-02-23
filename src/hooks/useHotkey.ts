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
      // When modifier is required (true), check it's pressed.
      // When modifier is NOT required (false/undefined), check it's NOT pressed.
      // This prevents plain "Escape" from firing on "Ctrl+Escape".
      const matchesCtrl = modifiers.ctrl ? (e.ctrlKey || e.metaKey) : !(e.ctrlKey || e.metaKey);
      const matchesShift = modifiers.shift ? e.shiftKey : !e.shiftKey;
      const matchesAlt = modifiers.alt ? e.altKey : !e.altKey;

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
