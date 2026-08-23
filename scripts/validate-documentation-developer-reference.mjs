#!/usr/bin/env node

import {existsSync, readFileSync, readdirSync} from 'node:fs';
import {basename, dirname, extname, join, normalize, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), '..');
const manifestPath = 'docs/publication/developer-reference.json';
const sourceManifestPath = 'docs/publication/sources.json';
const expectedIds = [
  'api-reference',
  'protocol-overview',
  'http-compatibility',
  'events-and-ui',
  'mcp',
  'a2a',
  'tools',
  'sdks',
  'rust-sdk',
  'python-sdk',
  'typescript-sdk',
  'configuration',
  'installation',
  'deployment',
  'upgrade',
];
const expectedFiles = [
  'website/docs/api/index.md',
  'website/docs/protocols/overview.md',
  'website/docs/protocols/http-compatibility.md',
  'website/docs/protocols/events-and-ui.md',
  'website/docs/protocols/mcp.md',
  'website/docs/protocols/a2a.md',
  'website/docs/tools/overview.md',
  'website/docs/sdks.md',
  'website/docs/sdk-rust/intro.md',
  'website/docs/sdk-python/intro.md',
  'website/docs/sdk-typescript/intro.md',
  'website/docs/configuration.md',
  'website/docs/installation.md',
  'website/docs/deployment.md',
  'website/docs/upgrade-guide.md',
];
const expectedRoutes = [
  '/docs/api',
  '/docs/protocols/overview',
  '/docs/protocols/http-compatibility',
  '/docs/protocols/events-and-ui',
  '/docs/protocols/mcp',
  '/docs/protocols/a2a',
  '/docs/tools/overview',
  '/docs/sdks',
  '/docs/sdk-rust/intro',
  '/docs/sdk-python/intro',
  '/docs/sdk-typescript/intro',
  '/docs/configuration',
  '/docs/installation',
  '/docs/deployment',
  '/docs/upgrade-guide',
];
const expectedProfiles = ['server-full', 'minimal', 'embedded-mobile'];
const allowedDispositions = new Set(['public', 'public-normalize', 'private-synthesis-only']);
const unsafeContent = [
  [/(?:^|[^A-Za-z])\/Users\/[^/\s]+\//mu, 'machine-local macOS path'],
  [/(?:^|[^A-Za-z])\/home\/[A-Za-z0-9._-]+\//mu, 'machine-local Linux path'],
  [/[A-Za-z]:\\Users\\[^\\\s]+\\/mu, 'machine-local Windows path'],
  [/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/mu, 'private-key material'],
  [/(?:api[_-]?key|access[_-]?token|password|client[_-]?secret)\s*[:=]\s*["'][A-Za-z0-9_./+-]{12,}["']/imu, 'credential-shaped assignment'],
  [/\/v9\/imaginary/mu, 'invented endpoint claim'],
  [/OpenAPI (?:is|provides) (?:the )?(?:complete|exhaustive) (?:API|route)/imu, 'exhaustive OpenAPI claim'],
  [/(?:complete|full) (?:OpenAI|Anthropic|AG-UI|A2UI|MCP|A2A) (?:parity|compatibility|conformance)/imu, 'complete protocol-parity claim'],
  [/discovery (?:automatically )?(?:grants|provides|means) authorization/imu, 'discovery-as-authorization claim'],
  [/uar-jwt-proxy (?:is|provides|acts as) (?:a )?production (?:authentication )?gateway/imu, 'production JWT-proxy claim'],
  [/(?:hosted|published) (?:generated )?(?:Python|Sphinx) (?:API )?reference/imu, 'hosted Python reference claim'],
  [/(?:package|version) metadata (?:proves|confirms|guarantees) (?:registry )?publication/imu, 'registry-from-metadata claim'],
  [/jwt_required:\s*false[\s\S]{0,180}0\.0\.0\.0/imu, 'unsafe anonymous listener example'],
  [/(?:server-full|minimal|embedded-mobile) evidence (?:automatically )?(?:applies|transfers) to (?:server-full|minimal|embedded-mobile)/imu, 'cross-profile evidence claim'],
  [/GitHub Actions (?:runs|performs) (?:unit|integration|lint|typecheck|routine) (?:tests|checks|verification)/imu, 'routine GitHub Actions test claim'],
];

const read = (root, path) => readFileSync(join(root, path), 'utf8');

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

function classify(path, sourceManifest) {
  return (sourceManifest.rules ?? []).filter((rule) => selectorMatches(path, rule.selector));
}

function frontmatter(body) {
  if (!body.startsWith('---\n')) return null;
  const end = body.indexOf('\n---\n', 4);
  if (end < 0) return null;
  const records = [];
  let inRecords = false;
  let authority = null;
  for (const line of body.slice(4, end).split(/\r?\n/u)) {
    if (/^source_records:\s*$/u.test(line)) {
      inRecords = true;
      continue;
    }
    const item = line.match(/^\s+-\s+["']?([^"']+?)["']?\s*$/u);
    if (inRecords && item) {
      records.push(item[1]);
      continue;
    }
    if (/^\S/u.test(line)) inRecords = false;
    const match = line.match(/^current_authority:\s*["']?([^"'\n]+)["']?\s*$/u);
    if (match) authority = match[1];
  }
  return {records, authority};
}

function collectDocRoutes(root) {
  const routes = new Set();
  const docsRoot = join(root, 'website/docs');
  if (!existsSync(docsRoot)) return routes;
  function walk(directory) {
    for (const entry of readdirSync(directory, {withFileTypes: true})) {
      const absolute = join(directory, entry.name);
      if (entry.isDirectory()) walk(absolute);
      else if (entry.isFile() && ['.md', '.mdx'].includes(extname(entry.name)) && !entry.name.startsWith('_')) {
        let relative = absolute.slice(docsRoot.length + 1).replace(/\\/gu, '/').replace(/\.(?:md|mdx)$/u, '');
        if (relative.endsWith('/index')) relative = relative.slice(0, -6);
        routes.add(`/docs/${relative}`.replace(/\/$/u, ''));
      }
    }
  }
  walk(docsRoot);
  return routes;
}

function validateHeadings(path, body, failures) {
  const levels = [...body.matchAll(/^(#{1,6})\s+\S/gmu)].map((match) => match[1].length);
  if (levels.filter((level) => level === 1).length !== 1) failures.push(`${path}: expected exactly one level-one heading`);
  for (let index = 1; index < levels.length; index += 1) {
    if (levels[index] > levels[index - 1] + 1) {
      failures.push(`${path}: heading level jumps from h${levels[index - 1]} to h${levels[index]}`);
      break;
    }
  }
}

function localLinkExists(root, sourcePath, href, knownRoutes) {
  const target = href.split('#', 1)[0].split('?', 1)[0];
  if (!target || target.startsWith('#') || /^[a-z][a-z0-9+.-]*:/iu.test(target)) return true;
  if (target.startsWith('/')) return knownRoutes.has(target.replace(/\/$/u, ''));
  const candidate = normalize(join(dirname(sourcePath), target));
  if (existsSync(join(root, candidate))) return true;
  if (!extname(candidate) && existsSync(join(root, `${candidate}.md`))) return true;
  if (!extname(candidate) && existsSync(join(root, candidate, 'index.md'))) return true;
  return false;
}

function validateLinks(root, path, body, knownRoutes, failures) {
  for (const match of body.matchAll(/\[[^\]]+\]\(([^)\s]+)(?:\s+["'][^"']*["'])?\)/gu)) {
    if (!localLinkExists(root, path, match[1], knownRoutes)) failures.push(`${path}: local link does not resolve: ${match[1]}`);
  }
}

function validateCompatibility(root, manifest, knownRoutes, failures) {
  const expected = [
    ['website/docs/api-reference.md', '/docs/api-reference', '/docs/api'],
    ['website/docs/configuration/intro.md', '/docs/configuration/intro', '/docs/configuration'],
    ['website/docs/dev-tools/intro.md', '/docs/dev-tools/intro', '/docs/tools/overview'],
    ['website/docs/a2ui/intro.md', '/docs/a2ui/intro', '/docs/protocols/events-and-ui'],
  ];
  const documents = manifest.compatibilityDocuments;
  if (!Array.isArray(documents) || documents.length !== expected.length) {
    failures.push(`${manifestPath}: expected exactly ${expected.length} compatibility documents`);
    return;
  }
  for (const [index, document] of documents.entries()) {
    const [file, route, authority] = expected[index];
    if (document.file !== file || document.route !== route || document.currentAuthority !== authority) {
      failures.push(`${manifestPath}: compatibility document ${index + 1} contract is invalid`);
      continue;
    }
    if (!existsSync(join(root, file))) {
      failures.push(`${file}: required compatibility document is missing`);
      continue;
    }
    const body = read(root, file);
    if (!body.includes(`](${authority})`)) failures.push(`${file}: current-authority link is missing: ${authority}`);
    if (body.length > 3000) failures.push(`${file}: compatibility page duplicates the detailed authority`);
    validateHeadings(file, body, failures);
    validateLinks(root, file, body, knownRoutes, failures);
    for (const [pattern, label] of unsafeContent) if (pattern.test(body)) failures.push(`${file}: publication safety rejected ${label}`);
  }
}

export function validateDocumentationDeveloperReference({root = defaultRoot} = {}) {
  const resolvedRoot = resolve(root);
  const failures = [];
  if (!existsSync(join(resolvedRoot, manifestPath))) return {failures: [`${manifestPath} is missing`], guideCount: 0};
  if (!existsSync(join(resolvedRoot, sourceManifestPath))) return {failures: [`${sourceManifestPath} is missing`], guideCount: 0};

  let manifest;
  let sourceManifest;
  try {
    manifest = JSON.parse(read(resolvedRoot, manifestPath));
    sourceManifest = JSON.parse(read(resolvedRoot, sourceManifestPath));
  } catch (error) {
    return {failures: [`developer-reference publication manifest is invalid JSON: ${error.message}`], guideCount: 0};
  }

  if (manifest.schemaVersion !== 1) failures.push(`${manifestPath}: schemaVersion must be 1`);
  if (JSON.stringify(manifest.profiles) !== JSON.stringify(expectedProfiles)) failures.push(`${manifestPath}: profiles must list server-full, minimal, and embedded-mobile exactly once`);
  if (!Array.isArray(manifest.guides) || manifest.guides.length !== expectedIds.length) {
    failures.push(`${manifestPath}: expected exactly ${expectedIds.length} guides`);
    return {failures, guideCount: manifest.guides?.length ?? 0};
  }
  if (JSON.stringify(manifest.guides.map((guide) => guide.id)) !== JSON.stringify(expectedIds)) failures.push(`${manifestPath}: guides are missing or out of dependency order`);

  const knownRoutes = collectDocRoutes(resolvedRoot);
  for (const route of expectedRoutes) knownRoutes.add(route);
  for (const document of manifest.compatibilityDocuments ?? []) knownRoutes.add(document.route);

  for (const [index, guide] of manifest.guides.entries()) {
    const label = guide.id ?? `<guide-${index + 1}>`;
    if (guide.position !== index + 1) failures.push(`${manifestPath}: ${label} has invalid position`);
    if (guide.file !== expectedFiles[index]) failures.push(`${manifestPath}: ${label} file must be ${expectedFiles[index]}`);
    if (guide.route !== expectedRoutes[index]) failures.push(`${manifestPath}: ${label} route must be ${expectedRoutes[index]}`);
    if (!guide.title) failures.push(`${manifestPath}: ${label} has no title`);
    if (!Array.isArray(guide.profiles) || guide.profiles.length === 0 || guide.profiles.some((profile) => !expectedProfiles.includes(profile))) failures.push(`${manifestPath}: ${label} has missing or invalid profile limits`);
    for (const field of ['sourceRecords', 'sourceAuthorities', 'requiredMarkers']) {
      if (!Array.isArray(guide[field]) || guide[field].length === 0) failures.push(`${manifestPath}: ${label} has no ${field}`);
    }
    for (const source of [...(guide.sourceRecords ?? []), ...(guide.sourceAuthorities ?? [])]) {
      if (!existsSync(join(resolvedRoot, source))) failures.push(`${manifestPath}: ${label} source does not exist: ${source}`);
    }
    for (const record of guide.sourceRecords ?? []) {
      const classifications = classify(record, sourceManifest);
      if (classifications.length === 0) failures.push(`${manifestPath}: ${label} source record is unclassified: ${record}`);
      else if (classifications.length > 1) failures.push(`${manifestPath}: ${label} source record is ambiguously classified: ${record}`);
      else if (!allowedDispositions.has(classifications[0].disposition)) failures.push(`${manifestPath}: ${label} source record is excluded: ${record}`);
    }

    if (!existsSync(join(resolvedRoot, guide.file))) {
      failures.push(`${guide.file}: required developer-reference guide is missing`);
      continue;
    }
    const body = read(resolvedRoot, guide.file);
    const metadata = frontmatter(body);
    if (!metadata) failures.push(`${guide.file}: publication frontmatter is missing`);
    else {
      if (metadata.authority !== guide.route) failures.push(`${guide.file}: current_authority must be ${guide.route}`);
      for (const record of guide.sourceRecords ?? []) if (!metadata.records.includes(record)) failures.push(`${guide.file}: source_records is missing ${record}`);
    }
    validateHeadings(guide.file, body, failures);
    for (const marker of guide.requiredMarkers ?? []) {
      if (!body.toLocaleLowerCase('en').includes(marker.toLocaleLowerCase('en'))) failures.push(`${guide.file}: required marker is missing: ${marker}`);
    }
    if (guide.requiresMermaid) {
      if (!/```mermaid\s[\s\S]*?```/u.test(body)) failures.push(`${guide.file}: required Mermaid diagram is missing`);
      if (!/^## Diagram in words\s*$/mu.test(body)) failures.push(`${guide.file}: diagram explanation is missing`);
    }
    for (const [pattern, labelText] of unsafeContent) if (pattern.test(body)) failures.push(`${guide.file}: publication safety rejected ${labelText}`);
    validateLinks(resolvedRoot, guide.file, body, knownRoutes, failures);
  }

  validateCompatibility(resolvedRoot, manifest, knownRoutes, failures);
  return {failures, guideCount: manifest.guides.length};
}

function main() {
  const root = process.argv[2] ?? defaultRoot;
  const result = validateDocumentationDeveloperReference({root});
  if (result.failures.length) {
    console.error(`Documentation developer-reference validation failed:\n- ${result.failures.join('\n- ')}`);
    process.exit(1);
  }
  console.log(`Documentation developer-reference validation passed (${result.guideCount} guides).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
