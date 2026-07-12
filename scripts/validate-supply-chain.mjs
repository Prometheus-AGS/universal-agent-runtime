#!/usr/bin/env node

import { readFileSync } from "node:fs";

const workflow = readFileSync(new URL("../.github/workflows/supply-chain.yml", import.meta.url), "utf8");
const schema = JSON.parse(readFileSync(new URL("../schemas/release-manifest.schema.json", import.meta.url), "utf8"));
const failures = [];

const requiredWorkflowContracts = [
  "permissions:\n  contents: read",
  "id-token: write",
  "packages: write",
  "linux/amd64,linux/arm64",
  "anchore/sbom-action/download-syft@v0",
  "sigstore/cosign-installer@v4.1.2",
  "cosign sign --yes",
  "cosign sign-blob --yes",
  "actions/attest@v4",
  "subject-checksums: evidence/SHA256SUMS",
  "cosign verify-blob",
  "cosign verify",
  "gh attestation verify",
  "--user 65532:65532",
  "node scripts/generate-release-manifest.mjs",
  "node scripts/validate-release-manifest.mjs",
  "softprops/action-gh-release@v2"
];
for (const value of requiredWorkflowContracts) {
  if (!workflow.includes(value)) failures.push(`missing workflow contract: ${value}`);
}

for (const permission of ["actions: write", "security-events: write"]) {
  if (workflow.includes(permission)) failures.push(`overbroad workflow permission: ${permission}`);
}

for (const property of ["schema_version", "release", "source", "workflow", "artifacts", "image", "evidence", "support_matrix"]) {
  if (!schema.required.includes(property)) failures.push(`schema does not require ${property}`);
}

if (failures.length) {
  console.error(`Supply-chain validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Supply-chain workflow and release-manifest schema validation passed.");
