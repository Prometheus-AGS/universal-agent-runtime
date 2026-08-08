import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import { inspectInitialJavaScriptBudget } from "./check-frontend-budgets.mjs";

const SHIKI_KEY = "src/shared/markdown/blocks/vendor-shiki.ts";
const MERMAID_KEY = "src/shared/markdown/blocks/vendor-mermaid.ts";
const execFileAsync = promisify(execFile);
const budgetCheckerPath = fileURLToPath(new URL("./check-frontend-budgets.mjs", import.meta.url));

const createFixture = async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "uar-budget-fixture-"));
  await mkdir(path.join(root, "assets"));
  await Promise.all([
    writeFile(path.join(root, "assets", "index.js"), "import './shared.js';\nconsole.log('entry');\n"),
    writeFile(path.join(root, "assets", "shared.js"), "export const shared = 'shared';\n"),
    writeFile(path.join(root, "assets", "cycle.js"), "export const cycle = 'cycle';\n"),
    writeFile(path.join(root, "assets", "vendor-pglite-fixture.js"), "export const pglite = 'excluded';\n"),
    writeFile(path.join(root, "assets", "vendor-shiki-fixture.js"), "export const shiki = true;\n"),
    writeFile(path.join(root, "assets", "vendor-mermaid-fixture.js"), "export const mermaid = true;\n"),
    writeFile(path.join(root, "assets", "pglite-fixture.data"), "pglite-data"),
    writeFile(path.join(root, "assets", "pglite-fixture.wasm"), "pglite-wasm"),
    writeFile(path.join(root, "assets", "pglite-seed-v3.tar.gz"), "pglite-seed"),
  ]);

  const manifest = {
    "index.html": {
      file: "assets/index.js",
      src: "index.html",
      isEntry: true,
      imports: ["_shared.js", "_cycle.js", "_vendor-pglite.js"],
      dynamicImports: [SHIKI_KEY, MERMAID_KEY],
      assets: ["assets/pglite-fixture.data"],
    },
    "_shared.js": { file: "assets/shared.js", imports: ["_cycle.js"] },
    "_cycle.js": { file: "assets/cycle.js", imports: ["_shared.js"] },
    "_vendor-pglite.js": { file: "assets/vendor-pglite-fixture.js", name: "vendor-pglite" },
    [SHIKI_KEY]: { file: "assets/vendor-shiki-fixture.js", isDynamicEntry: true },
    [MERMAID_KEY]: { file: "assets/vendor-mermaid-fixture.js", isDynamicEntry: true },
    "../node_modules/@electric-sql/pglite/dist/pglite.data": {
      file: "assets/pglite-fixture.data",
      src: "../node_modules/@electric-sql/pglite/dist/pglite.data",
    },
    "../node_modules/@electric-sql/pglite/dist/pglite.wasm": {
      file: "assets/pglite-fixture.wasm",
      src: "../node_modules/@electric-sql/pglite/dist/pglite.wasm",
    },
    "src/platform/pglite/pglite-seed-v3.tar.gz": {
      file: "assets/pglite-seed-v3.tar.gz",
      src: "src/platform/pglite/pglite-seed-v3.tar.gz",
    },
  };
  const engineGraph = {
    schemaVersion: 1,
    chunks: {
      "assets/index.js": { imports: ["assets/shared.js", "assets/cycle.js", "assets/vendor-pglite-fixture.js"], dynamicImports: [], engineModules: [] },
      "assets/shared.js": { imports: ["assets/cycle.js"], dynamicImports: [], engineModules: [] },
      "assets/cycle.js": { imports: ["assets/shared.js"], dynamicImports: [], engineModules: [] },
      "assets/vendor-pglite-fixture.js": { imports: [], dynamicImports: [], engineModules: [] },
      "assets/vendor-shiki-fixture.js": { imports: [], dynamicImports: [], engineModules: ["shiki/dist/index.mjs"] },
      "assets/vendor-mermaid-fixture.js": { imports: [], dynamicImports: [], engineModules: ["mermaid/dist/mermaid.esm.mjs"] },
    },
  };
  await writeFile(path.join(root, "manifest.json"), JSON.stringify(manifest));
  await writeFile(path.join(root, "engine-graph.json"), JSON.stringify(engineGraph));
  return { root, manifest, engineGraph };
};

const expectFailure = async (label, operation, pattern) => {
  await assert.rejects(operation, pattern, label);
};

