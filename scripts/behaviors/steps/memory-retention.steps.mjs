import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);
const RESTART_TIMEOUT = 420_000;
const FIXTURE = 'target/manual-memory-retention/fixture.json';
const DAY_MS = 86_400_000;
// Automatic capture runs after the turn; allow it this long to commit before judging.
const CAPTURE_WAIT_MS = 90_000;
const POLL_MS = 3_000;
const GONE = [404, 410];

async function request(w, method, path, payload, expected = [200], headers = {}) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace, ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 2000) });
  assert.ok(expected.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return { status: response.status, body: text && response.ok ? JSON.parse(text) : null };
}
const json = async (w, method, path, payload, expected) => (await request(w, method, path, payload, expected)).body;

async function say(w, label, content) {
  const session = randomUUID();
  w.sessions.push(session);
  const response = await fetch(new URL('/v1/chat/completions', w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(150_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify({ agent_id: w.agentId, model: w.model, session_id: session, stream: true, stream_mode: 'agui',
      memory_enabled: true, max_completion_tokens: 4096, messages: [{ role: 'user', content }] }),
  });
  const text = await response.text();
  assert.equal(response.status, 200, text.slice(0, 600));
  w.evidence.conversations[label] = { session_id: session };
}

const records = async w => {
  const listing = await json(w, 'GET', '/api/memory/records');
  return Array.isArray(listing) ? listing : (listing?.memories ?? listing?.records ?? []);
};
// Record ids arrive as {table, key}; the record routes take 'table:key'.
const recordId = record => typeof record.id === 'string' ? record.id
  : `${record.id.table}:${record.id.key?.String ?? record.id.key}`;
const find = async (w, id) => (await records(w)).find(record => recordId(record) === id);

async function capturedWithin(w, text) {
  const deadline = Date.now() + CAPTURE_WAIT_MS;
  while (Date.now() < deadline) {
    const found = (await records(w)).filter(record => JSON.stringify(record).includes(text));
    if (found.length) return found.map(recordId);
    await new Promise(done => setTimeout(done, POLL_MS));
  }
  return [];
}

async function assertAgedGone(w, label) {
  assert.equal(await find(w, w.fixture.aged.id), undefined, `${label}: the aged memory must not be listed`);
  const direct = await request(w, 'GET', `/api/memory/records/${w.fixture.aged.id}`, undefined, GONE);
  w.evidence[`aged_direct_${label}`] = direct.status;
}

function assertControlValid(w, control, label) {
  assert.ok(control, `${label}: the fresh control memory must stay listed`);
  const capture = control.metadata.uar_capture;
  const expected = Date.parse(capture.first_capture) + 7 * DAY_MS;
  assert.ok(Math.abs(Date.parse(capture.effective_expires_at) - expected) < 1_000, `${label}: control expiry is first_capture + 7 days`);
  assert.ok(Date.parse(capture.effective_expires_at) > Date.now(), `${label}: control is not expired`);
  w.evidence[`control_capture_${label}`] = capture;
}

Given('the host-provisioned aged automatic memory and fresh control memory under a 30-day policy', { timeout: RESTART_TIMEOUT }, async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/memory-retention/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.sessions = [];
  this.evidence = { scenario: 'E03-MEMORY-RETENTION-OPT-OUT-RESTART', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, clock_override: false, conversations: {}, restarts: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  this.fixture = JSON.parse(await readFile(resolve(FIXTURE), 'utf8'));
  this.agentId = this.fixture.agent_id;
  this.evidence.fixture = { path: FIXTURE, agent_id: this.agentId, aged: this.fixture.aged, control: this.fixture.control,
    store_edit: this.fixture.store_edit, created_at: this.fixture.created_at };
  this.evidence.policy_before = await json(this, 'GET', '/api/memory/policy');
  assert.equal(this.evidence.policy_before.enabled, true);
  assert.deepEqual(this.evidence.policy_before.retention, { kind: 'days', days: 30 });
  const aged = await find(this, this.fixture.aged.id);
  const control = await find(this, this.fixture.control.id);
  assert.ok(aged && control, 'Both provisioned memories are visible under the 30-day policy');
  for (const [label, record, minDays, maxDays] of [['aged', aged, 7.9, 8.5], ['control', control, 0, 1]]) {
    const capture = record.metadata.uar_capture;
    assert.equal(capture.automatic, true);
    const ageDays = (Date.now() - Date.parse(capture.first_capture)) / DAY_MS;
    assert.ok(ageDays >= minDays && ageDays <= maxDays, `${label} first_capture is ${ageDays.toFixed(2)} days old`);
    assert.ok(Math.abs(Date.parse(capture.effective_expires_at) - Date.parse(capture.first_capture) - 30 * DAY_MS) < 1_000);
    this.evidence[`${label}_capture_before`] = { ...capture, age_days: ageDays };
  }
});

When('I shorten the retention policy to 7 days', async function () {
  this.evidence.policy_7_days = await json(this, 'PUT', '/api/memory/policy', { enabled: true, retention: { kind: 'days', days: 7 } });
  this.evidence.policy_readback = await json(this, 'GET', '/api/memory/policy');
  assert.deepEqual(this.evidence.policy_readback.retention, { kind: 'days', days: 7 });
});

Then('the aged memory becomes unqueryable while the fresh control memory stays valid', async function () {
  await assertAgedGone(this, 'after_policy');
  assertControlValid(this, await find(this, this.fixture.control.id), 'after_policy');
});

When('I opt out of memory capture, tell the agent a new private fact, and the runtime restarts', { timeout: RESTART_TIMEOUT }, async function () {
  this.evidence.opt_out = await json(this, 'PUT', '/api/memory/policy', { enabled: false, retention: { kind: 'days', days: 7 } });
  assert.equal(this.evidence.opt_out.enabled, false);
  this.newCode = `GATE-${randomUUID().slice(0, 6).toUpperCase()}`;
  await say(this, 'tell_new', `By the way, my garden gate code is ${this.newCode}. Can you suggest a good way to organise a garden shed?`);
  this.evidence.new_captured_before_restart = await capturedWithin(this, this.newCode);
  this.evidence.restarts.after_opt_out = {};
  await restartOwnedUar(this.evidence.restarts.after_opt_out);
});

Then('my opt-out survives the restart and the new fact was never captured', async function () {
  this.evidence.policy_after_restart = await json(this, 'GET', '/api/memory/policy');
  assert.equal(this.evidence.policy_after_restart.enabled, false);
  assert.deepEqual(this.evidence.new_captured_before_restart, [], 'Nothing is captured after opting out');
  this.evidence.new_captured_after_restart = (await records(this))
    .filter(record => JSON.stringify(record).includes(this.newCode)).map(recordId);
  assert.deepEqual(this.evidence.new_captured_after_restart, [], 'No late capture after the restart');
});

Then('the aged memory stays gone after the restart while the control memory remains', async function () {
  await assertAgedGone(this, 'after_restart');
  assertControlValid(this, await find(this, this.fixture.control.id), 'after_restart');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: RESTART_TIMEOUT }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const code of [this.fixture?.aged.code, this.fixture?.control.code, this.newCode].filter(Boolean)) {
    try {
      for (const r of (await records(this)).filter(record => JSON.stringify(record).includes(code))) {
        await request(this, 'DELETE', `/api/memory/records/${recordId(r)}`, undefined, [200, 204, 404, 410]);
      }
    } catch (error) { errors.push(`Memory cleanup: ${error}`); }
  }
  if (this.fixture?.original_policy) {
    const original = this.fixture.original_policy;
    try { await request(this, 'PUT', '/api/memory/policy', { enabled: original.enabled, retention: original.retention ?? null }); }
    catch (error) { errors.push(`Policy restore: ${error}`); }
  }
  if (this.agentId) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); } catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); } catch (error) { errors.push(`Session cleanup: ${error}`); }
  }
  if (this.fixture?.original_settings) {
    try {
      for (const [key, original] of Object.entries(this.fixture.original_settings)) {
        const current = await json(this, 'GET', `/api/uar/settings/${key}`);
        if (original.source === 'Default') {
          await request(this, 'DELETE', `/api/uar/settings/${key}`, undefined, [200, 204], { 'If-Match': current.revision });
        } else {
          await request(this, 'PUT', `/api/uar/settings/${key}`, { value: original.saved, expected_revision: current.revision });
        }
      }
      this.evidence.restarts.restore = {};
      await restartOwnedUar(this.evidence.restarts.restore);
    } catch (error) { errors.push(`Settings restore: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});
