/**
 * useAgui — React hook for AG-UI event-driven streaming chat.
 *
 * Listens to Tauri `agui://event` events and maintains streaming state.
 * Replaces the polling-based chat flow with real-time event streaming.
 *
 * Supports all 30+ AG-UI event types including:
 * - Text streaming (TEXT_MESSAGE_*)
 * - Tool calls (TOOL_CALL_*)
 * - Extended reasoning (REASONING_*)
 * - Activity annotations (ACTIVITY_*)
 * - State synchronization (STATE_*, MESSAGES_SNAPSHOT)
 * - A2UI generative UI (via CUSTOM events)
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
import type { AgUiEvent, AgUiRunState, ChatMessage, A2uiMessage } from "../lib/types";
import { chatSendStreaming } from "../lib/tauri";
import { computeRootIds } from "../components/A2UIRenderer";

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
            a2uiSurfaces: new Map(),
            reasoningContent: "",
            activities: new Map(),
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

        case "TEXT_MESSAGE_CONTENT":
        case "TEXT_MESSAGE_CHUNK": {
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

        case "TOOL_CALL_ARGS":
        case "TOOL_CALL_CHUNK": {
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

        case "TOOL_CALL_RESULT": {
          // Structured result (includes messageId, content, role)
          if (!prev || !event.toolCallId) return prev;
          const toolCalls = new Map(prev.activeToolCalls);
          const tc = toolCalls.get(event.toolCallId);
          if (tc) {
            const resultStr =
              typeof event.content === "string"
                ? event.content
                : JSON.stringify(event.content);
            toolCalls.set(event.toolCallId, { ...tc, result: resultStr });
          }
          return { ...prev, activeToolCalls: toolCalls };
        }

        // ── Reasoning events (extended reasoning models) ────────────────
        case "REASONING_START":
        case "REASONING_MESSAGE_START": {
          // Start of reasoning block — reset accumulator
          if (!prev) return prev;
          return { ...prev, reasoningContent: "" };
        }

        case "REASONING_MESSAGE_CONTENT": {
          if (!prev) return prev;
          return {
            ...prev,
            reasoningContent: prev.reasoningContent + (event.delta ?? ""),
          };
        }

        case "REASONING_MESSAGE_END":
        case "REASONING_END": {
          // Reasoning block complete
          return prev;
        }

        case "REASONING_ENCRYPTED_VALUE": {
          // Opaque encrypted payload — store in metadata for debugging
          if (!prev) return prev;
          return {
            ...prev,
            metadata: {
              ...prev.metadata,
              encryptedReasoning: event.encryptedValue,
            },
          };
        }

        // ── Activity events (citations, thoughts, annotations) ──────────
        case "ACTIVITY_SNAPSHOT": {
          if (!prev || !event.messageId || !event.activityType) return prev;
          const activities = new Map(prev.activities);
          activities.set(event.messageId, {
            activityType: event.activityType,
            content: event.content,
          });
          return { ...prev, activities };
        }

        case "ACTIVITY_DELTA": {
          // JSON Patch on existing activity — apply patch client-side
          // For simplicity, store the raw patch alongside the activity
          if (!prev || !event.messageId) return prev;
          const activities = new Map(prev.activities);
          const existing = activities.get(event.messageId);
          if (existing) {
            activities.set(event.messageId, {
              ...existing,
              content: { ...(existing.content as object), _patch: event.patch },
            });
          }
          return { ...prev, activities };
        }

        // ── State events ────────────────────────────────────────────────
        case "MESSAGES_SNAPSHOT": {
          // Full thread snapshot — store as metadata
          if (!prev) return prev;
          return {
            ...prev,
            metadata: { ...prev.metadata, messagesSnapshot: event.messages },
          };
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

          // Handle generation stats
          if (event.name === "generation_stats" && event.value) {
            return {
              ...prev,
              metadata: event.value as Record<string, unknown>,
            };
          }

          // Handle A2UI messages transported via CUSTOM events
          if (event.name === "a2ui" && event.value) {
            const a2uiMsg = event.value as A2uiMessage;
            const surfaces = new Map(prev.a2uiSurfaces);

            if (a2uiMsg.createSurface) {
              const cs = a2uiMsg.createSurface;
              surfaces.set(cs.surfaceId, {
                surfaceId: cs.surfaceId,
                catalogId: cs.catalogId,
                theme: cs.theme,
                sendDataModel: cs.sendDataModel,
                components: new Map(),
                dataModel: {},
                rootIds: [],
              });
            }

            if (a2uiMsg.updateComponents) {
              const uc = a2uiMsg.updateComponents;
              const surface = surfaces.get(uc.surfaceId);
              if (surface) {
                const componentMap = new Map(surface.components);
                for (const comp of uc.components) {
                  componentMap.set(comp.id, comp);
                }
                const rootIds = computeRootIds(Array.from(componentMap.values()));
                surfaces.set(uc.surfaceId, {
                  ...surface,
                  components: componentMap,
                  rootIds,
                });
              }
            }

            if (a2uiMsg.updateDataModel) {
              const udm = a2uiMsg.updateDataModel;
              const surface = surfaces.get(udm.surfaceId);
              if (surface) {
                const dataModel = { ...surface.dataModel };
                if (!udm.path || udm.path === "/") {
                  // Replace entire data model
                  surfaces.set(udm.surfaceId, {
                    ...surface,
                    dataModel: (udm.value ?? {}) as Record<string, unknown>,
                  });
                } else {
                  // Set at specific path (JSON Pointer)
                  const parts = udm.path.replace(/^\//, "").split("/");
                  let current: Record<string, unknown> = dataModel;
                  for (let i = 0; i < parts.length - 1; i++) {
                    if (
                      current[parts[i]] === undefined ||
                      typeof current[parts[i]] !== "object"
                    ) {
                      current[parts[i]] = {};
                    }
                    current = current[parts[i]] as Record<string, unknown>;
                  }
                  current[parts[parts.length - 1]] = udm.value;
                  surfaces.set(udm.surfaceId, { ...surface, dataModel });
                }
              }
            }

            if (a2uiMsg.deleteSurface) {
              surfaces.delete(a2uiMsg.deleteSurface.surfaceId);
            }

            return { ...prev, a2uiSurfaces: surfaces };
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
