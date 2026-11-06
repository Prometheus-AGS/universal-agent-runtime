import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { getDbInstance } from "@/platform/pglite/client";
import type { PersistedRun, PersistedRunEvent } from "@/platform/pglite/run-event-repository";
import { isUarAguiEvent } from "@/platform/agui/agui-schema";
import { completeAguiUsage, toRuntimeChunk } from "@/platform/agui/runtime-chunk-projection";
import { aguiTerminalUsage } from "@/platform/agui/agui-normalizer";
import { fetchSessionMessages } from "@/services/chat-sessions-api";
import { matchesServerHistory, missingServerMessages } from "@/features/chat/model/session-backfill";
import type {
  ContextUpdateContentBlock,
  ContentBlock,
  MessageUsage,
  RagCitationMarker,
  RichMessage,
  StreamingState,
  ToolCallContentBlock,
} from "@/types/chat-content";
import type { Chunk, SkillActivationChunk } from "@/features/chat/model/chunk";
import { toChunks } from "@/features/chat/model/to-chunks";
import { A2UI_PROFILE } from "@/features/a2ui/a2ui-protocol";
import { AGENT_SELECTION_MIME, agentSelectionChunk, portableSelection, selectionArtifact } from "@/shared/content/agent-selection";
import { artifactDisplayChunk } from "@/shared/content/artifact-display";

// UI hydration ownership only. Concurrent callers await the same database read.
const historyLoads = new Map<string, Promise<void>>();

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

/** Per-message metadata surfaced in the action bar (agent that answered, model, usage). */
export interface MessageMeta {
  agentId?: string;
  model?: string;
  usage?: MessageUsage;
}

interface ChatMessageState {
  messagesByThread: Record<string, RichMessage[]>;
  streamingByThread: Record<string, StreamingState>;
}

interface ChatMessageActions {
  /** Load persisted messages for a thread from PGlite into Zustand. */
  loadMessagesFromDb(threadId: string): Promise<void>;
  /** Append turns the server session holds but the local history lost. */
  backfillFromServer(threadId: string): Promise<void>;
  initThread(threadId: string, messages: RichMessage[]): void;
  beginStream(threadId: string, runId: string): void;
  setAwaitingRetry(threadId: string, runId: string, attempt: number, maxAttempts: number, delayMs: number): void;
  markStreamStarted(threadId: string, runId: string): void;
  appendTextDelta(threadId: string, runId: string, text: string): void;
  appendThinkingDelta(threadId: string, runId: string, text: string, kind?: "thinking" | "reasoning"): void;
  addToolCall(threadId: string, runId: string, toolCall: ToolCallContentBlock): void;
  addRuntimeChunk(threadId: string, runId: string, chunk: Chunk): void;
  updateToolCall(threadId: string, toolCallId: string, update: Partial<Omit<ToolCallContentBlock, "type">>): void;
  addCitation(threadId: string, runId: string, citation: { source: string; content: string; url?: string }): void;
  /** Attach the full numbered citation set ([1], [2], ...) for a RAG-augmented
   * run to the streaming message, so the hover-to-source panel can resolve
   * `[n]` markers appearing in the response text. */
  addRagCitations(threadId: string, runId: string, citations: RagCitationMarker[]): void;
  addSkillActivation(threadId: string, runId: string, skill: { skillId: string; skillName: string; selectionMethod?: string; status: "active" | "complete" }): void;
  addContextUpdate(threadId: string, runId: string, update: Omit<ContextUpdateContentBlock, "type">): void;
  /** Attach agent/model/usage metadata to the in-flight streaming message. Must run before finishStream so it is persisted. */
  setMessageMeta(threadId: string, runId: string, meta: MessageMeta): void;
  finishStream(threadId: string, status?: RichMessage["status"], nativeRunId?: string): Promise<void>;
  setStreamError(threadId: string, error: string, runId?: string): void;
  clearThread(threadId: string): void;
}

type ChatMessageStore = ChatMessageState & ChatMessageActions;

const defaultStreamingState: StreamingState = {
  isStreaming: false,
  runId: null,
  streamingMessageId: null,
  awaitingFirstToken: false,
  retryAttempt: 0,
  retryMaxAttempts: 0,
  retryDelayMs: 0,
};

function ensureThread(state: ChatMessageState, threadId: string): RichMessage[] {
  if (!state.messagesByThread[threadId]) state.messagesByThread[threadId] = [];
  return state.messagesByThread[threadId];
}

function ensureStreaming(state: ChatMessageState, threadId: string): StreamingState {
  if (!state.streamingByThread[threadId]) state.streamingByThread[threadId] = { ...defaultStreamingState };
  return state.streamingByThread[threadId];
}

// Every assistant bubble a stream creates, by thread. A run that pauses or
// changes run id splits one turn across several bubbles; all are saved when the
// stream finishes, because the next send reloads history from PGlite.
const streamBubbles = new Map<string, Set<string>>();

function finalizeBubble(message: RichMessage, runId: string | undefined, status: RichMessage["status"] = "complete"): RichMessage {
  message.status = status;
  message.runId = runId;
  for (const block of message.content) if (block.type === "skill" && block.status === "active") block.status = "complete";
  message.chunks = ensureMessageChunks(message, runId).map((chunk) => {
    if (chunk.kind === "skill-activation" && chunk.status === "active") (chunk as SkillActivationChunk).status = "complete";
    if (chunk.kind === "text") {
      const { text, ...base } = chunk;
      return { ...base, kind: "markdown" as const, source: text };
    }
    return chunk;
  });
  // Snapshot outside the Immer proxy using JSON round-trip.
  // structuredClone() fails on Immer draft proxies; JSON parse/stringify is safe
  // for our plain JSON-serialisable content blocks.
  return JSON.parse(JSON.stringify({ ...message, content: message.content })) as RichMessage;
}

function isSettledMessage(message: RichMessage): boolean {
  return message.status === "complete" || message.status === "failed" || message.status === "cancelled";
}

