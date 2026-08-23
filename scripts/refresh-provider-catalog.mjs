import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--locked", "--format-version", "1"], {
    cwd: root,
    encoding: "utf8",
    // `cargo metadata` output has grown past Node's default 1 MB stdout cap
    // (ENOBUFS); allow up to 128 MB so the full JSON is captured.
    maxBuffer: 128 * 1024 * 1024,
  }),
);
const liter = metadata.packages.find((pkg) => pkg.name === "liter-llm");
if (!liter) throw new Error("liter-llm is absent from Cargo metadata");

const literProviders = JSON.parse(
  readFileSync(resolve(dirname(liter.manifest_path), "schemas/providers.json"), "utf8"),
);
const literCatalog = JSON.parse(
  readFileSync(resolve(dirname(liter.manifest_path), "schemas/catalog.json"), "utf8"),
);
const modelsDev = literCatalog.providers;
if (!modelsDev || typeof modelsDev !== "object" || Array.isArray(modelsDev)) {
  throw new Error("liter-llm catalog is missing its provider map");
}

function perMillion(value) {
  return typeof value === "number" ? value * 1_000_000 : undefined;
}

function modelCost(model) {
  if (model.cost) return model.cost;
  if (!model.pricing) return null;

  const input = perMillion(model.pricing.input_cost_per_token);
  const output = perMillion(model.pricing.output_cost_per_token);
  const cacheRead = perMillion(model.pricing.cache_read_input_token_cost);
  const cacheWrite = perMillion(model.pricing.cache_creation_input_token_cost);
  if (input === undefined && output === undefined) return null;

  return {
    input: input ?? 0,
    output: output ?? 0,
    ...(cacheRead === undefined ? {} : { cache_read: cacheRead }),
    ...(cacheWrite === undefined ? {} : { cache_write: cacheWrite }),
  };
}

function modelRecord(model) {
  const capabilities = model.capabilities ?? {};
  return {
    id: model.id ?? "",
    name: model.name ?? "",
    family: model.family ?? null,
    capabilities: {
      tool_call: model.tool_call ?? capabilities.function_calling ?? false,
      reasoning: model.reasoning ?? capabilities.reasoning ?? false,
      structured_output:
        model.structured_output ?? capabilities.structured_output ?? false,
      attachment:
        model.attachment ?? capabilities.attachment ?? capabilities.vision ?? false,
      temperature: model.temperature ?? false,
      streaming: true,
    },
    modalities: model.modalities ?? { input: ["text"], output: ["text"] },
    limits: {
      context_window: model.limit?.context ?? 0,
      max_output: model.limit?.output ?? 0,
    },
    cost: modelCost(model),
    release_date: model.release_date ?? null,
    open_weights: model.open_weights ?? capabilities.open_weights ?? false,
  };
}

function providerModels(provider) {
  return Object.values(provider?.models ?? {}).map(modelRecord);
}

const catalog = [];
const literNames = new Set(literProviders.providers.map((provider) => provider.name));
for (const provider of literProviders.providers) {
  const name = provider.name;
  const source =
    modelsDev[name] ??
    modelsDev[name.replaceAll("_", "-")] ??
    modelsDev[name.replaceAll("-", "_")];
  const auth =
    provider.auth ??
    (source?.env?.[0] ? { type: "bearer", env_var: source.env[0] } : null);
  catalog.push({
    id: name,
    display_name: provider.display_name ?? name,
    base_url: provider.base_url ?? null,
    auth,
    endpoints: provider.endpoints ?? [],
    model_prefixes: provider.model_prefixes ?? [],
    param_mappings: provider.param_mappings ?? null,
    models: providerModels(source),
  });
}

for (const [id, provider] of Object.entries(modelsDev)) {
  if (
    literNames.has(id) ||
    literNames.has(id.replaceAll("-", "_")) ||
    literNames.has(id.replaceAll("_", "-"))
  ) {
    continue;
  }
  catalog.push({
    id,
    display_name: provider.name ?? id,
    base_url: provider.api ?? null,
    auth: provider.env?.[0] ? { type: "bearer", env_var: provider.env[0] } : null,
    endpoints: ["chat"],
    model_prefixes: [],
    param_mappings: null,
    models: providerModels(provider),
    source: "models_dev_only",
  });
}

const output = JSON.stringify(catalog);
const destination = resolve(root, "catalog/provider_catalog.json");
writeFileSync(destination, output);
const digest = createHash("sha256").update(output).digest("hex");
console.log(`Wrote ${catalog.length} providers to ${destination}`);
console.log(`SHA-256: ${digest}`);
console.log("Review the catalog diff and update catalog/SNAPSHOT.md before committing.");
