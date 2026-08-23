#!/usr/bin/env node

import {existsSync, readFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), '..');
const expectedGuides = ['overview', 'architecture-decisions', 'timeline', 'corrections', 'process-provenance'];
const expectedDecisions = ['license-mit', 'react-first', 'flat2-authority', 'real-pages-artifacts', 'rustcrypto-jwt', 'local-verification', 'real-inference-evidence'];
const allowedStatuses = new Set(['accepted', 'superseded']);

const read = (root, path) => readFileSync(join(root, path), 'utf8');

function metadata(body) {
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

export function validateDocumentationHistory({root = defaultRoot} = {}) {
  const resolvedRoot = resolve(root);
  const failures = [];
  const manifestPath = 'docs/publication/architecture-history.json';
  if (!existsSync(join(resolvedRoot, manifestPath))) return {failures: [`${manifestPath} is missing`], guideCount: 0, decisionCount: 0};

  let manifest;
  try {
    manifest = JSON.parse(read(resolvedRoot, manifestPath));
  } catch (error) {
    return {failures: [`${manifestPath} is invalid JSON: ${error.message}`], guideCount: 0, decisionCount: 0};
  }

  if (manifest.schemaVersion !== 1) failures.push(`${manifestPath}: schemaVersion must be 1`);
  for (const field of ['prometheusFiles', 'prometheusWikiFiles', 'kbdPhaseDirectories', 'kbdReflections', 'openSpecChangeDirectories', 'retainedAdrs']) {
    if (!Number.isInteger(manifest.snapshot?.[field]) || manifest.snapshot[field] < 0) failures.push(`${manifestPath}: snapshot.${field} must be a non-negative integer`);
  }
  if (!manifest.snapshot?.selectionRule) failures.push(`${manifestPath}: snapshot selectionRule is required`);

  if (JSON.stringify((manifest.guides ?? []).map((guide) => guide.id)) !== JSON.stringify(expectedGuides)) {
    failures.push(`${manifestPath}: history guides are missing or out of order`);
  }
  const routes = new Set();
  for (const guide of manifest.guides ?? []) {
    if (routes.has(guide.route)) failures.push(`${manifestPath}: duplicate guide route ${guide.route}`);
    routes.add(guide.route);
    if (guide.file !== `website/docs/history/${guide.id}.md`) failures.push(`${manifestPath}: ${guide.id} has an invalid file`);
    if (guide.route !== `/docs/history/${guide.id}`) failures.push(`${manifestPath}: ${guide.id} has an invalid route`);
    if (!existsSync(join(resolvedRoot, guide.file))) {
      failures.push(`${guide.file}: required history guide is missing`);
      continue;
    }
    const body = read(resolvedRoot, guide.file);
    const meta = metadata(body);
    if (!meta) failures.push(`${guide.file}: publication frontmatter is missing`);
    else {
      if (meta.authority !== guide.route) failures.push(`${guide.file}: current_authority must be ${guide.route}`);
      for (const record of meta.records) {
        if (!existsSync(join(resolvedRoot, record))) failures.push(`${guide.file}: source record does not exist: ${record}`);
        if (record.includes('.prometheus/knowledge/wiki/')) failures.push(`${guide.file}: unreviewed wiki source is forbidden: ${record}`);
        if ((record.startsWith('.prometheus/') || record.startsWith('.kbd-orchestrator/')) && existsSync(join(resolvedRoot, record)) && meta.content === read(resolvedRoot, record).trim()) {
          failures.push(`${guide.file}: exact private history copy is forbidden: ${record}`);
        }
      }
    }
    for (const marker of guide.requiredMarkers ?? []) {
      if (!body.toLocaleLowerCase('en').includes(marker.toLocaleLowerCase('en'))) failures.push(`${guide.file}: required marker is missing: ${marker}`);
    }
  }

  if (!Array.isArray(manifest.adrs) || manifest.adrs.length !== 18) failures.push(`${manifestPath}: expected exactly 18 retained ADRs`);
  const adrIds = new Set();
  for (const adr of manifest.adrs ?? []) {
    if (!adr.id || adrIds.has(adr.id)) failures.push(`${manifestPath}: ADR id is missing or duplicated: ${adr.id ?? '<missing>'}`);
    adrIds.add(adr.id);
    if (!existsSync(join(resolvedRoot, adr.file ?? ''))) failures.push(`${manifestPath}: ADR source does not exist: ${adr.file ?? '<missing>'}`);
    if (!/^\d{4}-\d{2}-\d{2}$/u.test(adr.date ?? '')) failures.push(`${manifestPath}: ${adr.id} has invalid date`);
    if (!allowedStatuses.has(adr.status)) failures.push(`${manifestPath}: ${adr.id} has invalid status`);
    if (adr.status === 'superseded' && (!adr.supersededBy || !manifest.adrs.some((candidate) => candidate.id === adr.supersededBy))) failures.push(`${manifestPath}: ${adr.id} lacks a valid supersededBy`);
  }

  const decisionIds = (manifest.decisions ?? []).map((decision) => decision.id);
  if (JSON.stringify(decisionIds) !== JSON.stringify(expectedDecisions)) failures.push(`${manifestPath}: required correction decisions are missing or out of order`);
  for (const decision of manifest.decisions ?? []) {
    if (decision.status !== 'current') failures.push(`${manifestPath}: ${decision.id} must identify the current replacement`);
    if (!decision.supersedes) failures.push(`${manifestPath}: ${decision.id} has no superseded position`);
    if (!decision.currentAuthority || !existsSync(join(resolvedRoot, decision.currentAuthority))) failures.push(`${manifestPath}: ${decision.id} current authority does not exist`);
    if (!Array.isArray(decision.sources) || decision.sources.length === 0) failures.push(`${manifestPath}: ${decision.id} has no source records`);
    for (const source of decision.sources ?? []) {
      if (!existsSync(join(resolvedRoot, source))) failures.push(`${manifestPath}: ${decision.id} source does not exist: ${source}`);
      if (source.includes('.prometheus/knowledge/wiki/')) failures.push(`${manifestPath}: ${decision.id} uses an unreviewed wiki source`);
    }
  }

  return {failures, guideCount: manifest.guides?.length ?? 0, decisionCount: manifest.decisions?.length ?? 0};
}

function main() {
  const result = validateDocumentationHistory({root: process.argv[2] ?? defaultRoot});
  if (result.failures.length) {
    console.error(`Documentation history validation failed:\n- ${result.failures.join('\n- ')}`);
    process.exit(1);
  }
  console.log(`Documentation history validation passed (${result.guideCount} guides, 18 ADRs, ${result.decisionCount} correction decisions).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
