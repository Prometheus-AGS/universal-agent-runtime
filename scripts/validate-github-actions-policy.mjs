#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const defaultRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const root = resolve(process.argv[2] ?? defaultRoot);
const workflowsDir = join(root, ".github", "workflows");

const allowedWorkflows = new Map([
  ["deploy.yml", ["kubectl set image", "kubectl rollout status", "/readyz", "/healthz"]],
  ["docs.yml", ["actions/deploy-pages@"]],
  ["typescript-sdk-docs.yml", ["actions/deploy-pages@"]],
]);

const prohibitedCommands = [
  [/(?:^|\s)cargo\s+(?:test|check|clippy|fmt|bench|audit|mutants)\b/m, "Cargo development check"],
  [/(?:^|\s)(?:pnpm|npm|npx|bun)\s+(?:(?:run|exec)\s+)?(?:test|lint|typecheck|vitest|playwright|cucumber|coverage|audit)\b/m, "JavaScript development check"],
  [/\b(?:osv-scanner|grype|cargo-mutants|codecov|chromatic)\b/m, "non-deployment analysis service"],
  [/\bvale\s+--config\b/m, "prose lint"],
];

const failures = [];
if (!existsSync(workflowsDir)) {
  failures.push(`workflow directory is missing: ${workflowsDir}`);
} else {
  const workflowFiles = readdirSync(workflowsDir)
    .filter((name) => name.endsWith(".yml") || name.endsWith(".yaml"))
    .sort();

  for (const name of workflowFiles) {
    if (!allowedWorkflows.has(name)) {
      failures.push(`non-deployment workflow is prohibited: .github/workflows/${name}`);
      continue;
    }

    const source = readFileSync(join(workflowsDir, name), "utf8");
    for (const [pattern, label] of prohibitedCommands) {
      if (pattern.test(source)) failures.push(`${name} contains prohibited ${label}`);
    }
    for (const marker of allowedWorkflows.get(name)) {
      if (!source.includes(marker)) failures.push(`${name} is missing deployment marker: ${marker}`);
    }
  }

  for (const name of allowedWorkflows.keys()) {
    if (!workflowFiles.includes(name)) failures.push(`required deployment workflow is missing: ${name}`);
  }
}

const dockerfile = join(root, "Dockerfile");
if (existsSync(dockerfile)) {
  const source = readFileSync(dockerfile, "utf8");
  for (const [pattern, label] of prohibitedCommands) {
    if (pattern.test(source)) failures.push(`Dockerfile contains prohibited ${label}`);
  }
}

if (failures.length > 0) {
  console.error(`GitHub Actions policy validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}

console.log("GitHub Actions policy validation passed (deployment workflows only: deploy.yml, docs.yml, typescript-sdk-docs.yml).\n");
