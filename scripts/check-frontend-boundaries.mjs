#!/usr/bin/env node

import { readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const src = resolve(root, "frontend/src");
const fixtureIndex = process.argv.indexOf("--fixture-dir");
const scanRoot = fixtureIndex >= 0 ? resolve(root, process.argv[fixtureIndex + 1]) : src;
const infrastructureFetchExceptions = new Set([
  // Shared transport bootstrap, not feature/domain I/O.
  "frontend/src/entities/sync.ts",
  // Loads the PGlite WASM/data assets used to initialize client persistence.
  "frontend/src/platform/pglite/assets.ts",
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
  if (path.includes("/services/") || path.includes("/api/")) return "service";
  if (path.includes("/stores/") || /-store\.(?:ts|tsx)$/.test(name)) return "store";
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

function importsSingleLayer(specifier, layer) {
  if (!specifier.startsWith("@/") && !specifier.startsWith(".")) return false;
  const name = specifier.split("/").at(-1) ?? "";
  if (layer === "services") {
    return /(?:^|\/)services(?:\/|$)/.test(specifier)
      || /(?:^|\/)api(?:\/|$)/.test(specifier)
      || /-api$/.test(name);
  }
  if (layer === "stores") {
    return /(?:^|\/)stores(?:\/|$)/.test(specifier) || /-store$/.test(name);
  }
  if (layer === "hooks") {
    return /(?:^|\/)hooks(?:\/|$)/.test(specifier) || /^use(?:-|[A-Z])/.test(name);
  }
  return new RegExp(`(?:^|/)${layer}(?:/|$)`).test(specifier);
}

function importsLayer(modules, layers) {
  const candidates = layers.split("|");
  return modules.some((specifier) =>
    candidates.some((layer) => importsSingleLayer(specifier, layer)),
  );
}

function importTargetPath(sourcePath, specifier) {
  if (specifier.startsWith("@/")) {
    return resolve(scanRoot, specifier.slice(2));
  }
  if (specifier.startsWith(".")) {
    return resolve(dirname(sourcePath), specifier);
  }
  return null;
}

function architectureLayer(targetPath) {
  if (!targetPath) return null;
  const relativePath = relative(scanRoot, targetPath).replaceAll("\\", "/");
  if (relativePath === "app" || relativePath.startsWith("app/")) return "app";
  if (relativePath === "features" || relativePath.startsWith("features/")) return "feature";
  if (relativePath === "shared" || relativePath.startsWith("shared/")) return "shared";
  if (relativePath === "platform" || relativePath.startsWith("platform/")) return "platform";
  return null;
}

function featureName(targetPath) {
  if (!targetPath) return null;
  const parts = relative(scanRoot, targetPath).replaceAll("\\", "/").split("/");
  return parts[0] === "features" ? parts[1] ?? null : null;
}

function isPublicFeatureEntry(specifier, targetPath) {
  if (!targetPath || !specifier.startsWith("@/features/")) return false;
  const relativeParts = relative(scanRoot, targetPath).replaceAll("\\", "/").split("/");
  const namedRootEntry = relativeParts.length === 3
    && [".ts", ".tsx"].some((suffix) => {
      try {
        return statSync(`${targetPath}${suffix}`).isFile();
      } catch {
        return false;
      }
    });
  const directoryIndex = !/\.(?:ts|tsx)$/.test(targetPath)
    && (
      relativeParts.length === 2
      || (relativeParts.length === 3 && ["api", "model"].includes(relativeParts[2]))
    )
    && ["/index.ts", "/index.tsx"].some((suffix) => {
      try {
        return statSync(`${targetPath}${suffix}`).isFile();
      } catch {
        return false;
      }
    });
  return namedRootEntry || directoryIndex;
}

function isUpwardImport(sourceLayer, targetLayer) {
  if (sourceLayer === "platform") {
    return targetLayer === "feature" || targetLayer === "app";
  }
  if (sourceLayer === "shared") {
    return targetLayer === "feature" || targetLayer === "app";
  }
  return sourceLayer === "feature" && targetLayer === "app";
}

const violations = [];
for (const path of walk(scanRoot).filter(isProductionTs)) {
  const content = readFileSync(path, "utf8");
  const file = repoPath(path);
  const layer = layerFor(path);
  const modules = importedModules(content);
  const sourceArchitectureLayer = architectureLayer(path);

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

  for (const specifier of modules) {
    const targetPath = importTargetPath(path, specifier);
    const targetArchitectureLayer = architectureLayer(targetPath);
    if (
      sourceArchitectureLayer &&
      targetArchitectureLayer &&
      isUpwardImport(sourceArchitectureLayer, targetArchitectureLayer)
    ) {
      violations.push(
        `${file}|${sourceArchitectureLayer}-upward-${targetArchitectureLayer}-import`,
      );
    }

    const sourceFeature = featureName(path);
    const targetFeature = featureName(targetPath);
    if (
      sourceFeature &&
      targetFeature &&
      sourceFeature !== targetFeature &&
      !isPublicFeatureEntry(specifier, targetPath)
    ) {
      violations.push(`${file}|feature-cross-implementation-import`);
    }
  }
}

const actual = [...new Set(violations)].sort();
if (process.argv.includes("--print")) {
  process.stdout.write(`${actual.join("\n")}\n`);
  process.exit(0);
}

if (actual.length) {
  console.error("Frontend boundary violations (move I/O and mutations to the owning layer):");
  for (const item of actual) console.error(`  + ${item}`);
  process.exit(1);
}

console.log("Frontend boundary gate passed (0 production violations).");
