import { type FC, useMemo, useState } from "react";
import { RefreshCw, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { AdminEmptyInline, AdminError, AdminListSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import { useModelsBrowse } from "@/hooks/use-models-browse";
import type { CatalogModel } from "@/types";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface FlatModel {
  key: string;           // "openai/gpt-4o"
  provider_id: string;
  provider_name: string;
  model_id: string;
  model: CatalogModel;
  configured: boolean;
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export const ModelsPage: FC = () => {
  const { response, loading, error, load } = useModelsBrowse();
  const [selectedProvider, setSelectedProvider] = useState<string>("all");
  const [query, setQuery] = useState("");
  const [capabilities, setCapabilities] = useState<{ tools: boolean; reasoning: boolean; vision: boolean }>({
    tools: false, reasoning: false, vision: false,
  });

  // Flatten into a filterable list
  const allModels: FlatModel[] = useMemo(() => {
    return Object.entries(response).flatMap(([providerId, providerData]) =>
      Object.entries(providerData.models).map(([modelId, model]) => ({
        key: `${providerId}/${modelId}`,
        provider_id: providerId,
        provider_name: providerData.display_name,
        model_id: modelId,
        model,
        configured: providerData.configured,
      })),
    );
  }, [response]);

  const providers = useMemo(() => {
    const seen = new Set<string>();
    return allModels
      .filter((m) => !seen.has(m.provider_id) && seen.add(m.provider_id))
      .map((m) => ({ id: m.provider_id, name: m.provider_name, configured: m.configured }));
  }, [allModels]);

  const filtered = useMemo(() => {
    let list = allModels;
    if (selectedProvider !== "all") list = list.filter((m) => m.provider_id === selectedProvider);
    if (capabilities.tools) list = list.filter((m) => m.model.tool_call);
    if (capabilities.reasoning) list = list.filter((m) => m.model.reasoning);
    if (capabilities.vision) list = list.filter((m) => m.model.modalities.input.includes("image"));
    if (query) {
      const q = query.toLowerCase();
      list = list.filter((m) => m.key.toLowerCase().includes(q) || m.model.name.toLowerCase().includes(q));
    }
    return list;
  }, [allModels, selectedProvider, capabilities, query]);

  const toggleCap = (cap: keyof typeof capabilities) =>
    setCapabilities((prev) => ({ ...prev, [cap]: !prev[cap] }));

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">Models</h2>
          <p className="font-mono text-xs text-muted-foreground">
            {filtered.length} of {allModels.length} models · {providers.length} providers
          </p>
        </div>
        <Button type="button" variant="outline" size="sm" onClick={() => void load()} className="gap-1.5">
          <RefreshCw size={13} className={cn(loading && "animate-spin")} />Refresh
        </Button>
      </div>

      {/* Filters */}
      <div className="flex flex-col gap-2 border-b border-border bg-background px-6 py-3">
        {/* Search */}
        <div className="relative">
          <Search size={13} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search models…"
            className="h-8 pl-8 font-mono text-xs"
          />
        </div>

        {/* Provider filter pills */}
        <div className="flex flex-wrap gap-1">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => setSelectedProvider("all")}
            className={cn(
              "h-7 rounded-full px-2.5 py-0.5 font-mono text-xs",
              selectedProvider === "all" ? "bg-primary/20 text-primary hover:bg-primary/25" : "text-muted-foreground hover:text-foreground",
            )}
          >
            All
          </Button>
          {providers.map((p) => (
            <Button
              key={p.id}
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => setSelectedProvider(p.id)}
              className={cn(
                "h-7 rounded-full px-2.5 py-0.5 font-mono text-xs",
                selectedProvider === p.id ? "bg-primary/20 text-primary hover:bg-primary/25" : "text-muted-foreground hover:text-foreground",
                !p.configured && "opacity-50",
              )}
            >
              {p.name}
            </Button>
          ))}
        </div>

        {/* Capability filters */}
        <div className="flex gap-1">
          {(["tools", "reasoning", "vision"] as const).map((cap) => (
            <Button
              key={cap}
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => toggleCap(cap)}
              className={cn(
                "h-7 rounded border px-2 py-0.5 font-mono text-xs capitalize",
                capabilities[cap]
                  ? "border-primary/40 bg-primary/10 text-primary hover:bg-primary/15"
                  : "border-border text-muted-foreground hover:border-primary/30 hover:text-foreground",
              )}
            >
              {cap}
            </Button>
          ))}
        </div>
      </div>

      {/* Model list */}
      <div className="flex-1 overflow-y-auto p-6">
        {loading && allModels.length === 0 && <AdminListSkeleton rows={8} />}
        <AdminError error={error} />
        {!loading && filtered.length === 0 && (
          <AdminEmptyInline>
            {allModels.length === 0
              ? "No models available. Configure a provider first."
              : "No models match the current filters."}
          </AdminEmptyInline>
        )}
        <div className="flex flex-col gap-1.5">
          {filtered.map((m) => (
            <ModelRow key={m.key} flat={m} />
          ))}
        </div>
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Model row
// ---------------------------------------------------------------------------

function ModelRow({ flat }: { flat: FlatModel }) {
  const { model, key, provider_name, configured } = flat;
  const ctx = model.limit.context;
  const inputCost = model.cost.input;
  const outputCost = model.cost.output;

  return (
    <div className={cn(
      "flex items-center gap-3 rounded-lg border px-4 py-3",
      configured ? "border-border bg-card" : "border-border/40 bg-muted/20 opacity-60",
    )}>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="font-mono text-[13px] font-medium text-foreground">{model.name || flat.model_id}</p>
          {!configured && <span className="font-mono text-xs text-muted-foreground">(not configured)</span>}
        </div>
        <p className="font-mono text-xs text-muted-foreground">{key}</p>
        <p className="font-mono text-xs text-muted-foreground">{provider_name}</p>
      </div>

      <div className="flex shrink-0 flex-wrap items-center gap-1.5">
        {ctx > 0 && (
          <span className="font-mono text-xs text-muted-foreground">{(ctx / 1000).toFixed(0)}k context</span>
        )}
        {inputCost > 0 && (
          <span className="font-mono text-xs text-muted-foreground">
            ${inputCost.toFixed(2)} in / ${outputCost.toFixed(2)} out per 1M tokens
          </span>
        )}
        {model.tool_call && <Badge variant="outline" className="h-4 px-1 font-mono text-xs">tools</Badge>}
        {model.reasoning && <Badge variant="outline" className="h-4 px-1 font-mono text-xs">reasoning</Badge>}
        {model.modalities.input.includes("image") && <Badge variant="outline" className="h-4 px-1 font-mono text-xs">vision</Badge>}
        {model.open_weights && <Badge variant="outline" className="h-4 px-1 font-mono text-xs">open</Badge>}
      </div>
    </div>
  );
}
