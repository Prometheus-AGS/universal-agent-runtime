// Compare synthetic data from two offline UAR stores through their exact SDK
// runners. Run only under the approved embedded-upgrade acceptance batch.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { readFile, realpath, stat } from 'node:fs/promises';
import { relative, resolve, sep } from 'node:path';

const groups = {
  persistence: {
    agui: ['agui_runs', 'agui_contexts'],
    history: ['durable_run_records', 'durable_run_events'],
    checkpoint: ['checkpoints', 'checkpoint_resume_envelopes', 'checkpoint_runtime_epochs',
      'checkpoint_resume_claims', 'checkpoint_resume_claim_history', 'tool_admission_evidence'],
    receipts: ['canonical_tool_receipt_runs', 'canonical_tool_receipts'],
    knowledge: ['principal_knowledge_bases', 'principal_knowledge_documents',
      'principal_knowledge_chunks', 'principal_knowledge_ingestion'],
  },
  memory: {
    recall: ['memory', 'memory_history'],
  },
};

const digest = async (file) => {
  const hash = createHash('sha256');
  for await (const part of createReadStream(file)) hash.update(part);
  return hash.digest('hex');
};

const args = process.argv.slice(2);
assert.equal(args.length, 1, 'Pass the exact synthetic parity manifest path');
const manifest = JSON.parse(await readFile(args[0], 'utf8'));
assert.equal(manifest.kind, 'uar.embedded-surrealdb-parity/v1');
assert.match(manifest.caseId, /^[a-z0-9][a-z0-9-]{7,63}$/);
const scratch = await realpath(resolve('target/uar-surrealdb4-fixtures', manifest.caseId));
assert.ok(Array.isArray(manifest.stores) && manifest.stores.length > 0);
const observedKinds = new Set();

const receipts = [];
for (const store of manifest.stores) {
  assert.ok(Object.hasOwn(groups, store.kind), 'Unsupported fixture store class');
  assert.ok(!observedKinds.has(store.kind), 'Duplicate store class');
  observedKinds.add(store.kind);
  assert.match(store.namespace, /^[a-zA-Z0-9_-]+$/);
  assert.match(store.database, /^[a-zA-Z0-9_-]+$/);

  const paths = await Promise.all([store.baseline, store.candidate].map(async (entry) => {
    assert.match(entry.runnerSha256, /^[0-9a-f]{64}$/);
    const runner = await realpath(entry.runner);
    assert.equal(await digest(runner), entry.runnerSha256);
    const path = await realpath(entry.store);
    const fromRoot = relative(scratch, path);
    assert.ok(fromRoot && fromRoot !== '..' && !fromRoot.startsWith('..' + sep));
    const metadata = await stat(path);
    assert.ok(metadata.isDirectory() && !path.includes('?'));
    return { path, runner, runnerSha256: entry.runnerSha256, inode: [metadata.dev, metadata.ino] };
  }));
  assert.notEqual(paths[0].runnerSha256, paths[1].runnerSha256, 'Require both SDK generations');
  const [baseline, candidate] = paths;
  assert.notDeepEqual(baseline.inode, candidate.inode, 'Store copies must have distinct identities');
  for (const [left, right] of [[baseline.path, candidate.path], [candidate.path, baseline.path]]) {
    const child = relative(left, right);
    assert.ok(child === '..' || child.startsWith('..' + sep), 'Store copies must be disjoint');
  }

  const cases = Object.entries(groups[store.kind]).flatMap(([group, tables]) =>
    tables.map(table => ({ group, table })));
  const sql = cases.map(({ table }) => `SELECT * FROM ${table} ORDER BY id;`).join('\n');
  const [before, after] = paths.map(({ path, runner }) => {
    const output = execFileSync(runner, [`surrealkv://${path}`, store.namespace, store.database], {
      input: sql, encoding: 'utf8', timeout: 120_000, maxBuffer: 8 * 1024 * 1024,
    });
    return output.trim().split('\n').map(line => JSON.parse(line));
  });
  assert.equal(before.length, cases.length);
  assert.equal(after.length, cases.length);
  const observed = cases.map(({ group, table }, i) => {
    assert.ok(Array.isArray(before[i]) && before[i].length > 0, `Seed ${table} before migration`);
    assert.deepEqual(after[i], before[i], `Logical rows changed for ${table}`);
    return { group, table, rows: before[i].length };
  });
  receipts.push({ kind: store.kind, namespace: store.namespace, database: store.database, paths, observed });
}
assert.ok(observedKinds.has('persistence'), 'The UAR persistence store is mandatory');
assert.equal(observedKinds.has('memory'), manifest.memoryEnabled === true);
console.log(JSON.stringify({ caseId: manifest.caseId, comparisons: receipts }));
