#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { basename, dirname, extname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { validateGitHubActionsPolicy } from "./validate-github-actions-policy.mjs";

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), "..");
const allowedDispositions = new Set(["public", "public-normalize", "private-synthesis-only", "excluded"]);
const allowedStatuses = new Set(["current", "historical"]);
const allowedPublicationModes = new Set(["direct", "normalize", "synthesis", "none"]);
const allowedProfiles = new Set(["server-full", "minimal", "embedded-mobile"]);
const textExtensions = new Set([".md", ".mdx", ".html", ".htm", ".json", ".js", ".mjs", ".cjs", ".ts", ".tsx", ".css", ".svg", ".txt", ".yaml", ".yml"]);

const sanitizerRules = [
  [/(?:^|[^A-Za-z])\/Users\/[^/\s]+\//m, "machine-local macOS path"],
  [/(?:^|[^A-Za-z])\/home\/[A-Za-z0-9._-]+\//m, "machine-local Linux path"],
  [/[A-Za-z]:\\Users\\[^\\\s]+\\/m, "machine-local Windows path"],
  [/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/m, "private-key material"],
  [/(?:api[_-]?key|access[_-]?token|password|client[_-]?secret)\s*[:=]\s*["'][A-Za-z0-9_./+-]{12,}["']/im, "credential-shaped assignment"],
  [/"(?:eventId|integrityHash|session_id|conversation_id)"\s*:/m, "raw event or session payload"],
];

const posix = (path) => path.split(sep).join("/").replace(/^\.\//, "");
const readJson = (root, path) => JSON.parse(readFileSync(join(root, path), "utf8"));
const sha256 = (body) => createHash("sha256").update(body).digest("hex");

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

function getTrackedPaths(root) {
  const output = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard", "-z"], {
    cwd: root,
    encoding: "utf8",
  });
  return output.split("\0").filter(Boolean).map(posix).sort();
}

function selectorMatches(path, selector = {}) {
  const pathBasename = basename(path);
  const included =
    (selector.paths ?? []).includes(path) ||
    (selector.prefixes ?? []).some((prefix) => path.startsWith(prefix)) ||
    (selector.basenames ?? []).includes(pathBasename);
  if (!included) return false;
  if ((selector.excludePaths ?? []).includes(path)) return false;
  if ((selector.excludePrefixes ?? []).some((prefix) => path.startsWith(prefix))) return false;
  return true;
}

function isSelected(path, selectors = {}) {
  return (
    (selectors.paths ?? []).includes(path) ||
    (selectors.prefixes ?? []).some((prefix) => path.startsWith(prefix)) ||
    (selectors.basenames ?? []).includes(basename(path))
  );
}

function findRule(path, rules) {
  const matches = rules.filter((rule) => selectorMatches(path, rule.selector));
  return { matches, rule: matches.length === 1 ? matches[0] : null };
}

function validateSourceManifest(root, manifest, trackedPaths, failures) {
  if (manifest.schemaVersion !== 1) failures.push("sources.json: schemaVersion must be 1");
  if (!manifest.trackedSelectors || !Array.isArray(manifest.rules)) failures.push("sources.json: trackedSelectors and rules are required");

  const ids = new Set();
  for (const rule of manifest.rules ?? []) {
    if (!rule.id || ids.has(rule.id)) failures.push(`sources.json: rule id is missing or duplicated: ${rule.id ?? "<missing>"}`);
    ids.add(rule.id);
    if (!rule.selector || !Object.keys(rule.selector).length) failures.push(`sources.json: ${rule.id} has no selector`);
    if (!allowedDispositions.has(rule.disposition)) failures.push(`sources.json: ${rule.id} has invalid disposition`);
    if (!rule.owner) failures.push(`sources.json: ${rule.id} has no owner`);
    if (!allowedStatuses.has(rule.status)) failures.push(`sources.json: ${rule.id} has invalid status`);
    if (!allowedPublicationModes.has(rule.publicationMode)) failures.push(`sources.json: ${rule.id} has invalid publicationMode`);
    if (!rule.canonicalAuthority) failures.push(`sources.json: ${rule.id} has no canonicalAuthority`);
    if (["public", "public-normalize"].includes(rule.disposition) && !rule.publicDestination) failures.push(`sources.json: ${rule.id} has no publicDestination`);
    if (["private-synthesis-only", "excluded"].includes(rule.disposition) && !rule.rationale) failures.push(`sources.json: ${rule.id} has no rationale`);
    if (rule.generatedFrom && !existsSync(join(root, rule.generatedFrom))) failures.push(`sources.json: ${rule.id} generatedFrom does not exist: ${rule.generatedFrom}`);
    if (rule.canonicalAuthority && !rule.canonicalAuthority.startsWith("/") && rule.canonicalAuthority !== "upstream-vendor" && !existsSync(join(root, rule.canonicalAuthority))) {
      failures.push(`sources.json: ${rule.id} canonicalAuthority does not exist: ${rule.canonicalAuthority}`);
    }
  }

  const selected = trackedPaths.filter((path) => isSelected(path, manifest.trackedSelectors));
  const resolved = new Map();
  for (const path of selected) {
    const { matches, rule } = findRule(path, manifest.rules ?? []);
    if (matches.length === 0) failures.push(`source classification missing: ${path}`);
    else if (matches.length > 1) failures.push(`source classification ambiguous: ${path} (${matches.map((item) => item.id).join(", ")})`);
    else resolved.set(path, rule);
  }

  return { selected, resolved };
}

function parseInventoryLabels(body) {
  return body
    .split(/\r?\n/)
    .filter((line) => /^\|\s*`/.test(line))
    .map((line) => line.split("|")[1].trim());
}

function docusaurusDocumentIds(root) {
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

function validateRoutes(root, routeManifest, failures) {
  if (routeManifest.schemaVersion !== 1) failures.push("routes.json: schemaVersion must be 1");
  if (!routeManifest.inventorySource || !existsSync(join(root, routeManifest.inventorySource))) {
    failures.push("routes.json: inventorySource is missing or does not exist");
    return;
  }

  const inventoryLabels = parseInventoryLabels(readFileSync(join(root, routeManifest.inventorySource), "utf8"));
  const routes = routeManifest.routes ?? [];
  const ids = new Set();
  const labels = new Set();
  const documents = docusaurusDocumentIds(root);

  for (const route of routes) {
    if (!route.id || ids.has(route.id)) failures.push(`routes.json: route id is missing or duplicated: ${route.id ?? "<missing>"}`);
    ids.add(route.id);
    if (!route.inventoryLabel || labels.has(route.inventoryLabel)) failures.push(`routes.json: inventory label is missing or duplicated: ${route.inventoryLabel ?? "<missing>"}`);
    labels.add(route.inventoryLabel);
    if (!["required", "excluded"].includes(route.status)) failures.push(`routes.json: ${route.id} has invalid status`);
    if (route.status === "excluded" && !route.reason) failures.push(`routes.json: ${route.id} is excluded without a reason`);
    if (route.status === "required") {
      if (!route.documentId || !route.route) failures.push(`routes.json: ${route.id} is missing documentId or route`);
      else if (!documents.has(route.documentId)) failures.push(`routes.json: ${route.id} documentId does not exist: ${route.documentId}`);
    }
    if (!Array.isArray(route.sources) || route.sources.length === 0) failures.push(`routes.json: ${route.id} has no governing sources`);
    for (const source of route.sources ?? []) {
      if (!existsSync(join(root, source))) failures.push(`routes.json: ${route.id} source does not exist: ${source}`);
    }
    if (!Array.isArray(route.profiles) || route.profiles.length === 0 || route.profiles.some((profile) => !allowedProfiles.has(profile))) {
      failures.push(`routes.json: ${route.id} has missing or invalid profiles`);
    }
  }

  for (const label of inventoryLabels) if (!labels.has(label)) failures.push(`routes.json: product surface is missing: ${label}`);
  for (const label of labels) if (!inventoryLabels.includes(label)) failures.push(`routes.json: route has no product inventory row: ${label}`);
}

function parseProvenance(body) {
  if (!body.startsWith("---")) return null;
  const parts = body.split("---", 3);
  if (parts.length < 3) return null;
  const lines = parts[1].split(/\r?\n/);
  const records = [];
  let readingRecords = false;
  let currentAuthority = null;
  for (const line of lines) {
    if (/^source_records:\s*$/.test(line)) {
      readingRecords = true;
      continue;
    }
    const item = line.match(/^\s+-\s+["']?([^"']+?)["']?\s*$/);
    if (readingRecords && item) {
      records.push(item[1].trim());
      continue;
    }
    if (/^\S/.test(line)) readingRecords = false;
    const authority = line.match(/^current_authority:\s*["']?([^"'\n]+)["']?\s*$/);
    if (authority) currentAuthority = authority[1].trim();
  }
  return records.length || currentAuthority ? { records, currentAuthority } : null;
}

function validatePublicFiles(root, trackedPaths, sourceManifest, resolved, builtOutput, failures) {
  const publicPaths = [];
  const privateHashes = new Map();

  for (const path of trackedPaths) {
    const rule = resolved.get(path) ?? findRule(path, sourceManifest.rules ?? []).rule;
    if (!rule || !existsSync(join(root, path)) || statSync(join(root, path)).isDirectory()) continue;
    if (rule.disposition === "private-synthesis-only") {
      privateHashes.set(sha256(readFileSync(join(root, path))), path);
    }
    if (["public", "public-normalize"].includes(rule.disposition) && ["direct", "normalize"].includes(rule.publicationMode)) publicPaths.push(path);
  }

  if (builtOutput) {
    const absoluteBuild = resolve(root, builtOutput);
    if (existsSync(absoluteBuild)) publicPaths.push(...walk(root, posix(relative(root, absoluteBuild))));
  }

  for (const path of [...new Set(publicPaths)].sort()) {
    const absolute = join(root, path);
    if (!existsSync(absolute) || statSync(absolute).isDirectory()) continue;
    const bodyBuffer = readFileSync(absolute);
    const privateCopy = privateHashes.get(sha256(bodyBuffer));
    if (privateCopy) failures.push(`${path}: exact copy of private-synthesis-only source (${privateCopy})`);
    if (!textExtensions.has(extname(path).toLowerCase()) && basename(path) !== "README.md") continue;
    const body = bodyBuffer.toString("utf8");
    for (const [pattern, label] of sanitizerRules) {
      if (pattern.test(body)) failures.push(`${path}: publication sanitizer rejected ${label}`);
    }

    if ([".md", ".mdx"].includes(extname(path).toLowerCase()) || basename(path) === "README.md") {
      const rule = resolved.get(path) ?? findRule(path, sourceManifest.rules ?? []).rule;
      if (rule?.status === "historical" && rule.publicationMode === "direct" && !/(?:Historical|Superseded)/i.test(body.slice(0, 800))) {
        failures.push(`${path}: historical public document lacks a supersession banner`);
      }
      const provenance = parseProvenance(body);
      if (provenance) {
        if (!provenance.currentAuthority) failures.push(`${path}: provenance is missing current_authority`);
        for (const record of provenance.records) {
          if (!existsSync(join(root, record))) {
            failures.push(`${path}: provenance source does not exist: ${record}`);
            continue;
          }
          const sourceRule = findRule(record, sourceManifest.rules ?? []).rule;
          if (!sourceRule) failures.push(`${path}: provenance source is unclassified: ${record}`);
          else if (sourceRule.disposition === "excluded") failures.push(`${path}: provenance source is excluded: ${record}`);
        }
      }
    }
  }
}

function runChild(root, script) {
  const result = spawnSync(process.execPath, [join(root, script), root], { cwd: root, encoding: "utf8" });
  return {
    script,
    status: result.status,
    output: `${result.stdout ?? ""}${result.stderr ?? ""}`.trim(),
  };
}

export function validateDocumentationPublication({ root = defaultRoot, trackedPaths = null, builtOutput = null, runChildren = true } = {}) {
  const resolvedRoot = resolve(root);
  const failures = [];
  const sourcesPath = "docs/publication/sources.json";
  const routesPath = "docs/publication/routes.json";
  if (!existsSync(join(resolvedRoot, sourcesPath))) failures.push(`${sourcesPath} is missing`);
  if (!existsSync(join(resolvedRoot, routesPath))) failures.push(`${routesPath} is missing`);
  if (failures.length) return { failures, counts: {}, childResults: [] };

  const sourceManifest = readJson(resolvedRoot, sourcesPath);
  const routeManifest = readJson(resolvedRoot, routesPath);
  const paths = (trackedPaths ?? getTrackedPaths(resolvedRoot)).map(posix).sort();
  const { selected, resolved: resolvedSources } = validateSourceManifest(resolvedRoot, sourceManifest, paths, failures);
  validateRoutes(resolvedRoot, routeManifest, failures);
  validatePublicFiles(resolvedRoot, paths, sourceManifest, resolvedSources, builtOutput, failures);

  const childResults = [];
  if (runChildren) {
    for (const script of ["scripts/validate-documentation-truth.mjs", "scripts/validate-documentation-architecture.mjs", "scripts/validate-github-actions-policy.mjs"]) {
      const result = runChild(resolvedRoot, script);
      childResults.push(result);
      if (result.status !== 0) failures.push(`${script}: child validator failed (exit ${result.status ?? "null"})`);
    }
  }

  const counts = {};
  for (const rule of resolvedSources.values()) counts[rule.disposition] = (counts[rule.disposition] ?? 0) + 1;
  return { failures, counts, childResults, pagesPublishers: validateGitHubActionsPolicy(resolvedRoot).pagesPublishers, selectedCount: selected.length };
}

function parseArgs(argv) {
  const options = { root: defaultRoot, builtOutput: null, runChildren: true };
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === "--root") options.root = argv[++index];
    else if (argv[index] === "--built-output") options.builtOutput = argv[++index];
    else if (argv[index] === "--no-children") options.runChildren = false;
    else throw new Error(`unknown argument: ${argv[index]}`);
  }
  return options;
}

function main() {
  let options;
  try {
    options = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    process.exit(2);
  }
  const result = validateDocumentationPublication(options);
  if (result.failures.length) {
    console.error(`Documentation publication validation failed:\n- ${result.failures.join("\n- ")}`);
    for (const child of result.childResults) if (child.output) console.error(`[${child.script}]\n${child.output}`);
    process.exit(1);
  }
  console.log(`Documentation publication validation passed (${result.selectedCount} classified paths: ${JSON.stringify(result.counts)}; Pages publisher: ${result.pagesPublishers[0]}).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
