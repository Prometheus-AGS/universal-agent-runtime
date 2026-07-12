#!/usr/bin/env node

import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";

const [manifestPath] = process.argv.slice(2);
if (!manifestPath) {
  console.error("usage: validate-release-manifest.mjs <release-manifest.json>");
  process.exit(2);
}
const root = dirname(manifestPath);
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
const failures = [];
const shaPattern = /^[0-9a-f]{64}$/;

if (manifest.schema_version !== "1.0.0") failures.push("unsupported schema_version");
if (!/^[0-9a-f]{40}$/.test(manifest.source?.sha ?? "")) failures.push("invalid source SHA");
if (!/^sha256:[0-9a-f]{64}$/.test(manifest.image?.digest ?? "")) failures.push("invalid image digest");
if (!Array.isArray(manifest.artifacts) || manifest.artifacts.length === 0) failures.push("artifacts must be non-empty");

for (const artifact of manifest.artifacts ?? []) {
  const path = join(root, artifact.name);
  if (!existsSync(path)) failures.push(`missing artifact: ${artifact.name}`);
  else {
    const actual = createHash("sha256").update(readFileSync(path)).digest("hex");
    if (!shaPattern.test(artifact.sha256) || artifact.sha256 !== actual) failures.push(`digest mismatch: ${artifact.name}`);
  }
  for (const evidence of [...(artifact.sboms ?? []), artifact.signature, artifact.provenance]) {
    if (!evidence || !existsSync(join(root, evidence))) failures.push(`missing linked evidence for ${artifact.name}: ${evidence}`);
  }
}

for (const evidence of manifest.image?.sboms ?? []) {
  if (!existsSync(join(root, evidence))) failures.push(`missing image SBOM: ${evidence}`);
}
if (manifest.support_matrix !== "docs/product-support-matrix.json") failures.push("unexpected support matrix reference");

if (failures.length) {
  console.error(`Release manifest validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log(`Release manifest validation passed (${manifest.artifacts.length} artifacts).`);
