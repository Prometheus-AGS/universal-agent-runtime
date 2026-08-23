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
const documentationWorkflowMarkers = [
  "npm --prefix website ci",
  "npm --prefix website run build",
  "cargo doc --locked --no-deps --workspace --features server-full",
  "npm --prefix sdks/typescript ci",
  "npm --prefix sdks/typescript run docs",
  "node scripts/stage-documentation-references.mjs",
  "steps.deployment.outputs.page_url",
  "docs/architecture/intro",
  "docs/api/rust/",
  "docs/api/typescript/",
  "curl --fail",
];
const documentationWorkflowProhibitions = [
  [/\b(?:pnpm|yarn|bun)\b/, "alternate package-manager invocation"],
  [/\bbuild:docs\b/, "nonexistent TypeScript documentation command"],
  [/\bplaceholder\b/i, "placeholder reference fallback"],
  [/\|\|\s*true/, "fail-open command fallback"],
];

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
    if (name === "docs.yml") {
      for (const marker of documentationWorkflowMarkers) {
        if (!source.includes(marker)) failures.push(`${name} is missing documentation deployment marker: ${marker}`);
      }
      for (const [pattern, label] of documentationWorkflowProhibitions) {
        if (pattern.test(source)) failures.push(`${name} contains prohibited ${label}`);
      }
    }
  }

  for (const name of allowedWorkflows.keys()) {
    if (!workflowFiles.includes(name)) failures.push(`required deployment workflow is missing: ${name}`);
  }

  if (pagesPublishers.length !== 1) {
    failures.push(`exactly one GitHub Pages publisher is required; found ${pagesPublishers.length}: ${pagesPublishers.join(", ") || "none"}`);
  }

  const websitePackagePath = join(resolvedRoot, "website", "package.json");
  const websiteLockPath = join(resolvedRoot, "website", "package-lock.json");
  if (!existsSync(websitePackagePath) || !existsSync(websiteLockPath)) {
    failures.push("website npm package contract is incomplete");
  } else {
    try {
      const websitePackage = JSON.parse(readFileSync(websitePackagePath, "utf8"));
      const buildCommand = websitePackage.scripts?.build ?? "";
      if (!buildCommand.includes("npm run copy:adr") || /\b(?:pnpm|yarn|bun)\b/.test(buildCommand)) {
        failures.push("website build must use the npm-managed copy and Docusaurus command chain");
      }
    } catch {
      failures.push("website/package.json is not valid JSON");
    }
  }

  const sdkPackagePath = join(resolvedRoot, "sdks", "typescript", "package.json");
  const sdkLockPath = join(resolvedRoot, "sdks", "typescript", "package-lock.json");
  if (!existsSync(sdkPackagePath) || !existsSync(sdkLockPath)) {
    failures.push("TypeScript SDK npm documentation contract is incomplete");
  } else {
    try {
      const sdkPackage = JSON.parse(readFileSync(sdkPackagePath, "utf8"));
      if (sdkPackage.scripts?.docs !== "typedoc") {
        failures.push("TypeScript SDK docs command must invoke the pinned TypeDoc contract");
      }
    } catch {
      failures.push("sdks/typescript/package.json is not valid JSON");
    }
  }

  if (!existsSync(join(resolvedRoot, "scripts", "stage-documentation-references.mjs"))) {
    failures.push("documentation reference staging command is missing");
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
