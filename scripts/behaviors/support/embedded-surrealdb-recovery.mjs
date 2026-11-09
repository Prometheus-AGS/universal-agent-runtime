// Recovery qualification for owned, synthetic offline SurrealKV stores only.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { lstat, mkdtemp, readdir, readFile, realpath, writeFile } from 'node:fs/promises';
import { join, relative, resolve, sep } from 'node:path';

const sha256 = (value) => createHash('sha256').update(value).digest('hex');
async function fileDigest(path) {
  const hash = createHash('sha256');
  for await (const bytes of createReadStream(path)) hash.update(bytes);
  return hash.digest('hex');
}
async function inventory(root, dir = root) {
  const result = [];
  for (const name of (await readdir(dir)).sort()) {
    const path = join(dir, name);
    const metadata = await lstat(path);
    if (metadata.isDirectory()) {
      result.push({ path: relative(root, path), directory: true });
      result.push(...await inventory(root, path));
    } else {
      assert.ok(metadata.isFile() && metadata.nlink === 1, 'Only regular fixture files are supported');
      result.push({ path: relative(root, path), size: metadata.size, sha256: await fileDigest(path) });
    }
  }
  return result;
}
function run(program, args, input) {
  const result = spawnSync(program, args, {
    encoding: 'utf8', timeout: 180_000, maxBuffer: 8 * 1024 * 1024, input,
  });
  if (result.error) throw result.error;
  return result;
}
function success(result) {
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim().split('\n').map(line => JSON.parse(line));
}

const args = process.argv.slice(2);
assert.equal(args.length, 1, 'Pass the exact synthetic parity manifest');
const manifest = JSON.parse(await readFile(args[0], 'utf8'));
assert.equal(manifest.kind, 'uar.embedded-surrealdb-parity/v1');
assert.match(manifest.caseId, /^[a-z0-9][a-z0-9-]{7,63}$/);
const root = await realpath(resolve('target/uar-surrealdb4-fixtures', manifest.caseId));
const store = manifest.stores.find(entry => entry.kind === 'persistence');
assert.ok(store, 'Persistence fixture required');
assert.match(store.namespace, /^[a-zA-Z0-9_-]+$/);
assert.match(store.database, /^[a-zA-Z0-9_-]+$/);
const [oldRunner, newRunner] = await Promise.all([store.baseline, store.candidate].map(async (entry) => {
  const path = await realpath(entry.runner);
  assert.match(entry.runnerSha256, /^[a-f0-9]{64}$/);
  assert.equal(await fileDigest(path), entry.runnerSha256);
  return path;
}));
assert.notEqual(store.baseline.runnerSha256, store.candidate.runnerSha256);
const snapshot = await realpath(store.baseline.immutableSnapshot);
const location = relative(root, snapshot);
assert.ok(location && location !== '..' && !location.startsWith('..' + sep));
const before = await inventory(snapshot);
assert.ok(before.some(entry => entry.size > 16_384), 'The synthetic store needs a file larger than the interruption limit');
const workspace = await mkdtemp(join(root, 'recovery-'));
const query = (runner, dir, sql) =>
  success(run(runner, [`surrealkv://${dir}`, store.namespace, store.database], sql));

const planPath = join(workspace, 'rollback.json');
const plan = {
  kind: 'uar.embedded-surrealdb-rollback/v1',
  snapshot,
  recovery_point: sha256(JSON.stringify(before)),
  writes_frozen: true,
  candidate_write_receipt_sha256: sha256('synthetic fixture has no accepted post-baseline writes'),
  accepted_writes: [],
  dispositions: {},
};
const savePlan = () => writeFile(planPath, JSON.stringify(plan), { mode: 0o600 });
const restore = destination => run(newRunner, ['--restore-3.3', snapshot, destination, planPath]);
async function rejectRestoreSource(source, name, expected) {
  const invalidPlan = join(workspace, `${name}.json`);
  const destination = join(workspace, name);
  await writeFile(invalidPlan, JSON.stringify({ ...plan, snapshot: source }), { mode: 0o600 });
  const rejected = run(newRunner, ['--restore-3.3', source, destination, invalidPlan]);
  assert.equal(rejected.status, 1);
  assert.match(rejected.stderr, expected);
  await assert.rejects(lstat(destination), { code: 'ENOENT' });
  return { source, destination, status: rejected.status, reason: rejected.stderr.trim() };
}
await savePlan();
const reference = join(workspace, 'reference-3.3');
assert.equal(success(restore(reference))[0].event, 'restored-copy');
const baselineRows = query(oldRunner, reference, 'SELECT * FROM agui_runs ORDER BY id;')[0];
assert.ok(baselineRows.length > 0);

