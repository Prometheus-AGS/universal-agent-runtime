import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);
const RESTART_TIMEOUT = 420_000;
const SETTINGS = { 'memory.enabled': true, 'memory.auto_capture': true };
// Automatic capture runs after the turn; allow it this long to commit before judging.
const CAPTURE_WAIT_MS = 90_000;
const POLL_MS = 3_000;

async function request(w, method, path, payload, expected = [200], headers = {}) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace, ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 2000) });
  assert.ok(expected.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

// One fresh conversation in AG-UI stream mode; injected memory is reported as agui.memory.context.
async function say(w, label, content, { tolerateErrors = false } = {}) {
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
  const events = text.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).replace(/^ /, '')).join('\n');
    if (!data || data === '[DONE]') return [];
    try { return [{ name, data: JSON.parse(data) }]; } catch { return [{ name, data }]; }
  });
  const answer = events.filter(e => e.name === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  const record = { session_id: session, memory_context: events.filter(e => e.name === 'agui.memory.context').map(e => e.data),
    errors: events.filter(e => /error/i.test(e.name ?? '')).map(e => e.data), answer: answer.slice(0, 1000) };
  w.evidence.conversations[label] = record;
  if (!tolerateErrors) assert.deepEqual(record.errors, [], `${label} must not error`);
  return record;
}

const records = async w => {
  const listing = await request(w, 'GET', '/api/memory/records');
  return Array.isArray(listing) ? listing : (listing?.memories ?? listing?.records ?? []);
};
const mentioning = async (w, text) => (await records(w)).filter(record => JSON.stringify(record).includes(text));
// Record ids arrive as {table, key}; the record routes take 'table:key'.
const recordId = record => typeof record.id === 'string' ? record.id
  : `${record.id.table}:${record.id.key?.String ?? record.id.key}`;

async function waitForCapture(w, text) {
  const deadline = Date.now() + CAPTURE_WAIT_MS;
  while (Date.now() < deadline) {
    const found = await mentioning(w, text);
    if (found.length) return found;
    await new Promise(done => setTimeout(done, POLL_MS));
  }
  return [];
}

// After a restart the first model call has timed out at stream start (recorded separately).
// One labelled warm-up turn records that outcome without failing the scenario.
async function restart(w, label) {
  w.evidence.restarts[label] = {};
  await restartOwnedUar(w.evidence.restarts[label]);
  if (!w.agentId || label === 'restore') return;
  try { await say(w, `warmup_after_${label}`, 'Hello.', { tolerateErrors: true }); }
  catch (error) { w.evidence.conversations[`warmup_after_${label}`] = { transport_error: String(error) }; }
}

