#!/usr/bin/env node

import {existsSync, readFileSync} from 'node:fs';
import {dirname, extname, join, normalize, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), '..');
const manifestPath = 'docs/publication/product-workflows.json';
const sourceManifestPath = 'docs/publication/sources.json';
const expectedIds = [
  'provider-configuration',
  'provider-models',
  'provider-inference',
  'agents',
  'skills',
  'knowledge',
  'memory',
];
const expectedFiles = [
  'website/docs/providers/configuration.md',
  'website/docs/providers/models.md',
  'website/docs/providers/inference.md',
  'website/docs/agents/overview.md',
  'website/docs/skills/overview.md',
  'website/docs/knowledge/overview.md',
  'website/docs/memory/overview.md',
];
const expectedRoutes = [
  '/docs/providers/configuration',
  '/docs/providers/models',
  '/docs/providers/inference',
  '/docs/agents/overview',
  '/docs/skills/overview',
  '/docs/knowledge/overview',
  '/docs/memory/overview',
];
const expectedProfiles = ['server-full', 'minimal', 'embedded-mobile'];
const allowedDispositions = new Set(['public', 'public-normalize', 'private-synthesis-only']);
const unsafeContent = [
  [/(?:^|[^A-Za-z])\/Users\/[^/\s]+\//mu, 'machine-local macOS path'],
  [/(?:^|[^A-Za-z])\/home\/[A-Za-z0-9._-]+\//mu, 'machine-local Linux path'],
  [/[A-Za-z]:\\Users\\[^\\\s]+\\/mu, 'machine-local Windows path'],
  [/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/mu, 'private-key material'],
  [/(?:api[_-]?key|access[_-]?token|password|client[_-]?secret)\s*[:=]\s*["'][A-Za-z0-9_./+-]{12,}["']/imu, 'credential-shaped assignment'],
  [/["'](?:eventId|integrityHash|session_id|conversation_id)["']\s*:/mu, 'raw event or session payload'],
];

const read = (root, path) => readFileSync(join(root, path), 'utf8');

function selectorMatches(path, selector = {}) {
  const basename = path.split('/').at(-1);
  const included =
    (selector.paths ?? []).includes(path) ||
    (selector.prefixes ?? []).some((prefix) => path.startsWith(prefix)) ||
    (selector.basenames ?? []).includes(basename);
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

export function validateDocumentationProductWorkflows({root = defaultRoot} = {}) {
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
    return {failures: [`product workflow publication manifest is invalid JSON: ${error.message}`], guideCount: 0};
  }

  if (manifest.schemaVersion !== 1) failures.push(`${manifestPath}: schemaVersion must be 1`);
  if (JSON.stringify(manifest.profiles) !== JSON.stringify(expectedProfiles)) {
    failures.push(`${manifestPath}: profiles must list server-full, minimal, and embedded-mobile exactly once`);
  }
  if (!Array.isArray(manifest.guides) || manifest.guides.length !== expectedIds.length) {
    failures.push(`${manifestPath}: expected exactly ${expectedIds.length} guides`);
    return {failures, guideCount: manifest.guides?.length ?? 0};
  }

  if (JSON.stringify(manifest.guides.map((guide) => guide.id)) !== JSON.stringify(expectedIds)) {
    failures.push(`${manifestPath}: guides are missing or out of dependency order`);
  }
  const knownRoutes = new Set([
    ...expectedRoutes,
    '/docs/skills',
    '/docs/architecture/profiles',
  ]);

  for (const [index, guide] of manifest.guides.entries()) {
    const label = guide.id ?? `<guide-${index + 1}>`;
    if (guide.position !== index + 1) failures.push(`${manifestPath}: ${label} has invalid position`);
    if (guide.file !== expectedFiles[index]) failures.push(`${manifestPath}: ${label} file must be ${expectedFiles[index]}`);
    if (guide.route !== expectedRoutes[index]) failures.push(`${manifestPath}: ${label} route must be ${expectedRoutes[index]}`);
    if (!guide.title) failures.push(`${manifestPath}: ${label} has no title`);
    if (JSON.stringify(guide.profiles) !== JSON.stringify(expectedProfiles)) failures.push(`${manifestPath}: ${label} has missing or invalid profile limits`);
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
      failures.push(`${guide.file}: required product workflow guide is missing`);
      continue;
    }
    const body = read(resolvedRoot, guide.file);
    const metadata = frontmatter(body);
    if (!metadata) failures.push(`${guide.file}: publication frontmatter is missing`);
    else {
      if (metadata.authority !== guide.route) failures.push(`${guide.file}: current_authority must be ${guide.route}`);
      for (const record of guide.sourceRecords ?? []) {
        if (!metadata.records.includes(record)) failures.push(`${guide.file}: source_records is missing ${record}`);
      }
    }
    validateHeadings(guide.file, body, failures);
    for (const marker of guide.requiredMarkers ?? []) {
      if (!body.toLocaleLowerCase('en').includes(marker.toLocaleLowerCase('en'))) failures.push(`${guide.file}: required marker is missing: ${marker}`);
    }
    if (guide.requiresMermaid) {
      if (!/```mermaid\s[\s\S]*?```/u.test(body)) failures.push(`${guide.file}: required Mermaid diagram is missing`);
      if (!/^## Diagram in words\s*$/mu.test(body)) failures.push(`${guide.file}: diagram explanation is missing`);
    }
    if (guide.id === 'provider-inference' && /(?:synthetic|mock(?:ed)?)\b[^\n]{0,60}\b(?:genuine|real model|certif)/iu.test(body)) {
      failures.push(`${guide.file}: synthetic evidence is represented as genuine inference evidence`);
    }
    for (const [pattern, labelText] of unsafeContent) {
      if (pattern.test(body)) failures.push(`${guide.file}: publication sanitizer rejected ${labelText}`);
    }
    validateLinks(resolvedRoot, guide.file, body, knownRoutes, failures);
  }

  const compatibility = manifest.compatibilityDocuments;
  if (!Array.isArray(compatibility) || compatibility.length !== 1) failures.push(`${manifestPath}: expected exactly one compatibility document`);
  else {
    const [document] = compatibility;
    if (document.file !== 'website/docs/skills.md' || document.route !== '/docs/skills' || document.currentAuthority !== '/docs/skills/overview') {
      failures.push(`${manifestPath}: skills compatibility document contract is invalid`);
    } else if (!existsSync(join(resolvedRoot, document.file))) {
      failures.push(`${document.file}: required compatibility document is missing`);
    } else {
      const body = read(resolvedRoot, document.file);
      for (const route of [
        '/docs/skills/overview',
        'https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/skill-pack-installation.md',
        'https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/skill-authoring.md',
      ]) {
        if (!body.includes(`](${route})`)) failures.push(`${document.file}: compatibility link is missing: ${route}`);
      }
      if (body.length > 2500 || /(?:conversation|agent|global)\s+(?:scope|precedence)/iu.test(body)) {
        failures.push(`${document.file}: compatibility page duplicates the lifecycle authority`);
      }
      validateHeadings(document.file, body, failures);
      validateLinks(resolvedRoot, document.file, body, knownRoutes, failures);
      for (const [pattern, labelText] of unsafeContent) {
        if (pattern.test(body)) failures.push(`${document.file}: publication sanitizer rejected ${labelText}`);
      }
    }
  }

  return {failures, guideCount: manifest.guides.length};
}

function main() {
  const root = process.argv[2] ?? defaultRoot;
  const result = validateDocumentationProductWorkflows({root});
  if (result.failures.length) {
    console.error(`Documentation product-workflow validation failed:\n- ${result.failures.join('\n- ')}`);
    process.exit(1);
  }
  console.log(`Documentation product-workflow validation passed (${result.guideCount} guides).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
