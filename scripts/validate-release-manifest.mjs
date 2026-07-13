#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { basename, dirname, join } from "node:path";

const [manifestPath] = process.argv.slice(2);
if (!manifestPath) {
  console.error("usage: validate-release-manifest.mjs <release-manifest.json>");
  process.exit(2);
}
const root = dirname(manifestPath);
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
const failures = [];
const shaPattern = /^[0-9a-f]{64}$/;
const safeNamePattern = /^(?!\.)(?!.*(?:^|\/)\.\.(?:\/|$))[^/\\]+$/;

const digest = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const requireFile = (name, label) => {
  if (!safeNamePattern.test(name ?? "")) {
    failures.push(`unsafe ${label} path: ${name}`);
    return false;
  }
  if (!existsSync(join(root, name)) || !statSync(join(root, name)).isFile()) {
    failures.push(`missing ${label}: ${name}`);
    return false;
  }
  return true;
};

function archiveBinary(artifact) {
  const archivePath = join(root, artifact.name);
  const zip = artifact.name.endsWith(".zip");
  try {
    const listing = execFileSync(zip ? "unzip" : "tar", zip ? ["-Z1", archivePath] : ["-tzf", archivePath], {
      encoding: "utf8",
    });
    const executable = zip ? "universal-agent-runtime.exe" : "universal-agent-runtime";
    const matches = listing
      .split("\n")
      .filter(Boolean)
      .filter((entry) => basename(entry) === executable && !entry.startsWith("/") && !entry.split("/").includes(".."));
    if (matches.length !== 1) throw new Error(`expected one ${executable}, found ${matches.length}`);
    const contents = execFileSync(zip ? "unzip" : "tar", zip ? ["-p", archivePath, matches[0]] : ["-xOzf", archivePath, matches[0]], {
      maxBuffer: 512 * 1024 * 1024,
    });
    const actual = createHash("sha256").update(contents).digest("hex");
    if (artifact.binary?.path !== matches[0] || artifact.binary?.sha256 !== actual) {
      failures.push(`embedded binary trace mismatch: ${artifact.name}`);
    }
  } catch (error) {
    failures.push(`cannot inspect embedded binary in ${artifact.name}: ${error.message}`);
  }
}

if (manifest.schema_version !== "1.0.0") failures.push("unsupported schema_version");
if (!/^v\d+\.\d+\.\d+(?:-rc\.\d+)?$/.test(manifest.release ?? "")) failures.push("invalid release tag");
if (!/^[0-9a-f]{40}$/.test(manifest.source?.sha ?? "")) failures.push("invalid source SHA");
if (!/^[0-9a-f]{40}$/.test(manifest.source?.git_tree ?? "")) failures.push("invalid source tree");
for (const field of ["cargo_lock_sha256", "catalog_sha256", "model_bundle_sha256"]) {
  if (!shaPattern.test(manifest.source?.[field] ?? "")) failures.push(`invalid source digest: ${field}`);
}
if (!Array.isArray(manifest.source?.model_inputs) || manifest.source.model_inputs.length === 0) {
  failures.push("model_inputs must be non-empty");
}
if (!/^sha256:[0-9a-f]{64}$/.test(manifest.image?.digest ?? "")) failures.push("invalid image digest");
if (!Array.isArray(manifest.artifacts) || manifest.artifacts.length === 0) failures.push("artifacts must be non-empty");

const linkedEvidence = new Set();
for (const artifact of manifest.artifacts ?? []) {
  const present = requireFile(artifact.name, "artifact");
  if (present && (!shaPattern.test(artifact.sha256 ?? "") || artifact.sha256 !== digest(join(root, artifact.name)))) {
    failures.push(`digest mismatch: ${artifact.name}`);
  }
  if (artifact.kind === "platform-archive") {
    if (!shaPattern.test(artifact.binary?.sha256 ?? "") || !artifact.binary?.path) {
      failures.push(`invalid embedded binary trace: ${artifact.name}`);
    } else if (present) {
      archiveBinary(artifact);
    }
    if (!Array.isArray(artifact.sboms) || artifact.sboms.length < 4) {
      failures.push(`archive and binary SBOMs required: ${artifact.name}`);
    }
  } else if (artifact.kind === "offline-source") {
    if (artifact.name !== "uar-offline-source.tar.gz") failures.push(`unexpected offline source artifact: ${artifact.name}`);
    if (artifact.binary !== undefined) failures.push("offline source artifact must not claim an embedded runtime binary");
    if (!Array.isArray(artifact.sboms) || artifact.sboms.length < 2) failures.push("offline source SBOMs are incomplete");
  } else {
    failures.push(`unknown artifact kind: ${artifact.kind}`);
  }
  for (const evidence of [...(artifact.sboms ?? []), artifact.signature, artifact.provenance]) {
    if (evidence) linkedEvidence.add(evidence);
    requireFile(evidence, `linked evidence for ${artifact.name}`);
  }
}

for (const evidence of [...(manifest.source?.sboms ?? []), ...(manifest.image?.sboms ?? [])]) {
  linkedEvidence.add(evidence);
  requireFile(evidence, "source/image SBOM");
}
if ((manifest.source?.sboms ?? []).length < 2) failures.push("source CycloneDX and SPDX SBOMs required");
if ((manifest.image?.sboms ?? []).length < 2) failures.push("image CycloneDX and SPDX SBOMs required");

