import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Before, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { STUB_LLM_MODEL, STUB_LLM_PROMPT, startDedicatedRuntime } from '../../acceptance/fixtures/dedicated-runtime.mjs';

setDefaultTimeout(240_000);

// `local` calls UAR directly on its loopback address with no Authorization
// header. `proxied` goes through the fixture's identity proxy, which supplies a
// different owner. The test itself never sends a credential.
async function request(w, via, method, path, payload, expected = [200]) {
  const base = via === 'local' ? w.runtime.runtime : w.runtime.base;
  const res = await fetch(new URL(path, base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json' },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await res.text();
  w.evidence.exchanges.push({ via, method, path, request: payload, status: res.status, response: text.slice(0, 4000) });
  assert.ok(expected.includes(res.status), `${via} ${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  if (!text) return null;
  // Refusals can carry a plain-text body; only a JSON body is parsed.
  try { return JSON.parse(text); } catch { return { error: text }; }
}

Before(function ({ pickle }) {
  this.scenarioName = pickle.name;
});

Given('a dedicated UAR with local_loopback_owner on, memory enabled and the stub model', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.holder = {};
  this.owned = [];
  this.record = resolve(`.prometheus/verification/local-identity/${randomUUID()}.json`);
  this.evidence = { scenario: this.scenarioName, platform: process.platform,
    authentication_supplied_by_test: false, exchanges: [], fixture: {} };
  // UAR runs with the run directory as its cwd, so the default ./data/memory.db stays inside it.
  this.runtime = await startDedicatedRuntime(this.evidence.fixture, this.holder,
    { backend: 'postgres', label: 'local-identity', stubLlm: true,
      config: { memory: { enabled: true }, security: { local_loopback_owner: true } } });
  // A fresh runtime admits a model only after it is verified, as an operator does in settings.
  const [provider, model] = STUB_LLM_MODEL.split('/');
  const verification = await request(this, 'local', 'POST', `/api/uar/providers/${provider}/test`,
    { model, output_token_cap: 128 });
  assert.equal(verification.ok, true, `Model verification: ${JSON.stringify(verification).slice(0, 300)}`);
});

Given('a dedicated UAR with memory enabled and local_loopback_owner off', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.holder = {};
  this.record = resolve(`.prometheus/verification/local-identity/${randomUUID()}.json`);
  this.evidence = { scenario: this.scenarioName, platform: process.platform,
    authentication_supplied_by_test: false, exchanges: [], fixture: {} };
  // The flag is left at its default (off), as in every existing deployment.
  this.runtime = await startDedicatedRuntime(this.evidence.fixture, this.holder,
    { backend: 'postgres', label: 'local-identity', config: { memory: { enabled: true } } });
});

Then('saving a memory record on the loopback address without credentials is refused', async function () {
  const refusal = await request(this, 'local', 'POST', '/api/memory',
    { content: 'Anonymous callers must not own memory.', categories: ['personal_fact'] }, [401, 403]);
  this.evidence.refusal = refusal;
  this.evidence.result = 'VERIFIED';
});

When('I save a memory record on the loopback address without credentials', async function () {
  this.memoryFact = `LOCAL-OWNER-${randomUUID().slice(0, 8)}`;
  const saved = await request(this, 'local', 'POST', '/api/memory',
    { content: `The local code word is ${this.memoryFact}.`, categories: ['personal_fact'] });
  assert.equal(saved?.status, 'success');
  this.memoryId = saved.id;
});

When('I chat on the loopback address without credentials', async function () {
  this.session = randomUUID();
  const completion = await request(this, 'local', 'POST', '/v1/chat/completions', { model: STUB_LLM_MODEL,
    session_id: this.session, stream: false, messages: [{ role: 'user', content: STUB_LLM_PROMPT }] });
  this.evidence.answer = completion.choices?.[0]?.message?.content;
  assert.match(this.evidence.answer ?? '', /SESSION-OK/);
});

Then('the memory record is readable on the loopback address without credentials', async function () {
  const record = await request(this, 'local', 'GET', `/api/memory/records/${this.memoryId}`);
  assert.ok(JSON.stringify(record).includes(this.memoryFact), 'The local owner reads its own memory');
});

Then('the session is listed on the loopback address without credentials', async function () {
  const sessions = JSON.stringify(await request(this, 'local', 'GET', '/api/uar/sessions'));
  assert.ok(sessions.includes(this.session), 'The local owner lists its own session');
});

Then('the proxied owner sees neither the memory record nor the session', async function () {
  const record = await request(this, 'proxied', 'GET', `/api/memory/records/${this.memoryId}`, undefined, [403, 404]);
  const sessions = JSON.stringify(await request(this, 'proxied', 'GET', '/api/uar/sessions'));
  this.evidence.isolation = { proxied_memory_status: record?.error ?? 'refused', proxied_lists_session: sessions.includes(this.session) };
  assert.ok(!sessions.includes(this.session), 'The proxied owner does not list the local session');
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.runtime?.uarAlive() && this.memoryId) {
    try { await request(this, 'local', 'DELETE', `/api/memory/records/${this.memoryId}`, undefined, [200, 204, 404]); }
    catch (error) { errors.push(`memory cleanup: ${error}`); }
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
