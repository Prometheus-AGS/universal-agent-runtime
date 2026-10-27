import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(125_000);
const RESTART_TIMEOUT = 420_000;
// Auto-capture is off during the scenario so a recall answer cannot create a
// second copy of the fact that would outlive the deleted record.
const SETTINGS = { 'memory.enabled': true, 'memory.auto_capture': false };
const QUESTION = 'What is my storage locker code? If you do not know it, reply exactly UNKNOWN.';

async function request(w, method, path, payload, expected = 200, headers = {}) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace, ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 4000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  if (!text || !response.ok) return null;
  try { return JSON.parse(text); } catch { return text; }
}

// One fresh conversation through the OpenAI-compatible route in AG-UI stream mode,
// which reports injected memory context as agui.memory.context.
async function ask(w, label) {
  const session = randomUUID();
  w.sessions.push(session);
  const response = await fetch(new URL('/v1/chat/completions', w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify({ agent_id: w.agentId, model: w.model, session_id: session, stream: true, stream_mode: 'agui',
      memory_enabled: true, max_completion_tokens: 4096, messages: [{ role: 'user', content: QUESTION }] }),
  });
  const text = await response.text();
  assert.equal(response.status, 200, `/v1/chat/completions: ${response.status}: ${text.slice(0, 800)}`);
  const events = text.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).replace(/^ /, '')).join('\n');
    if (!data || data === '[DONE]') return [];
    try { return [{ name, data: JSON.parse(data) }]; } catch { return [{ name, data }]; }
  });
  const answer = events.filter(e => e.name === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  const memory = events.filter(e => e.name === 'agui.memory.context').map(e => e.data);
  const toolCalls = events.filter(e => e.name === 'agui.tool_call.complete').map(e => ({ id: e.data?.id, name: e.data?.name, arguments: e.data?.arguments_json }));
  const toolResults = events.filter(e => e.name === 'agui.tool_result').map(e => ({ id: e.data?.id, name: e.data?.name, content: String(e.data?.content ?? '') }));
  const record = { session_id: session, event_names: [...new Set(events.map(e => e.name))], memory_context: memory,
    tool_calls: toolCalls, tool_results: toolResults.map(r => ({ ...r, content: r.content.slice(0, 2000) })),
    errors: events.filter(e => /error/i.test(e.name ?? '')).map(e => e.data), answer: answer.slice(0, 2000) };
  w.evidence.conversations[label] = record;
  return record;
}

async function restart(w, label) {
  w.evidence.restarts[label] = {};
  await restartOwnedUar(w.evidence.restarts[label]);
}

const exported = async w => {
  const listing = await request(w, 'GET', '/api/memory/records');
  return Array.isArray(listing) ? listing : (listing?.records ?? listing?.memories ?? []);
};
const mentions = (records, text) => records.filter(record => JSON.stringify(record).includes(text));
// Actual memory provenance: injected memory context, or a memory_recall result
// from the memory store that carries the expected value.
const recalledFromMemory = (turn, code) => turn.memory_context.some(event => event.count > 0)
  || turn.tool_results.some(result => /memory/i.test(result.name ?? '') && result.content.includes(code));

async function restoreSettings(w) {
  for (const [key, original] of Object.entries(w.originalSettings)) {
    const current = await request(w, 'GET', `/api/uar/settings/${key}`);
    if (original.source === 'Default') {
      await request(w, 'DELETE', `/api/uar/settings/${key}`, undefined, [200, 204], { 'If-Match': current.revision });
    } else {
      await request(w, 'PUT', `/api/uar/settings/${key}`, { value: original.saved, expected_revision: current.revision });
    }
  }
}

Given('durable memory is enabled for the owned runtime after an owned restart', { timeout: RESTART_TIMEOUT }, async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/memory-lifecycle/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.sessions = [];
  this.evidence = { scenario: 'C39-MEMORY-LIFECYCLE-01', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, conversations: {}, restarts: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  this.originalSettings = {};
  for (const key of Object.keys(SETTINGS)) {
    const setting = await request(this, 'GET', `/api/uar/settings/${key}`);
    this.originalSettings[key] = { saved: setting.saved, source: setting.source };
  }
  assert.equal(this.originalSettings['memory.enabled'].saved, false, 'Memory must start disabled');
  this.evidence.original_settings = this.originalSettings;
  this.settingsChanged = true;
  for (const [key, value] of Object.entries(SETTINGS)) {
    const current = await request(this, 'GET', `/api/uar/settings/${key}`);
    await request(this, 'PUT', `/api/uar/settings/${key}`, { value, expected_revision: current.revision });
  }
  await restart(this, 'enable');
  for (const [key, value] of Object.entries(SETTINGS)) {
    assert.equal((await request(this, 'GET', `/api/uar/settings/${key}`)).effective, value);
  }
  this.evidence.memory_policy = await request(this, 'GET', '/api/memory/policy');
});

