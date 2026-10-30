#!/usr/bin/env node

// The Rust SDK's `embedded` feature links the runtime crate through an exact
// path dependency. A root version bump without the matching pin leaves the
// embedded SDK unresolvable (issue #391), so a release must keep them equal.

import { readFileSync } from "node:fs";

const packageVersion = (manifest) => {
  const section = manifest.split(/^\[package\]\s*$/m)[1]?.split(/^\[/m)[0] ?? "";
  return section.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
};

const rootVersion = packageVersion(readFileSync("Cargo.toml", "utf8"));
const sdkManifest = readFileSync("sdks/rust/Cargo.toml", "utf8");
const pin = sdkManifest.match(/^universal-agent-runtime\s*=\s*\{[^}]*version\s*=\s*"=([^"]+)"[^}]*path\s*=\s*"\.\.\/\.\."/m)?.[1];

const failures = [];
if (!rootVersion) failures.push("Cargo.toml: no [package] version");
if (!pin) failures.push('sdks/rust/Cargo.toml: no exact universal-agent-runtime pin ("=<version>") with path "../.."');
if (rootVersion && pin && rootVersion !== pin) {
  failures.push(`sdks/rust/Cargo.toml pins universal-agent-runtime =${pin}, but Cargo.toml declares ${rootVersion}`);
}

if (failures.length > 0) {
  for (const failure of failures) console.error(failure);
  process.exit(1);
}
console.log(`Rust SDK runtime pin matches the root version (${rootVersion}).`);
