import { Download, Loader2, CheckCircle2, HardDrive } from "lucide-react";
import type { DownloadProgress } from "../lib/types";

interface DownloadProgressBarProps {
  progress: DownloadProgress;
  modelName: string;
}

export function DownloadProgressBar({
  progress,
  modelName,
}: DownloadProgressBarProps) {
  const { downloaded_bytes, total_bytes, phase } = progress;

  // Calculate percentage (avoid division by zero)
  const pct =
    total_bytes > 0
      ? Math.min((downloaded_bytes / total_bytes) * 100, 100)
      : 0;

  const downloadedMB = (downloaded_bytes / 1_048_576).toFixed(0);
  const totalMB = (total_bytes / 1_048_576).toFixed(0);

  const phaseLabel = (() => {
    switch (phase) {
      case "checking_cache":
        return "Verificando cache...";
      case "downloading":
        return `Descargando ${modelName}`;
      case "download_complete":
        return "Descarga completa";
      case "loading_model":
        return "Cargando modelo en memoria...";
      case "cached":
        return "Usando cache local";
      default:
        return "Preparando...";
    }
  })();

  const PhaseIcon = (() => {
    switch (phase) {
      case "checking_cache":
      case "cached":
        return HardDrive;
      case "downloading":
        return Download;
      case "download_complete":
        return CheckCircle2;
      case "loading_model":
        return Loader2;
      default:
        return Download;
    }
  })();

  const isAnimating = phase === "downloading" || phase === "loading_model" || phase === "checking_cache";

  return (
    <div className="mx-4 my-3 p-3.5 rounded-xl bg-ghost-surface border border-ghost-border/60 space-y-2.5 animate-in fade-in slide-in-from-bottom-1 duration-300">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <PhaseIcon
            className={`w-3.5 h-3.5 ${
              phase === "download_complete"
                ? "text-ghost-success"
                : "text-ghost-accent"
            } ${isAnimating && phase === "loading_model" ? "animate-spin" : ""}`}
          />
          <span className="text-xs font-medium text-ghost-text/80">
            {phaseLabel}
          </span>
        </div>

        {phase === "downloading" && (
          <span className="text-[10px] text-ghost-text-dim/50 tabular-nums font-mono">
            {downloadedMB} / {totalMB} MB
          </span>
        )}
      </div>

      {/* Progress bar */}
      {(phase === "downloading" || phase === "loading_model") && (
        <div className="relative h-1.5 rounded-full bg-ghost-border/40 overflow-hidden">
          <div
            className={`absolute top-0 left-0 h-full rounded-full transition-all duration-500 ease-out ${
              phase === "loading_model"
                ? "bg-ghost-accent/60 animate-pulse w-full"
                : "bg-gradient-to-r from-ghost-accent to-ghost-accent-dim"
            }`}
            style={
              phase === "downloading"
                ? { width: `${Math.max(pct, 1)}%` }
                : undefined
            }
          />

          {/* Shimmer effect on active download */}
          {phase === "downloading" && (
            <div
              className="absolute top-0 left-0 h-full w-full"
              style={{
                background:
                  "linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.08) 50%, transparent 100%)",
                animation: "shimmer 2s infinite",
              }}
            />
          )}
        </div>
      )}

      {/* Percentage */}
      {phase === "downloading" && (
        <div className="flex items-center justify-between text-[10px] text-ghost-text-dim/40">
          <span>Primera vez — se guarda en cache local</span>
          <span className="tabular-nums font-mono font-medium text-ghost-accent/70">
            {pct.toFixed(0)}%
          </span>
        </div>
      )}
    </div>
  );
}
