export function toneLabel(tone: "ok" | "warn") {
  return tone === "ok" ? "Healthy" : "Needs attention";
}
