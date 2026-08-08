import { type FC, useEffect, useState } from "react";
import { RefreshCw, Wrench } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminEmptyState, AdminError, AdminSidebarSkeleton } from "@/shared/ui/configuration/configuration-states";
import { ToolDetailPanel } from "./tool-detail-panel";
import { useGraphEntities } from "@/entities/hooks/use-graph-entities";
import { cn } from "@/lib/utils";
import type { ToolGraphRow } from "../model/tool-graph";
import { useToolsAdmin } from "../model/use-tools-admin";

// Page consumes the loose graph row shape with `_ns`/`_key`/`_builtin`.
type ToolWithNs = ToolGraphRow;

export const ToolsPage: FC = () => {
  const tools = useGraphEntities<ToolWithNs>("Tool");
  const admin = useToolsAdmin();
  const { load } = admin;
  const [search, setSearch] = useState("");

  // Tool registry is static after server startup — one-time fetch on mount.
  useEffect(() => {
    void load().catch(() => undefined);
  }, [load]);

  const loading = admin.loading && tools.length === 0;
  const [selectedTool, setSelectedTool] = useState<ToolWithNs | null>(null);

  const groups = tools
    .filter((t) => search === "" || t._key.toLowerCase().includes(search.toLowerCase()) || (t.description ?? "").toLowerCase().includes(search.toLowerCase()))
    .reduce<Record<string, ToolWithNs[]>>((acc, t) => {
      if (!acc[t._ns]) acc[t._ns] = [];
      acc[t._ns].push(t);
      return acc;
    }, {});

  return (
    <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
      {/* Tool list (left column) */}
      <div className={cn("flex shrink-0 flex-col border-b border-border bg-background md:w-80 md:border-b-0 md:border-r", selectedTool ? "hidden md:flex" : "flex flex-1 md:flex-initial")}>
        <div className="flex items-center justify-between border-b border-border bg-card px-4 py-3">
          <div>
            <p className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">Tools</p>
            <p className="font-mono text-xs text-muted-foreground">{tools.length} discovered</p>
          </div>
          <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => void load().catch(() => undefined)} aria-label="Refresh">
            <RefreshCw size={12} className={cn(loading && "animate-spin")} />
          </Button>
        </div>

        <div className="border-b border-border px-3 py-2">
          <Input placeholder="Search tools..." value={search} onChange={(e) => setSearch(e.target.value)} className="h-8 text-xs" />
        </div>

        <div className="flex-1 overflow-y-auto py-2">
          {loading && tools.length === 0 && <AdminSidebarSkeleton rows={5} />}
          <AdminError error={admin.error} />
          {!loading && Object.keys(groups).length === 0 && (
            <AdminEmptyState
              icon={Wrench}
              title="No tools discovered"
              description="Make sure tool servers are configured in mcp.json and running. Tools will appear here once connected."
            />
          )}
          {Object.entries(groups).map(([ns, nsTools]) => (
            <div key={ns} className="mb-4 px-3">
              <div className="mb-1.5 flex items-center gap-2">
                <span className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">{ns}</span>
                <div className="h-px flex-1 bg-border" />
                <span className="font-mono text-xs text-muted-foreground/60">{nsTools.length}</span>
              </div>
              <div className="flex flex-col gap-0.5">
                {nsTools.map((t) => (
                  <button
                    type="button"
                    key={t._key}
                    onClick={() => setSelectedTool(t)}
                    aria-label={`View tool ${t._key}`}
                    className={cn(
                      "flex w-full items-start gap-3 rounded-md border border-border/50 px-3 py-2.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-ring",
                      selectedTool?._key === t._key ? "bg-accent border-accent" : "bg-card hover:bg-muted/50",
                    )}
                  >
                    <Wrench size={13} className="mt-0.5 shrink-0 text-primary" />
                    <div className="min-w-0 flex-1">
                      <p className="font-mono text-xs font-medium text-foreground">{t._key}</p>
                      {t.description && <p className="font-body text-xs text-muted-foreground line-clamp-1">{t.description}</p>}
                    </div>
                    {t.source && <span className="shrink-0 rounded border border-border px-1 font-mono text-[9px] text-muted-foreground">{t.source}</span>}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Detail (right column) */}
      <div className={cn("flex flex-1 flex-col overflow-hidden", !selectedTool && "hidden md:flex")}>
        {selectedTool ? (
          <ToolDetailPanel
            key={selectedTool._key}
            tool={selectedTool}
            onBack={() => setSelectedTool(null)}
            onExecute={admin.execute}
            executing={admin.executing}
          />
        ) : (
          <div className="flex flex-1 items-center justify-center p-6">
            <div className="max-w-sm text-center">
              <div className="mx-auto mb-4 flex size-12 items-center justify-center rounded-xl bg-primary/10">
                <Wrench size={20} className="text-primary" />
              </div>
              <p className="font-display text-sm font-semibold text-foreground">Select a tool</p>
              <p className="mt-2 font-body text-xs text-muted-foreground">
                Choose a tool from the list to view its schema and test it with custom arguments.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
