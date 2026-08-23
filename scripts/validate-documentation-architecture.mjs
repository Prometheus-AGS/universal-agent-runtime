#!/usr/bin/env node

import {existsSync, readFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), '..');
const allowedProfiles = new Set(['server-full', 'minimal', 'embedded-mobile']);
const expectedIds = [
  'intro',
  'trust-boundary',
  'execution-lifecycle',
  'state-and-events',
  'profiles',
  'protocols',
  'delegation',
];

const read = (root, path) => readFileSync(join(root, path), 'utf8');

function frontmatter(body) {
  if (!body.startsWith('---\n')) return null;
  const end = body.indexOf('\n---\n', 4);
  if (end < 0) return null;
  const header = body.slice(4, end);
  const records = [];
  let inRecords = false;
  let authority = null;
  for (const line of header.split(/\r?\n/u)) {
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

export function validateDocumentationArchitecture({root = defaultRoot} = {}) {
  const resolvedRoot = resolve(root);
  const failures = [];
  const manifestPath = 'docs/publication/architecture.json';
  if (!existsSync(join(resolvedRoot, manifestPath))) return {failures: [`${manifestPath} is missing`], guideCount: 0};

  let manifest;
  try {
    manifest = JSON.parse(read(resolvedRoot, manifestPath));
  } catch (error) {
    return {failures: [`${manifestPath} is invalid JSON: ${error.message}`], guideCount: 0};
  }

  if (manifest.schemaVersion !== 1) failures.push(`${manifestPath}: schemaVersion must be 1`);
  if (JSON.stringify(manifest.profiles) !== JSON.stringify([...allowedProfiles])) {
    failures.push(`${manifestPath}: profiles must list server-full, minimal, and embedded-mobile exactly once`);
  }
  if (!Array.isArray(manifest.guides) || manifest.guides.length !== expectedIds.length) {
    failures.push(`${manifestPath}: expected exactly ${expectedIds.length} guides`);
    return {failures, guideCount: manifest.guides?.length ?? 0};
  }

  const ids = manifest.guides.map((guide) => guide.id);
  if (JSON.stringify(ids) !== JSON.stringify(expectedIds)) failures.push(`${manifestPath}: guides are missing or out of conceptual order`);
  const routes = new Set();
  const files = new Set();

  for (const [index, guide] of manifest.guides.entries()) {
    const label = guide.id ?? `<guide-${index + 1}>`;
    if (guide.position !== index + 1) failures.push(`${manifestPath}: ${label} has invalid position`);
    if (!guide.title) failures.push(`${manifestPath}: ${label} has no title`);
    if (routes.has(guide.route)) failures.push(`${manifestPath}: duplicate route ${guide.route}`);
    if (files.has(guide.file)) failures.push(`${manifestPath}: duplicate file ${guide.file}`);
    routes.add(guide.route);
    files.add(guide.file);

    const expectedRoute = `/docs/architecture/${guide.id}`;
    const expectedFile = `website/docs/architecture/${guide.id}.md`;
    if (guide.route !== expectedRoute) failures.push(`${manifestPath}: ${label} route must be ${expectedRoute}`);
    if (guide.file !== expectedFile) failures.push(`${manifestPath}: ${label} file must be ${expectedFile}`);
    if (!Array.isArray(guide.profiles) || guide.profiles.length === 0 || guide.profiles.some((profile) => !allowedProfiles.has(profile))) {
      failures.push(`${manifestPath}: ${label} has missing or invalid profiles`);
    }
    if (new Set(guide.profiles ?? []).size !== (guide.profiles ?? []).length) failures.push(`${manifestPath}: ${label} has duplicate profiles`);

    for (const field of ['sourceRecords', 'sourceAuthorities', 'requiredMarkers']) {
      if (!Array.isArray(guide[field]) || guide[field].length === 0) failures.push(`${manifestPath}: ${label} has no ${field}`);
    }
    for (const source of [...(guide.sourceRecords ?? []), ...(guide.sourceAuthorities ?? [])]) {
      if (!existsSync(join(resolvedRoot, source))) failures.push(`${manifestPath}: ${label} source does not exist: ${source}`);
    }
    for (const record of guide.sourceRecords ?? []) {
      if (!record.startsWith('openspec/specs/') && !record.startsWith('docs/')) failures.push(`${manifestPath}: ${label} publication record is not classified: ${record}`);
    }

    if (!existsSync(join(resolvedRoot, guide.file))) {
      failures.push(`${guide.file}: required architecture guide is missing`);
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
  }

  return {failures, guideCount: manifest.guides.length};
}

function main() {
  const root = process.argv[2] ?? defaultRoot;
  const result = validateDocumentationArchitecture({root});
  if (result.failures.length) {
    console.error(`Documentation architecture validation failed:\n- ${result.failures.join('\n- ')}`);
    process.exit(1);
  }
  console.log(`Documentation architecture validation passed (${result.guideCount} guides).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