function getOrCreateStreamingMessage(state: ChatMessageState, threadId: string, runId: string, replayContent = true): RichMessage | undefined {
  const messages = state.messagesByThread[threadId];
  const streaming = state.streamingByThread[threadId];
  if (streaming?.streamingMessageId) {
    const existing = messages?.find((m) => m.id === streaming.streamingMessageId);
    if (existing) {
      if (isSettledMessage(existing)) return;
      // Reattachment may deliver only terminal metadata. Keep the saved partial
      // until actual content replay starts, then rebuild it under the same ID.
      if (existing.status === "interrupted" && replayContent) {
        existing.content = [];
        existing.chunks = [];
        existing.status = "in_progress";
      }
      return existing;
    }
  }
  const msgId = `run-${runId}-assistant-${streamBubbles.get(threadId)?.size ?? 0}`;
  const replayed = messages?.find((message) => message.id === msgId);
  if (replayed) {
    // Retain the cursor on this original bubble so subsequent replay frames
    // cannot create a second bubble after observing settled history.
    state.streamingByThread[threadId] = { ...defaultStreamingState, isStreaming: true, runId, streamingMessageId: msgId };
    if (isSettledMessage(replayed)) return;
    if (replayContent) {
      replayed.content = [];
      replayed.chunks = [];
      replayed.status = "in_progress";
    }
    streamBubbles.set(threadId, (streamBubbles.get(threadId) ?? new Set<string>()).add(msgId));
    return replayed;
  }
  const newMsg: RichMessage = { id: msgId, role: "assistant", runId, content: [], chunks: [], createdAt: new Date(), status: "in_progress" };
  if (!state.messagesByThread[threadId]) state.messagesByThread[threadId] = [];
  state.messagesByThread[threadId].push(newMsg);
  streamBubbles.set(threadId, (streamBubbles.get(threadId) ?? new Set<string>()).add(msgId));
  state.streamingByThread[threadId] = {
    isStreaming: true,
    runId,
    streamingMessageId: msgId,
    awaitingFirstToken: false,
    retryAttempt: 0,
    retryMaxAttempts: 0,
    retryDelayMs: 0,
  };
  return newMsg;
}

function messageAt(message: RichMessage): string {
  return message.createdAt instanceof Date ? message.createdAt.toISOString() : String(message.createdAt);
}

function ensureMessageChunks(message: RichMessage, runId?: string): Chunk[] {
  if (!message.chunks) {
    message.chunks = toChunks(message.content, {
      messageId: message.id,
      at: messageAt(message),
      runId,
      finalized: message.status !== "in_progress",
    });
  }
  return message.chunks;
}

/** Keep known rich boundaries; canonical model history has no card offsets. */
function anchoredHistoryText(cached: string[], canonical: string): string[] {
  const text = cached.join("");
  if (canonical.startsWith(text) || text.startsWith(canonical)) {
    let offset = 0;
    return cached.map((part, index) => {
      const end = index === cached.length - 1 ? canonical.length : offset + part.length;
      const replacement = canonical.slice(offset, end);
      offset = end;
      return replacement;
    });
  }
  const replacements = cached.map(() => "");
  let first = 0;
  let start = 0;
  let last = cached.length - 1;
  let end = canonical.length;
  while (first <= last && canonical.slice(start, end).startsWith(cached[first])) {
    replacements[first] = cached[first];
    start += cached[first].length;
    first += 1;
  }
  while (last >= first && canonical.slice(start, end).endsWith(cached[last])) {
    replacements[last] = cached[last];
    end -= cached[last].length;
    last -= 1;
  }
  // No semantic guess: changed middle text occupies its first old text slot.
  // Independent cards stay in order; unknown intra-text timing is unavailable.
  if (first <= last) replacements[first] = canonical.slice(start, end);
  return replacements;
}

function warmTextHistory(message: RichMessage, remote: RichMessage): Pick<RichMessage, "content" | "chunks"> {
  const canonical = remote.content.flatMap(block => block.type === "text" ? [block.text] : []).join("");
  const contentText = anchoredHistoryText(message.content.flatMap(block => block.type === "text" ? [block.text] : []), canonical);
  let contentIndex = 0;
  const content = message.content.flatMap((block): ContentBlock[] => {
    if (block.type === "text") {
      const text = contentText[contentIndex++];
      return text ? [{ ...block, text }] : [];
    }
    return block.type === "artifact" && block.kind === "application/vnd.uar.history-provenance+json" ? [] : [block];
  });
  if (!contentText.length && canonical) content.unshift({ type: "text", text: canonical });
  content.push(...remote.content.filter(block => block.type !== "text"));

  const cached = ensureMessageChunks({ ...message }, message.runId);
  const textChunks = cached.filter(chunk => chunk.kind === "text" || chunk.kind === "markdown");
  const chunkTexts = anchoredHistoryText(textChunks.map(chunk => chunk.kind === "text" ? chunk.text : chunk.source), canonical);
  const projected = toChunks(remote.content, { messageId: message.id, at: messageAt(remote), runId: remote.runId, finalized: true });
  const remaining = projected.filter(chunk => chunk.kind !== "text" && chunk.kind !== "markdown");
  let chunkIndex = 0;
  const chunks = cached.flatMap((chunk): Chunk[] => {
    if (chunk.kind === "text" || chunk.kind === "markdown") {
      const source = chunkTexts[chunkIndex++];
      return source ? [{ id: chunk.id, seq: chunk.seq, runId: remote.runId, at: messageAt(remote), kind: "markdown", source }] : [];
    }
    if (chunk.kind === "artifact" && chunk.mime === "application/vnd.uar.history-provenance+json") return [];
    const index = remaining.findIndex(row => (chunk.kind === "usage" && row.kind === "usage")
      || (chunk.kind === "tool-approval" && row.kind === "tool-approval" && chunk.approvalId === row.approvalId));
    if (index < 0) return [chunk];
    const [replacement] = remaining.splice(index, 1);
    if (chunk.kind === "tool-approval" && replacement.kind === "tool-approval") {
      return [{ ...chunk, ...replacement, id: chunk.id, seq: chunk.seq, args: chunk.args,
        decision: replacement.decision ?? chunk.decision, decidedAt: replacement.decidedAt ?? chunk.decidedAt,
        historicalState: replacement.historicalState === "unavailable" ? chunk.historicalState : replacement.historicalState }];
    }
    return [{ ...chunk, ...replacement, id: chunk.id, seq: chunk.seq }];
  });
  if (!textChunks.length && canonical) chunks.unshift(...projected.filter(chunk => chunk.kind === "markdown"));
  chunks.push(...remaining);
  return { content, chunks };
}

function chunkBase(message: RichMessage, runId: string) {
  const chunks = ensureMessageChunks(message, runId);
  const seq = chunks.length;
  return { id: `${message.id}:chunk:${seq}`, at: messageAt(message), runId, seq };
}

function chunkText(chunk: Chunk): string | undefined {
  if (chunk.kind === "markdown") return chunk.source;
  if (chunk.kind === "text" || chunk.kind === "thinking" || chunk.kind === "reasoning") return chunk.text;
  return undefined;
}