const refusalCopy = join(workspace, 'normal-start-refusal-3.3');
assert.equal(success(restore(refusalCopy))[0].event, 'restored-copy');
const refusalBefore = await inventory(refusalCopy);
const refusal = run(newRunner, [`surrealkv://${refusalCopy}`, store.namespace, store.database],
  'SELECT * FROM agui_runs ORDER BY id;');
assert.notEqual(refusal.status, 0, 'Normal 4.0 connect must refuse unconverted 3.3 data');
assert.match(refusal.stderr, /upgrad.{0,40}(experiment|refus)|upgrade_from_3x/i);
assert.deepEqual(await inventory(refusalCopy), refusalBefore);
assert.deepEqual(query(oldRunner, refusalCopy, 'SELECT * FROM agui_runs ORDER BY id;')[0], baselineRows);

const interrupted = join(workspace, 'quarantined-incomplete');
const interruption = run('bash', [
  '-c', 'ulimit -f 8; exec "$@"', 'uar-surrealdb4-copy-limit',
  newRunner, '--migrate-3.3', snapshot, interrupted,
]);
assert.notEqual(interruption.status, 0);
const progress = interruption.stdout.trim().split('\n').map(line => JSON.parse(line));
assert.equal(progress[0].stage, 'copy-start');
assert.ok(progress.every(entry => entry.stage !== 'copied' && entry.stage !== 'closed'));
assert.ok((await lstat(interrupted)).isDirectory());
assert.ok((await lstat(join(interrupted, 'UAR_UPGRADE_INCOMPLETE'))).isFile());
const incompleteRefusal = await rejectRestoreSource(interrupted, 'invalid-incomplete-restore', /^Error: interrupted conversion is ineligible:/);
const recovered = join(workspace, 'interrupted-restore-3.3');
assert.equal(success(restore(recovered))[0].event, 'restored-copy');
assert.deepEqual(query(oldRunner, recovered, 'SELECT * FROM agui_runs ORDER BY id;')[0], baselineRows);

const candidate = join(workspace, 'post-baseline-4.0');
assert.equal(success(run(newRunner, ['--migrate-3.3', snapshot, candidate])).at(-1).stage, 'closed');
await assert.rejects(lstat(join(candidate, 'UAR_UPGRADE_INCOMPLETE')), { code: 'ENOENT' });
const convertedRefusal = await rejectRestoreSource(candidate, 'invalid-converted-restore', /not a legacy SurrealKV format 1 store|missing legacy SurrealKV manifest|RocksDB or interrupted conversion is ineligible/);
const writeId = 'agui_contexts:uar_upgrade_post_baseline';
const write = query(newRunner, candidate,
  `CREATE ${writeId} SET run_key = 'accepted-after-baseline'; SELECT * FROM ${writeId};`);
assert.equal(write[1][0].run_key, 'accepted-after-baseline');
plan.candidate_write_receipt_sha256 = sha256(JSON.stringify(write));
plan.accepted_writes = [writeId];
await savePlan();
const restored = join(workspace, 'disposition-restore-3.3');
const refused = restore(restored);
assert.notEqual(refused.status, 0);
assert.match(refused.stderr, /requires a recorded disposition/);
await assert.rejects(lstat(restored), { code: 'ENOENT' });
assert.equal(query(newRunner, candidate, `SELECT * FROM ${writeId};`)[0][0].run_key, 'accepted-after-baseline');

plan.dispositions = {
  [writeId]: { action: 'accept-loss', record: 'synthetic-fixture-operator-disposition' },
};
await savePlan();
const receipt = success(restore(restored))[0];
assert.deepEqual(receipt.accepted_loss, [writeId]);
assert.deepEqual(receipt.excluded_from_restored_baseline, [writeId]);
assert.deepEqual(query(oldRunner, restored, 'SELECT * FROM agui_runs ORDER BY id;')[0], baselineRows);
assert.deepEqual(query(oldRunner, restored, `SELECT * FROM ${writeId};`)[0], []);
assert.deepEqual(await inventory(snapshot), before, 'The immutable original snapshot changed');
console.log(JSON.stringify({
  caseId: manifest.caseId, workspace, interruption: { status: interruption.status, signal: interruption.signal },
  quarantined: interrupted, refusedNormalOpen: refusalCopy, recoveryPoint: plan.recovery_point, restored, acceptedLoss: receipt.accepted_loss,
  incompleteRefusal, convertedRefusal,
}));
