#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { basename } from "node:path";

const [subjectPath, outputPath] = process.argv.slice(2);
if (!subjectPath || !outputPath) {
  console.error("usage: generate-local-provenance.mjs <subject> <output.intoto.jsonl>");
  process.exit(2);
}

for (const name of ["SOURCE_SHA", "REPOSITORY", "BUILDER_IDENTITY", "RELEASE_TAG", "BUILD_STARTED_AT", "BUILD_FINISHED_AT"]) {
  if (!process.env[name]) throw new Error(`missing required environment variable: ${name}`);
}
if (!/^[0-9a-f]{40}$/.test(process.env.SOURCE_SHA)) throw new Error("SOURCE_SHA must be a full commit SHA");
for (const name of ["BUILD_STARTED_AT", "BUILD_FINISHED_AT"]) {
  if (Number.isNaN(Date.parse(process.env[name]))) throw new Error(`${name} must be an ISO-8601 timestamp`);
}

const sha256 = createHash("sha256").update(readFileSync(subjectPath)).digest("hex");
const statement = {
  _type: "https://in-toto.io/Statement/v1",
  subject: [{ name: basename(subjectPath), digest: { sha256 } }],
  predicateType: "https://slsa.dev/provenance/v1",
  predicate: {
    buildDefinition: {
      buildType: "https://github.com/Prometheus-AGS/universal-agent-runtime/local-release-evidence/v1",
      externalParameters: { release: process.env.RELEASE_TAG },
      internalParameters: {},
      resolvedDependencies: [
        {
          uri: `git+https://github.com/${process.env.REPOSITORY}.git@${process.env.SOURCE_SHA}`,
          digest: { gitCommit: process.env.SOURCE_SHA },
        },
      ],
    },
    runDetails: {
      builder: { id: process.env.BUILDER_IDENTITY },
      metadata: {
        invocationId: `${process.env.RELEASE_TAG}:${sha256}`,
        startedOn: process.env.BUILD_STARTED_AT,
        finishedOn: process.env.BUILD_FINISHED_AT,
      },
    },
  },
};

writeFileSync(outputPath, `${JSON.stringify(statement)}\n`);
console.log(`wrote local SLSA provenance for ${basename(subjectPath)} (${sha256})`);
