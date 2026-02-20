import { useRef, useEffect, useState, useMemo } from "react";
import {
  Bot,
  User,
  Sparkles,
  Loader2,
  Download,
  AlertCircle,
  Clock,
} from "lucide-react";
import { DownloadProgressBar } from "./DownloadProgress";
import type { ChatMessage, ChatStatus } from "../lib/types";

interface ChatMessagesProps {
  messages: ChatMessage[];
  isGenerating: boolean;
  /** Live streaming content from AG-UI (TEXT_MESSAGE_CONTENT deltas). */
  streamingContent?: string;
  status: ChatStatus | null;
  tokensInfo: string | null;
  error: string | null;
  onRetryDownload: () => void;
  /** Whether the app is running on a mobile device */
  isMobile?: boolean;
}

function getTimeGreeting(): { greeting: string; emoji: string } {
  const hour = new Date().getHours();
  if (hour >= 5 && hour < 12) return { greeting: "Buenos días", emoji: "sunrise" };
  if (hour >= 12 && hour < 18) return { greeting: "Buenas tardes", emoji: "sun" };
  return { greeting: "Buenas noches", emoji: "moon" };
}

function getFormattedTime(): string {
  return new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function getFormattedDate(): string {
  return new Date().toLocaleDateString("es", {
    weekday: "long",
    day: "numeric",
    month: "long",
  });
}

function getTips(isMobile: boolean): string[] {
  const base = [
    "Escribe ? para preguntar algo",
    "Busca archivos escribiendo su nombre",
  ];
  if (!isMobile) {
    base.push(
      "Ctrl+Space para mostrar/ocultar",
      "Ctrl+, para abrir configuración",
      "Ctrl+D para ver los logs de depuración"
    );
  }
  return base;
}

function SmartGreeting({ isMobile = false }: { isMobile?: boolean }) {
  const [time, setTime] = useState(getFormattedTime());
  const { greeting } = getTimeGreeting();
  const date = getFormattedDate();
  const tips = useMemo(() => getTips(isMobile), [isMobile]);
  const tip = tips[Math.floor(Math.random() * tips.length)];

  useEffect(() => {
    const id = setInterval(() => setTime(getFormattedTime()), 30_000);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="flex flex-col items-center justify-center flex-1 gap-4">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-2xl bg-ghost-accent/10 flex items-center justify-center">
          <Sparkles className="w-6 h-6 text-ghost-accent" />
        </div>
      </div>
      <div className="text-center space-y-1.5">
        <p className="text-base font-semibold text-ghost-text/80">
          {greeting}
        </p>
        <div className="flex items-center justify-center gap-1.5 text-ghost-text-dim/40">
          <Clock className="w-3 h-3" />
          <span className="text-xs font-mono">{time}</span>
          <span className="text-xs">·</span>
          <span className="text-xs capitalize">{date}</span>
        </div>
      </div>
      <p className="text-[11px] text-ghost-text-dim/30 max-w-60 text-center">
        {tip}
      </p>
    </div>
  );
}

export function ChatMessages({
  messages,
  isGenerating,
  streamingContent,
  status,
  tokensInfo,
  error,
  onRetryDownload,
  isMobile = false,
}: ChatMessagesProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isGenerating, streamingContent]);

  const isAvailable = status?.available ?? false;
  const isLoading = status?.loading ?? false;

  return (
    <div className="flex flex-col h-full overflow-y-auto px-3 py-3 space-y-3">
      {/* Download progress bar */}
      {isLoading && status?.download_progress && (
        <DownloadProgressBar
          progress={status.download_progress}
          modelName={status.model_name || "modelo"}
        />
      )}

      {/* Empty states */}
      {messages.length === 0 && !isLoading && (
        <div className="flex flex-col items-center justify-center flex-1 text-ghost-text-dim/50 gap-3">
          {isAvailable ? (
            <SmartGreeting isMobile={isMobile} />
          ) : status?.error ? (
            <>
              <div className="w-12 h-12 rounded-2xl bg-ghost-danger/10 flex items-center justify-center">
                <AlertCircle className="w-6 h-6 text-ghost-danger" />
              </div>
              <div className="text-center space-y-2">
                <p className="text-sm font-medium text-ghost-text/70">
                  Error al cargar modelo
                </p>
                <p className="text-xs text-ghost-danger/70 max-w-70 wrap-break-word">
                  {status.error}
                </p>
                <button
                  onClick={onRetryDownload}
                  className="mt-1 px-3 py-1.5 bg-ghost-accent/20 text-ghost-accent rounded-lg text-xs font-medium hover:bg-ghost-accent/30 transition-all"
                >
                  <Download className="w-3 h-3 inline mr-1" />
                  Reintentar
                </button>
              </div>
            </>
          ) : !isLoading ? (
            <>
              <div className="w-12 h-12 rounded-2xl bg-ghost-warning/10 flex items-center justify-center">
                <Download className="w-6 h-6 text-ghost-warning" />
              </div>
              <div className="text-center space-y-2">
                <p className="text-sm font-medium text-ghost-text/70">
                  Modelo no disponible
                </p>
                <p className="text-xs text-ghost-text-dim/40 max-w-60">
                  Se descarga automáticamente la primera vez.
                </p>
                <button
                  onClick={onRetryDownload}
                  className="mt-1 px-3 py-1.5 bg-ghost-accent/20 text-ghost-accent rounded-lg text-xs font-medium hover:bg-ghost-accent/30 transition-all"
                >
                  <Download className="w-3 h-3 inline mr-1" />
                  Descargar modelo
                </button>
              </div>
            </>
          ) : null}
        </div>
      )}

      {/* Loading state (no messages yet but downloading) */}
      {messages.length === 0 && isLoading && !status?.download_progress && (
        <div className="flex flex-col items-center justify-center flex-1 text-ghost-text-dim/50 gap-3">
          <Loader2 className="w-8 h-8 animate-spin text-ghost-accent" />
          <div className="text-center space-y-1">
            <p className="text-sm font-medium text-ghost-text/70">
              Preparando modelo...
            </p>
            <p className="text-xs text-ghost-text-dim/40">
              Primera vez puede tomar unos minutos
            </p>
          </div>
        </div>
      )}

      {/* Chat messages */}
      {messages.map((msg, i) => (
        <MessageBubble key={i} message={msg} />
      ))}

      {/* Generating indicator / streaming content */}
      {isGenerating && (
        <div className="flex items-start gap-2.5 px-1">
          <div className="w-6 h-6 rounded-lg bg-ghost-accent/15 flex items-center justify-center shrink-0 mt-0.5">
            <Bot className="w-3.5 h-3.5 text-ghost-accent" />
          </div>
          {streamingContent ? (
            <div className="max-w-[85%] px-3 py-2 rounded-xl text-sm leading-relaxed bg-ghost-surface text-ghost-text border border-ghost-border/50">
              <p className="whitespace-pre-wrap wrap-break-word">
                {streamingContent}
                <span className="inline-block w-1.5 h-4 bg-ghost-accent/60 animate-pulse ml-0.5 align-text-bottom rounded-sm" />
              </p>
            </div>
          ) : (
            <div className="flex items-center gap-2 text-sm text-ghost-text-dim/60">
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              Generando...
            </div>
          )}
        </div>
      )}

      {/* Token info */}
      {tokensInfo && (
        <div className="px-1 text-[10px] text-ghost-text-dim/30 text-right">
          {tokensInfo}
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="mx-1 px-3 py-2 bg-ghost-danger/10 border border-ghost-danger/20 rounded-lg text-xs text-ghost-danger">
          {error}
        </div>
      )}

      <div ref={messagesEndRef} />
    </div>
  );
}

function MessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === "user";

  return (
    <div
      className={`flex items-start gap-2.5 px-1 ${isUser ? "flex-row-reverse" : ""}`}
    >
      <div
        className={`w-6 h-6 rounded-lg flex items-center justify-center shrink-0 mt-0.5 ${
          isUser ? "bg-ghost-surface-hover" : "bg-ghost-accent/15"
        }`}
      >
        {isUser ? (
          <User className="w-3.5 h-3.5 text-ghost-text-dim" />
        ) : (
          <Bot className="w-3.5 h-3.5 text-ghost-accent" />
        )}
      </div>
      <div
        className={`max-w-[85%] px-3 py-2 rounded-xl text-sm leading-relaxed ${
          isUser
            ? "bg-ghost-accent/15 text-ghost-text"
            : "bg-ghost-surface text-ghost-text border border-ghost-border/50"
        }`}
      >
        <p className="whitespace-pre-wrap wrap-break-word">{message.content}</p>
      </div>
    </div>
  );
}
