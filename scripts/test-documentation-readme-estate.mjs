#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { validateReadmeEstate } from "./validate-documentation-readme-estate.mjs";

const write = (root, path, body) => {
  mkdirSync(join(root, path, ".."), { recursive: true });
  writeFileSync(join(root, path), body);
};
const writeJson = (root, path, value) => write(root, path, `${JSON.stringify(value, null, 2)}\n`);
const hash = (body) => createHash("sha256").update(body).digest("hex");
const portal = "https://prometheus-ags.github.io/universal-agent-runtime/docs/intro";
const hero = `<img src="website/static/img/brand/uar-wordmark-dark.svg" alt="Universal Agent Runtime" />

# Universal Agent Runtime

Governed execution. Typed protocols. One runtime boundary.

[![License](https://img.shields.io/badge/license-MIT-blue)](#)
[![Version](https://img.shields.io/badge/version-1.0.0-orange)](#)
[![Documentation](https://img.shields.io/badge/docs-portal-cyan)](${portal})
`;

function fixture() {
  const root = mkdtempSync(join(tmpdir(), "uar-readme-estate-"));
  const source = "# Generated source\n";
  const vendor = "# Upstream vendor\n";
  write(root, "README.md", hero);
  write(root, "docs/current/README.md", "> **Current authority:** [Guide](https://example.test/docs/product/chat)\n\n# Current package\n");
  write(root, "docs/history/README.md", "> **Historical — superseded 2026-08-23.** See [current](https://example.test/docs/product/chat).\n\n# Historical record\n");
  write(root, "mirrors/README.md", source);
  write(root, "source/README.md", source);
  write(root, "vendor/README.md", vendor);
  write(root, "website/docs/intro.md", "---\nid: intro\n---\n# Intro\n");
  write(root, "website/docs/product/chat.md", "---\nid: product/chat\n---\n# Chat\n");
  writeJson(root, "docs/publication/routes.json", {
    schemaVersion: 1,
    routes: [{ id: "chat", status: "required", documentId: "product/chat", route: "/docs/product/chat" }],
  });
  writeJson(root, "docs/publication/readme-estate.json", {
    schemaVersion: 1,
    canonicalPortal: portal,
    entries: [
      { path: "README.md", kind: "root", status: "current", owner: "UAR", action: "reconcile", authority: "/docs/intro", profiles: ["server-full"] },
      { path: "docs/current/README.md", kind: "uar-owned", status: "current", owner: "current", action: "reconcile", authority: "/docs/product/chat", profiles: ["server-full"] },
      { path: "docs/history/README.md", kind: "uar-owned", status: "historical", owner: "history", action: "preserve-with-banner", authority: "/docs/product/chat", profiles: [] },
      { path: "mirrors/README.md", kind: "generated-mirror", status: "current", owner: "generator", action: "regenerate", authority: "source/README.md", generatedFrom: "source/README.md", profiles: [] },
      { path: "source/README.md", kind: "uar-owned", status: "current", owner: "source", action: "reconcile", authority: "/docs/intro", profiles: [] },
      { path: "vendor/README.md", kind: "vendored", status: "historical", owner: "upstream", action: "exclude", authority: "upstream-vendor", sha256: hash(vendor), profiles: [] },
    ],
  });
  write(root, "source/README.md", `> **Current authority:** [Guide](https://example.test/docs/intro)\n\n${source}`);
  write(root, "mirrors/README.md", readFileSync(join(root, "source/README.md")));
  return {
    root,
    tracked: ["README.md", "docs/current/README.md", "docs/history/README.md", "mirrors/README.md", "source/README.md", "vendor/README.md"],
  };
}

function mutate(name, operation, expected) {
  const state = fixture();
  try {
    operation(state);
    const result = validateReadmeEstate({ root: state.root, trackedReadmes: state.tracked });
    if (!result.failures.some((failure) => failure.includes(expected))) {
      throw new Error(`${name}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(result.failures)}`);
    }
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(state.root, { recursive: true, force: true });
  }
}

const readManifest = (state) => JSON.parse(readFileSync(join(state.root, "docs/publication/readme-estate.json"), "utf8"));
const saveManifest = (state, manifest) => writeJson(state.root, "docs/publication/readme-estate.json", manifest);

mutate("denominator drift", (state) => state.tracked.push("new/README.md"), "README ownership missing");
mutate("duplicate ownership", (state) => {
  const manifest = readManifest(state);
  manifest.entries.push({ ...manifest.entries[1] });
  saveManifest(state, manifest);
}, "duplicate entry");
mutate("missing authority", (state) => {
  const manifest = readManifest(state);
  delete manifest.entries[1].authority;
  saveManifest(state, manifest);
}, "current authority is required");
mutate("stale current claim", (state) => write(state.root, "docs/current/README.md", "> **Current authority:** [Guide](https://example.test/docs/product/chat)\n\nPlaceholder\n"), "placeholder content");
mutate("historical banner", (state) => write(state.root, "docs/history/README.md", "# Historical record\n/docs/product/chat\n"), "dated historical banner is missing");
mutate("generated mirror drift", (state) => write(state.root, "mirrors/README.md", "drift\n"), "generated mirror differs");
mutate("vendored mutation", (state) => write(state.root, "vendor/README.md", "changed\n"), "vendored README hash changed");
mutate("root hero", (state) => write(state.root, "README.md", "# Universal Agent Runtime\n"), "existing UAR wordmark is missing");
mutate("root badges", (state) => write(state.root, "README.md", hero.replace("https://img.shields.io/badge/docs-portal-cyan", "https://example.test/docs-badge")), "license, version, and documentation badges are required");
mutate("root portal link", (state) => write(state.root, "README.md", hero.replace(portal, "https://example.test/docs")), "canonical portal link is missing");
mutate("missing frozen route", (state) => rmSync(join(state.root, "website/docs/product/chat.md")), "required document is missing");
mutate("unsafe public content", (state) => write(state.root, "docs/current/README.md", "> **Current authority:** [Guide](https://example.test/docs/product/chat)\n\n/Users/example/private\n"), "machine-local macOS path");
mutate("cross-profile transfer", (state) => write(state.root, "docs/current/README.md", "> **Current authority:** [Guide](https://example.test/docs/product/chat)\n\nserver-full and minimal run identically\n"), "cross-profile equivalence claim");
mutate("routine GitHub tests", (state) => write(state.root, "docs/current/README.md", "> **Current authority:** [Guide](https://example.test/docs/product/chat)\n\nGitHub Actions run unit tests for every build.\n"), "routine GitHub Actions testing claim");

const valid = fixture();
try {
  const result = validateReadmeEstate({ root: valid.root, trackedReadmes: valid.tracked });
  if (result.failures.length) throw new Error(`complete fixture failed: ${JSON.stringify(result.failures)}`);
  console.log("PASS positive control: complete README estate");
} finally {
  rmSync(valid.root, { recursive: true, force: true });
}
