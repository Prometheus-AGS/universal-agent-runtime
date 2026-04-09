import { type FC, useCallback, useEffect, useState } from "react";
import { ArrowLeft, CheckCircle2, ChevronRight, Circle, Loader2, Plus, RefreshCw, Server, XCircle } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { AdminEmptyInline, AdminError, AdminSidebarSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import type { CatalogProviderSummary, CatalogResponse, ProvidersResponse, UarProvider } from "@/types";

// ---------------------------------------------------------------------------
// API helpers
// ---------------------------------------------------------------------------

async function fetchCatalog(): Promise<CatalogResponse> {
  const res = await fetch("/api/catalog");
  if (!res.ok) throw new Error(`Catalog fetch failed: ${res.status}`);
  return res.json() as Promise<CatalogResponse>;
}

async function fetchConfiguredProviders(): Promise<ProvidersResponse> {
  const res = await fetch("/api/uar/providers");
  if (!res.ok) throw new Error(`Providers fetch failed: ${res.status}`);
  return res.json() as Promise<ProvidersResponse>;
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export const ProvidersPage: FC = () => {
  const [catalog, setCatalog] = useState<CatalogProviderSummary[]>([]);
  const [configured, setConfigured] = useState<UarProvider[]>([]);
  const [defaultId, setDefaultId] = useState<string | undefined>();
  const [selected, setSelected] = useState<CatalogProviderSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [addTarget, setAddTarget] = useState<CatalogProviderSummary | null>(null);
  const [form, setForm] = useState({ api_key: "", base_url: "" });
  const [saving, setSaving] = useState(false);
  const [filter, setFilter] = useState<"all" | "configured" | "unconfigured">("all");
  const [removeTarget, setRemoveTarget] = useState<string | null>(null);
  const [removing, setRemoving] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [catalogData, providersData] = await Promise.all([
        fetchCatalog(),
        fetchConfiguredProviders(),
      ]);
      setCatalog(catalogData.providers);
      setConfigured(providersData.providers ?? []);
      setDefaultId(providersData.default_id);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const handleConfigure = (provider: CatalogProviderSummary) => {
    setAddTarget(provider);
    setForm({ api_key: "", base_url: provider.base_url ?? "" });
    setShowAddDialog(true);
  };

  const handleSaveConfigure = async () => {
    if (!addTarget) return;
    setSaving(true);
    try {
      const body = {
        id: addTarget.id,
        display_name: addTarget.display_name,
        base_url: form.base_url || addTarget.base_url,
        api_key: form.api_key || undefined,
        protocol: "auto",
        enabled: true,
      };
      const res = await fetch("/api/uar/providers", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!res.ok && res.status !== 409) throw new Error(`${res.status}`);
      setShowAddDialog(false);
      await load();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSaving(false);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await fetch(`/api/uar/providers/${id}/default`, { method: "POST" });
      setDefaultId(id);
    } catch { /**/ }
  };

  const handleDelete = async () => {
    if (!removeTarget) return;
    setRemoving(true);
    try {
      await fetch(`/api/uar/providers/${removeTarget}`, { method: "DELETE" });
      if (selected?.id === removeTarget) setSelected(null);
      setRemoveTarget(null);
      await load();
    } catch { /**/ } finally { setRemoving(false); }
  };

  const visible = catalog.filter((p) => {
    if (filter === "configured") return p.configured;
    if (filter === "unconfigured") return !p.configured;
    return true;
  });

  return (
    <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
      {/* Provider list (left column) */}
      <div className={cn("flex shrink-0 flex-col border-b border-border bg-background md:w-72 md:border-b-0 md:border-r", selected ? "hidden md:flex" : "flex flex-1 md:flex-initial")}>
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <div>
            <p className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">Providers</p>
            <p className="font-mono text-xs text-muted-foreground">{catalog.length} available · {configured.length} configured</p>
          </div>
          <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => void load()} aria-label="Refresh">
            <RefreshCw size={12} className={cn(loading && "animate-spin")} />
          </Button>
        </div>

        {/* Filter tabs */}
        <div className="flex gap-1 border-b border-border px-3 py-2">
          {(["all", "configured", "unconfigured"] as const).map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={cn(
                "rounded px-2 py-0.5 font-mono text-xs transition-colors capitalize focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
                filter === f ? "bg-primary/15 text-primary" : "text-muted-foreground hover:text-foreground",
              )}
            >
              {f}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto py-2">
          {loading && catalog.length === 0 && <AdminSidebarSkeleton rows={6} />}
          {error && <AdminError error={error} />}
          {!loading && visible.length === 0 && (
            <AdminEmptyInline>No providers match this filter. Try a different category above.</AdminEmptyInline>
          )}
          {visible.map((p) => (
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
                {p.configured
                  ? <CheckCircle2 size={14} className="text-success" />
                  : <Server size={14} className="text-muted-foreground" />
                }
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate font-display text-sm font-semibold text-foreground">{p.display_name}</p>
                <p className="font-mono text-xs text-muted-foreground">{p.model_count} models</p>
              </div>
              {p.id === defaultId && (
                <span className="shrink-0 rounded-full bg-primary/15 px-1.5 py-0.5 font-mono text-xs text-primary">default</span>
              )}
              <ChevronRight size={13} className="shrink-0 text-muted-foreground/40" />
            </Button>
          ))}
        </div>
      </div>

      {/* Detail (right column) */}
      <div className={cn("flex flex-1 flex-col overflow-hidden", !selected && "hidden md:flex")}>
        {selected ? (
          <ProviderDetail
            provider={selected}
            configured={configured.find((c) => c.id === selected.id)}
            isDefault={selected.id === defaultId}
            onSetDefault={() => void handleSetDefault(selected.id)}
            onConfigure={() => handleConfigure(selected)}
            onDelete={() => setRemoveTarget(selected.id)}
            onBack={() => setSelected(null)}
          />
        ) : (
          <div className="flex flex-1 items-center justify-center p-6">
            <div className="max-w-sm text-center">
              <div className="mx-auto mb-4 flex size-12 items-center justify-center rounded-xl bg-primary/10">
                <Server size={20} className="text-primary" />
              </div>
              {configured.length === 0 ? (
                <>
                  <p className="font-display text-sm font-semibold text-foreground">
                    No providers configured yet
                  </p>
                  <p className="mt-2 font-body text-xs leading-relaxed text-muted-foreground">
                    Providers connect this runtime to AI models. Select a provider from the list and click <strong>Configure</strong> to add your API key. Start with the provider you use most — you can always add more later.
                  </p>
                </>
              ) : (
                <>
                  <p className="font-display text-sm font-semibold text-foreground">
                    Select a provider
                  </p>
                  <p className="mt-2 font-body text-xs text-muted-foreground">
                    Choose a provider from the list to view its models and configuration.
                  </p>
                </>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Remove Provider AlertDialog */}
      <AlertDialog open={removeTarget !== null} onOpenChange={(open) => !open && setRemoveTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove provider configuration?</AlertDialogTitle>
            <AlertDialogDescription>
              This will remove the configuration for <strong>{removeTarget}</strong>. The provider will remain in the catalog but will need to be reconfigured to use.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={removing}>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={(e) => { e.preventDefault(); void handleDelete(); }} disabled={removing} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
              {removing ? <Loader2 size={12} className="animate-spin" /> : "Remove"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Configure Provider Dialog */}
      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Configure {addTarget?.display_name ?? "Provider"}</DialogTitle>
          </DialogHeader>
          <div className="flex flex-col gap-3">
            {addTarget?.auth_env_var && (
              <p className="rounded-md border border-border bg-muted/40 px-3 py-2 font-mono text-xs text-muted-foreground">
                You can also set the <code className="text-foreground">{addTarget.auth_env_var}</code> environment variable instead of entering a key here.
              </p>
            )}
            <div>
              <Label htmlFor="cfg-api-key" className="mb-1 block font-mono text-xs text-muted-foreground">API Key</Label>
              <Input
                id="cfg-api-key"
                type="password"
                value={form.api_key}
                onChange={(e) => setForm({ ...form, api_key: e.target.value })}
                placeholder="sk-… (optional if set via environment)"
              />
            </div>
            <div>
              <Label htmlFor="cfg-base-url" className="mb-1 block font-mono text-xs text-muted-foreground">Base URL Override</Label>
              <Input
                id="cfg-base-url"
                value={form.base_url}
                onChange={(e) => setForm({ ...form, base_url: e.target.value })}
                placeholder={addTarget?.base_url ?? "https://..."}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setShowAddDialog(false)}>Cancel</Button>
            <Button onClick={() => void handleSaveConfigure()} disabled={saving}>
              {saving && <Loader2 size={14} className="animate-spin" />}Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Provider detail panel
// ---------------------------------------------------------------------------

interface ProviderDetailProps {
  provider: CatalogProviderSummary;
  configured?: UarProvider;
  isDefault: boolean;
  onSetDefault: () => void;
  onConfigure: () => void;
  onDelete: () => void;
  onBack: () => void;
}

function ProviderDetail({ provider, configured, isDefault, onSetDefault, onConfigure, onDelete, onBack }: ProviderDetailProps) {
  const [models, setModels] = useState<Array<{ id: string; name: string; tool_call: boolean; reasoning: boolean; context: number }>>([]);
  const [loadingModels, setLoadingModels] = useState(true);

  useEffect(() => {
    setLoadingModels(true);
    fetch("/api/models")
      .then((r) => r.json())
      .then((data: Record<string, { models: Record<string, { name: string; tool_call: boolean; reasoning: boolean; limit: { context: number } }> }>) => {
        const providerData = data[provider.id];
        if (providerData?.models) {
          const entries = Object.entries(providerData.models).map(([id, m]) => ({
            id,
            name: m.name,
            tool_call: m.tool_call,
            reasoning: m.reasoning,
            context: m.limit.context,
          }));
          setModels(entries);
        } else {
          setModels([]);
        }
      })
      .catch(() => setModels([]))
      .finally(() => setLoadingModels(false));
  }, [provider.id]);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="border-b border-border bg-card px-4 py-4 sm:px-6">
        <div className="flex items-start justify-between">
          <div>
            <div className="flex items-center gap-2">
              <Button variant="ghost" size="icon" className="h-7 w-7 md:hidden" onClick={onBack} aria-label="Back to providers">
                <ArrowLeft size={14} />
              </Button>
              <h2 className="font-display text-lg font-semibold text-foreground">{provider.display_name}</h2>
              {provider.configured
                ? <Badge variant="outline" className="gap-1 border-success/40 text-success text-xs"><CheckCircle2 size={10} />Configured</Badge>
                : <Badge variant="outline" className="gap-1 text-muted-foreground text-xs"><XCircle size={10} />Not Configured</Badge>
              }
              {isDefault && <Badge className="text-xs">Default</Badge>}
            </div>
            <p className="mt-0.5 font-mono text-xs text-muted-foreground">{provider.base_url ?? "Default API endpoint"}</p>
            {provider.auth_env_var && (
              <p className="mt-1 font-mono text-xs text-muted-foreground">
                Auth: <code className="text-foreground">{provider.auth_env_var}</code>
              </p>
            )}
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" className="h-7 text-xs" onClick={onConfigure}>
              <Plus size={12} />{provider.configured ? "Edit Config" : "Configure"}
            </Button>
            {provider.configured && !isDefault && (
              <Button variant="outline" size="sm" onClick={onSetDefault} className="h-7 text-xs">Set as default</Button>
            )}
            {provider.configured && (
              <Button variant="outline" size="sm" onClick={onDelete} className="h-7 border-destructive/40 text-xs text-destructive hover:bg-destructive/10">Remove</Button>
            )}
          </div>
        </div>
        {/* Endpoint badges */}
        {provider.endpoints.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-1">
            {provider.endpoints.map((ep) => (
              <span key={ep} className="rounded border border-border px-1.5 py-0.5 font-mono text-xs text-muted-foreground">{ep}</span>
            ))}
          </div>
        )}
      </div>

      {/* Model list */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        <div className="mb-3 flex items-center gap-2">
          <p className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">Models ({provider.model_count})</p>
          <Separator className="flex-1" />
        </div>

        {loadingModels ? (
          <div className="flex items-center gap-2 py-4">
            <Loader2 size={14} className="animate-spin text-muted-foreground" />
            <span className="font-mono text-xs text-muted-foreground">Loading models…</span>
          </div>
        ) : models.length === 0 ? (
          <p className="font-mono text-xs text-muted-foreground">Model details will appear once this provider is configured.</p>
        ) : (
          <div className="flex flex-col gap-1">
            {models.map((m) => (
              <div key={m.id} className="flex items-center gap-3 rounded-md border border-border/50 bg-card px-3 py-2.5">
                <Circle size={8} className="shrink-0 fill-success text-success" />
                <div className="min-w-0 flex-1">
                  <p className="truncate font-mono text-xs text-foreground">{m.name || m.id}</p>
                  <p className="font-mono text-xs text-muted-foreground">{m.id}</p>
                </div>
                <div className="flex shrink-0 items-center gap-1">
                  {m.context > 0 && <span className="font-mono text-xs text-muted-foreground">{(m.context / 1000).toFixed(0)}k</span>}
                  {m.tool_call && <span className="rounded border border-border px-1 font-mono text-xs text-muted-foreground">tools</span>}
                  {m.reasoning && <span className="rounded border border-border px-1 font-mono text-xs text-muted-foreground">reasoning</span>}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
