#!/usr/bin/env node

import { writeFileSync } from "node:fs";

const [outputPath] = process.argv.slice(2);
if (!outputPath) {
  console.error("usage: generate-local-image-provenance.mjs <output.json>");
  process.exit(2);
}
for (const name of ["SOURCE_SHA", "REPOSITORY", "BUILDER_IDENTITY", "RELEASE_TAG", "IMAGE_REFERENCE", "IMAGE_DIGEST", "BUILD_STARTED_AT", "BUILD_FINISHED_AT"]) {
  if (!process.env[name]) throw new Error(`missing required environment variable: ${name}`);
}
if (!/^[0-9a-f]{40}$/.test(process.env.SOURCE_SHA)) throw new Error("SOURCE_SHA must be a full commit SHA");
if (!/^sha256:[0-9a-f]{64}$/.test(process.env.IMAGE_DIGEST)) throw new Error("IMAGE_DIGEST must be a sha256 digest");
for (const name of ["BUILD_STARTED_AT", "BUILD_FINISHED_AT"]) {
  if (Number.isNaN(Date.parse(process.env[name]))) throw new Error(`${name} must be an ISO-8601 timestamp`);
}

const predicate = {
  buildDefinition: {
    buildType: "https://github.com/Prometheus-AGS/universal-agent-runtime/local-release-image/v1",
    externalParameters: {
      release: process.env.RELEASE_TAG,
      image: `${process.env.IMAGE_REFERENCE}@${process.env.IMAGE_DIGEST}`,
    },
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
      invocationId: `${process.env.RELEASE_TAG}:${process.env.IMAGE_DIGEST}`,
      startedOn: process.env.BUILD_STARTED_AT,
      finishedOn: process.env.BUILD_FINISHED_AT,
    },
  },
};

writeFileSync(outputPath, `${JSON.stringify(predicate)}\n`);
console.log(`wrote local image provenance for ${process.env.IMAGE_REFERENCE}@${process.env.IMAGE_DIGEST}`);
