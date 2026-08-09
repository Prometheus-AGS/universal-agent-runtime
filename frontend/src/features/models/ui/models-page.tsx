import { type FC, useCallback, useEffect, useMemo, useState } from "react";
import { Columns3, Loader2, Plus, RefreshCw, Search, Star, Trash2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
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
import { useModelsAdmin } from "../model/use-models-admin";
import {
  filterModelRows,
  projectModelRows,
  selectComparisonModels,
  type ModelRowShape,
} from "../model/model-catalog-view";
import type { CatalogModelBenchmark, UarModel, UarProvider } from "@/types";

const MAX_COMPARE = 4;

const DIMENSION_LABEL: Record<CatalogModelBenchmark["dimension"], string> = {
  coding: "coding",
  agentic: "agentic",
  context: "context recall",
};

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
  const {
    items,
    configured,
    refreshing,
    busyModelKey,
    error,
    load,
    addModel,
    setDefaultModel,
    removeModel,
  } = useModelsAdmin();
  const rawItems = items as ReadonlyArray<Record<string, unknown>>;

  const [selectedProvider, setSelectedProvider] = useState<string>("all");
  const [query, setQuery] = useState("");
  const [capabilities, setCapabilities] = useState<{ tools: boolean; reasoning: boolean; vision: boolean }>({
    tools: false, reasoning: false, vision: false,
  });

  const [removeTarget, setRemoveTarget] = useState<{ providerId: string; modelId: string } | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);

  // Compare state (CH-10): a bounded selection of catalog model keys, plus
  // whether the side-by-side dialog is open. Selection persists across
  // filter changes so switching providers to pick a second model doesn't
  // lose the first pick.
  const [compareKeys, setCompareKeys] = useState<string[]>([]);
  const [showCompare, setShowCompare] = useState(false);

  const toggleCompare = useCallback((key: string) => {
    setCompareKeys((prev) => {
      if (prev.includes(key)) return prev.filter((k) => k !== key);
      if (prev.length >= MAX_COMPARE) return prev;
      return [...prev, key];
    });
  }, []);

  const loading = refreshing && rawItems.length === 0;

  // Adapt the loose graph rows into typed catalog model rows for rendering.
  const allModels = useMemo(() => projectModelRows(rawItems), [rawItems]);

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

  const filtered = useMemo(
    () => filterModelRows(allModels, selectedProvider, capabilities, query),
    [allModels, selectedProvider, capabilities, query],
  );

  const toggleCap = (cap: keyof typeof capabilities) =>
    setCapabilities((prev) => ({ ...prev, [cap]: !prev[cap] }));

  // Preserve pick order (not catalog order) so a model stays where the user
  // put it when comparing.
  const compareModels = useMemo(
    () => selectComparisonModels(compareKeys, allModels),
    [compareKeys, allModels],
  );

  const handleAddModel = async (providerId: string, model: UarModel) => {
    try {
      await addModel(providerId, model);
      setShowAddDialog(false);
    } catch {
      /* error surfaced via state */
    }
  };

  const handleSetDefault = async (providerId: string, modelId: string) => {
    try {
      await setDefaultModel(providerId, modelId);
    } catch {
      /* error surfaced via state */
    }
  };

  const handleRemoveModel = async () => {
    if (!removeTarget) return;
    const { providerId, modelId } = removeTarget;
    try {
      await removeModel(providerId, modelId);
    } catch {
      /* error surfaced via state */
    } finally {
      setRemoveTarget(null);
    }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden font-mono text-[13px] text-[var(--color-fg)]">
      {/* Header */}
      <div className="flex items-center justify-between bg-[var(--color-surface)] px-6 py-4">
        <div>
          <h2 className="text-[20px] font-medium tracking-tight text-[var(--color-fg)]">models</h2>
          <p className="text-xs text-[var(--color-fg-sub)]">
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
            className="gap-1.5 border border-[color-mix(in_srgb,var(--color-ember)_40%,transparent)] bg-transparent text-[var(--color-ember)] hover:bg-[color-mix(in_srgb,var(--color-ember)_8%,transparent)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)] disabled:opacity-40"
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
            className="gap-1.5 bg-transparent text-[var(--color-fg)] hover:bg-[color-mix(in_srgb,var(--color-ember)_8%,transparent)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)]"
            aria-label="Refresh models"
          >
            <RefreshCw size={13} className={cn(refreshing && "animate-spin")} />refresh
          </Button>
        </div>
      </div>

      {/* Error bar */}
      {error && (
        <div className="border-b border-[var(--color-red)] bg-[color-mix(in_srgb,var(--color-red)_8%,transparent)] px-6 py-2 text-xs text-[var(--color-red)]">
          <span className="mr-2 font-semibold">ERR-MODELS</span>{error}
        </div>
      )}

      <div className="flex-1 overflow-y-auto">
        {/* Configured models section */}
        <section className=" px-6 py-4" aria-label="Configured models">
          <div className="mb-3 flex items-center gap-2">
            <span className="text-xs font-medium uppercase tracking-widest text-[var(--color-fg-sub)]">
              configured
            </span>
            <span className="flex-1" aria-hidden />
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
            <span className="text-xs font-medium uppercase tracking-widest text-[var(--color-fg-sub)]">
              catalog
            </span>
            <span className="flex-1" aria-hidden />
          </div>

          {/* Filters */}
          <div className="mb-3 flex flex-col gap-2">
            <div className="relative">
              <Search size={13} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-fg-sub)]" />
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="search models…"
                className="h-8 bg-[var(--color-surface)] pl-8 font-mono text-xs text-[var(--color-fg)] placeholder:text-[var(--color-fg-sub)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)]"
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
            <div className="flex items-center gap-2 text-[var(--color-fg-sub)]">
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
              <ModelRow
                key={m.key}
                row={m}
                compareChecked={compareKeys.includes(m.key)}
                compareDisabled={compareKeys.length >= MAX_COMPARE && !compareKeys.includes(m.key)}
                onToggleCompare={() => toggleCompare(m.key)}
              />
            ))}
          </div>
        </section>
      </div>

      {/* Compare bar (CH-10): appears once 1+ models are picked, so the user
          sees the running selection without losing catalog scroll position. */}
      {compareKeys.length > 0 && (
        <div
          className="flex items-center gap-3 border-t border-[color-mix(in_srgb,var(--color-ember)_40%,transparent)] bg-[var(--color-surface)] px-6 py-2.5"
          data-testid="compare-bar"
        >
          <Columns3 size={14} className="shrink-0 text-[var(--color-ember)]" aria-hidden />
          <div className="flex flex-1 flex-wrap items-center gap-1.5">
            {compareModels.map((m) => (
              <span
                key={m.key}
                className="inline-flex items-center gap-1 rounded py-0 pl-2 pr-1 text-xs text-[var(--color-fg)]"
              >
                {m.name || m.model_id}
                <button
                  type="button"
                  onClick={() => toggleCompare(m.key)}
                  aria-label={`Remove ${m.key} from compare`}
                  className="rounded p-0.5 text-[var(--color-fg-sub)] hover:text-[var(--color-red)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)]"
                >
                  <X size={10} />
                </button>
              </span>
            ))}
          </div>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => setCompareKeys([])}
            className="text-xs text-[var(--color-fg-sub)] hover:text-[var(--color-fg)]"
          >
            clear
          </Button>
          <Button
            type="button"
            size="sm"
            onClick={() => setShowCompare(true)}
            disabled={compareModels.length < 2}
            className="gap-1.5 border border-[color-mix(in_srgb,var(--color-ember)_50%,transparent)] bg-[color-mix(in_srgb,var(--color-ember)_12%,transparent)] text-[var(--color-ember)] hover:bg-[color-mix(in_srgb,var(--color-ember)_20%,transparent)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)] disabled:opacity-40"
            data-testid="open-compare"
          >
            <Columns3 size={13} />compare ({compareModels.length})
          </Button>
        </div>
      )}

      {/* Add model dialog */}
      <AddModelDialog
        open={showAddDialog}
        onOpenChange={setShowAddDialog}
        configured={configured}
        catalog={allModels}
        busy={busyModelKey !== null}
        onAdd={handleAddModel}
      />

      {/* Compare dialog (CH-10) */}
      <CompareDialog open={showCompare} onOpenChange={setShowCompare} models={compareModels} />

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
            <Select value={providerId} onValueChange={(v) => { if (v !== null) { setProviderId(v); setModelId(""); } }}>
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
            <Select value={modelId} onValueChange={(v) => { if (v !== null) setModelId(v); }} disabled={!providerId || available.length === 0}>
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
          ? "border-[color-mix(in_srgb,var(--color-ember)_50%,transparent)] bg-[color-mix(in_srgb,var(--color-ember)_6%,transparent)]"
          : " bg-[var(--color-surface)] hover:border-[color-mix(in_srgb,var(--color-ember)_40%,transparent)]",
      )}
      data-testid={`configured-model-${providerId}/${model.id}`}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="truncate text-[13px] font-medium text-[var(--color-fg)]">
            {model.display_name || model.id}
          </p>
          {isDefault && (
            <span className="inline-flex items-center gap-1 rounded border border-[color-mix(in_srgb,var(--color-ember)_50%,transparent)] px-1.5 py-0 text-xs text-[var(--color-ember)]">
              <Star size={10} className="fill-current" />default
            </span>
          )}
        </div>
        <p className="text-xs text-[var(--color-fg-sub)]">{providerId}/{model.id}</p>
      </div>

      <div className="flex shrink-0 flex-wrap items-center gap-1.5">
        {ctxK && <span className="text-xs text-[var(--color-fg-sub)]">{ctxK} ctx</span>}
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
            className="inline-flex h-7 items-center gap-1 rounded px-2 text-xs text-[var(--color-fg-sub)] transition-colors hover:border-[color-mix(in_srgb,var(--color-ember)_40%,transparent)] hover:text-[var(--color-ember)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)] disabled:opacity-40"
          >
            {busy ? <Loader2 size={11} className="animate-spin" /> : <Star size={11} />}default
          </button>
        )}
        <button
          type="button"
          onClick={onRemove}
          disabled={busy}
          aria-label={`Remove ${model.id} from ${providerId}`}
          className="inline-flex h-7 w-7 items-center justify-center rounded text-[var(--color-fg-sub)] transition-colors hover:border-[color-mix(in_srgb,var(--color-red)_50%,transparent)] hover:text-[var(--color-red)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)] disabled:opacity-40"
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
        "h-7 rounded border px-2.5 py-0.5 font-mono text-xs lowercase tracking-wide transition-colors duration-[160ms] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)]",
        active
          ? "border-[var(--color-ember)] bg-[color-mix(in_srgb,var(--color-ember)_12%,transparent)] text-[var(--color-ember)]"
          : " text-[var(--color-fg-sub)] hover:border-[color-mix(in_srgb,var(--color-ember)_40%,transparent)] hover:text-[var(--color-fg)]",
        dimmed && "opacity-40",
      )}
    >
      {children}
    </button>
  );
}

