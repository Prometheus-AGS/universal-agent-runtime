#!/usr/bin/env node

import { readFileSync } from "node:fs";

const producer = readFileSync(new URL("./prepare-release-evidence-local.sh", import.meta.url), "utf8");
const securityAudit = readFileSync(new URL("./security-audit-local.sh", import.meta.url), "utf8");
const provenance = readFileSync(new URL("./generate-local-provenance.mjs", import.meta.url), "utf8");
const imageProvenance = readFileSync(new URL("./generate-local-image-provenance.mjs", import.meta.url), "utf8");
const generator = readFileSync(new URL("./generate-release-manifest.mjs", import.meta.url), "utf8");
const validator = readFileSync(new URL("./validate-release-manifest.mjs", import.meta.url), "utf8");
const promoter = readFileSync(new URL("./promote-release-candidate.sh", import.meta.url), "utf8");
const schema = JSON.parse(readFileSync(new URL("../schemas/release-manifest.schema.json", import.meta.url), "utf8"));
const failures = [];

const requiredProducerContracts = [
  "checkout must be clean",
  "submodule status --recursive",
  "package-offline-source.sh",
  "product-support-matrix.json",
  "source.cyclonedx.json",
  "source.spdx.json",
  "image.cyclonedx.json",
  "image.spdx.json",
  ".binary.cyclonedx.json",
  ".binary.spdx.json",
  "generate-local-provenance.mjs",
  "generate-local-image-provenance.mjs",
  "cosign sign-blob --yes",
  "cosign sign --yes",
  "cosign attest --yes --type slsaprovenance",
  "cosign attest --yes --type cyclonedx",
  "generate-release-manifest.mjs",
  "verify-release.mjs",
  "No tag, release, archive, or image was built, uploaded, or promoted",
];
for (const value of requiredProducerContracts) {
  if (!producer.includes(value)) failures.push(`missing local evidence-producer contract: ${value}`);
}

for (const value of ["IMAGE_DIGEST", "resolvedDependencies", "gitCommit", "runDetails", "builder"]) {
  if (!imageProvenance.includes(value)) failures.push(`missing local image-provenance contract: ${value}`);
}
for (const prohibited of ["github.run_id", "github.workflow_ref", "actions/runs/", ".github/workflows/"]) {
  if (producer.includes(prohibited) || securityAudit.includes(prohibited) || generator.includes(prohibited) || validator.includes(prohibited) || promoter.includes(prohibited)) {
    failures.push(`release evidence must not depend on GitHub Actions identity: ${prohibited}`);
  }
}

for (const value of [
  "UAR_RELEASE_SIGNING_IDENTITY",
  "UAR_RELEASE_SIGNING_OIDC_ISSUER",
  "cosign verify-blob",
  "cosign verify",
  "cosign verify-attestation",
  "release manifest signing identity does not match operator policy",
  "release manifest signing issuer does not match operator policy",
]) {
  if (!promoter.includes(value)) failures.push(`missing local promotion verification contract: ${value}`);
}

for (const value of [
  "cargo audit",
  "pnpm-root-audit",
  "pnpm-frontend-audit",
  "npm-typescript-sdk-audit",
  "osv-scanner --recursive --skip-git",
  "grype \"$image\" --fail-on high",
  "dependabot/alerts",
  "no inline allowlist exists",
  "security-audit-evidence.json",
]) {
  if (!securityAudit.includes(value)) failures.push(`missing local security-audit contract: ${value}`);
}

for (const value of [
  "https://in-toto.io/Statement/v1",
  "https://slsa.dev/provenance/v1",
  "SOURCE_SHA",
  "BUILDER_IDENTITY",
  "resolvedDependencies",
  "gitCommit",
]) {
  if (!provenance.includes(value)) failures.push(`missing local provenance contract: ${value}`);
}

for (const value of [
  "BUILDER_IDENTITY",
  "BUILD_RECEIPT",
  "TEST_EVIDENCE",
  "SECURITY_AUDIT_EVIDENCE",
  "COSIGN_CERTIFICATE_IDENTITY",
  "COSIGN_CERTIFICATE_OIDC_ISSUER",
  "source_sha",
  "archiveBinary",
  "source.cyclonedx.json",
  "product-support-matrix.json",
  "promotion.json",
  'checksums: "SHA256SUMS"',
]) {
  if (!generator.includes(value)) failures.push(`missing manifest generator contract: ${value}`);
}

for (const value of [
  "release evidence builder must be local",
  "builder receipt digest mismatch",
  "source SHA does not match release source",
  "embedded binary trace mismatch",
  "provenance subject mismatch",
  "provenance source SHA mismatch",
  "provenance builder identity mismatch",
  "image provenance source SHA mismatch",
  "image provenance builder identity mismatch",
  "image provenance digest reference mismatch",
  "image signature reference mismatch",
  "SHA256SUMS does not cover current file",
  "missing SHA256SUMS signature bundle",
  "linked evidence is absent from evidence.files",
  "offline source artifact must not claim an embedded runtime binary",
]) {
  if (!validator.includes(value)) failures.push(`missing manifest validator contract: ${value}`);
}

for (const property of ["schema_version", "release", "source", "builder", "signing", "artifacts", "image", "evidence", "promotion", "support_matrix"]) {
  if (!schema.required.includes(property)) failures.push(`schema does not require ${property}`);
}
for (const property of ["kind", "identity", "source_sha", "receipt", "receipt_sha256"]) {
  if (!schema.properties.builder.required.includes(property)) failures.push(`schema builder does not require ${property}`);
}
for (const property of ["certificate_identity", "certificate_oidc_issuer"]) {
  if (!schema.properties.signing.required.includes(property)) failures.push(`schema signing does not require ${property}`);
}
for (const property of ["checksums", "files", "provenance", "tests", "audits"]) {
  if (!schema.properties.evidence.required.includes(property)) failures.push(`schema evidence does not require ${property}`);
}
if (schema.properties.support_matrix.const !== "product-support-matrix.json") {
  failures.push("schema must require the downloadable support matrix");
}

if (failures.length) {
  console.error(`Supply-chain validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Local supply-chain producer, provenance, manifest, validator, and schema contracts passed.");
