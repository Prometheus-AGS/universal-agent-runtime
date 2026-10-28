import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);

async function request(w, method, path, payload, expected = 200, timeout = 120_000) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(timeout),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

async function prompt(w, messages, turn = 'follow-up', onStarted) {
  const payload = { agent_id: w.agentId, model: w.model, session_id: w.session,
    stream: true, stream_mode: 'agui_spec', memory_enabled: false,
    max_completion_tokens: 4096, messages };
  const exchange = { turn, observed: [], method: 'POST', path: '/v1/chat/completions', request: payload, response: '' };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(exchange.path, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = response.status;
  const decoder = new TextDecoder();
  let pending = '';
  for await (const chunk of response.body) {
    const text = decoder.decode(chunk, { stream: true });
    exchange.response += text;
    pending += text;
    let boundary;
    while ((boundary = /\r?\n\r?\n/.exec(pending))) {
      const block = pending.slice(0, boundary.index);
      pending = pending.slice(boundary.index + boundary[0].length);
      const lines = block.split(/\r?\n/);
      const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
      const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
      if (!data || data === '[DONE]' || name?.startsWith('runtime.')) continue;
      const event = JSON.parse(data);
      if (event.type === 'RUN_STARTED') onStarted?.(event.runId);
      if (['RUN_STARTED', 'RUN_FINISHED', 'RUN_ERROR', 'RUN_CANCELLED'].includes(event.type)) {
        exchange.observed.push({ type: event.type, run_id: event.runId, elapsed_ms: performance.now() - w.started });
      }
    }
  }
  exchange.response += decoder.decode();
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
  return { runId: starts[0].runId, events, observed: exchange.observed, terminal: ends[0],
    final: raw.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim() };
}

Given('a disposable agent and a fresh conversation for overlapping turns', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.record = resolve(`.prometheus/verification/concurrent-conversation-restart/${this.session}.json`);
  this.evidence = { scenario: 'CONCURRENT-CONVERSATION-RESTART-01', platform: process.platform,
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
  this.agentId = `disposable-concurrent-conversation-restart-${this.session}`;
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable concurrent conversation assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Remember the facts the user gives you for subsequent questions in this conversation. Acknowledge each fact-bearing turn with only READY, without repeating its facts. Answer later questions using the earlier user facts and the exact JSON fields requested.', instructions: [] };
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

When('I send two fact-bearing turns concurrently in that conversation', { timeout: 420_000 }, async function () {
  this.alpha = `ALPHA-${randomUUID()}`;
  this.beta = `BETA-${randomUUID()}`;
  this.left = randomInt(101, 400);
  this.right = randomInt(501, 900);
  this.expected = { alpha: this.alpha, beta: this.beta, total: this.left + this.right };
  this.evidence.expected = this.expected;
  const messages = [
    `Remember for this conversation: alpha reference ${this.alpha}; alpha quantity ${this.left}. Acknowledge only READY without repeating any facts.`,
    `Remember for this conversation: beta reference ${this.beta}; beta quantity ${this.right}. Acknowledge only READY without repeating any facts.`,
  ];
  const signals = messages.map(() => {
    let resolveStarted;
    const started = new Promise(resolve => { resolveStarted = resolve; });
    return { started, resolveStarted };
  });
  // Install both settlement handlers immediately. Never clean up while a client is active.
  const settled = Promise.allSettled(messages.map((content, index) =>
    prompt(this, [{ role: 'user', content }], index === 0 ? 'alpha' : 'beta', signals[index].resolveStarted)
      .finally(() => signals[index].resolveStarted(null))));
  const ids = await Promise.all(signals.map(signal => signal.started));
  const snapshots = [];
  let observationError;
  try {
    assert.ok(ids.every(Boolean), 'Both runs must start');
    // A remains running across an observation of B running. Terminal runs do
    // not resume under the same ID, so buffered stream delivery cannot fake this.
    for (const id of [ids[0], ids[1], ids[0]]) {
      const run = await request(this, 'GET', `/api/uar/runs/${id}`, undefined, 200, 10_000);
      snapshots.push({ observed_at: new Date().toISOString(), run });
    }
  } catch (error) { observationError = error; }
  this.evidence.concurrent_live_snapshots = snapshots;
  const outcomes = await settled;
  this.evidence.concurrent_outcomes = outcomes.map(outcome => outcome.status === 'fulfilled'
    ? { status: outcome.status, run_id: outcome.value.runId }
    : { status: outcome.status, error: String(outcome.reason) });
  if (observationError) throw observationError;
  for (const { run } of snapshots) {
    assert.equal(run.status, 'running', 'Require genuine concurrent live execution');
    assert.equal(run.conversation_id, this.session);
    assert.equal(run.agent_id, this.agentId);
  }
  assert.ok(outcomes.every(outcome => outcome.status === 'fulfilled'), 'Both concurrent turns must succeed');
  this.turns = outcomes.map(outcome => outcome.value);
  assert.notEqual(this.turns[0].runId, this.turns[1].runId);
  for (const turn of this.turns) assert.equal(turn.final, 'READY');
  const startTimes = this.turns.map(turn => turn.observed.find(event => event.type === 'RUN_STARTED').elapsed_ms);
  const finishTimes = this.turns.map(turn => turn.observed.find(event => event.type === 'RUN_FINISHED').elapsed_ms);
  this.evidence.overlap = { start_times_ms: startTimes, finish_times_ms: finishTimes,
    duration_ms: Math.min(...finishTimes) - Math.max(...startTimes) };
  assert.ok(this.evidence.overlap.duration_ms > 0, 'Both client-observed stream intervals must also overlap');
  this.evidence.before_runs = [];
  for (const turn of this.turns) {
    const run = await request(this, 'GET', `/api/uar/runs/${turn.runId}`);
    assert.equal(run.conversation_id, this.session);
    assert.equal(run.agent_id, this.agentId);
    assert.equal(run.status, 'done');
    this.evidence.before_runs.push(run);
  }
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.before-restart.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
});

When('UAR restarts after both concurrent turns finish', { timeout: 360_000 }, async function () {
  this.evidence.restart = {};
  await restartOwnedUar(this.evidence.restart);
  this.evidence.after_endpoint_binding = await assertOwnedEndpoint(this.base);
  assert.deepEqual(await request(this, 'GET', `/api/agents/${this.agentId}`), this.saved);
});

When('I ask for both sets of facts without replaying either turn', async function () {
  const content = 'What were the alpha and beta references I previously gave you, and what is the sum of their quantities? Return only JSON with alpha, beta, and total.';
  for (const fact of [this.alpha, this.beta, String(this.left), String(this.right)]) assert.ok(!content.includes(fact));
  this.second = await prompt(this, [{ role: 'user', content }]);
  for (const turn of this.turns) assert.notEqual(this.second.runId, turn.runId);
  this.evidence.follow_up_run_id = this.second.runId;
});

Then('the answer retains the facts from both concurrent turns', async function () {
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