function recoveryKey(chunk: Chunk): string {
  switch (chunk.kind) {
    case "text": case "markdown": return "text";
    case "thinking": case "reasoning": return "thinking";
    case "tool-call": case "tool-denied": return `${chunk.kind}:${chunk.toolCallId}`;
    case "tool-approval": return `${chunk.kind}:${chunk.approvalId ?? chunk.toolCallId}`;
    case "artifact": return `${chunk.kind}:${chunk.artifactId}`;
    case "a2ui-input": return chunk.toolCallId ? `artifact:${chunk.toolCallId}` : `${chunk.kind}:${chunk.requestId}`;
    case "a2ui-display": return chunk.toolCallId ? `artifact:${chunk.toolCallId}` : `${chunk.kind}:${JSON.stringify(chunk.payload)}`;
    case "skill-activation": return `${chunk.kind}:${chunk.skillId}`;
    case "citation": return `${chunk.kind}:${chunk.source}:${chunk.content}`;
    case "rag-citations": return `${chunk.kind}:${JSON.stringify(chunk.citations)}`;
    case "memory-recall": return `${chunk.kind}:${JSON.stringify(chunk.items)}`;
    case "memory-mutation": return `${chunk.kind}:${chunk.memoryId}:${chunk.operation}`;
    case "context-update": return `${chunk.kind}:${chunk.strategy}:${chunk.messagesRemoved}:${chunk.tokensSaved}`;
    default: return `${chunk.kind}:${chunk.id}`;
  }
}

function recoveryMatch(cached: Chunk, native: Chunk): boolean {
  if (recoveryKey(cached) !== recoveryKey(native)) return false;
  const text = chunkText(cached);
  return text === undefined || chunkText(native)?.startsWith(text) === true;
}

/** ContextAction is host telemetry: its native adapter deliberately emits no
 * run/thread identity. The journal's owning row is correlation, not run proof. */
function isRunlessContextUpdate(event: Record<string, unknown>): boolean {
  if (event.type !== "CUSTOM" || event.name !== "uar.context.updated"
    || event.runId != null || event.threadId != null) return false;
  const value = record(event.value);
  return typeof value.strategy === "string"
    && ["sliding_window", "progressive_summarization", "hierarchical_memory", "keep_first_last", "none"].includes(value.strategy)
    && Number.isInteger(value.messagesRemoved) && (value.messagesRemoved as number) >= 0
    && Number.isInteger(value.tokensSaved) && (value.tokensSaved as number) >= 0
    && typeof value.wasApplied === "boolean" && typeof value.summaryGenerated === "boolean";
}

/** Original deltas are authoritative. Coalesced END rows summarize those same
 * deltas and must never be appended a second time. Reject incomplete/foreign
 * journals rather than turning an observer interruption into native proof. */
function localJournal(run: PersistedRun, events: PersistedRunEvent[]): { chunks: Chunk[]; terminal?: PersistedRunEvent } | undefined {
  const chunks: Chunk[] = [];
  let previousSequence: number | undefined;
  let previousMessage: string | undefined;
  let terminal: PersistedRunEvent | undefined;
  for (const row of events) {
    const event = row.payload;
    const runlessContext = isRunlessContextUpdate(event);
    if (!isUarAguiEvent(event) || row.runId !== run.id || (!runlessContext && event.runId !== run.id)
      || event.eventId !== row.eventId || event.type !== row.type || event.sequence !== row.wireSequence) return undefined;
    if (event.coalesced === true) {
      if (row.type !== "TEXT_MESSAGE_END" && row.type !== "REASONING_MESSAGE_END") return undefined;
      // Synthetic summaries reuse the last delta's sequence. Real END frames
      // have their own sequence and are still part of the native journal.
      if (row.eventId.endsWith(":coalesced")) continue;
    }
    // Native sequence is source cursor * 16; one source event can emit several
    // frames with the same sequence. Gaps and ties are legitimate producer data.
    if (previousSequence === undefined ? row.type !== "RUN_STARTED" : row.wireSequence < previousSequence) return undefined;
    if (row.type === "RUN_STARTED" && previousSequence !== undefined) return undefined;
    if (!Number.isFinite(new Date(row.at).getTime())) return undefined;
    previousSequence = row.wireSequence;
    if (terminal) return undefined;
    // Valid side data still participates in ordering/start/terminal checks, but
    // cannot split text, create a card, or supply terminal/surface authority.
    if (runlessContext) continue;
    if (row.type === "RUN_FINISHED" || row.type === "RUN_ERROR") terminal = row;
    if (row.type === "TEXT_MESSAGE_CONTENT" || row.type === "REASONING_MESSAGE_CONTENT") {
      const messageId = `${run.id}:${row.type === "TEXT_MESSAGE_CONTENT" ? "assistant" : "reasoning"}`;
      if (event.messageId !== messageId || typeof event.delta !== "string") return undefined;
      const chunk = toRuntimeChunk(event, new Date(row.at).getTime());
      if (!chunk) return undefined;
      const last = chunks.at(-1);
      if (last && previousMessage === event.messageId && last.kind === chunk.kind
        && (last.kind === "text" || last.kind === "reasoning")) last.text += event.delta;
      else chunks.push(chunk);
      previousMessage = event.messageId;
      continue;
    }
    // Runtime-only state/lifecycle frames do not split a chat text card. Rich
    // additions do; a result updating an existing tool does not add a card.
    const chunk = toRuntimeChunk(event, new Date(row.at).getTime());
    if (!chunk) continue;
    if ((chunk.kind === "raw" && row.type !== "RUN_FINISHED" && row.type !== "RUN_ERROR")
      || chunk.kind === "state-snapshot" || chunk.kind === "state-delta" || chunk.kind === "step") continue;
    if (chunk.kind === "a2ui-input") {
      const value = record(event.value);
      const artifactId = value.artifactId ?? value.artifact_id;
      if (typeof artifactId === "string") chunk.toolCallId = artifactId;
    }
    if (chunk.kind === "tool-call") {
      const existing = chunks.find((item) => item.kind === "tool-call" && item.toolCallId === chunk.toolCallId);
      if (existing?.kind === "tool-call") {
        if (row.type === "TOOL_CALL_RESULT") { existing.result = chunk.result; existing.status = chunk.status; }
        else Object.assign(existing, chunk, { id: existing.id });
        continue;
      }
    }
    previousMessage = undefined;
    chunks.push(chunk);
  }
  return previousSequence === undefined ? undefined : { chunks, terminal };
}

function portableRecoveryChunk(chunk: Chunk): ContentBlock[] {
  switch (chunk.kind) {
    case "text": return [{ type: "text", text: chunk.text }];
    case "markdown": return [{ type: "text", text: chunk.source }];
    case "thinking": case "reasoning": return [{ type: "thinking", text: chunk.text }];
    case "tool-call": return [{ type: "toolUse", id: chunk.toolCallId, name: chunk.toolName, inputJson: JSON.stringify(chunk.args) },
      ...(chunk.result === undefined ? [] : [{ type: "toolResult" as const, toolUseId: chunk.toolCallId, outputJson: chunk.result, isError: chunk.status === "failed" }])];
    case "artifact": return [{ type: "artifact", id: chunk.artifactId, kind: chunk.mime, title: chunk.title, content: chunk.content ?? "" }];
    case "a2ui-display": return [{ type: "artifact", id: chunk.toolCallId ?? chunk.id,
      kind: "application/vnd.uar.a2ui+json", content: JSON.stringify({ profile: chunk.profile,
        version: chunk.version, component: chunk.component, payload: chunk.payload,
        validation: chunk.validation, validationError: chunk.validationError }) }];
    case "error": return [{ type: "artifact", id: chunk.id, kind: "application/vnd.uar.error+json", content: JSON.stringify({ message: chunk.message, code: chunk.code }) }];
    default: return [];
  }
}

