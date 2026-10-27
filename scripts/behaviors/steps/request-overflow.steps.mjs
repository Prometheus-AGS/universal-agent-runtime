import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(180_000);
const SMALL_WINDOW = 4096;
const OUTPUT_CAP = 512;
const RULES = 320;
const TERMINAL = new Set(['done', 'error', 'cancelled']);
const OVERFLOW_CODES = new Set(['context_overflow', 'protected_history_overflow']);

async function request(w, method, path, payload, expected = 200) {
  const multipart = payload instanceof FormData;
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { ...(!multipart && { 'Content-Type': 'application/json' }), 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : multipart ? payload : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: multipart ? '<file>' : payload, status: response.status, response: text.slice(0, 2000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

Given('a disposable provider route whose model has a small context window', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/request-overflow/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.sessions = [];
  this.evidence = { scenario: 'C46-REQUEST-OVERFLOW-01', platform: process.platform,
    authentication_supplied_by_test: false, credentials_in_requests: false, previous_scenarios_rerun: false, exchanges: [] };
  const liter = await request(this, 'GET', '/api/uar/providers/liter');
  const model = liter.models.find(entry => entry.id === 'MiniMax-M3');
  assert.ok(model);
  this.providerId = `c46small${this.short}`;
  // Keyless loopback route to the same gateway: if UAR ever dispatched, the gateway
  // would refuse it unauthenticated, so the scenario cannot spend or fabricate.
  const { execution_profile: _ignored, ...base } = model;
  await request(this, 'POST', '/api/uar/providers', { id: this.providerId, display_name: 'C46 small-window route',
    base_url: liter.base_url, protocol: liter.protocol, default_model: model.id, enabled: true,
    models: [{ ...base, context_window: SMALL_WINDOW, max_output_tokens: OUTPUT_CAP }] }, [200, 201]);
  this.providerCreated = true;
  const saved = await request(this, 'GET', `/api/uar/providers/${this.providerId}`);
  assert.equal(saved.models[0].context_window, SMALL_WINDOW);
  this.evidence.provider = { id: this.providerId, model: model.id, context_window: SMALL_WINDOW, max_output_tokens: OUTPUT_CAP, credential_configured: saved.credential_configured };
});

Given('a disposable agent with long required instructions, tools and a tiny linked knowledge base', async function () {
  const kb = await request(this, 'POST', '/api/uar/knowledge-bases', { name: `C46 overflow ${this.id}`, description: 'Tiny owned KB',
    config: { chunk_strategy: 'fixed', chunk_size: 512, retrieval_min_score: 0, retrieval_top_k: 4 } }, 201);
  this.kbPath = `/api/uar/knowledge-bases/${encodeURIComponent(kb.id)}`;
  const form = new FormData();
  form.append('file', new Blob(['The Larkspur harbor ledger records 724 moorings.'], { type: 'text/plain' }), 'ledger.txt');
  const uploaded = await request(this, 'POST', `${this.kbPath}/documents`, form, 202);
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline && (await request(this, 'GET', `${this.kbPath}/documents/${uploaded.id}`)).status !== 'indexed') await delay(500);
  this.marker = `RULESET-${this.short.toUpperCase()}`;
  this.instructions = Array.from({ length: RULES }, (_, index) =>
    `Rule ${index + 1}: when answering harbor questions, state the ledger quantity, cite its source, and never omit rule ${index + 1} of ruleset ${this.marker}.`).join('\n');
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-overflow-${this.short}`;
  Object.assign(agent, { id: this.agentId, version: '1.0.0', extensions: {} });
  agent.metadata = { ...agent.metadata, title: 'Disposable overflow agent', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: this.instructions, instructions: [] };
  agent.policy.provider = { default: { provider: this.providerId, model: 'MiniMax-M3' }, fallbacks: [] };
  agent.policy.tools.allow = [];
  agent.memory.kb = { ...agent.memory.kb, enabled: true, knowledge_bases: [kb.id], citation_required: true };
  await request(this, 'POST', '/api/agents', agent, 201);
  this.agentCreated = true;
  Object.assign(this.evidence, { agent_id: this.agentId, knowledge_base_id: kb.id, instruction_rules: RULES, instruction_chars: this.instructions.length });
});

When('I send a request whose required content exceeds that window', async function () {
  const session = randomUUID();
  this.sessions.push(session);
  const created = await request(this, 'POST', '/api/uar/runs', { agent_id: this.agentId, session_id: session,
    input: 'How many moorings does the Larkspur harbor ledger record?' }, [200, 201, 202]);
  this.runId = created.run_id ?? created.runId;
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    this.status = (await request(this, 'GET', `/api/uar/runs/${this.runId}`)).status;
    if (TERMINAL.has(this.status)) break;
    await delay(500);
  }
  this.events = (await request(this, 'GET', `/api/uar/runs/${this.runId}/events`)).events;
  this.calls = await request(this, 'GET', `/api/uar/accounting/calls/${this.runId}`);
  Object.assign(this.evidence, { run_id: this.runId, status: this.status, event_names: [...new Set(this.events.map(e => e.eventName))],
    errors: this.events.filter(e => /error/i.test(e.eventName)).map(e => e.data),
    budget_events: this.events.filter(e => /budget|request/i.test(JSON.stringify(e.data?.artifact_id ?? e.data?.name ?? ''))).map(e => e.data),
    accounting_calls: this.calls });
});

Then('a typed budget refusal is reported before any provider dispatch', function () {
  assert.equal(this.status, 'error');
  const codes = this.evidence.errors.map(error => error.code);
  assert.ok(codes.some(code => OVERFLOW_CODES.has(code)), `Expected a typed overflow refusal, got ${JSON.stringify(this.evidence.errors).slice(0, 400)}`);
  assert.deepEqual(this.calls ?? [], [], 'No model call was admitted');
  assert.ok(!this.events.some(e => e.eventName === 'agui.message.delta'), 'No answer text was produced');
});

Then('no required content was silently dropped to make it fit', function () {
  // Refusal rather than a dispatched, trimmed request is the evidence: no call was
  // admitted, so the instructions were neither truncated nor sent.
  assert.ok(!this.events.some(e => e.eventName === 'agui.done' && e.data?.outcome?.type === 'success'));
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 60_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  for (const path of [this.agentCreated && `/api/agents/${this.agentId}`, this.kbPath, this.providerCreated && `/api/uar/providers/${this.providerId}`].filter(Boolean)) {
    try { await request(this, 'DELETE', path, undefined, [200, 204, 404]); }
    catch (error) { errors.push(`${path}: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});
