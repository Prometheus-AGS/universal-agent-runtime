#!/usr/bin/env node

import { cpSync, existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), "..");

function parseArguments(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key.startsWith("--") || !value || value.startsWith("--")) {
      throw new Error(`Invalid argument near ${key}`);
    }
    options[key.slice(2)] = value;
    index += 1;
  }
  return options;
}

function requireGeneratedReference(source, label) {
  const indexPath = join(source, "index.html");
  if (!existsSync(indexPath)) {
    throw new Error(`${label} reference is missing its generated index: ${indexPath}`);
  }
  if (readFileSync(indexPath, "utf8").trim().length === 0) {
    throw new Error(`${label} reference index is empty: ${indexPath}`);
  }
}

function requireDestinationWithin(buildOutput, destination) {
  const childPath = relative(buildOutput, destination);
  if (!childPath || childPath.startsWith("..") || isAbsolute(childPath)) {
    throw new Error(`Refusing to stage outside the documentation build directory: ${destination}`);
  }
}

export function stageDocumentationReferences({
  root = defaultRoot,
  rustSource,
  typescriptSource,
  buildOutput,
} = {}) {
  const resolvedRoot = resolve(root);
  const resolvedRustSource = resolve(rustSource ?? join(resolvedRoot, "target", "doc"));
  const resolvedTypescriptSource = resolve(
    typescriptSource ?? join(resolvedRoot, "sdks", "typescript", "docs", "api"),
  );
  const resolvedBuildOutput = resolve(buildOutput ?? join(resolvedRoot, "website", "build"));
  const destinations = {
    rust: join(resolvedBuildOutput, "docs", "api", "rust"),
    typescript: join(resolvedBuildOutput, "docs", "api", "typescript"),
  };

  requireGeneratedReference(resolvedRustSource, "Rust");
  requireGeneratedReference(resolvedTypescriptSource, "TypeScript");
  requireDestinationWithin(resolvedBuildOutput, destinations.rust);
  requireDestinationWithin(resolvedBuildOutput, destinations.typescript);

  for (const [label, source] of [
    ["rust", resolvedRustSource],
    ["typescript", resolvedTypescriptSource],
  ]) {
    const destination = destinations[label];
    rmSync(destination, { recursive: true, force: true });
    mkdirSync(dirname(destination), { recursive: true });
    cpSync(source, destination, { recursive: true, force: true });
  }

  return { buildOutput: resolvedBuildOutput, destinations };
}

function main() {
  const args = parseArguments(process.argv.slice(2));
  const result = stageDocumentationReferences({
    root: args.root,
    rustSource: args["rust-source"],
    typescriptSource: args["typescript-source"],
    buildOutput: args["build-output"],
  });
  console.log(`Staged Rust reference: ${result.destinations.rust}`);
  console.log(`Staged TypeScript reference: ${result.destinations.typescript}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    main();
  } catch (error) {
    console.error(`Documentation reference staging failed: ${error.message}`);
    process.exit(1);
  }
}
