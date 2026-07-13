#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const workflow = readFileSync(new URL("../.github/workflows/release.yml", import.meta.url), "utf8");
const failures = [];
const trackedPaths = execFileSync("git", ["ls-files", "-z"], { encoding: "utf8" }).split("\0").filter(Boolean);
for (const path of trackedPaths) {
  if (/[<>:"|?*\\]/.test(path)) failures.push(`Windows-incompatible tracked path: ${path}`);
}
const required = [
  "node-version: 22",
  "version: 10.33.0",
  "cargo check --locked --lib --features minimal",
  "cargo check --locked --lib --features server-full",
  "cargo check --locked --lib --features desktop-full",
  "cargo clippy --lib --features server-full --no-deps",
  "cargo build --locked --release --bin universal-agent-runtime --features server-full",
  "node scripts/validate-static-bundle.mjs static",
  "cp -R static",
  "Copy-Item static",
  "ubuntu-24.04-arm",
  "macos-15-intel",
  "runner: macos-15",
  "windows-latest",
  "http://127.0.0.1:1906/readyz",
  "scripts/package-offline-source.sh",
];

for (const value of required) {
  if (!workflow.includes(value)) failures.push(`missing release contract: ${value}`);
}

const prohibited = [
  ["node-version: '18'", "Node 18"],
  ["setup-bun", "Bun setup"],
  ["bun install", "Bun install"],
  ["--all-features", "all-features release build"],
  ["test-config.yaml", "test-only configuration"],
  ["redis/redis-stack", "unsupported Redis service"],
  ["static/main.js", "retired static asset assumption"],
];

for (const [value, label] of prohibited) {
  if (workflow.includes(value)) failures.push(`prohibited ${label}`);
}

const platformRows = workflow.match(/- name: (?:linux|macos|windows)-/g) ?? [];
if (platformRows.length !== 5) failures.push(`expected 5 certified platform rows, found ${platformRows.length}`);

const frontendInstalls = workflow.match(/pnpm -C frontend install --frozen-lockfile/g) ?? [];
if (frontendInstalls.length !== 2) {
  failures.push(`expected frontend lockfile install in validation and archive jobs, found ${frontendInstalls.length}`);
}
const entityBuilds = workflow.match(/pnpm -C frontend --filter @prometheus-ags\/prometheus-entity-management build/g) ?? [];
if (entityBuilds.length !== 2) {
  failures.push(`expected entity-management build in validation and archive jobs, found ${entityBuilds.length}`);
}
const topLevelCheckouts = workflow.match(/submodules: true/g) ?? [];
const recursiveRetries = workflow.match(/scripts\/update-submodules\.sh/g) ?? [];
if (topLevelCheckouts.length !== 3 || recursiveRetries.length !== 3) {
  failures.push(
    `expected credentialed top-level checkout plus recursive retry in all 3 source jobs, found ${topLevelCheckouts.length}/${recursiveRetries.length}`,
  );
}
const frontendCacheKeys = workflow.match(/cache-dependency-path:\s*\|[\s\S]*?frontend\/pnpm-lock\.yaml/g) ?? [];
if (frontendCacheKeys.length !== 2) {
  failures.push(`expected both pnpm caches to include frontend/pnpm-lock.yaml, found ${frontendCacheKeys.length}`);
}

if (failures.length) {
  console.error(`Release workflow validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Release workflow validation passed (5 native archive platforms).");