const evidenceNames = new Set();
for (const evidence of manifest.evidence?.files ?? []) {
  if (evidenceNames.has(evidence.name)) failures.push(`duplicate evidence file: ${evidence.name}`);
  evidenceNames.add(evidence.name);
  if (requireFile(evidence.name, "evidence file") && (!shaPattern.test(evidence.sha256 ?? "") || evidence.sha256 !== digest(join(root, evidence.name)))) {
    failures.push(`evidence digest mismatch: ${evidence.name}`);
  }
}
for (const name of linkedEvidence) {
  if (!evidenceNames.has(name)) failures.push(`linked evidence is absent from evidence.files: ${name}`);
}

const repositoryPattern = (manifest.source?.repository ?? "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const runUrlPattern = new RegExp(`^https://github\\.com/${repositoryPattern}/actions/runs/[0-9]+$`);
if (!runUrlPattern.test(manifest.workflow?.run_url ?? "")) failures.push("workflow must use an immutable repository run URL");
if (!(manifest.workflow?.identity ?? "").includes("/.github/workflows/supply-chain.yml@")) {
  failures.push("unexpected supply-chain workflow identity");
}
for (const [name, record] of Object.entries({ tests: manifest.evidence?.tests, audits: manifest.evidence?.audits })) {
  if (!runUrlPattern.test(record?.run_url ?? "")) failures.push(`${name} must use an immutable repository run URL`);
  if (record?.source_sha !== manifest.source?.sha) failures.push(`${name} source SHA does not match release source`);
}
const declaredProvenance = new Set(manifest.evidence?.provenance ?? []);
const artifactProvenance = new Set((manifest.artifacts ?? []).map(({ provenance }) => provenance));
for (const name of artifactProvenance) {
  if (!declaredProvenance.has(name)) failures.push(`artifact provenance absent from evidence.provenance: ${name}`);
}
for (const name of declaredProvenance) {
  if (!artifactProvenance.has(name)) failures.push(`unexpected evidence.provenance entry: ${name}`);
}

if (manifest.support_matrix !== "product-support-matrix.json") failures.push("unexpected support matrix reference");
else requireFile(manifest.support_matrix, "support matrix");
if (!evidenceNames.has(manifest.support_matrix)) failures.push("support matrix is absent from evidence.files");

if (manifest.promotion !== "promotion.json" || !requireFile(manifest.promotion, "promotion metadata")) {
  failures.push("unexpected promotion metadata reference");
} else {
  const promotion = JSON.parse(readFileSync(join(root, manifest.promotion), "utf8"));
  const expectedGa = (manifest.release ?? "").replace(/-rc\.[0-9]+$/, "");
  if (promotion.schema_version !== 1) failures.push("unsupported promotion schema_version");
  if (promotion.candidate !== manifest.release || promotion.ga !== expectedGa) failures.push("promotion version binding mismatch");
  if (promotion.source_sha !== manifest.source?.sha) failures.push("promotion source SHA mismatch");
  if (promotion.image !== `${manifest.image?.reference}@${manifest.image?.digest}`) failures.push("promotion image binding mismatch");
  if (promotion.rebuild !== false) failures.push("promotion must prohibit rebuilding");
  if (promotion.superseded_ga_sha !== null && !/^[0-9a-f]{40}$/.test(promotion.superseded_ga_sha ?? "")) {
    failures.push("invalid superseded GA SHA");
  }
}
if (!evidenceNames.has(manifest.promotion)) failures.push("promotion metadata is absent from evidence.files");

if (manifest.evidence?.checksums !== "SHA256SUMS") failures.push("unexpected checksum index reference");
const checksumPath = join(root, manifest.evidence?.checksums ?? "SHA256SUMS");
if (!existsSync(checksumPath)) {
  failures.push("missing final SHA256SUMS");
} else {
  const entries = new Map();
  for (const line of readFileSync(checksumPath, "utf8").trim().split("\n")) {
    const match = /^([0-9a-f]{64})  (.+)$/.exec(line);
    if (!match || !safeNamePattern.test(match[2])) {
      failures.push(`invalid SHA256SUMS line: ${line}`);
      continue;
    }
    if (entries.has(match[2])) failures.push(`duplicate SHA256SUMS entry: ${match[2]}`);
    entries.set(match[2], match[1]);
  }
  const downloadable = readdirSync(root)
    .filter((name) => name !== "SHA256SUMS" && statSync(join(root, name)).isFile())
    .sort();
  for (const name of downloadable) {
    if (entries.get(name) !== digest(join(root, name))) failures.push(`SHA256SUMS does not cover current file: ${name}`);
  }
  for (const name of entries.keys()) {
    if (!downloadable.includes(name)) failures.push(`SHA256SUMS contains unknown file: ${name}`);
  }
}

if (failures.length) {
  console.error(`Release manifest validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log(`Release manifest validation passed (${manifest.artifacts.length} artifacts, ${evidenceNames.size} evidence files).`);
