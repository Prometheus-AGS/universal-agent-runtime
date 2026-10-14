#!/usr/bin/env node

import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { afterEach, beforeEach, test } from "node:test";

const cli = resolve(import.meta.dirname, "generate-release-input-provenance.mjs");
let scratch;
let root;
let output;
let literCommit;
let skillsCommit;
const memoryCommit = "432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8";
const git = (directory, ...args) => execFileSync("git", ["-C", directory, ...args], { encoding: "utf8" }).trim();
const write = (directory, path, content) => {
  const absolute = join(directory, path);
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, content);
};
const commit = (directory) => {
  git(directory, "add", ".");
  git(directory, "commit", "--quiet", "-m", "fixture inputs");
  return git(directory, "rev-parse", "HEAD");
};
const init = (directory) => {
  mkdirSync(directory, { recursive: true });
  git(directory, "init", "--quiet");
  git(directory, "config", "user.name", "Release provenance fixture");
  git(directory, "config", "user.email", "release-fixture@example.invalid");
  git(directory, "config", "commit.gpgsign", "false");
};
const addSubmodule = (source, path) => {
  git(root, "-c", "protocol.file.allow=always", "submodule", "add", "--quiet", source, path);
};
const invoke = () => spawnSync(process.execPath, [cli, output, root], { encoding: "utf8" });
const reject = (pattern) => {
  const result = invoke();
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, pattern);
  assert.equal(result.stdout, "");
  assert.equal(existsSync(output), false, "rejection must not write a success receipt");
};
const replace = (path, from, to) => {
  const content = readFileSync(join(root, path), "utf8");
  assert.ok(content.includes(from));
  write(root, path, content.replace(from, to));
  commit(root);
};

