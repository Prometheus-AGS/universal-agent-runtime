#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { basename, dirname, extname, join, relative, resolve, sep } from "node:path";

const root = resolve(import.meta.dirname, "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");
const failures = [];
const manifest = JSON.parse(read("package.json"));
const cargo = read("Cargo.toml");
const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const cargoLicense = cargo.match(/^license\s*=\s*"([^"]+)"/m)?.[1];

const posix = (path) => path.split(sep).join("/");

function walk(directory) {
  const absolute = resolve(root, directory);
  if (!existsSync(absolute)) return [];
  const paths = [];
  for (const entry of readdirSync(absolute, { withFileTypes: true })) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) paths.push(...walk(posix(relative(root, child))));
    else if (entry.isFile()) paths.push(posix(relative(root, child)));
  }
  return paths;
}

function docusaurusRoutes() {
  const routes = new Map();
  for (const path of walk("website/docs")) {
    if (![".md", ".mdx"].includes(extname(path)) || basename(path).startsWith("_category_")) continue;
    const body = read(path);
    const frontmatter = body.startsWith("---") ? body.split("---", 3)[1] : "";
    const explicitId = frontmatter.match(/^id:\s*["']?([^"'\n]+)["']?\s*$/m)?.[1]?.trim();
    const explicitSlug = frontmatter.match(/^slug:\s*["']?([^"'\n]+)["']?\s*$/m)?.[1]?.trim();
    const fallback = path.replace(/^website\/docs\//, "").replace(/\.(?:md|mdx)$/, "");
    const inferredPath = fallback.endsWith("/index") ? fallback.slice(0, -"/index".length) : fallback;
    const route = explicitSlug
      ? `/docs/${explicitSlug.replace(/^\/+/, "")}`
      : `/docs/${explicitId ?? inferredPath}`;
    routes.set(route.replace(/\/$/, ""), path);
  }
  return routes;
}

const support = JSON.parse(read("docs/product-support-matrix.json"));
if (support.product_version !== cargoVersion) {
  failures.push(`product version mismatch: support=${support.product_version}, Cargo.toml=${cargoVersion}`);
}
const versionInventory = support.version_inventory;
const requiredVersionPaths = ["Cargo.toml", "package.json", "frontend/package.json", "website/package.json",
  "src-tauri/tauri.conf.json", "sdks/rust/Cargo.toml", "sdks/python/pyproject.toml", "sdks/typescript/package.json"];
if (!Array.isArray(versionInventory) || versionInventory.length === 0) {
  failures.push("support matrix lacks recorded independent version inventory");
} else {
  const seen = new Set();
  for (const row of versionInventory) {
    if (!requiredVersionPaths.includes(row.path) || seen.has(row.path)) {
      failures.push(`unexpected or duplicate version inventory path: ${row.path}`);
      continue;
    }
    seen.add(row.path);
    const source = read(row.path);
    const actual = row.format === "json" ? JSON.parse(source).version
      : row.format === "toml" ? source.match(/^version\s*=\s*"([^"]+)"/m)?.[1] : undefined;
    if (!row.role || !row.version || actual !== row.version) {
      failures.push(`recorded version mismatch: ${row.path}=${actual}, inventory=${row.version}`);
    }
    if (row.path === "Cargo.toml" && (row.role !== "runtime-product" || row.version !== cargoVersion)) {
      failures.push("runtime-product inventory does not match Cargo product identity");
    }
  }
  for (const path of requiredVersionPaths) {
    if (!seen.has(path)) failures.push(`version inventory missing: ${path}`);
  }
}
if (manifest.license !== cargoLicense) failures.push(`license mismatch: package.json=${manifest.license}, Cargo.toml=${cargoLicense}`);
for (const path of ["README.md", "website/docs/intro.md"]) {
  if (!read(path).includes(`**${cargoVersion}**`)) failures.push(`${path}: current Rust product version is not explicit`);
  if (/\b(?:269|316)[- ]provider/iu.test(read(path))) failures.push(`${path}: fixed provider total must derive from current catalog metadata`);
}
if (support.catalog_counts?.classification !== "metadata-only") failures.push("support matrix: provider count lost metadata-only scope");
if (support.catalog_counts?.numeric_total != null) {
  const catalog = JSON.parse(read("vendor/git/liter-llm/crates/liter-llm/schemas/catalog.json"));
  if (!catalog.providers || Array.isArray(catalog.providers)
    || Object.keys(catalog.providers).length !== support.catalog_counts.numeric_total) {
    failures.push("support matrix: numeric provider total disagrees with pinned provider map");
  }
}
for (const row of support.providers ?? []) {
  if (row.capabilities?.includes("vision") && /unverified|NOT_EXERCISED|source.authored/iu.test(row.current_candidate_evidence ?? "")) {
    failures.push(`support matrix: unqualified vision advertised in provider capabilities: ${row.id}`);
  }
}
if (support.runtime_measurement?.comparison_scope !== "NONE") {
  failures.push("support matrix: selected runtime comparison scope is no longer the recorded NONE disposition");
}
const fullHarness = support.execution_recovery?.interfaces?.find((row) => row.id === "full-harness");
if (fullHarness?.durability !== "process_ephemeral" || fullHarness?.recovery !== "unsupported_after_restart"
  || fullHarness?.steer_supported !== false) {
  failures.push("support matrix: full-harness recovery boundary drift");
}

