#!/usr/bin/env node

import {cpSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {validateDocumentationDeveloperReference} from './validate-documentation-developer-reference.mjs';

const scriptPath = fileURLToPath(import.meta.url);
const sourceRoot = resolve(dirname(scriptPath), '..');
const manifestPath = 'docs/publication/developer-reference.json';

function copy(root, path) {
  mkdirSync(join(root, dirname(path)), {recursive: true});
  cpSync(join(sourceRoot, path), join(root, path), {recursive: true});
}

function readJson(root) {
  return JSON.parse(readFileSync(join(root, manifestPath), 'utf8'));
}

function writeJson(root, value) {
  writeFileSync(join(root, manifestPath), `${JSON.stringify(value, null, 2)}\n`);
}

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'uar-doc-developer-reference-'));
  copy(root, manifestPath);
  copy(root, 'docs/publication/sources.json');
  copy(root, 'website/docs');
  const manifest = readJson(root);
  const paths = new Set(manifest.compatibilityDocuments.map((document) => document.file));
  for (const guide of manifest.guides) {
    paths.add(guide.file);
    for (const source of [...guide.sourceRecords, ...guide.sourceAuthorities]) paths.add(source);
  }
  for (const path of paths) copy(root, path);
  return root;
}

function replace(root, path, from, to) {
  const absolute = join(root, path);
  const body = readFileSync(absolute, 'utf8');
  if (!body.includes(from)) throw new Error(`fixture mutation source missing in ${path}: ${from}`);
  writeFileSync(absolute, body.replace(from, to));
}

function replaceEvery(root, path, from, to) {
  const absolute = join(root, path);
  const body = readFileSync(absolute, 'utf8');
  if (!body.includes(from)) throw new Error(`fixture mutation source missing in ${path}: ${from}`);
  writeFileSync(absolute, body.replaceAll(from, to));
}

function expectFailure(name, mutate, expected) {
  const root = fixture();
  try {
    mutate(root);
    const result = validateDocumentationDeveloperReference({root});
    if (!result.failures.some((failure) => failure.includes(expected))) {
      throw new Error(`${name}: expected ${JSON.stringify(expected)}, observed ${JSON.stringify(result.failures)}`);
    }
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
}

expectFailure('missing guide', (root) => {
  rmSync(join(root, 'website/docs/api/index.md'));
}, 'required developer-reference guide is missing');

expectFailure('unclassified authority record', (root) => {
  const manifest = readJson(root);
  const original = manifest.guides[0].sourceRecords[0];
  const replacement = 'unclassified/api-record.md';
  mkdirSync(join(root, dirname(replacement)), {recursive: true});
  cpSync(join(root, original), join(root, replacement));
  manifest.guides[0].sourceRecords[0] = replacement;
  writeJson(root, manifest);
}, 'source record is unclassified');

expectFailure('invented endpoint', (root) => {
  replace(root, 'website/docs/protocols/http-compatibility.md', '## Profile limits', 'Invented endpoint: /v9/imaginary.\n\n## Profile limits');
}, 'invented endpoint claim');

expectFailure('exhaustive OpenAPI claim', (root) => {
  replace(root, 'website/docs/api/index.md', '## Profile limits', 'OpenAPI is the complete API route inventory.\n\n## Profile limits');
}, 'exhaustive OpenAPI claim');

expectFailure('complete protocol parity claim', (root) => {
  replace(root, 'website/docs/protocols/http-compatibility.md', '## Profile limits', 'Complete OpenAI compatibility is provided.\n\n## Profile limits');
}, 'complete protocol-parity claim');

expectFailure('discovery as authorization', (root) => {
  replace(root, 'website/docs/tools/overview.md', '## Profile limits', 'Discovery automatically grants authorization.\n\n## Profile limits');
}, 'discovery-as-authorization claim');

expectFailure('production JWT proxy', (root) => {
  replace(root, 'website/docs/tools/overview.md', '## Profile limits', 'uar-jwt-proxy is a production authentication gateway.\n\n## Profile limits');
}, 'production JWT-proxy claim');

expectFailure('hosted Python reference', (root) => {
  replace(root, 'website/docs/sdk-python/intro.md', '## Profile limits', 'Hosted generated Python API reference is available.\n\n## Profile limits');
}, 'hosted Python reference claim');

expectFailure('registry publication from metadata', (root) => {
  replace(root, 'website/docs/sdks.md', '## Profile limits', 'Package metadata proves registry publication.\n\n## Profile limits');
}, 'registry-from-metadata claim');

expectFailure('unsafe anonymous listener', (root) => {
  replace(root, 'website/docs/configuration.md', '## Profile limits', 'jwt_required: false\nhost: 0.0.0.0\n\n## Profile limits');
}, 'unsafe anonymous listener example');

expectFailure('cross-profile evidence transfer', (root) => {
  replace(root, 'website/docs/installation.md', '## Profile limits', 'server-full evidence transfers to embedded-mobile.\n\n## Profile limits');
}, 'cross-profile evidence claim');

expectFailure('missing deployment health boundary', (root) => {
  replaceEvery(root, 'website/docs/deployment.md', '/readyz', '/status');
}, 'required marker is missing: /readyz');

expectFailure('missing rollback boundary', (root) => {
  replaceEvery(root, 'website/docs/upgrade-guide.md', 'Rollback', 'Recovery option');
  replaceEvery(root, 'website/docs/upgrade-guide.md', 'rollback', 'recovery option');
}, 'required marker is missing: Rollback');

expectFailure('routine tests in GitHub Actions', (root) => {
  replace(root, 'website/docs/deployment.md', '## Profile limits', 'GitHub Actions runs unit tests.\n\n## Profile limits');
}, 'routine GitHub Actions test claim');

expectFailure('unsafe private excerpt', (root) => {
  replace(root, 'website/docs/api/index.md', '## Profile limits', 'Machine source: /Users/private-user/worktree\n\n## Profile limits');
}, 'machine-local macOS path');

const current = validateDocumentationDeveloperReference({root: sourceRoot});
if (current.failures.length) throw new Error(`current developer-reference source failed: ${JSON.stringify(current.failures)}`);
console.log('PASS positive control: complete developer-reference source');
