import { type FC, useCallback, useEffect, useState } from "react";
import { Circle, CircleOff, Loader2, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { ModelsResponse, UarModel, UarProvider } from "@/types";

export const ModelsPage: FC = () => {
  const [models, setModels] = useState<UarModel[]>([]);
  const [providers, setProviders] = useState<UarProvider[]>([]);
  const [activeModel, setActiveModel] = useState<string | undefined>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedProvider, setSelectedProvider] = useState<string>("all");

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [modelsRes, providersRes] = await Promise.all([
        fetch("/api/models"),
        fetch("/api/providers"),
      ]);
      const modelsData = await modelsRes.json() as ModelsResponse & { data?: ModelsResponse };
      const providersData = await providersRes.json() as { providers?: UarProvider[]; data?: { providers?: UarProvider[] } };
      const p = providersData.data?.providers ?? providersData.providers ?? [];
      setProviders(p);
      const m = modelsData.data?.models ?? modelsData.models ?? [];
      setModels(m);
      setActiveModel(modelsData.data?.active_model ?? modelsData.active_model);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const handleSetActive = async (id: string) => {
    try {
      await fetch(`/api/models/${id}/activate`, { method: "POST" });
      setActiveModel(id);
    } catch { /**/ }
  };

  const filtered = selectedProvider === "all" ? models : models.filter((m) => m.id.startsWith(selectedProvider));

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">Models</h2>
          <p className="font-mono text-[11px] text-muted-foreground">{models.length} models configured</p>
        </div>
        <Button variant="outline" size="sm" onClick={() => void load()} className="gap-1.5">
          <RefreshCw size={13} className={cn(loading && "animate-spin")} />Refresh
        </Button>
      </div>

      {/* Provider filter */}
      {providers.length > 0 && (
        <div className="flex gap-2 overflow-x-auto border-b border-border bg-background px-6 py-2">
          <button onClick={() => setSelectedProvider("all")} className={cn("shrink-0 rounded-full px-3 py-1 font-mono text-[11px] transition-colors", selectedProvider === "all" ? "bg-primary/20 text-primary" : "text-muted-foreground hover:text-foreground")}>All</button>
          {providers.map((p) => (
            <button key={p.id} onClick={() => setSelectedProvider(p.id)} className={cn("shrink-0 rounded-full px-3 py-1 font-mono text-[11px] transition-colors", selectedProvider === p.id ? "bg-primary/20 text-primary" : "text-muted-foreground hover:text-foreground")}>
              {p.display_name ?? p.id}
            </button>
          ))}
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-6">
        {loading && <div className="flex items-center gap-2"><Loader2 size={16} className="animate-spin text-muted-foreground" /><span className="font-mono text-[11px] text-muted-foreground">Loading…</span></div>}
        {error && <p className="font-mono text-[11px] text-destructive">Error: {error}</p>}
        {!loading && filtered.length === 0 && <p className="font-mono text-[11px] text-muted-foreground">No models found</p>}
        <div className="flex flex-col gap-2">
          {filtered.map((m) => (
            <div key={m.id} className={cn("flex items-center gap-3 rounded-lg border px-4 py-3", m.id === activeModel ? "border-primary/40 bg-primary/5" : "border-border bg-card")}>
              <div className="shrink-0">
                {m.enabled !== false ? <Circle size={8} className="fill-success text-success" /> : <CircleOff size={8} className="text-muted-foreground" />}
              </div>
              <div className="min-w-0 flex-1">
                <p className="font-mono text-[13px] font-medium text-foreground">{m.display_name ?? m.id}</p>
                <p className="font-mono text-[10px] text-muted-foreground">{m.id}</p>
              </div>
              <div className="flex shrink-0 items-center gap-2">
                {m.context_window && <span className="font-mono text-[10px] text-muted-foreground">{(m.context_window / 1000).toFixed(0)}k ctx</span>}
                {m.supports_tools && <span className="rounded border border-border px-1 font-mono text-[9px] text-muted-foreground">tools</span>}
                {m.supports_vision && <span className="rounded border border-border px-1 font-mono text-[9px] text-muted-foreground">vision</span>}
                {m.id === activeModel && <span className="rounded-full bg-primary/15 px-2 py-0.5 font-mono text-[9px] text-primary">active</span>}
                {m.id !== activeModel && (
                  <Button variant="outline" size="sm" className="h-6 px-2 text-[11px]" onClick={() => void handleSetActive(m.id)}>Set active</Button>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
