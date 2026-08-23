#!/usr/bin/env node

import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { validateDocumentationPublication } from "./validate-documentation-publication.mjs";
import { validateGitHubActionsPolicy } from "./validate-github-actions-policy.mjs";

const write = (root, path, body) => {
  mkdirSync(join(root, path, ".."), { recursive: true });
  writeFileSync(join(root, path), body);
};
const readJson = (root, path) => JSON.parse(readFileSync(join(root, path), "utf8"));
const writeJson = (root, path, value) => write(root, path, `${JSON.stringify(value, null, 2)}\n`);

function fixture() {
  const root = mkdtempSync(join(tmpdir(), "uar-doc-publication-"));
  const trackedPaths = [
    ".prometheus/decision.md",
    "README.md",
    "docs/product-surface-inventory.md",
    "docs/publication/README.md",
    "docs/publication/routes.json",
    "docs/publication/sources.json",
    "openspec/specs/example/spec.md",
    "website/docs/product/chat.md",
  ];

  write(root, "README.md", "# Fixture\n");
  write(root, "docs/publication/README.md", "# Contract\n");
  write(root, "docs/product-surface-inventory.md", "| Route or surface | Owner |\n|---|---|\n| `/threads` chat | chat |\n");
  write(root, ".prometheus/decision.md", "Private decision evidence that is cited but not copied.\n");
  write(root, "openspec/specs/example/spec.md", "# Current specification\n");
  write(root, "website/docs/product/chat.md", "---\nsource_records:\n  - .prometheus/decision.md\ncurrent_authority: /docs/product/chat\n---\n# Chat\n");
  write(root, ".github/workflows/deploy.yml", "jobs:\n  deploy:\n    steps:\n      - run: kubectl set image x && kubectl rollout status x && curl /readyz && curl /healthz\n");
  write(root, ".github/workflows/docs.yml", `jobs:
  build:
    steps:
      - run: npm --prefix website ci
      - run: npm --prefix website run build
      - run: cargo doc --locked --no-deps --workspace --features server-full
      - run: npm --prefix sdks/typescript ci
      - run: npm --prefix sdks/typescript run docs
      - run: node scripts/stage-documentation-references.mjs
      - uses: actions/upload-pages-artifact@v5
  deploy:
    steps:
      - uses: actions/deploy-pages@v5
      - run: |
          echo steps.deployment.outputs.page_url
          curl --fail /docs/architecture/intro
          curl --fail /docs/api/rust/
          curl --fail /docs/api/typescript/
`);
  writeJson(root, "website/package.json", { scripts: { build: "npm run copy:adr && docusaurus build" } });
  write(root, "website/package-lock.json", "{}\n");
  writeJson(root, "sdks/typescript/package.json", { scripts: { docs: "typedoc" } });
  write(root, "sdks/typescript/package-lock.json", "{}\n");
  write(root, "scripts/stage-documentation-references.mjs", "process.exit(0);\n");
  write(root, "scripts/validate-documentation-truth.mjs", "process.exit(0);\n");
  write(root, "scripts/validate-documentation-architecture.mjs", "process.exit(0);\n");
  write(root, "scripts/validate-github-actions-policy.mjs", "process.exit(0);\n");

  writeJson(root, "docs/publication/sources.json", {
    schemaVersion: 1,
    trackedSelectors: { basenames: ["README.md"], prefixes: ["docs/", "website/", ".prometheus/", "openspec/"] },
    rules: [
      { id: "root", selector: { paths: ["README.md"] }, disposition: "public", owner: "fixture", status: "current", publicationMode: "direct", canonicalAuthority: "website/docs/product/chat.md", publicDestination: "/" },
      { id: "docs", selector: { prefixes: ["docs/"] }, disposition: "public-normalize", owner: "fixture", status: "current", publicationMode: "normalize", canonicalAuthority: "docs/publication/routes.json", publicDestination: "/docs" },
      { id: "website", selector: { prefixes: ["website/"] }, disposition: "public", owner: "fixture", status: "current", publicationMode: "direct", canonicalAuthority: "docs/publication/routes.json", publicDestination: "/" },
      { id: "private", selector: { prefixes: [".prometheus/"] }, disposition: "private-synthesis-only", owner: "fixture", status: "historical", publicationMode: "synthesis", canonicalAuthority: ".prometheus/decision.md", rationale: "Private evidence." },
      { id: "specs", selector: { prefixes: ["openspec/"] }, disposition: "public-normalize", owner: "fixture", status: "current", publicationMode: "synthesis", canonicalAuthority: "openspec/specs/example/spec.md", publicDestination: "/docs/reference" },
    ],
  });
  writeJson(root, "docs/publication/routes.json", {
    schemaVersion: 1,
    inventorySource: "docs/product-surface-inventory.md",
    routes: [{ id: "chat", inventoryLabel: "`/threads` chat", status: "required", documentId: "product/chat", route: "/docs/product/chat", sources: ["docs/product-surface-inventory.md"], profiles: ["server-full"] }],
  });
  return { root, trackedPaths };
}

