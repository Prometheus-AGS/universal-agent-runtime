import { create } from "zustand";

import {
  postChatCompletion,
  cancelRun,
  resumeRunStream,
  CHAT_COMPLETION_URL,
} from "@/services/chat-stream-api";
import { fetchResilienceSettings } from "@/services/settings-api";
import { onSettingsChanged } from "@/services/settings-change-bus";
import { useChatMessageStore } from "@/stores/chat-message-store";
import { useAgentStatusStore } from "@/stores/agent-status-store";
import type { ToolCallContentBlock } from "@/types/chat-content";
import type { AttachmentPayload } from "@/types";
import { ingestAgUiEvent, ingestRuntimeEvent } from "@/entities/runtime-ingest";

export interface UarChatPayload {
  message: string;
  attachments?: AttachmentPayload[];
  /** When false, skips memory context injection and auto-capture for this turn. Defaults server-side to true. */
  memory_enabled?: boolean;
  /** Agent id to use for this run. When provided, overrides the session-level
   * agent-config so selection is authoritative on the request, not a prior
   * best-effort side-channel POST. */
  agent_id?: string;
  /** Model override in "provider/model" format. */
  model?: string;
}
export interface StreamCallbacks {
  onComplete?: () => void;
  onError?: (error: Error) => void;
}

// AG-UI event types
interface AguiMessageDelta {
  kind: "message";
  phase: "delta";
  request_id: string;
  delta: { text: string };
}
interface AguiThinkingDelta {
  kind: "thinking";
  phase: "delta";
  request_id: string;
  delta: { text: string };
}
interface AguiReasoningDelta {
  kind: "reasoning";
  phase: "delta";
  request_id: string;
  delta: { text: string };
}
interface AguiCitationAdded {
  kind: "citation";
  phase: "added";
  request_id: string;
  citation: { index: number; url?: string; title?: string; snippet?: string };
}
interface AguiToolCallDelta {
  kind: "tool_call";
  phase: "delta";
  request_id: string;
  call_index: number;
  id: string;
  delta: { arguments: string };
}
interface AguiToolCallComplete {
  kind: "tool_call";
  phase: "complete";
  request_id: string;
  call_index: number;
  id: string;
  name: string;
  arguments_json: string;
}
interface AguiToolResult {
  kind: "tool_result";
  request_id: string;
  call_index: number;
  id: string;
  name: string;
  content: string;
  success: boolean;
}
interface AguiError {
  kind: "error";
  request_id: string;
  message: string;
  code?: string;
}
interface AguiSkillActivated {
  kind: "skill";
  phase: "activated";
  request_id: string;
  skill: { id: string; title: string };
  selection_method: string;
}
interface AguiContextUpdate {
  kind: "context";
  phase: "update";
  strategy: string;
  messages_removed: number;
  tokens_saved: number;
  was_applied: boolean;
  summary_generated: boolean;
}
interface AguiMemoryRecallItem {
  key: string;
  value: string;
  source: string;
  scope?: string;
  memory_type?: string;
  importance?: number;
}
interface AguiMemoryRecall {
  kind: "memory";
  phase: "recall";
  request_id: string;
  items: AguiMemoryRecallItem[];
  count?: number;
}
interface AguiMemoryMutation {
  kind: "memory";
  phase: "mutation";
  request_id: string;
  operation: string;
  memory_id: string;
  content: string;
  scope: string;
  memory_type: string;
}
interface AguiMemoryUpdate {
  kind: "memory";
  phase: "update";
  request_id: string;
  key: string;
  value: string;
  operation: string;
}
interface AguiArtifactInputRequest {
  kind: "artifact_input_request";
  request_id: string;
  artifact_id: string;
  artifact_type: string;
  title: string;
  content: string;
  metadata: Record<string, unknown>;
}
interface AguiArtifactDisplay {
  kind: "artifact";
  phase: "complete";
  request_id: string;
  artifact_id: string;
  artifact_type: string;
  title: string;
  content: string;
  language?: string;
  metadata: Record<string, unknown>;
}
interface AguiRaw {
  kind: "raw";
  event?: unknown;
  source?: string;
}
interface AguiCustom {
  kind: "custom";
  name?: string;
  value?: unknown;
  data?: unknown;
}

