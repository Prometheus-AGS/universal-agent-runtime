import { type FC, useCallback, useEffect, useState } from "react";
import { Code2, Loader2, Plus, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { UarCompilerSession } from "@/types";

export const CompilerPage: FC = () => {
  const [sessions, setSessions] = useState<UarCompilerSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/compiler/sessions");
      if (!res.ok) throw new Error(`${res.status}`);
      const data = await res.json() as { sessions?: UarCompilerSession[]; data?: { sessions?: UarCompilerSession[] } } | UarCompilerSession[];
      const list = Array.isArray(data) ? data : (data.data?.sessions ?? (data as { sessions?: UarCompilerSession[] }).sessions ?? []);
      setSessions(list);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const handleCreate = async () => {
    setCreating(true);
    try {
      const res = await fetch("/api/compiler/sessions", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({}) });
      if (!res.ok) throw new Error(`${res.status}`);
      await load();
    } catch (e) { setError((e as Error).message); } finally { setCreating(false); }
  };

  const statusColor = (status?: string) => {
    switch (status) {
      case "complete": return "text-success";
      case "running": case "compiling": return "text-warning";
      case "failed": return "text-destructive";
      default: return "text-muted-foreground";
    }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">Compiler Sessions</h2>
          <p className="font-mono text-[11px] text-muted-foreground">WASM skill compilation</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => void load()} className="gap-1.5"><RefreshCw size={13} className={cn(loading && "animate-spin")} />Refresh</Button>
          <Button size="sm" onClick={() => void handleCreate()} disabled={creating} className="gap-1.5">
            {creating ? <Loader2 size={13} className="animate-spin" /> : <Plus size={13} />}New session
          </Button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto p-6">
        {loading && <div className="flex items-center gap-2"><Loader2 size={16} className="animate-spin text-muted-foreground" /></div>}
        {error && <p className="font-mono text-[11px] text-destructive">Error: {error}</p>}
        {!loading && sessions.length === 0 && (
          <div className="flex flex-col items-center gap-4 py-16 text-center">
            <div className="flex size-16 items-center justify-center rounded-2xl border border-border bg-muted/30"><Code2 size={28} className="text-muted-foreground" /></div>
            <p className="font-mono text-[11px] text-muted-foreground">No compiler sessions yet</p>
            <Button size="sm" onClick={() => void handleCreate()} className="gap-1.5"><Plus size={13} />Create session</Button>
          </div>
        )}
        <div className="flex flex-col gap-2">
          {sessions.map((s) => (
            <div key={s.id} className="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
              <div className="flex size-8 items-center justify-center rounded-md bg-muted"><Code2 size={14} className="text-muted-foreground" /></div>
              <div className="min-w-0 flex-1">
                <p className="font-mono text-[12px] text-foreground">{s.id}</p>
                {s.created_at && <p className="font-mono text-[10px] text-muted-foreground">{new Date(s.created_at).toLocaleString()}</p>}
              </div>
              {s.skill_ids && s.skill_ids.length > 0 && <span className="font-mono text-[10px] text-muted-foreground">{s.skill_ids.length} skills</span>}
              {s.status && <span className={cn("font-mono text-[11px] font-medium capitalize", statusColor(s.status))}>{s.status}</span>}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
