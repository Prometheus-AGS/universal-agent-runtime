import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

async function prompt(w, messages) {
  const payload = { agent_id: w.agentId, model: w.model, session_id: w.session,
    stream: true, stream_mode: 'agui_spec', memory_enabled: false,
    max_completion_tokens: 4096, messages };
  const exchange = { method: 'POST', path: '/v1/chat/completions', request: payload, response: '' };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(exchange.path, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, 200);
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  const events = exchange.response.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
  exchange.events = events;
  assert.ok(!events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), 'Run must complete successfully');
  const starts = events.filter(e => e.type === 'RUN_STARTED');
  const ends = events.filter(e => e.type === 'RUN_FINISHED');
  assert.equal(starts.length, 1);
  assert.equal(ends.length, 1);
  assert.equal(starts[0].runId, ends[0].runId);
  const raw = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  return { runId: starts[0].runId, events, terminal: ends[0],
    final: raw.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim() };
}

Given('a disposable agent and a fresh durable conversation', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.record = resolve(`.prometheus/verification/conversation-restart/${this.session}.json`);
  this.evidence = { scenario: 'CONVERSATION-RESTART-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false,
    client_replays_history: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-conversation-restart-${this.session}`;
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable durable conversation assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Remember the facts the user gives you for subsequent questions in this conversation. Acknowledge the initial facts with only READY, without repeating them. Answer later questions using the earlier user facts and the exact JSON fields requested.', instructions: [] };
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    context_strategy: { type: 'none' }, memory_enabled: false,
    skills: { mode: 'none' }, tools: { mode: 'none' },
    mcp_servers: { mode: 'none' }, knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  await request(this, 'POST', '/api/agents', agent, 201);
  this.saved = await request(this, 'GET', `/api/agents/${this.agentId}`);
  this.evidence.agent_id = this.agentId;
  this.evidence.session_id = this.session;
  this.evidence.model = this.model;
});

When('I give it fresh facts before the restart', { timeout: 300_000 }, async function () {
  this.reference = `CASE-${randomUUID()}`;
  this.left = randomInt(101, 400);
  this.right = randomInt(501, 900);
  this.expected = { reference: this.reference, total: this.left + this.right };
  this.first = await prompt(this, [{ role: 'user', content:
    `Remember for this conversation: reference ${this.reference}; first quantity ${this.left}; second quantity ${this.right}. Acknowledge only READY without repeating any facts.` }]);
  assert.equal(this.first.final, 'READY');
  this.evidence.expected = this.expected;
  this.evidence.initial_run_id = this.first.runId;
  this.evidence.before_run = await request(this, 'GET', `/api/uar/runs/${this.first.runId}`);
  assert.equal(this.evidence.before_run.conversation_id, this.session);
  assert.equal(this.evidence.before_run.status, 'done');
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.before-restart.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
});

When('the owned UAR restarts with its existing store', { timeout: 360_000 }, async function () {
  this.evidence.restart = {};
  await restartOwnedUar(this.evidence.restart);
  this.evidence.after_endpoint_binding = await assertOwnedEndpoint(this.base);
  assert.deepEqual(await request(this, 'GET', `/api/agents/${this.agentId}`), this.saved);
});

When('I ask about those facts in the same conversation without replaying them', async function () {
  const content = 'What reference did I previously give you, and what is the sum of the two quantities? Return only JSON with reference and total.';
  for (const fact of [this.reference, String(this.left), String(this.right)]) assert.ok(!content.includes(fact));
  this.second = await prompt(this, [{ role: 'user', content }]);
  assert.notEqual(this.second.runId, this.first.runId);
  this.evidence.follow_up_run_id = this.second.runId;
});

Then('the restarted UAR answers from the persisted earlier conversation', async function () {
  const run = await request(this, 'GET', `/api/uar/runs/${this.second.runId}`);
  assert.equal(run.conversation_id, this.session);
  assert.equal(run.agent_id, this.agentId);
  assert.equal(run.status, 'done');
  this.evidence.after_run = run;
  let final = this.second.final;
  const fenced = final.match(/^```(?:json)?\s*\n([\s\S]*?)\n```$/);
  if (fenced) final = fenced[1];
  const answer = JSON.parse(final);
  assert.deepEqual(answer, this.expected);
  assert.equal(this.evidence.restart.previous_process_exited, true);
  assert.equal(this.evidence.restart.new_process_ready, true);
  this.evidence.answer = answer;
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.created) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, 204); this.evidence.disposable_agent_deleted = true; }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});
