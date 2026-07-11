import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const result = spawnSync(
  process.execPath,
  ["scripts/check-frontend-boundaries.mjs", "--fixture-dir", "scripts/fixtures/frontend-boundaries"],
  { cwd: root, encoding: "utf8" },
);
const output = `${result.stdout}\n${result.stderr}`;
const expectedRules = [
  "component-direct-fetch",
  "component-service-import",
  "component-store-import",
  "hook-service-import",
  "store-upward-import",
  "service-upward-import",
];

if (result.status === 0) throw new Error("Negative boundary fixture unexpectedly passed");
for (const rule of expectedRules) {
  if (!output.includes(rule)) throw new Error(`Negative boundary fixture did not trigger ${rule}`);
}
console.log(`Frontend boundary negative fixtures passed (${expectedRules.length} rules rejected).`);
