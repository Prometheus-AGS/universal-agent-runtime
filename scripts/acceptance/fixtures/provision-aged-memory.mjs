// Host fixture for catalog E03 (memory retention). The product genuinely
// captures two automatic memories under a 30-day policy: one that will be aged
// and a fresh control. Only the aged control's capture time is then moved eight
// days back, in the stopped store, with the SurrealDB SDK UAR links
// (example `surrealql_store_runner`). No clock override and no fake TTL.
//
// Usage: node scripts/acceptance/fixtures/provision-aged-memory.mjs <surrealql_store_runner binary>
// Env: UAR_BASE_URL, UAR_WORKSPACE_ID, UAR_MODEL. The test sends no credentials.
// Writes target/manual-memory-retention/fixture.json; the E03 scenario reads it,
// and restores the settings, policy and agent this fixture changes.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { randomUUID } from 'node:crypto';
import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, isAbsolute, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { restartOwnedUar } from '../../behaviors/support/restart-owned-uar.mjs';

const FIXTURE = resolve('target/manual-memory-retention/fixture.json');
const SETTINGS = { 'memory.enabled': true, 'memory.auto_capture': true };
const AGE_DAYS = 8;
const CAPTURE_WAIT_MS = 90_000;
const POLL_MS = 3_000;

const runner = process.argv[2];
assert.ok(runner, 'Pass the surrealql_store_runner binary path');
const base = new URL(process.env.UAR_BASE_URL);
const workspace = process.env.UAR_WORKSPACE_ID;
const model = process.env.UAR_MODEL;
assert.ok(workspace && model?.includes('/'), 'Set UAR_WORKSPACE_ID and UAR_MODEL');
assert.ok(['http:', 'https:'].includes(base.protocol) && !base.username && !base.password);
await access(FIXTURE).then(() => { throw new Error(`Fixture already exists: ${FIXTURE}`); }, () => {});

const fixture = { purpose: 'E03 aged automatic memory and fresh control', created_at: new Date().toISOString(),
  age_days: AGE_DAYS, model, restarts: {}, exchanges: 0 };

