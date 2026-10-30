import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { startDedicatedRuntime } from '../../acceptance/fixtures/dedicated-runtime.mjs';

setDefaultTimeout(180_000);
const RECOVERY_DEADLINE_MS = 90_000;
const LOST = 'Remote transport lost; reconnecting';
const RECOVERED = 'Remote transport recovered';

// The fixture's identity proxy supplies the owner; the test sends no credentials.
async function request(w, method, path, payload, expected = 200) {
  const res = await fetch(new URL(path, w.runtime.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json' },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await res.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: res.status, response: text });
  assert.equal(res.status, expected, `${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

async function saveSkill(w, label) {
  const name = `remote-surreal-${label}-${randomUUID().slice(0, 8)}`;
  const saved = await request(w, 'POST', '/api/uar/skills',
    { name, description: `Restart probe ${label}`, triggers: { keywords: [name] }, prompt_overlay: `Probe ${label}.` }, 201);
  assert.equal(saved.skill_id, name);
  w.owned.push(`/api/uar/skills/${name}`);
  return name;
}

Given('a standalone UAR backed by a remote SurrealDB server and an owned skill', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.owned = [];
  this.holder = {};
  const id = randomUUID();
  this.record = resolve(`.prometheus/verification/remote-surreal-restart/${id}.json`);
  this.evidence = { scenario: 'ISSUE-441-REMOTE-SURREAL-RESTART', platform: process.platform,
    authentication_supplied_by_test: false, uar_restarted_by_test: false, exchanges: [], readiness: [] };
  this.evidence.fixture = {};
  this.runtime = await startDedicatedRuntime(this.evidence.fixture, this.holder, { backend: 'surreal', label: 'remote-surreal-restart' });
  this.uarPid = this.runtime.uarPid;
  this.skill = await saveSkill(this, 'before');
});

When('the SurrealDB server restarts while UAR keeps running', { timeout: 240_000 }, async function () {
  this.evidence.restart_started_at = new Date().toISOString();
  this.runtime.restartDatabase();
  this.evidence.restart_completed_at = new Date().toISOString();
});

Then('readiness returns to ready within {int} seconds without restarting UAR', { timeout: 150_000 }, async function (seconds) {
  assert.equal(seconds * 1000, RECOVERY_DEADLINE_MS);
  const started = Date.now();
  while (Date.now() - started < RECOVERY_DEADLINE_MS) {
    assert.ok(this.runtime.uarAlive(), 'UAR must keep running through the SurrealDB restart');
    const res = await fetch(new URL('/readyz', this.runtime.runtime), { signal: AbortSignal.timeout(10_000) });
    const body = await res.text();
    this.evidence.readiness.push({ elapsed_ms: Date.now() - started, status: res.status, body });
    if (res.status === 200 && (await this.runtime.logMessages()).includes(RECOVERED)) break;
    await delay(1_000);
  }
  const last = this.evidence.readiness.at(-1);
  assert.equal(last?.status, 200, `UAR did not become ready within ${seconds} s`);
  const checks = JSON.parse(last.body).checks;
  assert.deepEqual(checks.persistence, { backend: 'surreal', status: 'ok' });
  assert.equal(checks.postgres, undefined, 'A Surreal backend must not be reported as postgres');
  assert.equal(this.runtime.uarPid, this.uarPid);
  this.evidence.recovered_after_ms = last.elapsed_ms;
});

Then('the owned skill is still readable and a new skill can be saved', async function () {
  const skill = await request(this, 'GET', `/api/uar/skills/${this.skill}`);
  assert.equal(skill.skill_id, this.skill);
  this.evidence.skill_after = await saveSkill(this, 'after');
});

Then('the UAR log records the transport loss and its recovery', async function () {
  const messages = await this.runtime.logMessages();
  const lost = messages.indexOf(LOST);
  const recovered = messages.indexOf(RECOVERED);
  this.evidence.log = { lost_index: lost, recovered_index: recovered,
    reconnect_failures: messages.filter(m => m === 'Remote transport reconnect failed').length };
  assert.ok(lost >= 0, 'The SurrealDB restart must be observed as a transport loss');
  assert.ok(recovered > lost, 'Recovery must follow the observed loss');
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.runtime) {
    for (const path of [...this.owned].reverse()) {
      try { await request(this, 'DELETE', path, undefined, 204); }
      catch (error) { errors.push(`${path}: ${error}`); }
    }
  }
  if (this.holder?.handle) errors.push(...await this.holder.handle.stop());
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});
