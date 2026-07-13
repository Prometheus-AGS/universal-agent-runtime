#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";

const [directory, output = "release-manifest.json"] = process.argv.slice(2);
if (!directory) {
  console.error("usage: generate-release-manifest.mjs <artifact-directory> [output]");
  process.exit(2);
}

const requiredEnvironment = [
  "RELEASE_TAG",
  "SOURCE_SHA",
  "REPOSITORY",
  "WORKFLOW_IDENTITY",
  "RUN_URL",
  "TEST_RUN_URL",
  "SECURITY_AUDIT_RUN_URL",
  "GA_TAG",
  "SUPERSEDED_GA_SHA",
  "IMAGE_REFERENCE",
  "IMAGE_DIGEST",
];
for (const name of requiredEnvironment) {
  if (!process.env[name]) throw new Error(`missing required environment variable: ${name}`);
}

const payloadPattern = /\.(?:tar\.gz|zip)$/;
const files = readdirSync(directory).filter((name) => payloadPattern.test(name)).sort();
if (files.length === 0) throw new Error(`no release archives found in ${directory}`);

const sha256 = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const repositoryRoot = resolve(import.meta.dirname, "..");
const sourceCommit = execFileSync("git", ["rev-parse", "HEAD"], {
  cwd: repositoryRoot,
  encoding: "utf8",
}).trim();
if (sourceCommit !== process.env.SOURCE_SHA) {
  throw new Error(`SOURCE_SHA ${process.env.SOURCE_SHA} does not match checked-out commit ${sourceCommit}`);
}

const modelDirectory = join(repositoryRoot, "src/uar/runtime/matching/models");
const modelInputs = readdirSync(modelDirectory).sort();
const modelBundleHash = createHash("sha256");
for (const name of modelInputs) {
  modelBundleHash.update(name);
  modelBundleHash.update("\0");
  modelBundleHash.update(readFileSync(join(modelDirectory, name)));
}
const sourceTree = execFileSync("git", ["rev-parse", "HEAD^{tree}"], {
  cwd: repositoryRoot,
  encoding: "utf8",
}).trim();

function archiveBinary(archiveName) {
  const archivePath = join(directory, archiveName);
  const zip = archiveName.endsWith(".zip");
  const listing = execFileSync(zip ? "unzip" : "tar", zip ? ["-Z1", archivePath] : ["-tzf", archivePath], {
    encoding: "utf8",
  });
  const executable = zip ? "universal-agent-runtime.exe" : "universal-agent-runtime";
  const matches = listing
    .split("\n")
    .filter(Boolean)
    .filter((entry) => basename(entry) === executable && !entry.startsWith("/") && !entry.split("/").includes(".."));
  if (matches.length !== 1) {
    throw new Error(`${archiveName} must contain exactly one ${executable}; found ${matches.length}`);
  }
  const contents = execFileSync(zip ? "unzip" : "tar", zip ? ["-p", archivePath, matches[0]] : ["-xOzf", archivePath, matches[0]], {
    maxBuffer: 512 * 1024 * 1024,
  });
  return { path: matches[0], sha256: createHash("sha256").update(contents).digest("hex") };
}

const artifacts = files.map((name) => {
  const common = {
    name,
    sha256: sha256(join(directory, name)),
    signature: `${name}.sigstore.json`,
    provenance: `${name}.intoto.jsonl`,
  };
  if (name === "uar-offline-source.tar.gz") {
    return {
      ...common,
      kind: "offline-source",
      sboms: [`${name}.cyclonedx.json`, `${name}.spdx.json`],
    };
  }
  return {
    ...common,
    kind: "platform-archive",
    binary: archiveBinary(name),
    sboms: [
      `${name}.cyclonedx.json`,
      `${name}.spdx.json`,
      `${name}.binary.cyclonedx.json`,
      `${name}.binary.spdx.json`,
    ],
  };
});

if (!/^v\d+\.\d+\.\d+$/.test(process.env.GA_TAG)) throw new Error("GA_TAG must be a stable v-prefixed semantic version");
const supersededGaSha = process.env.SUPERSEDED_GA_SHA === "null" ? null : process.env.SUPERSEDED_GA_SHA;
if (supersededGaSha !== null && !/^[0-9a-f]{40}$/.test(supersededGaSha)) {
  throw new Error("SUPERSEDED_GA_SHA must be null or a full commit SHA");
}
const promotion = {
  schema_version: 1,
  candidate: process.env.RELEASE_TAG,
  ga: process.env.GA_TAG,
  source_sha: process.env.SOURCE_SHA,
  image: `${process.env.IMAGE_REFERENCE}@${process.env.IMAGE_DIGEST}`,
  rebuild: false,
  superseded_ga_sha: supersededGaSha,
};
writeFileSync(join(directory, "promotion.json"), `${JSON.stringify(promotion, null, 2)}\n`);

const outputName = basename(output);
const evidenceFiles = readdirSync(directory)
  .filter((name) => name !== outputName && name !== "SHA256SUMS" && !payloadPattern.test(name))
  .filter((name) => statSync(join(directory, name)).isFile())
  .sort()
  .map((name) => ({ name, sha256: sha256(join(directory, name)) }));

const manifest = {
  schema_version: "1.0.0",
  release: process.env.RELEASE_TAG,
  source: {
    repository: process.env.REPOSITORY,
    sha: process.env.SOURCE_SHA,
    git_tree: sourceTree,
    cargo_lock_sha256: sha256(join(repositoryRoot, "Cargo.lock")),
    catalog_sha256: sha256(join(repositoryRoot, "catalog/provider_catalog.json")),
    model_bundle_sha256: modelBundleHash.digest("hex"),
    model_inputs: modelInputs,
    sboms: ["source.cyclonedx.json", "source.spdx.json"],
  },
  workflow: { identity: process.env.WORKFLOW_IDENTITY, run_url: process.env.RUN_URL },
  artifacts,
  image: {
    reference: process.env.IMAGE_REFERENCE,
    digest: process.env.IMAGE_DIGEST,
    platforms: ["linux/amd64", "linux/arm64"],
    sboms: ["image.cyclonedx.json", "image.spdx.json"],
    signature: `${process.env.IMAGE_REFERENCE}@${process.env.IMAGE_DIGEST}`,
    provenance: `${process.env.IMAGE_REFERENCE}@${process.env.IMAGE_DIGEST}#application/vnd.in-toto+json`,
  },
  evidence: {
    checksums: "SHA256SUMS",
    files: evidenceFiles,
    provenance: artifacts.map(({ provenance }) => provenance),
    tests: { run_url: process.env.TEST_RUN_URL, source_sha: process.env.SOURCE_SHA },
    audits: { run_url: process.env.SECURITY_AUDIT_RUN_URL, source_sha: process.env.SOURCE_SHA },
  },
  promotion: "promotion.json",
  support_matrix: "product-support-matrix.json",
};

writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`wrote ${outputName} with ${artifacts.length} release artifacts and ${evidenceFiles.length} evidence files`);
