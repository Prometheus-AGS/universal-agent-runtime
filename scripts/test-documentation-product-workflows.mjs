#!/usr/bin/env node

import {cpSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {validateDocumentationProductWorkflows} from './validate-documentation-product-workflows.mjs';

const scriptPath = fileURLToPath(import.meta.url);
const sourceRoot = resolve(dirname(scriptPath), '..');

function copy(root, path) {
  mkdirSync(join(root, dirname(path)), {recursive: true});
  cpSync(join(sourceRoot, path), join(root, path), {recursive: true});
}

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'uar-doc-product-workflows-'));
  copy(root, 'docs/publication/product-workflows.json');
  copy(root, 'docs/publication/sources.json');
  const manifest = readJson(root);
  const paths = new Set(manifest.compatibilityDocuments.map((document) => document.file));
  for (const guide of manifest.guides) {
    paths.add(guide.file);
    for (const source of [...guide.sourceRecords, ...guide.sourceAuthorities]) paths.add(source);
  }
  for (const path of paths) copy(root, path);
  return root;
}

function readJson(root) {
  return JSON.parse(readFileSync(join(root, 'docs/publication/product-workflows.json'), 'utf8'));
}

function writeJson(root, value) {
  writeFileSync(join(root, 'docs/publication/product-workflows.json'), `${JSON.stringify(value, null, 2)}\n`);
}

function replace(root, path, from, to) {
  const absolute = join(root, path);
  const body = readFileSync(absolute, 'utf8');
  if (!body.includes(from)) throw new Error(`fixture mutation source missing in ${path}: ${from}`);
  writeFileSync(absolute, body.replace(from, to));
}

function expectFailure(name, mutate, expected) {
  const root = fixture();
  try {
    mutate(root);
    const result = validateDocumentationProductWorkflows({root});
    if (!result.failures.some((failure) => failure.includes(expected))) {
      throw new Error(`${name}: expected ${JSON.stringify(expected)}, observed ${JSON.stringify(result.failures)}`);
    }
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
}

expectFailure('missing guide', (root) => {
  rmSync(join(root, 'website/docs/providers/configuration.md'));
}, 'required product workflow guide is missing');

expectFailure('unclassified authority record', (root) => {
  const manifest = readJson(root);
  const original = manifest.guides[0].sourceRecords[0];
  const replacement = 'unclassified/provider-record.md';
  mkdirSync(join(root, dirname(replacement)), {recursive: true});
  cpSync(join(root, original), join(root, replacement));
  manifest.guides[0].sourceRecords[0] = replacement;
  writeJson(root, manifest);
}, 'source record is unclassified');

expectFailure('missing packaged surface', (root) => {
  replace(root, 'website/docs/providers/configuration.md', '## Packaged UI workflow', '## Browser workflow');
}, 'required marker is missing: Packaged UI workflow');

expectFailure('missing profile and state limit', (root) => {
  replace(root, 'website/docs/providers/models.md', '## Realtime state and reload authority', '## State behavior');
}, 'required marker is missing: Realtime state and reload authority');

expectFailure('synthetic response claimed as genuine', (root) => {
  replace(root, 'website/docs/providers/inference.md', 'Illustrative and non-certifying', 'Synthetic response verified genuine inference');
}, 'synthetic evidence is represented as genuine inference evidence');

expectFailure('missing skill safety', (root) => {
  replace(root, 'website/docs/skills/overview.md', 'never hard-deletes', 'removes records');
}, 'required marker is missing: never hard-deletes');

expectFailure('knowledge and memory conflated', (root) => {
  replace(root, 'website/docs/knowledge/overview.md', 'Knowledge is not memory', 'Knowledge and memory are equivalent');
}, 'required marker is missing: Knowledge is not memory');

expectFailure('missing diagram explanation', (root) => {
  replace(root, 'website/docs/agents/overview.md', '## Diagram in words', '## Visual explanation');
}, 'diagram explanation is missing');

expectFailure('unsafe private excerpt', (root) => {
  replace(root, 'website/docs/providers/inference.md', '## Evidence limits', 'Machine source: /Users/private-user/worktree\n\n## Evidence limits');
}, 'publication sanitizer rejected machine-local macOS path');

const current = validateDocumentationProductWorkflows({root: sourceRoot});
if (current.failures.length) throw new Error(`current product-workflow source failed: ${JSON.stringify(current.failures)}`);
console.log('PASS positive control: complete product-workflow source');
