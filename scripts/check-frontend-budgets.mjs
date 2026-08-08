import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { gzipSync } from "node:zlib";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { inspectMarkdownLazyChunks } from "./check-markdown-lazy-chunks.mjs";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const INVENTORY_PATH = path.join(REPO_ROOT, "frontend", "performance-budgets.json");
const PGLITE_SOURCE_PATTERN = /(?:@electric-sql[+/]pglite.*\.(?:wasm|data)|platform\/pglite\/pglite-seed-v\d+\.tar\.gz)$/u;
const PGLITE_CHUNK_NAME = "vendor-pglite";
const REQUIRED_PGLITE_ASSETS = [
  { kind: "pglite-data", source: /\/pglite\.data$/u, emitted: /\.data$/u },
  { kind: "pglite-wasm", source: /\/pglite\.wasm$/u, emitted: /\.wasm$/u },
  { kind: "schema-seed", source: /\/pglite-seed-v\d+\.tar\.gz$/u, emitted: /\.gz$/u },
];

const isObject = (value) => value !== null && typeof value === "object" && !Array.isArray(value);

const readJson = async (filePath, label) => {
  let source;
  try {
    source = await readFile(filePath, "utf8");
  } catch (error) {
    throw new Error(`${label} is unavailable at ${filePath}: ${error.message}`);
  }

  try {
    return JSON.parse(source);
  } catch (error) {
    throw new Error(`${label} is malformed at ${filePath}: ${error.message}`);
  }
};

const requireStringArray = (value, label) => {
  if (value === undefined) return [];
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string")) {
    throw new Error(`${label} must be an array of strings`);
  }
  return [...value].sort();
};

const resolveBuildFile = (buildRoot, emittedFile) => {
  if (typeof emittedFile !== "string" || emittedFile.length === 0) {
    throw new Error("Manifest record has no emitted file");
  }
  if (path.isAbsolute(emittedFile)) {
    throw new Error(`Emitted file escapes the build root: ${emittedFile}`);
  }
  const resolved = path.resolve(buildRoot, emittedFile);
  const relative = path.relative(buildRoot, resolved);
  if (relative === "" || relative.startsWith(`..${path.sep}`) || relative === ".." || path.isAbsolute(relative)) {
    throw new Error(`Emitted file escapes the build root: ${emittedFile}`);
  }
  return resolved;
};

const validateManifest = (manifest) => {
  if (!isObject(manifest)) throw new Error("Vite manifest must be an object");
  const entries = Object.entries(manifest).filter(([, record]) => isObject(record) && record.isEntry === true);
  if (entries.length !== 1) {
    throw new Error(`Vite manifest must contain exactly one application entry; found ${entries.length}`);
  }
  return entries[0];
};

const collectStaticClosure = (manifest, entryKey) => {
  const pending = [entryKey];
  const visited = new Set();

  while (pending.length > 0) {
    const key = pending.shift();
    if (!key || visited.has(key)) continue;
    const record = manifest[key];
    if (!isObject(record)) throw new Error(`Manifest import does not resolve: ${key}`);
    if (typeof record.file !== "string" || !record.file.endsWith(".js")) {
      throw new Error(`Static JavaScript closure contains a non-JavaScript file for ${key}: ${String(record.file)}`);
    }
    visited.add(key);
    pending.push(...requireStringArray(record.imports, `Manifest imports for ${key}`));
  }

  return [...visited].sort();
};

const inspectPgliteAssets = async (manifest, buildRoot) => {
  const candidates = Object.entries(manifest)
    .filter(([key, record]) => isObject(record)
      && typeof record.src === "string"
      && PGLITE_SOURCE_PATTERN.test(record.src.replaceAll("\\", "/")))
    .map(([key, record]) => ({
      key,
      file: record.file,
      source: record.src.replaceAll("\\", "/"),
    }));

  for (const required of REQUIRED_PGLITE_ASSETS) {
    const matches = candidates.filter(({ source }) => required.source.test(source));
    if (matches.length !== 1) {
      throw new Error(`PGlite asset ownership requires exactly one ${required.kind}; found ${matches.length}`);
    }
    if (typeof matches[0].file !== "string" || !required.emitted.test(matches[0].file)) {
      throw new Error(`PGlite ${required.kind} has an unexpected emitted file: ${String(matches[0].file)}`);
    }
  }

  const assets = candidates
    .sort((left, right) => left.file.localeCompare(right.file));

  for (const asset of assets) {
    const filePath = resolveBuildFile(buildRoot, asset.file);
    const fileStat = await stat(filePath).catch(() => null);
    if (!fileStat?.isFile()) throw new Error(`PGlite non-JavaScript asset is missing: ${asset.file}`);
    asset.rawBytes = fileStat.size;
    asset.kind = REQUIRED_PGLITE_ASSETS.find(({ source }) => source.test(asset.source))?.kind
      ?? "pglite-runtime-support";
  }
  return assets;
};

