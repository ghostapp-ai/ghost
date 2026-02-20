import { useState, useEffect, useCallback } from "react";
import {
  Cpu,
  HardDrive,
  Download,
  CheckCircle2,
  Loader2,
  FolderSearch,
  Sparkles,
  Shield,
  Zap,
  ArrowRight,
} from "lucide-react";
import { DownloadProgressBar } from "./DownloadProgress";
import {
  getHardwareInfo,
  getRecommendedModel,
  getAvailableModels,
  chatStatus as fetchChatStatus,
  chatLoadModel,
  completeSetup,
  startDragging,
} from "../lib/tauri";
import type { HardwareInfo, ModelInfo, ChatStatus } from "../lib/types";

interface OnboardingProps {
  onComplete: () => void;
  /** Mobile layout: no drag region, safe-area padding */
  isMobile?: boolean;
}

type SetupPhase =
  | "welcome"
  | "detecting"
  | "hardware_ready"
  | "downloading"
  | "ready";

export function Onboarding({ onComplete, isMobile = false }: OnboardingProps) {
  const [phase, setPhase] = useState<SetupPhase>("welcome");
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [recommendedModelId, setRecommendedModelId] = useState<string>("");
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [chatSt, setChatSt] = useState<ChatStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Auto-advance from welcome to detecting after brief delay
  useEffect(() => {
    const timer = setTimeout(() => {
      setPhase("detecting");
    }, 1500);
    return () => clearTimeout(timer);
  }, []);

  // Detect hardware when entering detecting phase
  useEffect(() => {
    if (phase !== "detecting") return;

    const detect = async () => {
      try {
        const [hw, recModel, allModels] = await Promise.all([
          getHardwareInfo(),
          getRecommendedModel(),
          getAvailableModels(),
        ]);
        setHardware(hw);
        setRecommendedModelId(recModel);
        setModels(allModels);

        // Brief pause to show results, then advance
        setTimeout(() => setPhase("hardware_ready"), 800);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    };

    detect();
  }, [phase]);

  // Poll chat status during download phase
  useEffect(() => {
    if (phase !== "downloading") return;

    const refresh = () =>
      fetchChatStatus()
        .then((st) => {
          setChatSt(st);
          // Model is ready — advance to final phase
          if (st.available && !st.loading) {
            setPhase("ready");
          }
        })
        .catch(() => {});

    refresh();
    const id = setInterval(refresh, 1000);
    return () => clearInterval(id);
  }, [phase]);

  // Start model download
  const handleStartDownload = useCallback(async () => {
    setPhase("downloading");
    try {
      await chatLoadModel();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  // Finish onboarding
  const handleFinish = useCallback(async () => {
    try {
      await completeSetup();
      onComplete();
    } catch {
      // Even if save fails, let them proceed
      onComplete();
    }
  }, [onComplete]);

  // Skip setup — mark complete without waiting for download
  const handleSkip = useCallback(async () => {
    try {
      await completeSetup();
      onComplete();
    } catch {
      onComplete();
    }
  }, [onComplete]);

  const recommendedModel = models.find((m) => m.id === recommendedModelId);

  return (
    <div className="flex flex-col h-dvh bg-ghost-bg overflow-hidden md:rounded-2xl md:border md:border-ghost-border/50 md:shadow-2xl">
      {/* Draggable title bar — desktop only */}
      {!isMobile && (
        <div
          data-tauri-drag-region
          className="h-4 shrink-0 cursor-grab active:cursor-grabbing"
          onMouseDown={(e) => {
            if (
              e.button === 0 &&
              (e.target as HTMLElement).closest("[data-tauri-drag-region]") ===
                e.currentTarget
            ) {
              startDragging().catch(() => {});
            }
          }}
        />
      )}

      {/* Content */}
      <div className={`flex-1 flex flex-col items-center justify-center pb-8 overflow-y-auto ${
        isMobile ? "px-5 pt-safe" : "px-8"
      }`}>
        {phase === "welcome" && <WelcomePhase />}
        {phase === "detecting" && <DetectingPhase />}
        {phase === "hardware_ready" && (
          <HardwareReadyPhase
            hardware={hardware!}
            recommendedModel={recommendedModel ?? null}
            onStartDownload={handleStartDownload}
            onSkip={handleSkip}
          />
        )}
        {phase === "downloading" && (
          <DownloadingPhase
            chatStatus={chatSt}
            recommendedModel={recommendedModel ?? null}
            error={error}
            onRetry={handleStartDownload}
          />
        )}
        {phase === "ready" && (
          <ReadyPhase
            chatStatus={chatSt}
            onFinish={handleFinish}
          />
        )}

        {/* Error display */}
        {error && phase !== "downloading" && (
          <div className="mt-4 px-4 py-2 bg-ghost-danger/10 border border-ghost-danger/20 rounded-xl text-xs text-ghost-danger max-w-sm text-center">
            {error}
          </div>
        )}
      </div>

      {/* Bottom bar */}
      <footer className="shrink-0 px-6 py-3 border-t border-ghost-border/30 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Shield className="w-3 h-3 text-ghost-success/60" />
          <span className="text-[10px] text-ghost-text-dim/40">
            100% local — tus datos nunca salen de tu equipo
          </span>
        </div>
        {phase !== "welcome" && phase !== "detecting" && phase !== "ready" && (
          <button
            onClick={handleSkip}
            className="text-[10px] text-ghost-text-dim/30 hover:text-ghost-text-dim/60 transition-colors"
          >
            Omitir configuración
          </button>
        )}
      </footer>
    </div>
  );
}

// --- Phase components ---

function WelcomePhase() {
  return (
    <div className="flex flex-col items-center gap-6 animate-in fade-in duration-500">
      <div className="relative">
        <img
          src="/ghost-logo.svg"
          alt="Ghost"
          className="w-20 h-20 rounded-2xl"
          draggable={false}
        />
        <div className="absolute -bottom-1 -right-1 w-6 h-6 rounded-full bg-ghost-accent flex items-center justify-center">
          <Sparkles className="w-3.5 h-3.5 text-white" />
        </div>
      </div>

      <div className="text-center space-y-2">
        <h1 className="text-2xl font-bold text-ghost-text tracking-tight">
          Bienvenido a Ghost
        </h1>
        <p className="text-sm text-ghost-text-dim/60 max-w-xs">
          Tu asistente AI privado y local. Vamos a configurar todo en tu equipo.
        </p>
      </div>

      <div className="flex items-center gap-2 mt-2">
        <div className="w-2 h-2 rounded-full bg-ghost-accent animate-pulse" />
        <span className="text-xs text-ghost-text-dim/40">Iniciando...</span>
      </div>
    </div>
  );
}

function DetectingPhase() {
  return (
    <div className="flex flex-col items-center gap-6 animate-in fade-in duration-300">
      <div className="w-16 h-16 rounded-2xl bg-ghost-accent/10 flex items-center justify-center">
        <Cpu className="w-8 h-8 text-ghost-accent animate-pulse" />
      </div>

      <div className="text-center space-y-2">
        <h2 className="text-lg font-semibold text-ghost-text">
          Analizando tu hardware
        </h2>
        <p className="text-xs text-ghost-text-dim/50">
          Detectando CPU, RAM y GPU para seleccionar el mejor modelo AI...
        </p>
      </div>

      <div className="flex items-center gap-4 mt-2">
        <Loader2 className="w-4 h-4 animate-spin text-ghost-accent" />
        <span className="text-xs text-ghost-text-dim/40">
          Escaneando hardware...
        </span>
      </div>
    </div>
  );
}

function HardwareReadyPhase({
  hardware,
  recommendedModel,
  onStartDownload,
  onSkip,
}: {
  hardware: HardwareInfo;
  recommendedModel: ModelInfo | null;
  onStartDownload: () => void;
  onSkip: () => void;
}) {
  const gpuLabel = hardware.gpu_backend
    ? hardware.gpu_backend
    : "CPU";
  const simd = hardware.has_avx2
    ? "AVX2"
    : hardware.has_neon
      ? "NEON"
      : "Base";

  return (
    <div className="flex flex-col items-center gap-5 animate-in fade-in slide-in-from-bottom-2 duration-300 max-w-sm w-full">
      {/* Hardware summary */}
      <div className="w-full p-4 rounded-xl bg-ghost-surface border border-ghost-border/60 space-y-3">
        <div className="flex items-center gap-2 mb-1">
          <CheckCircle2 className="w-4 h-4 text-ghost-success" />
          <span className="text-sm font-medium text-ghost-text">
            Hardware detectado
          </span>
        </div>

        <div className="grid grid-cols-2 gap-2 text-xs">
          <HardwarePill
            icon={<Cpu className="w-3 h-3" />}
            label="CPU"
            value={`${hardware.cpu_cores} cores (${simd})`}
          />
          <HardwarePill
            icon={<HardDrive className="w-3 h-3" />}
            label="RAM"
            value={`${Math.round(hardware.total_ram_mb / 1024)} GB`}
          />
          <HardwarePill
            icon={<Zap className="w-3 h-3" />}
            label="GPU"
            value={gpuLabel}
          />
          <HardwarePill
            icon={<HardDrive className="w-3 h-3" />}
            label="Disponible"
            value={`${Math.round(hardware.available_ram_mb / 1024)} GB`}
          />
        </div>
      </div>

      {/* Recommended model */}
      {recommendedModel && (
        <div className="w-full p-4 rounded-xl bg-ghost-accent/5 border border-ghost-accent/20 space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-ghost-accent" />
              <span className="text-sm font-medium text-ghost-text">
                Modelo recomendado
              </span>
            </div>
            {recommendedModel.downloaded && (
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-ghost-success/15 text-ghost-success font-medium">
                Descargado
              </span>
            )}
          </div>

          <div className="space-y-1">
            <p className="text-sm font-semibold text-ghost-text">
              {recommendedModel.name}
            </p>
            <p className="text-xs text-ghost-text-dim/50">
              {recommendedModel.description}
            </p>
            <div className="flex items-center gap-3 text-[10px] text-ghost-text-dim/40">
              <span>{recommendedModel.parameters}</span>
              <span>·</span>
              <span>{recommendedModel.size_mb} MB</span>
              <span>·</span>
              <span>RAM mín: {recommendedModel.min_ram_mb} MB</span>
            </div>
          </div>
        </div>
      )}

      {/* Directory discovery notice */}
      <div className="w-full flex items-center gap-2 px-3 py-2 rounded-lg bg-ghost-surface/50 border border-ghost-border/30">
        <FolderSearch className="w-3.5 h-3.5 text-ghost-accent/60 shrink-0" />
        <span className="text-[11px] text-ghost-text-dim/50">
          Ghost indexará automáticamente tus carpetas de Documentos, Escritorio,
          Descargas e Imágenes.
        </span>
      </div>

      {/* Actions */}
      <div className="flex flex-col gap-2 w-full mt-1">
        {recommendedModel?.downloaded ? (
          <button
            onClick={onSkip}
            className="w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-ghost-accent text-white text-sm font-medium hover:bg-ghost-accent-dim transition-all"
          >
            <span>Comenzar a usar Ghost</span>
            <ArrowRight className="w-4 h-4" />
          </button>
        ) : (
          <button
            onClick={onStartDownload}
            className="w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-ghost-accent text-white text-sm font-medium hover:bg-ghost-accent-dim transition-all"
          >
            <Download className="w-4 h-4" />
            <span>
              Descargar modelo
              {recommendedModel
                ? ` (${recommendedModel.size_mb} MB)`
                : ""}
            </span>
          </button>
        )}
      </div>
    </div>
  );
}

function DownloadingPhase({
  chatStatus,
  recommendedModel,
  error,
  onRetry,
}: {
  chatStatus: ChatStatus | null;
  recommendedModel: ModelInfo | null;
  error: string | null;
  onRetry: () => void;
}) {
  const modelName = recommendedModel?.name ?? chatStatus?.model_name ?? "modelo";

  return (
    <div className="flex flex-col items-center gap-5 animate-in fade-in duration-300 max-w-sm w-full">
      <div className="w-14 h-14 rounded-2xl bg-ghost-accent/10 flex items-center justify-center">
        <Download className="w-7 h-7 text-ghost-accent" />
      </div>

      <div className="text-center space-y-1">
        <h2 className="text-lg font-semibold text-ghost-text">
          Configurando Ghost
        </h2>
        <p className="text-xs text-ghost-text-dim/50">
          Descargando el modelo AI para inferencia local...
        </p>
      </div>

      {/* Download progress */}
      {chatStatus?.download_progress && (
        <div className="w-full">
          <DownloadProgressBar
            progress={chatStatus.download_progress}
            modelName={modelName}
          />
        </div>
      )}

      {/* Loading state without download progress */}
      {chatStatus?.loading && !chatStatus.download_progress && (
        <div className="flex items-center gap-3 text-sm text-ghost-text-dim/60">
          <Loader2 className="w-4 h-4 animate-spin text-ghost-accent" />
          <span>Preparando modelo...</span>
        </div>
      )}

      {/* Error with retry */}
      {(chatStatus?.error || error) && (
        <div className="w-full p-3 rounded-xl bg-ghost-danger/10 border border-ghost-danger/20 space-y-2">
          <p className="text-xs text-ghost-danger">
            {chatStatus?.error || error}
          </p>
          <button
            onClick={onRetry}
            className="px-3 py-1.5 bg-ghost-accent/20 text-ghost-accent rounded-lg text-xs font-medium hover:bg-ghost-accent/30 transition-all"
          >
            <Download className="w-3 h-3 inline mr-1" />
            Reintentar
          </button>
        </div>
      )}

      {/* Fun tips while waiting */}
      <DownloadTips />
    </div>
  );
}

function ReadyPhase({
  chatStatus,
  onFinish,
}: {
  chatStatus: ChatStatus | null;
  onFinish: () => void;
}) {
  return (
    <div className="flex flex-col items-center gap-5 animate-in fade-in slide-in-from-bottom-2 duration-500 max-w-sm w-full">
      <div className="relative">
        <div className="w-16 h-16 rounded-2xl bg-ghost-success/10 flex items-center justify-center">
          <CheckCircle2 className="w-8 h-8 text-ghost-success" />
        </div>
      </div>

      <div className="text-center space-y-2">
        <h2 className="text-lg font-semibold text-ghost-text">
          ¡Ghost está listo!
        </h2>
        <p className="text-xs text-ghost-text-dim/50 max-w-xs">
          Todo configurado. Busca archivos, chatea con AI, todo 100% privado en
          tu equipo.
        </p>
      </div>

      {/* Summary */}
      <div className="w-full p-3 rounded-xl bg-ghost-surface border border-ghost-border/60 space-y-2">
        {chatStatus && (
          <div className="flex items-center justify-between text-xs">
            <span className="text-ghost-text-dim/50">Modelo AI</span>
            <span className="text-ghost-text font-medium">
              {chatStatus.model_name}
            </span>
          </div>
        )}
        <div className="flex items-center justify-between text-xs">
          <span className="text-ghost-text-dim/50">Acceso rápido</span>
          <kbd className="px-1.5 py-0.5 rounded bg-ghost-surface-hover text-ghost-text-dim text-[10px] font-mono">
            Ctrl+Space
          </kbd>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="text-ghost-text-dim/50">Privacidad</span>
          <span className="text-ghost-success font-medium">100% local</span>
        </div>
      </div>

      <button
        onClick={onFinish}
        className="w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-ghost-accent text-white text-sm font-medium hover:bg-ghost-accent-dim transition-all mt-1"
      >
        <span>Empezar a usar Ghost</span>
        <ArrowRight className="w-4 h-4" />
      </button>
    </div>
  );
}

// --- Small helper components ---

function HardwarePill({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-ghost-surface-hover/50 border border-ghost-border/30">
      <span className="text-ghost-accent/70">{icon}</span>
      <div className="flex flex-col">
        <span className="text-[9px] text-ghost-text-dim/40 uppercase tracking-wider">
          {label}
        </span>
        <span className="text-[11px] text-ghost-text font-medium">
          {value}
        </span>
      </div>
    </div>
  );
}

const tips = [
  "Ghost nunca envía datos a la nube",
  "Puedes buscar archivos con Ctrl+Space",
  "Los modelos se ejecutan 100% en tu equipo",
  "Ghost indexa PDF, DOCX, XLSX, Markdown y código",
  "Escribe una pregunta para chatear con AI",
  "Personaliza los directorios en Configuración",
];

function DownloadTips() {
  const [tipIndex, setTipIndex] = useState(0);

  useEffect(() => {
    const id = setInterval(() => {
      setTipIndex((prev) => (prev + 1) % tips.length);
    }, 4000);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-ghost-surface/40 border border-ghost-border/20 mt-2">
      <Sparkles className="w-3 h-3 text-ghost-accent/50 shrink-0" />
      <p className="text-[11px] text-ghost-text-dim/40 transition-all duration-300">
        {tips[tipIndex]}
      </p>
    </div>
  );
}
