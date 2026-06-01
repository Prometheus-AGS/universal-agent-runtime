import { type FC, useEffect, useState } from "react";
import { Code2, Plus, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { LoadingCursor } from "@/components/admin/loading-cursor";
import { EmptyFrame } from "@/components/admin/empty-frame";
import { ErrorBar } from "@/components/admin/error-bar";
import { cn } from "@/lib/utils";
import { useCompilerSessions } from "@/entities/hooks/use-compiler-sessions";
import { loadCompilerSessionsIntoGraph } from "@/entities/fetchers/compiler-sessions";
import { createCompilerSession } from "@/services/compiler-api";
import type { UarCompilerSession } from "@/types";

export const CompilerPage: FC = () => {
  const view = useCompilerSessions();
  const sessions = view.items as UarCompilerSession[];
  const [refreshing, setRefreshing] = useState(false);
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const load = async () => {
    setRefreshing(true);
    setError(null);
    try {
      await loadCompilerSessionsIntoGraph();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setRefreshing(false);
    }
  };

  const createNew = async () => {
    setCreating(true);
    setError(null);
    try {
      await createCompilerSession<unknown>();
      await loadCompilerSessionsIntoGraph();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setCreating(false);
    }
  };

  const loading = refreshing && sessions.length === 0;

  const statusColor = (status?: string) => {
    switch (status) {
      case "complete": return "text-[hsl(var(--phosphor))]";
      case "running":
      case "compiling": return "text-[hsl(var(--amber))]";
      case "failed": return "text-[hsl(var(--signal-red))]";
      default: return "text-[hsl(var(--terminal-fg-dim))]";
    }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden font-mono text-[13px] text-[hsl(var(--terminal-fg))]">
      <div className="flex items-center justify-between border-b border-[hsl(var(--terminal-line-strong))] bg-[hsl(var(--terminal-surface))] px-6 py-4">
        <div>
          <h2 className="text-[20px] font-medium tracking-tight">compiler sessions</h2>
          <p className="text-xs text-[hsl(var(--terminal-fg-dim))]">
            compile skills into portable wasm modules
            {refreshing && <LoadingCursor className="ml-2" />}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => void load()}
            className="gap-1.5 border border-[hsl(var(--terminal-line-strong))] bg-transparent text-[hsl(var(--terminal-fg))] hover:bg-[hsl(var(--phosphor)/0.08)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))]"
          >
            <RefreshCw size={13} className={cn(refreshing && "animate-spin")} />refresh
          </Button>
          <Button
            size="sm"
            onClick={() => void createNew()}
            disabled={creating}
            className="gap-1.5 border border-[hsl(var(--phosphor))] bg-[hsl(var(--phosphor)/0.12)] text-[hsl(var(--phosphor))] hover:bg-[hsl(var(--phosphor)/0.18)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))]"
          >
            {creating ? <LoadingCursor /> : <Plus size={13} />}new session
          </Button>
        </div>
      </div>

      {error && (
        <ErrorBar code="COMPILER" message={error} onDismiss={() => setError(null)} />
      )}

      <div className="flex-1 overflow-y-auto p-6">
        {loading && <LoadingCursor label="loading compiler sessions" />}
        {!loading && sessions.length === 0 && !error && (
          <EmptyFrame
            title="no compiler sessions"
            hint="compile skills into portable wasm modules that run anywhere"
            action={
              <Button
                onClick={() => void createNew()}
                disabled={creating}
                className="gap-1.5 border border-[hsl(var(--phosphor))] bg-[hsl(var(--phosphor)/0.12)] text-[hsl(var(--phosphor))] hover:bg-[hsl(var(--phosphor)/0.18)]"
                size="sm"
              >
                <Plus size={13} />create session
              </Button>
            }
          />
        )}
        <div className="flex flex-col gap-1.5">
          {sessions.map((s) => (
            <div
              key={s.id}
              className="flex items-center gap-3 border border-[hsl(var(--terminal-line-strong))] bg-[hsl(var(--terminal-surface))] px-4 py-3 transition-colors duration-[160ms] hover:border-[hsl(var(--phosphor)/0.4)]"
            >
              <div className="flex size-8 items-center justify-center border border-[hsl(var(--terminal-line-strong))] bg-[hsl(var(--terminal-bg))]">
                <Code2 size={14} className="text-[hsl(var(--terminal-fg-dim))]" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate text-xs">{s.id}</p>
                <p className={cn("text-xs", statusColor(s.status))}>{s.status ?? "unknown"}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