function validate(state, options = {}) {
  return validateDocumentationPublication({ root: state.root, trackedPaths: state.trackedPaths, runChildren: options.runChildren ?? false });
}

function expectFailure(name, mutate, expected, { runChildren = false } = {}) {
  const state = fixture();
  try {
    mutate(state);
    const result = validate(state, { runChildren });
    if (!result.failures.some((failure) => failure.includes(expected))) {
      throw new Error(`${name}: expected failure containing ${JSON.stringify(expected)}, observed ${JSON.stringify(result.failures)}`);
    }
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(state.root, { recursive: true, force: true });
  }
}

function expectValid() {
  const state = fixture();
  try {
    const result = validate(state, { runChildren: true });
    if (result.failures.length) throw new Error(`valid fixture failed: ${JSON.stringify(result.failures)}`);
    if (validateGitHubActionsPolicy(state.root).failures.length) throw new Error("valid Actions fixture failed");
    console.log("PASS positive control: complete publication fixture");
  } finally {
    rmSync(state.root, { recursive: true, force: true });
  }
}

function expectPolicyFailure(name, mutate, expected) {
  const state = fixture();
  try {
    mutate(state);
    const failures = validateGitHubActionsPolicy(state.root).failures;
    if (!failures.some((failure) => failure.includes(expected))) {
      throw new Error(`${name}: expected policy failure containing ${JSON.stringify(expected)}, observed ${JSON.stringify(failures)}`);
    }
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(state.root, { recursive: true, force: true });
  }
}

expectFailure("unclassified source", (state) => {
  write(state.root, "other/README.md", "# Orphan\n");
  state.trackedPaths.push("other/README.md");
}, "source classification missing: other/README.md");

expectFailure("ambiguous source", (state) => {
  const manifest = readJson(state.root, "docs/publication/sources.json");
  manifest.rules.push({ id: "duplicate-root", selector: { paths: ["README.md"] }, disposition: "public", owner: "fixture", status: "current", publicationMode: "direct", canonicalAuthority: "README.md", publicDestination: "/" });
  writeJson(state.root, "docs/publication/sources.json", manifest);
}, "source classification ambiguous: README.md");

expectFailure("missing product route", (state) => {
  const routes = readJson(state.root, "docs/publication/routes.json");
  routes.routes = [];
  writeJson(state.root, "docs/publication/routes.json", routes);
}, "product surface is missing");

expectFailure("duplicate product route", (state) => {
  const routes = readJson(state.root, "docs/publication/routes.json");
  routes.routes.push({ ...routes.routes[0] });
  writeJson(state.root, "docs/publication/routes.json", routes);
}, "route id is missing or duplicated");

expectFailure("excluded route lacks reason", (state) => {
  const routes = readJson(state.root, "docs/publication/routes.json");
  routes.routes[0].status = "excluded";
  writeJson(state.root, "docs/publication/routes.json", routes);
}, "excluded without a reason");

expectFailure("route document missing", (state) => {
  const routes = readJson(state.root, "docs/publication/routes.json");
  routes.routes[0].documentId = "product/missing";
  writeJson(state.root, "docs/publication/routes.json", routes);
}, "documentId does not exist");

expectFailure("provenance source missing", (state) => {
  write(state.root, "website/docs/product/chat.md", "---\nsource_records:\n  - .prometheus/missing.md\ncurrent_authority: /docs/product/chat\n---\n# Chat\n");
}, "provenance source does not exist");

