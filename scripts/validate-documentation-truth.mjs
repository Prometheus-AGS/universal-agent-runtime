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

if (manifest.version !== cargoVersion) failures.push(`version mismatch: package.json=${manifest.version}, Cargo.toml=${cargoVersion}`);
if (manifest.license !== cargoLicense) failures.push(`license mismatch: package.json=${manifest.license}, Cargo.toml=${cargoLicense}`);

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
