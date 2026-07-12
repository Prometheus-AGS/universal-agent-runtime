#!/usr/bin/env node

import { readFileSync } from "node:fs";

const workflow = readFileSync(new URL("../.github/workflows/release.yml", import.meta.url), "utf8");
const failures = [];
const required = [
  "node-version: 22",
  "version: 10.33.0",
  "cargo check --locked --lib --features minimal",
  "cargo check --locked --lib --features server-full",
  "cargo check --locked --lib --features desktop-full",
  "cargo clippy --lib --features server-full --no-deps",
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
const recursiveCheckouts = workflow.match(/submodules: recursive/g) ?? [];
if (recursiveCheckouts.length !== 3) {
  failures.push(`expected recursive submodule checkout in all 3 source jobs, found ${recursiveCheckouts.length}`);
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
