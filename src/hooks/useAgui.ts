/**
 * useAgui — React hook for AG-UI event-driven streaming chat.
 *
 * Listens to Tauri `agui://event` events and maintains streaming state.
 * Replaces the polling-based chat flow with real-time event streaming.
 *
 * Usage:
 *   const { runState, sendStreaming, isStreaming } = useAgui();
 *
 *   // Start a streaming chat
 *   await sendStreaming(messages);
 *
 *   // Read streaming content as it arrives
 *   <p>{runState?.content}</p>
 */

import { useState, useEffect, useCallback, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AgUiEvent, AgUiRunState, ChatMessage } from "../lib/types";
import { chatSendStreaming } from "../lib/tauri";

/**
 * AG-UI streaming chat hook.
 * Manages a single active run at a time.
 */
export function useAgui() {
  const [runState, setRunState] = useState<AgUiRunState | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const unlistenRef = useRef<UnlistenFn | null>(null);
  const activeRunIdRef = useRef<string | null>(null);

  // Set up the Tauri event listener on mount
  useEffect(() => {
    let mounted = true;

    const setup = async () => {
      const unlisten = await listen<AgUiEvent>("agui://event", (event) => {
        if (!mounted) return;
        const aguiEvent = event.payload;

        // Only process events for the active run
        if (
          activeRunIdRef.current &&
          aguiEvent.runId !== activeRunIdRef.current
        ) {
          return;
        }

        setRunState((prev) => processEvent(prev, aguiEvent));
      });

      if (mounted) {
        unlistenRef.current = unlisten;
      } else {
        unlisten();
      }
    };

    setup();

    return () => {
      mounted = false;
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    };
  }, []);

  // Process a single AG-UI event and update the run state
  const processEvent = useCallback(
    (prev: AgUiRunState | null, event: AgUiEvent): AgUiRunState | null => {
      switch (event.type) {
        case "RUN_STARTED": {
          return {
            runId: event.runId,
            status: "running",
            content: "",
            currentStep: null,
            activeToolCalls: new Map(),
            error: null,
            metadata: null,
          };
        }

        case "STEP_STARTED": {
          if (!prev) return prev;
          return { ...prev, currentStep: event.stepName ?? null };
        }

        case "STEP_FINISHED": {
          if (!prev) return prev;
          return { ...prev, currentStep: null };
        }

        case "TEXT_MESSAGE_START": {
          // Ready to receive content — no state change needed
          return prev;
        }

        case "TEXT_MESSAGE_CONTENT": {
          if (!prev) return prev;
          return {
            ...prev,
            content: prev.content + (event.delta ?? ""),
          };
        }

        case "TEXT_MESSAGE_END": {
          // Message streaming complete — content is already accumulated
          return prev;
        }

        case "TOOL_CALL_START": {
          if (!prev || !event.toolCallId) return prev;
          const toolCalls = new Map(prev.activeToolCalls);
          toolCalls.set(event.toolCallId, {
            name: event.toolCallName ?? "unknown",
            args: "",
          });
          return { ...prev, activeToolCalls: toolCalls };
        }

        case "TOOL_CALL_ARGS": {
          if (!prev || !event.toolCallId) return prev;
          const toolCalls = new Map(prev.activeToolCalls);
          const tc = toolCalls.get(event.toolCallId);
          if (tc) {
            toolCalls.set(event.toolCallId, {
              ...tc,
              args: tc.args + (event.delta ?? ""),
            });
          }
          return { ...prev, activeToolCalls: toolCalls };
        }

        case "TOOL_CALL_END": {
          if (!prev || !event.toolCallId) return prev;
          const toolCalls = new Map(prev.activeToolCalls);
          const tc = toolCalls.get(event.toolCallId);
          if (tc) {
            toolCalls.set(event.toolCallId, {
              ...tc,
              result: event.result ?? undefined,
            });
          }
          return { ...prev, activeToolCalls: toolCalls };
        }

        case "RUN_FINISHED": {
          setIsStreaming(false);
          activeRunIdRef.current = null;
          if (!prev) return prev;
          return { ...prev, status: "finished" };
        }

        case "RUN_ERROR": {
          setIsStreaming(false);
          activeRunIdRef.current = null;
          if (!prev) return prev;
          return {
            ...prev,
            status: "error",
            error: event.message ?? "Unknown error",
          };
        }

        case "CUSTOM": {
          if (!prev) return prev;
          if (event.name === "generation_stats" && event.value) {
            return {
              ...prev,
              metadata: event.value as Record<string, unknown>,
            };
          }
          return prev;
        }

        default:
          return prev;
      }
    },
    []
  );

  /**
   * Send a streaming chat request. Returns the run_id.
   * The runState will update reactively as events arrive.
   */
  const sendStreaming = useCallback(
    async (messages: ChatMessage[], maxTokens?: number): Promise<string> => {
      setIsStreaming(true);
      setRunState(null); // Reset previous run state
      const runId = await chatSendStreaming(messages, maxTokens);
      activeRunIdRef.current = runId;
      return runId;
    },
    []
  );

  /**
   * Reset the run state (e.g., for a new conversation).
   */
  const reset = useCallback(() => {
    setRunState(null);
    setIsStreaming(false);
    activeRunIdRef.current = null;
  }, []);

  return {
    /** Current streaming run state, or null if no active/completed run. */
    runState,
    /** Whether a streaming run is currently in progress. */
    isStreaming,
    /** Start a streaming chat. Returns the run_id. */
    sendStreaming,
    /** Reset the hook state for a new conversation. */
    reset,
  };
}
