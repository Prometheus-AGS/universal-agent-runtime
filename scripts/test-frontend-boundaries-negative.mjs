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
  "platform-upward-feature-import",
  "shared-upward-feature-import",
  "feature-upward-app-import",
  "feature-cross-implementation-import",
];

if (result.status === 0) throw new Error("Negative boundary fixture unexpectedly passed");
for (const rule of expectedRules) {
  if (!new RegExp(`:\\d+\\|${rule}(?:\\n|\\r|$)`).test(output)) {
    throw new Error(`Negative boundary fixture did not trigger ${rule} with a source location`);
  }
}
const forbiddenCases = [
  ["render-body-setter", "react-render-body-state-setter"],
  ["per-row-for-each", "feature-per-row-graph-write"],
  ["per-row-unbraced-for", "feature-per-row-graph-write"],
  ["per-row-braced-header-call", "feature-per-row-graph-write"],
  ["facade-management-root", "entity-facade-bypass"],
  ["facade-management-subpath", "entity-facade-bypass"],
  ["facade-core-root", "entity-facade-bypass"],
  ["facade-core-subpath", "entity-facade-bypass"],
  ["duplicate-entity-cache", "duplicate-graph-owned-cache"],
  // Presentational library (frontend/packages/uar-ui) may not reach app, state, graph, Tauri or I/O.
  ["library-direct-fetch", "library-direct-fetch"],
  ["library-app-import", "library-app-import"],
  ["library-relative-escape", "library-app-import"],
  ["library-state-import", "library-state-import"],
  ["library-entity-import", "library-entity-import"],
  ["library-tauri-import", "library-tauri-import"],
  ["library-service-import", "library-service-import"],
  // scripts/frontend-boundary-allowlist.txt may only shrink and may not go stale.
  ["allowlist-growth", "boundary-allowlist-growth", true],
  ["allowlist-stale", "boundary-allowlist-stale", true],
  ["allowlist-malformed", "boundary-allowlist-malformed", true],
];
for (const [fixture, rule, withAllowlist] of forbiddenCases) {
  const fixtureDir = `scripts/fixtures/frontend-boundary-cases/${fixture}`;
  const rejected = spawnSync(
    process.execPath,
    [
      "scripts/check-frontend-boundaries.mjs",
      "--fixture-dir",
      fixtureDir,
      ...(withAllowlist
        ? ["--allowlist", `${fixtureDir}/allowlist.txt`, "--allowlist-base", `${fixtureDir}/base.txt`]
        : []),
    ],
    { cwd: root, encoding: "utf8" },
  );
  const rejectedOutput = `${rejected.stdout}\n${rejected.stderr}`;
  if (rejected.status === 0) throw new Error(`${fixture} unexpectedly passed`);
  if (!new RegExp(`:\\d+\\|${rule}(?:\\n|\\r|$)`).test(rejectedOutput)) {
    throw new Error(`${fixture} did not trigger ${rule} with a source location`);
  }
}
const allowed = spawnSync(
  process.execPath,
  ["scripts/check-frontend-boundaries.mjs", "--fixture-dir", "scripts/fixtures/frontend-boundaries-allowed"],
  { cwd: root, encoding: "utf8" },
);
if (allowed.status !== 0) {
  throw new Error(`Allowed boundary fixtures unexpectedly failed:\n${allowed.stdout}\n${allowed.stderr}`);
}
console.log(`Frontend boundary fixtures passed (${expectedRules.length + forbiddenCases.length} independently checked forbidden rules rejected; allowed UI/domain paths accepted).`);
