import { type FC, useEffect } from "react";
import { Code2, Plus, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { LoadingCursor } from "@/shared/ui/configuration/loading-cursor";
import { EmptyFrame } from "@/shared/ui/configuration/empty-frame";
import { ErrorBar } from "@/shared/ui/configuration/error-bar";
import { cn } from "@/lib/utils";
import { useCompilerSessions } from "../model/use-compiler-sessions";
import { useCompiler } from "../model/use-compiler";
import type { UarCompilerSession } from "@/types";

export const CompilerPage: FC = () => {
  const view = useCompilerSessions();
  const sessions = view.items as UarCompilerSession[];
  const compiler = useCompiler();
  const { load } = compiler;

  useEffect(() => {
    void load().catch(() => undefined);
  }, [load]);

  const loading = compiler.loading && sessions.length === 0;

  const statusColor = (status?: string) => {
    switch (status) {
      case "complete": return "text-[var(--color-phosphor)]";
      case "running":
      case "compiling": return "text-[var(--color-amber)]";
      case "failed": return "text-[var(--color-signal-red)]";
      default: return "text-[var(--color-terminal-fg-dim)]";
    }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden font-mono text-[13px] text-[var(--color-terminal-fg)]">
      <div className="flex items-center justify-between border-b border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] px-6 py-4">
        <div>
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-[20px] font-medium tracking-tight">compiler sessions</h2>
            <Badge variant="outline">Experimental</Badge>
          </div>
          <p className="text-xs text-[var(--color-terminal-fg-dim)]">
            Preview the skill compilation workflow; packaged output is not GA-certified yet
            {compiler.loading && <LoadingCursor className="ml-2" />}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => void load().catch(() => undefined)}
            className="gap-1.5 border border-[var(--color-terminal-line-strong)] bg-transparent text-[var(--color-terminal-fg)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_8%,transparent)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)]"
          >
            <RefreshCw size={13} className={cn(compiler.loading && "animate-spin")} />refresh
          </Button>
          <Button
            size="sm"
            onClick={() => void compiler.createSession().catch(() => undefined)}
            disabled={compiler.creating}
            className="gap-1.5 border border-[var(--color-phosphor)] bg-[color-mix(in_srgb,var(--color-phosphor)_12%,transparent)] text-[var(--color-phosphor)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_18%,transparent)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)]"
          >
            {compiler.creating ? <LoadingCursor /> : <Plus size={13} />}new session
          </Button>
        </div>
      </div>

      {compiler.error && (
        <ErrorBar code="COMPILER" message={compiler.error} />
      )}

      <div className="flex-1 overflow-y-auto p-6">
        {loading && <LoadingCursor label="loading compiler sessions" />}
        {!loading && sessions.length === 0 && !compiler.error && (
          <EmptyFrame
            title="no compiler sessions"
            hint="compile skills into portable wasm modules that run anywhere"
            action={
              <Button
                onClick={() => void compiler.createSession().catch(() => undefined)}
                disabled={compiler.creating}
                className="gap-1.5 border border-[var(--color-phosphor)] bg-[color-mix(in_srgb,var(--color-phosphor)_12%,transparent)] text-[var(--color-phosphor)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_18%,transparent)]"
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
              className="flex items-center gap-3 border border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] px-4 py-3 transition-colors duration-[160ms] hover:border-[color-mix(in_srgb,var(--color-phosphor)_40%,transparent)]"
            >
              <div className="flex size-8 items-center justify-center border border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-bg)]">
                <Code2 size={14} className="text-[var(--color-terminal-fg-dim)]" />
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
