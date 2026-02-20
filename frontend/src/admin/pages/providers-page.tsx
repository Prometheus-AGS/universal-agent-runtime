import { type FC, useCallback, useEffect, useState } from "react";
import { ChevronRight, Circle, CircleOff, Loader2, Plus, RefreshCw, Server } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import type { ProvidersResponse, UarModel, UarProvider } from "@/types";

async function fetchProviders(): Promise<ProvidersResponse> {
  const res = await fetch("/api/providers");
  if (!res.ok) throw new Error(`${res.status}`);
  const data = await res.json() as ProvidersResponse & { data?: ProvidersResponse };
  if (data.data) return data.data;
  return data;
}

function groupModelsByCategory(models: UarModel[]): Record<string, UarModel[]> {
  const groups: Record<string, UarModel[]> = {};
  for (const m of models) {
    const name = m.display_name ?? m.id;
    const category = inferCategory(name);
    if (!groups[category]) groups[category] = [];
    groups[category].push(m);
  }
  return groups;
}

function inferCategory(name: string): string {
  const lower = name.toLowerCase();
  if (lower.includes("embed")) return "Embedding";
  if (lower.includes("vision") || lower.includes("4o") || lower.includes("claude-3") || lower.includes("gemini")) return "Vision & Reasoning";
  if (lower.includes("nano") || lower.includes("mini") || lower.includes("haiku") || lower.includes("flash")) return "Compact";
  if (lower.includes("o1") || lower.includes("o3") || lower.includes("reason") || lower.includes("think")) return "Reasoning";
  if (lower.includes("code") || lower.includes("coder")) return "Coding";
  return "Chat";
}

export const ProvidersPage: FC = () => {
  const [providers, setProviders] = useState<UarProvider[]>([]);
  const [defaultId, setDefaultId] = useState<string | undefined>();
  const [selected, setSelected] = useState<UarProvider | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [form, setForm] = useState({ id: "", display_name: "", base_url: "", api_key: "", protocol: "openai" });
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchProviders();
      setProviders(data.providers ?? []);
      setDefaultId(data.default_id);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const handleAddProvider = async () => {
    setSaving(true);
    try {
      const res = await fetch("/api/providers", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form),
      });
      if (!res.ok) throw new Error(`${res.status}`);
      setShowAddDialog(false);
      setForm({ id: "", display_name: "", base_url: "", api_key: "", protocol: "openai" });
      await load();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSaving(false);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await fetch(`/api/providers/${id}/default`, { method: "POST" });
      setDefaultId(id);
    } catch { /**/ }
  };

  const handleDelete = async (id: string) => {
    if (!confirm(`Delete provider "${id}"?`)) return;
    try {
      await fetch(`/api/providers/${id}`, { method: "DELETE" });
      if (selected?.id === id) setSelected(null);
      await load();
    } catch { /**/ }
  };

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Provider list (left column) */}
      <div className="flex w-64 shrink-0 flex-col border-r border-border bg-background">
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <p className="font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground">Providers</p>
          <div className="flex gap-1">
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => void load()} aria-label="Refresh">
              <RefreshCw size={12} className={cn(loading && "animate-spin")} />
            </Button>
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => setShowAddDialog(true)} aria-label="Add provider">
              <Plus size={12} />
            </Button>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto py-2">
          {loading && (
            <div className="flex items-center justify-center py-8">
              <Loader2 size={20} className="animate-spin text-muted-foreground" />
            </div>
          )}
          {error && <p className="px-4 py-4 font-mono text-[11px] text-destructive">Error: {error}</p>}
          {!loading && providers.length === 0 && (
            <p className="px-4 py-4 font-mono text-[11px] text-muted-foreground">No providers configured</p>
          )}
          {providers.map((p) => (
            <Button
              key={p.id}
              variant="ghost"
              onClick={() => setSelected(p)}
              className={cn(
                "flex h-auto w-full items-center justify-start gap-3 px-4 py-2.5 text-left transition-colors",
                selected?.id === p.id ? "bg-accent" : "hover:bg-muted/50",
              )}
              aria-current={selected?.id === p.id ? "true" : undefined}
            >
              <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted">
                <Server size={14} className="text-muted-foreground" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate font-display text-[13px] font-semibold text-foreground">{p.display_name ?? p.id}</p>
                <p className="font-mono text-[10px] text-muted-foreground">{p.protocol ?? "openai"}</p>
              </div>
              {p.id === defaultId && (
                <span className="shrink-0 rounded-full bg-primary/15 px-1.5 py-0.5 font-mono text-[9px] text-primary">default</span>
              )}
              <ChevronRight size={13} className="shrink-0 text-muted-foreground/40" />
            </Button>
          ))}
        </div>
      </div>

      {/* Detail + models (right column) */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {selected ? (
          <ProviderDetail
            provider={selected}
            isDefault={selected.id === defaultId}
            onSetDefault={() => void handleSetDefault(selected.id)}
            onDelete={() => void handleDelete(selected.id)}
          />
        ) : (
          <div className="flex flex-1 items-center justify-center">
            <p className="font-mono text-[11px] text-muted-foreground">← Select a provider</p>
          </div>
        )}
      </div>

      {/* Add Provider Dialog */}
      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent>
          <DialogHeader><DialogTitle>Add Provider</DialogTitle></DialogHeader>
          <div className="flex flex-col gap-3">
            <div><Label htmlFor="provider-id" className="mb-1 block font-mono text-[11px] text-muted-foreground">ID</Label><Input id="provider-id" value={form.id} onChange={(e) => setForm({ ...form, id: e.target.value })} placeholder="openai" /></div>
            <div><Label htmlFor="provider-display-name" className="mb-1 block font-mono text-[11px] text-muted-foreground">Display Name</Label><Input id="provider-display-name" value={form.display_name} onChange={(e) => setForm({ ...form, display_name: e.target.value })} placeholder="OpenAI" /></div>
            <div><Label htmlFor="provider-base-url" className="mb-1 block font-mono text-[11px] text-muted-foreground">Base URL</Label><Input id="provider-base-url" value={form.base_url} onChange={(e) => setForm({ ...form, base_url: e.target.value })} placeholder="https://api.openai.com/v1" /></div>
            <div><Label htmlFor="provider-api-key" className="mb-1 block font-mono text-[11px] text-muted-foreground">API Key</Label><Input id="provider-api-key" type="password" value={form.api_key} onChange={(e) => setForm({ ...form, api_key: e.target.value })} placeholder="sk-..." /></div>
            <div>
              <Label htmlFor="provider-protocol" className="mb-1 block font-mono text-[11px] text-muted-foreground">Protocol</Label>
              <Select value={form.protocol} onValueChange={(value) => setForm({ ...form, protocol: value })}>
                <SelectTrigger id="provider-protocol">
                  <SelectValue placeholder="Select protocol" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="openai">openai</SelectItem>
                  <SelectItem value="anthropic">anthropic</SelectItem>
                  <SelectItem value="google">google</SelectItem>
                  <SelectItem value="ollama">ollama</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setShowAddDialog(false)}>Cancel</Button>
            <Button onClick={() => void handleAddProvider()} disabled={saving || !form.id || !form.base_url}>
              {saving && <Loader2 size={14} className="animate-spin" />}Add Provider
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};

