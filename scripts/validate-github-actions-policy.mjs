#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), "..");

const allowedWorkflows = new Map([
  ["deploy.yml", ["kubectl set image", "kubectl rollout status", "/readyz", "/healthz"]],
  ["docs.yml", ["actions/upload-pages-artifact@", "actions/deploy-pages@"]],
]);

const prohibitedCommands = [
  [/(?:^|\s)cargo\s+(?:test|check|clippy|fmt|bench|audit|mutants)\b/m, "Cargo development check"],
  [/(?:^|\s)(?:pnpm|npm|npx|bun)\s+(?:(?:run|exec)\s+)?(?:test|lint|typecheck|vitest|playwright|cucumber|coverage|audit)\b/m, "JavaScript development check"],
  [/\b(?:osv-scanner|grype|cargo-mutants|codecov|chromatic)\b/m, "non-deployment analysis service"],
  [/\bvale\s+--config\b/m, "prose lint"],
];

const pagesPublisherPattern = /actions\/(?:upload-pages-artifact|deploy-pages)@/;

export function validateGitHubActionsPolicy(root = defaultRoot) {
  const resolvedRoot = resolve(root);
  const workflowsDir = join(resolvedRoot, ".github", "workflows");
  const failures = [];
  const pagesPublishers = [];

  if (!existsSync(workflowsDir)) {
    failures.push(`workflow directory is missing: ${workflowsDir}`);
    return { failures, pagesPublishers };
  }

  const workflowFiles = readdirSync(workflowsDir)
    .filter((name) => name.endsWith(".yml") || name.endsWith(".yaml"))
    .sort();

  for (const name of workflowFiles) {
    const source = readFileSync(join(workflowsDir, name), "utf8");
    if (pagesPublisherPattern.test(source)) pagesPublishers.push(name);

    if (!allowedWorkflows.has(name)) {
      failures.push(`non-deployment workflow is prohibited: .github/workflows/${name}`);
      continue;
    }

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

  if (pagesPublishers.length !== 1) {
    failures.push(`exactly one GitHub Pages publisher is required; found ${pagesPublishers.length}: ${pagesPublishers.join(", ") || "none"}`);
  }

  const dockerfile = join(resolvedRoot, "Dockerfile");
  if (existsSync(dockerfile)) {
    const source = readFileSync(dockerfile, "utf8");
    for (const [pattern, label] of prohibitedCommands) {
      if (pattern.test(source)) failures.push(`Dockerfile contains prohibited ${label}`);
    }
  }

  return { failures, pagesPublishers };
}

function main() {
  const root = resolve(process.argv[2] ?? defaultRoot);
  const { failures, pagesPublishers } = validateGitHubActionsPolicy(root);

  if (failures.length > 0) {
    console.error(`GitHub Actions policy validation failed:\n- ${failures.join("\n- ")}`);
    process.exit(1);
  }

  console.log(`GitHub Actions policy validation passed (deployment workflows only; Pages publisher: ${pagesPublishers[0]}).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
