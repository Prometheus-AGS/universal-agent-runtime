#!/usr/bin/env node

import { readFileSync } from "node:fs";

// Kept under its historical filename so existing package scripts remain
// compatible. The contract is now deliberately local; no workflow is read.
const certifier = readFileSync(new URL("./certify-release-candidate.sh", import.meta.url), "utf8");
const packager = readFileSync(new URL("./package-candidate-certification-local.sh", import.meta.url), "utf8");
const bundleValidator = readFileSync(new URL("./validate-candidate-certification-bundle.mjs", import.meta.url), "utf8");
const failures = [];

for (const contract of [
  "release-manifest.json",
  "candidate manifest/source SHA",
  "UAR_SOAK_DURATION_SECONDS",
]) {
  if (!certifier.includes(contract)) failures.push(`missing installed candidate certifier contract: ${contract}`);
}
for (const contract of [
  "checkout must be clean",
  "validate-release-manifest.mjs",
  "validate-candidate-certification.mjs",
  "candidate-certification-manifest.json",
  "CANDIDATE_CERTIFICATION_SHA256SUMS",
  "cosign sign-blob --yes",
  "No candidate tag, release, archive, or image was built, uploaded, or promoted",
]) {
  if (!packager.includes(contract)) failures.push(`missing local candidate packager contract: ${contract}`);
}
for (const contract of [
  "candidate certification builder must be local",
  "candidate certification builder receipt digest mismatch",
  "missing candidate certification checksum signature bundle",
  "candidate certification asset set is not exact",
]) {
  if (!bundleValidator.includes(contract)) failures.push(`missing candidate bundle validator contract: ${contract}`);
}
for (const prohibited of ["actions/runs/", ".github/workflows/", "signer-workflow", "github.run_id", "github.workflow_ref"]) {
  if (certifier.includes(prohibited) || packager.includes(prohibited) || bundleValidator.includes(prohibited)) {
    failures.push(`candidate certification must not depend on GitHub Actions: ${prohibited}`);
  }
}

if (failures.length) {
  console.error(`Local candidate certification validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log("Local installed-candidate certification, packaging, and bundle contracts passed.");
