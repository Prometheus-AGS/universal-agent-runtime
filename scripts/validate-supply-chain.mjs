#!/usr/bin/env node

import { readFileSync } from "node:fs";

const workflow = readFileSync(new URL("../.github/workflows/supply-chain.yml", import.meta.url), "utf8");
const ciWorkflow = readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
const generator = readFileSync(new URL("./generate-release-manifest.mjs", import.meta.url), "utf8");
const validator = readFileSync(new URL("./validate-release-manifest.mjs", import.meta.url), "utf8");
const schema = JSON.parse(readFileSync(new URL("../schemas/release-manifest.schema.json", import.meta.url), "utf8"));
const failures = [];

const requiredWorkflowContracts = [
  "permissions:\n  contents: read",
  "id-token: write",
  "packages: write",
  "attestations: read",
  "test_run_url:",
  "security_audit_run_url:",
  "existing_ga_sha",
  "actions/runs/${run_id}",
  ".github/workflows/ci.yml",
  ".github/workflows/security-audit.yml",
  "--jq .path",
  "linux/amd64,linux/arm64",
  "anchore/sbom-action/download-syft@v0",
  ".binary.cyclonedx.json",
  ".binary.spdx.json",
  "uar-offline-source.tar.gz",
  "source.cyclonedx.json",
  "source.spdx.json",
  "product-support-matrix.json",
  "sigstore/cosign-installer@v4.1.2",
  "cosign sign --yes",
  "cosign sign-blob --yes",
  "actions/attest@v4",
  "subject-checksums: evidence/PAYLOAD_SHA256SUMS",
  "subject-path: evidence/SHA256SUMS",
  "find . -maxdepth 1 -type f ! -name SHA256SUMS",
  "cosign verify-blob",
  "cosign verify",
  "gh attestation verify evidence/SHA256SUMS",
  "--user 65532:65532",
  "node scripts/generate-release-manifest.mjs",
  "node evidence/verify-release.mjs",
  "cp scripts/validate-release-manifest.mjs evidence/verify-release.mjs",
  "softprops/action-gh-release@v2",
];
for (const value of requiredWorkflowContracts) {
  if (!workflow.includes(value)) failures.push(`missing workflow contract: ${value}`);
}

for (const permission of ["actions: write", "security-events: write"]) {
  if (workflow.includes(permission)) failures.push(`overbroad workflow permission: ${permission}`);
}

for (const command of [
  "cargo clippy --locked --no-default-features --lib --features server-full --no-deps",
  "cargo check --locked --no-default-features --features server-full",
]) {
  if (!ciWorkflow.includes(command)) failures.push(`CI does not enforce authoritative command: ${command}`);
}

const requiredGeneratorContracts = [
  "TEST_RUN_URL",
  "SECURITY_AUDIT_RUN_URL",
  "SUPERSEDED_GA_SHA",
  "SOURCE_SHA",
  "archiveBinary",
  "source.cyclonedx.json",
  "product-support-matrix.json",
  "promotion.json",
  'checksums: "SHA256SUMS"',
];
for (const value of requiredGeneratorContracts) {
  if (!generator.includes(value)) failures.push(`missing manifest generator contract: ${value}`);
}

const requiredValidatorContracts = [
  "embedded binary trace mismatch",
  "source SHA does not match release source",
  "SHA256SUMS does not cover current file",
  "linked evidence is absent from evidence.files",
  "offline source artifact must not claim an embedded runtime binary",
];
for (const value of requiredValidatorContracts) {
  if (!validator.includes(value)) failures.push(`missing manifest validator contract: ${value}`);
}

for (const property of ["schema_version", "release", "source", "workflow", "artifacts", "image", "evidence", "promotion", "support_matrix"]) {
  if (!schema.required.includes(property)) failures.push(`schema does not require ${property}`);
}
for (const property of ["repository", "sha", "git_tree", "cargo_lock_sha256", "catalog_sha256", "model_bundle_sha256", "model_inputs", "sboms"]) {
  if (!schema.properties.source.required.includes(property)) failures.push(`schema source does not require ${property}`);
}
for (const property of ["checksums", "files", "provenance", "tests", "audits"]) {
  if (!schema.properties.evidence.required.includes(property)) failures.push(`schema evidence does not require ${property}`);
}
for (const property of ["kind", "name", "sha256", "binary", "sboms", "signature", "provenance"]) {
  if (!schema.$defs.platformArtifact.required.includes(property)) failures.push(`schema platform artifact does not require ${property}`);
}
for (const property of ["kind", "name", "sha256", "sboms", "signature", "provenance"]) {
  if (!schema.$defs.offlineSourceArtifact.required.includes(property)) failures.push(`schema offline source artifact does not require ${property}`);
}
if (schema.properties.support_matrix.const !== "product-support-matrix.json") {
  failures.push("schema must require the downloadable support matrix");
}

if (failures.length) {
  console.error(`Supply-chain validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Supply-chain workflow, manifest generator, validator, and schema contracts passed.");
