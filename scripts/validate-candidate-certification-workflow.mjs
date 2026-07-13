#!/usr/bin/env node

import { readFileSync } from "node:fs";

const workflow = readFileSync(new URL("../.github/workflows/candidate-certification.yml", import.meta.url), "utf8");
const failures = [];
const required = [
  "gh attestation verify candidate-assets/SHA256SUMS",
  "--signer-workflow \"${GITHUB_REPOSITORY}/.github/workflows/supply-chain.yml\"",
  "sha256sum --check SHA256SUMS",
  "node candidate-assets/verify-release.mjs candidate-assets/release-manifest.json",
  "candidate-certification-manifest.json",
  "CANDIDATE_CERTIFICATION_SHA256SUMS",
  "subject-path: certification-assets/CANDIDATE_CERTIFICATION_SHA256SUMS",
  "node scripts/validate-candidate-certification-bundle.mjs",
  "certification-assets/*",
];
for (const contract of required) {
  if (!workflow.includes(contract)) failures.push(`missing candidate certification workflow contract: ${contract}`);
}
if (workflow.includes("candidate-certification-${{ inputs.tag }}.sha256")) {
  failures.push("legacy unauthenticated candidate certification checksum is prohibited");
}
if (failures.length) {
  console.error(`Candidate certification workflow validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Candidate certification authentication and publication contracts passed.");