Given('durable memory with automatic capture is enabled for a disposable agent after an owned restart', { timeout: RESTART_TIMEOUT }, async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/memory-opt-out/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.sessions = [];
  this.evidence = { scenario: 'E03-MEMORY-OPT-OUT-RESTART', scope: 'opt-out-survives-restart half; retention expiry deferred until genuinely aged data exists',
    platform: process.platform, model: this.model, authentication_supplied_by_test: false, previous_scenarios_rerun: false,
    conversations: {}, restarts: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  this.originalSettings = {};
  for (const key of Object.keys(SETTINGS)) {
    const setting = await request(this, 'GET', `/api/uar/settings/${key}`);
    this.originalSettings[key] = { saved: setting.saved, source: setting.source };
  }
  this.settingsChanged = true;
  for (const [key, value] of Object.entries(SETTINGS)) {
    const current = await request(this, 'GET', `/api/uar/settings/${key}`);
    await request(this, 'PUT', `/api/uar/settings/${key}`, { value, expected_revision: current.revision });
  }
  await restart(this, 'enable');
  this.originalPolicy = await request(this, 'GET', '/api/memory/policy');
  this.evidence.original_policy = this.originalPolicy;
  const template = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-opt-out-agent-${this.short}`;
  const artifact = structuredClone(template);
  Object.assign(artifact, { id: this.agentId, version: '1.0.0', extensions: {} });
  artifact.metadata = { ...artifact.metadata, title: 'Disposable Memory Agent', description: 'Remembers what the user tells it.', tags: ['memory'] };
  artifact.prompt = { system: 'Answer briefly. Never save or delete memories yourself; the host decides what is remembered. '
    + 'When asked about the user, use only facts from memory. If memory does not contain the answer, reply exactly UNKNOWN. Never guess.', instructions: [] };
  artifact.runtime.entry = 'default';
  const split = this.model.indexOf('/');
  artifact.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  // Recall goes through the memory tools on this runtime, so tools stay on Auto (as in C39).
  artifact.policy.tools.allow = [];
  artifact.memory.kb.enabled = false;
  artifact.memory.conversation = { enabled: true };
  assert.equal((await request(this, 'POST', '/api/agents', artifact, [201])).id, this.agentId);
});

When('I opt in, tell the agent a private fact, and ask about it in a fresh conversation', async function () {
  this.evidence.opt_in = await request(this, 'PUT', '/api/memory/policy', { enabled: true, retention: { kind: 'days', days: 30 } });
  assert.equal(this.evidence.opt_in.enabled, true);
  this.firstCode = `LOCKER-${randomUUID().slice(0, 6).toUpperCase()}`;
  await say(this, 'tell_first', `By the way, my storage locker code is ${this.firstCode}. Can you suggest a good way to organise a storage locker?`);
  this.evidence.first_captured = (await waitForCapture(this, this.firstCode)).map(recordId);
  this.recall = await say(this, 'ask_first', 'What is my storage locker code? If you do not know it, reply exactly UNKNOWN.');
});

Then('the agent recalls the fact from memory', function () {
  assert.ok(this.evidence.first_captured.length >= 1, 'With capture on, the fact is remembered');
  assert.ok(this.recall.answer.includes(this.firstCode), `Recall answer: ${this.recall.answer}`);
});

When('I opt out of memory capture and the runtime restarts', { timeout: RESTART_TIMEOUT }, async function () {
  this.evidence.opt_out = await request(this, 'PUT', '/api/memory/policy', { enabled: false, retention: null });
  assert.equal(this.evidence.opt_out.enabled, false);
  await restart(this, 'after_opt_out');
});

Then('my opt-out is still in effect', async function () {
  this.evidence.policy_after_restart = await request(this, 'GET', '/api/memory/policy');
  assert.equal(this.evidence.policy_after_restart.enabled, false);
});

When('I tell the agent a new private fact and ask about it in a fresh conversation', async function () {
  this.secondCode = `GATE-${randomUUID().slice(0, 6).toUpperCase()}`;
  await say(this, 'tell_second', `By the way, my garden gate code is ${this.secondCode}. Can you suggest a good way to organise a garden shed?`);
  this.evidence.second_captured = (await waitForCapture(this, this.secondCode)).map(recordId);
  this.secondRecall = await say(this, 'ask_second', 'What is my garden gate code? If you do not know it, reply exactly UNKNOWN.');
});

Then('the new fact was never captured and the agent cannot recall it', function () {
  assert.deepEqual(this.evidence.second_captured, [], 'Nothing is captured after opting out');
  assert.ok(!this.secondRecall.answer.includes(this.secondCode), `Recall answer: ${this.secondRecall.answer}`);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: RESTART_TIMEOUT }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  // E03_EXTRA_CLEANUP_CODES names records an earlier failed attempt could not delete.
  const extra = (process.env.E03_EXTRA_CLEANUP_CODES ?? '').split(',').filter(Boolean);
  for (const code of [this.firstCode, this.secondCode, ...extra].filter(Boolean)) {
    try { for (const r of await mentioning(this, code)) await request(this, 'DELETE', `/api/memory/records/${recordId(r)}`, undefined, [200, 204, 404, 410]); }
    catch (error) { errors.push(`Memory cleanup: ${error}`); }
  }
  if (this.originalPolicy) {
    try { await request(this, 'PUT', '/api/memory/policy', { enabled: this.originalPolicy.enabled, retention: this.originalPolicy.retention ?? null }); }
    catch (error) { errors.push(`Policy restore: ${error}`); }
  }
  if (this.created) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); } catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); } catch (error) { errors.push(`Session cleanup: ${error}`); }
  }
  if (this.settingsChanged) {
    try {
      for (const [key, original] of Object.entries(this.originalSettings)) {
        const current = await request(this, 'GET', `/api/uar/settings/${key}`);
        if (original.source === 'Default') {
          await request(this, 'DELETE', `/api/uar/settings/${key}`, undefined, [200, 204], { 'If-Match': current.revision });
        } else {
          await request(this, 'PUT', `/api/uar/settings/${key}`, { value: original.saved, expected_revision: current.revision });
        }
      }
      await restart(this, 'restore');
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
