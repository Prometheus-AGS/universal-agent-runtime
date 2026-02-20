import { useCallback, useRef } from "react";
import { useChatMessageStore } from "@/stores/chat-message-store";
import type { ToolCallContentBlock } from "@/types/chat-content";
import type { AttachmentPayload } from "@/types";

const UAR_URL = "/api/chat/completion";

export interface UarChatPayload {
  message: string;
  attachments?: AttachmentPayload[];
}
export interface StreamCallbacks {
  onComplete?: () => void;
  onError?: (error: Error) => void;
}

// AG-UI event types
interface AguiMessageDelta { kind: "message"; phase: "delta"; request_id: string; delta: { text: string } }
interface AguiThinkingDelta { kind: "thinking"; phase: "delta"; request_id: string; delta: { text: string } }
interface AguiReasoningDelta { kind: "reasoning"; phase: "delta"; request_id: string; delta: { text: string } }
interface AguiCitationAdded { kind: "citation"; phase: "added"; request_id: string; citation: { index: number; url?: string; title?: string; snippet?: string } }
interface AguiToolCallDelta { kind: "tool_call"; phase: "delta"; request_id: string; call_index: number; id: string; delta: { arguments: string } }
interface AguiToolCallComplete { kind: "tool_call"; phase: "complete"; request_id: string; call_index: number; id: string; name: string; arguments_json: string }
interface AguiToolResult { kind: "tool_result"; request_id: string; call_index: number; id: string; name: string; content: string; success: boolean }
interface AguiError { kind: "error"; request_id: string; message: string; code?: string }
interface AguiSkillActivated { kind: "skill"; phase: "activated"; request_id: string; skill: { id: string; title: string }; selection_method: string }
interface AguiContextUpdate { kind: "context"; phase: "update"; strategy: string; messages_removed: number; tokens_saved: number; was_applied: boolean; summary_generated: boolean }

type AguiPayload = AguiMessageDelta | AguiThinkingDelta | AguiReasoningDelta | AguiCitationAdded | AguiToolCallDelta | AguiToolCallComplete | AguiToolResult | AguiError | AguiSkillActivated | AguiContextUpdate | { kind: string;[k: string]: unknown };

function parseSseBlock(raw: string): { event: string; data: string } | null {
  let event = "message";
  let data = "";
  for (const line of raw.split("\n")) {
    if (line.startsWith("event:")) event = line.slice(6).trim();
    else if (line.startsWith("data:")) data = line.slice(5).trim();
  }
  if (!data) return null;
  return { event, data };
}