/** Merge ordered native segments into an original bubble, keeping cached-only
 * evidence and IDs. A cached text that is not a prefix has no safe replay join. */
function reconcileLocalBubble(message: RichMessage, replay: Chunk[], status: RichMessage["status"]): RichMessage | undefined {
  const cached = ensureMessageChunks(message, message.runId);
  if (cached.some((chunk) => chunk.runId !== undefined && chunk.runId !== message.runId)) return undefined;
  const positions = new Map<number, number>();
  let cursor = 0;
  for (let index = 0; index < cached.length; index += 1) {
    const chunk = cached[index];
    const match = replay.findIndex((native, nativeIndex) => nativeIndex >= cursor && recoveryMatch(chunk, native));
    if (match < 0) {
      if (chunkText(chunk) !== undefined) return undefined;
      continue;
    }
    positions.set(match, index);
    cursor = match + 1;
  }
  // Associate portable blocks with their cached rich cards without rebuilding
  // the cached content from text or discarding runtime-only approval evidence.
  const blocks = new Map<number, ContentBlock[]>();
  const updatedTextBlocks = new Map<ContentBlock, string>();
  let textCursor = 0;
  const pending: ContentBlock[] = [];
  for (const block of message.content) {
    if (block.type === "text" || block.type === "thinking") {
      const key = block.type === "text" ? "text" : "thinking";
      let aggregate = "";
      let updated = "";
      let first = -1;
      let last = -1;
      for (let index = textCursor; index < cached.length; index += 1) {
        const chunk = cached[index];
        if (recoveryKey(chunk) !== key) {
          if (chunkText(chunk) !== undefined && first >= 0) break;
          continue;
        }
        if (first < 0) first = index;
        aggregate += chunkText(chunk) ?? "";
        const nativeIndex = [...positions].find(([, cachedIndex]) => cachedIndex === index)?.[0];
        updated += chunkText(nativeIndex === undefined ? chunk : replay[nativeIndex]) ?? "";
        last = index;
        if (aggregate === block.text || !block.text.startsWith(aggregate)) break;
      }
      if (first < 0 || aggregate !== block.text) return undefined;
      updatedTextBlocks.set(block, updated);
      blocks.set(first, [...(blocks.get(first) ?? []), ...pending.splice(0), block]);
      textCursor = last + 1;
      continue;
    }
    const projected = toChunks([block], { messageId: message.id, at: messageAt(message), runId: message.runId });
    const index = cached.findIndex((chunk) => {
      if (block.type === "toolResult") return chunk.kind === "tool-call" && chunk.toolCallId === block.toolUseId;
      return projected.some((item) => recoveryKey(item) === recoveryKey(chunk));
    });
    if (index < 0) {
      pending.push(block);
      continue;
    }
    blocks.set(index, [...(blocks.get(index) ?? []), ...pending.splice(0), block]);
  }
  const chunks: Chunk[] = [];
  const content: ContentBlock[] = [];
  let cachedCursor = 0;
  const retain = (index: number) => { chunks.push(cached[index]); content.push(...(blocks.get(index) ?? [])); };
  replay.forEach((native, index) => {
    const cachedIndex = positions.get(index);
    if (cachedIndex === undefined) { chunks.push(native); content.push(...portableRecoveryChunk(native)); return; }
    while (cachedCursor < cachedIndex) retain(cachedCursor++);
    const original = cached[cachedIndex];
    // A historical approval decision is independent evidence. Replay cannot
    // replace it with an awaiting card from its earlier request event.
    const merged: Chunk = chunkText(native) !== undefined
      ? { ...native, id: original.id, at: original.at, seq: original.seq }
      : original.kind === "tool-call" && native.kind === "tool-call"
        && (original.status === "running" || original.status === "pending")
        ? { ...original, result: native.result, status: native.status } : original;
    chunks.push(merged);
    const originalBlocks = blocks.get(cachedIndex) ?? [];
    content.push(...originalBlocks.map((block) => {
      if (block.type === "text" || block.type === "thinking") return { ...block, text: updatedTextBlocks.get(block) ?? block.text };
      if (merged.kind === "tool-call" && block.type === "toolResult" && merged.result !== undefined)
        return { ...block, outputJson: merged.result, isError: merged.status === "failed" };
      return block;
    }));
    if (merged.kind === "tool-call" && merged.result !== undefined && !originalBlocks.some((block) => block.type === "toolResult"))
      content.push({ type: "toolResult", toolUseId: merged.toolCallId, outputJson: merged.result, isError: merged.status === "failed" });
    cachedCursor = cachedIndex + 1;
  });
  while (cachedCursor < cached.length) retain(cachedCursor++);
  content.push(...pending);
  const recovered = { ...message, content, chunks };
  return status === "interrupted" ? { ...recovered, status } : finalizeBubble(recovered, message.runId, status);
}

function recoverLocalRun(run: PersistedRun, events: PersistedRunEvent[], messages: RichMessage[]): RichMessage[] {
  const journal = localJournal(run, events);
  if (!journal) return [];
  const terminal = journal.terminal;
  const terminalStatus = terminal?.type === "RUN_FINISHED" ? "finished"
    : terminal?.payload.code === "CANCELLED" ? "cancelled" : terminal ? "error" : undefined;
  const status: RichMessage["status"] = terminalStatus === run.status
    ? run.status === "finished" ? "complete" : run.status === "error" ? "failed" : "cancelled" : "interrupted";
  const owned = messages.filter((message) => message.role === "assistant" && message.runId === run.id);
  if (!owned.length) {
    if (!journal.chunks.some((chunk) => chunk.kind === "text")) return [];
    const recovered = reconcileLocalBubble({ id: `run-${run.id}-assistant-0`, runId: run.id, role: "assistant",
      content: [], chunks: [], createdAt: new Date(run.startedAt), status: "interrupted" }, journal.chunks, status);
    return recovered ? [recovered] : [];
  }
  // Current native producer has one stable text ID per run; the store groups
  // rich segments in deterministic bubbles. Multiple bubbles require unique
  // ordered cached anchors, rather than assigning all text to the last one.
  const starts: number[] = [];
  let cursor = 0;
  for (const message of owned) {
    if (!/^\d+$/.test(message.id.slice(`run-${run.id}-assistant-`.length))
      || !message.id.startsWith(`run-${run.id}-assistant-`)) return [];
    const first = ensureMessageChunks(message, run.id).find((chunk) => chunkText(chunk) !== undefined
      || journal.chunks.some((native) => recoveryMatch(chunk, native)));
    if (!first) return [];
    const matches = journal.chunks.flatMap((native, index) => index >= cursor && recoveryMatch(first, native) ? [index] : []);
    if (matches.length !== 1) return [];
    starts.push(matches[0]);
    cursor = matches[0] + 1;
  }
  const updates: RichMessage[] = [];
  for (let index = 0; index < owned.length; index += 1) {
    const message = owned[index];
    if (message.status !== "interrupted" && message.status !== "in_progress") continue;
    const replay = journal.chunks.slice(index === 0 ? 0 : starts[index], starts[index + 1]);
    const recovered = reconcileLocalBubble(message, replay, status);
    if (!recovered) return [];
    if (terminal && index === owned.length - 1) {
      const usage = aguiTerminalUsage(terminal.payload);
      recovered.usage = completeAguiUsage(usage);
      if (typeof usage?.model === "string") recovered.model = usage.model;
    }
    updates.push(recovered);
  }
  return updates;
}

