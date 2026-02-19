import { useState, useCallback, useEffect, useRef } from "react";
import {
  Send,
  Loader2,
  Bot,
  User,
  Sparkles,
  AlertCircle,
  RotateCcw,
  Download,
  Cpu,
} from "lucide-react";
import { chatSend, chatStatus, chatLoadModel } from "../lib/tauri";
import type { ChatMessage, ChatStatus } from "../lib/types";

export function ChatPanel() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [status, setStatus] = useState<ChatStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [tokensInfo, setTokensInfo] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Poll chat status
  useEffect(() => {
    const refresh = () => chatStatus().then(setStatus).catch(() => {});
    refresh();
    const interval = setInterval(refresh, 3000);
    return () => clearInterval(interval);
  }, []);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const sendMessage = useCallback(async () => {
    const trimmed = input.trim();
    if (!trimmed || isGenerating) return;

    setError(null);
    setTokensInfo(null);
    const userMsg: ChatMessage = { role: "user", content: trimmed };
    const newMessages = [...messages, userMsg];
    setMessages(newMessages);
    setInput("");
    setIsGenerating(true);

    try {
      const response = await chatSend(newMessages);
      const assistantMsg: ChatMessage = {
        role: "assistant",
        content: response.content,
      };
      setMessages([...newMessages, assistantMsg]);
      setTokensInfo(
        `${response.tokens_generated} tokens · ${(response.duration_ms / 1000).toFixed(1)}s · ${response.model_id}`
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setIsGenerating(false);
      inputRef.current?.focus();
    }
  }, [input, messages, isGenerating]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
      }
    },
    [sendMessage]
  );

  const clearChat = useCallback(() => {
    setMessages([]);
    setError(null);
    setTokensInfo(null);
    inputRef.current?.focus();
  }, []);

  const triggerDownload = useCallback(() => {
    chatLoadModel().catch(() => {});
  }, []);

  const isAvailable = status?.available ?? false;
  const isLoading = status?.loading ?? false;

  return (
    <div className="flex flex-col h-full">
      {/* Chat status bar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-ghost-border">
        <div className="flex items-center gap-2 text-xs text-ghost-text-dim">
          <Cpu className="w-3 h-3" />
          {isLoading ? (
            <span className="flex items-center gap-1.5 text-ghost-warning">
              <Loader2 className="w-3 h-3 animate-spin" />
              Cargando modelo...
            </span>
          ) : isAvailable ? (
            <span className="text-ghost-success">
              {status?.model_name} · {status?.backend}
            </span>
          ) : (
            <span className="text-ghost-danger">
              {status?.error ? "Error" : "Sin modelo"}
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          {messages.length > 0 && (
            <button
              onClick={clearChat}
              className="p-1.5 rounded-md text-ghost-text-dim/50 hover:text-ghost-text hover:bg-ghost-surface-hover transition-all"
              title="Nueva conversación"
            >
              <RotateCcw className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>

      {/* Messages area */}
      <div className="flex-1 overflow-y-auto px-3 py-3 space-y-3">
        {messages.length === 0 && !isLoading && (
          <div className="flex flex-col items-center justify-center h-full text-ghost-text-dim/50 gap-3">
            {isAvailable ? (
              <>
                <div className="w-12 h-12 rounded-2xl bg-ghost-accent/10 flex items-center justify-center">
                  <Sparkles className="w-6 h-6 text-ghost-accent" />
                </div>
                <div className="text-center space-y-1">
                  <p className="text-sm font-medium text-ghost-text/70">
                    Ghost Chat
                  </p>
                  <p className="text-xs text-ghost-text-dim/40 max-w-60">
                    100% local, zero cloud. Escribe algo para comenzar.
                  </p>
                </div>
              </>
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
                    onClick={triggerDownload}
                    className="mt-1 px-3 py-1.5 bg-ghost-accent/20 text-ghost-accent rounded-lg text-xs font-medium hover:bg-ghost-accent/30 transition-all"
                  >
                    <Download className="w-3 h-3 inline mr-1" />
                    Reintentar descarga
                  </button>
                </div>
              </>
            ) : (
              <>
                <div className="w-12 h-12 rounded-2xl bg-ghost-warning/10 flex items-center justify-center">
                  <Download className="w-6 h-6 text-ghost-warning" />
                </div>
                <div className="text-center space-y-2">
                  <p className="text-sm font-medium text-ghost-text/70">
                    Modelo no disponible
                  </p>
                  <p className="text-xs text-ghost-text-dim/40 max-w-60">
                    El modelo se descarga automáticamente la primera vez.
                  </p>
                  <button
                    onClick={triggerDownload}
                    className="mt-1 px-3 py-1.5 bg-ghost-accent/20 text-ghost-accent rounded-lg text-xs font-medium hover:bg-ghost-accent/30 transition-all"
                  >
                    <Download className="w-3 h-3 inline mr-1" />
                    Descargar modelo
                  </button>
                </div>
              </>
            )}
          </div>
        )}

        {messages.length === 0 && isLoading && (
          <div className="flex flex-col items-center justify-center h-full text-ghost-text-dim/50 gap-3">
            <Loader2 className="w-8 h-8 animate-spin text-ghost-accent" />
            <div className="text-center space-y-1">
              <p className="text-sm font-medium text-ghost-text/70">
                Descargando modelo...
              </p>
              <p className="text-xs text-ghost-text-dim/40">
                Primera vez puede tomar unos minutos
              </p>
            </div>
          </div>
        )}

        {messages.map((msg, i) => (
          <MessageBubble key={i} message={msg} />
        ))}

        {isGenerating && (
          <div className="flex items-start gap-2.5 px-1">
            <div className="w-6 h-6 rounded-lg bg-ghost-accent/15 flex items-center justify-center shrink-0 mt-0.5">
              <Bot className="w-3.5 h-3.5 text-ghost-accent" />
            </div>
            <div className="flex items-center gap-2 text-sm text-ghost-text-dim/60">
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              Pensando...
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Token info */}
      {tokensInfo && (
        <div className="px-4 py-1 text-[10px] text-ghost-text-dim/40 text-right">
          {tokensInfo}
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="mx-3 mb-2 px-3 py-2 bg-ghost-danger/10 border border-ghost-danger/20 rounded-lg text-xs text-ghost-danger">
          {error}
        </div>
      )}

      {/* Input area */}
      <div className="px-3 pb-3 pt-1">
        <div className="flex items-end gap-2 bg-ghost-surface border border-ghost-border rounded-xl px-3 py-2 focus-within:border-ghost-accent/40 transition-colors">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={
              isAvailable
                ? "Escribe un mensaje..."
                : isLoading
                  ? "Cargando modelo..."
                  : "Modelo no disponible"
            }
            disabled={!isAvailable || isGenerating}
            rows={1}
            className="flex-1 bg-transparent text-sm text-ghost-text placeholder:text-ghost-text-dim/30 outline-none resize-none disabled:opacity-40 max-h-32"
            style={{ minHeight: "1.5rem" }}
            onInput={(e) => {
              const el = e.target as HTMLTextAreaElement;
              el.style.height = "auto";
              el.style.height = Math.min(el.scrollHeight, 128) + "px";
            }}
          />
          <button
            onClick={sendMessage}
            disabled={!input.trim() || !isAvailable || isGenerating}
            className="p-1.5 rounded-lg bg-ghost-accent text-white disabled:opacity-30 disabled:cursor-not-allowed hover:bg-ghost-accent-dim transition-all shrink-0"
          >
            {isGenerating ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Send className="w-4 h-4" />
            )}
          </button>
        </div>
      </div>
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
