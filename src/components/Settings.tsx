import { useState, useCallback } from "react";
import {
  FolderPlus,
  FolderMinus,
  Play,
  X,
  Loader2,
} from "lucide-react";
import { indexDirectory } from "../lib/tauri";
import type { IndexStats } from "../lib/types";

interface SettingsProps {
  onClose: () => void;
}

export function Settings({ onClose }: SettingsProps) {
  const [directories, setDirectories] = useState<string[]>([]);
  const [newDir, setNewDir] = useState("");
  const [indexing, setIndexing] = useState(false);
  const [lastResult, setLastResult] = useState<IndexStats | null>(null);
  const [error, setError] = useState<string | null>(null);

  const addDirectory = useCallback(() => {
    const trimmed = newDir.trim();
    if (trimmed && !directories.includes(trimmed)) {
      setDirectories((prev) => [...prev, trimmed]);
      setNewDir("");
    }
  }, [newDir, directories]);

  const removeDirectory = useCallback((dir: string) => {
    setDirectories((prev) => prev.filter((d) => d !== dir));
  }, []);

  const startIndexing = useCallback(async () => {
    if (directories.length === 0) return;
    setIndexing(true);
    setError(null);
    setLastResult(null);

    let totalStats: IndexStats = { total: 0, indexed: 0, failed: 0 };

    for (const dir of directories) {
      try {
        const stats = await indexDirectory(dir);
        totalStats.total += stats.total;
        totalStats.indexed += stats.indexed;
        totalStats.failed += stats.failed;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    }

    setLastResult(totalStats);
    setIndexing(false);
  }, [directories]);

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-ghost-surface border border-ghost-border rounded-2xl w-full max-w-lg shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-ghost-border">
          <h2 className="text-lg font-semibold text-ghost-text">Settings</h2>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-ghost-text-dim hover:text-ghost-text hover:bg-ghost-surface-hover transition-all"
            aria-label="Close settings"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Watched Directories */}
          <div>
            <label className="text-sm font-medium text-ghost-text-dim mb-3 block">
              Watched Directories
            </label>

            <div className="flex gap-2 mb-3">
              <input
                type="text"
                value={newDir}
                onChange={(e) => setNewDir(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addDirectory()}
                placeholder="/home/user/Documents"
                className="flex-1 px-3 py-2 bg-ghost-bg border border-ghost-border rounded-lg text-sm text-ghost-text placeholder:text-ghost-text-dim/40 outline-none focus:border-ghost-accent/50"
              />
              <button
                onClick={addDirectory}
                disabled={!newDir.trim()}
                className="px-3 py-2 bg-ghost-accent/20 text-ghost-accent rounded-lg text-sm font-medium hover:bg-ghost-accent/30 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
              >
                <FolderPlus className="w-4 h-4" />
              </button>
            </div>

            {directories.length === 0 ? (
              <p className="text-xs text-ghost-text-dim/40 text-center py-4">
                No directories added. Add a directory to start indexing.
              </p>
            ) : (
              <div className="space-y-1.5">
                {directories.map((dir) => (
                  <div
                    key={dir}
                    className="flex items-center justify-between px-3 py-2 bg-ghost-bg rounded-lg border border-ghost-border"
                  >
                    <span className="text-sm text-ghost-text truncate">
                      {dir}
                    </span>
                    <button
                      onClick={() => removeDirectory(dir)}
                      className="p-1 rounded text-ghost-text-dim hover:text-ghost-danger transition-colors shrink-0"
                    >
                      <FolderMinus className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Index Button */}
          <button
            onClick={startIndexing}
            disabled={indexing || directories.length === 0}
            className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-ghost-accent text-white rounded-xl font-medium text-sm hover:bg-ghost-accent-dim disabled:opacity-40 disabled:cursor-not-allowed transition-all"
          >
            {indexing ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                Indexing...
              </>
            ) : (
              <>
                <Play className="w-4 h-4" />
                Index Now
              </>
            )}
          </button>

          {/* Results */}
          {lastResult && (
            <div className="px-4 py-3 bg-ghost-success/10 border border-ghost-success/20 rounded-xl text-sm text-ghost-success">
              Indexed {lastResult.indexed} files ({lastResult.failed} failed) out
              of {lastResult.total} total.
            </div>
          )}

          {error && (
            <div className="px-4 py-3 bg-ghost-danger/10 border border-ghost-danger/20 rounded-xl text-sm text-ghost-danger">
              {error}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