export const useChatMessageStore = create<ChatMessageStore>()(
  immer((set, get) => ({
    messagesByThread: {},
    streamingByThread: {},

    loadMessagesFromDb: (threadId) => {
      if (get().streamingByThread[threadId]?.isStreaming) return Promise.resolve();
      const pending = historyLoads.get(threadId);
      if (pending) return pending;
      const load = (async () => {
        const db = getDbInstance();
        const messages: RichMessage[] = (await db.getMessages(threadId)).map((message) => ({ ...message,
          chunks: message.chunks?.map((chunk) => chunk.kind === "tool-approval"
            ? { ...chunk, historical: true, historicalState: chunk.decision ?? "unavailable" } : chunk) }));
        const runs = (await db.getRuns()).filter((run) => run.threadId === threadId);
        for (const run of runs) {
          const events = await db.getRunEvents(run.id);
          // A chunk-only legacy association is not original bubble ownership.
          if (!messages.some((message) => message.runId === run.id)
            && messages.some((message) => message.chunks?.some((chunk) => chunk.runId === run.id))) continue;
          for (const recovered of recoverLocalRun(run, events, messages)) {
            await db.insertMessage(threadId, recovered);
            const index = messages.findIndex((message) => message.id === recovered.id);
            if (index < 0) messages.push(recovered);
            else messages[index] = recovered;
          }
        }
        messages.sort((a, b) => a.historyPosition !== undefined && b.historyPosition !== undefined
          ? a.historyPosition - b.historyPosition : new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime());
        set((state) => {
          // A late mount refresh cannot replace an already-owned live stream.
          if (state.streamingByThread[threadId]?.isStreaming) return;
          state.messagesByThread[threadId] = messages;
          if (!state.streamingByThread[threadId]) state.streamingByThread[threadId] = { ...defaultStreamingState };
        });
      })();
      const owned = load.finally(() => {
        if (historyLoads.get(threadId) === owned) historyLoads.delete(threadId);
      });
      historyLoads.set(threadId, owned);
      return owned;
    },

    backfillFromServer: async (threadId) => {
      if (get().streamingByThread[threadId]?.isStreaming) return;
      const before = get().messagesByThread[threadId] ?? [];
      const server = await fetchSessionMessages(threadId);
      if (!server || !matchesServerHistory(before, server)) return;
      const missing = missingServerMessages(before, server);
      const canonical = missingServerMessages([], server);
      const canonicalUsers = canonical.filter((message) => message.role === "user");
      let userIndex = -1;
      const updates = before.flatMap((message): RichMessage[] => {
        if (message.role === "user") userIndex += 1;
        const remote = canonical.find((row) => row.id === message.id)
          ?? (message.role === "user" ? canonicalUsers[userIndex] : undefined);
        if (!remote) return [];
        // The actual model-history producer supplies text/image content, not
        // live card offsets. Text-only rows can reuse the warm rich anchors.
        if (remote.content.every(block => block.type === "text"
          || (block.type === "artifact" && block.kind === "application/vnd.uar.history-provenance+json"))) {
          return [{ ...message, ...warmTextHistory(message, remote), createdAt: remote.createdAt,
            status: remote.status, runId: remote.runId, clientTurnId: remote.clientTurnId,
            createdAtKnown: remote.createdAtKnown, historyPosition: remote.historyPosition }];
        }
        const hasRemoteImage = remote.content.some((block) => block.type === "image");
        const retained = message.content.filter((block) => block.type !== "text"
          && (block.type !== "image" || !hasRemoteImage)
          && (block.type !== "artifact" || block.kind !== "application/vnd.uar.history-provenance+json")
          && !remote.content.some((row) =>
            (block.type === "toolUse" && row.type === "toolUse" && block.id === row.id)
            || (block.type === "toolResult" && row.type === "toolResult" && block.toolUseId === row.toolUseId)
            || (block.type === "artifact" && row.type === "artifact" && block.id === row.id)));
        const content = [...retained, ...remote.content];
        const projected = toChunks(remote.content, { messageId: message.id, at: messageAt(remote), runId: remote.runId, finalized: true });
        return [{ ...message, createdAt: remote.createdAt, status: remote.status, runId: remote.runId, clientTurnId: remote.clientTurnId, createdAtKnown: remote.createdAtKnown, historyPosition: remote.historyPosition, content,
          chunks: [...ensureMessageChunks({ ...message, createdAt: remote.createdAt }, remote.runId).filter((chunk) => {
            if (chunk.kind === "text" || chunk.kind === "markdown" || (chunk.kind === "image" && hasRemoteImage)) return false;
            if (chunk.kind === "artifact" && chunk.mime === "application/vnd.uar.history-provenance+json") return false;
            if (chunk.kind === "tool-call" && projected.some((row) => row.kind === "tool-call" && row.toolCallId === chunk.toolCallId)) return false;
            if (chunk.kind === "artifact" && projected.some((row) => row.kind === "artifact" && row.artifactId === chunk.artifactId)) return false;
            if (chunk.kind === "usage" && projected.some((row) => row.kind === "usage")) return false;
            return chunk.kind !== "tool-approval" || !projected.some((row) => row.kind === "tool-approval" && row.approvalId === chunk.approvalId);
          }), ...projected] }];
      });
      if (missing.length === 0 && updates.length === 0) return;
      // A send or another load during the request owns the history; leave it.
      if (get().streamingByThread[threadId]?.isStreaming || get().messagesByThread[threadId] !== before) return;
      const db = getDbInstance();
      const reconciled = before.map((message) => updates.find((row) => row.id === message.id) ?? message);
      for (const message of missing) {
        const next = reconciled.findIndex((row) => row.historyPosition !== undefined
          && row.historyPosition > message.historyPosition!);
        if (next >= 0) reconciled.splice(next, 0, message);
        else {
          // Keep a restored final answer ahead of later local-only turns.
          let previous = -1;
          for (let index = reconciled.length - 1; index >= 0; index -= 1) {
            const position = reconciled[index].historyPosition;
            if (position !== undefined && position < message.historyPosition!) {
              previous = index;
              break;
            }
          }
          reconciled.splice(previous < 0 ? reconciled.length : previous + 1, 0, message);
        }
      }
      // Older backfills appended missing rows after later turns. Restore the
      // canonical subsequence while keeping local-only evidence in its slots.
      const ordered = reconciled.filter((message) => message.historyPosition !== undefined)
        .sort((a, b) => a.historyPosition! - b.historyPosition!);
      let orderedIndex = 0;
      const chronological = reconciled.map((message) => message.historyPosition === undefined
        ? message : ordered[orderedIndex++]);
      const approvalsByRun = new Map<string, Set<string>>();
      const usageRuns = new Set<string>();
      const restored = chronological.map((message) => {
        const projected = ensureMessageChunks({ ...message }, message.runId);
        if (!message.runId) return { ...message, chunks: projected };
        const approvals = approvalsByRun.get(message.runId) ?? new Set<string>();
        approvalsByRun.set(message.runId, approvals);
        const chunks = projected.filter((chunk) => {
          if (chunk.kind === "tool-approval") {
            if (approvals.has(chunk.approvalId)) return false;
            approvals.add(chunk.approvalId);
          }
          if (chunk.kind === "usage") {
            if (usageRuns.has(message.runId!)) return false;
            usageRuns.add(message.runId!);
          }
          return true;
        });
        return { ...message, chunks };
      });
      for (const message of restored) await db.insertMessage(threadId, message);
      if (get().streamingByThread[threadId]?.isStreaming || get().messagesByThread[threadId] !== before) return;
      set((state) => {
        state.messagesByThread[threadId] = restored;
      });
    },

    initThread: (threadId, messages) =>
      set((state) => {
        state.messagesByThread[threadId] = messages;
        if (!state.streamingByThread[threadId]) state.streamingByThread[threadId] = { ...defaultStreamingState };
      }),

    beginStream: (threadId, runId) =>
      set((state) => {
        ensureThread(state, threadId);
        const owned = state.messagesByThread[threadId].filter((message) => message.role === "assistant" && message.runId === runId);
        const interrupted = owned.filter((message) => message.status === "interrupted");
        const settled = owned.length > 0 && owned.every(isSettledMessage) ? owned[0] : undefined;
        if (interrupted.length) streamBubbles.set(threadId, new Set(interrupted.map((message) => message.id)));
        state.streamingByThread[threadId] = {
          isStreaming: true,
          runId,
          streamingMessageId: interrupted.at(-1)?.id ?? settled?.id ?? null,
          awaitingFirstToken: true,
          retryAttempt: 0,
          retryMaxAttempts: 0,
          retryDelayMs: 0,
        };
      }),

    setAwaitingRetry: (threadId, runId, attempt, maxAttempts, delayMs) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (streaming.runId !== runId) return;
        streaming.awaitingFirstToken = true;
        streaming.retryAttempt = attempt;
        streaming.retryMaxAttempts = maxAttempts;
        streaming.retryDelayMs = delayMs;
      }),

    markStreamStarted: (threadId, runId) =>
      set((state) => {
        const streaming = ensureStreaming(state, threadId);
        if (streaming.runId !== runId) return;
        streaming.awaitingFirstToken = false;
        streaming.retryAttempt = 0;
        streaming.retryMaxAttempts = 0;
        streaming.retryDelayMs = 0;
      }),

    appendTextDelta: (threadId, runId, text) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        if (!msg) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        const last = messages[idx].content[messages[idx].content.length - 1];
        if (last?.type === "text") (last as { type: "text"; text: string }).text += text;
        else messages[idx].content.push({ type: "text", text });
        const chunks = ensureMessageChunks(messages[idx], runId);
        const lastChunk = chunks.at(-1);
        if (lastChunk?.kind === "text") lastChunk.text += text;
        else chunks.push({ ...chunkBase(messages[idx], runId), kind: "text", text });
      }),

    appendThinkingDelta: (threadId, runId, text, kind = "thinking") =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        if (!msg) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        const last = messages[idx].content[messages[idx].content.length - 1];
        if (last?.type === "thinking") last.text += text;
        else messages[idx].content.push({ type: "thinking", text });
        const chunks = ensureMessageChunks(messages[idx], runId);
        const lastChunk = chunks.at(-1);
        if (lastChunk?.kind === kind) lastChunk.text += text;
        else chunks.push({ ...chunkBase(messages[idx], runId), kind, text });
      }),

    addToolCall: (threadId, runId, toolCall) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        if (!msg) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        const message = messages[idx];
        const chunks = ensureMessageChunks(message, runId);
        const base = chunkBase(message, runId);
        if (toolCall.toolName === "__memory_recall__") {
          const items = Array.isArray(toolCall.args.items) ? toolCall.args.items.map((item, itemIndex) => {
            const memory = item && typeof item === "object" ? item as Record<string, unknown> : {};
            const id = typeof memory.id === "string" ? memory.id : typeof memory.key === "string" ? memory.key : `${base.id}:${itemIndex}`;
            const content = typeof memory.content === "string" ? memory.content : typeof memory.value === "string" ? memory.value : "";
            message.content.push({ type: "memory", operation: "recall", key: id, value: content });
            return { id, content, type: "semantic" as const, importance: typeof memory.importance === "number" ? memory.importance : undefined, score: typeof memory.score === "number" ? memory.score : undefined, pinned: typeof memory.pinned === "boolean" ? memory.pinned : undefined };
          }) : [];
          chunks.push({ ...base, kind: "memory-recall", items });
          return;
        }
        if (toolCall.toolName === "__memory_mutation__") {
          const operation = String(toolCall.args.operation ?? "update");
          const memoryId = String(toolCall.args.memoryId ?? "");
          const content = String(toolCall.args.content ?? "");
          message.content.push({ type: "memory", operation, key: memoryId, value: content || null });
          const normalizedOperation = operation === "create" || operation === "delete" || operation === "pin" || operation === "decay" ? operation : "update";
          chunks.push({ ...base, kind: "memory-mutation", operation: normalizedOperation, memoryId, content: content || undefined, memoryType: typeof toolCall.args.memoryType === "string" ? toolCall.args.memoryType : undefined });
          return;
        }
        if (toolCall.toolName === "__memory_update__") {
          const memoryKey = String(toolCall.args.key ?? "");
          const value = String(toolCall.args.value ?? "");
          message.content.push({ type: "memory", operation: String(toolCall.args.operation ?? "update"), key: memoryKey, value });
          chunks.push({ ...base, kind: "memory-update", scope: "session", summary: value, itemCount: 1 });
          return;
        }
        if (toolCall.toolName === "__a2ui_input__") {
          const inputChunk = { ...base, kind: "a2ui-input" as const, toolCallId: toolCall.toolCallId, profile: A2UI_PROFILE, component: String(toolCall.args.artifactType ?? "input"), requestId: String(toolCall.args.runId ?? toolCall.toolCallId), payload: toolCall.args, status: "awaiting" as const };
          message.content.push({ type: "artifact", id: toolCall.toolCallId, kind: "application/vnd.uar.a2ui-input+json", content: JSON.stringify({ profile: inputChunk.profile, component: inputChunk.component, requestId: inputChunk.requestId, payload: inputChunk.payload, status: inputChunk.status }) });
          chunks.push(inputChunk);
          return;
        }
        if (toolCall.toolName === "__a2ui_display__") {
          const artifactType = typeof toolCall.args.artifactType === "string" ? toolCall.args.artifactType : "";
          const language = typeof toolCall.args.language === "string" ? toolCall.args.language : "";
          const displayChunk = artifactDisplayChunk({ ...toolCall.args, artifactId: toolCall.toolCallId,
            content: toolCall.result }, base);
          if (!displayChunk) {
            const mime = [artifactType, language].some((value) => value === "json" || value === "application/json")
              ? "application/json" : "text/plain";
            const title = typeof toolCall.args.title === "string" ? toolCall.args.title : undefined;
            const content = toolCall.result ?? "";
            message.content.push({ type: "artifact", id: toolCall.toolCallId, kind: mime, title, content });
            chunks.push({ ...base, kind: "artifact", artifactId: toolCall.toolCallId, mime, title, content });
            return;
          }
          message.content.push(...portableRecoveryChunk(displayChunk));
          chunks.push(displayChunk);
          return;
        }
        message.content.push({ type: "toolUse", id: toolCall.toolCallId, name: toolCall.toolName, inputJson: JSON.stringify(toolCall.args) });
        if (toolCall.result !== undefined) message.content.push({ type: "toolResult", toolUseId: toolCall.toolCallId, outputJson: toolCall.result, isError: toolCall.status === "failed" });
        chunks.push({ ...base, kind: "tool-call", toolCallId: toolCall.toolCallId, toolName: toolCall.toolName, args: toolCall.args, result: toolCall.result, status: toolCall.status });
      }),

    addRuntimeChunk: (threadId, runId, chunk) =>
      set((state) => {
        ensureThread(state, threadId);
        if (chunk.kind === "agent-selection") {
          const selection = agentSelectionChunk(chunk, chunk, true);
          if (!selection) return;
          // Validate before deduplication: a malformed reserved ID cannot hide
          // a later genuine notice. Check history before creating a new bubble.
          const duplicate = state.messagesByThread[threadId].some((message) =>
            ensureMessageChunks({ ...message }).some((existing) => existing.kind === "agent-selection"
              && agentSelectionChunk(existing, existing)?.id === selection.id)
            || message.content.some((block) => block.type === "artifact"
              && block.kind === AGENT_SELECTION_MIME
              && selectionArtifact(block.id, block.content, selection)?.id === selection.id));
          if (duplicate) return;
          chunk = selection;
        }
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        }
        const message = getOrCreateStreamingMessage(state, threadId, runId);
        if (!message) return;
        if (chunk.kind === "agent-selection") {
          message.content.push({ type: "artifact", id: chunk.id, kind: AGENT_SELECTION_MIME,
            content: portableSelection(chunk) });
        }
        ensureMessageChunks(message, runId).push(chunk);
      }),

    updateToolCall: (threadId, toolCallId, update) =>
      set((state) => {
        const messages = state.messagesByThread[threadId];
        if (!messages) return;
        for (const msg of messages) {
          if (isSettledMessage(msg)) continue;
          const chunks = ensureMessageChunks(msg);
          const chunk = chunks.find((item) => (
            item.kind === "tool-call"
            || item.kind === "a2ui-input"
            || item.kind === "a2ui-display"
          ) && item.toolCallId === toolCallId);
          if (chunk?.kind === "tool-call") {
            Object.assign(chunk, update);
            if (update.result !== undefined || update.status === "complete" || update.status === "failed") {
              const existingResult = msg.content.find((block) => block.type === "toolResult" && block.toolUseId === toolCallId);
              const result = { type: "toolResult" as const, toolUseId: toolCallId, outputJson: update.result ?? (existingResult?.type === "toolResult" ? existingResult.outputJson : ""), isError: update.status === "failed" };
              if (existingResult) Object.assign(existingResult, result);
              else msg.content.push(result);
            }
            return;
          }
          if (chunk?.kind === "a2ui-input") {
            chunk.status = update.status === "complete" ? "submitted" : update.status === "failed" ? "cancelled" : "awaiting";
            if (update.result !== undefined) chunk.response = update.result;
            const artifact = msg.content.find((block) => block.type === "artifact" && block.id === toolCallId);
            if (artifact?.type === "artifact") artifact.content = JSON.stringify({ profile: chunk.profile, component: chunk.component, requestId: chunk.requestId, payload: chunk.payload, status: chunk.status, response: chunk.response });
            return;
          }
          if (chunk?.kind === "a2ui-display" && update.result !== undefined) {
            const updateArgs = record(update.args);
            const payload = chunk.payload && typeof chunk.payload === "object" && !Array.isArray(chunk.payload)
              ? chunk.payload as Record<string, unknown>
              : {};
            chunk.payload = { ...payload, content: update.result };
            if (updateArgs.validation === "invalid" || updateArgs.validation === "unknown-component") {
              chunk.validation = updateArgs.validation;
              chunk.validationError = typeof updateArgs.validationError === "string" ? updateArgs.validationError : undefined;
            }
            const artifact = msg.content.find((block) => block.type === "artifact" && block.id === toolCallId);
            if (artifact?.type === "artifact") artifact.content = JSON.stringify({ profile: chunk.profile, component: chunk.component, payload: chunk.payload, validation: chunk.validation, validationError: chunk.validationError });
            return;
          }
        }
      }),

    addCitation: (threadId, runId, citation) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        if (!msg) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === undefined || idx === -1) return;
        const message = messages[idx];
        message.content.push({ type: "citation", source: citation.source, quote: citation.content });
        ensureMessageChunks(message, runId).push({ ...chunkBase(message, runId), kind: "citation", source: citation.source, content: citation.content, url: citation.url });
      }),

    addRagCitations: (threadId, runId, citations) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        if (!msg) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === undefined || idx === -1) return;
        const message = messages[idx];
        message.content.push({ type: "artifact", id: `${message.id}:rag:${message.content.length}`, kind: "application/vnd.uar.rag-citations+json", content: JSON.stringify({ citations }) });
        ensureMessageChunks(message, runId).push({ ...chunkBase(message, runId), kind: "rag-citations", citations });
      }),

    addSkillActivation: (threadId, runId, skill) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        if (!msg) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === undefined || idx === -1) return;
        const message = messages[idx];
        message.content.push({ type: "skill", name: skill.skillName, status: skill.status });
        const allowedMethod = skill.selectionMethod === "keyword" || skill.selectionMethod === "embedding" || skill.selectionMethod === "hybrid" || skill.selectionMethod === "llm" || skill.selectionMethod === "explicit" ? skill.selectionMethod : undefined;
        ensureMessageChunks(message, runId).push({ ...chunkBase(message, runId), kind: "skill-activation", skillId: skill.skillId, skillName: skill.skillName, selectionMethod: allowedMethod, status: skill.status });
      }),

    addContextUpdate: (threadId, runId, update) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        if (!msg) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === undefined || idx === -1) return;
        const message = messages[idx];
        message.content.push({ type: "artifact", id: `${message.id}:context:${message.content.length}`, kind: "application/vnd.uar.context-update+json", content: JSON.stringify(update) });
        ensureMessageChunks(message, runId).push({ ...chunkBase(message, runId), kind: "context-update", ...update });
      }),

    setMessageMeta: (threadId, runId, meta) =>
      set((state) => {
        ensureThread(state, threadId);
        ensureStreaming(state, threadId);
        // Resolve (or create) the assistant message this run is producing, so
        // metadata attaches even if it arrives before the first content chunk.
        const msg = getOrCreateStreamingMessage(state, threadId, runId, false);
        if (!msg) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        if (meta.agentId !== undefined) messages[idx].agentId = meta.agentId;
        if (meta.model !== undefined) messages[idx].model = meta.model;
        if (meta.usage !== undefined) messages[idx].usage = meta.usage;
        if (meta.usage !== undefined) {
          const chunks = ensureMessageChunks(messages[idx], runId);
          const existing = chunks.find((chunk) => chunk.kind === "usage");
          const usage = { ...meta.usage, model: meta.model ?? messages[idx].model };
          if (existing?.kind === "usage") Object.assign(existing, usage);
          else chunks.push({ ...chunkBase(messages[idx], runId), kind: "usage" as const, ...usage });
        }
      }),

    finishStream: async (threadId, status = "complete", nativeRunId) => {
      const snapshots: RichMessage[] = [];
      set((state) => {
        const streaming = state.streamingByThread[threadId];
        if (!streaming) return;
        const messages = state.messagesByThread[threadId];
        const bubbleIds = new Set(streamBubbles.get(threadId) ?? []);
        if (streaming.streamingMessageId) bubbleIds.add(streaming.streamingMessageId);
        for (const message of messages ?? []) {
          // Stop can clear observation ownership while a native terminal write
          // is awaiting persistence. Its exact run still owns these partials.
          const belongsToTerminal = nativeRunId !== undefined && message.role === "assistant" && message.runId === nativeRunId;
          if ((nativeRunId !== undefined ? belongsToTerminal : bubbleIds.has(message.id)) && (message.status === "in_progress" || message.status === "interrupted")) {
            snapshots.push(finalizeBubble(message, message.runId ?? streaming.runId ?? undefined, status));
          }
        }
        state.streamingByThread[threadId] = { ...defaultStreamingState };
      });
      streamBubbles.delete(threadId);
      // Persist every completed assistant bubble of this stream to PGlite
      for (const snapshot of snapshots) await getDbInstance().insertMessage(threadId, snapshot);
      // Also update the thread's updated_at timestamp
      await getDbInstance().touchThread(threadId);
    },

    setStreamError: (threadId, error, runId) => {
      let snapshot: RichMessage | undefined;
      set((state) => {
        if (!state.messagesByThread[threadId]) state.messagesByThread[threadId] = [];
        const messages = state.messagesByThread[threadId];
        const streaming = state.streamingByThread[threadId];

        const messageId = streaming?.streamingMessageId ?? (runId
          ? messages.filter((message) => message.role === "assistant" && message.runId === runId && message.status === "failed").at(-1)?.id
          : undefined);
        if (messageId) {
          // In-flight message exists — mark it failed and append error block
          const idx = messages.findIndex((m) => m.id === messageId);
          if (idx !== -1) {
            if ((streaming?.isStreaming && isSettledMessage(messages[idx]))
              || messages[idx].status === "complete" || messages[idx].status === "cancelled") return;
            messages[idx].status = "failed";
            const message = messages[idx];
            message.content.push({ type: "artifact", id: `${message.id}:error`, kind: "application/vnd.uar.error+json", content: JSON.stringify({ message: error }) });
            ensureMessageChunks(message, runId ?? streaming?.runId ?? undefined).push({ ...chunkBase(message, runId ?? streaming?.runId ?? "error"), kind: "error", message: error });
            snapshot = JSON.parse(JSON.stringify(message)) as RichMessage;
          }
        } else {
          // No in-flight message (error hit before stream started) — inject a fresh error bubble
          const errMsg: RichMessage = {
            id: `error-${Date.now()}`,
            role: "assistant",
            content: [{ type: "artifact", id: `error-${Date.now()}`, kind: "application/vnd.uar.error+json", content: JSON.stringify({ message: error }) }],
            chunks: [{ id: `error-${Date.now()}`, kind: "error", at: new Date().toISOString(), seq: 0, message: error }],
            createdAt: new Date(),
            status: "failed",
          };
          messages.push(errMsg);
          snapshot = errMsg;
        }
        state.streamingByThread[threadId] = { ...defaultStreamingState };
      });
      streamBubbles.delete(threadId);
      if (snapshot) void getDbInstance().insertMessage(threadId, snapshot).catch(() => {});
    },

    clearThread: (threadId) =>
      set((state) => {
        delete state.messagesByThread[threadId];
        delete state.streamingByThread[threadId];
      }),

    // expose getter for callers needing to read outside React
    // (already available via .getState())
  })),
);

export const selectThreadMessages = (threadId: string) => (state: ChatMessageStore) =>
  state.messagesByThread[threadId] ?? [];

/** Look up a single message by id within a thread (for per-message metadata rendering). */
export const selectMessageById =
  (threadId: string, messageId: string) => (state: ChatMessageStore) =>
    state.messagesByThread[threadId]?.find((m) => m.id === messageId) ?? null;

export const selectIsStreaming = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.isStreaming ?? false;

export const selectIsAwaitingFirstToken = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.awaitingFirstToken ?? false;

export const selectRetryAttempt = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.retryAttempt ?? 0;

export const selectRetryMaxAttempts = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.retryMaxAttempts ?? 0;

export const selectRetryDelayMs = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.retryDelayMs ?? 0;
