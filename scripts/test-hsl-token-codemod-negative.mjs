import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const fixture = "scripts/fixtures/hsl-token-codemod/case-variant.tsx";
const result = spawnSync(
  process.execPath,
  ["scripts/check-hsl-token-codemod.mjs", "--fixture-file", fixture],
  { cwd: root, encoding: "utf8" },
);
const output = `${result.stdout}\n${result.stderr}`;

if (result.status === 0 || !output.includes(fixture) || !output.includes("2 legacy")) {
  throw new Error("HSL token codemod gate accepted a case-variant HSL/HSLA call site");
}

console.log("HSL token codemod negative fixture passed (case-variant HSL and HSLA syntax rejected).");