type AguiPayload =
  | AguiMessageDelta
  | AguiThinkingDelta
  | AguiReasoningDelta
  | AguiCitationAdded
  | AguiToolCallDelta
  | AguiToolCallComplete
  | AguiToolResult
  | AguiError
  | AguiSkillActivated
  | AguiContextUpdate
  | AguiMemoryRecall
  | AguiMemoryMutation
  | AguiMemoryUpdate
  | AguiArtifactInputRequest
  | AguiArtifactDisplay
  | AguiRaw
  | AguiCustom
  | { kind: string; [k: string]: unknown };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function extractA2uiEnvelope(value: unknown): Record<string, unknown> | null {
  if (!isRecord(value)) return null;
  const directKeys = [
    "surfaceUpdate",
    "dataModelUpdate",
    "beginRendering",
    "deleteSurface",
  ];
  if (directKeys.some((k) => k in value)) return value;

  if ("event" in value) {
    const nested = extractA2uiEnvelope(value.event);
    if (nested) return nested;
  }
  if ("value" in value) {
    const nested = extractA2uiEnvelope(value.value);
    if (nested) return nested;
  }
  if ("data" in value) {
    const nested = extractA2uiEnvelope(value.data);
    if (nested) return nested;
  }
  return null;
}

function parseSseBlock(
  raw: string,
): { event: string; data: string; id?: number } | null {
  let event = "message";
  let data = "";
  let id: number | undefined;
  for (const line of raw.split("\n")) {
    if (line.startsWith("event:")) event = line.slice(6).trim();
    else if (line.startsWith("data:")) data = line.slice(5).trim();
    else if (line.startsWith("id:")) {
      const n = Number(line.slice(3).trim());
      if (Number.isFinite(n)) id = n;
    }
  }
  if (!data) return null;
  return { event, data, id };
}

/// Maximum mid-stream reconnect attempts before giving up and finalizing.
const STREAM_MAX_RECONNECTS = 5;

/**
 * Read SSE blocks from a chat stream, yielding `{ event, data }` for each.
 *
 * Owns mid-stream recovery: if the underlying read fails after at least one
 * block has been yielded and the server `runId` is known, it reconnects via the
 * server's resume endpoint (`Last-Event-ID`) and continues — so a dropped
 * connection resumes the remaining response instead of truncating. Reconnection
 * is bounded by `maxReconnects`. Blocks with an id at or below the highest seen
 * are skipped (dedup across the reconnect boundary; the server also filters).
 * Aborts (user stop) propagate immediately.
 */
async function* streamSseBlocks(
  initialReader: ReadableStreamDefaultReader<Uint8Array>,
  opts: { runId: string | null; signal: AbortSignal; maxReconnects: number },
): AsyncGenerator<{ event: string; data: string }> {
  const decoder = new TextDecoder();
  let reader = initialReader;
  let lastEventId = 0;
  let yieldedAny = false;
  let reconnects = 0;

  while (true) {
    let buffer = "";
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) return;
        buffer += decoder.decode(value, { stream: true });
        const blocks = buffer.split("\n\n");
        buffer = blocks.pop() ?? "";
        for (const raw of blocks) {
          if (!raw.trim()) continue;
          const block = parseSseBlock(raw);
          if (!block) continue;
          if (block.id !== undefined) {
            if (block.id <= lastEventId) continue;
            lastEventId = block.id;
          }
          yieldedAny = true;
          yield { event: block.event, data: block.data };
        }
      }
    } catch (err) {
      // User-initiated abort (stop) ends immediately; never reconnect.
      if ((err as Error).name === "AbortError") throw err;
      // Only resume a stream that actually started and whose run we can target.
      if (!yieldedAny || !opts.runId || reconnects >= opts.maxReconnects) {
        throw err;
      }
      reconnects += 1;
      const res = await resumeRunStream(opts.runId, lastEventId, opts.signal);
      if (!res.ok || !res.body) throw err;
      reader = res.body.getReader();
      // Continue the outer loop with the new reader from the resume point.
    }
  }
}

export interface StreamRetryPolicy {
  retriesEnabled: boolean;
  maxAttempts: number;
  streamStartTimeoutMs: number;
  baseDelayMs: number;
  backoffMultiplier: number;
  maxDelayMs: number;
  respectRetryAfter: boolean;
  retryableHttpStatuses: number[];
  retryableTransportErrors: boolean;
  retryBudgetMs: number;
}

export const DEFAULT_STREAM_RETRY_POLICY: StreamRetryPolicy = {
  retriesEnabled: true,
  maxAttempts: 3,
  streamStartTimeoutMs: 15_000,
  baseDelayMs: 1000,
  backoffMultiplier: 2,
  maxDelayMs: 10_000,
  respectRetryAfter: true,
  retryableHttpStatuses: [408, 425, 429, 500, 502, 503, 504],
  retryableTransportErrors: true,
  retryBudgetMs: 20_000,
};

