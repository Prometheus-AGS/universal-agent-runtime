#!/usr/bin/env node

import { readFileSync, readdirSync, statSync } from "node:fs";
import { relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const src = resolve(root, "frontend/src");
const allowlistPath = resolve(root, "scripts/frontend-boundary-allowlist.txt");
const infrastructureFetchExceptions = new Set([
  // Shared transport bootstrap, not feature/domain I/O.
  "frontend/src/entities/sync.ts",
  // Loads the PGlite WASM/data assets used to initialize client persistence.
  "frontend/src/lib/pglite-assets.ts",
]);

function walk(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = resolve(dir, entry);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

function repoPath(path) {
  return relative(root, path).replaceAll("\\", "/");
}

function isProductionTs(path) {
  return /\.(ts|tsx)$/.test(path) && !/\.(test|spec)\.(ts|tsx)$/.test(path) && !path.includes("/__tests__/");
}

function layerFor(path) {
  const name = path.split("/").at(-1) ?? "";
  if (path.includes("/services/")) return "service";
  if (path.includes("/stores/")) return "store";
  if (path.includes("/hooks/") || /^use(?:-|[A-Z])[^.]*\.(ts|tsx)$/.test(name)) return "hook";
  if (path.endsWith(".tsx")) return "component";
  return "module";
}

function importedModules(content) {
  const modules = [];
  const staticImport = /\b(?:import|export)\s+(?:[^"'`;]*?\sfrom\s*)?["']([^"']+)["']/g;
  const dynamicImport = /\bimport\s*\(\s*["']([^"']+)["']\s*\)/g;
  for (const match of content.matchAll(staticImport)) modules.push(match[1]);
  for (const match of content.matchAll(dynamicImport)) modules.push(match[1]);
  return modules;
}

function importsLayer(modules, layer) {
  return modules.some((specifier) =>
    new RegExp(`(?:^|/)(?:${layer})(?:/|$)`).test(specifier),
  );
}

const violations = [];
for (const path of walk(src).filter(isProductionTs)) {
  const content = readFileSync(path, "utf8");
  const file = repoPath(path);
  const layer = layerFor(path);
  const modules = importedModules(content);

  if (
    layer !== "service" &&
    !infrastructureFetchExceptions.has(file) &&
    /(?:\bfetch|(?:window|globalThis)\.fetch)\s*\(/.test(content)
  ) {
    violations.push(`${file}|${layer}-direct-fetch`);
  }
  if (layer === "component" && importsLayer(modules, "services")) {
    violations.push(`${file}|component-service-import`);
  }
  if (layer === "component" && importsLayer(modules, "stores")) {
    violations.push(`${file}|component-store-import`);
  }
  if (layer === "hook" && importsLayer(modules, "services")) {
    violations.push(`${file}|hook-service-import`);
  }
  if (layer === "store" && importsLayer(modules, "components|hooks")) {
    violations.push(`${file}|store-upward-import`);
  }
  if (layer === "service" && importsLayer(modules, "components|hooks|stores")) {
    violations.push(`${file}|service-upward-import`);
  }
}

const actual = [...new Set(violations)].sort();
if (process.argv.includes("--print")) {
  process.stdout.write(`${actual.join("\n")}\n`);
  process.exit(0);
}

const expected = readFileSync(allowlistPath, "utf8")
  .split(/\r?\n/)
  .map((line) => line.trim())
  .filter((line) => line && !line.startsWith("#"))
  .sort();

const unexpected = actual.filter((item) => !expected.includes(item));
const stale = expected.filter((item) => !actual.includes(item));

if (unexpected.length || stale.length) {
  if (unexpected.length) {
    console.error("New frontend boundary violations (fix; do not extend the allowlist):");
    for (const item of unexpected) console.error(`  + ${item}`);
  }
  if (stale.length) {
    console.error("Resolved allowlist entries (remove them from the allowlist):");
    for (const item of stale) console.error(`  - ${item}`);
  }
  process.exit(1);
}

console.log(`Frontend boundary gate passed (${actual.length} tracked legacy violations, 0 new).`);
