import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const result = spawnSync(
  process.execPath,
  ["scripts/check-platform-adapters.mjs", "--fixture-dir", "scripts/fixtures/platform-adapters"],
  { cwd: root, encoding: "utf8" },
);
const output = `${result.stdout}\n${result.stderr}`;
const expectedRules = [
  "direct-entity-package-import",
  "retired-agui-file",
  "retired-agui-import",
  "retired-pglite-file",
  "retired-pglite-import",
  "platform-react-boundary",
];

if (result.status === 0) throw new Error("Negative platform-adapter fixture unexpectedly passed");
for (const rule of expectedRules) {
  if (!output.includes(rule)) {
    throw new Error(`Negative platform-adapter fixture did not trigger ${rule}`);
  }
}
for (const fixture of [
  "lib/pglite/index.ts",
  "platform/react-dom.ts",
  "platform/react.ts",
  "platform/widget.tsx",
  "protocols/agui/index.ts",
]) {
  if (!output.includes(fixture)) {
    throw new Error(`Negative platform-adapter fixture did not reject ${fixture}`);
  }
}

const cleanResult = spawnSync(
  process.execPath,
  ["scripts/check-platform-adapters.mjs", "--fixture-dir", "scripts/fixtures/platform-adapters-clean"],
  { cwd: root, encoding: "utf8" },
);
if (cleanResult.status !== 0) {
  throw new Error(`Clean platform-adapter fixture failed:\n${cleanResult.stdout}\n${cleanResult.stderr}`);
}

const printResult = spawnSync(
  process.execPath,
  [
    "scripts/check-platform-adapters.mjs",
    "--fixture-dir",
    "scripts/fixtures/platform-adapters",
    "--print",
  ],
  { cwd: root, encoding: "utf8" },
);
if (printResult.status === 0 || !printResult.stdout.includes("direct-entity-package-import")) {
  throw new Error("Platform adapter --print mode did not preserve failing gate semantics");
}

const missingArgument = spawnSync(
  process.execPath,
  ["scripts/check-platform-adapters.mjs", "--fixture-dir"],
  { cwd: root, encoding: "utf8" },
);
if (missingArgument.status === 0 || !missingArgument.stderr.includes("requires a path")) {
  throw new Error("Platform adapter gate accepted a missing --fixture-dir value");
}

const missingRoot = spawnSync(
  process.execPath,
  ["scripts/check-platform-adapters.mjs", "--fixture-dir", "scripts/fixtures/does-not-exist"],
  { cwd: root, encoding: "utf8" },
);
if (missingRoot.status === 0 || !missingRoot.stderr.includes("missing-scan-root")) {
  throw new Error("Platform adapter gate did not report a missing scan root cleanly");
}

const missingAdapter = spawnSync(
  process.execPath,
  [
    "scripts/check-platform-adapters.mjs",
    "--fixture-dir",
    "scripts/fixtures/platform-adapters",
    "--check-required",
  ],
  { cwd: root, encoding: "utf8" },
);
if (missingAdapter.status === 0 || !missingAdapter.stderr.includes("missing-platform-adapter")) {
  throw new Error("Platform adapter gate did not reject a missing required adapter");
}
console.log(`Platform adapter fixtures passed (${expectedRules.length} rules rejected; clean control accepted).`);
