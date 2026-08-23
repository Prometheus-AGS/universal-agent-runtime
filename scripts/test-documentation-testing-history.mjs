#!/usr/bin/env node

import {cpSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {validateDocumentationTestingHistory} from './validate-documentation-testing-history.mjs';

const scriptPath = fileURLToPath(import.meta.url);
const sourceRoot = resolve(dirname(scriptPath), '..');

function copy(root, path) {
  mkdirSync(join(root, dirname(path)), {recursive: true});
  cpSync(join(sourceRoot, path), join(root, path), {recursive: true});
}

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'uar-doc-testing-history-'));
  copy(root, 'docs/publication/testing-history.json');
  const manifest = readManifest(root);
  const paths = new Set(manifest.sources);
  for (const guide of manifest.guides) paths.add(guide.file);
  for (const path of paths) copy(root, path);
  return root;
}

function readManifest(root) {
  return JSON.parse(readFileSync(join(root, 'docs/publication/testing-history.json'), 'utf8'));
}

function writeManifest(root, manifest) {
  writeFileSync(join(root, 'docs/publication/testing-history.json'), `${JSON.stringify(manifest, null, 2)}\n`);
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
    const result = validateDocumentationTestingHistory({root});
    if (!result.failures.some((failure) => failure.includes(expected))) throw new Error(`${name}: expected ${JSON.stringify(expected)}, observed ${JSON.stringify(result.failures)}`);
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
}

expectFailure('missing evidence limit', (root) => {
  const manifest = readManifest(root);
  delete manifest.evidenceClasses[0].doesNotProve;
  writeManifest(root, manifest);
}, 'is missing doesNotProve');

expectFailure('synthetic inference certification', (root) => {
  const manifest = readManifest(root);
  manifest.evidenceClasses[2].proves = 'Genuine model inference through UAR.';
  writeManifest(root, manifest);
}, 'must not certify genuine model inference');

expectFailure('missing negative-control failure', (root) => {
  replace(root, 'website/docs/history/negative-controls.md', '**observed to fail**', '**checked once**');
}, 'required marker is missing: observed to fail');

expectFailure('profile transfer', (root) => {
  const manifest = readManifest(root);
  manifest.profiles.push('all-profiles');
  writeManifest(root, manifest);
}, 'without transfer');

expectFailure('duration-only soak', (root) => {
  const manifest = readManifest(root);
  manifest.evidenceClasses[5].proves = 'The workload ran for three hours.';
  writeManifest(root, manifest);
}, 'requires an objective beyond elapsed duration');

expectFailure('routine Actions testing', (root) => {
  const path = 'website/docs/history/local-verification.md';
  writeFileSync(join(root, path), `${readFileSync(join(root, path), 'utf8')}\nGitHub Actions run unit tests.\n`);
}, 'routine GitHub Actions testing is forbidden');

expectFailure('copied private testing history', (root) => {
  const path = 'website/docs/history/testing-methodology.md';
  const body = readFileSync(join(root, path), 'utf8');
  const end = body.indexOf('\n---\n', 4);
  const privateBody = readFileSync(join(root, '.prometheus/decisions.md'), 'utf8').trim();
  writeFileSync(join(root, path), `${body.slice(0, end + 5)}${privateBody}\n`);
}, 'exact private history copy is forbidden');

const current = validateDocumentationTestingHistory({root: sourceRoot});
if (current.failures.length) throw new Error(`current testing-history source failed: ${JSON.stringify(current.failures)}`);
console.log('PASS positive control: complete testing methodology source');
