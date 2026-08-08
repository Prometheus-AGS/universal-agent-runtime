import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router";
import {
  Activity,
  Bot,
  Boxes,
  Brain,
  CheckCircle2,
  Clock3,
  FileCode2,
  GitBranch,
  Route,
  ShieldAlert,
  Sparkles,
  TerminalSquare,
  Wrench,
  XCircle,
} from "lucide-react";
import type { EntityType } from "@/platform/entities";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import type {
  RuntimeAgUiEventEntity,
  RuntimeApprovalEntity,
  RuntimeArtifactEntity,
  RuntimeA2uiSurfaceEntity,
  RuntimeMemoryEventEntity,
  RuntimeModelRouteDecisionEntity,
  RuntimeProviderHealthEntity,
  RuntimeRunEntity,
  RuntimeRunStepEntity,
  RuntimeToolCallEntity,
} from "@/entities/types";
import { useGraphEntities } from "@/entities/hooks/use-graph-entities";
import { useRuntimeConsoleActions } from "../model/use-runtime-console";
import { useThreadRegistryActions } from "@/hooks/use-thread-registry-actions";
import { chatMessageAnchorId, RunTracePanel, type RunTraceContext } from "@/features/chat";

function useEntities<T>(type: EntityType): T[] {
  return useGraphEntities<T>(type);
}