export function useMessageStream() {
  const abortRef = useRef<AbortController | null>(null);

  const cancelStream = useCallback(() => {
    abortRef.current?.abort();
    abortRef.current = null;
  }, []);

  const startStream = useCallback(
    async (threadId: string, payload: UarChatPayload, callbacks?: StreamCallbacks): Promise<void> => {
      cancelStream();

      const userMsgId = `user-${Date.now()}`;
      // Build user message content: text + any image attachments for in-thread display
      const imageBlocks = (payload.attachments ?? [])
        .filter((a) => a.content_type.startsWith("image/"))
        .map((a) => ({ type: "image" as const, url: a.url, alt: a.filename }));
      useChatMessageStore.getState().initThread(threadId, [
        ...(useChatMessageStore.getState().messagesByThread[threadId] ?? []),
        { id: userMsgId, role: "user", content: [{ type: "text", text: payload.message }, ...imageBlocks], createdAt: new Date(), status: "complete" },
      ]);

      const controller = new AbortController();
      abortRef.current = controller;
      const runId = `run-${Date.now()}`;
      const pendingArgs = new Map<string, string>();

      try {
        const res = await fetch(UAR_URL, {
          method: "POST",
          headers: { "Content-Type": "application/json", "X-UAR-Session-ID": threadId },
          body: JSON.stringify({
            message: payload.message,
            stream: true,
            stream_mode: "dual",
            ...(payload.attachments?.length ? { attachments: payload.attachments } : {}),
          }),
          signal: controller.signal,
        });

        if (!res.ok) {
          const body = await res.text().catch(() => "(no response body)");
          const ts = new Date().toISOString();
          throw new Error(
            `[${ts}] LLM request failed\n` +
            `  URL: POST ${UAR_URL}\n` +
            `  Status: ${res.status} ${res.statusText}\n` +
            `  Session: ${threadId}\n` +
            `  Response body:\n${body}`
          );
        }
        if (!res.body) throw new Error("Response body is null");

        const reader = res.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          const blocks = buffer.split("\n\n");
          buffer = blocks.pop() ?? "";

          for (const raw of blocks) {
            if (!raw.trim()) continue;
            const block = parseSseBlock(raw);
            if (!block) continue;
            const { event, data } = block;

            if (event.startsWith("agui.")) {
              let agui: AguiPayload;
              try { agui = JSON.parse(data) as AguiPayload; } catch { continue; }

              const store = useChatMessageStore.getState();

              switch (event) {
                case "agui.message.delta": {
                  const e = agui as AguiMessageDelta;
                  if (e.delta?.text) store.appendTextDelta(threadId, runId, e.delta.text);
                  break;
                }
                case "agui.thinking.delta":
                case "agui.reasoning.delta": {
                  const e = agui as AguiThinkingDelta | AguiReasoningDelta;
                  if (e.delta?.text) store.appendThinkingDelta(threadId, runId, e.delta.text);
                  break;
                }
                case "agui.citation.added": {
                  const e = agui as AguiCitationAdded;
                  if (e.citation) store.addCitation(threadId, { source: e.citation.title ?? e.citation.url ?? "Source", content: e.citation.snippet ?? "", url: e.citation.url });
                  break;
                }
                case "agui.tool_call.delta": {
                  const e = agui as AguiToolCallDelta;
                  pendingArgs.set(e.id, (pendingArgs.get(e.id) ?? "") + (e.delta?.arguments ?? ""));
                  break;
                }
                case "agui.tool_call.complete": {
                  const e = agui as AguiToolCallComplete;
                  let args: Record<string, unknown>;
                  try { args = JSON.parse(e.arguments_json) as Record<string, unknown>; } catch { args = { _raw: e.arguments_json }; }
                  const toolCall: ToolCallContentBlock = { type: "tool-call", toolCallId: e.id, toolName: e.name, args, status: "running" };
                  store.addToolCall(threadId, toolCall);
                  pendingArgs.delete(e.id);
                  break;
                }
                case "agui.tool_result": {
                  const e = agui as AguiToolResult;
                  store.updateToolCall(threadId, e.id, { result: e.content, status: e.success ? "complete" : "failed" });
                  break;
                }
                case "agui.error": {
                  const e = agui as AguiError;
                  const ts = new Date().toISOString();
                  const detail = [
                    `[${ts}] Agent stream error`,
                    e.code ? `  Code: ${e.code}` : null,
                    `  Message: ${e.message}`,
                    `  Session: ${threadId}`,
                    `  Request: ${e.request_id}`,
                  ].filter(Boolean).join("\n");
                  store.setStreamError(threadId, detail);
                  callbacks?.onError?.(new Error(detail));
                  return;
                }
                case "agui.skill.activated": {
                  const e = agui as AguiSkillActivated;
                  store.addSkillActivation(threadId, { skillId: e.skill.id, skillName: e.skill.title, selectionMethod: e.selection_method, status: "active" });
                  break;
                }
                case "agui.context.update": {
                  const e = agui as AguiContextUpdate;
                  if (e.was_applied) store.addContextUpdate(threadId, { strategy: e.strategy, messagesRemoved: e.messages_removed, tokensSaved: e.tokens_saved, wasApplied: e.was_applied, summaryGenerated: e.summary_generated });
                  break;
                }
                case "agui.done":
                  store.finishStream(threadId);
                  callbacks?.onComplete?.();
                  return;
                default:
                  break;
              }
              continue;
            }

            if (event === "message" && data === "[DONE]") {
              useChatMessageStore.getState().finishStream(threadId);
              callbacks?.onComplete?.();
              return;
            }
          }
        }

        useChatMessageStore.getState().finishStream(threadId);
        callbacks?.onComplete?.();
      } catch (err) {
        if ((err as Error).name === "AbortError") return;
        const error = err instanceof Error ? err : new Error(String(err));
        // error.message already has full context (URL, status, body, timestamp)
        // from the throw above; for unexpected runtime errors, add stack info
        const detail = error.message.startsWith("[20") ? error.message
          : `[${new Date().toISOString()}] Unexpected error\n  ${error.message}\n  Session: ${threadId}`;
        useChatMessageStore.getState().setStreamError(threadId, detail);
        callbacks?.onError?.(new Error(detail));
      }
    },
    [cancelStream],
  );

  return { startStream, cancelStream };
}
