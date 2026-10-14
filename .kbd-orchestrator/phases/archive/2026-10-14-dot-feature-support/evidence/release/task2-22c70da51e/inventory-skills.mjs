import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync, readlinkSync } from "node:fs";
import { join, resolve } from "node:path";

const root = resolve(process.argv[2]);
const entries = [];
function walk(directory, prefix = "") {
  for (const name of readdirSync(directory).sort()) {
    if (name === ".git") continue;
    const path = prefix ? `${prefix}/${name}` : name;
    const absolute = join(directory, name);
    const stat = lstatSync(absolute);
    if (stat.isDirectory()) walk(absolute, path);
    else if (stat.isSymbolicLink()) entries.push({ path, kind: "symlink", target: readlinkSync(absolute) });
    else if (stat.isFile()) entries.push({ path, kind: "file", size: stat.size,
      sha256: createHash("sha256").update(readFileSync(absolute)).digest("hex") });
    else throw new Error(`unsupported skill input: ${path}`);
  }
}
walk(root);
entries.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
console.log(JSON.stringify({ root, git_metadata_excluded: true, entries,
  inventory_sha256: createHash("sha256").update(JSON.stringify(entries)).digest("hex") }, null, 2));
