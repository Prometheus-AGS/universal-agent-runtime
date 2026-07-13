#!/usr/bin/env node

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(process.argv[2] ?? "static");
const indexPath = resolve(root, "index.html");
if (!existsSync(indexPath)) {
  throw new Error("frontend bundle is missing " + indexPath);
}

const index = readFileSync(indexPath, "utf8");
const references = [...index.matchAll(/(?:src|href)="\/(assets\/[^"]+)"/g)].map(
  (match) => match[1],
);

if (references.length === 0) {
  throw new Error("frontend index does not reference any generated assets");
}

const missing = references.filter((asset) => !existsSync(resolve(root, asset)));
if (missing.length > 0) {
  throw new Error("frontend index references missing assets: " + missing.join(", "));
}

console.log("Frontend bundle valid (" + references.length + " referenced assets).");
