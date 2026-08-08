import { Bar, BarChart, Legend, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { ChartModel } from "./chart-model";
import { ChunkSurface } from "./chunk-surface";

const SERIES_COLORS = ["var(--color-ember)", "var(--color-cyan)", "var(--color-success)", "var(--color-warning)"] as const;

export function ChartChunkView({ model }: { model: ChartModel }) {
  const data = model.labels.map((label, index) => Object.fromEntries([
    ["label", label],
    ...model.series.map((series) => [series.name, series.values[index]]),
  ]));
  const series = model.series.map((item, index) => model.kind === "bar"
    ? <Bar key={item.name} dataKey={item.name} fill={SERIES_COLORS[index % SERIES_COLORS.length]} radius={[4, 4, 0, 0]} isAnimationActive="auto" />
    : <Line key={item.name} dataKey={item.name} stroke={SERIES_COLORS[index % SERIES_COLORS.length]} strokeWidth={2} dot={false} isAnimationActive="auto" />);
  const axes = <><XAxis dataKey="label" aria-label={model.xLabel ?? "Categories"} /><YAxis width="auto" aria-label={model.yLabel ?? "Values"} /><Tooltip /><Legend /></>;
  return (
    <ChunkSurface label={`Chart: ${model.title}`}>
      <h3 className="mb-3 font-display text-sm font-semibold">{model.title}</h3>
      <div className="h-64 min-w-0" data-chart-model="application-owned">
        <ResponsiveContainer width="100%" height="100%">
          {model.kind === "bar"
            ? <BarChart data={data} accessibilityLayer>{axes}{series}</BarChart>
            : <LineChart data={data} accessibilityLayer>{axes}{series}</LineChart>}
        </ResponsiveContainer>
      </div>
    </ChunkSurface>
  );
}