let retryPolicyCache: { value: StreamRetryPolicy; loadedAt: number } | null =
  null;
const RETRY_POLICY_CACHE_TTL_MS = 30_000;
let retryPolicySettingsUnsubscribe: (() => void) | null = null;

function ensureRetryPolicySettingsSubscription() {
  if (retryPolicySettingsUnsubscribe) return;
  retryPolicySettingsUnsubscribe = onSettingsChanged((detail) => {
    if (detail.namespace === "resilience") {
      retryPolicyCache = null;
    }
  });
}

function toNumber(value: unknown, fallback: number): number {
  const n = typeof value === "number" ? value : Number(value);
  return Number.isFinite(n) ? n : fallback;
}

function toBoolean(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

function toNumberArray(value: unknown, fallback: number[]): number[] {
  if (!Array.isArray(value)) return fallback;
  const parsed = value
    .map((entry) => toNumber(entry, Number.NaN))
    .filter((entry) => Number.isFinite(entry));
  return parsed.length > 0 ? parsed : fallback;
}

function retryPolicyFromSettingsResponse(settings: unknown): StreamRetryPolicy {
  const map = new Map<string, unknown>();
  if (Array.isArray(settings)) {
    for (const entry of settings) {
      if (isRecord(entry) && typeof entry.key === "string") {
        map.set(entry.key, entry.data);
      }
    }
  }

  const read = (leafKey: string): unknown => map.get(`resilience.${leafKey}`);
  return {
    retriesEnabled: toBoolean(
      read("retries_enabled"),
      DEFAULT_STREAM_RETRY_POLICY.retriesEnabled,
    ),
    maxAttempts: Math.max(
      1,
      Math.trunc(
        toNumber(
          read("retry_max_attempts"),
          DEFAULT_STREAM_RETRY_POLICY.maxAttempts,
        ),
      ),
    ),
    streamStartTimeoutMs: Math.max(
      1000,
      Math.trunc(
        toNumber(
          read("stream_start_timeout_ms"),
          DEFAULT_STREAM_RETRY_POLICY.streamStartTimeoutMs,
        ),
      ),
    ),
    baseDelayMs: Math.max(
      100,
      Math.trunc(
        toNumber(
          read("retry_base_delay_ms"),
          DEFAULT_STREAM_RETRY_POLICY.baseDelayMs,
        ),
      ),
    ),
    backoffMultiplier: Math.max(
      1.1,
      toNumber(
        read("retry_backoff_multiplier"),
        DEFAULT_STREAM_RETRY_POLICY.backoffMultiplier,
      ),
    ),
    maxDelayMs: Math.max(
      100,
      Math.trunc(
        toNumber(
          read("retry_max_delay_ms"),
          DEFAULT_STREAM_RETRY_POLICY.maxDelayMs,
        ),
      ),
    ),
    respectRetryAfter: toBoolean(
      read("retry_respect_retry_after"),
      DEFAULT_STREAM_RETRY_POLICY.respectRetryAfter,
    ),
    retryableHttpStatuses: toNumberArray(
      read("retryable_http_statuses"),
      DEFAULT_STREAM_RETRY_POLICY.retryableHttpStatuses,
    ),
    retryableTransportErrors: toBoolean(
      read("retryable_transport_errors"),
      DEFAULT_STREAM_RETRY_POLICY.retryableTransportErrors,
    ),
    retryBudgetMs: Math.max(
      0,
      Math.trunc(
        toNumber(
          read("retry_budget_ms"),
          DEFAULT_STREAM_RETRY_POLICY.retryBudgetMs,
        ),
      ),
    ),
  };
}

async function loadStreamRetryPolicy(): Promise<StreamRetryPolicy> {
  ensureRetryPolicySettingsSubscription();
  const now = Date.now();
  if (
    retryPolicyCache &&
    now - retryPolicyCache.loadedAt <= RETRY_POLICY_CACHE_TTL_MS
  ) {
    return retryPolicyCache.value;
  }

  try {
    const body = await fetchResilienceSettings();
    const parsed = retryPolicyFromSettingsResponse(body);
    retryPolicyCache = { value: parsed, loadedAt: now };
    return parsed;
  } catch {
    retryPolicyCache = { value: DEFAULT_STREAM_RETRY_POLICY, loadedAt: now };
    return DEFAULT_STREAM_RETRY_POLICY;
  }
}

class HttpStatusError extends Error {
  status: number;
  statusText: string;
  responseBody: string;
  retryAfterHeader: string | null;

  constructor(
    status: number,
    statusText: string,
    responseBody: string,
    retryAfterHeader: string | null,
  ) {
    super(`HTTP ${status} ${statusText}`);
    this.name = "HttpStatusError";
    this.status = status;
    this.statusText = statusText;
    this.responseBody = responseBody;
    this.retryAfterHeader = retryAfterHeader;
  }
}

function parseRetryAfterMs(
  retryAfterHeader: string | null,
  maxDelayMs: number,
): number | null {
  if (!retryAfterHeader) return null;

  const numericSeconds = Number(retryAfterHeader);
  if (Number.isFinite(numericSeconds) && numericSeconds > 0) {
    return Math.min(Math.round(numericSeconds * 1000), maxDelayMs);
  }

  const dateMs = Date.parse(retryAfterHeader);
  if (Number.isFinite(dateMs)) {
    const deltaMs = dateMs - Date.now();
    if (deltaMs > 0) return Math.min(deltaMs, maxDelayMs);
  }
  return null;
}

export function isRetryableHttpStatus(
  status: number,
  policy: StreamRetryPolicy = DEFAULT_STREAM_RETRY_POLICY,
): boolean {
  return policy.retryableHttpStatuses.includes(status);
}

export function isRetryableTransportError(error: unknown): boolean {
  if (!(error instanceof Error)) return false;
  const name = error.name.toLowerCase();
  const message = error.message.toLowerCase();
  if (name === "aborterror") return false;
  return (
    name.includes("streamstarttimeout") ||
    name.includes("network") ||
    name.includes("typeerror") ||
    message.includes("stream start timeout") ||
    message.includes("failed to fetch") ||
    message.includes("networkerror") ||
    message.includes("network changed") ||
    message.includes("err_network_changed")
  );
}

export function computeRetryDelayMs(
  attempt: number,
  retryAfterHeader: string | null = null,
  policy: StreamRetryPolicy = DEFAULT_STREAM_RETRY_POLICY,
): number {
  const headerDelay = policy.respectRetryAfter
    ? parseRetryAfterMs(retryAfterHeader, policy.maxDelayMs)
    : null;
  if (headerDelay !== null) return headerDelay;
  const expDelay = policy.baseDelayMs * policy.backoffMultiplier ** attempt;
  return Math.min(Math.round(expDelay), policy.maxDelayMs);
}

function shouldRetryStreamAttempt(
  error: unknown,
  attempt: number,
  sawFirstStreamChunk: boolean,
  policy: StreamRetryPolicy,
  cumulativeRetryDelayMs: number,
): { retry: boolean; delayMs: number; reason: string } {
  if (sawFirstStreamChunk)
    return { retry: false, delayMs: 0, reason: "stream_already_started" };
  if (!policy.retriesEnabled)
    return { retry: false, delayMs: 0, reason: "retries_disabled" };
  if (attempt + 1 >= policy.maxAttempts)
    return { retry: false, delayMs: 0, reason: "max_attempts_reached" };
  if (policy.retryBudgetMs <= 0)
    return { retry: false, delayMs: 0, reason: "retry_budget_exceeded" };

  const budgetAllows = (delayMs: number): boolean =>
    cumulativeRetryDelayMs + delayMs <= policy.retryBudgetMs;
  if (error instanceof HttpStatusError) {
    if (!isRetryableHttpStatus(error.status, policy))
      return { retry: false, delayMs: 0, reason: "http_status_not_retryable" };
    const delayMs = computeRetryDelayMs(
      attempt,
      error.retryAfterHeader,
      policy,
    );
    if (!budgetAllows(delayMs))
      return { retry: false, delayMs: 0, reason: "retry_budget_exceeded" };
    return { retry: true, delayMs, reason: "http_retryable" };
  }
  if (policy.retryableTransportErrors && isRetryableTransportError(error)) {
    const delayMs = computeRetryDelayMs(attempt, null, policy);
    if (!budgetAllows(delayMs))
      return { retry: false, delayMs: 0, reason: "retry_budget_exceeded" };
    return { retry: true, delayMs, reason: "transport_retryable" };
  }
  return { retry: false, delayMs: 0, reason: "error_not_retryable" };
}

function abortError(): Error {
  const err = new Error("The operation was aborted.");
  err.name = "AbortError";
  return err;
}

function sleepWithSignal(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal.aborted) {
      reject(abortError());
      return;
    }
    const timeout = setTimeout(() => {
      signal.removeEventListener("abort", onAbort);
      resolve();
    }, ms);
    const onAbort = () => {
      clearTimeout(timeout);
      signal.removeEventListener("abort", onAbort);
      reject(abortError());
    };
    signal.addEventListener("abort", onAbort, { once: true });
  });
}

