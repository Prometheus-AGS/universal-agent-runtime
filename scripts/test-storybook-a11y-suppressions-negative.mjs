#!/usr/bin/env node

import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = resolve(import.meta.dirname, "..");
const fixtureRoot = resolve(repoRoot, "scripts/fixtures/storybook-a11y-suppression");
const result = spawnSync(
  process.execPath,
  [resolve(repoRoot, "scripts/check-storybook-a11y-suppressions.mjs"), "--root", fixtureRoot],
  { cwd: repoRoot, encoding: "utf8" },
);

const expectedFixtures = [
  "disabled.stories.tsx",
  "disabled-assignment.stories.tsx",
  "disabled-computed.stories.tsx",
  "disabled-double-quoted.stories.tsx",
  "disabled-single-quoted.stories.tsx",
];

if (result.status === 0 || expectedFixtures.some((fixture) => !result.stderr.includes(fixture))) {
  process.stderr.write(result.stdout);
  process.stderr.write(result.stderr);
  throw new Error("Storybook accessibility negative fixture was not rejected");
}

console.log(`Storybook accessibility negative fixtures passed (${expectedFixtures.length} syntax forms rejected).`);
