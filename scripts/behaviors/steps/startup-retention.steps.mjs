import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, Then, When, setDefaultTimeout } from '@cucumber/cucumber';
import { ageSession, logLines, restartDisposable, startDisposable, stopDisposable } from '../../manual-host/c52-disposable-instance.mjs';

setDefaultTimeout(240_000);
const TTL_SECS = 3_600;
const BUILTINS = ['default-agent', 'orchestrator-agent', 'general-purpose', 'rust-reviewer', 'compiler-agent'];
const SETTING = 'harness.skill_reattachment';
const OVERRIDE = { per_skill_tokens: 4_321, total_tokens: 12_345 };
const SEEDED = 'Built-in agent seeded';
const PURGED = 'persisted session purged by TTL sweep';
const SWEEP_WAIT_MS = 60_000;
const SETTLE_MS = 10_000;

// The disposable instance is a fresh store, so no workspace header is sent; the
// owner comes from the external proxy in front of it. The test sends no credentials.
async function request(w, method, path, body, accepted = [200]) {
  const exchange = { method, path, request: body };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(180_000),
    headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  exchange.status = response.status;
  const text = await response.text();
  exchange.response = text.slice(0, 2_000);
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 600)}`);
  return text ? JSON.parse(text) : null;
}

async function turn(w, label, sessionId, content) {
  const reply = await request(w, 'POST', '/v1/chat/completions', { agent_id: 'default-agent', model: process.env.UAR_MODEL,
    session_id: sessionId, stream: false, max_completion_tokens: 512, messages: [{ role: 'user', content }] });
  const answer = reply?.choices?.[0]?.message?.content ?? '';
  w.evidence.turns[label] = { session_id: sessionId, answer: String(answer).slice(0, 500) };
  assert.ok(String(answer).trim(), `${label}: empty answer`);
}

const session = (w, id, accepted = [200]) => request(w, 'GET', `/api/uar/sessions/${id}`, undefined, accepted);

async function builtins(w) {
  const agents = {};
  for (const id of BUILTINS) agents[id] = await request(w, 'GET', `/api/agents/${id}`);
  return agents;
}

Given('a disposable UAR has completed its initial startup on an empty owned store', { timeout: 360_000 }, async function () {
  assert.ok(process.env.UAR_MODEL, 'Set UAR_MODEL');
  this.record = resolve(`.prometheus/verification/startup-retention/${randomUUID()}.json`);
  this.evidence = { scenario: 'C52-STARTUP-RETENTION-01', platform: process.platform, authentication_supplied_by_test: false,
    previous_scenarios_rerun: false, owned_runtime_touched: false, exchanges: [], turns: {}, fixture: {} };
  this.fx = await startDisposable(this.evidence.fixture, TTL_SECS);
  this.base = new URL(this.fx.baseUrl);
  // A fresh Surreal 3.3 server defines only its own default namespace 'main'; the store namespace must not exist yet.
  const namespaces = this.evidence.fixture.empty_store_before_start?.[0]?.namespaces ?? null;
  assert.ok(namespaces && typeof namespaces === 'object', 'Missing root namespace listing');
  assert.deepEqual(Object.keys(namespaces), ['main'], 'Only the server default namespace may exist before the initial startup');
  assert.ok(!Object.hasOwn(namespaces, 'prompt'), 'The store namespace must not exist before the initial startup');
  this.initialLog = this.evidence.fixture.startups[0].log;
  this.evidence.log_lines = { initial_seeded: await logLines(this.initialLog, SEEDED) };
  this.evidence.initial_seed_lines = this.evidence.log_lines.initial_seeded.length;
  this.builtinsInitial = await builtins(this);
  this.seeded = await request(this, 'GET', `/api/uar/settings/${SETTING}`);
  this.evidence.seeded_setting = { source: this.seeded.source, effective: this.seeded.effective, revision: this.seeded.revision };
});

When('I save an operator override of a seeded setting', async function () {
  await request(this, 'PUT', `/api/uar/settings/${SETTING}`, { value: OVERRIDE, expected_revision: this.seeded.revision });
  this.saved = await request(this, 'GET', `/api/uar/settings/${SETTING}`);
  assert.deepEqual(this.saved.effective, OVERRIDE);
  this.evidence.saved_setting = { source: this.saved.source, effective: this.saved.effective, revision: this.saved.revision };
});

When('the disposable UAR performs a repeat startup on the same store', { timeout: 360_000 }, async function () {
  const repeat = await restartDisposable(this.fx, this.evidence.fixture, 'repeat');
  this.repeatLog = repeat.log;
  assert.notEqual(repeat.pid, repeat.previous_pid);
});

Then('built-ins were seeded once and the override is still effective', async function () {
  assert.equal(this.evidence.initial_seed_lines, BUILTINS.length, 'Each built-in is seeded on the initial startup');
  this.evidence.log_lines.repeat_seeded = await logLines(this.repeatLog, SEEDED);
  this.evidence.repeat_seed_lines = this.evidence.log_lines.repeat_seeded.length;
  assert.equal(this.evidence.repeat_seed_lines, 0, 'No built-in is seeded again on repeat startup');
  this.builtinsAfterRepeat = await builtins(this);
  assert.deepEqual(this.builtinsAfterRepeat, this.builtinsInitial, 'Built-in definitions are unchanged by repeat startup');
  const after = await request(this, 'GET', `/api/uar/settings/${SETTING}`);
  this.evidence.setting_after_repeat = { source: after.source, effective: after.effective, revision: after.revision };
  assert.deepEqual(after.effective, OVERRIDE);
  assert.equal(after.revision, this.saved.revision, 'The saved override keeps its revision across restart');
});

When('I create a terminal session and a fresh control session', async function () {
  // A fresh store has no model verification; verify the configured model through the product API first.
  const split = process.env.UAR_MODEL.indexOf('/');
  assert.ok(split > 0, 'UAR_MODEL must be provider/model');
  const [provider, model] = [process.env.UAR_MODEL.slice(0, split), process.env.UAR_MODEL.slice(split + 1)];
  const verification = await request(this, 'POST', `/api/uar/providers/${encodeURIComponent(provider)}/test`,
    { model, output_token_cap: 512 });
  this.evidence.model_verification = { provider, model, ok: verification?.ok, model_id: verification?.model_id };
  assert.equal(verification?.ok, true, `Verification of ${process.env.UAR_MODEL}: ${JSON.stringify(verification).slice(0, 300)}`);
  this.terminal = randomUUID();
  this.control = randomUUID();
  await turn(this, 'terminal', this.terminal, 'Reply with only the word READY.');
  await turn(this, 'control', this.control, 'Reply with only the word READY.');
  const terminal = await session(this, this.terminal);
  const control = await session(this, this.control);
  this.owner = terminal.owner_id;
  assert.equal(control.owner_id, this.owner);
  this.evidence.before_aging = { terminal: terminal.last_activity, control: control.last_activity };
});

When('the host ages only the terminal session past retention', async function () {
  const aged = new Date(Date.now() - 2 * TTL_SECS * 1_000).toISOString();
  this.evidence.host_aging = ageSession(this.fx, this.terminal, this.owner, aged);
  const terminal = await session(this, this.terminal);
  const control = await session(this, this.control);
  assert.equal(Date.parse(terminal.last_activity), Date.parse(aged));
  assert.equal(control.last_activity, this.evidence.before_aging.control, 'The control session is not aged');
  this.evidence.after_aging = { terminal: terminal.last_activity, control: control.last_activity };
});

When('a new turn publishes session activity', async function () {
  this.trigger = randomUUID();
  await turn(this, 'trigger', this.trigger, 'Reply with only the word READY.');
  const deadline = Date.now() + SWEEP_WAIT_MS;
  let removed = false;
  while (!removed && Date.now() < deadline) {
    removed = (await fetch(new URL(`/api/uar/sessions/${this.terminal}`, this.base))).status === 404;
    if (!removed) await new Promise(done => setTimeout(done, 1_000));
  }
  this.evidence.terminal_removed_within_ms = removed ? SWEEP_WAIT_MS - (deadline - Date.now()) : null;
});

Then('the retention worker removes only the expired session', async function () {
  await session(this, this.terminal, [404]);
  await session(this, this.control);
  await session(this, this.trigger);
  this.evidence.log_lines.purged_after_first_signal = await logLines(this.repeatLog, PURGED);
  this.evidence.purge_lines_after_first_signal = this.evidence.log_lines.purged_after_first_signal.length;
  assert.equal(this.evidence.purge_lines_after_first_signal, 1, 'Exactly one session is purged');
});

When('a second activity signal arrives', async function () {
  await turn(this, 'second_signal', this.trigger, 'Reply with only the word AGAIN.');
  await new Promise(done => setTimeout(done, SETTLE_MS));
});

Then('nothing else is removed and the protected definitions remain', async function () {
  await session(this, this.control);
  await session(this, this.trigger);
  this.evidence.log_lines.purged_after_second_signal = await logLines(this.repeatLog, PURGED);
  this.evidence.purge_lines_after_second_signal = this.evidence.log_lines.purged_after_second_signal.length;
  assert.equal(this.evidence.purge_lines_after_second_signal, 1, 'The second signal purges nothing more');
  assert.deepEqual(await builtins(this), this.builtinsInitial, 'Built-in definitions survive the sweep unchanged');
  const setting = await request(this, 'GET', `/api/uar/settings/${SETTING}`);
  assert.deepEqual(setting.effective, OVERRIDE);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 180_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.fx) {
    try { await stopDisposable(this.fx, this.evidence.fixture); } catch (error) { errors.push(`teardown: ${error.message}`); }
  }
  this.evidence.status = result?.status === 'PASSED' ? 'passed' : String(result?.status ?? 'unknown').toLowerCase();
  if (result?.message) this.evidence.error = result.message.slice(0, 4_000);
  this.evidence.cleanup_errors = errors;
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  console.log(`EVIDENCE ${this.record} status=${this.evidence.status}`);
});
