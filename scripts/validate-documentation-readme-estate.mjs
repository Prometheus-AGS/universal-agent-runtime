#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { basename, dirname, extname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), "..");
const allowedKinds = new Set(["root", "uar-owned", "generated-mirror", "vendored"]);
const allowedStatuses = new Set(["current", "historical"]);
const allowedActions = new Set(["reconcile", "replace-placeholder", "preserve-with-banner", "regenerate", "exclude"]);
const allowedProfiles = new Set(["server-full", "minimal", "embedded-mobile"]);
const unsafePatterns = [
  [/(?:^|[^A-Za-z])\/Users\/[^/\s]+\//m, "machine-local macOS path"],
  [/(?:^|[^A-Za-z])\/home\/[A-Za-z0-9._-]+\//m, "machine-local Linux path"],
  [/[A-Za-z]:\\Users\\[^\\\s]+\\/m, "machine-local Windows path"],
  [/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/m, "private-key material"],
  [/(?:api[_-]?key|access[_-]?token|password|client[_-]?secret)\s*[:=]\s*["'][A-Za-z0-9_./+-]{12,}["']/im, "credential-shaped assignment"],
  [/"(?:eventId|integrityHash|session_id|conversation_id)"\s*:/m, "raw event or session payload"],
];
const staleCurrentPatterns = [
  [/\b142\+ providers\b/i, "blanket provider claim"],
  [/\b(?:no|without) React\b/i, "retired no-React claim"],
  [/\bHTMX(?:\s*\/\s*Web Components)?\b[^\n]{0,100}\bprimary UI\b/i, "retired HTMX primary-UI claim"],
  [/\bbun install\b/i, "retired Bun install command"],
  [/\bproduction[- ]ready\b/i, "unscoped production-ready claim"],
  [/^\s*(?:#+\s*)?placeholder(?:\s+(?:content|examples?))?(?:\s*\([^)]*\))?\s*$/im, "placeholder content"],
  [/GitHub Actions[^\n]{0,120}\b(?:unit|integration|lint|typecheck|test)\b/i, "routine GitHub Actions testing claim"],
  [/\b(?:server-full|minimal|embedded-mobile)\b[^\n]{0,120}\brun identically\b/i, "cross-profile equivalence claim"],
];

const posix = (path) => path.split(sep).join("/").replace(/^\.\//, "");
const sha256 = (body) => createHash("sha256").update(body).digest("hex");
const readJson = (root, path) => JSON.parse(readFileSync(join(root, path), "utf8"));

function walk(root, directory) {
  const absolute = join(root, directory);
  if (!existsSync(absolute)) return [];
  const paths = [];
  for (const entry of readdirSync(absolute, { withFileTypes: true })) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) paths.push(...walk(root, posix(relative(root, child))));
    else if (entry.isFile()) paths.push(posix(relative(root, child)));
  }
  return paths;
}

function trackedReadmePaths(root) {
  const output = execFileSync("git", ["ls-files", "-z"], { cwd: root });
  return output
    .toString("utf8")
    .split("\0")
    .filter((path) => path === "README.md" || path.endsWith("/README.md"))
    .map(posix)
    .sort();
}

function docusaurusRoutes(root) {
  const routes = new Set();
  for (const path of walk(root, "website/docs")) {
    if (![".md", ".mdx"].includes(extname(path)) || basename(path).startsWith("_category_")) continue;
    const body = readFileSync(join(root, path), "utf8");
    const frontmatter = body.startsWith("---") ? body.split("---", 3)[1] : "";
    const explicitId = frontmatter.match(/^id:\s*["']?([^"'\n]+)["']?\s*$/m)?.[1]?.trim();
    const fallback = path.replace(/^website\/docs\//, "").replace(/\.(?:md|mdx)$/, "");
    routes.add(`/docs/${explicitId ?? fallback}`);
  }
  return routes;
}

function routeDocuments(root) {
  const ids = new Set();
  for (const path of walk(root, "website/docs")) {
    if (![".md", ".mdx"].includes(extname(path)) || basename(path).startsWith("_category_")) continue;
    const body = readFileSync(join(root, path), "utf8");
    const frontmatter = body.startsWith("---") ? body.split("---", 3)[1] : "";
    const explicitId = frontmatter.match(/^id:\s*["']?([^"'\n]+)["']?\s*$/m)?.[1]?.trim();
    const fallback = path.replace(/^website\/docs\//, "").replace(/\.(?:md|mdx)$/, "");
    ids.add(explicitId ?? fallback);
  }
  return ids;
}

export function validateReadmeEstate({ root = defaultRoot, trackedReadmes = null } = {}) {
  const resolvedRoot = resolve(root);
  const failures = [];
  const manifestPath = "docs/publication/readme-estate.json";
  if (!existsSync(join(resolvedRoot, manifestPath))) {
    return { failures: [`${manifestPath} is missing`], counts: {} };
  }

  const manifest = readJson(resolvedRoot, manifestPath);
  if (manifest.schemaVersion !== 1) failures.push("readme-estate.json: schemaVersion must be 1");
  if (!/^https:\/\/[^/]+\/.+\/docs\/intro$/.test(manifest.canonicalPortal ?? "")) {
    failures.push("readme-estate.json: canonicalPortal must be the repository Pages intro URL");
  }

  const entries = manifest.entries ?? [];
  const paths = (trackedReadmes ?? trackedReadmePaths(resolvedRoot)).map(posix).sort();
  const byPath = new Map();
  for (const entry of entries) {
    if (!entry.path) {
      failures.push("readme-estate.json: entry path is required");
      continue;
    }
    if (byPath.has(entry.path)) failures.push(`readme-estate.json: duplicate entry: ${entry.path}`);
    else byPath.set(entry.path, entry);
  }
  for (const path of paths) if (!byPath.has(path)) failures.push(`README ownership missing: ${path}`);
  for (const path of byPath.keys()) if (!paths.includes(path)) failures.push(`README manifest entry is not tracked: ${path}`);

  const portalRoutes = docusaurusRoutes(resolvedRoot);
  const documents = routeDocuments(resolvedRoot);
  for (const entry of entries) {
    const absolute = join(resolvedRoot, entry.path ?? "");
    if (!existsSync(absolute)) {
      failures.push(`${entry.path}: README does not exist`);
      continue;
    }
    if (!allowedKinds.has(entry.kind)) failures.push(`${entry.path}: invalid kind ${entry.kind}`);
    if (!allowedStatuses.has(entry.status)) failures.push(`${entry.path}: invalid status ${entry.status}`);
    if (!allowedActions.has(entry.action)) failures.push(`${entry.path}: invalid action ${entry.action}`);
    if (!entry.owner) failures.push(`${entry.path}: owner is required`);
    if (!entry.authority) failures.push(`${entry.path}: current authority is required`);
    if (!Array.isArray(entry.profiles) || entry.profiles.some((profile) => !allowedProfiles.has(profile))) {
      failures.push(`${entry.path}: profiles are missing or invalid`);
    }
    if (entry.authority?.startsWith("/docs/") && !portalRoutes.has(entry.authority)) {
      failures.push(`${entry.path}: portal authority does not exist: ${entry.authority}`);
    } else if (entry.authority && !entry.authority.startsWith("/docs/") && entry.authority !== "upstream-vendor" && !existsSync(join(resolvedRoot, entry.authority))) {
      failures.push(`${entry.path}: file authority does not exist: ${entry.authority}`);
    }

    const body = readFileSync(absolute, "utf8");
    if (["root", "uar-owned"].includes(entry.kind)) {
      for (const [pattern, label] of unsafePatterns) {
        pattern.lastIndex = 0;
        if (pattern.test(body)) failures.push(`${entry.path}: ${label}`);
      }
    }
    if (entry.kind === "uar-owned" && entry.status === "current") {
      if (!/> \*\*Current authority:\*\*/.test(body)) failures.push(`${entry.path}: current authority block is missing`);
      if (entry.authority?.startsWith("/docs/") && !body.includes(entry.authority)) failures.push(`${entry.path}: current authority link is missing: ${entry.authority}`);
      for (const [pattern, label] of staleCurrentPatterns) {
        pattern.lastIndex = 0;
        if (pattern.test(body)) failures.push(`${entry.path}: ${label}`);
      }
    }
    if (entry.kind === "uar-owned" && entry.status === "historical") {
      if (!/> \*\*Historical — superseded \d{4}-\d{2}-\d{2}\.\*\*/.test(body)) failures.push(`${entry.path}: dated historical banner is missing`);
      if (entry.authority?.startsWith("/docs/") && !body.includes(entry.authority)) failures.push(`${entry.path}: historical current-authority link is missing: ${entry.authority}`);
    }
    if (entry.kind === "generated-mirror") {
      if (!entry.generatedFrom || !existsSync(join(resolvedRoot, entry.generatedFrom))) {
        failures.push(`${entry.path}: generated source is missing`);
      } else if (!readFileSync(absolute).equals(readFileSync(join(resolvedRoot, entry.generatedFrom)))) {
        failures.push(`${entry.path}: generated mirror differs from ${entry.generatedFrom}`);
      }
    }
    if (entry.kind === "vendored") {
      if (!/^[a-f0-9]{64}$/.test(entry.sha256 ?? "")) failures.push(`${entry.path}: vendored SHA-256 is missing`);
      else if (sha256(readFileSync(absolute)) !== entry.sha256) failures.push(`${entry.path}: vendored README hash changed`);
    }
  }

  const rootBody = existsSync(join(resolvedRoot, "README.md")) ? readFileSync(join(resolvedRoot, "README.md"), "utf8") : "";
  const firstScreen = rootBody.slice(0, 1800);
  if (!firstScreen.includes("website/static/img/brand/uar-wordmark-dark.svg")) failures.push("README.md: existing UAR wordmark is missing from the hero");
  if (!firstScreen.includes("Governed execution. Typed protocols. One runtime boundary.")) failures.push("README.md: current portal tagline is missing from the hero");
  if ((firstScreen.match(/img\.shields\.io/g) ?? []).length < 3) failures.push("README.md: license, version, and documentation badges are required");
  if (!rootBody.includes(manifest.canonicalPortal ?? "")) failures.push("README.md: canonical portal link is missing");

  if (existsSync(join(resolvedRoot, "docs/publication/routes.json"))) {
    const routeManifest = readJson(resolvedRoot, "docs/publication/routes.json");
    for (const route of routeManifest.routes ?? []) {
      if (route.status === "required" && !documents.has(route.documentId)) failures.push(`routes.json: required document is missing: ${route.documentId}`);
    }
  }

  const counts = {};
  for (const entry of entries) counts[entry.kind] = (counts[entry.kind] ?? 0) + 1;
  return { failures, counts, readmeCount: paths.length };
}

if (process.argv[1] && resolve(process.argv[1]) === scriptPath) {
  const root = process.argv[2] ? resolve(process.argv[2]) : defaultRoot;
  const result = validateReadmeEstate({ root });
  if (result.failures.length) {
    console.error(`Documentation README-estate validation failed:\n- ${result.failures.join("\n- ")}`);
    process.exit(1);
  }
  console.log(`Documentation README-estate validation passed (${result.readmeCount} READMEs: ${Object.entries(result.counts).map(([kind, count]) => `${count} ${kind}`).join(", ")}).`);
}