Given('a disposable agent with conversation memory and no knowledge or tools', async function () {
  const template = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-memory-agent-${this.short}`;
  const artifact = structuredClone(template);
  Object.assign(artifact, { id: this.agentId, version: '1.0.0', extensions: {} });
  artifact.metadata = { ...artifact.metadata, title: 'Disposable Memory Agent', description: 'Answers from saved user memory only.', tags: ['memory'] };
  artifact.prompt = { system: 'Answer briefly. Use only facts from the memory context provided to you. '
    + 'If the memory context does not contain the answer, reply exactly UNKNOWN. Never guess.', instructions: [] };
  artifact.runtime.entry = 'default';
  const split = this.model.indexOf('/');
  artifact.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  artifact.policy.tools.allow = [];
  artifact.memory.kb.enabled = false;
  artifact.memory.conversation = { enabled: true };
  assert.equal((await request(this, 'POST', '/api/agents', artifact, 201)).id, this.agentId);
  this.evidence.agent_id = this.agentId;
});

When('I save a private locker code to memory', async function () {
  this.firstCode = `LOCKER-${randomUUID().slice(0, 6).toUpperCase()}`;
  const saved = await request(this, 'POST', '/api/memory', { content: `The user's storage locker code is ${this.firstCode}.`,
    categories: ['personal_fact'], agent_id: this.agentId });
  assert.equal(saved?.status, 'success');
  assert.ok(saved.id, 'The saved memory has an id');
  this.memoryId = saved.id;
  this.evidence.memory = { id: this.memoryId, first_code: this.firstCode, saved };
  this.evidence.memory.record_after_save = await request(this, 'GET', `/api/memory/records/${this.memoryId}`);
});

Then('a fresh conversation recalls that locker code from memory', async function () {
  assert.ok(!QUESTION.includes(this.firstCode));
  const turn = await ask(this, 'recall_first');
  assert.ok(recalledFromMemory(turn, this.firstCode), 'The code came from the memory store');
  assert.ok(turn.answer.includes(this.firstCode), `The answer recalls the saved code: ${turn.answer.slice(0, 300)}`);
});

When('I update the saved locker code and UAR restarts', { timeout: RESTART_TIMEOUT }, async function () {
  this.secondCode = `LOCKER-${randomUUID().slice(0, 6).toUpperCase()}`;
  this.evidence.memory.second_code = this.secondCode;
  this.evidence.memory.updated = await request(this, 'PATCH', `/api/memory/records/${this.memoryId}`,
    { content: `The user's storage locker code is ${this.secondCode}.` });
  await restart(this, 'after_update');
  const record = await request(this, 'GET', `/api/memory/records/${this.memoryId}`);
  this.evidence.memory.record_after_restart = record;
  this.evidence.memory.history = await request(this, 'GET', `/api/memory/records/${this.memoryId}/history`);
  assert.ok(JSON.stringify(record).includes(this.secondCode) && !JSON.stringify(record).includes(this.firstCode),
    'The cold-loaded record holds only the updated value');
});

Then('a fresh conversation recalls only the updated locker code from memory', async function () {
  const turn = await ask(this, 'recall_updated');
  assert.ok(recalledFromMemory(turn, this.secondCode), 'The code came from the memory store');
  assert.ok(turn.answer.includes(this.secondCode), `The answer recalls the updated code: ${turn.answer.slice(0, 300)}`);
  assert.ok(!turn.answer.includes(this.firstCode), 'The superseded code is not recalled');
  assert.deepEqual(mentions(await exported(this), this.firstCode), [], 'No record still holds the superseded code');
});

When('I delete the saved locker code', async function () {
  this.evidence.memory.deleted = await request(this, 'DELETE', `/api/memory/records/${this.memoryId}`, undefined, [200, 204]);
  await request(this, 'GET', `/api/memory/records/${this.memoryId}`, undefined, [404, 410]);
});

Then('a fresh conversation no longer knows the locker code', async function () {
  const turn = await ask(this, 'after_delete');
  assert.ok(!turn.answer.includes(this.secondCode) && !turn.answer.includes(this.firstCode), `Deleted memory was recalled: ${turn.answer.slice(0, 300)}`);
  assert.ok(!turn.tool_results.some(result => result.content.includes(this.secondCode)), 'No memory result carries the deleted code');
  assert.match(turn.answer, /UNKNOWN|don't know|do not know|not (have|know)|no (record|information)/i, 'The answer says the code is unknown');
});

Then('after another restart the deleted memory is still absent', { timeout: RESTART_TIMEOUT }, async function () {
  await restart(this, 'after_delete');
  await request(this, 'GET', `/api/memory/records/${this.memoryId}`, undefined, [404, 410]);
  const records = await exported(this);
  this.evidence.memory.export_after_reload_count = records.length;
  assert.deepEqual(mentions(records, this.secondCode), [], 'No record holds the deleted code after reload');
  const turn = await ask(this, 'after_reload');
  assert.ok(!turn.answer.includes(this.secondCode), `Deleted memory was recalled after reload: ${turn.answer.slice(0, 300)}`);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: RESTART_TIMEOUT }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.memoryId && this.settingsChanged) {
    try { await request(this, 'DELETE', `/api/memory/records/${this.memoryId}`, undefined, [200, 204, 404, 410]); }
    catch (error) { errors.push(`Memory cleanup: ${error}`); }
  }
  if (this.created) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  if (this.settingsChanged) {
    try {
      await restoreSettings(this);
      await restart(this, 'restore');
      this.evidence.restored_settings = {};
      for (const key of Object.keys(SETTINGS)) {
        const setting = await request(this, 'GET', `/api/uar/settings/${key}`);
        this.evidence.restored_settings[key] = { saved: setting.saved, effective: setting.effective, source: setting.source };
        assert.deepEqual(setting.saved, this.originalSettings[key].saved);
      }
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