beforeEach(() => {
  scratch = mkdtempSync(join(tmpdir(), "uar-release-inputs-"));
  root = join(scratch, "root");
  output = join(scratch, "release-input-provenance.json");
  init(root);
  const liter = join(scratch, "liter");
  init(liter);
  write(liter, "Cargo.toml", '[workspace]\nmembers = ["crates/liter-llm"]\n');
  write(liter, "crates/liter-llm/Cargo.toml", '[package]\nname = "liter-llm"\nversion = "2.1.1"\n');
  literCommit = commit(liter);
  addSubmodule(liter, "vendor/git/liter-llm");
  const skills = join(scratch, "skills");
  init(skills);
  write(skills, "skills/example/SKILL.md", "# Fixture source skill\n");
  skillsCommit = commit(skills);
  addSubmodule(skills, "crates/prometheus-skill-system");
  const entity = join(scratch, "entity");
  init(entity);
  write(entity, "package.json", '{"name":"entity-fixture","private":true}\n');
  write(entity, "pnpm-workspace.yaml", 'packages:\n  - "packages/*"\n');
  write(entity, "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
  write(entity, "packages/core/package.json", '{"name":"core-fixture"}\n');
  write(entity, "templates/example/package.json", '{"name":"excluded-template"}\n');
  commit(entity);
  addSubmodule(entity, "frontend/packages/prometheus-entity-management");
  write(root, "versions.toml", `[pins]\nliter_llm = "2.1.1"\nliter_llm_commit = "${literCommit}"\nprometheus_skill_system_commit = "${skillsCommit}"\nsurreal_memory_server_commit = "${memoryCommit}"\n`);
  write(root, "Cargo.toml", '[package]\nname = "fixture"\nversion = "0.1.0"\n[dependencies]\nliter-llm = { path = "vendor/git/liter-llm/crates/liter-llm" }\nsurreal-memory = { path = "vendor/git/surreal-memory-server/crates/surreal-memory" }\n');
  write(root, "Cargo.lock", 'version = 4\n[[package]]\nname = "liter-llm"\nversion = "2.1.1"\n[[package]]\nname = "surreal-memory"\nversion = "0.1.0"\n');
  write(root, "vendor/git/README.md", `| Package | Upstream | Commit | License |\n| \`surreal-memory\` | \`https://example.invalid/memory.git\` | \`${memoryCommit}\` | MIT |\n`);
  write(root, "vendor/git/surreal-memory-server/crates/surreal-memory/Cargo.toml", '[package]\nname = "surreal-memory"\nversion = "0.1.0"\n');
  write(root, "vendor/git/surreal-memory-server/crates/surreal-memory/src/lib.rs", "// exported source fixture\n");
  write(root, "package.json", '{"name":"root-fixture","private":true}\n');
  write(root, "pnpm-workspace.yaml", "packages:\n  - .\n  - frontend\n  - frontend/packages/*\n  - frontend/packages/prometheus-entity-management/packages/*\n");
  write(root, "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
  write(root, "frontend/package.json", '{"name":"frontend-fixture","private":true}\n');
  write(root, "frontend/pnpm-workspace.yaml", 'packages:\n  - "."\n  - "packages/*"\n');
  write(root, "frontend/pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
  write(root, "Dockerfile", "RUN cargo build --release --locked\nRUN pnpm install --frozen-lockfile\n");
  write(root, ".dockerignore", ".git/\n");
  write(root, "catalog/provider_catalog.json", "{}\n");
  write(root, "src/uar/compiler/collaboration/fixture.rs", 'const SCHEMA: &str = include_str!("../../../../docs/example.schema.json");\n');
  write(root, "docs/example.schema.json", '{"type":"object"}\n');
  commit(root);
});

afterEach(() => rmSync(scratch, { recursive: true, force: true }));

test("public CLI binds real source, recursive gitlinks, workspace manifests and distinct memory identities", () => {
  const result = invoke();
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /3 pnpm workspaces/);
  const receipt = JSON.parse(readFileSync(output, "utf8"));
  assert.equal(receipt.source_sha, git(root, "rev-parse", "HEAD"));
  assert.equal(receipt.source_tree, git(root, "rev-parse", "HEAD^{tree}"));
  assert.equal(receipt.liter.policy_commit, literCommit);
  assert.equal(receipt.liter.checkout_commit, literCommit);
  assert.equal(receipt.skills.policy_commit, skillsCommit);
  assert.equal(receipt.skills.launched_runtime_verified, false);
  assert.equal(receipt.memory_export.policy_upstream_commit, memoryCommit);
  assert.equal(receipt.memory_export.exported_tree,
    git(root, "rev-parse", "HEAD:vendor/git/surreal-memory-server"));
  assert.notEqual(receipt.memory_export.exported_tree, memoryCommit);
  assert.equal(receipt.memory_export.upstream_byte_comparison_performed, false);
  assert.equal(receipt.memory_export.files.length, 2);
  const workspace = receipt.pnpm_workspaces.find((entry) => entry.directory === "frontend/packages/prometheus-entity-management");
  assert.deepEqual(workspace.manifests.map((entry) => entry.path), [
    "frontend/packages/prometheus-entity-management/package.json",
    "frontend/packages/prometheus-entity-management/packages/core/package.json",
  ]);
  for (const input of [...receipt.inputs, ...receipt.compiled_schemas,
    ...receipt.pnpm_workspaces.flatMap((entry) => [entry.definition, entry.lock, ...entry.manifests])]) {
    assert.equal(input.sha256, createHash("sha256").update(readFileSync(join(root, input.path))).digest("hex"));
  }
});

test("rejects policy commit drift before emitting success", () => {
  replace("versions.toml", literCommit, "0".repeat(40));
  reject(/policy\/gitlink mismatch/);
});

test("rejects a real submodule checkout different from its committed gitlink", () => {
  const checkout = join(root, "vendor/git/liter-llm");
  git(checkout, "config", "user.name", "Release provenance fixture");
  git(checkout, "config", "user.email", "release-fixture@example.invalid");
  git(checkout, "config", "commit.gpgsign", "false");
  write(checkout, "changed.txt", "different checkout\n");
  commit(checkout);
  reject(/checkout must be clean|checkout\/gitlink mismatch/);
});

test("rejects declared crate and lock version disagreement", () => {
  replace("Cargo.lock", 'version = "2.1.1"', 'version = "2.1.0"');
  reject(/Liter crate\/lock version mismatch/);
});

test("rejects recorded memory upstream commit disagreement without comparing it to a tree", () => {
  replace("vendor/git/README.md", memoryCommit, "1".repeat(40));
  reject(/memory export recorded\/policy commit mismatch/);
});

test("rejects a missing actual workspace lock", () => {
  git(root, "rm", "--quiet", "frontend/pnpm-lock.yaml");
  commit(root);
  reject(/required input must be a tracked regular file: frontend\/pnpm-lock.yaml/);
});

test("rejects a missing compiled schema input", () => {
  git(root, "rm", "--quiet", "docs/example.schema.json");
  commit(root);
  reject(/required input must be a tracked regular file: docs\/example.schema.json/);
});

test("rejects dirty source inputs", () => {
  write(root, "catalog/provider_catalog.json", '{"dirty":true}\n');
  reject(/checkout must be clean/);
});
