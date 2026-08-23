#!/usr/bin/env node

import {cpSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {validateDocumentationSecurityOperations} from './validate-documentation-security-operations.mjs';

const scriptPath = fileURLToPath(import.meta.url);
const sourceRoot = resolve(dirname(scriptPath), '..');
const manifestPath = 'docs/publication/security-operations.json';

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
  const root = mkdtempSync(join(tmpdir(), 'uar-doc-security-operations-'));
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
    const result = validateDocumentationSecurityOperations({root});
    if (!result.failures.some((failure) => failure.includes(expected))) {
      throw new Error(`${name}: expected ${JSON.stringify(expected)}, observed ${JSON.stringify(result.failures)}`);
    }
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
}

expectFailure('missing guide', (root) => {
  rmSync(join(root, 'website/docs/security/authentication.md'));
}, 'required security/operations guide is missing');

expectFailure('unclassified authority record', (root) => {
  const manifest = readJson(root);
  const original = manifest.guides[0].sourceRecords[0];
  const replacement = 'unclassified/security-record.md';
  mkdirSync(join(root, dirname(replacement)), {recursive: true});
  cpSync(join(root, original), join(root, replacement));
  manifest.guides[0].sourceRecords[0] = replacement;
  writeJson(root, manifest);
}, 'source record is unclassified');

expectFailure('unsafe credential example', (root) => {
  replace(root, 'website/docs/security/credentials.md', '## Profile limits', 'api_key = "sensitive-example-value"\n\n## Profile limits');
}, 'credential-shaped assignment');

expectFailure('unverified tenant identity claim', (root) => {
  replace(root, 'website/docs/tenancy/overview.md', '## Profile limits', 'Trusted tenant identity from request metadata.\n\n## Profile limits');
}, 'unverified tenant identity claim');

expectFailure('blanket tenant isolation claim', (root) => {
  replace(root, 'website/docs/tenancy/overview.md', '## Profile limits', 'Tenant isolation applies to every UAR subsystem.\n\n## Profile limits');
}, 'blanket tenant-isolation claim');

expectFailure('universal fail-closed governance claim', (root) => {
  replace(root, 'website/docs/governance/overview.md', '## Profile limits', 'Cedar enforcement is fail closed in every runtime profile.\n\n## Profile limits');
}, 'universal fail-closed governance claim');

expectFailure('approval override claim', (root) => {
  replace(root, 'website/docs/governance/approvals.md', '## Profile limits', 'Human approval overrides Cedar denial.\n\n## Profile limits');
}, 'approval-overrides-denial claim');

expectFailure('missing approval timeout', (root) => {
  replace(root, 'website/docs/governance/approvals.md', 'five minutes', 'operator-defined duration');
}, 'required marker is missing: five minutes');

expectFailure('durable realtime claim', (root) => {
  replace(root, 'website/docs/operations/realtime.md', '## Profile limits', 'Reconnect guarantees durable replay.\n\n## Profile limits');
}, 'durable-realtime claim');

expectFailure('authoritative billing claim', (root) => {
  replace(root, 'website/docs/operations/cost.md', '## Profile limits', 'The cost dashboard is the authoritative invoice.\n\n## Profile limits');
}, 'authoritative-billing claim');

expectFailure('missing shutdown deadline', (root) => {
  replaceEvery(root, 'website/docs/operations/recovery-and-shutdown.md', 'shutdown deadline', 'shutdown window');
}, 'required marker is missing: shutdown deadline');

expectFailure('missing restore read-back', (root) => {
  replaceEvery(root, 'website/docs/operations/recovery-and-shutdown.md', 'functional read-back', 'archive inspection');
  replaceEvery(root, 'website/docs/operations/recovery-and-shutdown.md', 'Functional read-back', 'Archive inspection');
}, 'required marker is missing: functional read-back');

expectFailure('missing profile and state owner', (root) => {
  replace(root, 'website/docs/operations/observability.md', 'State ownership and durability', 'Signal lifecycle');
}, 'required marker is missing: State ownership and durability');

expectFailure('unsafe private excerpt', (root) => {
  replace(root, 'website/docs/operations/runtime-console.md', '## Profile limits', 'Machine source: /Users/private-user/worktree\n\n## Profile limits');
}, 'machine-local macOS path');

const current = validateDocumentationSecurityOperations({root: sourceRoot});
if (current.failures.length) throw new Error(`current security/operations source failed: ${JSON.stringify(current.failures)}`);
console.log('PASS positive control: complete security/operations source');
