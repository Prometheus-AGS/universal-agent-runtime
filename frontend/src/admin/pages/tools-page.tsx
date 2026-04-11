import { type FC, useState } from "react";
import { RefreshCw, Wrench } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminEmptyState, AdminError, AdminListSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import { useToolsDiscovery } from "@/hooks/use-tools-discovery";
import type { ToolWithNs } from "@/stores/tools-discovery-store";

export const ToolsPage: FC = () => {
  const { tools, loading, error, load } = useToolsDiscovery();
  const [search, setSearch] = useState("");

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
