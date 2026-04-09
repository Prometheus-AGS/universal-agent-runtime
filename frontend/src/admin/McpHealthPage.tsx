import { type FC, useCallback, useEffect, useRef, useState } from "react";
import { RefreshCw, Server, Wrench } from "lucide-react";
import { Button } from "@/components/ui/button";
import { AdminEmptyState, AdminError, AdminListSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";

interface McpServerHealth {
  name: string;
  transport: string;
  status: "connected" | "disconnected" | "error";
  tool_count: number;
  error?: string;
}

const AUTO_REFRESH_MS = 30_000;

export const McpHealthPage: FC = () => {
  const [servers, setServers] = useState<McpServerHealth[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/uar/mcp/health");
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = (await res.json()) as
        | { servers?: McpServerHealth[] }
        | McpServerHealth[];
      const list = Array.isArray(data) ? data : data.servers ?? [];
      setServers(list);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Initial load + auto-refresh
  useEffect(() => {
    void load();
    intervalRef.current = setInterval(() => {
      void load();
    }, AUTO_REFRESH_MS);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [load]);

  const statusDot = (status: McpServerHealth["status"]) => {
    const color =
      status === "connected"
        ? "bg-success"
        : status === "error"
          ? "bg-destructive"
          : "bg-muted-foreground";
    return (
      <span
        className={cn("inline-block h-2 w-2 shrink-0 rounded-full", color)}
        aria-label={status}
      />
    );
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">
            Tool Server Health
          </h2>
          <p className="font-mono text-xs text-muted-foreground">
            {servers.length} servers &middot; auto-refresh 30s
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void load()}
          className="gap-1.5"
        >
          <RefreshCw size={13} className={cn(loading && "animate-spin")} />
          Refresh
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {loading && servers.length === 0 && <AdminListSkeleton rows={3} />}
        <AdminError error={error} />
        {!loading && servers.length === 0 && !error && (
          <AdminEmptyState
            icon={Server}
            title="No tool servers configured"
            description="Add tool servers to mcp.json to enable agent capabilities like web search, code execution, and more."
          />
        )}

        {servers.length > 0 && (
          <div className="flex flex-col gap-2">
            {servers.map((server) => (
              <div
                key={server.name}
                className="flex items-center gap-4 rounded-lg border border-border/50 bg-card px-4 py-3"
              >
                <Server size={15} className="shrink-0 text-primary" />
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    {statusDot(server.status)}
                    <p className="font-mono text-xs font-medium text-foreground">
                      {server.name}
                    </p>
                  </div>
                  {server.error && (
                    <p className="mt-0.5 font-mono text-xs text-destructive">
                      {server.error}
                    </p>
                  )}
                </div>
                <span className="shrink-0 rounded border border-border px-1.5 py-0.5 font-mono text-[9px] uppercase text-muted-foreground">
                  {server.transport}
                </span>
                <div className="flex shrink-0 items-center gap-1 font-mono text-xs text-muted-foreground">
                  <Wrench size={11} />
                  {server.tool_count}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
