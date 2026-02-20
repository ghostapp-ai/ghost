import { useState, useEffect, useCallback } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface UpdateState {
  /** Whether we're currently checking for updates */
  checking: boolean;
  /** Whether an update is available */
  available: boolean;
  /** Whether we're downloading/installing the update */
  downloading: boolean;
  /** Download progress percentage (0-100) */
  progress: number;
  /** New version string */
  version: string | null;
  /** Release notes body */
  releaseNotes: string | null;
  /** Error message if check or install failed */
  error: string | null;
  /** Whether the update was installed and restart is pending */
  installed: boolean;
}

const initialState: UpdateState = {
  checking: false,
  available: false,
  downloading: false,
  progress: 0,
  version: null,
  releaseNotes: null,
  error: null,
  installed: false,
};

/**
 * Hook for managing in-app auto-updates via Tauri updater plugin.
 *
 * On mount, checks for updates automatically (silent). Provides
 * manual `checkForUpdate()` and `installUpdate()` callbacks.
 */
export function useUpdater(autoCheck = true) {
  const [state, setState] = useState<UpdateState>(initialState);
  const [pendingUpdate, setPendingUpdate] = useState<Update | null>(null);

  const checkForUpdate = useCallback(async () => {
    setState((s) => ({ ...s, checking: true, error: null }));
    try {
      const update = await check();
      if (update?.available) {
        setPendingUpdate(update);
        setState((s) => ({
          ...s,
          checking: false,
          available: true,
          version: update.version,
          releaseNotes: update.body ?? null,
        }));
      } else {
        setState((s) => ({
          ...s,
          checking: false,
          available: false,
          version: null,
          releaseNotes: null,
        }));
      }
    } catch (err) {
      setState((s) => ({
        ...s,
        checking: false,
        error: String(err),
      }));
    }
  }, []);

  const installUpdate = useCallback(async () => {
    if (!pendingUpdate) return;
    setState((s) => ({ ...s, downloading: true, error: null, progress: 0 }));
    try {
      let totalBytes = 0;
      let downloadedBytes = 0;
      await pendingUpdate.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            totalBytes = event.data.contentLength ?? 0;
            break;
          case "Progress":
            downloadedBytes += event.data.chunkLength;
            if (totalBytes > 0) {
              setState((s) => ({
                ...s,
                progress: Math.round((downloadedBytes / totalBytes) * 100),
              }));
            }
            break;
          case "Finished":
            setState((s) => ({ ...s, progress: 100, installed: true, downloading: false }));
            break;
        }
      });
      // Auto-restart after install
      await relaunch();
    } catch (err) {
      setState((s) => ({
        ...s,
        downloading: false,
        error: String(err),
      }));
    }
  }, [pendingUpdate]);

  const dismiss = useCallback(() => {
    setState(initialState);
    setPendingUpdate(null);
  }, []);

  // Auto-check on mount (silent — no error UI for background check)
  useEffect(() => {
    if (autoCheck) {
      checkForUpdate().catch(() => {
        // Silent fail on auto-check — user can manually check later
      });
    }
  }, [autoCheck, checkForUpdate]);

  return {
    ...state,
    checkForUpdate,
    installUpdate,
    dismiss,
  };
}
