#!/usr/bin/env node

import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const SETTINGS_UI_ROOT = path.join(
  REPO_ROOT,
  "frontend/src/features/settings/ui",
);
const MAX_PRODUCTION_TSX_LINES = 600;
const EXPECTED_KEYS = [
  "llm",
  "provider",
  "vision",
  "context_management",
  "context_strategy",
  "rag",
  "knowledge_bases",
  "memory",
  "models",
  "file_processing",
  "unstructured",
  "mistral_ocr",
  "kreuzberg",
  "resilience",
  "server",
  "persistence",
  "sandbox",
  "intent_classifier",
  "security",
  "governance",
  "sycophancy",
  "agent_config",
  "skill_config",
  "native_tools",
  "skill_evolution",
  "acp",
  "llm_failover",
  "prompt_caching",
  "user_settings",
];

async function listFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = await Promise.all(
    entries.map((entry) => {
      const target = path.join(directory, entry.name);
      return entry.isDirectory() ? listFiles(target) : [target];
    }),
  );
  return files.flat();
}

function orderedKeys(source, startMarker, endMarker, pattern) {
  const start = source.indexOf(startMarker);
  const end = source.indexOf(endMarker, start + startMarker.length);
  if (start === -1 || end === -1) {
    throw new Error(`Missing inventory markers: ${startMarker} → ${endMarker}`);
  }
  return [...source.slice(start, end).matchAll(pattern)].map((match) => match[1]);
}

function assertSameKeys(label, actual) {
  if (JSON.stringify(actual) !== JSON.stringify(EXPECTED_KEYS)) {
    throw new Error(
      `${label} changed. Expected ${EXPECTED_KEYS.join(",")}; received ${actual.join(",")}`,
    );
  }
}

const files = await listFiles(SETTINGS_UI_ROOT);
const productionTsx = files.filter(
  (file) => file.endsWith(".tsx") && !/\.(test|stories)\.tsx$/.test(file),
);
const sizes = [];
for (const file of productionTsx) {
  const source = await readFile(file, "utf8");
  const lines = source.split("\n").length;
  sizes.push({ file: path.relative(REPO_ROOT, file), lines });
  if (lines > MAX_PRODUCTION_TSX_LINES) {
    throw new Error(
      `${path.relative(REPO_ROOT, file)} has ${lines} lines; limit is ${MAX_PRODUCTION_TSX_LINES}`,
    );
  }
}

const navigationSource = await readFile(
  path.join(SETTINGS_UI_ROOT, "settings-navigation.tsx"),
  "utf8",
);
assertSameKeys(
  "Navigation inventory",
  orderedKeys(
    navigationSource,
    "export const NAV_ITEMS",
    "export const CATEGORIES",
    /key: "([^"]+)"/g,
  ),
);

const registrySource = await readFile(
  path.join(SETTINGS_UI_ROOT, "settings-panel-registry.tsx"),
  "utf8",
);
assertSameKeys(
  "Panel registry",
  orderedKeys(
    registrySource,
    "export const PANEL_MAP",
    "};",
    /^  ([a-z_]+):/gm,
  ),
);

const pageSource = await readFile(
  path.join(SETTINGS_UI_ROOT, "settings-page.tsx"),
  "utf8",
);
if (!pageSource.includes('useState<string>("provider")')) {
  throw new Error("Settings default active panel is no longer provider");
}
if (/function (?:Provider|Resilience|Memory|UserSettings)Panel/.test(pageSource)) {
  throw new Error("Domain panel implementations returned to settings-page.tsx");
}

const featureEntry = await readFile(
  path.join(REPO_ROOT, "frontend/src/features/settings/index.ts"),
  "utf8",
);
const expectedFeatureEntry = 'export { SettingsPage } from "./ui/settings-page";';
if (featureEntry.trim() !== expectedFeatureEntry) {
  throw new Error("The settings feature public SettingsPage contract changed");
}

const largest = sizes.sort((a, b) => b.lines - a.lines)[0];
console.log(
  `Settings UI decomposition gate passed (${productionTsx.length} modules; largest ${largest.file} at ${largest.lines}/${MAX_PRODUCTION_TSX_LINES} lines; 29 navigation and panel keys preserved).`,
);
