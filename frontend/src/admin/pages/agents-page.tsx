import { type FC, useCallback, useEffect, useState } from "react";
import { Bot, Brain, Loader2, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { AgentsResponse, UarAgent } from "@/types";

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

function TriToggle({ value, onChange }: { value: boolean | null; onChange: (v: boolean | null) => void }) {
  const labels: { v: boolean | null; label: string }[] = [
    { v: null, label: "Inherit" },
    { v: true, label: "On" },
    { v: false, label: "Off" },
  ];
  return (
    <div className="inline-flex rounded-lg border border-border overflow-hidden text-[11px] font-mono">
      {labels.map(({ v, label }) => (
        <button
          key={label}
          type="button"
          onClick={() => onChange(v)}
          className={cn(
            "px-3 py-1 transition-colors",
            value === v
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-muted/50"
          )}
        >
          {label}
        </button>
      ))}
    </div>
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

      const r = await fetch(`/api/agents/${agent.id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!r.ok) throw new Error(`${r.status}`);
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
        <p className="font-mono text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
          Memory (per-agent override)
        </p>
      </div>

      <div className="space-y-3 rounded-lg border border-border bg-card p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="font-mono text-[12px] font-medium text-foreground">Memory Enabled</p>
            <p className="font-mono text-[10px] text-muted-foreground mt-0.5">Override global memory on/off for this agent.</p>
          </div>
          <TriToggle value={state.memory_enabled} onChange={(v) => setState((s) => ({ ...s, memory_enabled: v }))} />
        </div>
        <div className="flex items-center justify-between">
          <div>
            <p className="font-mono text-[12px] font-medium text-foreground">Auto-Capture</p>
            <p className="font-mono text-[10px] text-muted-foreground mt-0.5">Extract memories after each turn.</p>
          </div>
          <TriToggle value={state.auto_capture} onChange={(v) => setState((s) => ({ ...s, auto_capture: v }))} />
        </div>
        <div className="flex items-center justify-between">
          <div>
            <p className="font-mono text-[12px] font-medium text-foreground">Context Injection</p>
            <p className="font-mono text-[10px] text-muted-foreground mt-0.5">Inject memories as system prompt prefix.</p>
          </div>
          <TriToggle value={state.inject_context} onChange={(v) => setState((s) => ({ ...s, inject_context: v }))} />
        </div>
        <div className="flex items-center justify-between">
          <div>
            <p className="font-mono text-[12px] font-medium text-foreground">Default Scope</p>
            <p className="font-mono text-[10px] text-muted-foreground mt-0.5">Scope for memories saved by this agent.</p>
          </div>
          <select
            value={state.memory_scope}
            onChange={(e) => setState((s) => ({ ...s, memory_scope: e.target.value }))}
            className="h-8 rounded-md border border-input bg-background px-3 font-mono text-[12px] text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          >
            {SCOPE_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>{o.label}</option>
            ))}
          </select>
        </div>

        {error && (
          <p className="font-mono text-[11px] text-destructive">{error}</p>
        )}

        <div className="flex items-center gap-2 pt-1">
          <Button size="sm" onClick={() => void save()} disabled={saving} className="gap-1.5">
            {saving ? <Loader2 size={12} className="animate-spin" /> : null}
            Save Memory Settings
          </Button>
          {saved && (
            <span className="font-mono text-[11px] text-green-400">Saved ✓</span>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Main Agents Page ───────────────────────────────────────────────────────

export const AgentsPage: FC = () => {
  const [agents, setAgents] = useState<UarAgent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<UarAgent | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/agents");
      if (!res.ok) throw new Error(`${res.status}`);
      const data = await res.json() as AgentsResponse & { data?: AgentsResponse };
      const r = data.data?.runtime_agents ?? data.runtime_agents ?? [];
      const f = data.data?.federated_agents ?? data.federated_agents ?? [];
      setAgents([...r.map((a) => ({ ...a, _type: "runtime" })), ...f.map((a) => ({ ...a, _type: "federated" }))]);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Agent list */}
      <div className="flex w-64 shrink-0 flex-col border-r border-border bg-background">
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <p className="font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground">Agents</p>
          <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => void load()} aria-label="Refresh">
            <RefreshCw size={12} className={cn(loading && "animate-spin")} />
          </Button>
        </div>
        <div className="flex-1 overflow-y-auto py-2">
          {loading && <div className="flex justify-center py-8"><Loader2 size={16} className="animate-spin text-muted-foreground" /></div>}
          {error && <p className="px-4 py-4 font-mono text-[11px] text-destructive">Error: {error}</p>}
          {!loading && agents.length === 0 && <p className="px-4 py-4 font-mono text-[11px] text-muted-foreground">No agents configured</p>}
          {agents.map((a) => (
            <button key={a.id} onClick={() => setSelected(a)} className={cn("flex w-full items-center gap-3 px-4 py-2.5 text-left transition-colors", selected?.id === a.id ? "bg-accent" : "hover:bg-muted/50")}>
              <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary/15"><Bot size={14} className="text-primary" /></div>
              <div className="min-w-0 flex-1">
                <p className="truncate font-display text-[13px] font-semibold text-foreground">{a.metadata?.title ?? a.id}</p>
                <p className="font-mono text-[10px] text-muted-foreground">{a.kind ?? "agent"}</p>
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Detail */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {selected ? (
          <div className="flex-1 overflow-y-auto p-6">
            <h2 className="font-display text-lg font-semibold text-foreground">{selected.metadata?.title ?? selected.id}</h2>
            {selected.metadata?.description && <p className="mt-1 font-body text-sm text-muted-foreground">{selected.metadata.description}</p>}
            <div className="mt-4">
              <p className="mb-1 font-mono text-[10px] uppercase tracking-widest text-muted-foreground">ID</p>
              <p className="font-mono text-sm text-foreground">{selected.id}</p>
            </div>
            {selected.kind && (
              <div className="mt-4">
                <p className="mb-1 font-mono text-[10px] uppercase tracking-widest text-muted-foreground">Kind</p>
                <p className="font-mono text-sm text-foreground">{selected.kind}</p>
              </div>
            )}
            {selected.skills && selected.skills.length > 0 && (
              <div className="mt-4">
                <p className="mb-2 font-mono text-[10px] uppercase tracking-widest text-muted-foreground">Skills ({selected.skills.length})</p>
                <div className="flex flex-col gap-1">
                  {selected.skills.map((sk) => (
                    <div key={sk.skill_id} className="rounded-md border border-border bg-card px-3 py-2">
                      <p className="font-mono text-[12px] text-foreground">{sk.title}</p>
                      {sk.description && <p className="font-body text-xs text-muted-foreground">{sk.description}</p>}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Memory section */}
            <AgentMemorySection agent={selected} />
          </div>
        ) : (
          <div className="flex flex-1 items-center justify-center">
            <p className="font-mono text-[11px] text-muted-foreground">← Select an agent</p>
          </div>
        )}
      </div>
    </div>
  );
};
