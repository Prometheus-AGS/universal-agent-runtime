import { type FC, useEffect, useState } from "react";
import { AlertTriangle, ArrowLeft, CheckCircle2, ChevronRight, Circle, Loader2, Plus, RefreshCw, Search, Server, Star, XCircle } from "lucide-react";
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
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { AdminEmptyInline, AdminError, AdminSidebarSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import { useProviderModels } from "@/hooks/use-provider-models";
import { useProviders } from "@/entities/hooks/use-providers";
import { useProviderDefault } from "@/entities/hooks/use-provider-default";
import type { CatalogProviderSummary } from "@/types";
import { loadProvidersIntoGraph } from "@/entities/fetchers/providers";
import { optimisticUpsert, optimisticRemove } from "@/lib/realtime/optimistic";
import {
  createProvider as createProviderApi,
  deleteProvider as deleteProviderApi,
  setDefaultProvider as setDefaultProviderApi,
} from "@/services/providers-api";

export const ProvidersPage: FC = () => {
  // Reads come from the entity graph (hydrated by loadProvidersIntoGraph and
  // kept fresh by SSE realtime).
  const providersView = useProviders();
  const defaultId = useProviderDefault() ?? undefined;

  // Local UI state — no longer routed through a Zustand store.
  const [saving, setSaving] = useState(false);
  const [removing, setRemoving] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // The graph stores rich ProviderEntity rows that are a superset of
  // CatalogProviderSummary; cast for the legacy render code below.
  const catalog = providersView.items as unknown as CatalogProviderSummary[];
  const configured = catalog.filter((p) => p.configured);
  const loading = providersView.items.length === 0;

  // Populate the entity graph on mount and provide a manual refresh.
  useEffect(() => {
    void loadProvidersIntoGraph();
  }, []);
  const load = () => loadProvidersIntoGraph();

  // ── Mutations ────────────────────────────────────────────────────────────
  //
  // - configureProvider: non-optimistic per the global rule for creates; on
  //   success we re-run loadProvidersIntoGraph so the new row + ProviderMeta
  //   reach the graph.
  // - setDefault: optimistic — flip ProviderMeta singleton immediately;
  //   rollback the singleton's default_id on failure.
  // - removeProvider: capture the entity snapshot, optimistically remove from
  //   the graph, and on failure re-upsert the snapshot.

  const configureProvider = async (args: {
    addTarget: CatalogProviderSummary;
    apiKey: string;
    baseUrl: string;
  }) => {
    const { addTarget, apiKey, baseUrl } = args;
    setSaving(true);
    setError(null);
    try {
      const resolvedBase = (baseUrl.trim() || addTarget.base_url) ?? "";
      const res = await createProviderApi({
        id: addTarget.id,
        display_name: addTarget.display_name ?? "",
        base_url: resolvedBase,
        api_key: apiKey || undefined,
        protocol: "auto",
        enabled: true,
      });
      if (!res.ok && res.status !== 409) throw new Error(`${res.status}`);
      await loadProvidersIntoGraph();
    } catch (e) {
      setError((e as Error).message);
      throw e;
    } finally {
      setSaving(false);
    }
  };

  const setDefault = async (id: string) => {
    try {
      await optimisticUpsert(
        "ProviderMeta",
        "current",
        { id: "current", default_id: id },
        async () => {
          await setDefaultProviderApi(id);
        },
      );
    } catch (e) {
      setError(`Failed to set default: ${(e as Error).message}`);
    }
  };

  const removeProvider = async (id: string) => {
    setRemoving(id);
    setError(null);
    try {
      await optimisticRemove("Provider", id, async () => {
        await deleteProviderApi(id);
      });
    } catch (e) {
      setError(`Failed to remove provider: ${(e as Error).message}`);
    } finally {
      setRemoving(null);
    }
  };

  const [selected, setSelected] = useState<CatalogProviderSummary | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [addTarget, setAddTarget] = useState<CatalogProviderSummary | null>(null);
  const [form, setForm] = useState({ api_key: "", base_url: "" });
  const [filter, setFilter] = useState<"all" | "configured" | "unconfigured">("all");
  const [searchTerm, setSearchTerm] = useState("");
  const [removeTarget, setRemoveTarget] = useState<string | null>(null);

  const handleConfigure = (provider: CatalogProviderSummary) => {
    setAddTarget(provider);
    setForm({ api_key: "", base_url: provider.base_url ?? "" });
    setShowAddDialog(true);
  };

  const handleSaveConfigure = async () => {
    if (!addTarget) return;
    try {
      await configureProvider({
        addTarget,
        apiKey: form.api_key,
        baseUrl: form.base_url,
      });
      setShowAddDialog(false);
    } catch {
      /* error surfaced via store */
    }
  };

  const handleSetDefault = (id: string) => {
    void setDefault(id);
  };

  const handleDelete = async () => {
    if (!removeTarget) return;
    await removeProvider(removeTarget);
    if (selected?.id === removeTarget) setSelected(null);
    setRemoveTarget(null);
  };

  const visible = catalog
    .filter((p) => {
      if (filter === "configured") return p.configured;
      if (filter === "unconfigured") return !p.configured;
      return true;
    })
    .filter((p) => {
      if (!searchTerm.trim()) return true;
      const q = searchTerm.toLowerCase();
      return p.display_name.toLowerCase().includes(q) || p.id.toLowerCase().includes(q);
    })
    .sort((a, b) => {
      // Configured first
      if (a.configured !== b.configured) return a.configured ? -1 : 1;
      // Then alphabetical
      return a.display_name.localeCompare(b.display_name);
    });

  return (
    <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
      {/* Provider list (left column) */}
      <div className={cn("flex shrink-0 flex-col border-b border-border bg-background md:w-72 md:border-b-0 md:border-r", selected ? "hidden md:flex" : "flex flex-1 md:flex-initial")}>
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <div>
            <h2
              className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground"
              data-testid="providers-heading"
            >
              Providers
            </h2>
            <p className="font-mono text-xs text-muted-foreground">{catalog.length} available · {configured.length} configured</p>
          </div>
          <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => void load()} aria-label="Refresh">
            <RefreshCw size={12} className={cn(loading && "animate-spin")} />
          </Button>
        </div>

        {/* Search */}
        <div className="border-b border-border px-3 py-2">
          <div className="relative">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search providers..."
              className="h-8 pl-8 font-mono text-xs"
            />
          </div>
        </div>

        {/* Filter tabs */}
        <div className="flex gap-1 border-b border-border px-3 py-2">
          {(["all", "configured", "unconfigured"] as const).map((f) => (
            <button
              type="button"
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
                {p.configured ? (
                  <CheckCircle2 size={14} className="text-success" />
                ) : p.status === "credential-blocked" ? (
                  <AlertTriangle size={14} className="text-amber-500" />
                ) : (
                  <Server size={14} className="text-muted-foreground" />
                )}
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate font-display text-sm font-semibold text-foreground">{p.display_name}</p>
                <p className="font-mono text-xs text-muted-foreground">{p.model_count} models</p>
              </div>
              {p.id === defaultId && <ProviderStatusPill tone="primary" label="default" />}
              {!p.configured && p.status === "credential-blocked" && (
                <ProviderStatusPill tone="warning" label="credential" />
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
            key={selected.id}
            provider={selected}
            isDefault={selected.id === defaultId}
            onSetDefault={() => handleSetDefault(selected.id)}
            onConfigure={() => handleConfigure(selected)}
            onDelete={() => setRemoveTarget(selected.id)}
            onBack={() => setSelected(null)}
          />
        ) : (
          <div className="flex flex-1 flex-col overflow-hidden">
            {/* Default model banner */}
            <DefaultModelBanner defaultId={defaultId} catalog={catalog} />
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
            <AlertDialogCancel disabled={removing !== null}>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={(e) => { e.preventDefault(); void handleDelete(); }} disabled={removing !== null} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
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
            <DialogDescription className="sr-only">
              Optional API key and base URL override for this LLM provider.
            </DialogDescription>
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
// Default model banner (shown when no provider is selected)
// ---------------------------------------------------------------------------

function DefaultModelBanner({
  defaultId,
  catalog,
}: {
  defaultId: string | undefined;
  catalog: CatalogProviderSummary[];
}) {
  const defaultProvider = defaultId ? catalog.find((p) => p.id === defaultId) : undefined;

  if (defaultId && defaultProvider) {
    return (
      <div className="flex items-center gap-3 border-b border-border bg-primary/5 px-4 py-3">
        <Star size={14} className="shrink-0 text-primary" />
        <div className="min-w-0 flex-1">
          <p className="font-mono text-xs text-muted-foreground">Default provider</p>
          <p className="truncate font-display text-sm font-semibold text-foreground">
            {defaultProvider.display_name}
            <span className="ml-1.5 font-mono text-xs font-normal text-muted-foreground">({defaultId})</span>
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-3 border-b border-amber-500/20 bg-amber-500/5 px-4 py-3">
      <div className="flex size-6 shrink-0 items-center justify-center rounded-full bg-amber-500/15">
        <AlertTriangle size={12} className="text-amber-500" />
      </div>
      <div className="min-w-0 flex-1">
        <p className="font-display text-sm font-semibold text-foreground">No default provider set</p>
        <p className="font-body text-xs text-muted-foreground">
          Select a configured provider and click "Set as default" to designate the default model for chat.
        </p>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Provider detail panel
// ---------------------------------------------------------------------------

interface ProviderDetailProps {
  provider: CatalogProviderSummary;
  isDefault: boolean;
  onSetDefault: () => void;
  onConfigure: () => void;
  onDelete: () => void;
  onBack: () => void;
}

function ProviderDetail({ provider, isDefault, onSetDefault, onConfigure, onDelete, onBack }: ProviderDetailProps) {
  const { models, loadingModels } = useProviderModels(provider.id);
  const statusCopy = getProviderStatusCopy(provider);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Default model banner when this provider IS the default */}
      {isDefault && (
        <div className="flex items-center gap-3 border-b border-primary/20 bg-primary/5 px-4 py-2.5 sm:px-6">
          <Star size={14} className="shrink-0 fill-primary text-primary" />
          <p className="font-display text-sm font-semibold text-primary">
            Default Provider
          </p>
          {models.length > 0 && (
            <span className="font-mono text-xs text-muted-foreground">
              {provider.id}/{models[0].id}
            </span>
          )}
        </div>
      )}

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
              {provider.status === "credential-blocked" && (
                <Badge variant="outline" className="gap-1 border-amber-500/40 text-amber-600 text-xs">
                  <AlertTriangle size={10} />Credential blocked
                </Badge>
              )}
              {isDefault && <Badge className="text-xs">Default</Badge>}
            </div>
            <p className="mt-0.5 font-mono text-xs text-muted-foreground">{provider.base_url ?? "Default API endpoint"}</p>
            {provider.auth_env_var && (
              <p className="mt-1 font-mono text-xs text-muted-foreground">
                Auth: <code className="text-foreground">{provider.auth_env_var}</code>
              </p>
            )}
            <p className="mt-1 font-body text-xs text-muted-foreground">{statusCopy}</p>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" className="h-7 text-xs" onClick={onConfigure}>
              <Plus size={12} />{provider.configured ? "Edit Config" : "Configure"}
            </Button>
            {provider.configured && !isDefault && (
              <Button variant="outline" size="sm" onClick={onSetDefault} className="h-7 text-xs">
                <Star size={12} />Set as default
              </Button>
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

function ProviderStatusPill({ tone, label }: { tone: "primary" | "warning"; label: string }) {
  return (
    <span
      className={cn(
        "shrink-0 rounded-full px-1.5 py-0.5 font-mono text-xs",
        tone === "primary" ? "bg-primary/15 text-primary" : "bg-amber-500/10 text-amber-600",
      )}
    >
      {label}
    </span>
  );
}

function getProviderStatusCopy(provider: CatalogProviderSummary): string {
  if (provider.status_detail) return provider.status_detail;
  if (provider.configured) return `${provider.display_name} is configured and enabled.`;
  if (provider.auth_env_var) return `${provider.display_name} requires ${provider.auth_env_var} before live requests can be tested.`;
  return `${provider.display_name} is available in the catalog but is not configured.`;
}
