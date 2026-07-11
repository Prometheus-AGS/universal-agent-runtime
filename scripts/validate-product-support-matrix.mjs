import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const matrix = JSON.parse(readFileSync(resolve(root, "docs/product-support-matrix.json"), "utf8"));
const cargo = readFileSync(resolve(root, "Cargo.toml"), "utf8");
const featureBlock = cargo.match(/\[features\]([\s\S]*?)\n\n\[dependencies\]/)?.[1] ?? "";
const cargoFeatures = new Set(
  [...featureBlock.matchAll(/^([a-z][a-z0-9-]*)\s*=/gm)].map((match) => match[1]),
);
const matrixFeatures = new Set(matrix.features.map((feature) => feature.id));

for (const feature of cargoFeatures) {
  if (feature === "default") continue;
  if (!matrixFeatures.has(feature)) throw new Error(`Cargo feature missing from matrix: ${feature}`);
}
for (const feature of matrixFeatures) {
  if (!cargoFeatures.has(feature)) throw new Error(`Matrix feature missing from Cargo.toml: ${feature}`);
}

const collections = ["features", "bundles", "providers", "persistence", "routing", "tools", "platforms"];
for (const collection of collections) {
  if (!Array.isArray(matrix[collection]) || matrix[collection].length === 0) {
    throw new Error(`Matrix collection is empty: ${collection}`);
  }
  for (const row of matrix[collection]) {
    if (row.status === "stable" && (!row.gate || row.gate === "none")) {
      throw new Error(`Stable ${collection} row lacks executable gate: ${row.id ?? row.level ?? row.mode ?? row.deployment}`);
    }
  }
}

const today = new Date(`${matrix.verified_at}T00:00:00Z`);
for (const provider of matrix.providers) {
  const verified = new Date(`${provider.verified_at}T00:00:00Z`);
  const ageDays = (today.getTime() - verified.getTime()) / 86_400_000;
  if (!Number.isFinite(ageDays) || ageDays < 0 || ageDays > 180) {
    throw new Error(`Provider verification date is invalid or stale: ${provider.id}`);
  }
}

for (const required of ["bossfang", "flint-gate", "flint-realtime-fabric", "flint-forge", "flint-platform-agent"]) {
  if (!matrix.integrations.some((integration) => integration.id === required)) {
    throw new Error(`Required integration boundary missing: ${required}`);
  }
}

console.log(`Product support matrix valid (${matrix.features.length} features, ${matrix.providers.length} provider tiers).`);