async function request(method, path, payload, expected = [200], headers = {}) {
  const response = await fetch(new URL(path, base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': workspace, ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  fixture.exchanges += 1;
  const text = await response.text();
  assert.ok(expected.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

async function say(agentId, content) {
  const response = await fetch(new URL('/v1/chat/completions', base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(150_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': workspace },
    body: JSON.stringify({ agent_id: agentId, model, session_id: randomUUID(), stream: true, stream_mode: 'agui',
      memory_enabled: true, max_completion_tokens: 4096, messages: [{ role: 'user', content }] }),
  });
  await response.text();
  return response.status;
}

const records = async () => {
  const listing = await request('GET', '/api/memory/records');
  return Array.isArray(listing) ? listing : (listing?.memories ?? listing?.records ?? []);
};
const recordId = record => typeof record.id === 'string' ? record.id
  : `${record.id.table}:${record.id.key?.String ?? record.id.key}`;

async function capture(agentId, code, content) {
  await say(agentId, content);
  const deadline = Date.now() + CAPTURE_WAIT_MS;
  while (Date.now() < deadline) {
    const found = (await records()).filter(record => JSON.stringify(record).includes(code));
    if (found.length) {
      assert.equal(found.length, 1, `Expected one captured memory for ${code}`);
      assert.equal(found[0].metadata?.uar_capture?.automatic, true, `${code} must be an automatic capture`);
      return found[0];
    }
    await delay(POLL_MS);
  }
  throw new Error(`The product did not capture ${code} within ${CAPTURE_WAIT_MS} ms`);
}

function alive(pid) {
  try { process.kill(pid, 0); return true; } catch { return false; }
}

// 1. Turn on durable memory with automatic capture; restarting also records the
// adapter's recovery context, which step 4 uses to start the stopped runtime.
fixture.original_settings = {};
for (const key of Object.keys(SETTINGS)) {
  const setting = await request('GET', `/api/uar/settings/${key}`);
  fixture.original_settings[key] = { saved: setting.saved, source: setting.source };
}
for (const [key, value] of Object.entries(SETTINGS)) {
  const current = await request('GET', `/api/uar/settings/${key}`);
  await request('PUT', `/api/uar/settings/${key}`, { value, expected_revision: current.revision });
}
fixture.restarts.enable = {};
await restartOwnedUar(fixture.restarts.enable);
fixture.original_policy = await request('GET', '/api/memory/policy');

// 2. A disposable agent; the product captures two facts under a 30-day policy.
const template = await request('GET', '/api/agents/default-agent');
const agentId = `disposable-retention-agent-${randomUUID().slice(0, 8)}`;
const agent = structuredClone(template);
Object.assign(agent, { id: agentId, version: '1.0.0', extensions: {} });
agent.metadata = { ...agent.metadata, title: 'Disposable Retention Agent', description: 'Remembers what the user tells it.', tags: ['memory'] };
agent.prompt = { system: 'Answer briefly. Never save or delete memories yourself; the host decides what is remembered. '
  + 'When asked about the user, use only facts from memory. If memory does not contain the answer, reply exactly UNKNOWN. Never guess.', instructions: [] };
agent.runtime.entry = 'default';
const split = model.indexOf('/');
agent.policy.provider = { default: { provider: model.slice(0, split), model: model.slice(split + 1) }, fallbacks: [] };
agent.policy.tools.allow = [];
agent.memory.kb.enabled = false;
agent.memory.conversation = { enabled: true };
await request('POST', '/api/agents', agent, [201]);
fixture.agent_id = agentId;
fixture.warmup_status = await say(agentId, 'Hello.');
fixture.policy_30_days = await request('PUT', '/api/memory/policy', { enabled: true, retention: { kind: 'days', days: 30 } });

const agedCode = `LOCKER-${randomUUID().slice(0, 6).toUpperCase()}`;
const controlCode = `BIKE-${randomUUID().slice(0, 6).toUpperCase()}`;
const aged = await capture(agentId, agedCode, `By the way, my storage locker code is ${agedCode}. Can you suggest a good way to organise a storage locker?`);
const control = await capture(agentId, controlCode, `By the way, my bicycle lock code is ${controlCode}. Can you suggest a good bicycle maintenance routine?`);
fixture.aged = { code: agedCode, id: recordId(aged), capture_before: aged.metadata.uar_capture };
fixture.control = { code: controlCode, id: recordId(control), capture: control.metadata.uar_capture };

// 3. Stop the owned runtime and move only the aged control's capture time.
const launch = JSON.parse(await readFile(resolve('target/prompt-macos-launch.json'), 'utf8'));
const config = JSON.parse(await readFile(launch.config, 'utf8'));
const memory = config.memory ?? {};
const dbPath = isAbsolute(memory.db_path) ? memory.db_path : resolve(dirname(launch.config), memory.db_path);
const endpoint = memory.surreal_endpoint ?? `surrealkv://${dbPath}`;
const namespace = memory.namespace ?? 'uar';
const database = `${memory.database ?? 'memory'}__owner_v1`;
const agedKey = fixture.aged.id.replace(/^memory:/, '');
const controlKey = fixture.control.id.replace(/^memory:/, '');
for (const key of [agedKey, controlKey]) assert.match(key, /^[A-Za-z0-9_-]+$/);
process.kill(launch.pid, 'SIGTERM');
const stopDeadline = Date.now() + 45_000;
while (alive(launch.pid) && Date.now() < stopDeadline) await delay(250);
assert.equal(alive(launch.pid), false, 'Owned runtime must stop gracefully before the store is edited');
const statements = [
  `UPDATE uar_memory_control SET first_capture = time::now() - ${AGE_DAYS}d, expires_at = time::now() - ${AGE_DAYS}d + 30d `
    + `WHERE record::id(memory_id) = '${agedKey}' AND automatic = true AND deleted = false RETURN AFTER;`,
  `SELECT record::id(memory_id) AS key, automatic, first_capture, expires_at, retention_days, deleted FROM uar_memory_control `
    + `WHERE record::id(memory_id) IN ['${agedKey}', '${controlKey}'];`,
].join('\n');
const edit = spawnSync(runner, [endpoint, namespace, database], { input: statements, encoding: 'utf8', timeout: 120_000 });
fixture.store_edit = { endpoint_kind: endpoint.split(':')[0], namespace, database, statements, status: edit.status,
  stdout: edit.stdout?.trim().split('\n'), stderr_tail: edit.stderr?.trim().split('\n').slice(-5) };

// 4. Start the stopped runtime again with the recorded recovery context.
fixture.restarts.after_store_edit = {};
await restartOwnedUar(fixture.restarts.after_store_edit, undefined, undefined, true);
assert.equal(edit.status, 0, `Store edit failed: ${edit.stderr}`);
const updated = JSON.parse(fixture.store_edit.stdout[0]);
assert.equal(updated.length, 1, 'Exactly the aged control must be updated');
fixture.warmup_after_edit_status = await say(agentId, 'Hello.');
const listed = (await records()).find(record => recordId(record) === fixture.aged.id);
assert.ok(listed, 'The aged memory stays visible under the 30-day policy');
fixture.aged.capture_after = listed.metadata.uar_capture;
const ageMs = Date.now() - Date.parse(listed.metadata.uar_capture.first_capture);
assert.ok(ageMs > (AGE_DAYS - 0.1) * 86_400_000 && ageMs < (AGE_DAYS + 0.1) * 86_400_000, 'first_capture is eight days old');
await mkdir(dirname(FIXTURE), { recursive: true });
await writeFile(FIXTURE, JSON.stringify(fixture, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
console.log(JSON.stringify({ ok: true, fixture: FIXTURE, agent_id: agentId, aged: fixture.aged.id, control: fixture.control.id,
  aged_first_capture: fixture.aged.capture_after.first_capture }));
