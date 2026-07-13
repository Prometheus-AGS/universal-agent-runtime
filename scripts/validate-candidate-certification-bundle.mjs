#!/usr/bin/env node

import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { basename, dirname, join } from "node:path";

const [directory, supplyManifestPath] = process.argv.slice(2);
if (!directory || !supplyManifestPath) {
  console.error("usage: validate-candidate-certification-bundle.mjs <certification-assets> <release-manifest.json>");
  process.exit(2);
}

const failures = [];
const digest = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const readJson = (path, label) => {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    failures.push(`invalid ${label}: ${error.message}`);
    return {};
  }
};
const supply = readJson(supplyManifestPath, "release manifest");
const manifestPath = join(directory, "candidate-certification-manifest.json");
const manifest = readJson(manifestPath, "candidate certification manifest");
const indexPath = join(directory, "CANDIDATE_CERTIFICATION_SHA256SUMS");

if (manifest.schema_version !== 1) failures.push("unsupported candidate certification manifest schema");
if (manifest.candidate !== supply.release) failures.push("certification candidate does not match supply release");
if (manifest.source_sha !== supply.source?.sha) failures.push("certification source does not match supply source");
if (manifest.supply?.release_manifest_sha256 !== digest(supplyManifestPath)) failures.push("certification release-manifest digest mismatch");
const supplyIndex = join(dirname(supplyManifestPath), "SHA256SUMS");
if (!existsSync(supplyIndex) || manifest.supply?.checksum_index_sha256 !== digest(supplyIndex)) {
  failures.push("certification supply checksum-index digest mismatch");
}

const repositoryPattern = (supply.source?.repository ?? "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const runPattern = new RegExp(`^https://github\\.com/${repositoryPattern}/actions/runs/[0-9]+$`);
if (!runPattern.test(manifest.workflow?.run_url ?? "")) failures.push("certification workflow run URL is not immutable");
if (!(manifest.workflow?.identity ?? "").includes("/.github/workflows/candidate-certification.yml@")) {
  failures.push("unexpected candidate certification workflow identity");
}

const expectedArchive = `candidate-certification-${supply.release}.tar.gz`;
if (manifest.archive?.name !== expectedArchive || basename(manifest.archive?.name ?? "") !== manifest.archive?.name) {
  failures.push("unexpected candidate certification archive name");
}
const archivePath = join(directory, manifest.archive?.name ?? "missing");
if (!existsSync(archivePath) || !statSync(archivePath).isFile()) failures.push("missing candidate certification archive");
else if (manifest.archive?.sha256 !== digest(archivePath)) failures.push("candidate certification archive digest mismatch");

if (!existsSync(indexPath)) {
  failures.push("missing candidate certification checksum index");
} else {
  const entries = new Map();
  for (const line of readFileSync(indexPath, "utf8").trim().split("\n")) {
    const match = /^([0-9a-f]{64})  ([A-Za-z0-9._@+-]+)$/.exec(line);
    if (!match) {
      failures.push(`invalid certification checksum line: ${line}`);
      continue;
    }
    entries.set(match[2], match[1]);
  }
  const expected = ["candidate-certification-manifest.json", expectedArchive].sort();
  const actual = readdirSync(directory)
    .filter((name) => name !== "CANDIDATE_CERTIFICATION_SHA256SUMS" && statSync(join(directory, name)).isFile())
    .sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push("candidate certification asset set is not exact");
  for (const name of expected) {
    if (!existsSync(join(directory, name)) || entries.get(name) !== digest(join(directory, name))) {
      failures.push(`candidate certification checksum mismatch: ${name}`);
    }
  }
  if (entries.size !== expected.length) failures.push("candidate certification checksum index has unexpected entries");
}

if (failures.length) {
  console.error(`Candidate certification bundle validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log(`Candidate certification bundle passed for ${manifest.candidate} (${manifest.source_sha}).`);