interface ModelRowProps {
  row: ModelRowShape;
  compareChecked: boolean;
  compareDisabled: boolean;
  onToggleCompare: () => void;
}

function ModelRow({ row, compareChecked, compareDisabled, onToggleCompare }: ModelRowProps) {
  const ctxK = row.context > 0 ? `${Math.round(row.context / 1000)}k` : null;
  return (
    <div
      className={cn(
        "flex items-center gap-3 border px-4 py-3 transition-colors duration-[160ms]",
        compareChecked && "border-[var(--color-ember)] bg-[color-mix(in_srgb,var(--color-ember)_6%,transparent)]",
        !compareChecked && row.provider_configured &&
          " bg-[var(--color-surface)] hover:border-[color-mix(in_srgb,var(--color-ember)_40%,transparent)]",
        !compareChecked && !row.provider_configured && " bg-transparent opacity-50",
      )}
    >
      <Checkbox
        checked={compareChecked}
        disabled={compareDisabled}
        onCheckedChange={onToggleCompare}
        aria-label={`Select ${row.name || row.model_id} for comparison`}
        data-testid={`compare-checkbox-${row.key}`}
      />

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="truncate text-[13px] font-medium text-[var(--color-fg)]">{row.name || row.model_id}</p>
          {!row.provider_configured && (
            <span className="text-xs text-[var(--color-amber)]">[not configured]</span>
          )}
        </div>
        <p className="text-xs text-[var(--color-fg-sub)]">{row.key}</p>
      </div>

      <div className="flex shrink-0 flex-wrap items-center gap-1.5">
        {ctxK && <span className="text-xs text-[var(--color-fg-sub)]">{ctxK} ctx</span>}
        {row.cost_input > 0 && (
          <span className="text-xs text-[var(--color-fg-sub)]">
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

// ---------------------------------------------------------------------------
// Compare dialog (CH-10)
// ---------------------------------------------------------------------------

function CompareDialog({
  open,
  onOpenChange,
  models,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  models: ModelRowShape[];
}) {
  // Union of every dimension any selected model has a score for — keeps the
  // benchmark section from padding out with all-empty rows for dimensions
  // nobody in the current selection has data for.
  const dimensions = useMemo(() => {
    const seen = new Set<CatalogModelBenchmark["dimension"]>();
    for (const m of models) for (const b of m.benchmarks) seen.add(b.dimension);
    return Array.from(seen);
  }, [models]);

  const bestScore = (dimension: CatalogModelBenchmark["dimension"]) =>
    Math.max(
      0,
      ...models.map((m) =>
        Math.max(0, ...m.benchmarks.filter((b) => b.dimension === dimension).map((b) => b.score)),
      ),
    );

  const cheapestInput = Math.min(...models.map((m) => m.cost_input || Number.POSITIVE_INFINITY));
  const largestContext = Math.max(...models.map((m) => m.context));

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-3xl font-mono">
        <DialogHeader>
          <DialogTitle>compare models</DialogTitle>
          <DialogDescription>
            side-by-side cost, capabilities, and sourced benchmark scores for {models.length} selected
            models.
          </DialogDescription>
        </DialogHeader>
        <div className="overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow className=" hover:bg-transparent">
                <TableHead className="text-xs text-[var(--color-fg-sub)]">metric</TableHead>
                {models.map((m) => (
                  <TableHead key={m.key} className="text-xs text-[var(--color-fg)]">
                    {m.name || m.model_id}
                    <div className="font-normal normal-case text-[var(--color-fg-sub)]">{m.key}</div>
                  </TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow>
                <TableCell className="text-xs text-[var(--color-fg-sub)]">context window</TableCell>
                {models.map((m) => (
                  <TableCell key={m.key} className="text-xs">
                    <ComparedValue highlight={m.context === largestContext && m.context > 0}>
                      {m.context > 0 ? `${Math.round(m.context / 1000)}k tokens` : "—"}
                    </ComparedValue>
                  </TableCell>
                ))}
              </TableRow>
              <TableRow>
                <TableCell className="text-xs text-[var(--color-fg-sub)]">cost / 1M tokens</TableCell>
                {models.map((m) => (
                  <TableCell key={m.key} className="text-xs">
                    <ComparedValue highlight={m.cost_input > 0 && m.cost_input === cheapestInput}>
                      {m.cost_input > 0 ? `$${m.cost_input.toFixed(2)} in / $${m.cost_output.toFixed(2)} out` : "—"}
                    </ComparedValue>
                  </TableCell>
                ))}
              </TableRow>
              <TableRow>
                <TableCell className="text-xs text-[var(--color-fg-sub)]">capabilities</TableCell>
                {models.map((m) => (
                  <TableCell key={m.key} className="text-xs">
                    <div className="flex flex-wrap gap-1">
                      {m.tool_call && <CapBadge>tools</CapBadge>}
                      {m.reasoning && <CapBadge>reasoning</CapBadge>}
                      {m.vision && <CapBadge>vision</CapBadge>}
                      {m.open_weights && <CapBadge tone="amber">open</CapBadge>}
                      {!m.tool_call && !m.reasoning && !m.vision && !m.open_weights && (
                        <span className="text-[var(--color-fg-sub)]">—</span>
                      )}
                    </div>
                  </TableCell>
                ))}
              </TableRow>
              {dimensions.map((dimension) => (
                <TableRow key={dimension}>
                  <TableCell className="text-xs text-[var(--color-fg-sub)]">
                    {DIMENSION_LABEL[dimension]}
                  </TableCell>
                  {models.map((m) => {
                    const score = Math.max(
                      0,
                      ...m.benchmarks.filter((b) => b.dimension === dimension).map((b) => b.score),
                    );
                    const hasScore = m.benchmarks.some((b) => b.dimension === dimension);
                    return (
                      <TableCell key={m.key} className="text-xs">
                        <ComparedValue highlight={hasScore && score === bestScore(dimension) && score > 0}>
                          {hasScore ? score.toFixed(1) : "no data"}
                        </ComparedValue>
                      </TableCell>
                    );
                  })}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/** Highlights the best value in a comparison row (cheapest cost, largest
 * context, highest benchmark score) so a scan across columns lands on the
 * winner without reading every cell. */
function ComparedValue({ children, highlight }: { children: React.ReactNode; highlight: boolean }) {
  return (
    <span className={cn(highlight ? "font-medium text-[var(--color-ember)]" : "text-[var(--color-fg)]")}>
      {children}
    </span>
  );
}

function CapBadge({ children, tone }: { children: React.ReactNode; tone?: "amber" }) {
  return (
    <span
      className={cn(
        "rounded border px-1.5 py-0 text-xs lowercase tracking-tight",
        tone === "amber"
          ? "border-[color-mix(in_srgb,var(--color-amber)_50%,transparent)] text-[var(--color-amber)]"
          : "border-[color-mix(in_srgb,var(--color-ember)_40%,transparent)] text-[var(--color-ember)]",
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
      className={cn("inline-block text-[var(--color-ember)]", className)}
      style={{ animation: "terminal-cursor-blink 600ms steps(1, end) infinite" }}
    >
      ▍
    </span>
  );
}

/** Inline ASCII-frame empty state. The shared component lands in change-4. */
function EmptyState({ title, hint }: { title: string; hint: string }) {
  return (
    <div className="mx-auto max-w-md py-12 text-center text-xs text-[var(--color-fg-sub)]">
      <pre className="select-none text-[var(--color-fg-sub)] opacity-60" aria-hidden>
{`┌────────────────────────────┐
│                            │
│         ${title.padEnd(20, " ").slice(0, 20)}│
│                            │
└────────────────────────────┘`}
      </pre>
      <p className="mt-3 text-[var(--color-fg-sub)]">{hint}</p>
    </div>
  );
}