export const inspectInitialJavaScriptBudget = async ({
  manifest,
  engineGraph,
  buildRoot,
  limitBytes,
}) => {
  if (!Number.isSafeInteger(limitBytes) || limitBytes < 0) {
    throw new Error(`Bundle limit must be a non-negative safe integer; received ${String(limitBytes)}`);
  }
  const [entryKey, entryRecord] = validateManifest(manifest);
  const closureKeys = collectStaticClosure(manifest, entryKey);
  const seenFiles = new Set();
  const files = [];

  for (const key of closureKeys) {
    const emittedFile = manifest[key].file;
    if (seenFiles.has(emittedFile)) continue;
    seenFiles.add(emittedFile);
    const filePath = resolveBuildFile(buildRoot, emittedFile);
    let source;
    try {
      source = await readFile(filePath);
    } catch (error) {
      throw new Error(`Static JavaScript file is missing: ${emittedFile}: ${error.message}`);
    }
    files.push({
      key,
      file: emittedFile,
      rawBytes: source.byteLength,
      gzipBytes: gzipSync(source, { level: 9, mtime: 0 }).byteLength,
      accounting: manifest[key].name === PGLITE_CHUNK_NAME ? "excluded-pglite" : "counted",
    });
  }
  files.sort((left, right) => left.file.localeCompare(right.file));

  const excludedPgliteJavaScript = files.filter((file) => file.accounting === "excluded-pglite");
  if (excludedPgliteJavaScript.length !== 1) {
    throw new Error(
      `Static JavaScript closure must contain exactly one named ${PGLITE_CHUNK_NAME} chunk; found ${excludedPgliteJavaScript.length}`,
    );
  }

  const lazyEngines = inspectMarkdownLazyChunks(manifest, engineGraph);
  const engineFailures = [
    ...lazyEngines.forbiddenStatic.map(({ file, moduleId }) => `eager engine module ${moduleId} in ${file}`),
    ...lazyEngines.missingDynamic.map((key) => `missing dynamic engine entry ${key}`),
    ...lazyEngines.invalidNames.map((key) => `invalid dynamic engine entry ${key}`),
    ...lazyEngines.absoluteModuleIds.map(({ file, moduleId }) => `absolute engine module id ${moduleId} in ${file}`),
  ];
  if (engineFailures.length > 0) {
    throw new Error(`Lazy engine integrity failed: ${engineFailures.join("; ")}`);
  }

  const excludedPgliteAssets = await inspectPgliteAssets(manifest, buildRoot);
  const countedFiles = files.filter((file) => file.accounting === "counted");
  const rawBytes = countedFiles.reduce((total, file) => total + file.rawBytes, 0);
  const gzipBytes = countedFiles.reduce((total, file) => total + file.gzipBytes, 0);
  const result = {
    schemaVersion: 1,
    entry: { key: entryKey, file: entryRecord.file },
    files,
    rawBytes,
    gzipBytes,
    limitBytes,
    exclusions: {
      pgliteJavaScript: excludedPgliteJavaScript,
      pgliteNonJavaScriptAssets: excludedPgliteAssets,
      lazyMarkdownEngines: ["mermaid", "shiki"],
    },
    verdict: gzipBytes <= limitBytes ? "pass" : "fail",
  };
  if (result.verdict === "fail") {
    throw Object.assign(
      new Error(`Initial JavaScript closure is ${gzipBytes} gzip bytes; limit is ${limitBytes}`),
      { budgetResult: result },
    );
  }
  return result;
};

const parseArguments = (arguments_) => {
  const options = {};
  for (let index = 0; index < arguments_.length; index += 2) {
    const flag = arguments_[index];
    const value = arguments_[index + 1];
    if (!flag?.startsWith("--") || value === undefined) {
      throw new Error(`Expected --flag value arguments; received ${arguments_.slice(index).join(" ")}`);
    }
    options[flag.slice(2)] = value;
  }
  return options;
};

const main = async () => {
  const options = parseArguments(process.argv.slice(2));
  const buildRoot = path.resolve(options.root ?? path.join(REPO_ROOT, "static"));
  const manifestPath = path.resolve(options.manifest ?? path.join(buildRoot, ".vite", "manifest.json"));
  const engineGraphPath = path.resolve(options["engine-graph"] ?? path.join(buildRoot, ".vite", "markdown-engine-graph.json"));

  try {
    const inventory = await readJson(INVENTORY_PATH, "Frontend performance budget inventory");
    const configuredLimit = inventory?.bundle?.initialJavaScriptGzipBytes;
    const limitBytes = options.limit === undefined ? configuredLimit : Number(options.limit);
    const manifest = await readJson(manifestPath, "Vite manifest");
    const engineGraph = await readJson(engineGraphPath, "Markdown engine graph");
    const result = await inspectInitialJavaScriptBudget({ manifest, engineGraph, buildRoot, limitBytes });
    if (options.output) {
      const outputPath = path.resolve(options.output);
      await mkdir(path.dirname(outputPath), { recursive: true });
      await writeFile(outputPath, `${JSON.stringify(result, null, 2)}\n`);
    }
    console.log(`Frontend bundle budget passed: ${result.gzipBytes}/${result.limitBytes} gzip bytes across ${result.files.length} files`);
    console.log(JSON.stringify(result, null, 2));
  } catch (error) {
    const failureResult = error.budgetResult
      ? { ...error.budgetResult, error: { message: error.message } }
      : { schemaVersion: 1, verdict: "fail", error: { message: error.message } };
    if (options.output) {
      const outputPath = path.resolve(options.output);
      await mkdir(path.dirname(outputPath), { recursive: true });
      await writeFile(outputPath, `${JSON.stringify(failureResult, null, 2)}\n`);
    }
    if (error.budgetResult) console.error(JSON.stringify(error.budgetResult, null, 2));
    throw error;
  }
};

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main().catch((error) => {
    console.error(`Frontend bundle budget failed: ${error.message}`);
    process.exitCode = 1;
  });
}
