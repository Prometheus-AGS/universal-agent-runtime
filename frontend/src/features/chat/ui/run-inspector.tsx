import { useMemo, useRef, useState } from "react";
import { Check, Clipboard, ExternalLink, Play, RefreshCw, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type {
  RunCheckpoint,
  RunReplayProjection,
  RunTraceContext,
  RunTraceNetworkState,
  RunTraceTreeNode,
  RunEventTiming,
} from "@/features/chat/model/run-trace-types";

function ordered(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(ordered);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, entry]) => [key, ordered(entry)]),
    );
  }
  return value;
}

function json(value: unknown): string {
  return JSON.stringify(ordered(value), null, 2);
}

function eventMessageId(node: RunTraceTreeNode | undefined): string | null {
  if (node?.type !== "event") return null;
  const value = node.event.payload.messageId ?? node.event.payload.message_id;
  return typeof value === "string" && value.length > 0 ? value : null;
}

export function RunInspector({
  context,
  selectedNode,
  timing,
  checkpoints,
  selectedCheckpointId,
  replay,
  network,
  canResume,
  onSelectCheckpoint,
  onRefreshReplay,
  onResume,
  onOpenConversation,
  supplemental,
}: {
  context: RunTraceContext;
  selectedNode: RunTraceTreeNode | undefined;
  timing: RunEventTiming | undefined;
  checkpoints: RunCheckpoint[];
  selectedCheckpointId: string | null;
  replay: RunReplayProjection | null;
  network: RunTraceNetworkState;
  canResume: boolean;
  onSelectCheckpoint: (checkpointId: string) => void;
  onRefreshReplay: () => void;
  onResume: () => void;
  onOpenConversation?: (threadId: string, messageId: string) => void;
  supplemental?: React.ReactNode;
}) {
  const [announcement, setAnnouncement] = useState("");
  const copySequence = useRef(0);
  const messageId = eventMessageId(selectedNode);
  const selectedCheckpoint = checkpoints.find((checkpoint) => checkpoint.id === selectedCheckpointId);
  const payload = selectedNode?.type === "event" ? selectedNode.event.payload : selectedNode ?? null;
  const payloadText = useMemo(() => json(payload), [payload]);
  const rawText = useMemo(() => json(selectedNode?.type === "event"
    ? { type: selectedNode.event.type, ...selectedNode.event.payload }
    : selectedNode ?? null), [selectedNode]);

  const copy = async (value: string): Promise<void> => {
    try {
      await navigator.clipboard.writeText(value);
      copySequence.current += 1;
      setAnnouncement(`Copied inspector JSON (${copySequence.current})`);
    } catch {
      copySequence.current += 1;
      setAnnouncement(`Could not copy inspector JSON (${copySequence.current})`);
    }
  };

  return (
    <aside aria-labelledby="run-inspector-heading" className="flex min-h-0 flex-col gap-3 rounded-xl bg-surface p-3">
      <div>
        <p className="font-mono text-[10px] uppercase tracking-widest text-ember">Selected record</p>
        <h2 id="run-inspector-heading" className="truncate font-display text-base font-semibold text-foreground">
          {selectedNode?.label ?? "Inspector"}
        </h2>
      </div>

      <Tabs defaultValue="payload" className="min-h-0">
        <TabsList variant="line" aria-label="Inspector views" className="min-h-11 w-full justify-start">
          <TabsTrigger className="min-h-11 focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember" value="payload">Payload</TabsTrigger>
          <TabsTrigger className="min-h-11 focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember" value="timing">Timing</TabsTrigger>
          <TabsTrigger className="min-h-11 focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember" value="raw">Raw AG-UI</TabsTrigger>
        </TabsList>
        <TabsContent value="payload" className="min-h-0">
          <pre className="max-h-72 overflow-auto rounded-lg bg-card p-3 font-mono text-[11px] leading-relaxed text-fg-sub">{payloadText}</pre>
          <Button type="button" variant="ghost" size="sm" className="mt-2 min-h-11 focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember" onClick={() => void copy(payloadText)}>
            <Clipboard aria-hidden="true" /> Copy payload
          </Button>
        </TabsContent>
        <TabsContent value="timing">
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 rounded-lg bg-card p-3 text-sm">
            <dt className="text-fg-faint">Start</dt>
            <dd className="truncate font-mono text-fg-sub">{timing?.start ?? "not available"}</dd>
            <dt className="text-fg-faint">Duration</dt>
            <dd className="font-mono text-fg-sub">
              {timing?.durationMs == null ? "instant" : `${timing.durationMs} ms (${timing.durationSource})`}
            </dd>
            <dt className="text-fg-faint">Preceding gap</dt>
            <dd className="font-mono text-fg-sub">{timing ? `${timing.gapMs} ms` : "not available"}</dd>
            <dt className="text-fg-faint">Sequence</dt>
            <dd className="font-mono text-fg-sub">
              {selectedNode?.type === "event"
                ? `${selectedNode.event.seq} durable · ${selectedNode.event.wireSequence} wire`
                : "not available"}
            </dd>
          </dl>
        </TabsContent>
        <TabsContent value="raw" className="min-h-0">
          <pre className="max-h-72 overflow-auto rounded-lg bg-card p-3 font-mono text-[11px] leading-relaxed text-fg-sub">{rawText}</pre>
          <Button type="button" variant="ghost" size="sm" className="mt-2 min-h-11 focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember" onClick={() => void copy(rawText)}>
            <Clipboard aria-hidden="true" /> Copy raw event
          </Button>
        </TabsContent>
      </Tabs>

      {messageId && context.threadId && onOpenConversation && (
        <Button
          type="button"
          variant="ghost"
          className="min-h-11 justify-start bg-card focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember"
          onClick={() => onOpenConversation(context.threadId!, messageId)}
        >
          <ExternalLink aria-hidden="true" /> Open in conversation
        </Button>
      )}

      <section aria-labelledby="checkpoint-heading" className="rounded-lg bg-card p-3">
        <div className="flex items-center justify-between gap-2">
          <h3 id="checkpoint-heading" className="text-sm font-semibold text-foreground">Checkpoints</h3>
          <span className="font-mono text-[10px] text-fg-faint">{checkpoints.length}</span>
        </div>
        {network.checkpoints.status === "error" && (
          <p className="mt-2 text-xs text-destructive">{network.checkpoints.error}</p>
        )}
        {network.checkpoints.status === "loading" && (
          <p className="mt-2 text-xs text-fg-faint" role="status">Loading checkpoints…</p>
        )}
        <div className="mt-2 flex max-h-28 flex-col gap-1 overflow-auto">
          {checkpoints.map((checkpoint) => (
            <button
              key={checkpoint.id}
              type="button"
              aria-pressed={checkpoint.id === selectedCheckpointId}
              className="min-h-11 rounded-lg bg-surface px-2 text-left font-mono text-[10px] text-fg-sub focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember aria-pressed:bg-card-hov aria-pressed:text-foreground"
              onClick={() => onSelectCheckpoint(checkpoint.id)}
            >
              {checkpoint.node_id} · iteration {checkpoint.iteration}
            </button>
          ))}
          {checkpoints.length === 0 && network.checkpoints.status !== "loading" && (
            <p className="py-2 text-xs text-fg-faint">No persisted checkpoints.</p>
          )}
        </div>
        {selectedCheckpoint && (
          <div className="mt-2 rounded-lg bg-surface p-2">
            <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
              <dt className="text-fg-faint">Node</dt>
              <dd className="truncate font-mono text-fg-sub">{selectedCheckpoint.node_id}</dd>
              <dt className="text-fg-faint">Iteration</dt>
              <dd className="font-mono text-fg-sub">{selectedCheckpoint.iteration}</dd>
              <dt className="text-fg-faint">Saved</dt>
              <dd className="truncate font-mono text-fg-sub">{selectedCheckpoint.created_at}</dd>
            </dl>
            <pre className="mt-2 max-h-32 overflow-auto rounded-lg bg-card p-2 font-mono text-[10px] leading-relaxed text-fg-sub">
              {json({ state: selectedCheckpoint.state, messages: selectedCheckpoint.messages })}
            </pre>
          </div>
        )}
        <Button
          type="button"
          size="sm"
          className="mt-2 min-h-11 w-full focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember"
          disabled={!canResume || network.resume.status === "loading"}
          onClick={onResume}
        >
          <Play aria-hidden="true" />
          {network.resume.status === "loading" ? "Resuming…" : "Resume latest checkpoint"}
        </Button>
        {network.agent.status === "loading" && (
          <p className="mt-2 text-xs text-fg-faint" role="status">Resolving runtime agent…</p>
        )}
        {network.agent.status === "error" && (
          <p className="mt-2 text-xs text-destructive">{network.agent.error}</p>
        )}
        {network.resume.status === "error" && <p className="mt-2 text-xs text-destructive">{network.resume.error}</p>}
      </section>

      <section aria-labelledby="replay-heading" className="rounded-lg bg-card p-3">
        <div className="flex items-center justify-between gap-2">
          <div>
            <h3 id="replay-heading" className="text-sm font-semibold text-foreground">A2UI replay</h3>
            <p className="font-mono text-[10px] text-fg-faint">
              {replay ? `${replay.appliedOperations} operations · ${Object.keys(replay.state.surfaces).length} surfaces` : "No replay loaded"}
            </p>
          </div>
          <Button type="button" variant="ghost" size="icon-sm" className="size-11 focus-visible:ring-0 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember" aria-label="Refresh A2UI replay" onClick={onRefreshReplay}>
            <RefreshCw aria-hidden="true" className={network.replay.status === "loading" ? "animate-spin" : undefined} />
          </Button>
        </div>
        <p className="mt-2 flex items-center gap-1 text-xs text-fg-sub">
          {network.replay.status === "error"
            ? <X aria-hidden="true" className="text-destructive" />
            : network.replay.status === "success"
              ? <Check aria-hidden="true" className="text-phase-tool" />
              : <RefreshCw aria-hidden="true" className={network.replay.status === "loading" ? "animate-spin" : undefined} />}
          {network.replay.status === "error"
            ? network.replay.error
            : network.replay.status === "success"
              ? "Validated inert metadata only"
              : network.replay.status === "loading"
                ? "Validating replay metadata…"
                : "Replay metadata not loaded"}
        </p>
      </section>

      {supplemental}
      <p className="sr-only" aria-live="polite">{announcement}</p>
    </aside>
  );
}
