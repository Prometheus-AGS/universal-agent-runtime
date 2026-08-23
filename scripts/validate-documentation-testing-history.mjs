#!/usr/bin/env node

import {existsSync, readFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), '..');
const expectedGuides = ['testing-methodology', 'evidence-taxonomy', 'negative-controls', 'local-verification'];
const expectedClasses = ['static-type', 'focused-unit-component', 'synthetic-recorded', 'packaged-functional', 'real-model-functional', 'load-soak-resilience', 'deployment-validation'];
const expectedProfiles = ['server-full', 'minimal', 'embedded-mobile'];

const read = (root, path) => readFileSync(join(root, path), 'utf8');

function frontmatter(body) {
  if (!body.startsWith('---\n')) return null;
  const end = body.indexOf('\n---\n', 4);
  if (end < 0) return null;
  const header = body.slice(4, end);
  const authority = header.match(/^current_authority:\s*["']?([^"'\n]+)["']?\s*$/mu)?.[1];
  const records = [];
  let reading = false;
  for (const line of header.split(/\r?\n/u)) {
    if (/^source_records:\s*$/u.test(line)) {
      reading = true;
      continue;
    }
    const item = reading ? line.match(/^\s+-\s+(.+?)\s*$/u) : null;
    if (item) records.push(item[1].replace(/^["']|["']$/gu, ''));
    else if (/^\S/u.test(line)) reading = false;
  }
  return {authority, records, content: body.slice(end + 5).trim()};
}

export function validateDocumentationTestingHistory({root = defaultRoot} = {}) {
  const resolvedRoot = resolve(root);
  const failures = [];
  const manifestPath = 'docs/publication/testing-history.json';
  if (!existsSync(join(resolvedRoot, manifestPath))) return {failures: [`${manifestPath} is missing`], guideCount: 0, evidenceClassCount: 0};

  let manifest;
  try {
    manifest = JSON.parse(read(resolvedRoot, manifestPath));
  } catch (error) {
    return {failures: [`${manifestPath} is invalid JSON: ${error.message}`], guideCount: 0, evidenceClassCount: 0};
  }

  if (manifest.schemaVersion !== 1) failures.push(`${manifestPath}: schemaVersion must be 1`);
  if (JSON.stringify(manifest.profiles) !== JSON.stringify(expectedProfiles)) failures.push(`${manifestPath}: profiles must remain server-full, minimal, and embedded-mobile without transfer`);
  if (JSON.stringify((manifest.guides ?? []).map((guide) => guide.id)) !== JSON.stringify(expectedGuides)) failures.push(`${manifestPath}: testing-history guides are missing or out of order`);
  if (JSON.stringify((manifest.evidenceClasses ?? []).map((item) => item.id)) !== JSON.stringify(expectedClasses)) failures.push(`${manifestPath}: evidence classes are missing or out of order`);

  const privateBodies = new Map();
  for (const source of manifest.sources ?? []) {
    if (!existsSync(join(resolvedRoot, source))) failures.push(`${manifestPath}: source does not exist: ${source}`);
    if (source.includes('.prometheus/knowledge/wiki/')) failures.push(`${manifestPath}: unreviewed wiki source is forbidden: ${source}`);
    if ((source.startsWith('.prometheus/') || source.startsWith('.kbd-orchestrator/')) && existsSync(join(resolvedRoot, source))) privateBodies.set(source, read(resolvedRoot, source).trim());
  }

  for (const guide of manifest.guides ?? []) {
    if (guide.file !== `website/docs/history/${guide.id}.md`) failures.push(`${manifestPath}: ${guide.id} has an invalid file`);
    if (guide.route !== `/docs/history/${guide.id}`) failures.push(`${manifestPath}: ${guide.id} has an invalid route`);
    if (!existsSync(join(resolvedRoot, guide.file))) {
      failures.push(`${guide.file}: required testing-history guide is missing`);
      continue;
    }
    const body = read(resolvedRoot, guide.file);
    const meta = frontmatter(body);
    if (!meta) failures.push(`${guide.file}: publication frontmatter is missing`);
    else {
      if (meta.authority !== guide.route) failures.push(`${guide.file}: current_authority must be ${guide.route}`);
      for (const source of meta.records) {
        if (!existsSync(join(resolvedRoot, source))) failures.push(`${guide.file}: source record does not exist: ${source}`);
        if (source.includes('.prometheus/knowledge/wiki/')) failures.push(`${guide.file}: unreviewed wiki source is forbidden: ${source}`);
        if (privateBodies.get(source) === meta.content) failures.push(`${guide.file}: exact private history copy is forbidden: ${source}`);
      }
    }
    for (const marker of guide.requiredMarkers ?? []) if (!body.toLocaleLowerCase('en').includes(marker.toLocaleLowerCase('en'))) failures.push(`${guide.file}: required marker is missing: ${marker}`);
    if (/GitHub Actions\s+(?:run|runs|execute|executes)\s+(?:unit|integration|conformance|lint|format|type|routine)/iu.test(body)) failures.push(`${guide.file}: routine GitHub Actions testing is forbidden`);
  }

  for (const item of manifest.evidenceClasses ?? []) {
    if (!item.proves) failures.push(`${manifestPath}: ${item.id} is missing proves`);
    if (!item.doesNotProve) failures.push(`${manifestPath}: ${item.id} is missing doesNotProve`);
  }
  const synthetic = manifest.evidenceClasses?.find((item) => item.id === 'synthetic-recorded');
  if (/genuine model inference/iu.test(synthetic?.proves ?? '') || !/genuine model inference/iu.test(synthetic?.doesNotProve ?? '')) failures.push(`${manifestPath}: synthetic/recorded evidence must not certify genuine model inference`);
  const realModel = manifest.evidenceClasses?.find((item) => item.id === 'real-model-functional');
  if (!/genuine inference/iu.test(realModel?.proves ?? '')) failures.push(`${manifestPath}: real-model evidence must cross genuine inference`);
  const soak = manifest.evidenceClasses?.find((item) => item.id === 'load-soak-resilience');
  if (!/(failure model|volume|operating period|statistical objective)/iu.test(soak?.proves ?? '')) failures.push(`${manifestPath}: load/soak evidence requires an objective beyond elapsed duration`);

  return {failures, guideCount: manifest.guides?.length ?? 0, evidenceClassCount: manifest.evidenceClasses?.length ?? 0};
}

function main() {
  const result = validateDocumentationTestingHistory({root: process.argv[2] ?? defaultRoot});
  if (result.failures.length) {
    console.error(`Documentation testing-history validation failed:\n- ${result.failures.join('\n- ')}`);
    process.exit(1);
  }
  console.log(`Documentation testing-history validation passed (${result.guideCount} guides, ${result.evidenceClassCount} evidence classes).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
