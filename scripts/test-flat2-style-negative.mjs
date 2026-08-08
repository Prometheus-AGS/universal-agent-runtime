import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const frontend = resolve(root, "frontend");
const result = spawnSync(
  process.execPath,
  [
    "scripts/check-flat2-style.mjs",
    "--fixture-dir",
    "frontend/test-fixtures/flat2-style",
    "--print",
  ],
  { cwd: root, encoding: "utf8" },
);

if (result.status !== 0) {
  process.stderr.write(result.stderr);
  process.exit(result.status ?? 1);
}

const findings = result.stdout.trim().split(/\r?\n/).filter(Boolean);
const styleFindings = findings.filter((line) => line.includes("|no-restricted-syntax|"));
const filenameFindings = findings.filter((line) => line.includes("|unicorn/filename-case|"));

if (styleFindings.length !== 4) {
  throw new Error(`Expected four prohibited-style findings, received ${styleFindings.length}`);
}
if (filenameFindings.length !== 1) {
  throw new Error(`Expected one filename-case finding, received ${filenameFindings.length}`);
}

for (const [filename, source, ruleId] of [
  ["src/new-flat2-surface.tsx", "export const className = \"border\";\n", "no-restricted-syntax"],
  ["src/NewFlat2Surface.tsx", "export const value = 1;\n", "unicorn/filename-case"],
]) {
  const lint = spawnSync(
    "pnpm",
    ["exec", "eslint", "--stdin", "--stdin-filename", filename],
    { cwd: frontend, encoding: "utf8", input: source },
  );
  if (lint.status === 0 || !`${lint.stdout}\n${lint.stderr}`.includes(ruleId)) {
    throw new Error(`Normal frontend lint did not reject ${filename} with ${ruleId}`);
  }
}

const fatalFixture = mkdtempSync(join(frontend, "test-fixtures/flat2-fatal-"));
try {
  writeFileSync(join(fatalFixture, "broken.tsx"), "export const broken = <div>;\n");
  const fatal = spawnSync(
    process.execPath,
    ["scripts/check-flat2-style.mjs", "--fixture-dir", fatalFixture],
    { cwd: root, encoding: "utf8" },
  );
  if (fatal.status !== 2 || !fatal.stderr.includes("Flat 2.0 baseline could not be parsed")) {
    throw new Error("A fatal parser diagnostic did not fail the baseline gate");
  }
} finally {
  rmSync(fatalFixture, { recursive: true, force: true });
}

const temp = mkdtempSync(join(tmpdir(), "uar-flat2-style-"));
try {
  const newFindingAllowlist = join(temp, "new-finding.txt");
  writeFileSync(newFindingAllowlist, `${[filenameFindings[0], styleFindings[0]].sort().join("\n")}\n`);
  const newFinding = spawnSync(
    process.execPath,
    [
      "scripts/check-flat2-style.mjs",
      "--fixture-dir",
      "frontend/test-fixtures/flat2-style",
      "--allowlist",
      newFindingAllowlist,
    ],
    { cwd: root, encoding: "utf8" },
  );
  if (newFinding.status === 0 || !newFinding.stderr.includes(styleFindings[1])) {
    throw new Error("An added finding inside an allowlisted file did not fail the gate");
  }

  const staleAllowlist = join(temp, "stale.txt");
  const staleEntry = "frontend/test-fixtures/flat2-style/PascalFixture.tsx|no-restricted-syntax|\"resolved\"|occurrence=1";
  writeFileSync(staleAllowlist, `${[...findings, staleEntry].sort().join("\n")}\n`);
  const stale = spawnSync(
    process.execPath,
    [
      "scripts/check-flat2-style.mjs",
      "--fixture-dir",
      "frontend/test-fixtures/flat2-style",
      "--allowlist",
      staleAllowlist,
    ],
    { cwd: root, encoding: "utf8" },
  );
  if (stale.status === 0 || !stale.stderr.includes(staleEntry)) {
    throw new Error("A resolved allowlist entry did not fail the gate as stale");
  }
} finally {
  rmSync(temp, { recursive: true, force: true });
}

console.log("Flat 2.0 negative fixtures passed (literal/template syntax, filenames, parse failures, additions, and stale entries rejected).");
