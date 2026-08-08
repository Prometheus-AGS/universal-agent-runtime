export interface ChartSeries {
  name: string;
  values: number[];
}

export interface ChartModel {
  kind: "bar" | "line";
  title: string;
  labels: string[];
  series: ChartSeries[];
  xLabel?: string;
  yLabel?: string;
}

function exactKeys(value: Record<string, unknown>, allowed: readonly string[]): boolean {
  return Object.keys(value).every((key) => allowed.includes(key));
}

export function parseChartModel(source: string): ChartModel | null {
  let raw: unknown;
  try { raw = JSON.parse(source) as unknown; } catch { return null; }
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
  const value = raw as Record<string, unknown>;
  if (!exactKeys(value, ["kind", "title", "labels", "series", "xLabel", "yLabel"])) return null;
  if ((value.kind !== "bar" && value.kind !== "line") || typeof value.title !== "string") return null;
  if (!Array.isArray(value.labels) || value.labels.length === 0 || value.labels.length > 200 || !value.labels.every((label) => typeof label === "string" && label.length <= 120)) return null;
  if (!Array.isArray(value.series) || value.series.length === 0 || value.series.length > 12) return null;
  const series: ChartSeries[] = [];
  for (const rawSeries of value.series) {
    if (!rawSeries || typeof rawSeries !== "object" || Array.isArray(rawSeries)) return null;
    const item = rawSeries as Record<string, unknown>;
    if (!exactKeys(item, ["name", "values"]) || typeof item.name !== "string" || item.name.length > 120 || !Array.isArray(item.values) || item.values.length !== value.labels.length || !item.values.every((point) => typeof point === "number" && Number.isFinite(point))) return null;
    series.push({ name: item.name, values: item.values as number[] });
  }
  if (value.xLabel !== undefined && typeof value.xLabel !== "string") return null;
  if (value.yLabel !== undefined && typeof value.yLabel !== "string") return null;
  return { kind: value.kind, title: value.title, labels: value.labels as string[], series, xLabel: value.xLabel as string | undefined, yLabel: value.yLabel as string | undefined };
}
