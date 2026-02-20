import { type FC, useCallback, useEffect, useState } from "react";
import { Bot, Loader2, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { AgentsResponse, UarAgent } from "@/types";

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