expectFailure("provenance source excluded", (state) => {
  write(state.root, "vendor/README.md", "Third-party source.\n");
  state.trackedPaths.push("vendor/README.md");
  const manifest = readJson(state.root, "docs/publication/sources.json");
  manifest.rules.push({ id: "vendor", selector: { prefixes: ["vendor/"] }, disposition: "excluded", owner: "third-party", status: "historical", publicationMode: "none", canonicalAuthority: "upstream-vendor", rationale: "Third-party." });
  writeJson(state.root, "docs/publication/sources.json", manifest);
  write(state.root, "website/docs/product/chat.md", "---\nsource_records:\n  - vendor/README.md\ncurrent_authority: /docs/product/chat\n---\n# Chat\n");
}, "provenance source is excluded");

expectFailure("provenance authority missing", (state) => {
  write(state.root, "website/docs/product/chat.md", "---\nsource_records:\n  - .prometheus/decision.md\n---\n# Chat\n");
}, "provenance is missing current_authority");

expectFailure("historical banner missing", (state) => {
  write(state.root, "docs/history.md", "# Old design\n");
  state.trackedPaths.push("docs/history.md");
  const manifest = readJson(state.root, "docs/publication/sources.json");
  manifest.rules.find((rule) => rule.id === "docs").selector.excludePaths = ["docs/history.md"];
  manifest.rules.push({ id: "history", selector: { paths: ["docs/history.md"] }, disposition: "public", owner: "fixture", status: "historical", publicationMode: "direct", canonicalAuthority: "website/docs/product/chat.md", publicDestination: "/docs/history" });
  writeJson(state.root, "docs/publication/sources.json", manifest);
}, "lacks a supersession banner");

expectFailure("machine-local path sanitized", (state) => {
  write(state.root, "website/docs/product/chat.md", "# Chat\nLocal path: /Users/private-user/project\n");
}, "machine-local macOS path");

expectFailure("credential assignment sanitized", (state) => {
  write(state.root, "website/docs/product/chat.md", "# Chat\napi_key = \"sensitive-example-value\"\n");
}, "credential-shaped assignment");

expectFailure("private key sanitized", (state) => {
  write(state.root, "website/docs/product/chat.md", "# Chat\n-----BEGIN PRIVATE KEY-----\n");
}, "private-key material");

expectFailure("raw event payload sanitized", (state) => {
  write(state.root, "website/docs/product/chat.md", "# Chat\n{\"eventId\":\"example\"}\n");
}, "raw event or session payload");

expectFailure("private source exact copy sanitized", (state) => {
  write(state.root, "website/docs/product/chat.md", "Private decision evidence that is cited but not copied.\n");
}, "exact copy of private-synthesis-only source");

expectFailure("child validator failure preserved", (state) => {
  write(state.root, "scripts/validate-documentation-truth.mjs", "process.exit(7);\n");
}, "child validator failed (exit 7)", { runChildren: true });

expectPolicyFailure("missing Pages publisher", (state) => {
  write(state.root, ".github/workflows/docs.yml", "jobs: {}\n");
}, "found 0");

expectPolicyFailure("website package-manager mismatch", (state) => {
  writeJson(state.root, "website/package.json", { scripts: { build: "pnpm copy:adr && docusaurus build" } });
}, "website build must use the npm-managed");

expectPolicyFailure("missing reference staging", (state) => {
  const workflow = readFileSync(join(state.root, ".github/workflows/docs.yml"), "utf8");
  write(state.root, ".github/workflows/docs.yml", workflow.replace("node scripts/stage-documentation-references.mjs", "echo skipped"));
}, "missing documentation deployment marker: node scripts/stage-documentation-references.mjs");

expectPolicyFailure("placeholder reference fallback", (state) => {
  const workflow = readFileSync(join(state.root, ".github/workflows/docs.yml"), "utf8");
  write(state.root, ".github/workflows/docs.yml", `${workflow}\n# placeholder reference fallback\n`);
}, "placeholder reference fallback");

expectPolicyFailure("missing deployed TypeScript route", (state) => {
  const workflow = readFileSync(join(state.root, ".github/workflows/docs.yml"), "utf8");
  write(state.root, ".github/workflows/docs.yml", workflow.replace("docs/api/typescript/", "docs/api/missing/"));
}, "missing documentation deployment marker: docs/api/typescript/");

expectValid();

expectPolicyFailure("competing Pages publisher", (state) => {
  write(state.root, ".github/workflows/typescript-sdk-docs.yml", "jobs:\n  deploy:\n    steps:\n      - uses: actions/upload-pages-artifact@v5\n      - uses: actions/deploy-pages@v5\n");
}, "found 2");
