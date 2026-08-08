#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const fixtureIndex = process.argv.indexOf("--fixture-dir");
const usingFixture = fixtureIndex >= 0;
if (usingFixture && !process.argv[fixtureIndex + 1]) {
  console.error("Platform adapter gate requires a path after --fixture-dir.");
  process.exit(2);
}
const frontendRoot = resolve(root, "frontend");
const fixtureRoot = usingFixture ? resolve(root, process.argv[fixtureIndex + 1]) : null;
const scanRoots = usingFixture
  ? [fixtureRoot]
  : [resolve(frontendRoot, "src"), resolve(frontendRoot, "e2e")];
const sourceBase = fixtureRoot ?? frontendRoot;

function walk(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = resolve(dir, entry);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

function sourcePath(path) {
  return relative(sourceBase, path).replaceAll("\\", "/");
}

function importedModules(content) {
  const modules = [];
  const staticImport = /\b(?:import|export)\s*(?:[^"'`;]*?\bfrom\s*)?["']([^"']+)["']/g;
  const dynamicImport = /\bimport\s*\(\s*["']([^"']+)["']\s*\)/g;
  for (const match of content.matchAll(staticImport)) modules.push(match[1]);
  for (const match of content.matchAll(dynamicImport)) modules.push(match[1]);
  return modules;
}

const violations = [];
const sourceFiles = [];
for (const scanRoot of scanRoots) {
  if (!scanRoot || !existsSync(scanRoot)) {
    violations.push(`${scanRoot ?? "(missing)"}|missing-scan-root`);
    continue;
  }
  sourceFiles.push(...walk(scanRoot).filter((file) => /\.(ts|tsx)$/.test(file)));
}
for (const path of sourceFiles) {
  const file = sourcePath(path);
  const content = readFileSync(path, "utf8");
  const modules = importedModules(content);

  if (
    modules.some((module) =>
      module === "@prometheus-ags/prometheus-entity-management" ||
      module.startsWith("@prometheus-ags/prometheus-entity-management/")) &&
    file !== (usingFixture ? "platform/entities/index.ts" : "src/platform/entities/index.ts")
  ) {
    violations.push(`${file}|direct-entity-package-import`);
  }
  if (modules.some((module) => /^(?:@\/|.*\/)protocols\/agui(?:-|\/|$)/.test(module))) {
    violations.push(`${file}|retired-agui-import`);
  }
  if (modules.some((module) => /^(?:@\/|.*\/)lib\/(?:db(?:\/|$)|pglite(?:-|\/|$))/.test(module))) {
    violations.push(`${file}|retired-pglite-import`);
  }
  if (/(?:^|\/)protocols\/agui(?:-|\/)/.test(file)) {
    violations.push(`${file}|retired-agui-file`);
  }
  if (/(?:^|\/)lib\/(?:db(?:\.ts|\/|$)|pglite(?:-|\/))/.test(file)) {
    violations.push(`${file}|retired-pglite-file`);
  }
  if (
    /(?:^|\/)platform\//.test(file) &&
    (file.endsWith(".tsx") || modules.some((module) => /^(?:react|react-dom)(?:\/|$)/.test(module)))
  ) {
    violations.push(`${file}|platform-react-boundary`);
  }
}

if (!usingFixture || process.argv.includes("--check-required")) {
  const requiredRoot = usingFixture ? fixtureRoot : resolve(frontendRoot, "src");
  for (const required of [
    "platform/agui/agui-adapter.ts",
    "platform/agui/agui-schema.ts",
    "platform/entities/index.ts",
    "platform/pglite/assets.ts",
    "platform/pglite/client.ts",
  ]) {
    if (!requiredRoot || !existsSync(resolve(requiredRoot, required))) {
      violations.push(`${required}|missing-platform-adapter`);
    }
  }
}

const actual = [...new Set(violations)].sort();
if (process.argv.includes("--print")) {
  process.stdout.write(`${actual.join("\n")}\n`);
  process.exit(actual.length ? 1 : 0);
}

if (actual.length) {
  console.error("Platform adapter boundary violations:");
  for (const item of actual) console.error(`  + ${item}`);
  process.exit(1);
}

console.log("Platform adapter gate passed (sole PEM facade; retired paths absent; platform is React-free).");
