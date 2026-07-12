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

if (failures.length) {
  console.error(`Release workflow validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Release workflow validation passed (5 native archive platforms).");
