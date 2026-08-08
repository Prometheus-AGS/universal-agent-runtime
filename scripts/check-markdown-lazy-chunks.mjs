import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const ENGINE_ENTRIES = [
  "src/shared/markdown/blocks/vendor-shiki.ts",
  "src/shared/markdown/blocks/vendor-mermaid.ts",
];

const requireRecord = (value, label) => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object record`);
  }
  return value;
};

const requireStringArray = (value, label) => {
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string")) {
    throw new Error(`${label} must be an array of strings`);
  }
  return value;
};

/** Inspect Vite's manifest and emitted chunk-module graph. */
export const inspectMarkdownLazyChunks = (manifest, engineGraph) => {
  requireRecord(manifest, "Vite manifest");
  const entry = requireRecord(manifest["index.html"], "Vite manifest index.html entry");
  if (!entry) throw new Error("Vite manifest has no index.html entry");
  if (typeof entry.file !== "string") throw new Error("Vite manifest index.html entry has no emitted file");
  const chunks = requireRecord(engineGraph?.chunks, "Engine graph chunks");

  const manifestPending = ["index.html"];
  const manifestVisited = new Set();
  const manifestStaticFiles = new Set();
  while (manifestPending.length > 0) {
    const key = manifestPending.shift();
    if (!key || manifestVisited.has(key)) continue;
    manifestVisited.add(key);
    const record = requireRecord(manifest[key], `Vite manifest static record ${key}`);
    if (typeof record.file !== "string") {
      throw new Error(`Vite manifest static record ${key} has no emitted file`);
    }
    if (record.file.endsWith(".js")) manifestStaticFiles.add(record.file);
    manifestPending.push(...requireStringArray(record.imports ?? [], `Vite manifest imports for ${key}`));
  }

  for (const file of manifestStaticFiles) {
    const chunk = requireRecord(chunks[file], `Engine graph static record ${file}`);
    requireStringArray(chunk.imports, `Engine graph imports for ${file}`);
    requireStringArray(chunk.engineModules, `Engine graph engineModules for ${file}`);
  }

  const pending = [entry.file];
  const staticReachable = new Set();

  while (pending.length > 0) {
    const key = pending.shift();
    if (!key || staticReachable.has(key)) continue;
    const chunk = requireRecord(chunks[key], `Engine graph static record ${key}`);
    staticReachable.add(key);
    pending.push(...requireStringArray(chunk.imports, `Engine graph imports for ${key}`));
  }

  const missingGraphFiles = [...manifestStaticFiles].filter((file) => !staticReachable.has(file));
  const extraGraphFiles = [...staticReachable].filter((file) => !manifestStaticFiles.has(file));
  if (missingGraphFiles.length || extraGraphFiles.length) {
    throw new Error(`Engine graph static closure mismatch: missing [${missingGraphFiles.join(", ")}], extra [${extraGraphFiles.join(", ")}]`);
  }

  const forbiddenStatic = [...staticReachable].flatMap((file) =>
    chunks[file].engineModules.map((moduleId) => ({ file, moduleId })),
  );
  const dynamicPending = [{ key: "index.html", crossedDynamicBoundary: false }];
  const dynamicVisited = new Set();
  const dynamicReachable = new Set();
  while (dynamicPending.length > 0) {
    const current = dynamicPending.shift();
    if (!current) continue;
    const visitKey = `${current.crossedDynamicBoundary ? "dynamic" : "static"}:${current.key}`;
    if (dynamicVisited.has(visitKey)) continue;
    dynamicVisited.add(visitKey);
    if (current.crossedDynamicBoundary) dynamicReachable.add(current.key);
    const record = manifest[current.key];
    if (!record) continue;
    for (const key of record.imports ?? []) {
      dynamicPending.push({ key, crossedDynamicBoundary: current.crossedDynamicBoundary });
    }
    for (const key of record.dynamicImports ?? []) {
      dynamicPending.push({ key, crossedDynamicBoundary: true });
    }
  }
  const missingDynamic = ENGINE_ENTRIES.filter((key) => !dynamicReachable.has(key));
  const invalidNames = ENGINE_ENTRIES.filter((key) => {
    const expected = key.includes("shiki") ? "vendor-shiki-" : "vendor-mermaid-";
    const file = manifest[key]?.file;
    return !file?.startsWith(`assets/${expected}`)
      || !(chunks[file]?.engineModules?.length > 0);
  });
  const absoluteModuleIds = Object.entries(chunks).flatMap(([file, chunk]) =>
    (chunk.engineModules ?? [])
      .filter((moduleId) => /^(?:[A-Za-z]:[\\/]|\/)/u.test(moduleId))
      .map((moduleId) => ({ file, moduleId })),
  );

  return {
    staticReachable: [...staticReachable],
    forbiddenStatic,
    missingDynamic,
    invalidNames,
    absoluteModuleIds,
  };
};

const main = async () => {
  const manifestPath = process.argv[2] ?? "static/.vite/manifest.json";
  const engineGraphPath = process.argv[3] ?? "static/.vite/markdown-engine-graph.json";
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  const engineGraph = JSON.parse(await readFile(engineGraphPath, "utf8"));
  const result = inspectMarkdownLazyChunks(manifest, engineGraph);

  console.log(JSON.stringify(result, null, 2));
  if (
    result.forbiddenStatic.length
    || result.missingDynamic.length
    || result.invalidNames.length
    || result.absoluteModuleIds.length
  ) {
    process.exitCode = 1;
  }
};

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
