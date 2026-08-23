#!/usr/bin/env node

import {cpSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {validateDocumentationHistory} from './validate-documentation-history.mjs';

const scriptPath = fileURLToPath(import.meta.url);
const sourceRoot = resolve(dirname(scriptPath), '..');

function copy(root, path) {
  mkdirSync(join(root, dirname(path)), {recursive: true});
  cpSync(join(sourceRoot, path), join(root, path), {recursive: true});
}

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'uar-doc-history-'));
  copy(root, 'docs/publication/architecture-history.json');
  const manifest = JSON.parse(readFileSync(join(root, 'docs/publication/architecture-history.json'), 'utf8'));
  const paths = new Set(['AGENTS.md']);
  for (const guide of manifest.guides) paths.add(guide.file);
  for (const adr of manifest.adrs) paths.add(adr.file);
  for (const decision of manifest.decisions) {
    paths.add(decision.currentAuthority);
    for (const source of decision.sources) paths.add(source);
  }
  for (const guide of manifest.guides) {
    const body = readFileSync(join(sourceRoot, guide.file), 'utf8');
    for (const match of body.matchAll(/^\s+-\s+(.+)$/gmu)) {
      const path = match[1].trim();
      if (path.includes('/') && !path.startsWith('http') && !path.startsWith('[') && existsInSource(path)) paths.add(path);
    }
  }
  for (const path of paths) copy(root, path);
  return root;
}

function existsInSource(path) {
  try {
    readFileSync(join(sourceRoot, path));
    return true;
  } catch {
    return false;
  }
}

function readManifest(root) {
  return JSON.parse(readFileSync(join(root, 'docs/publication/architecture-history.json'), 'utf8'));
}

function writeManifest(root, manifest) {
  writeFileSync(join(root, 'docs/publication/architecture-history.json'), `${JSON.stringify(manifest, null, 2)}\n`);
}

function replace(root, path, from, to) {
  const absolute = join(root, path);
  const body = readFileSync(absolute, 'utf8');
  if (!body.includes(from)) throw new Error(`${path}: fixture mutation source missing: ${from}`);
  writeFileSync(absolute, body.replace(from, to));
}

function expectFailure(name, mutate, expected) {
  const root = fixture();
  try {
    mutate(root);
    const result = validateDocumentationHistory({root});
    if (!result.failures.some((failure) => failure.includes(expected))) throw new Error(`${name}: expected ${JSON.stringify(expected)}, observed ${JSON.stringify(result.failures)}`);
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
}

expectFailure('missing correction decision', (root) => {
  const manifest = readManifest(root);
  manifest.decisions.pop();
  writeManifest(root, manifest);
}, 'required correction decisions are missing');

expectFailure('missing decision source', (root) => {
  const manifest = readManifest(root);
  manifest.decisions[0].sources[0] = 'docs/adr/missing.md';
  writeManifest(root, manifest);
}, 'source does not exist');

expectFailure('missing supersession', (root) => {
  const manifest = readManifest(root);
  delete manifest.decisions[0].supersedes;
  writeManifest(root, manifest);
}, 'has no superseded position');

expectFailure('missing current authority', (root) => {
  const manifest = readManifest(root);
  manifest.decisions[0].currentAuthority = 'docs/missing-authority.md';
  writeManifest(root, manifest);
}, 'current authority does not exist');

expectFailure('missing correction coverage', (root) => {
  replace(root, 'website/docs/history/corrections.md', '**synthetic soak**', '**duration check**');
}, 'required marker is missing: synthetic soak');

expectFailure('direct wiki authority', (root) => {
  const manifest = readManifest(root);
  const wiki = '.prometheus/knowledge/wiki/unreviewed.md';
  mkdirSync(join(root, dirname(wiki)), {recursive: true});
  writeFileSync(join(root, wiki), '# Unreviewed\n');
  manifest.decisions[0].sources[0] = wiki;
  writeManifest(root, manifest);
}, 'unreviewed wiki source');

expectFailure('copied private history', (root) => {
  const path = 'website/docs/history/overview.md';
  const body = readFileSync(join(root, path), 'utf8');
  const end = body.indexOf('\n---\n', 4);
  const privateBody = readFileSync(join(root, '.prometheus/decisions.md'), 'utf8').trim();
  writeFileSync(join(root, path), `${body.slice(0, end + 5)}${privateBody}\n`);
}, 'exact private history copy is forbidden');

const current = validateDocumentationHistory({root: sourceRoot});
if (current.failures.length) throw new Error(`current history source failed: ${JSON.stringify(current.failures)}`);
console.log('PASS positive control: complete architecture history source');
