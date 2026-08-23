#!/usr/bin/env node

import { existsSync, mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptPath = fileURLToPath(import.meta.url);
const repositoryRoot = resolve(dirname(scriptPath), "..");
const stagingScript = join(repositoryRoot, "scripts", "stage-documentation-references.mjs");

function writeFixture(path, content) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

function createFixture({ rust = true, typescript = true } = {}) {
  const root = mkdtempSync(join(tmpdir(), "uar-doc-staging-"));
  if (rust) writeFixture(join(root, "target", "doc", "index.html"), "<h1>Rust reference</h1>");
  if (typescript) {
    writeFixture(join(root, "sdks", "typescript", "docs", "api", "index.html"), "<h1>TypeScript reference</h1>");
  }
  writeFixture(join(root, "website", "build", "index.html"), "<h1>Portal</h1>");
  return root;
}

function runFixture(root) {
  return spawnSync(process.execPath, [stagingScript, "--root", root], {
    cwd: repositoryRoot,
    encoding: "utf8",
  });
}

function requireControl(label, condition, detail) {
  if (!condition) throw new Error(`${label} failed: ${detail}`);
  console.log(`PASS ${label}`);
}

const missingRust = runFixture(createFixture({ rust: false }));
requireControl(
  "negative control: missing Rust reference",
  missingRust.status !== 0 && missingRust.stderr.includes("Rust reference is missing"),
  `exit=${missingRust.status}`,
);

const missingTypeScript = runFixture(createFixture({ typescript: false }));
requireControl(
  "negative control: missing TypeScript reference",
  missingTypeScript.status !== 0 && missingTypeScript.stderr.includes("TypeScript reference is missing"),
  `exit=${missingTypeScript.status}`,
);

const completeRoot = createFixture();
const complete = runFixture(completeRoot);
const rustOutput = join(completeRoot, "website", "build", "docs", "api", "rust", "index.html");
const typescriptOutput = join(
  completeRoot,
  "website",
  "build",
  "docs",
  "api",
  "typescript",
  "index.html",
);
requireControl(
  "positive control: complete reference staging",
  complete.status === 0 &&
    existsSync(rustOutput) &&
    existsSync(typescriptOutput) &&
    readFileSync(rustOutput, "utf8").includes("Rust reference") &&
    readFileSync(typescriptOutput, "utf8").includes("TypeScript reference"),
  `exit=${complete.status}; stderr=${complete.stderr.trim()}`,
);
