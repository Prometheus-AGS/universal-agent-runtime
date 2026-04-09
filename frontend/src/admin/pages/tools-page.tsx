import { type FC, useCallback, useEffect, useState } from "react";
import { RefreshCw, Wrench } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminEmptyState, AdminError, AdminListSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import type { DiscoveryResponse, UarTool } from "@/types";

interface ToolWithNs extends UarTool { _ns: string; _key: string }

export const ToolsPage: FC = () => {
  const [tools, setTools] = useState<ToolWithNs[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/tools");
      if (!res.ok) throw new Error(`${res.status}`);
      const data = await res.json() as DiscoveryResponse & { data?: DiscoveryResponse };
      const d = data.data ?? data;
      const all: ToolWithNs[] = [
        ...(d.tools ?? []).map((t) => addNs(t, false)),
        ...(d.built_in_tools ?? []).map((t) => addNs(t, true)),
      ];
      setTools(all);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  function addNs(t: UarTool, builtin: boolean): ToolWithNs {
    const fullName = t.namespaced_name ?? t.name;
    const parts = fullName.split("::");
    return { ...t, _ns: parts.length > 1 ? parts[0] : (builtin ? "built-in" : "global"), _key: fullName };
  }

  const groups = tools
    .filter((t) => search === "" || t._key.toLowerCase().includes(search.toLowerCase()) || (t.description ?? "").toLowerCase().includes(search.toLowerCase()))
    .reduce<Record<string, ToolWithNs[]>>((acc, t) => {
      if (!acc[t._ns]) acc[t._ns] = [];
      acc[t._ns].push(t);
      return acc;
    }, {});

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">Tools</h2>
          <p className="font-mono text-xs text-muted-foreground">{tools.length} discovered</p>
        </div>
        <Button variant="outline" size="sm" onClick={() => void load()} className="gap-1.5">
          <RefreshCw size={13} className={cn(loading && "animate-spin")} />Refresh
        </Button>
      </div>

      <div className="border-b border-border px-6 py-2">
        <Input placeholder="Search tools…" value={search} onChange={(e) => setSearch(e.target.value)} className="h-8 text-xs" />
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {loading && tools.length === 0 && <AdminListSkeleton rows={5} />}
        <AdminError error={error} />
        {!loading && Object.keys(groups).length === 0 && (
          <AdminEmptyState
            icon={Wrench}
            title="No tools discovered"
            description="Make sure tool servers are configured in mcp.json and running. Tools will appear here once connected."
          />
        )}
        {Object.entries(groups).map(([ns, nsTools]) => (
          <div key={ns} className="mb-6">
            <div className="mb-2 flex items-center gap-2">
              <span className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">{ns}</span>
              <div className="h-px flex-1 bg-border" />
              <span className="font-mono text-xs text-muted-foreground/60">{nsTools.length}</span>
            </div>
            <div className="flex flex-col gap-1">
              {nsTools.map((t) => (
                <div key={t._key} className="flex items-start gap-3 rounded-md border border-border/50 bg-card px-3 py-2.5">
                  <Wrench size={13} className="mt-0.5 shrink-0 text-primary" />
                  <div className="min-w-0 flex-1">
                    <p className="font-mono text-xs font-medium text-foreground">{t._key}</p>
                    {t.description && <p className="font-body text-xs text-muted-foreground line-clamp-2">{t.description}</p>}
                  </div>
                  {t.source && <span className="shrink-0 rounded border border-border px-1 font-mono text-[9px] text-muted-foreground">{t.source}</span>}
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
