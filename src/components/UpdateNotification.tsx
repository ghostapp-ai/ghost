import { Download, RefreshCw, X, CheckCircle, AlertCircle, Loader2 } from "lucide-react";
import type { UpdateState } from "../hooks/useUpdater";

interface UpdateNotificationProps {
  state: UpdateState;
  onInstall: () => void;
  onDismiss: () => void;
}

/**
 * Floating notification banner for app updates.
 * Shows when an update is available, downloading, or installed.
 */
export function UpdateNotification({ state, onInstall, onDismiss }: UpdateNotificationProps) {
  if (!state.available && !state.downloading && !state.installed && !state.error) {
    return null;
  }

  return (
    <div className="fixed bottom-16 left-1/2 -translate-x-1/2 z-50 w-[calc(100%-2rem)] max-w-md">
      <div className="bg-[#1a1a2e]/95 backdrop-blur-xl border border-purple-500/30 rounded-xl p-4 shadow-2xl shadow-purple-900/20">
        {/* Close button */}
        {!state.downloading && (
          <button
            type="button"
            onClick={onDismiss}
            className="absolute top-2 right-2 p-1 text-white/40 hover:text-white/80 transition-colors"
            aria-label="Cerrar notificación"
          >
            <X size={14} />
          </button>
        )}

        {/* Error state */}
        {state.error && (
          <div className="flex items-center gap-3">
            <AlertCircle size={20} className="text-red-400 shrink-0" />
            <div className="min-w-0">
              <p className="text-sm font-medium text-white/90">
                Error al verificar actualización
              </p>
              <p className="text-xs text-white/50 truncate mt-0.5">{state.error}</p>
            </div>
          </div>
        )}

        {/* Update available */}
        {state.available && !state.downloading && !state.installed && !state.error && (
          <div className="flex items-center gap-3">
            <Download size={20} className="text-purple-400 shrink-0" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-white/90">
                Ghost {state.version} disponible
              </p>
              {state.releaseNotes && (
                <p className="text-xs text-white/50 line-clamp-2 mt-0.5">
                  {state.releaseNotes}
                </p>
              )}
            </div>
            <button
              type="button"
              onClick={onInstall}
              className="shrink-0 px-3 py-1.5 bg-purple-600 hover:bg-purple-500 text-white text-xs font-medium rounded-lg transition-colors"
            >
              Actualizar
            </button>
          </div>
        )}

        {/* Downloading */}
        {state.downloading && (
          <div className="space-y-2">
            <div className="flex items-center gap-3">
              <Loader2 size={20} className="text-purple-400 shrink-0 animate-spin" />
              <p className="text-sm font-medium text-white/90">
                Descargando actualización…
              </p>
              <span className="text-xs text-white/50 ml-auto">{state.progress}%</span>
            </div>
            <div className="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
              <div
                className="h-full bg-linear-to-r from-purple-500 to-indigo-500 rounded-full transition-all duration-300"
                style={{ width: `${state.progress}%` }}
              />
            </div>
          </div>
        )}

        {/* Installed (restarting) */}
        {state.installed && !state.downloading && (
          <div className="flex items-center gap-3">
            <CheckCircle size={20} className="text-green-400 shrink-0" />
            <p className="text-sm font-medium text-white/90">
              Actualización instalada. Reiniciando…
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

interface CheckUpdateButtonProps {
  checking: boolean;
  onCheck: () => void;
}

/**
 * Button for Settings panel to manually check for updates.
 */
export function CheckUpdateButton({ checking, onCheck }: CheckUpdateButtonProps) {
  return (
    <button
      type="button"
      onClick={onCheck}
      disabled={checking}
      className="flex items-center gap-2 px-4 py-2 bg-white/5 hover:bg-white/10 border border-white/10 rounded-lg text-sm text-white/80 transition-colors disabled:opacity-50"
    >
      <RefreshCw size={14} className={checking ? "animate-spin" : ""} />
      {checking ? "Verificando…" : "Buscar actualizaciones"}
    </button>
  );
}