/** Active stream controller — module scope so async stream loop can assign without React. */
let streamAbortRef: AbortController | null = null;
// Server-assigned run id (from the `x-uar-run-id` response header). Lets the
// stop action request deterministic server-side cancellation rather than
// relying solely on the last-subscriber-drop guard that fires on disconnect.
let serverRunIdRef: string | null = null;

interface ChatStreamActions {
  startStream: (
    threadId: string,
    payload: UarChatPayload,
    callbacks?: StreamCallbacks,
  ) => Promise<void>;
  cancelStream: () => void;
}

export const useChatStreamStore = create<ChatStreamActions>(() => ({
  cancelStream: () => {
    // Request explicit server-side cancellation (deterministic, immediate) in
    // addition to aborting the local stream. The abort alone would also cancel
    // the run via the server's last-subscriber-drop guard, but the explicit
    // call avoids the grace-period delay.
    if (serverRunIdRef) {
      void cancelRun(serverRunIdRef);
      serverRunIdRef = null;
    }
    streamAbortRef?.abort();
    streamAbortRef = null;
  },

  startStream: async (
    threadId: string,
    payload: UarChatPayload,
    callbacks?: StreamCallbacks,
  ): Promise<void> => {
    useChatStreamStore.getState().cancelStream();

    const userMsgId = `user-${Date.now()}`;
    // Build user message content: text + any image attachments for in-thread display
    const imageBlocks = (payload.attachments ?? [])
      .filter((a) => a.content_type.startsWith("image/"))
      .map((a) => ({ type: "image" as const, url: a.url, alt: a.filename }));
    useChatMessageStore.getState().initThread(threadId, [
      ...(useChatMessageStore.getState().messagesByThread[threadId] ?? []),
      {
        id: userMsgId,
        role: "user",
        content: [{ type: "text", text: payload.message }, ...imageBlocks],
        createdAt: new Date(),
        status: "complete",
      },
    ]);

    const runId = `run-${Date.now()}`;
    const pendingArgs = new Map<string, string>();
    useChatMessageStore.getState().beginStream(threadId, runId);
    const retryPolicy = await loadStreamRetryPolicy();
    const maxAttempts = Math.max(1, retryPolicy.maxAttempts);
    let cumulativeRetryDelayMs = 0;
    const requestBody = JSON.stringify({
      message: payload.message,
      stream: true,
      stream_mode: "dual",
      ...(payload.attachments?.length
        ? { attachments: payload.attachments }
        : {}),
      ...(payload.memory_enabled === false ? { memory_enabled: false } : {}),
      // Include agent_id / model if provided so the server uses the correct
      // agent without depending solely on the session agent-config side-channel.
      ...(payload.agent_id ? { agent_id: payload.agent_id } : {}),
      ...(payload.model ? { model: payload.model } : {}),
    });

    for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
      const controller = new AbortController();
      streamAbortRef = controller;
      let sawFirstStreamChunk = false;
      let streamStartTimedOut = false;
      pendingArgs.clear();
      const streamStartTimer = setTimeout(() => {
        if (!sawFirstStreamChunk) {
          streamStartTimedOut = true;
          controller.abort();
        }
      }, retryPolicy.streamStartTimeoutMs);

      try {
        const res = await postChatCompletion(
          requestBody,
          threadId,
          controller.signal,
        );

        if (!res.ok) {
          const body = await res.text().catch(() => "(no response body)");
          throw new HttpStatusError(
            res.status,
            res.statusText,
            body,
            res.headers.get("retry-after"),
          );
        }
        if (!res.body) throw new Error("Response body is null");

        // Capture the server-assigned run id so the stop action can cancel it.
        serverRunIdRef = res.headers.get("x-uar-run-id");

        const reader = res.body.getReader();

        for await (const { event, data } of streamSseBlocks(reader, {
          runId: serverRunIdRef,
          signal: controller.signal,
          maxReconnects: STREAM_MAX_RECONNECTS,
        })) {
            if (event.startsWith("agui.")) {
              let agui: AguiPayload;
              try {
                agui = JSON.parse(data) as AguiPayload;
              } catch {
                continue;
              }

              // Mirror AG-UI frames into the Runtime Console entity graph so the
              // Protocols (AG-UI Events), Memory Activity, and Artifacts panels
              // render live. High-frequency token deltas are excluded to keep the
              // AG-UI events panel readable. Chat rendering below is unaffected.
              if (
                event !== "agui.message.delta" &&
                event !== "agui.thinking.delta" &&
                event !== "agui.reasoning.delta" &&
                event !== "agui.tool_call.delta"
              ) {
                ingestAgUiEvent(runId, {
                  type: event,
                  payload: agui as unknown as Record<string, unknown>,
                });
              }
              if (
                event === "agui.memory.recall" ||
                event === "agui.memory.mutation"
              ) {
                ingestRuntimeEvent({
                  type:
                    event === "agui.memory.recall"
                      ? "memory_recalled"
                      : "memory_updated",
                  run_id: runId,
                  payload: agui as unknown as Record<string, unknown>,
                });
              }
              if (
                event === "agui.artifact" ||
                event === "agui.artifact_input_request"
              ) {
                ingestRuntimeEvent({
                  type: "artifact_created",
                  run_id: runId,
                  payload: agui as unknown as Record<string, unknown>,
                });
              }

              const store = useChatMessageStore.getState();
              if (
                event !== "agui.error" &&
                event !== "agui.done" &&
                event !== "agui.cancelled"
              ) {
                sawFirstStreamChunk = true;
                clearTimeout(streamStartTimer);
                store.markStreamStarted(threadId, runId);
              }

              switch (event) {
                case "agui.message.delta": {
                  const e = agui as AguiMessageDelta;
                  if (e.delta?.text)
                    store.appendTextDelta(threadId, runId, e.delta.text);
                  useAgentStatusStore.getState().setThinking();
                  break;
                }
                case "agui.thinking.delta":
                case "agui.reasoning.delta": {
                  const e = agui as AguiThinkingDelta | AguiReasoningDelta;
                  if (e.delta?.text)
                    store.appendThinkingDelta(threadId, runId, e.delta.text);
                  break;
                }
                case "agui.citation.added": {
                  const e = agui as AguiCitationAdded;
                  if (e.citation)
                    store.addCitation(threadId, runId, {
                      source: e.citation.title ?? e.citation.url ?? "Source",
                      content: e.citation.snippet ?? "",
                      url: e.citation.url,
                    });
                  break;
                }
                case "agui.tool_call.delta": {
                  const e = agui as AguiToolCallDelta;
                  pendingArgs.set(
                    e.id,
                    (pendingArgs.get(e.id) ?? "") + (e.delta?.arguments ?? ""),
                  );
                  break;
                }
                case "agui.tool_call.complete": {
                  const e = agui as AguiToolCallComplete;
                  let args: Record<string, unknown>;
                  try {
                    args = JSON.parse(e.arguments_json) as Record<
                      string,
                      unknown
                    >;
                  } catch {
                    args = { _raw: e.arguments_json };
                  }
                  const toolCall: ToolCallContentBlock = {
                    type: "tool-call",
                    toolCallId: e.id,
                    toolName: e.name,
                    args,
                    status: "running",
                  };
                  store.addToolCall(threadId, runId, toolCall);
                  pendingArgs.delete(e.id);
                  const nameLower = e.name.toLowerCase();
                  if (
                    nameLower.includes("search") ||
                    nameLower.includes("tavily")
                  ) {
                    useAgentStatusStore.getState().setSearching();
                  } else {
                    useAgentStatusStore.getState().setExecuting(e.name);
                  }
                  break;
                }
                case "agui.tool_result": {
                  const e = agui as AguiToolResult;
                  store.updateToolCall(threadId, e.id, {
                    result: e.content,
                    status: e.success ? "complete" : "failed",
                  });
                  break;
                }
                case "agui.error": {
                  const e = agui as AguiError;
                  clearTimeout(streamStartTimer);
                  const ts = new Date().toISOString();
                  const detail = [
                    `[${ts}] Agent stream error`,
                    e.code ? `  Code: ${e.code}` : null,
                    `  Message: ${e.message}`,
                    `  Session: ${threadId}`,
                    `  Request: ${e.request_id}`,
                  ]
                    .filter(Boolean)
                    .join("\n");
                  store.setStreamError(threadId, detail);
                  callbacks?.onError?.(new Error(detail));
                  return;
                }
                case "agui.skill.activated": {
                  const e = agui as AguiSkillActivated;
                  store.addSkillActivation(threadId, runId, {
                    skillId: e.skill.id,
                    skillName: e.skill.title,
                    selectionMethod: e.selection_method,
                    status: "active",
                  });
                  break;
                }
                case "agui.context.update": {
                  const e = agui as AguiContextUpdate;
                  if (e.was_applied)
                    store.addContextUpdate(threadId, runId, {
                      strategy: e.strategy,
                      messagesRemoved: e.messages_removed,
                      tokensSaved: e.tokens_saved,
                      wasApplied: e.was_applied,
                      summaryGenerated: e.summary_generated,
                    });
                  break;
                }
                case "agui.memory.recall": {
                  const e = agui as AguiMemoryRecall;
                  const toolCall: ToolCallContentBlock = {
                    type: "tool-call",
                    toolCallId: `memory-recall-${Date.now()}`,
                    toolName: "__memory_recall__",
                    args: { items: e.items, count: e.count ?? e.items.length },
                    status: "complete",
                  };
                  store.addToolCall(threadId, runId, toolCall);
                  break;
                }
                case "agui.memory.mutation": {
                  const e = agui as AguiMemoryMutation;
                  const toolCall: ToolCallContentBlock = {
                    type: "tool-call",
                    toolCallId: e.memory_id || `memory-mutation-${Date.now()}`,
                    toolName: "__memory_mutation__",
                    args: {
                      operation: e.operation,
                      memoryId: e.memory_id,
                      content: e.content,
                      scope: e.scope,
                      memoryType: e.memory_type,
                    },
                    status: "complete",
                  };
                  store.addToolCall(threadId, runId, toolCall);
                  break;
                }
                case "agui.memory.update": {
                  const e = agui as AguiMemoryUpdate;
                  const toolCall: ToolCallContentBlock = {
                    type: "tool-call",
                    toolCallId: `memory-update-${Date.now()}`,
                    toolName: "__memory_update__",
                    args: {
                      key: e.key,
                      operation: e.operation,
                      value: e.value,
                    },
                    status: "complete",
                  };
                  store.addToolCall(threadId, runId, toolCall);
                  break;
                }
                case "agui.artifact_input_request": {
                  const e = agui as AguiArtifactInputRequest;
                  const toolCall: ToolCallContentBlock = {
                    type: "tool-call",
                    toolCallId: e.artifact_id,
                    toolName: "__a2ui_input__",
                    args: {
                      runId: e.request_id,
                      artifactId: e.artifact_id,
                      artifactType: e.artifact_type,
                      title: e.title,
                      content: e.content,
                      metadata: e.metadata,
                    },
                    status: "running",
                  };
                  store.addToolCall(threadId, runId, toolCall);
                  break;
                }
                case "agui.artifact": {
                  const e = agui as AguiArtifactDisplay;
                  const toolCall: ToolCallContentBlock = {
                    type: "tool-call",
                    toolCallId: e.artifact_id,
                    toolName: "__a2ui_display__",
                    args: {
                      artifactType: e.artifact_type,
                      title: e.title,
                      language: e.language,
                      metadata: e.metadata,
                    },
                    result: e.content,
                    status: "complete",
                  };
                  store.addToolCall(threadId, runId, toolCall);
                  break;
                }
                case "agui.custom":
                case "agui.raw": {
                  const envelope = extractA2uiEnvelope(agui);
                  if (!envelope) break;
                  const envelopeType =
                    "surfaceUpdate" in envelope
                      ? "surfaceUpdate"
                      : "dataModelUpdate" in envelope
                        ? "dataModelUpdate"
                        : "beginRendering" in envelope
                          ? "beginRendering"
                          : "deleteSurface" in envelope
                            ? "deleteSurface"
                            : "chunk";
                  store.addToolCall(threadId, runId, {
                    type: "tool-call",
                    toolCallId: `a2ui-${envelopeType}-${Date.now()}`,
                    toolName: "__a2ui_display__",
                    args: {
                      artifactType: "a2ui/chunk",
                      title: `A2UI ${envelopeType}`,
                      language: "json",
                      metadata: {},
                    },
                    result: JSON.stringify(envelope, null, 2),
                    status: "complete",
                  });
                  break;
                }
                case "agui.stream.start": {
                  const e = agui as { agent_id?: string };
                  if (e.agent_id)
                    store.setMessageMeta(threadId, runId, { agentId: e.agent_id });
                  break;
                }
                case "agui.cancelled": {
                  // Run was cancelled (explicit stop, last-subscriber drop, or
                  // server shutdown). Terminal — finalize the stream cleanly,
                  // distinct from done/error.
                  clearTimeout(streamStartTimer);
                  store.finishStream(threadId);
                  useAgentStatusStore.getState().setIdle();
                  callbacks?.onComplete?.();
                  return;
                }
                case "agui.done": {
                  const e = agui as {
                    usage?: {
                      input_tokens?: number;
                      output_tokens?: number;
                      total_tokens?: number;
                      model?: string | null;
                    };
                  };
                  const u = e.usage;
                  if (u) {
                    store.setMessageMeta(threadId, runId, {
                      model: u.model ?? undefined,
                      usage:
                        u.input_tokens != null || u.output_tokens != null
                          ? {
                              inputTokens: u.input_tokens ?? 0,
                              outputTokens: u.output_tokens ?? 0,
                              totalTokens:
                                u.total_tokens ??
                                (u.input_tokens ?? 0) + (u.output_tokens ?? 0),
                            }
                          : undefined,
                    });
                  }
                  clearTimeout(streamStartTimer);
                  store.finishStream(threadId);
                  useAgentStatusStore.getState().setIdle();
                  callbacks?.onComplete?.();
                  return;
                }
                default:
                  break;
              }
              continue;
            }

            // Runtime entity events — feed the Runtime Console (Cockpit/Runs/Approvals).
            // These are emitted alongside agui.* events from the backend for run/step/
            // tool-call/approval lifecycle. ingestRuntimeEvent() upserts into the entity
            // graph so the Runtime Console panels update in real-time.
            if (event.startsWith("runtime.")) {
              try {
                const payload = JSON.parse(data);
                ingestRuntimeEvent(payload);
              } catch {
                // Malformed runtime event — skip.
              }
              continue;
            }

            if (event === "message" && data === "[DONE]") {
              clearTimeout(streamStartTimer);
              useChatMessageStore.getState().finishStream(threadId);
              useAgentStatusStore.getState().setIdle();
              callbacks?.onComplete?.();
              return;
            }
            sawFirstStreamChunk = true;
            clearTimeout(streamStartTimer);
            useChatMessageStore.getState().markStreamStarted(threadId, runId);
        }

        clearTimeout(streamStartTimer);
        useChatMessageStore.getState().finishStream(threadId);
        callbacks?.onComplete?.();
        return;
      } catch (err) {
        clearTimeout(streamStartTimer);
        if ((err as Error).name === "AbortError" && !streamStartTimedOut) {
          useChatMessageStore.getState().finishStream(threadId);
          return;
        }
        const normalizedError = streamStartTimedOut
          ? Object.assign(
              new Error(
                `Stream start timeout after ${retryPolicy.streamStartTimeoutMs}ms`,
              ),
              { name: "StreamStartTimeoutError" },
            )
          : err;

        const retryDecision = shouldRetryStreamAttempt(
          normalizedError,
          attempt,
          sawFirstStreamChunk,
          retryPolicy,
          cumulativeRetryDelayMs,
        );
        if (retryDecision.retry) {
          console.warn("[uar.stream.retry]", {
            threadId,
            attempt: attempt + 1,
            maxAttempts,
            delayMs: retryDecision.delayMs,
            cumulativeRetryDelayMs,
            reason: retryDecision.reason,
          });
          useChatMessageStore
            .getState()
            .setAwaitingRetry(
              threadId,
              runId,
              attempt + 1,
              maxAttempts,
              retryDecision.delayMs,
            );
          cumulativeRetryDelayMs += retryDecision.delayMs;
          try {
            await sleepWithSignal(retryDecision.delayMs, controller.signal);
            continue;
          } catch (sleepErr) {
            if ((sleepErr as Error).name === "AbortError") {
              useChatMessageStore.getState().finishStream(threadId);
              return;
            }
          }
        }
        console.warn("[uar.stream.retry.stop]", {
          threadId,
          attempt: attempt + 1,
          maxAttempts,
          reason: retryDecision.reason,
          cumulativeRetryDelayMs,
        });

        const error =
          normalizedError instanceof Error
            ? normalizedError
            : new Error(String(normalizedError));
        const ts = new Date().toISOString();
        const detail =
          error instanceof HttpStatusError
            ? `[${ts}] LLM request failed\n` +
              `  URL: POST ${CHAT_COMPLETION_URL}\n` +
              `  Status: ${error.status} ${error.statusText}\n` +
              `  Session: ${threadId}\n` +
              `  Attempts: ${attempt + 1}\n` +
              `  Response body:\n${error.responseBody}`
            : error.message.startsWith("[20")
              ? error.message
              : `[${ts}] Unexpected error\n  ${error.message}\n  Session: ${threadId}`;
        useChatMessageStore.getState().setStreamError(threadId, detail);
        callbacks?.onError?.(new Error(detail));
        return;
      }
    }
  },
}));
