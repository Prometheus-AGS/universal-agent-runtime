import { type FC, useEffect, useMemo, useState } from "react";
import { RefreshCw, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useModels } from "@/entities/hooks/use-models";
import { loadModelsIntoGraph } from "@/entities/fetchers/models";

// ---------------------------------------------------------------------------
// Page-local model row shape
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

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export const ModelsPage: FC = () => {
  // Reads come from the entity graph. Hydration is the page's responsibility.
  const view = useModels();
  const rawItems = view.items as ReadonlyArray<Record<string, unknown>>;

  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedProvider, setSelectedProvider] = useState<string>("all");
  const [query, setQuery] = useState("");
  const [capabilities, setCapabilities] = useState<{ tools: boolean; reasoning: boolean; vision: boolean }>({
    tools: false, reasoning: false, vision: false,
  });

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const load = async () => {
    setRefreshing(true);
    setError(null);
    try {
      await loadModelsIntoGraph();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setRefreshing(false);
    }
  };

  const loading = refreshing && rawItems.length === 0;

  // Adapt the loose graph rows into typed model rows for rendering.
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

  return (
    <div className="flex flex-1 flex-col overflow-hidden font-mono text-[13px] text-[hsl(var(--terminal-fg))]">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[hsl(var(--terminal-line-strong))] bg-[hsl(var(--terminal-surface))] px-6 py-4">
        <div>
          <h2 className="text-[20px] font-medium tracking-tight text-[hsl(var(--terminal-fg))]">models</h2>
          <p className="text-xs text-[hsl(var(--terminal-fg-dim))]">
            <span data-testid="models-count">{filtered.length}</span> of {allModels.length} models · {providers.length} providers
            {refreshing && <TerminalCursor className="ml-2" />}
          </p>
        </div>
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

      {/* Error bar */}
      {error && (
        <div className="border-b border-[hsl(var(--signal-red))] bg-[hsl(var(--signal-red)/0.08)] px-6 py-2 text-xs text-[hsl(var(--signal-red))]">
          <span className="mr-2 font-semibold">ERR-MODELS</span>{error}
        </div>
      )}

      {/* Filters */}
      <div className="flex flex-col gap-2 border-b border-[hsl(var(--terminal-line))] bg-[hsl(var(--terminal-bg))] px-6 py-3">
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

      {/* Model list */}
      <div className="flex-1 overflow-y-auto p-6">
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
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Pieces
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