const fixture = await createFixture();
try {
  const passing = await inspectInitialJavaScriptBudget({
    ...fixture,
    buildRoot: fixture.root,
    limitBytes: Number.MAX_SAFE_INTEGER,
  });
  assert.equal(passing.files.length, 4, "shared and cyclic JavaScript files must be traversed once");
  assert.equal(passing.exclusions.pgliteJavaScript.length, 1, "one named static PGlite JavaScript chunk must be excluded and reported");
  assert.equal(passing.exclusions.pgliteNonJavaScriptAssets.length, 3, "PGlite data, WASM, and schema seed must be reported outside the JavaScript total");
  assert.ok(passing.exclusions.pgliteNonJavaScriptAssets.every(({ rawBytes }) => rawBytes > 0), "PGlite asset byte sizes must be reported");

  const exact = await inspectInitialJavaScriptBudget({
    ...fixture,
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  });
  assert.equal(exact.verdict, "pass", "exact-limit closure must pass");

  await expectFailure("over-limit closure", () => inspectInitialJavaScriptBudget({
    ...fixture,
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes - 1,
  }), /Initial JavaScript closure/u);

  await expectFailure("malformed manifest", () => inspectInitialJavaScriptBudget({
    manifest: [], engineGraph: fixture.engineGraph, buildRoot: fixture.root, limitBytes: passing.gzipBytes,
  }), /manifest must be an object/iu);

  await expectFailure("missing import", () => inspectInitialJavaScriptBudget({
    manifest: { ...fixture.manifest, "index.html": { ...fixture.manifest["index.html"], imports: ["missing.js"] } },
    engineGraph: fixture.engineGraph,
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  }), /does not resolve/iu);

  await expectFailure("missing static PGlite JavaScript", () => inspectInitialJavaScriptBudget({
    manifest: {
      ...fixture.manifest,
      "index.html": { ...fixture.manifest["index.html"], imports: ["_shared.js", "_cycle.js"] },
    },
    engineGraph: fixture.engineGraph,
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  }), /exactly one named vendor-pglite chunk/iu);

  const manifestWithoutWasm = { ...fixture.manifest };
  delete manifestWithoutWasm["../node_modules/@electric-sql/pglite/dist/pglite.wasm"];
  await expectFailure("missing PGlite WASM", () => inspectInitialJavaScriptBudget({
    manifest: manifestWithoutWasm,
    engineGraph: fixture.engineGraph,
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  }), /exactly one pglite-wasm/u);

  await expectFailure("duplicate PGlite schema seed", () => inspectInitialJavaScriptBudget({
    manifest: {
      ...fixture.manifest,
      "duplicate/pglite-seed-v3.tar.gz": fixture.manifest["src/platform/pglite/pglite-seed-v3.tar.gz"],
    },
    engineGraph: fixture.engineGraph,
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  }), /exactly one schema-seed/u);

  await expectFailure("PGlite asset type confusion", () => inspectInitialJavaScriptBudget({
    manifest: {
      ...fixture.manifest,
      "../node_modules/@electric-sql/pglite/dist/pglite.wasm": {
        ...fixture.manifest["../node_modules/@electric-sql/pglite/dist/pglite.wasm"],
        file: "assets/pglite-fixture.data",
      },
    },
    engineGraph: fixture.engineGraph,
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  }), /unexpected emitted file/u);

  await expectFailure("path escape", () => inspectInitialJavaScriptBudget({
    manifest: { ...fixture.manifest, "_shared.js": { file: "../escape.js" } },
    engineGraph: fixture.engineGraph,
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  }), /escapes the build root/iu);

  await expectFailure("eager engine", () => inspectInitialJavaScriptBudget({
    manifest: fixture.manifest,
    engineGraph: {
      ...fixture.engineGraph,
      chunks: {
        ...fixture.engineGraph.chunks,
        "assets/index.js": {
          ...fixture.engineGraph.chunks["assets/index.js"],
          engineModules: ["shiki/dist/index.mjs"],
        },
      },
    },
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  }), /Lazy engine integrity failed/iu);

  const { ["assets/shared.js"]: omittedStaticRecord, ...incompleteChunks } = fixture.engineGraph.chunks;
  assert.ok(omittedStaticRecord, "fixture must contain the static record being removed");
  await expectFailure("incomplete engine static graph", () => inspectInitialJavaScriptBudget({
    manifest: fixture.manifest,
    engineGraph: { ...fixture.engineGraph, chunks: incompleteChunks },
    buildRoot: fixture.root,
    limitBytes: passing.gzipBytes,
  }), /Engine graph static record assets\/shared\.js/iu);

  const failureOutput = path.join(fixture.root, "failure.json");
  let cliFailure;
  try {
    await execFileAsync(process.execPath, [
      budgetCheckerPath,
      "--root", fixture.root,
      "--manifest", path.join(fixture.root, "manifest.json"),
      "--engine-graph", path.join(fixture.root, "engine-graph.json"),
      "--limit", "0",
      "--output", failureOutput,
    ]);
  } catch (error) {
    cliFailure = error;
  }
  assert.ok(cliFailure, "over-limit CLI fixture must fail");
  if (!await readFile(failureOutput, "utf8").catch(() => null)) {
    throw new Error(`CLI failure evidence was not written: ${cliFailure.stderr ?? cliFailure.message}`);
  }
  const retainedFailure = JSON.parse(await readFile(failureOutput, "utf8"));
  assert.equal(retainedFailure.verdict, "fail", "CLI failures must retain machine-readable evidence");
  assert.match(retainedFailure.error.message, /Initial JavaScript closure/u);

  const malformedManifest = path.join(fixture.root, "malformed-manifest.json");
  const malformedOutput = path.join(fixture.root, "malformed-failure.json");
  await writeFile(malformedManifest, "{");
  await assert.rejects(execFileAsync(process.execPath, [
    budgetCheckerPath,
    "--root", fixture.root,
    "--manifest", malformedManifest,
    "--engine-graph", path.join(fixture.root, "engine-graph.json"),
    "--limit", "0",
    "--output", malformedOutput,
  ]));
  const retainedMalformedFailure = JSON.parse(await readFile(malformedOutput, "utf8"));
  assert.equal(retainedMalformedFailure.verdict, "fail");
  assert.match(retainedMalformedFailure.error.message, /manifest.*malformed/iu);

  console.log("Frontend bundle budget fixture proofs passed");
} finally {
  await rm(fixture.root, { recursive: true, force: true });
}
