import { type FC, useEffect, useState } from "react";
import { AlertTriangle, ArrowLeft, Bot, Brain, Loader2, Pencil, Plus, RefreshCw, Sparkles, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { AdminEmptyInline, AdminError, AdminSidebarSkeleton } from "@/admin/components/admin-states";
import { AgentAiBuilder } from "@/admin/components/agent-ai-builder";
import { AgentEditor } from "@/admin/components/agent-editor";
import { cn } from "@/lib/utils";
import { deleteAgent, patchAgent as patchAgentApi } from "@/services/agents-api";
import { loadAgentsIntoGraph } from "@/entities/fetchers/agents";
import { useAgents } from "@/entities/hooks/use-agents";
import { optimisticUpsert, optimisticRemove } from "@/lib/realtime/optimistic";
import type { UarAgent } from "@/types";

// ── Agent Memory Section ───────────────────────────────────────────────────

interface AgentMemoryState {
  memory_enabled: boolean | null;   // null = inherit global
  auto_capture: boolean | null;
  inject_context: boolean | null;
  memory_scope: string;
}

const SCOPE_OPTIONS = [
  { value: "agent", label: "Agent-scoped" },
  { value: "user", label: "User-scoped" },
  { value: "global", label: "Global" },
  { value: "session", label: "Session-scoped" },
];

type TriValue = "inherit" | "on" | "off";

function encodeTriValue(v: boolean | null): TriValue {
  if (v === null) return "inherit";
  return v ? "on" : "off";
}

function decodeTriValue(s: string): boolean | null {
  if (s === "on") return true;
  if (s === "off") return false;
  return null;
}

function TriToggle({ value, onChange }: { value: boolean | null; onChange: (v: boolean | null) => void }) {
  return (
    <ToggleGroup
      value={[encodeTriValue(value)]}
      onValueChange={(vals) => { const s = vals[0]; if (s) onChange(decodeTriValue(s)); }}
      className="gap-0 overflow-hidden rounded-lg border border-border"
    >
      {(["inherit", "on", "off"] as TriValue[]).map((v) => (
        <ToggleGroupItem
          key={v}
          value={v}
          className="rounded-none border-0 px-3 py-1 font-mono text-xs capitalize data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
        >
          {v.charAt(0).toUpperCase() + v.slice(1)}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}

function AgentMemorySection({ agent }: { agent: UarAgent }) {
  const [state, setState] = useState<AgentMemoryState>({
    memory_enabled: null,
    auto_capture: null,
    inject_context: null,
    memory_scope: (agent as unknown as Record<string, unknown>).memory_scope as string ?? "agent",
  });
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const save = async () => {
    setSaving(true);
    setError(null);
    try {
      const body: Record<string, unknown> = {};
      if (state.memory_enabled !== null) body.memory_enabled = state.memory_enabled;
      if (state.auto_capture !== null) body.memory_auto_capture = state.auto_capture;
      if (state.inject_context !== null) body.memory_inject_context = state.inject_context;
      body.memory_scope = state.memory_scope;

      await optimisticUpsert("Agent", agent.id, body, async () => {
        await patchAgentApi(agent.id, body);
      });
      setSaved(true);
      setTimeout(() => setSaved(false), 2500);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="mt-6">
      <div className="mb-3 flex items-center gap-2">
        <Brain size={14} className="text-primary" />
        <p className="font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground">
          Memory (per-agent override)
        </p>
      </div>

      <Card className="rounded-lg border-border shadow-none">
        <CardContent className="space-y-3 p-4">
          {/* Memory Enabled */}
          <div className="flex items-center justify-between">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">Memory Enabled</p>
              <p className="mt-0.5 font-mono text-xs text-muted-foreground">Override global memory on/off for this agent.</p>
            </div>
            <TriToggle value={state.memory_enabled} onChange={(v) => setState((s) => ({ ...s, memory_enabled: v }))} />
          </div>

          {/* Auto-Capture */}
          <div className="flex items-center justify-between">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">Auto-Capture</p>
              <p className="mt-0.5 font-mono text-xs text-muted-foreground">Extract memories after each turn.</p>
            </div>
            <TriToggle value={state.auto_capture} onChange={(v) => setState((s) => ({ ...s, auto_capture: v }))} />
          </div>

          {/* Context Injection */}
          <div className="flex items-center justify-between">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">Context Injection</p>
              <p className="mt-0.5 font-mono text-xs text-muted-foreground">Inject memories as system prompt prefix.</p>
            </div>
            <TriToggle value={state.inject_context} onChange={(v) => setState((s) => ({ ...s, inject_context: v }))} />
          </div>

          {/* Default Scope */}
          <div className="flex items-center justify-between">
            <div>
              <p className="font-mono text-xs font-medium text-foreground">Default Scope</p>
              <p className="mt-0.5 font-mono text-xs text-muted-foreground">Scope for memories saved by this agent.</p>
            </div>
            <Select
              value={state.memory_scope}
              onValueChange={(v) => v !== null && setState((s) => ({ ...s, memory_scope: v }))}
            >
              <SelectTrigger className="h-8 w-36 font-mono text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SCOPE_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value} className="font-mono text-xs">
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {error && <AdminError error={error} />}

          <div className="flex items-center gap-2 pt-1">
            <Button size="sm" onClick={() => void save()} disabled={saving} className="gap-1.5">
              {saving ? <Loader2 size={12} className="animate-spin" /> : null}
              Save Memory Settings
            </Button>
            {saved && (
              <span className="font-mono text-xs text-green-400">Saved</span>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

/** Returns true if an agent has no model configured in its policy. */
function agentLacksModel(a: UarAgent): boolean {
  const raw = a as unknown as Record<string, unknown>;
  const policy = (raw.policy as Record<string, unknown>) ?? {};
  const providerPolicy = (policy.provider as Record<string, unknown>) ?? {};
  const def = (providerPolicy.default as Record<string, unknown>) ?? {};
  return !def.provider || !def.model;
}

// ── Main Agents Page ───────────────────────────────────────────────────────

export const AgentsPage: FC = () => {
  // Reads come from the entity graph (hydrated by loadAgentsIntoGraph and
  // kept fresh by SSE realtime). Mutations continue through the legacy hook
  // until the next change in this phase migrates them too.
  const agentsView = useAgents();
  const agents = agentsView.items as unknown as UarAgent[];
  const loading = agents.length === 0;
  const [error, setError] = useState<string | null>(null);
  const load = () => {
    setError(null);
    return loadAgentsIntoGraph().catch((e) => setError((e as Error).message));
  };
  const [selected, setSelected] = useState<UarAgent | null>(null);

  // Populate the entity graph on mount.
  useEffect(() => {
    void loadAgentsIntoGraph();
  }, []);

  // Editor state
  const [editorOpen, setEditorOpen] = useState(false);
  const [editorAgent, setEditorAgent] = useState<UarAgent | null>(null);

  // AI builder state
  const [showAiBuilder, setShowAiBuilder] = useState(false);

  // Delete confirmation state
  const [deleteTarget, setDeleteTarget] = useState<UarAgent | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const openCreate = () => {
    setEditorAgent(null);
    setEditorOpen(true);
  };

  const openEdit = (agent: UarAgent) => {
    setEditorAgent(agent);
    setEditorOpen(true);
  };

  const handleEditorSave = () => {
    void load();
  };

  const handleAiGenerated = (agentData: Record<string, unknown>) => {
    setShowAiBuilder(false);
    setEditorAgent(agentData as unknown as UarAgent);
    setEditorOpen(true);
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeleting(true);
    setDeleteError(null);
    try {
      await optimisticRemove("Agent", deleteTarget.id, async () => {
        const res = await deleteAgent(deleteTarget.id);
        if (!res.ok) {
          const text = await res.text();
          throw new Error(text || `${res.status}`);
        }
      });
      if (selected?.id === deleteTarget.id) setSelected(null);
      setDeleteTarget(null);
    } catch (e) {
      setDeleteError((e as Error).message);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
      {/* Agent list */}
      <div className={cn("flex shrink-0 flex-col border-b border-border bg-background md:w-72 md:border-b-0 md:border-r", selected ? "hidden md:flex" : "flex flex-1 md:flex-initial")}>
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <div>
            <p className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">Agents</p>
            <p className="font-mono text-xs text-muted-foreground">{agents.length} configured</p>
          </div>
          <div className="flex items-center gap-1">
            <Button variant="ghost" size="sm" className="h-6 gap-1 px-1.5" onClick={() => setShowAiBuilder(true)} aria-label="Create with AI">
              <Sparkles size={12} />
              <span className="hidden sm:inline font-mono text-xs">AI</span>
            </Button>
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={openCreate} aria-label="New agent">
              <Plus size={12} />
            </Button>
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => void load()} aria-label="Refresh">
              <RefreshCw size={12} className={cn(loading && "animate-spin")} />
            </Button>
          </div>
        </div>
        <ScrollArea className="flex-1">
          <div className="py-2">
            {loading && agents.length === 0 && <AdminSidebarSkeleton rows={4} />}
            {error && <div className="px-3 py-2"><AdminError error={error} /></div>}
            {!loading && agents.length === 0 && !error && (
              <AdminEmptyInline>No agents configured. Click + to create one.</AdminEmptyInline>
            )}
            {agents.map((a) => (
              <Button
                key={a.id}
                variant="ghost"
                onClick={() => setSelected(a)}
                className={cn(
                  "h-auto w-full cursor-pointer justify-start rounded-none px-4 py-2.5 text-left",
                  selected?.id === a.id ? "bg-accent hover:bg-accent" : "hover:bg-muted/50",
                )}
              >
                <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary/15">
                  <Bot size={14} className="text-primary" />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate font-display text-sm font-semibold text-foreground">
                    {a.metadata?.title ?? a.id}
                  </p>
                  <p className="font-mono text-xs text-muted-foreground">{a.kind ?? "agent"}</p>
                </div>
                {agentLacksModel(a) && (
                  <AlertTriangle
                    size={13}
                    className="shrink-0 text-amber-500"
                    aria-label="No model configured"
                  />
                )}
              </Button>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* Detail */}
      <div className={cn("flex flex-1 flex-col overflow-hidden", !selected && "hidden md:flex")}>
        {selected ? (
          <ScrollArea className="flex-1">
            <div className="p-4 sm:p-6">
              <div className="flex items-center gap-2">
                <Button variant="ghost" size="icon" className="h-7 w-7 md:hidden" onClick={() => setSelected(null)} aria-label="Back to agents">
                  <ArrowLeft size={14} />
                </Button>
                <h2 className="flex-1 font-display text-lg font-semibold text-foreground">
                  {selected.metadata?.title ?? selected.id}
                </h2>
                <Button variant="outline" size="sm" className="gap-1.5" onClick={() => openEdit(selected)}>
                  <Pencil size={12} />
                  Edit
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="gap-1.5 text-destructive hover:bg-destructive/10 hover:text-destructive"
                  onClick={() => setDeleteTarget(selected)}
                >
                  <Trash2 size={12} />
                  Delete
                </Button>
              </div>
              {selected.metadata?.description && (
                <p className="mt-1 font-body text-sm text-muted-foreground">{selected.metadata.description}</p>
              )}

              <div className="mt-4">
                <p className="mb-1 font-mono text-xs uppercase tracking-widest text-muted-foreground">ID</p>
                <p className="font-mono text-sm text-foreground">{selected.id}</p>
              </div>

              {selected.kind && (
                <div className="mt-4">
                  <p className="mb-1 font-mono text-xs uppercase tracking-widest text-muted-foreground">Kind</p>
                  <p className="font-mono text-sm text-foreground">{selected.kind}</p>
                </div>
              )}

              {selected.skills && selected.skills.length > 0 && (
                <div className="mt-4">
                  <p className="mb-2 font-mono text-xs uppercase tracking-widest text-muted-foreground">
                    Skills ({selected.skills.length})
                  </p>
                  <div className="flex flex-col gap-1.5">
                    {selected.skills.map((sk) => (
                      <Card key={sk.skill_id} className="rounded-md border-border shadow-none">
                        <CardContent className="px-3 py-2">
                          <p className="font-mono text-xs text-foreground">{sk.title}</p>
                          {sk.description && (
                            <p className="font-body text-xs text-muted-foreground">{sk.description}</p>
                          )}
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                </div>
              )}

              {/* Memory section */}
              <AgentMemorySection agent={selected} />
            </div>
          </ScrollArea>
        ) : (
          <div className="flex flex-1 items-center justify-center p-6">
            <div className="max-w-sm text-center">
              <div className="mx-auto mb-4 flex size-12 items-center justify-center rounded-xl bg-primary/10">
                <Bot size={20} className="text-primary" />
              </div>
              <p className="font-display text-sm font-semibold text-foreground">Select an agent</p>
              <p className="mt-2 font-body text-xs text-muted-foreground">
                Choose an agent from the list to view its configuration, skills, and memory settings.
              </p>
            </div>
          </div>
        )}
      </div>

      {/* AI Builder Dialog */}
      <AgentAiBuilder
        open={showAiBuilder}
        onOpenChange={setShowAiBuilder}
        onGenerated={handleAiGenerated}
      />

      {/* Agent Editor Sheet */}
      <AgentEditor
        agent={editorAgent}
        open={editorOpen}
        onOpenChange={setEditorOpen}
        onSave={handleEditorSave}
      />

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => { if (!open) { setDeleteTarget(null); setDeleteError(null); } }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="font-display">Delete Agent</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete <strong>{deleteTarget?.metadata?.title ?? deleteTarget?.id}</strong>? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          {deleteError && (
            <p className="rounded-md bg-destructive/10 px-3 py-2 font-mono text-xs text-destructive">{deleteError}</p>
          )}
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={(e) => { e.preventDefault(); void handleDelete(); }}
              disabled={deleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90 gap-1.5"
            >
              {deleting && <Loader2 size={14} className="animate-spin" />}
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};
