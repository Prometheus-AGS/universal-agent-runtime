#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const frontend = resolve(root, "frontend");
const allowlistIndex = process.argv.indexOf("--allowlist");
const allowlistPath = allowlistIndex >= 0
  ? resolve(root, process.argv[allowlistIndex + 1])
  : resolve(root, "scripts/frontend-flat2-style-allowlist.txt");
const fixtureIndex = process.argv.indexOf("--fixture-dir");
const scanRoot = fixtureIndex >= 0
  ? resolve(root, process.argv[fixtureIndex + 1])
  : resolve(frontend, "src");
const target = relative(frontend, scanRoot).replaceAll("\\", "/");

const result = spawnSync(
  "pnpm",
  [
    "exec",
    "eslint",
    "--config",
    "eslint-flat2-baseline.config.js",
    "--format",
    "json",
    target,
  ],
  {
    cwd: frontend,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  },
);

if (result.error) throw result.error;
if (result.status !== 0 && result.status !== 1) {
  process.stderr.write(result.stderr);
  process.stderr.write(result.stdout);
  process.exit(result.status ?? 2);
}

let reports;
try {
  reports = JSON.parse(result.stdout);
} catch (error) {
  process.stderr.write(result.stderr);
  throw new Error(`Flat 2.0 ESLint output was not valid JSON: ${error.message}`);
}

const fatalMessages = reports.flatMap((report) =>
  report.messages
    .filter((message) => message.fatal)
    .map((message) => `${relative(root, report.filePath).replaceAll("\\", "/")}:${message.line}:${message.column} ${message.message}`),
);
if (fatalMessages.length) {
  console.error("Flat 2.0 baseline could not be parsed:");
  for (const message of fatalMessages) console.error(`  ! ${message}`);
  process.exit(2);
}

function repoPath(path) {
  return relative(root, path).replaceAll("\\", "/");
}

function offsetAt(source, line, column) {
  const lines = source.split("\n");
  let offset = 0;
  for (let index = 1; index < line; index += 1) offset += lines[index - 1].length + 1;
  return offset + column - 1;
}

function sourceFragment(report, message) {
  if (!report.source || !message.endLine || !message.endColumn) return "<unknown>";
  const start = offsetAt(report.source, message.line, message.column);
  const end = offsetAt(report.source, message.endLine, message.endColumn);
  return report.source.slice(start, end);
}

const bases = [];
for (const report of reports) {
  const file = repoPath(report.filePath);
  for (const message of report.messages) {
    if (message.ruleId === "unicorn/filename-case") {
      bases.push(`${file}|${message.ruleId}`);
    }
    if (message.ruleId === "no-restricted-syntax") {
      bases.push(`${file}|${message.ruleId}|${JSON.stringify(sourceFragment(report, message))}`);
    }
  }
}

bases.sort();
const occurrences = new Map();
const actual = bases.map((base) => {
  const occurrence = (occurrences.get(base) ?? 0) + 1;
  occurrences.set(base, occurrence);
  return `${base}|occurrence=${occurrence}`;
});

if (process.argv.includes("--print")) {
  process.stdout.write(`${actual.join("\n")}\n`);
} else {
  const expected = readFileSync(allowlistPath, "utf8")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"))
    .sort();
  const unexpected = actual.filter((item) => !expected.includes(item));
  const stale = expected.filter((item) => !actual.includes(item));

  if (unexpected.length || stale.length) {
    if (unexpected.length) {
      console.error("New Flat 2.0 style violations (fix; do not extend the allowlist):");
      for (const item of unexpected) console.error(`  + ${item}`);
    }
    if (stale.length) {
      console.error("Resolved Flat 2.0 allowlist entries (remove them from the allowlist):");
      for (const item of stale) console.error(`  - ${item}`);
    }
    process.exitCode = 1;
  } else {
    console.log(`Flat 2.0 style gate passed (${actual.length} tracked legacy violations, 0 new).`);
  }
}
