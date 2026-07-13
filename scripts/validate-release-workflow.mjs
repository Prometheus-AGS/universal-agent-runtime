#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const workflow = readFileSync(new URL("../.github/workflows/release.yml", import.meta.url), "utf8");
const operationalWorkflow = readFileSync(
  new URL("../.github/workflows/operational-resilience.yml", import.meta.url),
  "utf8",
);
const cargoManifest = readFileSync(new URL("../Cargo.toml", import.meta.url), "utf8");
const failures = [];
const trackedPaths = execFileSync("git", ["ls-files", "-z"], { encoding: "utf8" }).split("\0").filter(Boolean);
for (const path of trackedPaths) {
  if (/[<>:"|?*\\]/.test(path)) failures.push(`Windows-incompatible tracked path: ${path}`);
}
const required = [
  "node-version: 22",
  "version: 10.33.0",
  "UAR_LLM__MODEL: openai/gpt-5.4-mini",
  "cargo check --locked --no-default-features --lib --features minimal",
  "cargo check --locked --no-default-features --lib --features server-full",
  "cargo check --locked --no-default-features --lib --features desktop-full",
  "cargo clippy --locked --no-default-features --lib --features server-full --no-deps",
  "cargo test --locked --no-default-features --features server-full",
  "cargo build --locked --release --no-default-features --bin universal-agent-runtime --features server-full",
  "node scripts/validate-release-workflow.mjs",
  "node scripts/validate-static-bundle.mjs static",
  "cp -R static",
  "Copy-Item static",
  "ubuntu-24.04-arm",
  "macos-15-intel",
  "runner: macos-15",
  "windows-latest",
  "http://127.0.0.1:1906/readyz",
  "http://127.0.0.1:1906/healthz",
  "scripts/package-offline-source.sh",
];

for (const value of required) {
  if (!workflow.includes(value)) failures.push(`missing release contract: ${value}`);
}

for (const value of ["protobuf-compiler", "docker stop --timeout 45 uar-resilience"]) {
  if (!operationalWorkflow.includes(value)) failures.push(`missing operational release contract: ${value}`);
}

if (!/\[\[test\]\]\s+name = "test_a2a_grpc"\s+path = "tests\/test_a2a_grpc\.rs"\s+required-features = \["a2a-transport"\]/m.test(cargoManifest)) {
  failures.push("A2A gRPC integration test must require the a2a-transport feature");
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

const tagTrigger = workflow.match(/push:\s*\n\s*tags:\s*\n([\s\S]*?)\n\s*permissions:/)?.[1] ?? "";
if (!tagTrigger.includes("'v*.*.*-rc.*'")) failures.push("candidate tag trigger is missing");
if (!tagTrigger.includes("'release-test-*'")) failures.push("release-test tag trigger is missing");
if (tagTrigger.includes("'v*.*.*'")) {
  failures.push("GA semantic-version tags must not trigger a rebuild; promotion reuses candidate assets");
}

const readyProbes = workflow.match(/http:\/\/127\.0\.0\.1:1906\/readyz/g) ?? [];
const healthProbes = workflow.match(/http:\/\/127\.0\.0\.1:1906\/healthz/g) ?? [];
if (readyProbes.length !== 2 || healthProbes.length !== 2) {
  failures.push(
    `expected readiness and liveness probes in Unix and Windows archive jobs, found ${readyProbes.length}/${healthProbes.length}`,
  );
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
