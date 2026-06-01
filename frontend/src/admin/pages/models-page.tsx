import { type FC, useCallback, useEffect, useMemo, useState } from "react";
import { Loader2, Plus, RefreshCw, Search, Star, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";
import { useModels } from "@/entities/hooks/use-models";
import { loadModelsIntoGraph } from "@/entities/fetchers/models";
import { fetchConfiguredProviders, updateProvider } from "@/services/providers-api";
import type { UarModel, UarProvider } from "@/types";

// ---------------------------------------------------------------------------
// Page-local model row shape (catalog view)
// ---------------------------------------------------------------------------

interface ModelRowShape {
  key: string;             // "openai/gpt-4o"
  provider_id: string;
  provider_name: string;
  provider_configured: boolean;
  model_id: string;
  name: string;
  context: number;
  tool_call: boolean;
  reasoning: boolean;
  vision: boolean;
  open_weights?: boolean;
  cost_input: number;
  cost_output: number;
}

/** A configured model belonging to a provider, paired with its provider. */
interface ConfiguredModelRow {
  provider: UarProvider;
  model: UarModel;
  isDefault: boolean;
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export const ModelsPage: FC = () => {
  // Catalog reads come from the entity graph. Hydration is the page's job.
  const view = useModels();
  const rawItems = view.items as ReadonlyArray<Record<string, unknown>>;

  // Configured providers (authoritative `models[]` + `default_model`) are
  // fetched directly — they are not mirrored into the catalog Model entities.
  const [configured, setConfigured] = useState<UarProvider[]>([]);

  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedProvider, setSelectedProvider] = useState<string>("all");
  const [query, setQuery] = useState("");
  const [capabilities, setCapabilities] = useState<{ tools: boolean; reasoning: boolean; vision: boolean }>({
    tools: false, reasoning: false, vision: false,
  });

  // Mutation state
  const [busyModelKey, setBusyModelKey] = useState<string | null>(null);
  const [removeTarget, setRemoveTarget] = useState<{ providerId: string; modelId: string } | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);

  const loadConfigured = useCallback(async () => {
    const res = await fetchConfiguredProviders();
    setConfigured(res.providers);
  }, []);

  const load = useCallback(async () => {
    setRefreshing(true);
    setError(null);
    try {
      await Promise.all([loadModelsIntoGraph(), loadConfigured()]);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setRefreshing(false);
    }
  }, [loadConfigured]);

  useEffect(() => {
    // Defer past the synchronous effect body so the initial `setRefreshing`
    // inside `load` does not run as a synchronous setState in the effect.
    const id = setTimeout(() => void load(), 0);
    return () => clearTimeout(id);
  }, [load]);

  const loading = refreshing && rawItems.length === 0;

  // Adapt the loose graph rows into typed catalog model rows for rendering.
  const allModels: ModelRowShape[] = useMemo(() => {
    return rawItems.map((row) => ({
      key: row.id as string,
      provider_id: row.provider_id as string,
      provider_name: (row.provider_name as string) ?? (row.provider_id as string),
      provider_configured: row.provider_configured === true,
      model_id: (row.model_id as string) ?? "",
      name: (row.name as string) ?? "",
      context: (row.context as number) ?? 0,
      tool_call: row.tool_call === true,
      reasoning: row.reasoning === true,
      vision: row.vision === true,
      open_weights: row.open_weights === true,
      cost_input: (row.cost_input as number) ?? 0,
      cost_output: (row.cost_output as number) ?? 0,
    }));
  }, [rawItems]);

  const providers = useMemo(() => {
    const seen = new Set<string>();
    return allModels
      .filter((m) => !seen.has(m.provider_id) && seen.add(m.provider_id))
      .map((m) => ({ id: m.provider_id, name: m.provider_name, configured: m.provider_configured }));
  }, [allModels]);

  // Flatten configured providers into per-model rows for the configured section.
  const configuredRows: ConfiguredModelRow[] = useMemo(() => {
    const rows: ConfiguredModelRow[] = [];
    for (const provider of configured) {
      for (const model of provider.models ?? []) {
        rows.push({
          provider,
          model,
          isDefault: provider.default_model === model.id,
        });
      }
    }
    return rows;
  }, [configured]);

  const configuredCount = configuredRows.length;

  const filtered = useMemo(() => {
    let list = allModels;
    if (selectedProvider !== "all") list = list.filter((m) => m.provider_id === selectedProvider);
    if (capabilities.tools) list = list.filter((m) => m.tool_call);
    if (capabilities.reasoning) list = list.filter((m) => m.reasoning);
    if (capabilities.vision) list = list.filter((m) => m.vision);
    if (query) {
      const q = query.toLowerCase();
      list = list.filter((m) => m.key.toLowerCase().includes(q) || m.name.toLowerCase().includes(q));
    }
    return list;
  }, [allModels, selectedProvider, capabilities, query]);

  const toggleCap = (cap: keyof typeof capabilities) =>
    setCapabilities((prev) => ({ ...prev, [cap]: !prev[cap] }));

  // ── Mutations ──────────────────────────────────────────────────────────
  //
  // All writes go through PUT /api/uar/providers/{id} with the full provider
  // config. We optimistically patch local `configured` state, call the PUT,
  // then reconcile from the server response; on failure we restore the prior
  // snapshot. There is no dedicated models write endpoint.

  const putProvider = useCallback(
    async (next: UarProvider) => {
      const snapshot = configured;
      setConfigured((prev) => prev.map((p) => (p.id === next.id ? next : p)));
      setError(null);
      try {
        const saved = await updateProvider(next.id, next);
        setConfigured((prev) => prev.map((p) => (p.id === saved.id ? saved : p)));
      } catch (e) {
        setConfigured(snapshot);
        setError((e as Error).message);
        throw e;
      }
    },
    [configured],
  );

  const handleAddModel = async (providerId: string, model: UarModel) => {
    const provider = configured.find((p) => p.id === providerId);
    if (!provider) return;
    const next: UarProvider = {
      ...provider,
      models: [...(provider.models ?? []), model],
    };
    setBusyModelKey(`${providerId}/${model.id}`);
    try {
      await putProvider(next);
      setShowAddDialog(false);
    } catch {
      /* error surfaced via state */
    } finally {
      setBusyModelKey(null);
    }
  };

  const handleSetDefault = async (providerId: string, modelId: string) => {
    const provider = configured.find((p) => p.id === providerId);
    if (!provider) return;
    const next: UarProvider = { ...provider, default_model: modelId };
    setBusyModelKey(`${providerId}/${modelId}`);
    try {
      await putProvider(next);
    } catch {
      /* error surfaced via state */
    } finally {
      setBusyModelKey(null);
    }
  };

  const handleRemoveModel = async () => {
    if (!removeTarget) return;
    const { providerId, modelId } = removeTarget;
    const provider = configured.find((p) => p.id === providerId);
    if (!provider) {
      setRemoveTarget(null);
      return;
    }
    const next: UarProvider = {
      ...provider,
      models: (provider.models ?? []).filter((m) => m.id !== modelId),
      // Drop the default pointer if it referenced the removed model.
      default_model: provider.default_model === modelId ? undefined : provider.default_model,
    };
    setBusyModelKey(`${providerId}/${modelId}`);
    try {
      await putProvider(next);
    } catch {
      /* error surfaced via state */
    } finally {
      setBusyModelKey(null);
      setRemoveTarget(null);
    }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden font-mono text-[13px] text-[hsl(var(--terminal-fg))]">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[hsl(var(--terminal-line-strong))] bg-[hsl(var(--terminal-surface))] px-6 py-4">
        <div>
          <h2 className="text-[20px] font-medium tracking-tight text-[hsl(var(--terminal-fg))]">models</h2>
          <p className="text-xs text-[hsl(var(--terminal-fg-dim))]">
            <span data-testid="models-configured-count">{configuredCount}</span> configured ·{" "}
            <span data-testid="models-count">{filtered.length}</span> of {allModels.length} in catalog · {providers.length} providers
            {refreshing && <TerminalCursor className="ml-2" />}
          </p>
        </div>
        <div className="flex items-center gap-1.5">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => setShowAddDialog(true)}
            disabled={configured.length === 0}
            className="gap-1.5 border border-[hsl(var(--phosphor)/0.4)] bg-transparent text-[hsl(var(--phosphor))] hover:bg-[hsl(var(--phosphor)/0.08)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))] disabled:opacity-40"
            aria-label="Add model from catalog"
            data-testid="models-add-button"
          >
            <Plus size={13} />add model
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => void load()}
            className="gap-1.5 border border-[hsl(var(--terminal-line-strong))] bg-transparent text-[hsl(var(--terminal-fg))] hover:bg-[hsl(var(--phosphor)/0.08)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))]"
            aria-label="Refresh models"
          >
            <RefreshCw size={13} className={cn(refreshing && "animate-spin")} />refresh
          </Button>
        </div>
      </div>

      {/* Error bar */}
      {error && (
        <div className="border-b border-[hsl(var(--signal-red))] bg-[hsl(var(--signal-red)/0.08)] px-6 py-2 text-xs text-[hsl(var(--signal-red))]">
          <span className="mr-2 font-semibold">ERR-MODELS</span>{error}
        </div>
      )}

      <div className="flex-1 overflow-y-auto">
        {/* Configured models section */}
        <section className="border-b border-[hsl(var(--terminal-line))] px-6 py-4" aria-label="Configured models">
          <div className="mb-3 flex items-center gap-2">
            <span className="text-xs font-medium uppercase tracking-widest text-[hsl(var(--terminal-fg-dim))]">
              configured
            </span>
            <span className="h-px flex-1 bg-[hsl(var(--terminal-line))]" aria-hidden />
          </div>

          {configuredRows.length === 0 ? (
            <EmptyState
              title="0 models configured"
              hint={
                configured.length === 0
                  ? "configure a provider first, then add a model"
                  : "click add model to enable one from the catalog"
              }
            />
          ) : (
            <div className="flex flex-col gap-1.5" data-testid="configured-models-list">
              {configuredRows.map(({ provider, model, isDefault }) => {
                const rowKey = `${provider.id}/${model.id}`;
                const busy = busyModelKey === rowKey;
                return (
                  <ConfiguredModelRow
                    key={rowKey}
                    providerId={provider.id}
                    providerName={provider.display_name ?? provider.id}
                    model={model}
                    isDefault={isDefault}
                    busy={busy}
                    onSetDefault={() => void handleSetDefault(provider.id, model.id)}
                    onRemove={() => setRemoveTarget({ providerId: provider.id, modelId: model.id })}
                  />
                );
              })}
            </div>
          )}
        </section>

        {/* Catalog section */}
        <section className="px-6 py-4" aria-label="Model catalog">
          <div className="mb-3 flex items-center gap-2">
            <span className="text-xs font-medium uppercase tracking-widest text-[hsl(var(--terminal-fg-dim))]">
              catalog
            </span>
            <span className="h-px flex-1 bg-[hsl(var(--terminal-line))]" aria-hidden />
          </div>

          {/* Filters */}
          <div className="mb-3 flex flex-col gap-2">
            <div className="relative">
              <Search size={13} className="absolute left-3 top-1/2 -translate-y-1/2 text-[hsl(var(--terminal-fg-dim))]" />
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="search models…"
                className="h-8 border-[hsl(var(--terminal-line-strong))] bg-[hsl(var(--terminal-surface))] pl-8 font-mono text-xs text-[hsl(var(--terminal-fg))] placeholder:text-[hsl(var(--terminal-fg-dim))] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))]"
              />
            </div>

            <div className="flex flex-wrap gap-1">
              <FilterPill active={selectedProvider === "all"} onClick={() => setSelectedProvider("all")}>all</FilterPill>
              {providers.map((p) => (
                <FilterPill
                  key={p.id}
                  active={selectedProvider === p.id}
                  dimmed={!p.configured}
                  onClick={() => setSelectedProvider(p.id)}
                >
                  {p.name}
                </FilterPill>
              ))}
            </div>

            <div className="flex gap-1">
              {(["tools", "reasoning", "vision"] as const).map((cap) => (
                <FilterPill key={cap} active={capabilities[cap]} onClick={() => toggleCap(cap)}>
                  {cap}
                </FilterPill>
              ))}
            </div>
          </div>

          {loading && (
            <div className="flex items-center gap-2 text-[hsl(var(--terminal-fg-dim))]">
              <TerminalCursor /> <span className="text-xs">loading models</span>
            </div>
          )}
          {!loading && filtered.length === 0 && (
            <EmptyState
              title={allModels.length === 0 ? "no models available" : "no models match filters"}
              hint={
                allModels.length === 0
                  ? "configure a provider first to populate the catalog"
                  : "relax the filters or clear the search to see more"
              }
            />
          )}
          <div className="flex flex-col gap-1.5">
            {filtered.map((m) => (
              <ModelRow key={m.key} row={m} />
            ))}
          </div>
        </section>
      </div>

      {/* Add model dialog */}
      <AddModelDialog
        open={showAddDialog}
        onOpenChange={setShowAddDialog}
        configured={configured}
        catalog={allModels}
        busy={busyModelKey !== null}
        onAdd={handleAddModel}
      />

      {/* Remove model confirmation */}
      <AlertDialog open={removeTarget !== null} onOpenChange={(open) => !open && setRemoveTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove configured model?</AlertDialogTitle>
            <AlertDialogDescription>
              This removes <strong>{removeTarget?.modelId}</strong> from{" "}
              <strong>{removeTarget?.providerId}</strong>. The model stays in the catalog and can be
              re-enabled at any time.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={busyModelKey !== null}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={(e) => { e.preventDefault(); void handleRemoveModel(); }}
              disabled={busyModelKey !== null}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {busyModelKey !== null ? <Loader2 size={12} className="animate-spin" /> : "Remove"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Add model dialog
// ---------------------------------------------------------------------------

interface AddModelDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  configured: UarProvider[];
  catalog: ModelRowShape[];
  busy: boolean;
  onAdd: (providerId: string, model: UarModel) => void;
}

function AddModelDialog({ open, onOpenChange, configured, catalog, busy, onAdd }: AddModelDialogProps) {
  const [providerId, setProviderId] = useState<string>("");
  const [modelId, setModelId] = useState<string>("");

  // Reset selections whenever the dialog opens. Deferred so the resets do not
  // run as synchronous setState within the effect body.
  useEffect(() => {
    if (!open) return;
    const id = setTimeout(() => {
      setProviderId(configured[0]?.id ?? "");
      setModelId("");
    }, 0);
    return () => clearTimeout(id);
  }, [open, configured]);

  const selectedProvider = configured.find((p) => p.id === providerId);
  const enabledIds = new Set((selectedProvider?.models ?? []).map((m) => m.id));

  // Catalog models for the chosen provider that are not already enabled.
  const available = useMemo(
    () =>
      catalog.filter((m) => m.provider_id === providerId && !enabledIds.has(m.model_id)),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [catalog, providerId, selectedProvider],
  );

  const handleAdd = () => {
    const row = available.find((m) => m.model_id === modelId);
    if (!providerId || !row) return;
    const model: UarModel = {
      id: row.model_id,
      display_name: row.name || undefined,
      context_window: row.context > 0 ? row.context : undefined,
      supports_tools: row.tool_call,
      supports_vision: row.vision,
      enabled: true,
    };
    onAdd(providerId, model);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add model from catalog</DialogTitle>
          <DialogDescription>
            Enable a catalog model on a configured provider. This appends the model to the
            provider's configuration.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-3">
          <div>
            <Label htmlFor="add-model-provider" className="mb-1 block font-mono text-xs text-muted-foreground">
              Provider
            </Label>
            <Select value={providerId} onValueChange={(v) => { setProviderId(v); setModelId(""); }}>
              <SelectTrigger id="add-model-provider" data-testid="add-model-provider">
                <SelectValue placeholder="Select a provider" />
              </SelectTrigger>
              <SelectContent>
                {configured.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    {p.display_name ?? p.id}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div>
            <Label htmlFor="add-model-model" className="mb-1 block font-mono text-xs text-muted-foreground">
              Model
            </Label>
            <Select value={modelId} onValueChange={setModelId} disabled={!providerId || available.length === 0}>
              <SelectTrigger id="add-model-model" data-testid="add-model-model">
                <SelectValue
                  placeholder={
                    !providerId
                      ? "Select a provider first"
                      : available.length === 0
                        ? "All catalog models already enabled"
                        : "Select a model"
                  }
                />
              </SelectTrigger>
              <SelectContent>
                {available.map((m) => (
                  <SelectItem key={m.model_id} value={m.model_id}>
                    {m.name || m.model_id}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button onClick={handleAdd} disabled={busy || !providerId || !modelId} data-testid="add-model-confirm">
            {busy && <Loader2 size={14} className="animate-spin" />}Add model
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// ---------------------------------------------------------------------------
// Configured model row
// ---------------------------------------------------------------------------

interface ConfiguredModelRowProps {
  providerId: string;
  providerName: string;
  model: UarModel;
  isDefault: boolean;
  busy: boolean;
  onSetDefault: () => void;
  onRemove: () => void;
}

function ConfiguredModelRow({
  providerId,
  providerName,
  model,
  isDefault,
  busy,
  onSetDefault,
  onRemove,
}: ConfiguredModelRowProps) {
  const ctxK = model.context_window && model.context_window > 0
    ? `${Math.round(model.context_window / 1000)}k`
    : null;
  return (
    <div
      className={cn(
        "flex items-center gap-3 border px-4 py-3 transition-colors duration-[160ms]",
        isDefault
          ? "border-[hsl(var(--phosphor)/0.5)] bg-[hsl(var(--phosphor)/0.06)]"
          : "border-[hsl(var(--terminal-line-strong))] bg-[hsl(var(--terminal-surface))] hover:border-[hsl(var(--phosphor)/0.4)]",
      )}
      data-testid={`configured-model-${providerId}/${model.id}`}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="truncate text-[13px] font-medium text-[hsl(var(--terminal-fg))]">
            {model.display_name || model.id}
          </p>
          {isDefault && (
            <span className="inline-flex items-center gap-1 rounded border border-[hsl(var(--phosphor)/0.5)] px-1.5 py-0 text-xs text-[hsl(var(--phosphor))]">
              <Star size={10} className="fill-current" />default
            </span>
          )}
        </div>
        <p className="text-xs text-[hsl(var(--terminal-fg-dim))]">{providerId}/{model.id}</p>
      </div>

      <div className="flex shrink-0 flex-wrap items-center gap-1.5">
        {ctxK && <span className="text-xs text-[hsl(var(--terminal-fg-dim))]">{ctxK} ctx</span>}
        {model.supports_tools && <CapBadge>tools</CapBadge>}
        {model.supports_vision && <CapBadge>vision</CapBadge>}
        <span className="sr-only">{providerName}</span>
      </div>

      <div className="flex shrink-0 items-center gap-1">
        {!isDefault && (
          <button
            type="button"
            onClick={onSetDefault}
            disabled={busy}
            aria-label={`Set ${model.id} as default for ${providerId}`}
            className="inline-flex h-7 items-center gap-1 rounded border border-[hsl(var(--terminal-line-strong))] px-2 text-xs text-[hsl(var(--terminal-fg-dim))] transition-colors hover:border-[hsl(var(--phosphor)/0.4)] hover:text-[hsl(var(--phosphor))] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))] disabled:opacity-40"
          >
            {busy ? <Loader2 size={11} className="animate-spin" /> : <Star size={11} />}default
          </button>
        )}
        <button
          type="button"
          onClick={onRemove}
          disabled={busy}
          aria-label={`Remove ${model.id} from ${providerId}`}
          className="inline-flex h-7 w-7 items-center justify-center rounded border border-[hsl(var(--terminal-line-strong))] text-[hsl(var(--terminal-fg-dim))] transition-colors hover:border-[hsl(var(--signal-red)/0.5)] hover:text-[hsl(var(--signal-red))] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))] disabled:opacity-40"
        >
          <Trash2 size={12} />
        </button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Catalog pieces
// ---------------------------------------------------------------------------

function FilterPill({
  active,
  dimmed,
  onClick,
  children,
}: {
  active: boolean;
  dimmed?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "h-7 rounded border px-2.5 py-0.5 font-mono text-xs lowercase tracking-wide transition-colors duration-[160ms] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))]",
        active
          ? "border-[hsl(var(--phosphor))] bg-[hsl(var(--phosphor)/0.12)] text-[hsl(var(--phosphor))]"
          : "border-[hsl(var(--terminal-line-strong))] text-[hsl(var(--terminal-fg-dim))] hover:border-[hsl(var(--phosphor)/0.4)] hover:text-[hsl(var(--terminal-fg))]",
        dimmed && "opacity-40",
      )}
    >
      {children}
    </button>
  );
}

function ModelRow({ row }: { row: ModelRowShape }) {
  const ctxK = row.context > 0 ? `${Math.round(row.context / 1000)}k` : null;
  return (
    <div
      className={cn(
        "flex items-center gap-3 border px-4 py-3 transition-colors duration-[160ms]",
        row.provider_configured
          ? "border-[hsl(var(--terminal-line-strong))] bg-[hsl(var(--terminal-surface))] hover:border-[hsl(var(--phosphor)/0.4)]"
          : "border-[hsl(var(--terminal-line))] bg-transparent opacity-50",
      )}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="truncate text-[13px] font-medium text-[hsl(var(--terminal-fg))]">{row.name || row.model_id}</p>
          {!row.provider_configured && (
            <span className="text-xs text-[hsl(var(--amber))]">[not configured]</span>
          )}
        </div>
        <p className="text-xs text-[hsl(var(--terminal-fg-dim))]">{row.key}</p>
      </div>

      <div className="flex shrink-0 flex-wrap items-center gap-1.5">
        {ctxK && <span className="text-xs text-[hsl(var(--terminal-fg-dim))]">{ctxK} ctx</span>}
        {row.cost_input > 0 && (
          <span className="text-xs text-[hsl(var(--terminal-fg-dim))]">
            ${row.cost_input.toFixed(2)}/${row.cost_output.toFixed(2)} per 1M
          </span>
        )}
        {row.tool_call && <CapBadge>tools</CapBadge>}
        {row.reasoning && <CapBadge>reasoning</CapBadge>}
        {row.vision && <CapBadge>vision</CapBadge>}
        {row.open_weights && <CapBadge tone="amber">open</CapBadge>}
      </div>
    </div>
  );
}

function CapBadge({ children, tone }: { children: React.ReactNode; tone?: "amber" }) {
  return (
    <span
      className={cn(
        "rounded border px-1.5 py-0 text-xs lowercase tracking-tight",
        tone === "amber"
          ? "border-[hsl(var(--amber)/0.5)] text-[hsl(var(--amber))]"
          : "border-[hsl(var(--phosphor)/0.4)] text-[hsl(var(--phosphor))]",
      )}
    >
      {children}
    </span>
  );
}

/** Flicker-cursor placeholder ▍ for inline loading. */
function TerminalCursor({ className }: { className?: string }) {
  return (
    <span
      aria-hidden
      className={cn("inline-block text-[hsl(var(--phosphor))]", className)}
      style={{ animation: "terminal-cursor-blink 600ms steps(1, end) infinite" }}
    >
      ▍
    </span>
  );
}

/** Inline ASCII-frame empty state. The shared component lands in change-4. */
function EmptyState({ title, hint }: { title: string; hint: string }) {
  return (
    <div className="mx-auto max-w-md py-12 text-center text-xs text-[hsl(var(--terminal-fg-dim))]">
      <pre className="select-none text-[hsl(var(--terminal-fg-dim))] opacity-60" aria-hidden>
{`┌────────────────────────────┐
│                            │
│         ${title.padEnd(20, " ").slice(0, 20)}│
│                            │
└────────────────────────────┘`}
      </pre>
      <p className="mt-3 text-[hsl(var(--terminal-fg-dim))]">{hint}</p>
    </div>
  );
}
