import { type FC, useMemo } from "react";
import { AlertTriangle, DollarSign, TrendingUp, Wallet } from "lucide-react";
import { useGraphStore } from "@/platform/entities";
import type { EntityType } from "@/platform/entities";
import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";
import { EmptyFrame } from "@/shared/ui/configuration/empty-frame";
import { LoadingCursor } from "@/shared/ui/configuration/loading-cursor";
import { cn } from "@/lib/utils";
import type { RuntimeBudgetAlertEntity, RuntimeRunEntity } from "@/entities/types";

// CH-07 cost-dashboard.
//
// Data source: the shared entity graph, fed by the runtime's SSE stream
// (`runtime.run` on `RunDoneWithUsage` for per-run spend, `runtime.budget_alert`
// on `NormalizedEvent::BudgetAlert` for threshold crossings — both wired by
// CH-06 cost-budgets-backend). Like the rest of the Runtime Console, this is
// in-memory/session-scoped: there is no persisted run-history endpoint yet,
// so spend shown here covers runs completed since the page was last loaded,
// not all-time totals. A durable roll-up is a reasonable follow-up.

function useEntities<T>(type: EntityType): T[] {
  const entityMap = useGraphStore((state) => state.entities[type]);
  return useMemo(() => Object.values(entityMap ?? {}) as T[], [entityMap]);
}

const chartConfig: ChartConfig = {
  cost: { label: "spend (USD)", color: "var(--color-ember)" },
};

