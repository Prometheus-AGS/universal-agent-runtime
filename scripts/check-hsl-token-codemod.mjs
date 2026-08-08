#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const fixtureIndex = process.argv.indexOf("--fixture-file");
const fixtureFile = fixtureIndex >= 0 ? process.argv[fixtureIndex + 1] : null;
if (fixtureIndex >= 0 && !fixtureFile) {
  console.error("HSL token codemod gate requires a path after --fixture-file.");
  process.exit(2);
}
const migratedFiles = fixtureFile ? [fixtureFile] : [
  "frontend/src/index.css",
  "frontend/src/components/assistant-ui/enhanced-thread.tsx",
  "frontend/src/shared/ui/configuration/loading-cursor.tsx",
  "frontend/src/shared/ui/configuration/error-bar.tsx",
  "frontend/src/shared/ui/configuration/empty-frame.tsx",
  "frontend/src/shared/ui/uar-logo.tsx",
  "frontend/src/features/compiler/ui/compiler-page.tsx",
  "frontend/src/features/cost/ui/cost-dashboard-page.tsx",
  "frontend/src/features/memory/ui/memory-page.tsx",
  "frontend/src/features/models/ui/models-page.tsx",
  "frontend/src/features/skills/ui/skills-page.tsx",
];
const legacyCall = /hsla?\s*\(\s*var\s*\(/gi;
const violations = [];
const semanticDefinitions = new Set(
  ["frontend/src/shared/theme/tokens.css", "frontend/src/index.css"].flatMap((file) =>
    [...readFileSync(resolve(root, file), "utf8").matchAll(/(--color-[a-z0-9-]+)\s*:/gi)].map(
      (match) => match[1],
    ),
  ),
);

function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(dir, entry.name);
    return entry.isDirectory() ? walk(path) : [path];
  });
}

for (const file of migratedFiles) {
  const content = readFileSync(resolve(root, file), "utf8");
  const count = [...content.matchAll(legacyCall)].length;
  if (count) violations.push(`${file}: ${count} legacy HSL-channel call site(s)`);
  for (const match of content.matchAll(/var\((--color-[a-z0-9-]+)\)/gi)) {
    if (!semanticDefinitions.has(match[1])) {
      violations.push(`${file}: undefined semantic token ${match[1]}`);
    }
  }
}

const deferredCount = fixtureFile
  ? null
  : (existsSync(resolve(root, "frontend/src/admin/pages"))
    ? walk(resolve(root, "frontend/src/admin/pages"))
      .filter((file) => file.endsWith(".tsx"))
      .reduce(
        (count, file) => count + [...readFileSync(file, "utf8").matchAll(legacyCall)].length,
        0,
      )
    : 0);

if (violations.length) {
  console.error("HSL token codemod scope violations:");
  for (const violation of violations) console.error(`  + ${violation}`);
  process.exit(1);
}

if (fixtureFile) {
  console.log("HSL token codemod fixture passed.");
} else {
  console.log(`HSL token codemod gate passed (migrated file set clean; ${deferredCount} admin-page call sites currently deferred).`);
}
