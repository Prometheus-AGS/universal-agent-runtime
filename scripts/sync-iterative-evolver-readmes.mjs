#!/usr/bin/env node

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const source = resolve(
  root,
  "crates/prometheus-skill-system/skills/process/iterative-evolver/README.md",
);
const mirrors = [
  ".agent/skills/iterative-evolver/README.md",
  ".claude/skills/iterative-evolver/README.md",
  ".codex/skills/iterative-evolver/README.md",
  ".cursor/skills/iterative-evolver/README.md",
  ".windsurf/skills/iterative-evolver/README.md",
];
const write = process.argv.includes("--write");

if (!existsSync(source)) {
  console.error(`Iterative-evolver README source is missing: ${source}`);
  process.exit(1);
}

const expected = readFileSync(source);
const drifted = [];
for (const path of mirrors) {
  const absolute = resolve(root, path);
  const current = existsSync(absolute) ? readFileSync(absolute) : null;
  if (current?.equals(expected)) continue;
  drifted.push(path);
  if (write) writeFileSync(absolute, expected);
}

if (write) {
  console.log(`Synchronized ${drifted.length} iterative-evolver README mirror(s).`);
  process.exit(0);
}

if (drifted.length) {
  console.error(`Iterative-evolver README mirrors are stale:\n- ${drifted.join("\n- ")}`);
  process.exit(1);
}

console.log(`Iterative-evolver README mirrors match the pinned source (${mirrors.length}).`);