const canonical = [
  "README.md",
  "package.json",
  "docs/ARCHITECTURE.md",
  "docs/frontend-architecture.md",
  "docs/configuration.md",
  "docs/product-support-matrix.md",
  "website/docs/intro.md",
  "website/docs/api-reference.md",
  "website/docs/installation.md",
  "website/docs/troubleshooting.md",
  "website/docs/upgrade-guide.md",
];
const prohibited = [
  [/142\+ providers/gi, "blanket 142+ provider claim"],
  [/\b(?:no|without) React\b/gi, "retired no-React claim"],
  [/run identically[^\n]*(?:desktop|mobile)/gi, "identical-platform claim"],
  [/build\.rs[^\n]*(?:fetch|download)[^\n]*models\.dev/gi, "networked ordinary-build claim"],
  [/\bproduction[- ]ready\b/gi, "unscoped production-ready claim"],
  [/tribehealth\/universal-agent-runtime/gi, "retired container registry"],
  [/\bbun (?:install|run)\b/gi, "retired Bun release toolchain"],
  [/persistence[^\n]*(?:have|has) no compiled defaults/gi, "retired missing-persistence-default claim"],
];

for (const path of canonical) {
  const body = read(path);
  for (const [pattern, label] of prohibited) {
    pattern.lastIndex = 0;
    if (pattern.test(body)) failures.push(`${path}: ${label}`);
  }
}

if (!read("docker-compose.prod.yaml").includes("image: ghcr.io/prometheus-ags/universal-agent-runtime:v1.0.0")) {
  failures.push("docker-compose.prod.yaml: Stable image is not the pinned GHCR release");
}

const linkPattern = /\[[^\]]+\]\(([^)]+)\)/g;
const portalRoutes = docusaurusRoutes();
for (const path of canonical.filter((value) => extname(value) === ".md")) {
  for (const match of read(path).matchAll(linkPattern)) {
    const target = match[1].trim().replace(/^<|>$/g, "").split(/[?#]/, 1)[0].replace(/\/$/, "");
    if (!target || /^(?:https?:|mailto:)/.test(target)) continue;
    if (target.startsWith("/docs/")) {
      if (!portalRoutes.has(target)) failures.push(`${path}: broken Docusaurus route ${match[1]}`);
      continue;
    }
    const destination = resolve(root, dirname(path), decodeURIComponent(target));
    const markdownDestination = `${destination}.md`;
    if (!existsSync(destination) && !existsSync(markdownDestination)) failures.push(`${path}: broken link ${match[1]}`);
    else if (existsSync(destination) && statSync(destination).isDirectory()) failures.push(`${path}: link targets directory ${match[1]}`);
  }
}

if (failures.length) {
  console.error(`Documentation truth gate failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log(`Documentation truth gate passed (${canonical.length} canonical files).`);
