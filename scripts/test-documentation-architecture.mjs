#!/usr/bin/env node

import {cpSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {validateDocumentationArchitecture} from './validate-documentation-architecture.mjs';

const scriptPath = fileURLToPath(import.meta.url);
const sourceRoot = resolve(dirname(scriptPath), '..');

function copy(root, path) {
  mkdirSync(join(root, dirname(path)), {recursive: true});
  cpSync(join(sourceRoot, path), join(root, path), {recursive: true});
}

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'uar-doc-architecture-'));
  copy(root, 'docs/publication/architecture.json');
  const manifest = JSON.parse(readFileSync(join(root, 'docs/publication/architecture.json'), 'utf8'));
  const paths = new Set();
  for (const guide of manifest.guides) {
    paths.add(guide.file);
    for (const source of [...guide.sourceRecords, ...guide.sourceAuthorities]) paths.add(source);
  }
  for (const path of paths) copy(root, path);
  return root;
}

function readJson(root) {
  return JSON.parse(readFileSync(join(root, 'docs/publication/architecture.json'), 'utf8'));
}

function writeJson(root, value) {
  writeFileSync(join(root, 'docs/publication/architecture.json'), `${JSON.stringify(value, null, 2)}\n`);
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
    const result = validateDocumentationArchitecture({root});
    if (!result.failures.some((failure) => failure.includes(expected))) {
      throw new Error(`${name}: expected ${JSON.stringify(expected)}, observed ${JSON.stringify(result.failures)}`);
    }
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
}

expectFailure('missing architecture page', (root) => {
  rmSync(join(root, 'website/docs/architecture/intro.md'));
}, 'required architecture guide is missing');

expectFailure('missing source authority', (root) => {
  const manifest = readJson(root);
  manifest.guides[0].sourceAuthorities[0] = 'src/missing-architecture-authority.rs';
  writeJson(root, manifest);
}, 'source does not exist');

expectFailure('invalid profile', (root) => {
  const manifest = readJson(root);
  manifest.guides[0].profiles.push('all-profiles');
  writeJson(root, manifest);
}, 'missing or invalid profiles');

expectFailure('missing provenance record', (root) => {
  replace(root, 'website/docs/architecture/intro.md', '  - openspec/specs/customer-documentation/spec.md\n', '');
}, 'source_records is missing');

expectFailure('missing profile limit', (root) => {
  replace(root, 'website/docs/architecture/profiles.md', '## Profile limits', '## Capability notes');
}, 'required marker is missing: Profile limits');

expectFailure('missing trust boundary', (root) => {
  replace(root, 'website/docs/architecture/trust-boundary.md', 'Intent is not an effect', 'A request begins here');
}, 'required marker is missing: intent is not an effect');

expectFailure('missing diagram explanation', (root) => {
  replace(root, 'website/docs/architecture/intro.md', '## Diagram in words', '## Visual summary');
}, 'diagram explanation is missing');

const current = validateDocumentationArchitecture({root: sourceRoot});
if (current.failures.length) throw new Error(`current architecture source failed: ${JSON.stringify(current.failures)}`);
console.log('PASS positive control: complete architecture source');
