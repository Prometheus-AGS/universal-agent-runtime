#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

const repoRoot = resolve(import.meta.dirname, "..");
const outputIndex = process.argv.indexOf("--output");
const outputPath = outputIndex >= 0 ? resolve(repoRoot, process.argv[outputIndex + 1]) : null;
const startedAt = new Date().toISOString();
const commands = [
  [process.execPath, [resolve(repoRoot, "scripts/check-storybook-a11y-suppressions.mjs")]],
  [process.execPath, [resolve(repoRoot, "scripts/test-storybook-a11y-suppressions-negative.mjs")]],
];
const results = [];

for (const [command, args] of commands) {
  const result = spawnSync(command, args, { cwd: repoRoot, encoding: "utf8" });
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  results.push({
    command: [command, ...args].join(" "),
    exitCode: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
  });
  if (result.status !== 0) break;
}

const receipt = {
  schemaVersion: 1,
  startedAt,
  completedAt: new Date().toISOString(),
  exitCode: results.every((result) => result.exitCode === 0) ? 0 : 1,
  results,
};

if (outputPath) writeFileSync(outputPath, `${JSON.stringify(receipt, null, 2)}\n`);
if (receipt.exitCode !== 0) process.exit(receipt.exitCode);