export const CostDashboardPage: FC = () => {
  const runs = useEntities<RuntimeRunEntity>("RuntimeRun");
  const alerts = useEntities<RuntimeBudgetAlertEntity>("RuntimeBudgetAlert");

  const pricedRuns = useMemo(
    () =>
      runs
        .filter((r): r is RuntimeRunEntity & { cost_usd_estimate: number } =>
          typeof r.cost_usd_estimate === "number",
        )
        .sort((a, b) => new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime()),
    [runs],
  );

  const totalSpend = useMemo(
    () => pricedRuns.reduce((sum, r) => sum + r.cost_usd_estimate, 0),
    [pricedRuns],
  );

  const chartData = useMemo(
    () =>
      pricedRuns.map((r, i) => ({
        index: i + 1,
        run: r.title ?? r.id.slice(0, 8),
        cost: r.cost_usd_estimate,
      })),
    [pricedRuns],
  );

  const byModel = useMemo(() => {
    const totals = new Map<string, number>();
    for (const r of pricedRuns) {
      const key = r.model ?? "unknown";
      totals.set(key, (totals.get(key) ?? 0) + r.cost_usd_estimate);
    }
    return Array.from(totals.entries())
      .map(([model, spend]) => ({ model, spend }))
      .sort((a, b) => b.spend - a.spend);
  }, [pricedRuns]);

  const activeAlerts = useMemo(
    () => [...alerts].sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()),
    [alerts],
  );
  const exceededCount = activeAlerts.filter((a) => a.exceeded).length;
  const warningCount = activeAlerts.length - exceededCount;

  const loading = runs.length === 0 && alerts.length === 0;

  return (
    <div className="flex flex-1 flex-col overflow-hidden font-mono text-[13px] text-[var(--color-fg)]">
      {/* Header */}
      <div className="flex items-center justify-between bg-[var(--color-surface)] px-6 py-4">
        <div>
          <h2 className="text-[20px] font-medium tracking-tight text-[var(--color-fg)]">cost</h2>
          <p className="text-xs text-[var(--color-fg-sub)]">
            <span data-testid="cost-run-count">{pricedRuns.length}</span> priced runs this session
            {loading && <LoadingCursor className="ml-2" />}
          </p>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Stat tiles */}
        <section
          className="grid grid-cols-2 md:grid-cols-4"
          aria-label="Spend summary"
        >
          <StatTile label="total spend" value={`$${totalSpend.toFixed(4)}`} icon={DollarSign} />
          <StatTile label="priced runs" value={pricedRuns.length} icon={TrendingUp} />
          <StatTile
            label="budget warnings"
            value={warningCount}
            icon={Wallet}
            tone={warningCount > 0 ? "amber" : undefined}
          />
          <StatTile
            label="budgets exceeded"
            value={exceededCount}
            icon={AlertTriangle}
            tone={exceededCount > 0 ? "red" : undefined}
          />
        </section>

        {/* Spend-over-time chart */}
        <section className=" px-6 py-4" aria-label="Spend over time">
          <SectionLabel>spend over time</SectionLabel>
          {chartData.length === 0 ? (
            <EmptyFrame
              title="no priced runs yet"
              hint="spend appears here once a run completes with cost tracking enabled"
            />
          ) : (
            <ChartContainer config={chartConfig} className="aspect-auto h-56 w-full">
              <BarChart data={chartData} accessibilityLayer>
                <CartesianGrid vertical={false} stroke="var(--color-border)" />
                <XAxis
                  dataKey="index"
                  tickLine={false}
                  axisLine={false}
                  tickMargin={8}
                  stroke="var(--color-fg-sub)"
                  fontSize={11}
                />
                <YAxis
                  tickLine={false}
                  axisLine={false}
                  tickMargin={8}
                  width={56}
                  stroke="var(--color-fg-sub)"
                  fontSize={11}
                  tickFormatter={(v: number) => `$${v.toFixed(3)}`}
                />
                <ChartTooltip content={<ChartTooltipContent labelKey="run" nameKey="cost" />} />
                <Bar dataKey="cost" fill="var(--color-ember)" radius={2} />
              </BarChart>
            </ChartContainer>
          )}
        </section>

        {/* Per-model breakdown */}
        <section className=" px-6 py-4" aria-label="Spend by model">
          <SectionLabel>spend by model</SectionLabel>
          {byModel.length === 0 ? (
            <EmptyFrame title="no model breakdown yet" hint="grouped once priced runs are recorded" />
          ) : (
            <div className="flex flex-col gap-1.5" data-testid="cost-by-model">
              {byModel.map((row) => (
                <div
                  key={row.model}
                  className="flex items-center justify-between gap-3 bg-[var(--color-surface)] px-4 py-2.5"
                >
                  <span className="truncate text-[13px] text-[var(--color-fg)]">{row.model}</span>
                  <span className="shrink-0 font-medium text-[var(--color-ember)]">${row.spend.toFixed(4)}</span>
                </div>
              ))}
            </div>
          )}
        </section>

        {/* Budget alerts */}
        <section className="px-6 py-4" aria-label="Budget alerts">
          <SectionLabel>budget alerts</SectionLabel>
          {activeAlerts.length === 0 ? (
            <EmptyFrame
              title="no budget alerts"
              hint="alerts appear here when a run crosses a configured warning or hard limit"
            />
          ) : (
            <div className="flex flex-col gap-1.5" data-testid="budget-alerts-list">
              {activeAlerts.map((alert) => (
                <BudgetAlertRow key={alert.id} alert={alert} />
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  );
};

function SectionLabel({ children }: { children: React.ReactNode }) {
  return (
    <div className="mb-3 flex items-center gap-2">
      <span className="text-xs font-medium uppercase tracking-widest text-[var(--color-fg-sub)]">
        {children}
      </span>
      <span className="flex-1" aria-hidden />
    </div>
  );
}

function StatTile({
  label,
  value,
  icon: Icon,
  tone,
}: {
  label: string;
  value: string | number;
  icon: typeof DollarSign;
  tone?: "amber" | "red";
}) {
  return (
    <div className="px-4 py-3">
      <div className="mb-2 flex items-center justify-between">
        <span className="text-xs text-[var(--color-fg-sub)]">{label}</span>
        <Icon
          size={15}
          className={cn(
            tone === "amber" && "text-[var(--color-amber)]",
            tone === "red" && "text-[var(--color-red)]",
            !tone && "text-[var(--color-ember)]",
          )}
        />
      </div>
      <div
        className={cn(
          "text-2xl font-semibold",
          tone === "amber" && "text-[var(--color-amber)]",
          tone === "red" && "text-[var(--color-red)]",
          !tone && "text-[var(--color-fg)]",
        )}
      >
        {value}
      </div>
    </div>
  );
}

function BudgetAlertRow({ alert }: { alert: RuntimeBudgetAlertEntity }) {
  return (
    <div
      className={cn(
        "flex items-center gap-3 border px-4 py-2.5",
        alert.exceeded
          ? "border-[color-mix(in_srgb,var(--color-red)_50%,transparent)] bg-[color-mix(in_srgb,var(--color-red)_6%,transparent)]"
          : "border-[color-mix(in_srgb,var(--color-amber)_50%,transparent)] bg-[color-mix(in_srgb,var(--color-amber)_6%,transparent)]",
      )}
      data-testid={`budget-alert-${alert.id}`}
    >
      <AlertTriangle
        size={14}
        className={cn("shrink-0", alert.exceeded ? "text-[var(--color-red)]" : "text-[var(--color-amber)]")}
      />
      <div className="min-w-0 flex-1">
        <p className="truncate text-[13px] text-[var(--color-fg)]">
          {alert.scope}/{alert.scope_id}
        </p>
        <p className="text-xs text-[var(--color-fg-sub)]">
          ${alert.spent_usd.toFixed(4)} of ${alert.limit_usd.toFixed(4)} spent
        </p>
      </div>
      <span
        className={cn(
          "shrink-0 rounded border px-1.5 py-0 text-xs lowercase",
          alert.exceeded
            ? "border-[color-mix(in_srgb,var(--color-red)_50%,transparent)] text-[var(--color-red)]"
            : "border-[color-mix(in_srgb,var(--color-amber)_50%,transparent)] text-[var(--color-amber)]",
        )}
      >
        {alert.exceeded ? "exceeded" : "warning"}
      </span>
    </div>
  );
}
