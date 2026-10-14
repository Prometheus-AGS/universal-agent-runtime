#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { posix, resolve } from "node:path";

const [output, repository = resolve(import.meta.dirname, ".."), ...extra] = process.argv.slice(2);
const candidateInventory = extra.length === 2 && extra[0] === "--candidate-inventory" ? extra[1] : null;
if (!output || (extra.length && !candidateInventory)) {
  console.error("usage: generate-release-input-provenance.mjs <output> [repository-root] [--candidate-inventory <path>]");
  process.exit(2);
}

try {
  const root = realpathSync(resolve(repository));
  const git = (directory, ...args) => execFileSync("git", ["-C", directory, ...args], {
    encoding: "utf8", maxBuffer: 64 * 1024 * 1024,
  }).trimEnd();
  const requireEqual = (actual, expected, label) => {
    if (actual !== expected) throw new Error(`${label} mismatch: expected ${expected}, found ${actual}`);
  };
  requireEqual(git(root, "rev-parse", "--show-toplevel"), root, "repository root");
  const sourceSha = git(root, "rev-parse", "HEAD");
  const candidateIdentity = () => JSON.parse(execFileSync("python3", ["-c",
    "import json,sys; sys.path.insert(0,sys.argv[1]+'/scripts/dot'); import acceptance; " +
    "value=acceptance.boundary(sys.argv[2]); print(json.dumps(value['source']))",
    root, resolve(candidateInventory)], {
      encoding: "utf8", maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, PYTHONDONTWRITEBYTECODE: "1" },
    }));
  // The opt-in draft path is local-only and requires the same complete, current
  // source inventory as the runner. The normal release path remains immutable.
  const candidateSource = candidateInventory ? candidateIdentity() : null;
  const files = new Set();
  const submodules = [];
  function inspectRepository(directory, prefix = "") {
    if (git(directory, "status", "--porcelain", "--untracked-files=all") &&
        (!candidateInventory || prefix)) {
      throw new Error(`checkout must be clean: ${prefix || "."}`);
    }
    const entries = git(directory, "ls-tree", "-rz", "HEAD").split("\0").filter(Boolean);
    for (const entry of entries) {
      const [metadata, path] = entry.split("\t");
      const [mode, type, object] = metadata.split(" ");
      const fullPath = prefix ? `${prefix}/${path}` : path;
      if (mode === "160000") {
        const checkout = resolve(root, fullPath);
        // rev-parse alone can silently discover the parent when a submodule is absent.
        requireEqual(git(checkout, "rev-parse", "--show-toplevel"), checkout, `${fullPath} repository`);
        const commit = git(checkout, "rev-parse", "HEAD");
        requireEqual(commit, object, `${fullPath} checkout/gitlink`);
        submodules.push({ path: fullPath, gitlink: object, checkout_commit: commit,
          checkout_tree: git(checkout, "rev-parse", "HEAD^{tree}") });
        inspectRepository(checkout, fullPath);
      } else if (type === "blob") {
        files.add(fullPath);
      }
    }
    if (candidateInventory && !prefix) {
      for (const path of git(directory, "ls-files", "--others", "--exclude-standard", "-z").split("\0").filter(Boolean)) {
        if (lstatSync(resolve(root, path)).isFile()) files.add(path);
      }
    }
  }
  inspectRepository(root);
  const read = (path) => {
    if (!files.has(path) || !lstatSync(resolve(root, path)).isFile()) {
      throw new Error(`required input must be a tracked regular file: ${path}`);
    }
    return readFileSync(resolve(root, path));
  };
  const text = (path) => read(path).toString("utf8");
  const hash = (path) => ({ path, sha256: createHash("sha256").update(read(path)).digest("hex") });
  // Only the repository's quoted scalar pin/version fields are read here, not arbitrary TOML.
  const scalar = (content, section, key) => {
    const body = section ? content.split(`[${section}]`)[1]?.split(/^\[/m)[0] : content;
    const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const matches = [...(body || "").matchAll(new RegExp(`^${escaped}\\s*=\\s*"([^"\\n]+)"\\s*(?:#.*)?$`, "gm"))];
    if (matches.length !== 1) throw new Error(`expected one quoted ${section}.${key}`);
    return matches[0][1];
  };
  const policy = text("versions.toml");
  const pin = (name) => scalar(policy, "pins", name);
  const commitPin = (name) => {
    const value = pin(name);
    if (!/^[0-9a-f]{40}$/.test(value)) throw new Error(`policy ${name} must be a full commit`);
    return value;
  };
  const lock = text("Cargo.lock");
  const lockVersion = (name) => {
    const entries = lock.split("[[package]]").slice(1)
      .filter((entry) => scalar(entry, "", "name") === name);
    if (entries.length !== 1) throw new Error(`expected one Cargo.lock package: ${name}`);
    if (/^source\s*=/m.test(entries[0])) throw new Error(`${name} must remain a path dependency`);
    return scalar(entries[0], "", "version");
  };
  const dependency = (name, path) => {
    const line = text("Cargo.toml").split("\n").find((entry) => entry.startsWith(`${name} = `));
    if (!line?.includes(`path = "${path}"`)) throw new Error(`${name} path dependency mismatch`);
  };
  const pinnedSubmodule = (path, key) => {
    const module = submodules.find((entry) => entry.path === path);
    if (!module) throw new Error(`missing required submodule: ${path}`);
    const policyCommit = commitPin(key);
    requireEqual(module.gitlink, policyCommit, `${path} policy/gitlink`);
    return { ...module, policy_commit: policyCommit };
  };
  dependency("liter-llm", "vendor/git/liter-llm/crates/liter-llm");
  const liter = pinnedSubmodule("vendor/git/liter-llm", "liter_llm_commit");
  const literManifest = "vendor/git/liter-llm/crates/liter-llm/Cargo.toml";
  const literVersion = scalar(text(literManifest), "package", "version");
  requireEqual(literVersion, pin("liter_llm"), "Liter policy/crate version");
  requireEqual(lockVersion("liter-llm"), literVersion, "Liter crate/lock version");
  const skills = pinnedSubmodule("crates/prometheus-skill-system", "prometheus_skill_system_commit");
  const skillsRoot = resolve(root, skills.path);
  requireEqual(git(skillsRoot, "cat-file", "-t", "HEAD:skills"), "tree", "skills object type");

  dependency("surreal-memory", "vendor/git/surreal-memory-server/crates/surreal-memory");
  const memoryPath = "vendor/git/surreal-memory-server";
  const memoryManifest = `${memoryPath}/crates/surreal-memory/Cargo.toml`;
  const memoryCommit = commitPin("surreal_memory_server_commit");
  const provenanceRow = text("vendor/git/README.md").split("\n")
    .filter((line) => /^\| `surreal-memory` \|/.test(line));
  if (provenanceRow.length !== 1) throw new Error("expected one surreal-memory export provenance row");
  const recordedCommit = provenanceRow[0].match(/`([0-9a-f]{40})`/)?.[1];
  requireEqual(recordedCommit, memoryCommit, "memory export recorded/policy commit");
  requireEqual(git(root, "cat-file", "-t", `HEAD:${memoryPath}`), "tree", "memory export object type");
  const memoryVersion = scalar(text(memoryManifest), "package", "version");
  requireEqual(lockVersion("surreal-memory"), memoryVersion, "memory crate/lock version");

  // These are install workspaces, not workspace templates shipped as skill resources.
  const workspacePaths = [...files].filter((path) => path === "pnpm-workspace.yaml" ||
    (path.startsWith("frontend/") && path.endsWith("/pnpm-workspace.yaml"))).sort();
  if (!workspacePaths.includes("pnpm-workspace.yaml") || !workspacePaths.includes("frontend/pnpm-workspace.yaml") ||
      !workspacePaths.includes("frontend/packages/prometheus-entity-management/pnpm-workspace.yaml")) {
    throw new Error("missing root, frontend or entity-management pnpm workspace");
  }
  const glob = (pattern) => {
    if (!/^[A-Za-z0-9_./*\-]+$/.test(pattern) || pattern.split("/").includes("..")) {
      throw new Error(`unsupported workspace package pattern: ${pattern}`);
    }
    return new RegExp(`^${pattern.replace(/\./g, "\\.").replace(/\*\*/g, "\u0001")
      .replace(/\*/g, "[^/]*").replace(/\u0001/g, ".*")}$`);
  };
  const workspaces = workspacePaths.map((path) => {
    const directory = posix.dirname(path);
    const prefix = directory === "." ? "" : `${directory}/`;
    const packageLines = text(path).match(/^packages:\s*\n((?:[ \t]+.*\n?)*)/m)?.[1];
    if (!packageLines) throw new Error(`missing workspace package patterns: ${path}`);
    const patterns = packageLines.split("\n").filter((line) => line.trim() && !line.trim().startsWith("#"))
      .map((line) => {
        const value = line.match(/^\s+-\s+(?:"([^"\n]+)"|'([^'\n]+)'|([^#\s]+))\s*(?:#.*)?$/);
        if (!value) throw new Error(`unsupported packages syntax: ${path}`);
        return value[1] || value[2] || value[3];
      });
    const positive = patterns.filter((entry) => !entry.startsWith("!")).map(glob);
    const negative = patterns.filter((entry) => entry.startsWith("!")).map((entry) => glob(entry.slice(1)));
    const manifestPaths = [...files].filter((entry) => entry.startsWith(prefix) &&
      (entry === `${prefix}package.json` || entry.endsWith("/package.json")))
      .filter((entry) => {
        const packageDirectory = posix.dirname(entry.slice(prefix.length));
        return packageDirectory === "." || (positive.some((pattern) => pattern.test(packageDirectory)) &&
          !negative.some((pattern) => pattern.test(packageDirectory)));
      }).sort();
    if (!manifestPaths.includes(`${prefix}package.json`)) throw new Error(`missing workspace root manifest: ${path}`);
    return { directory, definition: hash(path), lock: hash(`${prefix}pnpm-lock.yaml`),
      manifests: manifestPaths.map(hash) };
  });

  const schemaPaths = new Set();
  for (const path of [...files].filter((entry) => entry.startsWith("src/uar/compiler/collaboration/") && entry.endsWith(".rs"))) {
    for (const match of text(path).matchAll(/include_str!\(\s*"([^"]+\.schema\.json)"\s*\)/g)) {
      schemaPaths.add(posix.normalize(posix.join(posix.dirname(path), match[1])));
    }
  }
  if (schemaPaths.size === 0) throw new Error("missing compiled collaboration schemas");
  const inputs = ["versions.toml", "Cargo.toml", "Cargo.lock", "Dockerfile", ".dockerignore",
    "catalog/provider_catalog.json", "vendor/git/README.md", "vendor/git/liter-llm/Cargo.toml",
    literManifest, memoryManifest];
  const receipt = {
    schema_version: 1,
    source_sha: sourceSha,
    source_tree: candidateInventory ? null : git(root, "rev-parse", "HEAD^{tree}"),
    scope: candidateInventory ? "local_candidate" : "approved source inputs; not build, offline closure or launched runtime certification",
    input_boundary: candidateInventory ? "local_candidate" : "release",
    ...(candidateInventory ? {
      base_source_tree: git(root, "rev-parse", "HEAD^{tree}"),
      candidate_inventory_sha256: createHash("sha256").update(readFileSync(resolve(candidateInventory))).digest("hex"),
      candidate_source: candidateSource,
      release_certification: false,
    } : {}),
    policy: hash("versions.toml"),
    submodules: submodules.sort((a, b) => a.path.localeCompare(b.path)),
    liter: { ...liter, policy_version: pin("liter_llm"), crate_version: literVersion,
      lock_version: lockVersion("liter-llm"), manifest: hash(literManifest) },
    skills: { ...skills, source: "embedded_submodule", relative_root: `${skills.path}/skills`,
      content_tree: git(skillsRoot, "rev-parse", "HEAD:skills"), launched_runtime_verified: false },
    memory_export: { path: memoryPath, policy_upstream_commit: memoryCommit,
      recorded_upstream_commit: recordedCommit, root_entry_type: "tree",
      exported_tree: git(root, "rev-parse", `HEAD:${memoryPath}`), crate_version: memoryVersion,
      lock_version: lockVersion("surreal-memory"), adapted_manifest: hash(memoryManifest),
      upstream_byte_comparison_performed: false,
      files: [...files].filter((path) => path.startsWith(`${memoryPath}/`)).sort().map(hash) },
    pnpm_workspaces: workspaces,
    inputs: inputs.map(hash),
    compiled_schemas: [...schemaPaths].sort().map(hash),
  };
  if (candidateInventory) requireEqual(JSON.stringify(candidateIdentity()), JSON.stringify(candidateSource),
    "candidate source changed during input capture");
  // No success file or success message is emitted until every comparison and input read passes.
  writeFileSync(resolve(output), `${JSON.stringify(receipt, null, 2)}\n`, { flag: "wx" });
  console.log(`wrote release input provenance for ${sourceSha} (${workspaces.length} pnpm workspaces)`);
} catch (error) {
  console.error(`release input provenance rejected: ${error.message}`);
  process.exitCode = 1;
}
