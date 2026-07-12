#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";

const [directory, output = "release-manifest.json"] = process.argv.slice(2);
if (!directory) {
  console.error("usage: generate-release-manifest.mjs <artifact-directory> [output]");
  process.exit(2);
}

const requiredEnvironment = ["RELEASE_TAG", "SOURCE_SHA", "REPOSITORY", "WORKFLOW_IDENTITY", "RUN_URL", "IMAGE_REFERENCE", "IMAGE_DIGEST"];
for (const name of requiredEnvironment) {
  if (!process.env[name]) throw new Error(`missing required environment variable: ${name}`);
}

const payloadPattern = /\.(?:tar\.gz|zip)$/;
const files = readdirSync(directory).filter((name) => payloadPattern.test(name)).sort();
if (files.length === 0) throw new Error(`no release archives found in ${directory}`);

const sha256 = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const artifacts = files.map((name) => ({
  name,
  sha256: sha256(join(directory, name)),
  sboms: [`${name}.cyclonedx.json`, `${name}.spdx.json`],
  signature: `${name}.sigstore.json`,
  provenance: `${name}.intoto.jsonl`
}));

const manifest = {
  schema_version: "1.0.0",
  release: process.env.RELEASE_TAG,
  source: { repository: process.env.REPOSITORY, sha: process.env.SOURCE_SHA },
  workflow: { identity: process.env.WORKFLOW_IDENTITY, run_url: process.env.RUN_URL },
  artifacts,
  image: {
    reference: process.env.IMAGE_REFERENCE,
    digest: process.env.IMAGE_DIGEST,
    platforms: ["linux/amd64", "linux/arm64"],
    sboms: ["image.cyclonedx.json", "image.spdx.json"],
    signature: `${process.env.IMAGE_REFERENCE}@${process.env.IMAGE_DIGEST}`,
    provenance: `${process.env.IMAGE_REFERENCE}@${process.env.IMAGE_DIGEST}#application/vnd.in-toto+json`
  },
  evidence: {
    checksums: "SHA256SUMS",
    provenance: artifacts.map(({ provenance }) => provenance),
    tests: `${process.env.RUN_URL}#artifacts-and-image`,
    audits: `https://github.com/${process.env.REPOSITORY}/actions/workflows/security-audit.yml`
  },
  support_matrix: "docs/product-support-matrix.json"
};

writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`wrote ${basename(output)} with ${artifacts.length} release artifacts`);