interface ProviderDetailProps {
  provider: UarProvider;
  isDefault: boolean;
  onSetDefault: () => void;
  onDelete: () => void;
}

function ProviderDetail({ provider, isDefault, onSetDefault, onDelete }: ProviderDetailProps) {
  const [models, setModels] = useState<UarModel[]>([]);
  const [loadingModels, setLoadingModels] = useState(true);

  useEffect(() => {
    setLoadingModels(true);
    fetch(`/api/providers/${provider.id}/models`)
      .then((r) => r.json())
      .then((data: { models?: UarModel[] } | UarModel[]) => {
        setModels(Array.isArray(data) ? data : (data.models ?? []));
      })
      .catch(() => setModels([]))
      .finally(() => setLoadingModels(false));
  }, [provider.id]);

  const grouped = groupModelsByCategory(models);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Provider header */}
      <div className="border-b border-border bg-card px-6 py-4">
        <div className="flex items-start justify-between">
          <div>
            <h2 className="font-display text-lg font-semibold text-foreground">{provider.display_name ?? provider.id}</h2>
            <p className="mt-0.5 font-mono text-[11px] text-muted-foreground">{provider.base_url}</p>
            <div className="mt-2 flex items-center gap-2">
              <span className="rounded-full border border-border px-2 py-0.5 font-mono text-[10px] text-muted-foreground">{provider.protocol ?? "openai"}</span>
              {isDefault && <span className="rounded-full bg-primary/15 px-2 py-0.5 font-mono text-[10px] text-primary">default</span>}
            </div>
          </div>
          <div className="flex gap-2">
            {!isDefault && (
              <Button variant="outline" size="sm" onClick={onSetDefault} className="h-7 text-xs">Set as default</Button>
            )}
            <Button variant="outline" size="sm" onClick={onDelete} className="h-7 border-destructive/40 text-xs text-destructive hover:bg-destructive/10">Delete</Button>
          </div>
        </div>
      </div>

      {/* Models grouped by category */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        <div className="mb-4 flex items-center justify-between">
          <p className="font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground">Models ({models.length})</p>
        </div>

        {loadingModels ? (
          <div className="flex items-center gap-2 py-4">
            <Loader2 size={14} className="animate-spin text-muted-foreground" />
            <span className="font-mono text-[11px] text-muted-foreground">Loading models…</span>
          </div>
        ) : models.length === 0 ? (
          <p className="font-mono text-[11px] text-muted-foreground">No models available for this provider</p>
        ) : (
          <div className="flex flex-col gap-4">
            {Object.entries(grouped).map(([category, categoryModels]) => (
              <div key={category}>
                <div className="mb-2 flex items-center gap-2">
                  <span className="font-mono text-[10px] font-medium uppercase tracking-widest text-muted-foreground">{category}</span>
                  <Separator className="flex-1" />
                </div>
                <div className="flex flex-col gap-1">
                  {categoryModels.map((m) => (
                    <ModelRow key={m.id} model={m} />
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function ModelRow({ model }: { model: UarModel }) {
  return (
    <div className="flex items-center gap-3 rounded-md border border-border/50 bg-card px-3 py-2.5">
      <div className="shrink-0">
        {model.enabled !== false ? <Circle size={8} className="fill-success text-success" /> : <CircleOff size={8} className="text-muted-foreground" />}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate font-mono text-[12px] text-foreground">{model.display_name ?? model.id}</p>
        {model.context_window && <p className="font-mono text-[10px] text-muted-foreground">{(model.context_window / 1000).toFixed(0)}k ctx</p>}
      </div>
      <div className="flex shrink-0 gap-1">
        {model.supports_tools && <span className="rounded border border-border px-1 font-mono text-[9px] text-muted-foreground">tools</span>}
        {model.supports_vision && <span className="rounded border border-border px-1 font-mono text-[9px] text-muted-foreground">vision</span>}
      </div>
    </div>
  );
}