function formatTime(value?: string) {
  if (!value) return "pending";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function statusTone(status?: string) {
  switch (status) {
    case "completed":
    case "healthy":
    case "approved":
    case "rendered":
      return "border-emerald-500/25 bg-emerald-500/10 text-emerald-600 dark:text-emerald-300";
    case "running":
    case "waiting":
    case "pending":
      return "border-cyan-500/25 bg-cyan-500/10 text-cyan-600 dark:text-cyan-300";
    case "failed":
    case "offline":
    case "denied":
    case "error":
      return "border-destructive/25 bg-destructive/10 text-destructive";
    case "degraded":
    case "expired":
      return "border-amber-500/25 bg-amber-500/10 text-amber-600 dark:text-amber-300";
    default:
      return "border-border bg-muted text-muted-foreground";
  }
}

function SectionFrame({
  title,
  eyebrow,
  children,
  action,
}: {
  title: string;
  eyebrow: string;
  children: React.ReactNode;
  action?: React.ReactNode;
}) {
  return (
    <section className="flex min-h-0 flex-col border-b border-border bg-background last:border-b-0">
      <div className="flex shrink-0 items-center justify-between border-b border-border px-4 py-3">
        <div>
          <p className="font-mono text-[10px] uppercase tracking-widest text-primary">
            {eyebrow}
          </p>
          <h2 className="font-display text-base font-semibold text-foreground">{title}</h2>
        </div>
        {action}
      </div>
      {children}
    </section>
  );
}

function StatTile({
  label,
  value,
  icon: Icon,
  tone = "text-primary",
}: {
  label: string;
  value: string | number;
  icon: typeof Activity;
  tone?: string;
}) {
  return (
    <div className="border-r border-border bg-card px-4 py-3 last:border-r-0">
      <div className="mb-2 flex items-center justify-between">
        <span className="text-xs text-muted-foreground">{label}</span>
        <Icon size={15} className={tone} />
      </div>
      <div className="font-mono text-2xl font-semibold text-foreground">{value}</div>
    </div>
  );
}

function EmptyRuntimeState({ label }: { label: string }) {
  return (
    <div className="flex min-h-36 flex-col items-center justify-center gap-2 px-6 py-10 text-center">
      <div className="flex size-10 items-center justify-center rounded-md border border-border bg-muted">
        <Sparkles size={16} className="text-muted-foreground" />
      </div>
      <p className="text-sm font-medium text-foreground">{label}</p>
      <p className="max-w-md text-xs leading-5 text-muted-foreground">
        Runtime entities will appear here as SSE, AG-UI, A2UI, provider, and memory events are normalized into the shared graph.
      </p>
    </div>
  );
}

function RunRow({ run, onInspect }: { run: RuntimeRunEntity; onInspect?: (runId: string) => void }) {
  return (
    <div className="grid grid-cols-[1fr_auto] gap-3 border-b border-border px-4 py-3 last:border-b-0 md:grid-cols-[1fr_140px_120px_auto]">
      <div className="min-w-0">
        <p className="truncate text-sm font-semibold text-foreground">
          {run.title ?? run.id}
        </p>
        <p className="truncate font-mono text-xs text-muted-foreground">
          {run.agent_id ?? "agent:auto"} · {run.model ?? "model:auto"}
        </p>
      </div>
      <Badge variant="outline" className={cn("w-fit", statusTone(run.status))}>
        {run.status}
      </Badge>
      <span className="hidden self-center font-mono text-xs text-muted-foreground md:block">
        {formatTime(run.updated_at)}
      </span>
      <Button
        variant="ghost"
        size="sm"
        className="h-7 justify-self-end px-2"
        disabled={!onInspect}
        onClick={() => onInspect?.(run.id)}
      >
        Inspect
      </Button>
    </div>
  );
}

function TimelineRow({ step }: { step: RuntimeRunStepEntity }) {
  // The backend `runtime.step` frame carries `step` (identifier) + `status`
  // (`started`/`finished`) rather than the entity's title/kind/summary, so
  // normalize for display here.
  const raw = step as RuntimeRunStepEntity & {
    step?: string | number;
    event_type?: string;
  };
  const rawStatus = String(step.status ?? "");
  const done = rawStatus === "completed" || rawStatus === "finished";
  const failed = rawStatus === "failed" || rawStatus === "error";
  const title =
    (typeof step.title === "string" && step.title) ||
    (raw.step != null ? `Step ${raw.step}` : raw.event_type ?? "Runtime step");
  const detail =
    step.summary ?? (done ? "completed" : failed ? "failed" : "in progress");
  return (
    <div className="grid grid-cols-[24px_1fr_auto] gap-3 border-b border-border px-4 py-3 last:border-b-0">
      <div className="mt-0.5 flex size-6 items-center justify-center rounded border border-border bg-muted">
        {done ? (
          <CheckCircle2 size={13} className="text-emerald-500" />
        ) : failed ? (
          <XCircle size={13} className="text-destructive" />
        ) : (
          <Clock3 size={13} className="text-primary" />
        )}
      </div>
      <div className="min-w-0">
        <p className="truncate text-sm font-medium text-foreground">{title}</p>
        <p className="truncate text-xs text-muted-foreground">
          {step.kind ?? "step"} · {detail}
        </p>
      </div>
      <Badge variant="outline" className={cn("h-6", statusTone(step.status))}>
        {rawStatus || "pending"}
      </Badge>
    </div>
  );
}

export function RuntimeCockpitPage() {
  const runs = useEntities<RuntimeRunEntity>("RuntimeRun");
  const steps = useEntities<RuntimeRunStepEntity>("RuntimeRunStep");
  const tools = useEntities<RuntimeToolCallEntity>("RuntimeToolCall");
  const approvals = useEntities<RuntimeApprovalEntity>("RuntimeApproval");
  const memory = useEntities<RuntimeMemoryEventEntity>("RuntimeMemoryEvent");
  const health = useEntities<RuntimeProviderHealthEntity>("RuntimeProviderHealth");
  const navigate = useNavigate();

  const running = runs.filter((run) => run.status === "running" || run.status === "waiting");
  const pendingApprovals = approvals.filter((approval) => approval.status === "pending");

  // No run-detail column exists on this page — inspecting a run means
  // handing off to the Runs page with that run preselected.
  const inspectRun = useCallback(
    (runId: string) => navigate(`/admin/runs?run=${encodeURIComponent(runId)}`),
    [navigate],
  );

  return (
    <ScrollArea className="flex-1">
      <div className="grid min-h-full grid-cols-1 xl:grid-cols-[1fr_320px]">
        <div className="min-w-0">
          <div className="grid border-b border-border md:grid-cols-4">
            <StatTile label="Active runs" value={running.length} icon={Activity} />
            <StatTile label="Tool calls" value={tools.length} icon={Wrench} tone="text-cyan-500" />
            <StatTile label="Approvals" value={pendingApprovals.length} icon={ShieldAlert} tone="text-amber-500" />
            <StatTile label="Memory events" value={memory.length} icon={Brain} tone="text-emerald-500" />
          </div>

          <SectionFrame title="Live Runs" eyebrow="Runtime">
            {runs.length > 0 ? runs.slice(0, 8).map((run) => <RunRow key={run.id} run={run} onInspect={inspectRun} />) : (
              <EmptyRuntimeState label="No runtime runs observed yet" />
            )}
          </SectionFrame>

          <SectionFrame title="Execution Timeline" eyebrow="Steps">
            {steps.length > 0 ? steps.slice(0, 10).map((step) => <TimelineRow key={step.id} step={step} />) : (
              <EmptyRuntimeState label="No run steps in the graph yet" />
            )}
          </SectionFrame>
        </div>

        <aside className="border-l border-border bg-card/60">
          <SectionFrame title="Provider Health" eyebrow="Providers">
            {health.length > 0 ? health.map((row) => (
              <div key={row.id} className="flex items-center justify-between border-b border-border px-4 py-3 last:border-b-0">
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium text-foreground">{row.provider_id}</p>
                  <p className="font-mono text-xs text-muted-foreground">
                    {row.latency_ms ? `${row.latency_ms}ms` : "latency pending"}
                  </p>
                </div>
                <Badge variant="outline" className={cn(statusTone(row.status))}>{row.status}</Badge>
              </div>
            )) : (
              <EmptyRuntimeState label="No provider health reported yet" />
            )}
          </SectionFrame>

          <SectionFrame title="Workflow State" eyebrow="KBD + OpenSpec">
            <div className="space-y-3 px-4 py-4">
              {[
                ["OpenSpec", "runtime-console-entity-workflow"],
                ["KBD phase", "runtime-console-ux"],
                ["Memory mirror", "surreal_memory /mcp/memory"],
              ].map(([label, value]) => (
                <div key={label} className="rounded-md border border-border bg-background px-3 py-2">
                  <p className="text-xs text-muted-foreground">{label}</p>
                  <p className="truncate font-mono text-xs text-foreground">{value}</p>
                </div>
              ))}
            </div>
          </SectionFrame>

          <SectionFrame title="Memory Activity" eyebrow={`${memory.length} events`}>
            {memory.length > 0 ? memory.slice(0, 6).map((event) => (
              <div key={event.id} className="border-b border-border px-4 py-3 last:border-b-0">
                <div className="flex items-center justify-between gap-3">
                  <p className="truncate text-sm font-medium text-foreground">{event.action}</p>
                  <span className="font-mono text-xs text-muted-foreground">{event.source_tool ?? "runtime"}</span>
                </div>
                <p className="truncate text-xs text-muted-foreground">{event.summary}</p>
              </div>
            )) : (
              <EmptyRuntimeState label="No memory activity observed yet" />
            )}
          </SectionFrame>
        </aside>
      </div>
    </ScrollArea>
  );
}

export function RuntimeRunsPage() {
  const runs = useEntities<RuntimeRunEntity>("RuntimeRun");
  const tools = useEntities<RuntimeToolCallEntity>("RuntimeToolCall");
  const artifacts = useEntities<RuntimeArtifactEntity>("RuntimeArtifact");
  const navigate = useNavigate();
  const { setActiveThread } = useThreadRegistryActions();
  const [searchParams, setSearchParams] = useSearchParams();
  const [selectedRunId, setSelectedRunId] = useState<string | null>(searchParams.get("run"));

  // Preselect from ?run=<id> whenever it changes (e.g. arriving via the
  // Cockpit's Inspect handoff); falls back to the first run once loaded.
  useEffect(() => {
    const fromQuery = searchParams.get("run");
    if (fromQuery) setSelectedRunId(fromQuery);
  }, [searchParams]);

  const selectedRun = selectedRunId
    ? runs.find((run) => run.id === selectedRunId)
    : runs[0];
  const selectRun = useCallback((runId: string) => {
    setSelectedRunId(runId);
    setSearchParams((current) => {
      const next = new URLSearchParams(current);
      next.set("run", runId);
      return next;
    });
  }, [setSearchParams]);
  const selectedArtifacts = useMemo(
    () => artifacts.filter((artifact) => artifact.run_id === selectedRun?.id),
    [artifacts, selectedRun?.id],
  );
  const selectedTools = useMemo(
    () => tools.filter((tool) => tool.run_id === selectedRun?.id),
    [tools, selectedRun?.id],
  );
  const traceContext = useMemo<RunTraceContext | null>(() => selectedRun ? {
    runId: selectedRun.id,
    threadId: selectedRun.thread_id ?? null,
    messageId: typeof selectedRun.message_id === "string" ? selectedRun.message_id : null,
    agentId: selectedRun.agent_id ?? null,
    sessionId: typeof selectedRun.session_id === "string" ? selectedRun.session_id : null,
  } : null, [selectedRun]);

  const openConversation = useCallback(
    (threadId: string, messageId: string) => {
      setActiveThread(threadId);
      navigate("/threads");
      let attempts = 0;
      const focusMessage = (): void => {
        const element = document.getElementById(chatMessageAnchorId(messageId));
        if (element) {
          element.scrollIntoView({ block: "center" });
          element.focus({ preventScroll: true });
          return;
        }
        attempts += 1;
        if (attempts < 60) window.requestAnimationFrame(focusMessage);
      };
      window.requestAnimationFrame(focusMessage);
    },
    [navigate, setActiveThread],
  );

  return (
    <div className="grid flex-1 grid-cols-1 overflow-hidden lg:grid-cols-[360px_minmax(0,1fr)]">
      <ScrollArea className="border-r border-border">
        <SectionFrame title="Runs" eyebrow={`${runs.length} observed`}>
          {runs.length > 0 ? runs.map((run) => (
            <RunRow key={run.id} run={run} onInspect={selectRun} />
          )) : (
            <EmptyRuntimeState label="No runs are available" />
          )}
        </SectionFrame>
      </ScrollArea>

      {traceContext ? (
        <RunTracePanel
          context={traceContext}
          onOpenConversation={openConversation}
          onRunHandoff={selectRun}
          supplemental={(
            <div className="grid gap-2">
              <section aria-labelledby="run-artifacts-heading" className="rounded-lg bg-card p-3">
                <h3 id="run-artifacts-heading" className="text-sm font-semibold text-foreground">
                  Artifacts · {selectedArtifacts.length}
                </h3>
                <div className="mt-2 grid gap-1">
                  {selectedArtifacts.map((artifact) => (
                    <div key={artifact.id} className="rounded-lg bg-surface px-2 py-2">
                      <p className="truncate text-xs font-medium text-foreground">{artifact.title}</p>
                      <p className="truncate font-mono text-[10px] text-fg-faint">{artifact.kind} · {artifact.mime_type ?? "unknown"}</p>
                    </div>
                  ))}
                  {selectedArtifacts.length === 0 && <p className="text-xs text-fg-faint">No artifacts for this run.</p>}
                </div>
              </section>
              <section aria-labelledby="run-tools-heading" className="rounded-lg bg-card p-3">
                <h3 id="run-tools-heading" className="text-sm font-semibold text-foreground">
                  Tool calls · {selectedTools.length}
                </h3>
                <div className="mt-2 grid gap-1">
                  {selectedTools.map((tool) => (
                    <div key={tool.id} className="rounded-lg bg-surface px-2 py-2">
                      <div className="flex items-center justify-between gap-2">
                        <p className="truncate text-xs font-medium text-foreground">{tool.tool_name}</p>
                        <span className="font-mono text-[10px] text-fg-faint">{tool.status}</span>
                      </div>
                      <p className="truncate font-mono text-[10px] text-fg-faint">{tool.namespace ?? "internal"}</p>
                    </div>
                  ))}
                  {selectedTools.length === 0 && <p className="text-xs text-fg-faint">No tool calls for this run.</p>}
                </div>
              </section>
            </div>
          )}
        />
      ) : (
        <EmptyRuntimeState label={selectedRunId ? "Waiting for the selected run to appear" : "Select a run to inspect its trace"} />
      )}
    </div>
  );
}

export function RuntimeApprovalsPage() {
  const approvals = useEntities<RuntimeApprovalEntity>("RuntimeApproval");
  const { resolveApproval } = useRuntimeConsoleActions();

  return (
    <ScrollArea className="flex-1">
      <div className="mx-auto w-full max-w-5xl px-4 py-4">
        <div className="mb-4 flex items-center justify-between">
          <div>
            <p className="font-mono text-[10px] uppercase tracking-widest text-primary">Interrupts</p>
            <h1 className="font-display text-xl font-semibold">Tool Approvals</h1>
          </div>
          <Badge variant="outline" className={statusTone("pending")}>
            {approvals.filter((approval) => approval.status === "pending").length} pending
          </Badge>
        </div>
        <div className="overflow-hidden rounded-md border border-border bg-card">
          {approvals.length > 0 ? approvals.map((approval) => (
            <div key={approval.id} className="grid gap-3 border-b border-border px-4 py-3 last:border-b-0 md:grid-cols-[1fr_auto_auto] md:items-center">
              <div className="min-w-0">
                <p className="truncate text-sm font-semibold text-foreground">{approval.id}</p>
                <p className="truncate text-xs text-muted-foreground">
                  run {approval.run_id} · tool {(approval as unknown as Record<string, string>)["tool_name"] ?? approval.tool_call_id ?? "unknown"}
                </p>
              </div>
              <Badge variant="outline" className={cn("w-fit", statusTone(approval.status))}>
                {approval.status}
              </Badge>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  disabled={approval.status !== "pending"}
                  onClick={() => void resolveApproval(approval, false)}
                >
                  Deny
                </Button>
                <Button
                  size="sm"
                  disabled={approval.status !== "pending"}
                  onClick={() => void resolveApproval(approval, true)}
                >
                  Approve
                </Button>
              </div>
            </div>
          )) : (
            <EmptyRuntimeState label="No approvals are waiting" />
          )}
        </div>
      </div>
    </ScrollArea>
  );
}

export function RuntimeProtocolsPage() {
  const agUi = useEntities<RuntimeAgUiEventEntity>("RuntimeAgUiEvent");
  const routes = useEntities<RuntimeModelRouteDecisionEntity>("RuntimeModelRouteDecision");
  const surfaces = useEntities<RuntimeA2uiSurfaceEntity>("RuntimeA2uiSurface");

  const protocolCards = [
    { label: "Anthropic REST", value: "/v1/messages", icon: Bot, detail: "Prompt caching and Claude Code compatibility diagnostics" },
    { label: "OpenAI REST", value: "/v1/chat/completions", icon: TerminalSquare, detail: "OpenAI-compatible chat and model catalog surface" },
    { label: "AG-UI", value: `${agUi.length} events`, icon: GitBranch, detail: "Run, step, text, tool, state, message, and raw events" },
    { label: "A2UI", value: `${surfaces.length} surfaces`, icon: FileCode2, detail: "Generated UI schemas and renderable artifacts" },
    { label: "MCP", value: "/mcp/memory", icon: Boxes, detail: "Surreal Memory workflow mirror and runtime memory tools" },
    { label: "liter-llm", value: `${routes.length} route decisions`, icon: Route, detail: "Provider/model routing with capability requirements" },
  ];

  return (
    <ScrollArea className="flex-1">
      <div className="grid min-h-full grid-cols-1 xl:grid-cols-[1fr_360px]">
        <div className="p-4">
          <div className="mb-4">
            <p className="font-mono text-[10px] uppercase tracking-widest text-primary">Protocol Surface</p>
            <h1 className="font-display text-xl font-semibold">Compatibility Console</h1>
          </div>
          <div className="grid gap-3 md:grid-cols-2">
            {protocolCards.map(({ label, value, icon: Icon, detail }) => (
              <div key={label} className="rounded-md border border-border bg-card p-4">
                <div className="mb-3 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Icon size={16} className="text-primary" />
                    <h2 className="text-sm font-semibold text-foreground">{label}</h2>
                  </div>
                  <Badge variant="outline">{value}</Badge>
                </div>
                <p className="text-sm leading-6 text-muted-foreground">{detail}</p>
              </div>
            ))}
          </div>
        </div>
        <aside className="border-l border-border bg-card/60">
          <SectionFrame title="Recent AG-UI Events" eyebrow="Event stream">
            {agUi.length > 0 ? agUi.slice(0, 12).map((event) => (
              <div key={event.id} className="border-b border-border px-4 py-3 last:border-b-0">
                <div className="flex items-center justify-between gap-3">
                  <p className="truncate text-sm font-medium text-foreground">{event.event_type}</p>
                  <span className="font-mono text-xs text-muted-foreground">#{event.sequence}</span>
                </div>
                <p className="truncate font-mono text-xs text-muted-foreground">{event.run_id}</p>
              </div>
            )) : (
              <EmptyRuntimeState label="No AG-UI events observed yet" />
            )}
          </SectionFrame>
          <SectionFrame title="Model Routing" eyebrow="liter-llm">
            {routes.length > 0 ? routes.slice(0, 8).map((route) => (
              <div key={route.id} className="border-b border-border px-4 py-3 last:border-b-0">
                <p className="truncate text-sm font-medium text-foreground">{route.selected_model}</p>
                <p className="truncate text-xs text-muted-foreground">{route.reason ?? route.selected_provider ?? "route reason pending"}</p>
              </div>
            )) : (
              <EmptyRuntimeState label="No routing decisions yet" />
            )}
          </SectionFrame>
          <SectionFrame title="A2UI Surfaces" eyebrow="Protocol UI">
            {surfaces.length > 0 ? surfaces.slice(0, 8).map((surface) => (
              <div key={surface.id} className="border-b border-border px-4 py-3 last:border-b-0">
                <div className="flex items-center justify-between gap-3">
                  <p className="truncate text-sm font-medium text-foreground">{surface.title}</p>
                  <Badge variant="outline" className={cn(statusTone(surface.status))}>{surface.status}</Badge>
                </div>
                <p className="truncate text-xs text-muted-foreground">
                  {surface.schema_id ?? "schema:auto"} · {String(surface.payload?.body ?? "payload pending")}
                </p>
              </div>
            )) : (
              <EmptyRuntimeState label="No A2UI surfaces registered yet" />
            )}
          </SectionFrame>
        </aside>
      </div>
    </ScrollArea>
  );
}
